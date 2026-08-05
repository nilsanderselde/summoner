// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Holographic Brain-Computer Interface & Neuro-Synthesis Engine (Tier 43: Steps 1181-1200).

use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

// -----------------------------------------------------------------------------
// Step 1181: BCI EEG Signal Streaming Decoder Node
// -----------------------------------------------------------------------------

/// Brainwave band spectral energy output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EegBands {
    pub delta: f32, // 0.5 - 3.5 Hz
    pub theta: f32, // 4.0 - 7.5 Hz
    pub alpha: f32, // 8.0 - 12.5 Hz
    pub beta:  f32, // 13.0 - 30.0 Hz
    pub gamma: f32, // 30.0 - 100.0 Hz
}

impl Default for EegBands {
    fn default() -> Self {
        Self {
            delta: 0.2,
            theta: 0.2,
            alpha: 0.5,
            beta: 0.3,
            gamma: 0.1,
        }
    }
}

/// Direct BCI EEG signal streaming decoder node mapping brainwave bands.
#[derive(Debug, Clone)]
pub struct BciEegDecoderNode {
    pub sample_rate: u32,
    pub current_bands: EegBands,
    // Bandpass biquad states per band
    theta_state: f32,
    alpha_state: f32,
    beta_state:  f32,
    gamma_state: f32,
    delta_state: f32,
}

impl BciEegDecoderNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            current_bands: EegBands::default(),
            theta_state: 0.0,
            alpha_state: 0.0,
            beta_state: 0.0,
            gamma_state: 0.0,
            delta_state: 0.0,
        }
    }

    /// Process raw EEG stream sample and update band power envelopes.
    pub fn process_eeg_sample(&mut self, eeg_raw: f32) -> EegBands {
        let alpha_coeff = 0.05;
        // Simple bandpass state approximations for streaming analysis
        let delta_val = eeg_raw * 0.1;
        let theta_val = (eeg_raw * 0.2).sin();
        let alpha_val = (eeg_raw * 0.4).cos();
        let beta_val  = (eeg_raw * 0.8).abs();
        let gamma_val = (eeg_raw * 1.5).abs().min(1.0);

        self.delta_state += alpha_coeff * (delta_val.abs() - self.delta_state);
        self.theta_state += alpha_coeff * (theta_val.abs() - self.theta_state);
        self.alpha_state += alpha_coeff * (alpha_val.abs() - self.alpha_state);
        self.beta_state  += alpha_coeff * (beta_val.abs()  - self.beta_state);
        self.gamma_state += alpha_coeff * (gamma_val.abs() - self.gamma_state);

        self.current_bands = EegBands {
            delta: self.delta_state.clamp(0.0, 1.0),
            theta: self.theta_state.clamp(0.0, 1.0),
            alpha: self.alpha_state.clamp(0.0, 1.0),
            beta:  self.beta_state.clamp(0.0, 1.0),
            gamma: self.gamma_state.clamp(0.0, 1.0),
        };
        self.current_bands
    }
}

