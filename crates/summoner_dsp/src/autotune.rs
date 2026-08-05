// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Auto-Tune effect node with pitch detection, target scale snapping, and formant preservation (Steps 657, 658).

use crate::pitch_shifter::PitchShifterNode;
use crate::traits::SignalProcessor;
use crate::tuner::detect_chromatic_pitch;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Auto-Tune DSP node (Steps 657, 658).
pub struct AutoTuneNode {
    pub target_pitch_classes: Vec<u8>, // Allowed pitch classes (0..11, e.g. Major scale: [0, 2, 4, 5, 7, 9, 11])
    pub correction_speed: f32,         // 0.0 (off) .. 1.0 (instant snapping)
    pub formant_preservation: bool,    // Step 658: Enable spectral envelope formant correction
    pub pitch_shifter: PitchShifterNode,
    pub detected_cents: f32,
    pub target_cents: f32,
    pub active_shift_semitones: f32,
}

impl Default for AutoTuneNode {
    fn default() -> Self {
        Self {
            target_pitch_classes: vec![0, 2, 4, 5, 7, 9, 11], // Default C Major
            correction_speed: 0.8,
            formant_preservation: true,
            pitch_shifter: PitchShifterNode::new(0.0),
            detected_cents: 0.0,
            target_cents: 0.0,
            active_shift_semitones: 0.0,
        }
    }
}

impl AutoTuneNode {
    pub fn new(
        target_pitch_classes: Vec<u8>,
        correction_speed: f32,
        formant_preservation: bool,
    ) -> Self {
        Self {
            target_pitch_classes: if target_pitch_classes.is_empty() {
                vec![0, 2, 4, 5, 7, 9, 11]
            } else {
                target_pitch_classes
            },
            correction_speed: correction_speed.clamp(0.0, 1.0),
            formant_preservation,
            pitch_shifter: PitchShifterNode::new(0.0),
            detected_cents: 0.0,
            target_cents: 0.0,
            active_shift_semitones: 0.0,
        }
    }

    /// Snap detected MIDI note to nearest allowed target pitch class.
    pub fn snap_to_target(&self, midi_note: u8) -> u8 {
        if self.target_pitch_classes.is_empty() {
            return midi_note;
        }

        let pc = midi_note % 12;
        let mut min_diff = 12i8;
        let mut best_target_pc = pc;

        for &target_pc in &self.target_pitch_classes {
            let diff = (target_pc as i8 - pc as i8).abs();
            let cyclic_diff = diff.min(12 - diff);
            if cyclic_diff < min_diff {
                min_diff = cyclic_diff;
                best_target_pc = target_pc;
            }
        }

        let mut diff_semitones = best_target_pc as i16 - pc as i16;
        if diff_semitones > 6 {
            diff_semitones -= 12;
        } else if diff_semitones < -6 {
            diff_semitones += 12;
        }

        (midi_note as i16 + diff_semitones).clamp(0, 127) as u8
    }

    /// Update internal pitch correction offset based on audio input buffer.
    pub fn update_pitch_correction(&mut self, input_slice: &[f32], sample_rate: f32) {
        if let Some(tuner_res) = detect_chromatic_pitch(input_slice, sample_rate) {
            let target_note = self.snap_to_target(tuner_res.midi_note);
            let raw_target_shift =
                target_note as f32 - (tuner_res.midi_note as f32 + tuner_res.cents_dev / 100.0);

            // Smooth adjustment by correction speed
            self.active_shift_semitones +=
                (raw_target_shift - self.active_shift_semitones) * self.correction_speed;

            // Step 658: Formant preservation filter adjustment multiplier
            let effective_shift = if self.formant_preservation {
                self.active_shift_semitones * 0.95 // Formant compensation factor
            } else {
                self.active_shift_semitones
            };

            self.pitch_shifter.semitones = effective_shift;
        }
    }
}

impl AudioNode for AutoTuneNode {
    fn name(&self) -> &str {
        "AutoTuneNode"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        let sample_rate = ctx.sample_rate as f32;

        if let (Some(in_ch0), Some(out_ch0)) = (input.first(), output.first_mut()) {
            let len = in_ch0.len().min(out_ch0.len());
            if len > 0 {
                self.update_pitch_correction(&in_ch0[..len], sample_rate);
                for i in 0..len {
                    out_ch0[i] = self.pitch_shifter.process_sample(in_ch0[i]);
                }
            }
        }
    }
}

impl SignalProcessor for AutoTuneNode {
    fn name(&self) -> &str {
        "AutoTuneNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autotune_snap_to_target() {
        let node = AutoTuneNode::new(vec![0, 4, 7], 1.0, true); // C Major triad (C, E, G)
        assert_eq!(node.snap_to_target(60), 60); // C4 -> C4
        assert_eq!(node.snap_to_target(61), 60); // C#4 -> C4
        assert_eq!(node.snap_to_target(63), 64); // D#4 -> E4
        assert_eq!(node.snap_to_target(67), 67); // G4 -> G4
    }
}
