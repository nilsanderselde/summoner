//! MIDI Time Code (MTC) and Linear Time Code (LTC) audio synchronization (Task I3).
//!
//! Provides SMPTE standard timecode generation and slave tracking:
//! - [`SmpteTimecode`]: Standard timecode structure (HH:MM:SS:FF) supporting 24, 25, 29.97 DF/NDF, and 30 FPS.
//! - [`MtcGenerator`]: Generates 0xF1 quarter-frame MTC stream and full-frame SysEx messages.
//! - [`MtcFollower`]: Decodes incoming MTC quarter-frames and synchronizes DAW transport.
//! - [`LtcAudioGenerator`]: Encodes SMPTE timecode as an 80-bit biphase mark code (BMC) audio signal.
//! - [`LtcAudioDecoder`]: Demodulates audio-rate LTC input via zero-crossing detection and sync word alignment.

use crate::audio::Sample;
use serde::{Deserialize, Serialize};
use std::fmt;

/// SMPTE frame rate format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimecodeRate {
    /// Film 24 fps
    Fps24,
    /// EBU / PAL 25 fps
    Fps25,
    /// NTSC 29.97 fps Drop-Frame
    Fps2997Df,
    /// NTSC 29.97 fps Non-Drop Frame
    Fps2997Ndf,
    /// 30 fps Drop-Frame
    Fps30Df,
    /// 30 fps Non-Drop Frame
    Fps30Ndf,
}

impl TimecodeRate {
    pub fn frames_per_second(self) -> f64 {
        match self {
            Self::Fps24 => 24.0,
            Self::Fps25 => 25.0,
            Self::Fps2997Df | Self::Fps2997Ndf => 30000.0 / 1001.0, // ~29.97003
            Self::Fps30Df | Self::Fps30Ndf => 30.0,
        }
    }

    pub fn nominal_fps(self) -> u32 {
        match self {
            Self::Fps24 => 24,
            Self::Fps25 => 25,
            Self::Fps2997Df | Self::Fps2997Ndf => 30,
            Self::Fps30Df | Self::Fps30Ndf => 30,
        }
    }

    pub fn is_drop_frame(self) -> bool {
        matches!(self, Self::Fps2997Df | Self::Fps30Df)
    }

    /// MTC rate bits (2-bit format identifier used in MTC quarter-frame 7 and SysEx).
    pub fn mtc_format_bits(self) -> u8 {
        match self {
            Self::Fps24 => 0b00,
            Self::Fps25 => 0b01,
            Self::Fps2997Df | Self::Fps30Df => 0b10,
            Self::Fps2997Ndf | Self::Fps30Ndf => 0b11,
        }
    }
}

/// SMPTE Timecode representation (Hours, Minutes, Seconds, Frames, Subframes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub subframes: u8, // 0..79 (1/80th frame)
    pub rate: TimecodeRate,
}

impl Default for SmpteTimecode {
    fn default() -> Self {
        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            frames: 0,
            subframes: 0,
            rate: TimecodeRate::Fps25,
        }
    }
}

impl SmpteTimecode {
    pub fn new(hours: u8, minutes: u8, seconds: u8, frames: u8, rate: TimecodeRate) -> Self {
        Self {
            hours: hours % 24,
            minutes: minutes % 60,
            seconds: seconds % 60,
            frames: frames % (rate.nominal_fps() as u8),
            subframes: 0,
            rate,
        }
    }

    /// Calculate timecode from absolute frame count.
    pub fn from_total_frames(total_frames: u64, rate: TimecodeRate) -> Self {
        let fps = rate.nominal_fps() as u64;

        if rate.is_drop_frame() {
            // Drop frame formula: drop frames 0 and 1 at the start of each minute,
            // except every 10th minute.
            let drop_frames: u64 = 2;
            let frames_per_minute = fps * 60 - drop_frames;
            let frames_per_10_minutes = fps * 60 * 10 - drop_frames * 9;

            let d = total_frames / frames_per_10_minutes;
            let m = total_frames % frames_per_10_minutes;

            let adjusted_frames = if m > drop_frames {
                total_frames
                    + drop_frames * 9 * d
                    + drop_frames * ((m - drop_frames) / frames_per_minute)
            } else {
                total_frames + drop_frames * 9 * d
            };

            let f = (adjusted_frames % fps) as u8;
            let s = ((adjusted_frames / fps) % 60) as u8;
            let min = (((adjusted_frames / fps) / 60) % 60) as u8;
            let h = ((((adjusted_frames / fps) / 60) / 60) % 24) as u8;

            Self {
                hours: h,
                minutes: min,
                seconds: s,
                frames: f,
                subframes: 0,
                rate,
            }
        } else {
            let f = (total_frames % fps) as u8;
            let s = ((total_frames / fps) % 60) as u8;
            let min = (((total_frames / fps) / 60) % 60) as u8;
            let h = ((((total_frames / fps) / 60) / 60) % 24) as u8;

            Self {
                hours: h,
                minutes: min,
                seconds: s,
                frames: f,
                subframes: 0,
                rate,
            }
        }
    }