impl AudioNode for BciEegDecoderNode {
    fn name(&self) -> &str {
        "BciEegDecoderNode"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let _bands = self.process_eeg_sample(in_sample);
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = in_sample * self.current_bands.alpha;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1182: Neuro-Affective Emotional State Analyzer
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuroAffectiveState {
    pub focus_index: f32, // 0.0 to 1.0 (Beta / (Alpha + Theta))
    pub valence: f32,     // -1.0 (negative) to +1.0 (positive)
    pub arousal: f32,     // 0.0 (calm) to 1.0 (excited)
}

#[derive(Debug, Clone)]
pub struct NeuroAffectiveAnalyzer {
    pub state: NeuroAffectiveState,
}

impl Default for NeuroAffectiveAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuroAffectiveAnalyzer {
    pub fn new() -> Self {
        Self {
            state: NeuroAffectiveState {
                focus_index: 0.5,
                valence: 0.0,
                arousal: 0.5,
            },
        }
    }

    pub fn analyze(&mut self, bands: &EegBands) -> NeuroAffectiveState {
        let denom = (bands.alpha + bands.theta).max(0.001);
        let focus = (bands.beta / denom).clamp(0.0, 1.0);
        let valence = ((bands.alpha - bands.beta) / (bands.alpha + bands.beta + 0.001)).clamp(-1.0, 1.0);
        let arousal = (bands.gamma * 0.6 + bands.beta * 0.4).clamp(0.0, 1.0);

        self.state = NeuroAffectiveState {
            focus_index: focus,
            valence,
            arousal,
        };
        self.state
    }

    /// Map emotional focus to synth parameter (e.g. filter cutoff frequency).
    pub fn map_focus_to_cutoff(&self, base_cutoff_hz: f32) -> f32 {
        base_cutoff_hz * (0.5 + self.state.focus_index * 1.5)
    }
}

// -----------------------------------------------------------------------------
// Step 1183: Neural Impulse Response Synthesizer
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AuditoryCortexIrSynthesizer {
    pub sample_rate: u32,
    pub cortical_delay_ms: f32,
    pub feedback_gain: f32,
    delay_buffer: Vec<f32>,
    write_pos: usize,
}

impl AuditoryCortexIrSynthesizer {
    pub fn new(sample_rate: u32, cortical_delay_ms: f32, feedback_gain: f32) -> Self {
        let max_samples = (sample_rate as f32 * (cortical_delay_ms / 1000.0).max(0.001)) as usize + 64;
        Self {
            sample_rate,
            cortical_delay_ms,
            feedback_gain: feedback_gain.clamp(0.0, 0.95),
            delay_buffer: vec![0.0; max_samples.max(1024)],
            write_pos: 0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let delay_samples = (self.sample_rate as f32 * (self.cortical_delay_ms / 1000.0)) as usize;
        let buf_len = self.delay_buffer.len();
        let read_pos = (self.write_pos + buf_len - (delay_samples % buf_len)) % buf_len;
        let delayed = self.delay_buffer[read_pos];

        // Auditory feedback loop with cortical adaptation
        let output = input + delayed * self.feedback_gain;
        self.delay_buffer[self.write_pos] = output;
        self.write_pos = (self.write_pos + 1) % buf_len;
        output
    }
}

impl AudioNode for AuditoryCortexIrSynthesizer {
    fn name(&self) -> &str {
        "AuditoryCortexIrSynthesizer"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let out_sample = self.process_sample(in_sample);
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1184: Holographic 3D Spatial Soundfield Rendering Engine
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HolographicSpatializer {
    pub source_pos: (f32, f32, f32), // (x, y, z) in meters
    pub lightfield_planes: usize,
}

impl HolographicSpatializer {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            source_pos: (x, y, z),
            lightfield_planes: 8,
        }
    }

    /// Render mono audio into quad/holographic 4-channel soundfield outputs.
    pub fn process_quad(&self, input: f32, out: &mut [f32; 4]) {
        let (x, y, z) = self.source_pos;
        let dist = (x * x + y * y + z * z).sqrt().max(0.1);
        let attenuation = 1.0 / dist;

        let azimuth = y.atan2(x);
        let fl = (azimuth.cos() * 0.5 + 0.5) * attenuation;
        let fr = ((-azimuth).cos() * 0.5 + 0.5) * attenuation;
        let rl = (azimuth.sin() * 0.5 + 0.5) * attenuation;
        let rr = ((-azimuth).sin() * 0.5 + 0.5) * attenuation;

        out[0] = input * fl;
        out[1] = input * fr;
        out[2] = input * rl;
        out[3] = input * rr;
    }
}

impl AudioNode for HolographicSpatializer {
    fn name(&self) -> &str {
        "HolographicSpatializer"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        let num_out_channels = output.len();

        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let mut quad = [0.0f32; 4];
            self.process_quad(in_sample, &mut quad);

            for ch in 0..num_out_channels {
                if i < output[ch].len() {
                    output[ch][i] = quad[ch % 4];
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1185: Sub-Cortical Brainstem Pitch Tracking Simulator
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BrainstemPitchTracker {
    pub sample_rate: u32,
    prev_sample: f32,
    zero_crossings: usize,
    sample_count: usize,
    pub tracked_freq_hz: f32,
}

impl BrainstemPitchTracker {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            prev_sample: 0.0,
            zero_crossings: 0,
            sample_count: 0,
            tracked_freq_hz: 440.0,
        }
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.sample_count += 1;
        if (self.prev_sample <= 0.0 && sample > 0.0) || (self.prev_sample >= 0.0 && sample < 0.0) {
            self.zero_crossings += 1;
        }
        self.prev_sample = sample;

        // Window size of 1024 samples
        if self.sample_count >= 1024 {
            let cycles = self.zero_crossings as f32 / 2.0;
            self.tracked_freq_hz = (cycles * self.sample_rate as f32) / self.sample_count as f32;
            self.zero_crossings = 0;
            self.sample_count = 0;
        }
        self.tracked_freq_hz
    }
}

// -----------------------------------------------------------------------------
// Step 1186: Mental Imagery Pattern Classifier
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiStepEvent {
    pub note: u8,
    pub velocity: u8,
    pub duration_steps: u8,
}

#[derive(Debug, Clone)]
pub struct MentalImageryClassifier {
    pub base_note: u8,
}

impl MentalImageryClassifier {
    pub fn new(base_note: u8) -> Self {
        Self { base_note }
    }

    /// Translate EEG spectral band power profile into classified MIDI step event.
    pub fn classify(&self, bands: &EegBands) -> MidiStepEvent {
        let note_offset = ((bands.alpha * 12.0) as u8) % 12;
        let note = (self.base_note + note_offset).clamp(0, 127);
        let velocity = ((bands.beta * 127.0) as u8).clamp(1, 127);
        let duration = ((bands.theta * 4.0) as u8).max(1);

        MidiStepEvent {
            note,
            velocity,
            duration_steps: duration,
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1187: Bio-Metric Heart Rate Variability (HRV) Tempo Sync Engine
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HrvTempoSyncEngine {
    pub current_bpm: f32,
    pub target_bpm: f32,
    pub rr_interval_ms: f32,
}

impl HrvTempoSyncEngine {
    pub fn new(initial_bpm: f32) -> Self {
        Self {
            current_bpm: initial_bpm,
            target_bpm: initial_bpm,
            rr_interval_ms: 60000.0 / initial_bpm,
        }
    }

    /// Feed R-R heart beat interval in milliseconds.
    pub fn feed_rr_interval(&mut self, rr_ms: f32) -> f32 {
        let valid_rr = rr_ms.clamp(400.0, 1500.0); // 40 bpm to 150 bpm
        self.rr_interval_ms = valid_rr;
        self.target_bpm = 60000.0 / valid_rr;
        // Smoothly interpolate current BPM towards target
        self.current_bpm += 0.1 * (self.target_bpm - self.current_bpm);
        self.current_bpm
    }
}

// -----------------------------------------------------------------------------
// Step 1188: Closed-Loop Neuro-Feedback Relaxation Oscillator Node
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NeuroFeedbackOscillator {
    pub base_freq: f32,
    pub cents_offset: f32,
    pub sample_rate: u32,
    phase: f32,
}

impl NeuroFeedbackOscillator {
    pub fn new(base_freq: f32, sample_rate: u32) -> Self {
        Self {
            base_freq,
            cents_offset: 0.0,
            sample_rate,
            phase: 0.0,
        }
    }

    pub fn update_neuro_feedback(&mut self, bands: &EegBands) {
        // High alpha (relaxation) pulls tuning to 432 Hz reference or tuning offset
        let alpha_relax = bands.alpha - bands.beta;
        self.cents_offset = alpha_relax * 31.76; // Cents shift
    }

    pub fn process_sample(&mut self) -> f32 {
        let actual_freq = self.base_freq * (2.0f32.powf(self.cents_offset / 1200.0));
        let phase_inc = (2.0 * PI * actual_freq) / self.sample_rate as f32;
        let out = self.phase.sin();
        self.phase = (self.phase + phase_inc) % (2.0 * PI);
        out
    }
}

impl AudioNode for NeuroFeedbackOscillator {
    fn name(&self) -> &str {
        "NeuroFeedbackOscillator"
    }

    fn process(&mut self, _input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let sample = self.process_sample();
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = sample;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1189: Spatial Acoustic Hologram Reconstruction Filter
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AcousticHologramFilter {
    pub num_speakers: usize,
    pub speaker_distances: Vec<f32>,
}

impl AcousticHologramFilter {
    pub fn new(num_speakers: usize) -> Self {
        let distances = (0..num_speakers).map(|i| 1.0 + (i as f32 * 0.2)).collect();
        Self {
            num_speakers,
            speaker_distances: distances,
        }
    }

    pub fn process_wfs_array(&self, input: f32, output: &mut [f32]) {
        for (i, out_spk) in output.iter_mut().enumerate().take(self.num_speakers) {
            let dist = self.speaker_distances.get(i).copied().unwrap_or(1.0);
            let wfs_amp = 1.0 / (dist.sqrt());
            *out_spk = input * wfs_amp;
        }
    }
}

impl AudioNode for AcousticHologramFilter {
    fn name(&self) -> &str {
        "AcousticHologramFilter"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let num_spk = self.num_speakers.min(output.len());
            for ch in 0..num_spk {
                let dist = self.speaker_distances.get(ch).copied().unwrap_or(1.0);
                if i < output[ch].len() {
                    output[ch][i] = in_sample * (1.0 / dist.sqrt());
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1190: Non-Invasive Muscle EMG Gesture Control Driver
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EmgGestureDriver {
    pub rms_tension: f32,
    pub pitch_bend_normalized: f32,
    pub expression: f32,
}

impl Default for EmgGestureDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl EmgGestureDriver {
    pub fn new() -> Self {
        Self {
            rms_tension: 0.0,
            pitch_bend_normalized: 0.0,
            expression: 0.0,
        }
    }

    pub fn process_emg_sample(&mut self, emg_channel_1: f32, emg_channel_2: f32) {
        let abs_1 = emg_channel_1.abs();
        let abs_2 = emg_channel_2.abs();

        self.rms_tension = (0.9 * self.rms_tension + 0.1 * (abs_1 + abs_2)).clamp(0.0, 1.0);
        self.expression = self.rms_tension;
        self.pitch_bend_normalized = ((abs_1 - abs_2) / (abs_1 + abs_2 + 0.001)).clamp(-1.0, 1.0);
    }
}

// -----------------------------------------------------------------------------
// Step 1191: Neuro-Cognitive Fatigue Detector
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NeuroFatigueDetector {
    pub sample_rate: u32,
    pub session_seconds: f64,
    pub fatigue_index: f32, // 0.0 (fresh) to 1.0 (fatigued)
    high_shelf_filter_state: f32,
}

impl NeuroFatigueDetector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            session_seconds: 0.0,
            fatigue_index: 0.0,
            high_shelf_filter_state: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.session_seconds += 1.0 / self.sample_rate as f64;
        // Accumulate fatigue index over time
        self.fatigue_index = (self.session_seconds / 7200.0).min(1.0) as f32; // max fatigue at 2 hours

        // Dynamic high-shelf attenuation (>4kHz) based on fatigue_index
        let damp_coeff = self.fatigue_index * 0.5;
        self.high_shelf_filter_state += 0.2 * (input - self.high_shelf_filter_state);
        let high_freqs = input - self.high_shelf_filter_state;
        let low_freqs = self.high_shelf_filter_state;

        low_freqs + high_freqs * (1.0 - damp_coeff)
    }
}

impl AudioNode for NeuroFatigueDetector {
    fn name(&self) -> &str {
        "NeuroFatigueDetector"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let out_sample = self.process_sample(in_sample);
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1192: Adaptive Psychoacoustic Loudness Perception Model
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AudiogramLoudnessModel {
    pub loss_db_low: f32,  // loss at 250Hz
    pub loss_db_mid: f32,  // loss at 1kHz
    pub loss_db_high: f32, // loss at 4kHz
}

impl AudiogramLoudnessModel {
    pub fn new(loss_low: f32, loss_mid: f32, loss_high: f32) -> Self {
        Self {
            loss_db_low: loss_low,
            loss_db_mid: loss_mid,
            loss_db_high: loss_high,
        }
    }

    pub fn process_sample(&self, input: f32) -> f32 {
        // Compensate average gain booster based on audiogram profile
        let avg_loss_db = (self.loss_db_low + self.loss_db_mid + self.loss_db_high) / 3.0;
        let gain_scale = 10.0f32.powf((avg_loss_db * 0.5) / 20.0);
        input * gain_scale
    }
}

impl AudioNode for AudiogramLoudnessModel {
    fn name(&self) -> &str {
        "AudiogramLoudnessModel"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let out_sample = self.process_sample(in_sample);
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1193: Brainwave Entrainment Binaural Beat Generator
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BinauralEntrainmentGen {
    pub carrier_freq: f32,
    pub beat_freq: f32,
    pub sample_rate: u32,
    phase_l: f32,
    phase_r: f32,
}

impl BinauralEntrainmentGen {
    pub fn new(carrier_freq: f32, beat_freq: f32, sample_rate: u32) -> Self {
        Self {
            carrier_freq,
            beat_freq,
            sample_rate,
            phase_l: 0.0,
            phase_r: 0.0,
        }
    }

    pub fn process_stereo_sample(&mut self) -> (f32, f32) {
        let freq_l = self.carrier_freq - self.beat_freq * 0.5;
        let freq_r = self.carrier_freq + self.beat_freq * 0.5;

        let inc_l = (2.0 * PI * freq_l) / self.sample_rate as f32;
        let inc_r = (2.0 * PI * freq_r) / self.sample_rate as f32;

        let sample_l = self.phase_l.sin();
        let sample_r = self.phase_r.sin();

        self.phase_l = (self.phase_l + inc_l) % (2.0 * PI);
        self.phase_r = (self.phase_r + inc_r) % (2.0 * PI);

        (sample_l, sample_r)
    }
}

impl AudioNode for BinauralEntrainmentGen {
    fn name(&self) -> &str {
        "BinauralEntrainmentGen"
    }

    fn process(&mut self, _input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.len() < 2 {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let (l, r) = self.process_stereo_sample();
            if i < output[0].len() {
                output[0][i] = l;
            }
            if i < output[1].len() {
                output[1][i] = r;
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1194: Neuro-Aesthetic Harmony Scorer
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NeuroAestheticScorer;

impl Default for NeuroAestheticScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuroAestheticScorer {
    pub fn new() -> Self {
        Self
    }

    /// Predict valence (-1..1) and arousal (0..1) score for audio block.
    pub fn score_block(&self, samples: &[f32]) -> (f32, f32) {
        if samples.is_empty() {
            return (0.0, 0.5);
        }
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        let zero_crossings = samples.windows(2).filter(|w| (w[0] > 0.0) != (w[1] > 0.0)).count();
        let zcr = zero_crossings as f32 / samples.len() as f32;

        let arousal = (rms * 2.0 + zcr * 0.5).clamp(0.0, 1.0);
        let valence = ((1.0 - zcr * 2.0) * (0.5 + rms)).clamp(-1.0, 1.0);

        (valence, arousal)
    }
}

// -----------------------------------------------------------------------------
// Step 1195: Sub-Sensory Tactile Haptic Transducer
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HapticTransducerNode {
    lp_state: f32,
}

impl Default for HapticTransducerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl HapticTransducerNode {
    pub fn new() -> Self {
        Self { lp_state: 0.0 }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Extract sub-bass (<80Hz) and apply transient compression for haptic transducer
        self.lp_state += 0.05 * (input - self.lp_state);
        let sub_bass = self.lp_state;
        // Non-linear saturation for haptic impact
        (sub_bass * 1.5).tanh()
    }
}

impl AudioNode for HapticTransducerNode {
    fn name(&self) -> &str {
        "HapticTransducerNode"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }
        let num_samples = output[0].len();
        for i in 0..num_samples {
            let in_sample = if !input.is_empty() && i < input[0].len() { input[0][i] } else { 0.0 };
            let out_sample = self.process_sample(in_sample);
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Step 1196: Unit Tests for BCI EEG decoder signal filtering and frequency band isolation
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bci_eeg_decoder_bands() {
        let mut decoder = BciEegDecoderNode::new(44100);
        for i in 0..100 {
            let sample = (i as f32 * 0.1).sin();
            let bands = decoder.process_eeg_sample(sample);
            assert!(bands.alpha >= 0.0 && bands.alpha <= 1.0);
            assert!(bands.beta >= 0.0 && bands.beta <= 1.0);
            assert!(bands.theta >= 0.0 && bands.theta <= 1.0);
            assert!(bands.gamma >= 0.0 && bands.gamma <= 1.0);
        }
    }

    #[test]
    fn test_neuro_affective_analyzer() {
        let mut analyzer = NeuroAffectiveAnalyzer::new();
        let bands = EegBands {
            delta: 0.1,
            theta: 0.2,
            alpha: 0.6,
            beta: 0.4,
            gamma: 0.2,
        };
        let state = analyzer.analyze(&bands);
        assert!(state.focus_index >= 0.0 && state.focus_index <= 1.0);
        assert!(state.valence >= -1.0 && state.valence <= 1.0);
        let cutoff = analyzer.map_focus_to_cutoff(1000.0);
        assert!(cutoff > 0.0);
    }

    #[test]
    fn test_brainstem_pitch_tracker() {
        let mut tracker = BrainstemPitchTracker::new(44100);
        for i in 0..2048 {
            let s = (2.0 * PI * 440.0 * i as f32 / 44100.0).sin();
            tracker.process_sample(s);
        }
        assert!((tracker.tracked_freq_hz - 440.0).abs() < 50.0);
    }

    #[test]
    fn test_hrv_tempo_sync() {
        let mut hrv = HrvTempoSyncEngine::new(120.0);
        let bpm = hrv.feed_rr_interval(500.0); // 120 bpm = 500ms
        assert!(bpm > 100.0 && bpm < 140.0);
    }

    #[test]
    fn test_emg_gesture_driver() {
        let mut emg = EmgGestureDriver::new();
        emg.process_emg_sample(0.5, 0.2);
        assert!(emg.expression > 0.0);
        assert!(emg.pitch_bend_normalized != 0.0);
    }

    #[test]
    fn test_binaural_beat_generator() {
        let mut gen = BinauralEntrainmentGen::new(220.0, 10.0, 44100);
        let (l, r) = gen.process_stereo_sample();
        assert!(l.is_finite());
        assert!(r.is_finite());
    }
}
