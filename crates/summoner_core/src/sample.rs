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

//! Content-addressed audio sample topology and BLAKE3 slicing engine.

use crate::audio::Sample;

/// 256-bit BLAKE3 content hash identifier for audio sample payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SampleHash {
    pub bytes: [u8; 32],
}

impl SampleHash {
    pub fn from_hex(hex_str: &str) -> Result<Self, &'static str> {
        let mut bytes = [0u8; 32];
        if hex_str.len() != 64 {
            return Err("Hex string must be 64 characters");
        }
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
                .map_err(|_| "Invalid hex character")?;
        }
        Ok(Self { bytes })
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(64);
        for b in &self.bytes {
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }
}

/// Compute BLAKE3 content hash for raw sample data.
pub fn hash_sample_data(data: &[Sample]) -> SampleHash {
    let mut hasher = blake3::Hasher::new();
    for sample in data {
        hasher.update(&sample.to_le_bytes());
    }
    let hash = hasher.finalize();
    SampleHash {
        bytes: *hash.as_bytes(),
    }
}

/// Content-addressed sample slice reference descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSlice {
    /// BLAKE3 hash of raw sample payload.
    pub content_hash: SampleHash,
    /// Starting frame offset within payload.
    pub start_frame: usize,
    /// Ending frame offset within payload.
    pub end_frame: usize,
    /// Channel count.
    pub channels: usize,
}

impl SampleSlice {
    pub fn new(
        content_hash: SampleHash,
        start_frame: usize,
        end_frame: usize,
        channels: usize,
    ) -> Self {
        assert!(start_frame <= end_frame, "start_frame must be <= end_frame");
        Self {
            content_hash,
            start_frame,
            end_frame,
            channels,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.end_frame - self.start_frame
    }
}
