// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! WSOLA (Waveform Similarity Overlap-Add) Elastic Audio Time-Stretching & Pitch-Shift Engine.

use std::f32::consts::PI;

/// A warp marker mapping a source frame in an audio asset to a target timeline frame.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WarpMarker {
    pub source_frame: usize,
    pub target_frame: usize,
}

impl WarpMarker {
    pub fn new(source_frame: usize, target_frame: usize) -> Self {
        Self {
            source_frame,
            target_frame,
        }
    }
}

/// Time-stretching and audio warping algorithm mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WarpMode {
    /// Slices audio at transients; preserves sharp attacks with zero smearing.
    #[default]
    Beats,
    /// Pitch-period autocorrelation WSOLA tuned for monophonic vocals, basses, and solo leads.
    Tones,
    /// Multi-grain overlap-add with randomized LCG phase scatter for ambient textures and clouds.
    Texture,
    /// Dual-band phase-coherent WSOLA splitting low-end fundamental from upper harmonics.
    Complex,
    /// Classic turntable/tape speed-linked pitch+time stretch using 4-point Hermite interpolation.
    RePitch,
    /// Formant-preserving spectral/granular stretch with IDM grain jitter and micro-scatter.
    SpectralIdm,
}

/// Detect transient onset frames in an audio buffer using dynamic spectral energy differences.
pub fn detect_transients(audio: &[f32], sample_rate: u32, sensitivity: f32) -> Vec<usize> {
    if audio.is_empty() {
        return Vec::new();
    }

    let frame_size = ((sample_rate as usize) / 100).max(128); // ~10ms window
    let hop_size = frame_size / 2;
    let min_interval = ((sample_rate as usize) / 20).max(frame_size); // ~50ms min separation

    let mut energies = Vec::new();
    let mut pos = 0;
    while pos + frame_size <= audio.len() {
        let frame = &audio[pos..pos + frame_size];
        let energy: f32 = frame.iter().map(|&s| s * s).sum::<f32>() / (frame_size as f32);
        energies.push(energy.sqrt());
        pos += hop_size;
    }

    if energies.is_empty() {
        return Vec::new();
    }

    let avg_energy: f32 = energies.iter().sum::<f32>() / (energies.len() as f32);
    let threshold = (avg_energy * sensitivity.clamp(0.1, 5.0)).max(0.001);

    let mut transients = Vec::new();
    let mut last_transient_frame = 0;

    for i in 1..energies.len() {
        let diff = energies[i] - energies[i - 1];
        if diff > threshold {
            let frame_idx = i * hop_size;
            if transients.is_empty() || frame_idx >= last_transient_frame + min_interval {
                transients.push(frame_idx);
                last_transient_frame = frame_idx;
            }
        }
    }

    transients
}

/// WSOLA (Waveform Similarity Overlap-Add) time-stretching engine.
///
/// Provides pitch-preserving time stretching without phase artifacts or comb filtering
/// by finding maximal cross-correlation alignment between consecutive synthesis grains.
#[derive(Debug, Clone)]
pub struct WsolaTimeStretcher {
    pub sample_rate: u32,
    pub window_size: usize,
    pub hop_size: usize,
    pub max_delta: usize,
    pub stretch_ratio: f32, // Time stretch factor (0.25 to 4.0). 2.0 = twice as long (half speed)
    window: Vec<f32>,
    target_grain: Vec<f32>,
    output_accumulator: Vec<f32>,
    weight_accumulator: Vec<f32>,
}

impl WsolaTimeStretcher {
    pub fn new(sample_rate: u32, window_size: usize, hop_size: usize) -> Self {
        let win_sz = window_size.max(64);
        let hop_sz = hop_size.max(16).min(win_sz / 2);
        let max_delta = hop_sz;

        // Symmetric Hanning window
        let mut window = vec![0.0f32; win_sz];
        for (i, val) in window.iter_mut().enumerate() {
            *val = 0.5 * (1.0 - (2.0 * PI * i as f32 / win_sz as f32).cos());
        }

        Self {
            sample_rate,
            window_size: win_sz,
            hop_size: hop_sz,
            max_delta,
            stretch_ratio: 1.0,
            window,
            target_grain: vec![0.0; win_sz],
            output_accumulator: Vec::new(),
            weight_accumulator: Vec::new(),
        }
    }

    pub fn set_stretch_ratio(&mut self, ratio: f32) {
        self.stretch_ratio = ratio.clamp(0.25, 4.0);
    }

    /// Finds the best alignment offset delta in [-max_delta, +max_delta] via normalized cross-correlation.
    fn find_best_alignment(&self, input: &[f32], nominal_pos: isize) -> isize {
        let in_len = input.len() as isize;
        let win_sz = self.window_size as isize;
        let max_d = self.max_delta as isize;

        let mut best_delta: isize = 0;
        let mut max_corr = -1e9f32;

        for delta in -max_d..=max_d {
            let start = nominal_pos + delta;
            if start < 0 || start + win_sz > in_len {
                continue;
            }

            let mut corr = 0.0f32;
            let mut energy = 0.0f32;

            for k in 0..self.window_size {
                let s = input[(start + k as isize) as usize];
                let t = self.target_grain[k];
                corr += s * t;
                energy += s * s;
            }

            let norm_corr = if energy > 1e-6 {
                corr / (energy.sqrt() + 1e-6)
            } else {
                corr
            };

            if norm_corr > max_corr {
                max_corr = norm_corr;
                best_delta = delta;
            }
        }

        best_delta
    }

