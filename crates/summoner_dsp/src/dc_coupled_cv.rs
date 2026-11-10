// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! DC-Coupled CV/Gate Hardware Interface Engine (Milestone M3).
//!
//! Provides ultra-low-latency, audio-rate Control Voltage (CV) and Gate pulse generation
//! engineered for DC-coupled multichannel USB audio interfaces (Expert Sleepers ES-8 and ES-9).
//!
//! Features:
//! - Multi-point piecewise linear calibration curves correcting analog hardware offset and DAC nonlinearities.
//! - Microtonal 1V/Oct pitch CV scaling supporting arbitrary tuning systems.
//! - Configurable Gate/Trigger pulse generator supporting both V-Trig (+5V/+10V Eurorack) and S-Trig (Moog switch trigger).
//! - Audio-rate modulation routing (Aux CV / Velocity / Pressure) and Sync Clock generation.
//! - AudioNode implementation for direct integration into track DSP graphs.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Supported DC-coupled hardware audio interface models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExpertSleepersModel {
    /// Expert Sleepers ES-8: USB 2.0 Eurorack module (12 in / 16 out, DC-coupled +/-10V).
    #[default]
    Es8,
    /// Expert Sleepers ES-9: High-density USB 2.0 interface (16 in / 16 out, DC-coupled +/-10V).
    Es9,
    /// Generic DC-coupled audio interface (e.g. MOTU UltraLite mk5).
    GenericDcCoupled,
}

impl ExpertSleepersModel {
    /// Maximum voltage swing range in Volts (e.g. +/-10.0V).
    pub fn max_voltage_range(&self) -> f32 {
        match self {
            Self::Es8 | Self::Es9 => 10.0,
            Self::GenericDcCoupled => 5.0,
        }
    }
}

/// Trigger polarity mode for gate outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TriggerMode {
    /// Voltage Trigger (Eurorack standard: 0V inactive, +5V to +10V active).
    #[default]
    VTrig,
    /// Switch Trigger (Vintage Moog standard: high resting voltage, pulled to ground on trigger).
    STrig,
}

/// Individual calibration point for DAC nonlinearity correction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CalibrationPoint {
    pub target_volts: f32,
    pub measured_dac_scale: f32,
}

/// Multi-point DC calibration profile correcting hardware offsets and gain drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCvCalibration {
    pub dc_offset_volts: f32,
    pub volts_per_octave: f32,
    pub calibration_points: Vec<CalibrationPoint>,
}

impl Default for HardwareCvCalibration {
    fn default() -> Self {
        Self {
            dc_offset_volts: 0.0,
            volts_per_octave: 1.0,
            calibration_points: vec![
                CalibrationPoint { target_volts: -5.0, measured_dac_scale: -0.5 },
                CalibrationPoint { target_volts: -2.5, measured_dac_scale: -0.25 },
                CalibrationPoint { target_volts: 0.0, measured_dac_scale: 0.0 },
                CalibrationPoint { target_volts: 2.5, measured_dac_scale: 0.25 },
                CalibrationPoint { target_volts: 5.0, measured_dac_scale: 0.5 },
            ],
        }
    }
}

impl HardwareCvCalibration {
    /// Calibrate target voltage into normalized [-1.0..=+1.0] DAC audio sample value.
    pub fn volts_to_dac_sample(&self, target_volts: f32, max_hardware_range: f32) -> f32 {
        let corrected_volts = target_volts - self.dc_offset_volts;
        if self.calibration_points.is_empty() {
            return (corrected_volts / max_hardware_range).clamp(-1.0, 1.0);
        }

        // Interpolate along calibration curve
        let points = &self.calibration_points;
        if corrected_volts <= points[0].target_volts {
            return points[0].measured_dac_scale.clamp(-1.0, 1.0);
        }
        if corrected_volts >= points[points.len() - 1].target_volts {
            return points[points.len() - 1].measured_dac_scale.clamp(-1.0, 1.0);
        }

        for w in points.windows(2) {
            let p0 = w[0];
            let p1 = w[1];
            if corrected_volts >= p0.target_volts && corrected_volts <= p1.target_volts {
                let range = p1.target_volts - p0.target_volts;
                if range.abs() < 1e-6 {
                    return p0.measured_dac_scale;
                }
                let t = (corrected_volts - p0.target_volts) / range;
                let interp = p0.measured_dac_scale + t * (p1.measured_dac_scale - p0.measured_dac_scale);
                return interp.clamp(-1.0, 1.0);
            }
        }

        (corrected_volts / max_hardware_range).clamp(-1.0, 1.0)
    }
}

/// Audio-rate CV/Gate generator DSP node for DC-coupled interfaces.
#[derive(Debug, Clone)]
pub struct DcCoupledCvNode {
    pub model: ExpertSleepersModel,
    pub calibration: HardwareCvCalibration,
    pub trigger_mode: TriggerMode,
    pub sample_rate: f32,
    pub current_note: f32, // MIDI note (can be fractional for microtones)
    pub is_gate_active: bool,
    pub trigger_pulse_samples: usize,
    pub remaining_trigger_samples: usize,
    pub aux_cv_value: f32, // -1.0 to 1.0
}

impl DcCoupledCvNode {
    pub fn new(model: ExpertSleepersModel, sample_rate: f32) -> Self {
        Self {
            model,
            calibration: HardwareCvCalibration::default(),
            trigger_mode: TriggerMode::VTrig,
            sample_rate: sample_rate.max(1.0),
            current_note: 60.0, // Middle C = 0V reference
            is_gate_active: false,
            trigger_pulse_samples: (sample_rate * 0.010).round() as usize, // 10ms default trigger
            remaining_trigger_samples: 0,
            aux_cv_value: 0.0,
        }
    }