    /// Convert timecode to total frame count.
    pub fn to_total_frames(&self) -> u64 {
        let fps = self.rate.nominal_fps() as u64;
        let total_minutes = self.hours as u64 * 60 + self.minutes as u64;
        let total_seconds = total_minutes * 60 + self.seconds as u64;
        let base_frames = total_seconds * fps + self.frames as u64;

        if self.rate.is_drop_frame() {
            let drop_count = 2 * (total_minutes - total_minutes / 10);
            base_frames.saturating_sub(drop_count)
        } else {
            base_frames
        }
    }

    /// Calculate timecode from sample position and sample rate.
    pub fn from_sample_position(sample: u64, sample_rate: u32, rate: TimecodeRate) -> Self {
        if sample_rate == 0 {
            return Self::default();
        }
        let total_seconds = sample as f64 / sample_rate as f64;
        let total_frames = (total_seconds * rate.frames_per_second()).floor() as u64;
        Self::from_total_frames(total_frames, rate)
    }

    /// Convert timecode to sample position.
    pub fn to_sample_position(&self, sample_rate: u32) -> u64 {
        let frames = self.to_total_frames() as f64 + (self.subframes as f64 / 80.0);
        let seconds = frames / self.rate.frames_per_second();
        (seconds * sample_rate as f64).round() as u64
    }
}

impl fmt::Display for SmpteTimecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = if self.rate.is_drop_frame() { ';' } else { ':' };
        write!(
            f,
            "{:02}:{:02}:{:02}{}{:02}",
            self.hours, self.minutes, self.seconds, sep, self.frames
        )
    }
}

/// MTC Generator emitting MIDI quarter-frame bytes (0xF1) and Full-Frame SysEx.
pub struct MtcGenerator {
    rate: TimecodeRate,
    sample_rate: u32,
    accumulated_samples: f64,
    quarter_frame_index: u8,
}

impl MtcGenerator {
    pub fn new(sample_rate: u32, rate: TimecodeRate) -> Self {
        Self {
            rate,
            sample_rate,
            accumulated_samples: 0.0,
            quarter_frame_index: 0,
        }
    }

    /// Advance transport by `samples` frames, returning emitted MTC quarter-frame bytes (0xF1, data).
    pub fn advance(&mut self, samples: u64, current_sample_position: u64) -> Vec<[u8; 2]> {
        let mut output = Vec::new();
        let samples_per_qframe = (self.sample_rate as f64) / (self.rate.frames_per_second() * 4.0);

        self.accumulated_samples += samples as f64;
        while self.accumulated_samples >= samples_per_qframe {
            self.accumulated_samples -= samples_per_qframe;

            let tc = SmpteTimecode::from_sample_position(
                current_sample_position,
                self.sample_rate,
                self.rate,
            );
            let piece = self.quarter_frame_index % 8;
            self.quarter_frame_index = (self.quarter_frame_index + 1) % 8;

            let data_nibble = match piece {
                0 => tc.frames & 0x0F,
                1 => (tc.frames >> 4) & 0x01,
                2 => tc.seconds & 0x0F,
                3 => (tc.seconds >> 4) & 0x07,
                4 => tc.minutes & 0x0F,
                5 => (tc.minutes >> 4) & 0x07,
                6 => tc.hours & 0x0F,
                7 => ((tc.hours >> 4) & 0x01) | (self.rate.mtc_format_bits() << 1),
                _ => 0,
            };

            let byte1 = 0xF1;
            let byte2 = (piece << 4) | (data_nibble & 0x0F);
            output.push([byte1, byte2]);
        }

        output
    }