    /// Process a mono slice of audio using WSOLA time-stretching.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> usize {
        if input.is_empty() || output.is_empty() {
            return 0;
        }

        let out_len = output.len();
        let in_len = input.len();
        let win_sz = self.window_size;
        let s_s = self.hop_size; // Synthesis hop size
        let s_a = (s_s as f32 / self.stretch_ratio).round().max(1.0) as usize; // Analysis hop size

        if self.output_accumulator.len() < out_len + win_sz {
            self.output_accumulator.resize(out_len + win_sz, 0.0);
            self.weight_accumulator.resize(out_len + win_sz, 0.0);
        }
        self.output_accumulator.fill(0.0);
        self.weight_accumulator.fill(0.0);

        let mut out_pos = 0usize;
        let mut in_pos = 0isize;
        let mut prev_best_in_pos = 0isize;

        // Initialize target grain with first window of input
        for (k, grain_val) in self.target_grain.iter_mut().enumerate().take(win_sz) {
            if k < in_len {
                *grain_val = input[k] * self.window[k];
            } else {
                *grain_val = 0.0;
            }
        }

        while out_pos + win_sz <= out_len + win_sz && (in_pos as usize) < in_len {
            // Find best matching grain near nominal in_pos
            let nominal_pos = prev_best_in_pos + s_a as isize;
            let delta = self.find_best_alignment(input, nominal_pos);
            let best_in_pos =
                (nominal_pos + delta).clamp(0, (in_len.saturating_sub(win_sz)) as isize);
            prev_best_in_pos = best_in_pos;

            // Overlap-add windowed grain
            for k in 0..win_sz {
                let in_idx = (best_in_pos as usize + k).min(in_len - 1);
                let w = self.window[k];
                let sample = input[in_idx] * w;
                let acc_idx = out_pos + k;
                if acc_idx < self.output_accumulator.len() {
                    self.output_accumulator[acc_idx] += sample;
                    self.weight_accumulator[acc_idx] += w * w;
                }
            }

            // Prepare target grain for next iteration from synthesized natural continuation
            let next_target_nominal = best_in_pos + s_s as isize;
            for k in 0..win_sz {
                let idx =
                    (next_target_nominal + k as isize).clamp(0, (in_len - 1) as isize) as usize;
                self.target_grain[k] = input[idx];
            }

            out_pos += s_s;
            in_pos += s_a as isize;
        }

        // Normalize by accumulated window weights
        let written = out_len.min(out_pos);
        for (i, out_val) in output.iter_mut().enumerate().take(written) {
            let weight = self.weight_accumulator[i];
            if weight > 1e-4 {
                *out_val = (self.output_accumulator[i] / weight).clamp(-1.0, 1.0);
            } else {
                *out_val = self.output_accumulator[i].clamp(-1.0, 1.0);
            }
        }

        written
    }

    /// Process a stereo pair of audio slices using WSOLA time-stretching.
    pub fn process_stereo(
        &mut self,
        in_l: &[f32],
        in_r: &[f32],
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) -> usize {
        let frames_l = self.process(in_l, out_l);
        let frames_r = self.process(in_r, out_r);
        frames_l.min(frames_r)
    }
}

/// Detected musical tempo, key, and transient structure of an audio signal (REC-3).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioAnalysisResult {
    pub estimated_bpm: f64,
    pub downbeat_frame: usize,
    pub root_note: u8,
    pub is_minor: bool,
    pub key_name: String,
    pub transient_frames: Vec<usize>,
}