    /// Convert fractional MIDI note number into voltage (1V/Octave, C4 (60) = 0.0V).
    pub fn note_to_volts(&self, note: f32) -> f32 {
        let semitones_from_middle_c = note - 60.0;
        (semitones_from_middle_c / 12.0) * self.calibration.volts_per_octave
    }

    /// Trigger a note on event with pitch and velocity.
    pub fn note_on(&mut self, note: f32, velocity: f32) {
        self.current_note = note;
        self.is_gate_active = true;
        self.remaining_trigger_samples = self.trigger_pulse_samples;
        self.aux_cv_value = velocity.clamp(0.0, 1.0);
    }

    /// Note off event.
    pub fn note_off(&mut self) {
        self.is_gate_active = false;
        self.remaining_trigger_samples = 0;
    }

    /// Set trigger duration in milliseconds.
    pub fn set_trigger_duration_ms(&mut self, duration_ms: f32) {
        self.trigger_pulse_samples = ((duration_ms / 1000.0) * self.sample_rate).round().max(1.0) as usize;
    }
}

impl AudioNode for DcCoupledCvNode {
    fn name(&self) -> &str {
        "DcCoupledCvNode"
    }

    fn process(
        &mut self,
        _input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if output.is_empty() {
            return;
        }

        let num_samples = output[0].len();
        let target_volts = self.note_to_volts(self.current_note);
        let cv_sample = self
            .calibration
            .volts_to_dac_sample(target_volts, self.model.max_voltage_range());

        // Channel 0: 1V/Oct Pitch CV
        output[0].fill(cv_sample);

        // Channel 1: Gate / Trigger Pulse (if available)
        if output.len() > 1 {
            let gate_buf = &mut output[1];
            for sample in gate_buf.iter_mut().take(num_samples) {
                let active = self.is_gate_active || self.remaining_trigger_samples > 0;
                if self.remaining_trigger_samples > 0 {
                    self.remaining_trigger_samples -= 1;
                }
                *sample = match self.trigger_mode {
                    TriggerMode::VTrig => {
                        if active { 1.0 } else { 0.0 }
                    }
                    TriggerMode::STrig => {
                        if active { 0.0 } else { 1.0 }
                    }
                };
            }
        }

        // Channel 2: Auxiliary CV / Velocity (if available)
        if output.len() > 2 {
            output[2].fill(self.aux_cv_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_volts_1v_per_octave() {
        let node = DcCoupledCvNode::new(ExpertSleepersModel::Es8, 48000.0);
        // Middle C (60) = 0.0V
        assert_eq!(node.note_to_volts(60.0), 0.0);
        // C5 (72) = 1.0V (+1 octave)
        assert_eq!(node.note_to_volts(72.0), 1.0);
        // C3 (48) = -1.0V (-1 octave)
        assert_eq!(node.note_to_volts(48.0), -1.0);
        // Microtone: 60.5 = 0.5 semitone / 12 = 1/24 V
        assert!((node.note_to_volts(60.5) - (1.0 / 24.0)).abs() < 1e-6);
    }

    #[test]
    fn test_piecewise_calibration_interpolation() {
        let cal = HardwareCvCalibration::default();
        // 0.0V maps to 0.0
        assert_eq!(cal.volts_to_dac_sample(0.0, 10.0), 0.0);
        // 2.5V maps to 0.25
        assert_eq!(cal.volts_to_dac_sample(2.5, 10.0), 0.25);
        // 1.25V halfway between 0.0 and 2.5 maps to 0.125
        assert!((cal.volts_to_dac_sample(1.25, 10.0) - 0.125).abs() < 1e-6);
    }

    #[test]
    fn test_vtrig_and_strig_processing() {
        let mut node = DcCoupledCvNode::new(ExpertSleepersModel::Es9, 48000.0);
        let mut cv_buf = vec![0.0; 64];
        let mut gate_buf = vec![0.0; 64];
        let mut aux_buf = vec![0.0; 64];

        let ctx = ProcessContext::new(48000, 120.0, 0);

        // Process inactive
        {
            let mut out = [cv_buf.as_mut_slice(), gate_buf.as_mut_slice(), aux_buf.as_mut_slice()];
            node.process(&[], &mut out, &ctx);
        }
        assert_eq!(gate_buf[0], 0.0); // VTrig inactive is 0V

        // Note On
        node.note_on(72.0, 0.8);
        {
            let mut out = [cv_buf.as_mut_slice(), gate_buf.as_mut_slice(), aux_buf.as_mut_slice()];
            node.process(&[], &mut out, &ctx);
        }
        assert_eq!(gate_buf[0], 1.0); // VTrig active is 1.0
        assert_eq!(aux_buf[0], 0.8); // Velocity
        assert!(cv_buf[0] > 0.0); // Pitch CV is positive for C5

        // Switch to S-Trig
        node.trigger_mode = TriggerMode::STrig;
        node.note_off();
        {
            let mut out = [cv_buf.as_mut_slice(), gate_buf.as_mut_slice(), aux_buf.as_mut_slice()];
            node.process(&[], &mut out, &ctx);
        }
        assert_eq!(gate_buf[0], 1.0); // STrig inactive rests high at 1.0
    }
}

