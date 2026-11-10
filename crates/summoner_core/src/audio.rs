// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Real-time zero-allocation audio primitives and buffer abstractions.

/// Audio sample primitive type (32-bit floating point).
pub type Sample = f32;

/// Fixed-size multi-channel sample frame. Zero heap allocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame<const CHANNELS: usize> {
    pub channels: [Sample; CHANNELS],
}

impl<const CHANNELS: usize> Frame<CHANNELS> {
    /// Create a new frame initialized to silence (0.0).
    pub const fn silent() -> Self {
        Self {
            channels: [0.0; CHANNELS],
        }
    }

    /// Create a frame with all channels set to `value`.
    pub const fn splat(value: Sample) -> Self {
        Self {
            channels: [value; CHANNELS],
        }
    }

    /// Scale all channels in the frame by `gain`.
    #[inline]
    pub fn scale(&mut self, gain: Sample) {
        for ch in self.channels.iter_mut() {
            *ch *= gain;
        }
    }

    /// Mix another frame into this frame.
    #[inline]
    pub fn mix(&mut self, other: &Self) {
        for (dst, src) in self.channels.iter_mut().zip(other.channels.iter()) {
            *dst += src;
        }
    }
}

impl<const CHANNELS: usize> Default for Frame<CHANNELS> {
    fn default() -> Self {
        Self::silent()
    }
}

/// Fixed-capacity stack audio buffer for heap-free audio block processing.
#[derive(Debug, Clone)]
pub struct FixedAudioBuffer<const CHANNELS: usize, const MAX_FRAMES: usize> {
    data: [[Sample; MAX_FRAMES]; CHANNELS],
    active_frames: usize,
}

impl<const CHANNELS: usize, const MAX_FRAMES: usize> FixedAudioBuffer<CHANNELS, MAX_FRAMES> {
    /// Create a new silent buffer.
    pub fn new() -> Self {
        Self {
            data: [[0.0; MAX_FRAMES]; CHANNELS],
            active_frames: MAX_FRAMES,
        }
    }

    /// Reset buffer contents to 0.0.
    pub fn clear(&mut self) {
        for ch in 0..CHANNELS {
            self.data[ch].fill(0.0);
        }
    }

    pub fn num_channels(&self) -> usize {
        CHANNELS
    }

    pub fn num_frames(&self) -> usize {
        self.active_frames
    }

    pub fn set_active_frames(&mut self, frames: usize) {
        assert!(
            frames <= MAX_FRAMES,
            "active frames exceeds MAX_FRAMES capacity"
        );
        self.active_frames = frames;
    }

    pub fn channel(&self, ch: usize) -> &[Sample] {
        &self.data[ch][..self.active_frames]
    }

    pub fn channel_mut(&mut self, ch: usize) -> &mut [Sample] {
        &mut self.data[ch][..self.active_frames]
    }

    pub fn channels_ref_2(&self) -> [&[Sample]; 2] {
        assert!(CHANNELS >= 2, "channels_ref_2 requires CHANNELS >= 2");
        let active = self.active_frames;
        [&self.data[0][..active], &self.data[1][..active]]
    }

    pub fn channels_mut_2(&mut self) -> [&mut [Sample]; 2] {
        assert!(CHANNELS >= 2, "channels_mut_2 requires CHANNELS >= 2");
        let active = self.active_frames;
        let (left, right) = self.data.split_at_mut(1);
        [&mut left[0][..active], &mut right[0][..active]]
    }

    pub fn get_frame(&self, frame_idx: usize) -> Frame<CHANNELS> {
        let mut f = Frame::silent();
        for ch in 0..CHANNELS {
            f.channels[ch] = self.data[ch][frame_idx];
        }
        f
    }

    pub fn set_frame(&mut self, frame_idx: usize, frame: Frame<CHANNELS>) {
        for ch in 0..CHANNELS {
            self.data[ch][frame_idx] = frame.channels[ch];
        }
    }
}

impl<const CHANNELS: usize, const MAX_FRAMES: usize> Default
    for FixedAudioBuffer<CHANNELS, MAX_FRAMES>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Standard audio channel layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ChannelLayout {
    /// 1 Channel (Mono)
    Mono,
    /// 2 Channels (Stereo L, R)
    Stereo,
    /// 6 Channels (5.1: L, R, C, LFE, Ls, Rs)
    Surround5_1,
    /// 8 Channels (7.1: L, R, C, LFE, Ls, Rs, Lb, Rb)
    Surround7_1,
    /// 12 Channels (7.1.4 Dolby Atmos: L, R, C, LFE, Ls, Rs, Lb, Rb, Tfl, Tfr, Tbl, Tbr)
    Surround7_1_4,
    /// Custom layout with N channels
    Custom(usize),
}