const NOTE_NAMES_SHARP: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Estimate tempo (BPM), downbeat position, musical key, and transient markers from mono audio (REC-3).
pub fn detect_bpm_and_key(audio: &[f32], sample_rate: u32) -> AudioAnalysisResult {
    let transients = detect_transients(audio, sample_rate, 0.75);
    let downbeat_frame = transients.first().copied().unwrap_or(0);
    if audio.is_empty() || sample_rate == 0 {
        return AudioAnalysisResult {
            estimated_bpm: 120.0,
            downbeat_frame: 0,
            root_note: 0,
            is_minor: false,
            key_name: "C Major".to_string(),
            transient_frames: transients,
        };
    }

    // 1. Onset novelty autocorrelation BPM detector (60..=200 BPM)
    let hop = ((sample_rate as usize) / 100).max(64);
    let win = hop * 2;
    let mut energies = Vec::new();
    let mut pos = 0usize;
    while pos + win <= audio.len() {
        let mut sq = 0.0f64;
        for &s in &audio[pos..pos + win] {
            sq += (s as f64) * (s as f64);
        }
        energies.push((sq / win as f64).sqrt());
        pos += hop;
    }

    let mut novelty = Vec::with_capacity(energies.len());
    novelty.push(0.0f64);
    for w in energies.windows(2) {
        novelty.push((w[1] - w[0]).max(0.0));
    }

    let mut best_bpm = 120.0f64;
    let mut best_score = 0.0f64;
    if novelty.len() > 8 {
        let frames_per_sec = sample_rate as f64 / hop as f64;
        for half_bpm in 120..=400 {
            let bpm = half_bpm as f64 * 0.5;
            let lag = ((60.0 / bpm) * frames_per_sec).round() as usize;
            if lag == 0 || lag >= novelty.len() {
                continue;
            }
            let mut corr = 0.0f64;
            for i in 0..(novelty.len() - lag) {
                corr += novelty[i] * novelty[i + lag];
            }
            // Gently prefer the 90..155 BPM perceptual octave window
            let octave_prior = if (90.0..=155.0).contains(&bpm) {
                1.12
            } else {
                1.0
            };
            let score = corr * octave_prior;
            if score > best_score {
                best_score = score;
                best_bpm = bpm;
            }
        }
    }

    // 2. 12-bin chromagram across MIDI 36..=84 via Goertzel resonance + Krumhansl-Schmuckler fit
    let analyze_len = audio.len().min((sample_rate as usize) * 4);
    let slice = &audio[..analyze_len];
    let mut chroma = [0.0f64; 12];
    for midi in 36u8..=84u8 {
        let freq = 440.0 * 2.0f64.powf((midi as f64 - 69.0) / 12.0);
        if freq * 2.0 >= sample_rate as f64 {
            continue;
        }
        let w = 2.0 * std::f64::consts::PI * freq / sample_rate as f64;
        let coeff = 2.0 * w.cos();
        let (mut q1, mut q2) = (0.0f64, 0.0f64);
        for &x in slice {
            let q0 = coeff * q1 - q2 + x as f64;
            q2 = q1;
            q1 = q0;
        }
        let mag2 = (q1 * q1 + q2 * q2 - q1 * q2 * coeff).max(0.0);
        chroma[(midi % 12) as usize] += mag2.sqrt();
    }

    const MAJOR_PROFILE: [f64; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
    ];
    const MINOR_PROFILE: [f64; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
    ];

    let mut best_root = 0u8;
    let mut best_minor = false;
    let mut best_key_score = f64::NEG_INFINITY;
    let chroma_sum: f64 = chroma.iter().sum();
    if chroma_sum > 1e-6 {
        let chroma_mean = chroma_sum / 12.0;
        for root in 0..12usize {
            for (is_minor, profile) in [(false, &MAJOR_PROFILE), (true, &MINOR_PROFILE)] {
                let p_mean = profile.iter().sum::<f64>() / 12.0;
                let mut num = 0.0f64;
                let mut d_c = 0.0f64;
                let mut d_p = 0.0f64;
                for i in 0..12 {
                    let cv = chroma[(root + i) % 12] - chroma_mean;
                    let pv = profile[i] - p_mean;
                    num += cv * pv;
                    d_c += cv * cv;
                    d_p += pv * pv;
                }
                let denom = (d_c * d_p).sqrt();
                let r = if denom > 1e-12 { num / denom } else { 0.0 };
                if r > best_key_score {
                    best_key_score = r;
                    best_root = root as u8;
                    best_minor = is_minor;
                }
            }
        }
    }

    let key_name = format!(
        "{} {}",
        NOTE_NAMES_SHARP[best_root as usize],
        if best_minor { "Minor" } else { "Major" }
    );

    AudioAnalysisResult {
        estimated_bpm: best_bpm,
        downbeat_frame,
        root_note: best_root,
        is_minor: best_minor,
        key_name,
        transient_frames: transients,
    }
}

#[inline]
fn hermite4(s: &[f32], pos: f64) -> f32 {
    if s.is_empty() {
        return 0.0;
    }
    let idx = pos.floor() as isize;
    let t = (pos - idx as f64) as f32;
    let get = |k: isize| -> f32 { s[k.clamp(0, s.len() as isize - 1) as usize] };
    let xm1 = get(idx - 1);
    let x0 = get(idx);
    let x1 = get(idx + 1);
    let x2 = get(idx + 2);
    let c0 = x0;
    let c1 = 0.5 * (x1 - xm1);
    let c2 = xm1 - 2.5 * x0 + 2.0 * x1 - 0.5 * x2;
    let c3 = 0.5 * (x2 - xm1) + 1.5 * (x0 - x1);
    (((c3 * t + c2) * t + c1) * t + c0).clamp(-1.0, 1.0)
}

/// Non-destructive elastic audio warping engine using piecewise dynamic time-stretching & pitch shifting.
#[derive(Debug, Clone)]
pub struct ElasticWarpEngine {
    pub sample_rate: u32,
    pub mode: WarpMode,
    pub markers: Vec<WarpMarker>,
    pub pitch_semitones: f32,
    pub formant_semitones: f32,
    pub grain_ms: f32,
    pub flux: f32,
    stretcher: WsolaTimeStretcher,
}

