#![allow(clippy::all)]

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;
use std::sync::Arc;

/// Loop mode for sample playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    NoLoop,
    LoopContinuous,
}

/// A shared buffer holding audio samples (e.g., loaded from a WAV/FLAC file).
#[derive(Debug, Clone)]
pub struct SampleBuffer {
    pub data: Vec<Sample>,
    pub sample_rate: u32,
    pub channels: usize,
}

impl SampleBuffer {
    pub fn new(data: Vec<Sample>, sample_rate: u32, channels: usize) -> Self {
        Self {
            data,
            sample_rate,
            channels,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// A mapped region in a multi-sampled instrument.
#[derive(Debug, Clone)]
pub struct SampleRegion {
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub loop_mode: LoopMode,
    pub loop_start: usize,
    pub loop_end: usize,
    pub sample_path: String,
    pub buffer: Option<Arc<SampleBuffer>>,
}

impl SampleRegion {
    pub fn new(lokey: u8, hikey: u8, pitch_keycenter: u8, sample_path: impl Into<String>) -> Self {
        Self {
            lokey,
            hikey,
            pitch_keycenter,
            lovel: 0,
            hivel: 127,
            loop_mode: LoopMode::NoLoop,
            loop_start: 0,
            loop_end: 0,
            sample_path: sample_path.into(),
            buffer: None,
        }
    }

    pub fn matches(&self, note: u8, velocity: u8) -> bool {
        note >= self.lokey && note <= self.hikey && velocity >= self.lovel && velocity <= self.hivel
    }
}

/// Bank of multi-sample regions for SFZ instrument loading.
#[derive(Debug, Clone, Default)]
pub struct MultiSampleBank {
    pub regions: Vec<SampleRegion>,
}

impl MultiSampleBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_region(&mut self, region: SampleRegion) {
        self.regions.push(region);
    }

    pub fn find_region(&self, note: u8, velocity: u8) -> Option<&SampleRegion> {
        self.regions.iter().find(|r| r.matches(note, velocity))
    }
}

/// A Sampler node that plays back a loaded single `SampleBuffer`.
#[derive(Debug, Clone, Default)]
pub struct SamplerNode {
    buffer: Option<Arc<SampleBuffer>>,
    playback_position: f64,
    playback_rate: f64,
    playing: bool,
}

impl SamplerNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_buffer(&mut self, buffer: Arc<SampleBuffer>) {
        self.buffer = Some(buffer);
        self.playback_position = 0.0;
    }

    pub fn trigger(&mut self, rate: f64) {
        self.playback_rate = rate;
        self.playback_position = 0.0;
        self.playing = true;
    }
    
    pub fn stop(&mut self) {
        self.playing = false;
    }
}

impl SignalProcessor for SamplerNode {
    fn name(&self) -> &str {
        "SamplerNode"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let block_size = outputs[0].len();

        if let Some(buf) = &self.buffer {
            if self.playing {
                let channels = outputs.len().min(buf.channels);
                let total_frames = buf.data.len() / buf.channels;
                
                for i in 0..block_size {
                    let pos_floor = self.playback_position.floor() as usize;
                    let frac = (self.playback_position - pos_floor as f64) as f32;
                    
                    if pos_floor >= total_frames {
                        self.playing = false;
                        for ch in 0..outputs.len() {
                            outputs[ch][i..block_size].fill(0.0);
                        }
                        break;
                    }
                    
                    let next_pos = (pos_floor + 1).min(total_frames - 1);
                    for ch in 0..channels {
                        let s0 = buf.data[pos_floor * buf.channels + ch];
                        let s1 = buf.data[next_pos * buf.channels + ch];
                        outputs[ch][i] = s0 + frac * (s1 - s0);
                    }
                    
                    self.playback_position += self.playback_rate;
                }
            } else {
                for out in outputs.iter_mut() {
                    out.fill(0.0);
                }
            }
        } else {
            for out in outputs.iter_mut() {
                out.fill(0.0);
            }
        }
    }
}

/// Multi-region Sampler Node supporting SFZ regions, linear interpolation, pitch scaling, and continuous looping.
#[derive(Debug, Clone, Default)]
pub struct MultiSamplerNode {
    pub bank: MultiSampleBank,
    active_region_idx: Option<usize>,
    playback_position: f64,
    playback_rate: f64,
    playing: bool,
}

impl MultiSamplerNode {
    pub fn new(bank: MultiSampleBank) -> Self {
        Self {
            bank,
            active_region_idx: None,
            playback_position: 0.0,
            playback_rate: 1.0,
            playing: false,
        }
    }

    pub fn trigger_note(&mut self, note: u8, velocity: u8) {
        if let Some(idx) = self.bank.regions.iter().position(|r| r.matches(note, velocity)) {
            let region = &self.bank.regions[idx];
            let semitone_diff = note as f64 - region.pitch_keycenter as f64;
            self.playback_rate = 2.0f64.powf(semitone_diff / 12.0);
            self.active_region_idx = Some(idx);
            self.playback_position = 0.0;
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }
}

impl SignalProcessor for MultiSamplerNode {
    fn name(&self) -> &str {
        "MultiSamplerNode"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let block_size = outputs[0].len();

        if let Some(region_idx) = self.active_region_idx {
            let region = &self.bank.regions[region_idx];
            if let Some(buf) = &region.buffer {
                if self.playing {
                    let channels = outputs.len().min(buf.channels);
                    let total_frames = buf.data.len() / buf.channels;

                    for i in 0..block_size {
                        let pos_floor = self.playback_position.floor() as usize;
                        let frac = (self.playback_position - pos_floor as f64) as f32;

                        if region.loop_mode == LoopMode::LoopContinuous && region.loop_end > region.loop_start && region.loop_end <= total_frames {
                            if pos_floor >= region.loop_end {
                                let loop_len = region.loop_end - region.loop_start;
                                self.playback_position = region.loop_start as f64 + ((self.playback_position - region.loop_start as f64) % loop_len as f64);
                            }
                        } else if pos_floor >= total_frames {
                            self.playing = false;
                            for ch in 0..outputs.len() {
                                outputs[ch][i..block_size].fill(0.0);
                            }
                            break;
                        }

                        let cur_pos = self.playback_position.floor() as usize;
                        let next_pos = (cur_pos + 1).min(total_frames - 1);
                        for ch in 0..channels {
                            let s0 = buf.data[cur_pos * buf.channels + ch];
                            let s1 = buf.data[next_pos * buf.channels + ch];
                            outputs[ch][i] = s0 + frac * (s1 - s0);
                        }

                        self.playback_position += self.playback_rate;
                    }
                } else {
                    for out in outputs.iter_mut() {
                        out.fill(0.0);
                    }
                }
            } else {
                for out in outputs.iter_mut() {
                    out.fill(0.0);
                }
            }
        } else {
            for out in outputs.iter_mut() {
                out.fill(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_sampler_node_pitch_and_region_matching() {
        let mut bank = MultiSampleBank::new();
        let mut reg = SampleRegion::new(60, 72, 60, "samples/Piano/C4.wav");

        let sin_data: Vec<f32> = (0..44100).map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin()).collect();
        reg.buffer = Some(Arc::new(SampleBuffer::new(sin_data, 44100, 1)));

        bank.add_region(reg);
        let mut sampler = MultiSamplerNode::new(bank);

        sampler.trigger_note(60, 100);
        assert!(sampler.playing);
        assert_eq!(sampler.playback_rate, 1.0);

        sampler.trigger_note(72, 100);
        assert!((sampler.playback_rate - 2.0).abs() < 1e-4);
    }
}