impl ChannelLayout {
    /// Return the channel count for this layout.
    pub fn channels(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround5_1 => 6,
            Self::Surround7_1 => 8,
            Self::Surround7_1_4 => 12,
            Self::Custom(n) => *n,
        }
    }

    /// Return channel labels.
    pub fn channel_names(&self) -> Vec<&'static str> {
        match self {
            Self::Mono => vec!["C"],
            Self::Stereo => vec!["L", "R"],
            Self::Surround5_1 => vec!["L", "R", "C", "LFE", "Ls", "Rs"],
            Self::Surround7_1 => vec!["L", "R", "C", "LFE", "Ls", "Rs", "Lb", "Rb"],
            Self::Surround7_1_4 => vec![
                "L", "R", "C", "LFE", "Ls", "Rs", "Lb", "Rb", "Tfl", "Tfr", "Tbl", "Tbr",
            ],
            Self::Custom(n) => (0..*n).map(|_| "Ch").collect(),
        }
    }
}

/// Multichannel audio buffer supporting arbitrary N-channel layouts up to 16 channels.
#[derive(Debug, Clone)]
pub struct MultichannelAudioBuffer {
    layout: ChannelLayout,
    data: Vec<Vec<Sample>>,
    active_frames: usize,
    max_frames: usize,
}

impl MultichannelAudioBuffer {
    pub fn new(layout: ChannelLayout) -> Self {
        Self::with_max_frames(layout, 1024)
    }

    pub fn with_max_frames(layout: ChannelLayout, max_frames: usize) -> Self {
        let chs = layout.channels();
        let data = vec![vec![0.0; max_frames]; chs];
        Self {
            layout,
            data,
            active_frames: max_frames,
            max_frames,
        }
    }

    pub fn layout(&self) -> ChannelLayout {
        self.layout
    }

    pub fn num_channels(&self) -> usize {
        self.data.len()
    }

    pub fn num_frames(&self) -> usize {
        self.active_frames
    }

    pub fn set_active_frames(&mut self, frames: usize) {
        assert!(
            frames <= self.max_frames,
            "active frames exceeds MAX_FRAMES"
        );
        self.active_frames = frames;
    }

    pub fn clear(&mut self) {
        for ch in self.data.iter_mut() {
            ch[..self.active_frames].fill(0.0);
        }
    }

    pub fn channel(&self, ch: usize) -> &[Sample] {
        &self.data[ch][..self.active_frames]
    }

    pub fn channel_mut(&mut self, ch: usize) -> &mut [Sample] {
        &mut self.data[ch][..self.active_frames]
    }