impl ElasticWarpEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            mode: WarpMode::Complex,
            markers: Vec::new(),
            pitch_semitones: 0.0,
            formant_semitones: 0.0,
            grain_ms: 35.0,
            flux: 0.0,
            stretcher: WsolaTimeStretcher::new(sample_rate, 1024, 256),
        }
    }

    pub fn with_mode(mut self, mode: WarpMode) -> Self {
        self.mode = mode;
        self.stretcher = match mode {
            WarpMode::Beats => WsolaTimeStretcher::new(self.sample_rate, 512, 128),
            WarpMode::Tones => WsolaTimeStretcher::new(self.sample_rate, 1024, 256),
            WarpMode::Texture => WsolaTimeStretcher::new(self.sample_rate, 2048, 512),
            WarpMode::Complex | WarpMode::RePitch | WarpMode::SpectralIdm => {
                WsolaTimeStretcher::new(self.sample_rate, 1024, 256)
            }
        };
        self
    }

    pub fn with_pitch_semitones(mut self, semitones: f32) -> Self {
        self.pitch_semitones = semitones.clamp(-36.0, 36.0);
        self
    }

    pub fn with_formant_semitones(mut self, semitones: f32) -> Self {
        self.formant_semitones = semitones.clamp(-24.0, 24.0);
        self
    }

    pub fn with_grain_ms(mut self, grain_ms: f32) -> Self {
        self.grain_ms = grain_ms.clamp(5.0, 250.0);
        self
    }

    pub fn with_flux(mut self, flux: f32) -> Self {
        self.flux = flux.clamp(0.0, 1.0);
        self
    }

    pub fn add_marker(&mut self, source_frame: usize, target_frame: usize) {
        self.markers
            .push(WarpMarker::new(source_frame, target_frame));
        self.markers.sort_by_key(|m| m.target_frame);
    }

    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    /// Automatically detect transients and generate tempo-mapped warp markers.
    pub fn auto_detect_warp_markers(&mut self, audio: &[f32], source_bpm: f64, target_bpm: f64) {
        let transients = detect_transients(audio, self.sample_rate, 1.0);
        let tempo_ratio = (target_bpm / source_bpm).max(0.1);
        self.markers.clear();

        for src_frame in transients {
            let tgt_frame = (src_frame as f64 * tempo_ratio).round() as usize;
            self.add_marker(src_frame, tgt_frame);
        }
    }

    fn estimate_pitch_period(&self, input: &[f32]) -> usize {
        let sr = self.sample_rate.max(8000) as usize;
        let min_lag = (sr / 1000).max(16);
        let max_lag = (sr / 60).min(input.len() / 2);
        if max_lag <= min_lag || input.len() < max_lag * 2 {
            return 256;
        }
        let check_len = (input.len() - max_lag).min(2048);
        let mut best_lag = min_lag;
        let mut best_corr = f32::NEG_INFINITY;
        for lag in min_lag..=max_lag {
            let mut num = 0.0f32;
            let mut den = 0.0f32;
            for i in 0..check_len {
                num += input[i] * input[i + lag];
                den += input[i + lag] * input[i + lag];
            }
            let score = if den > 1e-8 { num / den.sqrt() } else { 0.0 };
            if score > best_corr {
                best_corr = score;
                best_lag = lag;
            }
        }
        best_lag
    }

    fn run_granular_stretch(
        &self,
        input: &[f32],
        output_len: usize,
        grain_len: usize,
        hop_out: usize,
        jitter_amount: f32,
        inner_pitch_ratio: f32,
    ) -> Vec<f32> {
        let mut acc = vec![0.0f32; output_len + grain_len];
        let mut wsum = vec![0.0f32; output_len + grain_len];
        let mut win = vec![0.0f32; grain_len];
        for (i, w) in win.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (2.0 * PI * i as f32 / grain_len as f32).cos());
        }
        let mut rng = 0x9E37_79B9u32;
        let time_step = input.len() as f64 / output_len.max(1) as f64;
        let mut out_pos = 0usize;
        while out_pos < output_len {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let rand_unit = ((rng >> 9) as f32 / (1u32 << 23) as f32) - 0.5;
            let jitter_samples = rand_unit * jitter_amount * (grain_len as f32 * 0.5);
            let center_in = (out_pos as f64 * time_step + jitter_samples as f64)
                .clamp(0.0, (input.len().saturating_sub(1)) as f64);
            for (k, &w) in win.iter().enumerate() {
                let sample_pos =
                    center_in + (k as f64 - grain_len as f64 * 0.5) * inner_pitch_ratio as f64;
                let s = hermite4(input, sample_pos);
                let dst = out_pos + k;
                acc[dst] += s * w;
                wsum[dst] += w * w;
            }
            out_pos += hop_out.max(1);
        }
        let mut out = vec![0.0f32; output_len];
        for i in 0..output_len {
            let w = wsum[i];
            out[i] = if w > 1e-5 {
                (acc[i] / w).clamp(-1.0, 1.0)
            } else {
                0.0
            };
        }
        out
    }

    fn warp_segment_by_mode(&self, input: &[f32], tgt_len: usize) -> Vec<f32> {
        if input.is_empty() || tgt_len == 0 {
            return vec![0.0; tgt_len];
        }
        match self.mode {
            WarpMode::RePitch => {
                let step = input.len() as f64 / tgt_len as f64;
                let mut out = vec![0.0f32; tgt_len];
                for (i, sample) in out.iter_mut().enumerate() {
                    *sample = hermite4(input, i as f64 * step);
                }
                out
            }
            WarpMode::Beats => {
                let transients = detect_transients(input, self.sample_rate, 0.85);
                if transients.is_empty() || input.len() < 256 {
                    let mut out = vec![0.0f32; tgt_len];
                    let mut s = WsolaTimeStretcher::new(self.sample_rate, 512, 128);
                    s.set_stretch_ratio(tgt_len as f32 / input.len() as f32);
                    s.process(input, &mut out);
                    return out;
                }
                let attack_len = ((self.sample_rate as usize) / 50).clamp(64, 1024);
                let stretch = tgt_len as f64 / input.len() as f64;
                let mut out = vec![0.0f32; tgt_len];
                let mut s = WsolaTimeStretcher::new(self.sample_rate, 512, 128);
                s.set_stretch_ratio(stretch as f32);
                s.process(input, &mut out);

                // Overlay 1:1 sharp transient attacks with 64-sample cosine fade into stretched tail
                let xfade = 64usize.min(attack_len / 2);
                for &tr in &transients {
                    let tgt_tr = ((tr as f64) * stretch).round() as usize;
                    let max_copy = attack_len
                        .min(input.len().saturating_sub(tr))
                        .min(out.len().saturating_sub(tgt_tr));
                    for k in 0..max_copy {
                        let tail_blend = if k + xfade >= max_copy && xfade > 0 {
                            let u = (max_copy - k) as f32 / xfade as f32;
                            0.5 * (1.0 - (PI * u).cos())
                        } else {
                            1.0
                        };
                        out[tgt_tr + k] =
                            out[tgt_tr + k] * (1.0 - tail_blend) + input[tr + k] * tail_blend;
                    }
                }
                out
            }
            WarpMode::Tones => {
                let period = self.estimate_pitch_period(input);
                let win = (period * 4).clamp(256, 2048);
                let hop = (win / 4).clamp(64, 512);
                let mut out = vec![0.0f32; tgt_len];
                let mut s = WsolaTimeStretcher::new(self.sample_rate, win, hop);
                s.set_stretch_ratio(tgt_len as f32 / input.len() as f32);
                s.process(input, &mut out);
                out
            }
            WarpMode::Texture => {
                let grain_len = ((self.grain_ms * self.sample_rate as f32 / 1000.0).round()
                    as usize)
                    .clamp(256, 4096);
                let jitter = if self.flux > 0.0 { self.flux } else { 0.35 };
                self.run_granular_stretch(input, tgt_len, grain_len, grain_len / 4, jitter, 1.0)
            }
            WarpMode::Complex => {
                // Split into low band (< 220Hz) for tight phase coherence and high band for crispness
                let alpha = (2.0 * PI * 220.0 / self.sample_rate.max(1) as f32).clamp(0.005, 0.5);
                let mut low = vec![0.0f32; input.len()];
                let mut high = vec![0.0f32; input.len()];
                let mut lp_state = 0.0f32;
                for (i, &x) in input.iter().enumerate() {
                    lp_state += alpha * (x - lp_state);
                    low[i] = lp_state;
                    high[i] = x - lp_state;
                }
                let ratio = tgt_len as f32 / input.len() as f32;
                let mut out_low = vec![0.0f32; tgt_len];
                let mut out_high = vec![0.0f32; tgt_len];
                let mut s_low = WsolaTimeStretcher::new(self.sample_rate, 2048, 512);
                s_low.set_stretch_ratio(ratio);
                s_low.process(&low, &mut out_low);

                let mut s_high = WsolaTimeStretcher::new(self.sample_rate, 768, 192);
                s_high.set_stretch_ratio(ratio);
                s_high.process(&high, &mut out_high);

                out_low
                    .into_iter()
                    .zip(out_high)
                    .map(|(l, h)| (l + h).clamp(-1.0, 1.0))
                    .collect()
            }
            WarpMode::SpectralIdm => {
                let grain_len = ((self.grain_ms * self.sample_rate as f32 / 1000.0).round()
                    as usize)
                    .clamp(128, 4096);
                let inner_pitch = 2.0f32.powf(-self.formant_semitones / 12.0);
                self.run_granular_stretch(
                    input,
                    tgt_len,
                    grain_len,
                    (grain_len / 4).max(32),
                    self.flux * 0.6,
                    inner_pitch,
                )
            }
        }
    }

    fn stretch_and_pitch_segment(&self, input: &[f32], output_len: usize) -> Vec<f32> {
        if input.is_empty() || output_len == 0 {
            return vec![0.0; output_len];
        }
        if self.pitch_semitones.abs() <= 1e-3 || self.mode == WarpMode::RePitch {
            return self.warp_segment_by_mode(input, output_len);
        }

        let pitch_factor = 2.0f64.powf(self.pitch_semitones as f64 / 12.0);
        let inter_len = ((output_len as f64) * pitch_factor)
            .round()
            .clamp(32.0, (output_len.max(64) * 8) as f64) as usize;
        let intermediate = self.warp_segment_by_mode(input, inter_len);
        let mut resampled = summoner_core::resample::resample_mono(
            &intermediate,
            intermediate.len().max(1) as u32,
            output_len.max(1) as u32,
        );
        if resampled.len() < output_len {
            let last = *resampled.last().unwrap_or(&0.0);
            resampled.resize(output_len, last);
        } else {
            resampled.truncate(output_len);
        }
        resampled
    }

    /// Warp a mono audio signal according to warp markers into the given output length.
    pub fn warp_audio(&self, input: &[f32], output_len: usize) -> Vec<f32> {
        if input.is_empty() || output_len == 0 {
            return vec![0.0; output_len];
        }

        if self.markers.is_empty() {
            return self.stretch_and_pitch_segment(input, output_len);
        }

        // Piecewise stretch across marker segments
        let mut sorted_markers = self.markers.clone();
        if sorted_markers
            .first()
            .is_none_or(|m| m.source_frame > 0 || m.target_frame > 0)
        {
            sorted_markers.insert(0, WarpMarker::new(0, 0));
        }
        if sorted_markers
            .last()
            .is_none_or(|m| m.target_frame < output_len)
        {
            sorted_markers.push(WarpMarker::new(input.len(), output_len));
        }

        let mut output = vec![0.0f32; output_len];

        for window in sorted_markers.windows(2) {
            let m0 = window[0];
            let m1 = window[1];

            let src_start = m0.source_frame.min(input.len());
            let src_end = m1.source_frame.min(input.len());
            let tgt_start = m0.target_frame.min(output_len);
            let tgt_end = m1.target_frame.min(output_len);

            if src_end <= src_start || tgt_end <= tgt_start {
                continue;
            }

            let src_slice = &input[src_start..src_end];
            let tgt_len = tgt_end - tgt_start;
            let seg_out = self.stretch_and_pitch_segment(src_slice, tgt_len);
            let copy_len = seg_out.len().min(tgt_len).min(output_len - tgt_start);
            output[tgt_start..tgt_start + copy_len].copy_from_slice(&seg_out[..copy_len]);
        }

        output
    }

    /// Warp a stereo pair of audio signals non-destructively according to warp markers.
    pub fn warp_stereo(
        &self,
        in_l: &[f32],
        in_r: &[f32],
        output_len: usize,
    ) -> (Vec<f32>, Vec<f32>) {
        let out_l = self.warp_audio(in_l, output_len);
        let out_r = self.warp_audio(in_r, output_len);
        (out_l, out_r)
    }
}