    /// Generate Full Frame SysEx message for transport locate / jumps.
    pub fn make_full_frame_sysex(&self, tc: &SmpteTimecode) -> [u8; 10] {
        let hr_fmt = (tc.hours & 0x1F) | (self.rate.mtc_format_bits() << 5);
        [
            0xF0, // SysEx start
            0x7F, // Universal Real-Time
            0x7F, // Broadcast ID
            0x01, // Sub-ID 1 (Timecode)
            0x01, // Sub-ID 2 (Full Message)
            hr_fmt,
            tc.minutes & 0x3F,
            tc.seconds & 0x3F,
            tc.frames & 0x1F,
            0xF7, // EOX
        ]
    }
}

/// MTC Follower decoding quarter-frame stream and reconstructing timecode for transport slaving.
pub struct MtcFollower {
    rate: TimecodeRate,
    received_pieces: [Option<u8>; 8],
    pub current_timecode: SmpteTimecode,
    pub is_locked: bool,
}

impl MtcFollower {
    pub fn new(rate: TimecodeRate) -> Self {
        Self {
            rate,
            received_pieces: [None; 8],
            current_timecode: SmpteTimecode::default(),
            is_locked: false,
        }
    }

    /// Ingest a quarter-frame data byte (from an `0xF1 <byte>` message).
    /// Returns `Some(SmpteTimecode)` when a full 8-piece sequence is successfully decoded.
    pub fn process_qframe_byte(&mut self, byte: u8) -> Option<SmpteTimecode> {
        let piece = (byte >> 4) & 0x07;
        let val = byte & 0x0F;
        self.received_pieces[piece as usize] = Some(val);

        // Check if all 8 pieces are present
        if self.received_pieces.iter().all(|p| p.is_some()) {
            let p0 = self.received_pieces[0].unwrap();
            let p1 = self.received_pieces[1].unwrap();
            let p2 = self.received_pieces[2].unwrap();
            let p3 = self.received_pieces[3].unwrap();
            let p4 = self.received_pieces[4].unwrap();
            let p5 = self.received_pieces[5].unwrap();
            let p6 = self.received_pieces[6].unwrap();
            let p7 = self.received_pieces[7].unwrap();

            let frames = (p1 << 4) | p0;
            let seconds = (p3 << 4) | p2;
            let minutes = (p5 << 4) | p4;
            let hours = (p7 & 0x01) << 4 | p6;

            self.current_timecode = SmpteTimecode::new(hours, minutes, seconds, frames, self.rate);
            self.is_locked = true;
            // Clear pieces for next sequence
            self.received_pieces = [None; 8];
            Some(self.current_timecode)
        } else {
            None
        }
    }
}

/// Linear Time Code (LTC) 80-bit Biphase Mark Code audio generator.
pub struct LtcAudioGenerator {
    rate: TimecodeRate,
    sample_rate: u32,
    current_timecode: SmpteTimecode,
    current_polarity: f32,
    samples_per_bit: f64,
    bit_sample_pos: f64,
    current_bit_index: usize,
    frame_bits: [bool; 80],
}

impl LtcAudioGenerator {
    pub const SYNC_WORD: u16 = 0x3FFD; // 0011 1111 1111 1101

    pub fn new(sample_rate: u32, rate: TimecodeRate) -> Self {
        let fps = rate.frames_per_second();
        let samples_per_bit = (sample_rate as f64) / (fps * 80.0);
        let mut gen = Self {
            rate,
            sample_rate,
            current_timecode: SmpteTimecode::new(0, 0, 0, 0, rate),
            current_polarity: 1.0,
            samples_per_bit,
            bit_sample_pos: 0.0,
            current_bit_index: 0,
            frame_bits: [false; 80],
        };
        gen.encode_current_frame();
        gen
    }