    /// Downmix current multichannel audio into a 2-channel stereo buffer.
    pub fn downmix_to_stereo(&self, stereo_l: &mut [Sample], stereo_r: &mut [Sample]) {
        let frames = self.active_frames.min(stereo_l.len()).min(stereo_r.len());
        stereo_l[..frames].fill(0.0);
        stereo_r[..frames].fill(0.0);

        let chs = self.num_channels();
        if chs == 0 {
            return;
        }

        if chs == 1 {
            stereo_l[..frames].copy_from_slice(&self.data[0][..frames]);
            stereo_r[..frames].copy_from_slice(&self.data[0][..frames]);
            return;
        }

        // Downmix matrix weights based on ITU-R BS.775
        let weights: Vec<(f32, f32)> = match self.layout {
            ChannelLayout::Stereo => vec![(1.0, 0.0), (0.0, 1.0)],
            ChannelLayout::Surround5_1 => vec![
                (1.0, 0.0),     // L
                (0.0, 1.0),     // R
                (0.707, 0.707), // C
                (0.0, 0.0),     // LFE
                (0.707, 0.0),   // Ls
                (0.0, 0.707),   // Rs
            ],
            ChannelLayout::Surround7_1 => vec![
                (1.0, 0.0),     // L
                (0.0, 1.0),     // R
                (0.707, 0.707), // C
                (0.0, 0.0),     // LFE
                (0.707, 0.0),   // Ls
                (0.0, 0.707),   // Rs
                (0.5, 0.0),     // Lb
                (0.0, 0.5),     // Rb
            ],
            ChannelLayout::Surround7_1_4 => vec![
                (1.0, 0.0),     // L
                (0.0, 1.0),     // R
                (0.707, 0.707), // C
                (0.0, 0.0),     // LFE
                (0.707, 0.0),   // Ls
                (0.0, 0.707),   // Rs
                (0.5, 0.0),     // Lb
                (0.0, 0.5),     // Rb
                (0.5, 0.0),     // Tfl
                (0.0, 0.5),     // Tfr
                (0.35, 0.0),    // Tbl
                (0.0, 0.35),    // Tbr
            ],
            _ => (0..chs)
                .map(|i| if i % 2 == 0 { (1.0, 0.0) } else { (0.0, 1.0) })
                .collect(),
        };

        for (ch_idx, (wl, wr)) in weights.iter().enumerate().take(chs) {
            let src = &self.data[ch_idx][..frames];
            for i in 0..frames {
                stereo_l[i] += src[i] * wl;
                stereo_r[i] += src[i] * wr;
            }
        }
    }
}

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Lock-free Single-Producer Single-Consumer (SPSC) Audio Ring Buffer.
/// Safe for zero-heap-allocation audio callback transfers under `AllocGuard`.
#[derive(Debug)]
pub struct AudioRingBuffer {
    data: Box<[AtomicU32]>,
    capacity: usize,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

impl AudioRingBuffer {
    /// Create a new ring buffer with the specified capacity in samples.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(64);
        let mut data = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            data.push(AtomicU32::new(0.0f32.to_bits()));
        }
        Self {
            data: data.into_boxed_slice(),
            capacity,
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    /// Total sample capacity of the ring buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of samples currently available to read.
    pub fn available_read(&self) -> usize {
        let w = self.write_idx.load(Ordering::Acquire);
        let r = self.read_idx.load(Ordering::Acquire);
        if w >= r {
            w - r
        } else {
            self.capacity - (r - w)
        }
    }

    /// Number of samples space currently available to write.
    pub fn available_write(&self) -> usize {
        self.capacity.saturating_sub(self.available_read() + 1)
    }

    /// Push slice into ring buffer without heap allocation.
    /// Returns the number of samples written.
    pub fn push_slice(&self, src: &[Sample]) -> usize {
        let r = self.read_idx.load(Ordering::Acquire);
        let w = self.write_idx.load(Ordering::Relaxed);
        let avail = if w >= r {
            self.capacity.saturating_sub(w - r + 1)
        } else {
            r - w - 1
        };
        let to_write = src.len().min(avail);
        for (i, &sample) in src.iter().enumerate().take(to_write) {
            let idx = (w + i) % self.capacity;
            self.data[idx].store(sample.to_bits(), Ordering::Relaxed);
        }
        let new_w = (w + to_write) % self.capacity;
        self.write_idx.store(new_w, Ordering::Release);
        to_write
    }

    /// Pop samples from ring buffer into destination slice without heap allocation.
    /// Returns the number of samples read.
    pub fn pop_slice(&self, dst: &mut [Sample]) -> usize {
        let w = self.write_idx.load(Ordering::Acquire);
        let r = self.read_idx.load(Ordering::Relaxed);
        let avail = if w >= r {
            w - r
        } else {
            self.capacity - (r - w)
        };
        let to_read = dst.len().min(avail);
        for (i, slot) in dst.iter_mut().enumerate().take(to_read) {
            let idx = (r + i) % self.capacity;
            *slot = f32::from_bits(self.data[idx].load(Ordering::Relaxed));
        }
        let new_r = (r + to_read) % self.capacity;
        self.read_idx.store(new_r, Ordering::Release);
        to_read
    }

    /// Reset ring buffer indices to empty.
    pub fn clear(&self) {
        self.read_idx.store(0, Ordering::Release);
        self.write_idx.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_ring_buffer_push_pop_round_trip() {
        let rb = AudioRingBuffer::new(256);
        assert_eq!(rb.available_read(), 0);
        assert!(rb.available_write() >= 250);

        let input_samples = [0.1f32, 0.2, 0.3, 0.4, 0.5, -0.6, -0.7];
        let written = rb.push_slice(&input_samples);
        assert_eq!(written, input_samples.len());
        assert_eq!(rb.available_read(), input_samples.len());

        let mut read_samples = [0.0f32; 7];
        let read = rb.pop_slice(&mut read_samples);
        assert_eq!(read, 7);
        assert_eq!(read_samples, input_samples);
        assert_eq!(rb.available_read(), 0);
    }

    #[test]
    fn test_audio_ring_buffer_zero_alloc_under_guard() {
        use crate::allocator::AllocGuard;

        let rb = AudioRingBuffer::new(512);
        let input = [0.42f32; 128];
        let mut output = [0.0f32; 128];

        {
            let _guard = AllocGuard::new();
            let pushed = rb.push_slice(&input);
            assert_eq!(pushed, 128);
            let popped = rb.pop_slice(&mut output);
            assert_eq!(popped, 128);
        }

        assert_eq!(output[0], 0.42);
    }
}