const RT_WARP_RING_LEN: usize = 16384;
const RT_WARP_RING_MASK: usize = RT_WARP_RING_LEN - 1;
const RT_HANN_LEN: usize = 4096;

/// Zero-allocation real-time granular pitch/time warp processor node (`AudioNode`, `rt_safety`-verified).
#[derive(Debug, Clone)]
pub struct RealtimeWarpNode {
    pub sample_rate: u32,
    pub pitch_semitones: f32,
    pub stretch_ratio: f32,
    pub grain_ms: f32,
    pub flux: f32,
    pub mix: f32,
    pub mode: WarpMode,
    ring_l: Vec<f32>,
    ring_r: Vec<f32>,
    hann_lut: Vec<f32>,
    write_pos: usize,
    read_pos_a: f64,
    read_pos_b: f64,
    grain_phase: f64,
    rng: u32,
}

impl RealtimeWarpNode {
    pub fn new(sample_rate: u32) -> Self {
        let mut hann_lut = vec![0.0f32; RT_HANN_LEN];
        for (i, w) in hann_lut.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (2.0 * PI * i as f32 / RT_HANN_LEN as f32).cos());
        }
        Self {
            sample_rate: sample_rate.max(1),
            pitch_semitones: 0.0,
            stretch_ratio: 1.0,
            grain_ms: 40.0,
            flux: 0.0,
            mix: 1.0,
            mode: WarpMode::Complex,
            ring_l: vec![0.0f32; RT_WARP_RING_LEN],
            ring_r: vec![0.0f32; RT_WARP_RING_LEN],
            hann_lut,
            write_pos: 0,
            read_pos_a: 0.0,
            read_pos_b: 0.0,
            grain_phase: 0.0,
            rng: 0x1234_5678,
        }
    }

    #[inline]
    fn sample_ring(ring: &[f32], pos: f64) -> f32 {
        let i0 = (pos.floor() as isize).rem_euclid(RT_WARP_RING_LEN as isize) as usize;
        let i1 = (i0 + 1) & RT_WARP_RING_MASK;
        let frac = (pos - pos.floor()) as f32;
        ring[i0] + frac * (ring[i1] - ring[i0])
    }

    #[inline]
    fn hann_at(&self, norm_phase: f64) -> f32 {
        let idx =
            ((norm_phase.clamp(0.0, 0.9999) * RT_HANN_LEN as f64) as usize).min(RT_HANN_LEN - 1);
        self.hann_lut[idx]
    }
}

