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
        assert!(frames <= MAX_FRAMES, "active frames exceeds MAX_FRAMES capacity");
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

impl<const CHANNELS: usize, const MAX_FRAMES: usize> Default for FixedAudioBuffer<CHANNELS, MAX_FRAMES> {
    fn default() -> Self {
        Self::new()
    }
}