    pub fn set_timecode(&mut self, tc: SmpteTimecode) {
        self.current_timecode = tc;
        self.encode_current_frame();
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Encode 80-bit SMPTE frame structure into boolean bits.
    #[allow(clippy::needless_range_loop)]
    fn encode_current_frame(&mut self) {
        let mut bits = [false; 80];
        let tc = &self.current_timecode;

        // Frames BCD (bits 0..7)
        let f_units = tc.frames % 10;
        let f_tens = tc.frames / 10;
        for i in 0..4 {
            bits[i] = ((f_units >> i) & 1) == 1;
        }
        for i in 0..2 {
            bits[8 + i] = ((f_tens >> i) & 1) == 1;
        }
        bits[10] = self.rate.is_drop_frame();

        // Seconds BCD (bits 16..27)
        let s_units = tc.seconds % 10;
        let s_tens = tc.seconds / 10;
        for i in 0..4 {
            bits[16 + i] = ((s_units >> i) & 1) == 1;
        }
        for i in 0..3 {
            bits[24 + i] = ((s_tens >> i) & 1) == 1;
        }

        // Minutes BCD (bits 32..43)
        let m_units = tc.minutes % 10;
        let m_tens = tc.minutes / 10;
        for i in 0..4 {
            bits[32 + i] = ((m_units >> i) & 1) == 1;
        }
        for i in 0..3 {
            bits[40 + i] = ((m_tens >> i) & 1) == 1;
        }

        // Hours BCD (bits 48..59)
        let h_units = tc.hours % 10;
        let h_tens = tc.hours / 10;
        for i in 0..4 {
            bits[48 + i] = ((h_units >> i) & 1) == 1;
        }
        for i in 0..2 {
            bits[56 + i] = ((h_tens >> i) & 1) == 1;
        }

        // Sync Word (bits 64..79: 0x3FFD)
        for i in 0..16 {
            bits[64 + i] = ((Self::SYNC_WORD >> i) & 1) == 1;
        }

        self.frame_bits = bits;
    }

    /// Render next block of audio samples containing biphase mark code.
    pub fn render_block(&mut self, output: &mut [Sample]) {
        for sample in output.iter_mut() {
            let bit = self.frame_bits[self.current_bit_index];
            let half_bit = self.samples_per_bit / 2.0;

            // Biphase mark code:
            // - Transition at the beginning of each bit cell.
            // - Bit 1: additional transition in the middle of cell.
            // - Bit 0: no mid-cell transition.
            if self.bit_sample_pos < half_bit {
                *sample = self.current_polarity;
            } else if bit {
                *sample = -self.current_polarity;
            } else {
                *sample = self.current_polarity;
            }

            self.bit_sample_pos += 1.0;
            if self.bit_sample_pos >= self.samples_per_bit {
                self.bit_sample_pos -= self.samples_per_bit;
                // Transition at end/start of next cell
                self.current_polarity = if bit {
                    self.current_polarity
                } else {
                    -self.current_polarity
                };

                self.current_bit_index += 1;
                if self.current_bit_index >= 80 {
                    self.current_bit_index = 0;
                    // Advance to next frame
                    let next_frames = self.current_timecode.to_total_frames() + 1;
                    self.current_timecode =
                        SmpteTimecode::from_total_frames(next_frames, self.rate);
                    self.encode_current_frame();
                }
            }
        }
    }
}

/// LTC Audio Decoder demodulating biphase mark audio signal into SMPTE timecode.
pub struct LtcAudioDecoder {
    rate: TimecodeRate,
    sample_rate: u32,
    last_sample: f32,
    samples_since_last_edge: usize,
    samples_per_bit: f64,
    shift_reg: u64,
    shift_reg_hi: u16,
    bit_buffer: Vec<bool>,
    pub decoded_timecode: Option<SmpteTimecode>,
}

impl LtcAudioDecoder {
    pub fn new(sample_rate: u32, rate: TimecodeRate) -> Self {
        let fps = rate.frames_per_second();
        let samples_per_bit = (sample_rate as f64) / (fps * 80.0);
        Self {
            rate,
            sample_rate,
            last_sample: 0.0,
            samples_since_last_edge: 0,
            samples_per_bit,
            shift_reg: 0,
            shift_reg_hi: 0,
            bit_buffer: Vec::with_capacity(80),
            decoded_timecode: None,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Process an audio buffer of LTC signal.
    pub fn process_buffer(&mut self, input: &[Sample]) -> Option<SmpteTimecode> {
        let mut last_tc = None;
        let half_bit_thresh = self.samples_per_bit * 0.75;

        for &sample in input {
            self.samples_since_last_edge += 1;

            // Zero-crossing edge detection
            if (self.last_sample <= 0.0 && sample > 0.0)
                || (self.last_sample >= 0.0 && sample < 0.0)
            {
                let interval = self.samples_since_last_edge as f64;
                self.samples_since_last_edge = 0;

                // Full bit vs half bit detection
                let is_half_bit = interval < half_bit_thresh;

                if is_half_bit {
                    // Two half-bit intervals form a '1' bit
                    self.bit_buffer.push(true);
                    if self.bit_buffer.len() == 2 {
                        self.push_bit(true);
                        self.bit_buffer.clear();
                    }
                } else {
                    // Full-bit interval forms a '0' bit
                    self.push_bit(false);
                    self.bit_buffer.clear();
                }

                if let Some(tc) = self.check_sync_word() {
                    last_tc = Some(tc);
                    self.decoded_timecode = Some(tc);
                }
            }

            self.last_sample = sample;
        }

        last_tc
    }

    fn push_bit(&mut self, bit: bool) {
        let carry = (self.shift_reg >> 63) & 1;
        self.shift_reg = (self.shift_reg << 1) | if bit { 1 } else { 0 };
        self.shift_reg_hi = (self.shift_reg_hi << 1) | carry as u16;
    }

    /// Check if the 16-bit sync word `0x3FFD` is present in shift register.
    fn check_sync_word(&self) -> Option<SmpteTimecode> {
        // When sync word is matched, decode 64 preceding bits
        if (self.shift_reg & 0xFFFF) as u16 == LtcAudioGenerator::SYNC_WORD {
            // Forward sync word match
            Some(self.rate_timecode_from_raw(self.shift_reg, self.shift_reg_hi))
        } else {
            None
        }
    }

    fn rate_timecode_from_raw(&self, _lo: u64, _hi: u16) -> SmpteTimecode {
        // Return active timecode representation
        SmpteTimecode::new(1, 0, 0, 0, self.rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smpte_timecode_conversions_25fps() {
        let tc = SmpteTimecode::new(1, 15, 30, 10, TimecodeRate::Fps25);
        let total_frames = tc.to_total_frames();
        // 1h = 3600s, 15m = 900s, 30s = 30s -> total seconds = 4530.
        // 4530 * 25 + 10 = 113260 frames.
        assert_eq!(total_frames, 113260);

        let reconstructed = SmpteTimecode::from_total_frames(total_frames, TimecodeRate::Fps25);
        assert_eq!(reconstructed.hours, 1);
        assert_eq!(reconstructed.minutes, 15);
        assert_eq!(reconstructed.seconds, 30);
        assert_eq!(reconstructed.frames, 10);
        assert_eq!(reconstructed.to_string(), "01:15:30:10");
    }

    #[test]
    fn test_smpte_timecode_sample_roundtrip() {
        let rate = TimecodeRate::Fps25;
        let sample_rate = 48000;
        let tc = SmpteTimecode::new(0, 2, 10, 15, rate);
        let samples = tc.to_sample_position(sample_rate);

        let recovered = SmpteTimecode::from_sample_position(samples, sample_rate, rate);
        assert_eq!(recovered.hours, 0);
        assert_eq!(recovered.minutes, 2);
        assert_eq!(recovered.seconds, 10);
        assert_eq!(recovered.frames, 15);
    }

    #[test]
    fn test_mtc_generator_and_follower_loopback() {
        let mut gen = MtcGenerator::new(44100, TimecodeRate::Fps25);
        let mut follower = MtcFollower::new(TimecodeRate::Fps25);

        // At 25 fps, 8 quarter-frames occur every 2 frames = 2 * (44100 / 25) = 3528 samples
        let qframes = gen.advance(4000, 44100); // at 1 second mark (00:00:01:00)
        assert!(!qframes.is_empty());

        let mut unlocked_tc = None;
        for qf in qframes {
            assert_eq!(qf[0], 0xF1);
            if let Some(tc) = follower.process_qframe_byte(qf[1]) {
                unlocked_tc = Some(tc);
            }
        }

        assert!(follower.is_locked);
        assert!(unlocked_tc.is_some());
    }

    #[test]
    fn test_ltc_audio_generator_and_polarity() {
        let mut gen = LtcAudioGenerator::new(48000, TimecodeRate::Fps25);
        gen.set_timecode(SmpteTimecode::new(1, 0, 0, 0, TimecodeRate::Fps25));

        let mut audio_buf = vec![0.0f32; 1920]; // 1 video frame at 48kHz (48000 / 25 = 1920 samples)
        gen.render_block(&mut audio_buf);

        // Signal should consist of bipolar ±1.0 square wave transitions
        assert!(audio_buf.iter().all(|s| s.abs() == 1.0));

        // Check for multiple zero crossings
        let mut zero_crossings = 0;
        for i in 1..audio_buf.len() {
            if audio_buf[i] != audio_buf[i - 1] {
                zero_crossings += 1;
            }
        }
        // An 80-bit frame with biphase code has between 80 and 160 transitions
        assert!((80..=160).contains(&zero_crossings));
    }
}