impl summoner_core::node::AudioNode for RealtimeWarpNode {
    fn name(&self) -> &str {
        "RealtimeWarpNode"
    }

    fn process(
        &mut self,
        input: &[&[f32]],
        output: &mut [&mut [f32]],
        _ctx: &summoner_core::node::ProcessContext,
    ) {
        if output.is_empty() {
            return;
        }
        let frames = output.iter().take(2).map(|ch| ch.len()).min().unwrap_or(0);
        let in_l = input.first().copied().unwrap_or(&[]);
        let in_r = input.get(1).copied().unwrap_or(in_l);
        let grain_samples = ((self.grain_ms.clamp(10.0, 200.0) * self.sample_rate as f32 / 1000.0)
            as f64)
            .clamp(128.0, 4096.0);
        let speed = 2.0f64.powf(self.pitch_semitones.clamp(-24.0, 24.0) as f64 / 12.0)
            * (self.stretch_ratio.clamp(0.25, 4.0) as f64);
        let mix = self.mix.clamp(0.0, 1.0);
        let (first_ch, rest_ch) = output.split_at_mut(1);
        let out_l_buf = &mut first_ch[0][..frames];
        let mut out_r_buf = rest_ch.first_mut().map(|ch| &mut ch[..frames]);

        for (i, out_l_slot) in out_l_buf.iter_mut().enumerate() {
            let dry_l = in_l.get(i).copied().unwrap_or(0.0);
            let dry_r = in_r.get(i).copied().unwrap_or(dry_l);
            self.ring_l[self.write_pos] = dry_l;
            self.ring_r[self.write_pos] = dry_r;

            let phase_a = self.grain_phase;
            let phase_b = (self.grain_phase + 0.5) % 1.0;
            let win_a = self.hann_at(phase_a);
            let win_b = self.hann_at(phase_b);
            let w_sum = (win_a + win_b).max(1e-5);

            let wet_l = (Self::sample_ring(&self.ring_l, self.read_pos_a) * win_a
                + Self::sample_ring(&self.ring_l, self.read_pos_b) * win_b)
                / w_sum;
            let wet_r = (Self::sample_ring(&self.ring_r, self.read_pos_a) * win_a
                + Self::sample_ring(&self.ring_r, self.read_pos_b) * win_b)
                / w_sum;

            *out_l_slot = dry_l * (1.0 - mix) + wet_l * mix;
            if let Some(ref mut out_r_slice) = out_r_buf {
                out_r_slice[i] = dry_r * (1.0 - mix) + wet_r * mix;
            }

            self.write_pos = (self.write_pos + 1) & RT_WARP_RING_MASK;
            let prev_phase = self.grain_phase;
            self.grain_phase = (self.grain_phase + 1.0 / grain_samples) % 1.0;
            self.read_pos_a += speed;
            self.read_pos_b += speed;

            // Reset grain read heads smoothly at grain boundaries without heap allocs
            if self.grain_phase < prev_phase {
                self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let scatter = if self.flux > 0.0 {
                    (((self.rng >> 9) as f64 / (1u32 << 23) as f64) - 0.5)
                        * (self.flux as f64)
                        * grain_samples
                        * 0.5
                } else {
                    0.0
                };
                self.read_pos_a = (self.write_pos as f64) - grain_samples * 0.5 + scatter;
            } else if prev_phase < 0.5 && self.grain_phase >= 0.5 {
                self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let scatter = if self.flux > 0.0 {
                    (((self.rng >> 9) as f64 / (1u32 << 23) as f64) - 0.5)
                        * (self.flux as f64)
                        * grain_samples
                        * 0.5
                } else {
                    0.0
                };
                self.read_pos_b = (self.write_pos as f64) - grain_samples * 0.5 + scatter;
            }
        }
    }

