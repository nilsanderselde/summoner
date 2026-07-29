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

pub fn load_flac(path: &std::path::Path) -> Result<SampleBuffer, String> {
    let mut reader = claxon::FlacReader::open(path).map_err(|e| e.to_string())?;
    let info = reader.streaminfo();
    let mut data = Vec::new();
    
    let scale = 1.0 / (1i64 << (info.bits_per_sample - 1)) as f32;
    
    for sample in reader.samples() {
        let s = sample.map_err(|e| e.to_string())?;
        data.push(s as f32 * scale);
    }
    
    Ok(SampleBuffer::new(data, info.sample_rate, info.channels as usize))
}

pub fn load_wav(path: &std::path::Path) -> Result<SampleBuffer, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let mut data = Vec::new();
    
    if spec.sample_format == hound::SampleFormat::Float {
        for sample in reader.samples::<f32>() {
            data.push(sample.map_err(|e| e.to_string())?);
        }
    } else {
        let scale = 1.0 / (1i64 << (spec.bits_per_sample - 1)) as f32;
        for sample in reader.samples::<i32>() {
            let s = sample.map_err(|e| e.to_string())?;
            data.push(s as f32 * scale);
        }
    }
    
    Ok(SampleBuffer::new(data, spec.sample_rate, spec.channels as usize))
}

pub fn load_sample_file(path: &std::path::Path) -> Result<SampleBuffer, String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("flac") | Some("FLAC") => load_flac(path),
        Some("wav") | Some("WAV") => load_wav(path),
        _ => Err(format!("Unsupported file format for {}", path.display())),
    }
}

pub fn load_bank_buffers(bank: &mut MultiSampleBank, base_path: &std::path::Path) -> Vec<String> {
    let mut errors = Vec::new();
    for region in &mut bank.regions {
        if region.buffer.is_none() {
            let full_path = base_path.join(&region.sample_path);
            match load_sample_file(&full_path) {
                Ok(buf) => region.buffer = Some(Arc::new(buf)),
                Err(e) => errors.push(e),
            }
        }
    }
    errors
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
    #[test]
    fn test_wav_file_loading() {
        use hound::{WavSpec, WavWriter, SampleFormat};
        
        let file_path = std::env::temp_dir().join("test_load.wav");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let mut writer = WavWriter::create(&file_path, spec).unwrap();
        // Generate a 440 Hz sine wave
        for t in 0..44100 {
            let sample = (t as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin();
            let amplitude = i16::MAX as f32;
            writer.write_sample((sample * amplitude) as i16).unwrap();
        }
        writer.finalize().unwrap();
        
        let buffer = super::load_wav(&file_path).unwrap();
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, 1);
        assert_eq!(buffer.data.len(), 44100);
        
        // Check first few samples
        for t in 0..10 {
            let expected = (t as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin();
            assert!((buffer.data[t] - expected).abs() < 1e-4);
        }
        
        std::fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_load_bank_buffers_fills_regions() {
        use hound::{WavSpec, WavWriter, SampleFormat};
        let temp_dir = std::env::temp_dir();
        let wav_path = temp_dir.join("bank_test.wav");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&wav_path, spec).unwrap();
        writer.write_sample(0i16).unwrap();
        writer.finalize().unwrap();

        let mut bank = MultiSampleBank::new();
        let reg = SampleRegion::new(60, 60, 60, "bank_test.wav");
        bank.add_region(reg);

        let errs = load_bank_buffers(&mut bank, &temp_dir);
        assert!(errs.is_empty(), "load_bank_buffers returned errors: {:?}", errs);
        assert!(bank.regions[0].buffer.is_some(), "Region buffer should be loaded");

        let _ = std::fs::remove_file(wav_path);
    }

    #[test]
    fn test_load_wav_round_trip() {
        use hound::{WavSpec, WavWriter, SampleFormat};
        
        let file_path = std::env::temp_dir().join("test_load_wav_round_trip.wav");
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let mut writer = WavWriter::create(&file_path, spec).unwrap();
        for t in 0..100 {
            writer.write_sample((t * 100) as i16).unwrap();
            writer.write_sample((-t * 100) as i16).unwrap();
        }
        writer.finalize().unwrap();
        
        let buffer = load_wav(&file_path).unwrap();
        assert_eq!(buffer.sample_rate, 48000);
        assert_eq!(buffer.channels, 2);
        assert_eq!(buffer.data.len(), 200);
        
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_flac_round_trip() {
        let flac_path = std::path::Path::new("local/FreePatsGM-SFZ+FLAC-20221026/samples/Applause/Applause.flac");
        if flac_path.exists() {
            let buffer = load_flac(flac_path).unwrap();
            assert!(buffer.sample_rate > 0);
            assert!(buffer.channels > 0);
            assert!(!buffer.data.is_empty());
        }
    }

    #[test]
    fn test_load_sample_file_wav() {
        use hound::{WavSpec, WavWriter, SampleFormat};
        let file_path = std::env::temp_dir().join("test_sample_file.wav");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&file_path, spec).unwrap();
        writer.write_sample(1000i16).unwrap();
        writer.finalize().unwrap();

        let buffer = load_sample_file(&file_path).unwrap();
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, 1);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_sample_file_flac() {
        let flac_path = std::path::Path::new("local/FreePatsGM-SFZ+FLAC-20221026/samples/Applause/Applause.flac");
        if flac_path.exists() {
            let buffer = load_sample_file(flac_path).unwrap();
            assert!(buffer.sample_rate > 0);
            assert!(buffer.channels > 0);
            assert!(!buffer.data.is_empty());
        }
    }
}