    fn handle_event(
        &mut self,
        event: &summoner_core::node::NodeEvent,
        _ctx: &summoner_core::node::ProcessContext,
    ) {
        if let summoner_core::node::NodeEvent::SetParam { id, value } = event {
            match *id {
                0 => self.pitch_semitones = value.clamp(-24.0, 24.0),
                1 => self.stretch_ratio = value.clamp(0.25, 4.0),
                2 => self.grain_ms = value.clamp(10.0, 200.0),
                3 => self.flux = value.clamp(0.0, 1.0),
                4 => self.mix = value.clamp(0.0, 1.0),
                5 => {
                    self.mode = match value.round() as i32 {
                        0 => WarpMode::Beats,
                        1 => WarpMode::Tones,
                        2 => WarpMode::Texture,
                        3 => WarpMode::Complex,
                        4 => WarpMode::RePitch,
                        _ => WarpMode::SpectralIdm,
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsola_time_stretching_mono_and_stereo() {
        let sample_rate = 44100;
        let mut stretcher = WsolaTimeStretcher::new(sample_rate, 512, 128);
        stretcher.set_stretch_ratio(1.5); // 1.5x longer

        let input_len = 2048;
        let input: Vec<f32> = (0..input_len)
            .map(|i| (i as f32 * 440.0 * 2.0 * PI / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 3072];

        let written = stretcher.process(&input, &mut output);
        assert!(written > 0);
        assert!(output.iter().any(|&s| s.abs() > 0.0));
        assert!(output.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_elastic_warp_engine_piecewise_markers() {
        let sample_rate = 44100;
        let mut warp = ElasticWarpEngine::new(sample_rate);
        warp.add_marker(1000, 1500); // 1.5x stretch first segment
        warp.add_marker(2000, 2200); // 0.7x compress second segment

        let input: Vec<f32> = (0..2000)
            .map(|i| (i as f32 * 220.0 * 2.0 * PI / 44100.0).sin())
            .collect();

        let warped = warp.warp_audio(&input, 2500);
        assert_eq!(warped.len(), 2500);
        assert!(warped.iter().any(|&s| s.abs() > 0.0));
        assert!(warped.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_transient_detection_and_warp_modes() {
        let sample_rate = 44100;
        let mut audio = vec![0.0f32; 44100]; // 1 second of audio

        // Create sharp transient hits at 0.2s and 0.6s
        let hit1 = 44100 / 5; // 8820
        let hit2 = 44100 * 3 / 5; // 26460
        for i in 0..1000 {
            audio[hit1 + i] = ((1000 - i) as f32 / 1000.0) * (i as f32 * 0.5).sin();
            audio[hit2 + i] = ((1000 - i) as f32 / 1000.0) * (i as f32 * 0.8).sin();
        }

        let transients = detect_transients(&audio, sample_rate, 0.5);
        assert!(!transients.is_empty(), "Transients should be detected");

        let mut warp = ElasticWarpEngine::new(sample_rate).with_mode(WarpMode::Beats);
        warp.auto_detect_warp_markers(&audio, 120.0, 140.0);
        assert_eq!(warp.markers.len(), transients.len());

        let warped = warp.warp_audio(&audio, 40000);
        assert_eq!(warped.len(), 40000);
    }

    #[test]
    fn test_all_six_warp_modes_and_independent_pitch_shift() {
        let sr = 44100u32;
        let input: Vec<f32> = (0..8192)
            .map(|i| {
                let t = i as f32 / sr as f32;
                0.5 * (2.0 * PI * 220.0 * t).sin() + 0.3 * (2.0 * PI * 660.0 * t).sin()
            })
            .collect();

        for mode in [
            WarpMode::Beats,
            WarpMode::Tones,
            WarpMode::Texture,
            WarpMode::Complex,
            WarpMode::RePitch,
            WarpMode::SpectralIdm,
        ] {
            let warp = ElasticWarpEngine::new(sr)
                .with_mode(mode)
                .with_pitch_semitones(5.0)
                .with_formant_semitones(-2.0)
                .with_flux(0.25);
            let out = warp.warp_audio(&input, 10240);
            assert_eq!(out.len(), 10240, "mode {:?} wrong len", mode);
            assert!(
                out.iter().all(|s| s.is_finite()),
                "mode {:?} non-finite",
                mode
            );
            assert!(
                out.iter().any(|&s| s.abs() > 0.01),
                "mode {:?} produced silence",
                mode
            );
        }
    }

    #[test]
    fn test_detect_bpm_and_key_identifies_pulse_and_triad() {
        let sr = 44100u32;
        // 3 seconds containing 120 BPM clicks (every 0.5s) + A Minor triad (A3=220, C4=261.63, E4=329.63)
        let len = sr as usize * 3;
        let mut audio = vec![0.0f32; len];
        for (i, sample) in audio.iter_mut().enumerate() {
            let t = i as f32 / sr as f32;
            let chord = 0.25
                * ((2.0 * PI * 220.0 * t).sin()
                    + (2.0 * PI * 261.63 * t).sin()
                    + (2.0 * PI * 329.63 * t).sin());
            let beat_phase = (t * 2.0) % 1.0; // 120 BPM = 2 beats/sec
            let pulse = if beat_phase < 0.04 {
                0.7 * (1.0 - beat_phase / 0.04)
            } else {
                0.0
            };
            *sample = (chord + pulse).clamp(-1.0, 1.0);
        }
        let analysis = detect_bpm_and_key(&audio, sr);
        assert!(
            (analysis.estimated_bpm - 120.0).abs() <= 4.0,
            "expected ~120 BPM, got {}",
            analysis.estimated_bpm
        );
        assert_eq!(analysis.root_note, 9); // A
        assert!(analysis.is_minor);
        assert_eq!(analysis.key_name, "A Minor");
        assert!(!analysis.transient_frames.is_empty());
    }

    #[test]
    fn test_realtime_warp_node_processes_stereo_without_panicking() {
        use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};
        let mut node = RealtimeWarpNode::new(44100);
        let ctx = ProcessContext::new(44100, 120.0, 0);
        node.handle_event(&NodeEvent::SetParam { id: 0, value: 7.0 }, &ctx);
        node.handle_event(&NodeEvent::SetParam { id: 4, value: 1.0 }, &ctx);

        let in_l = vec![0.4f32; 512];
        let in_r = vec![-0.4f32; 512];
        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        node.process(&[&in_l, &in_r], &mut [&mut out_l, &mut out_r], &ctx);
        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().any(|&s| s.abs() > 0.05));
    }
}
