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

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// A lock-free parameter value for zero-allocation reading on the audio thread.
#[derive(Debug)]
pub struct AtomicParam {
    value: AtomicU32,
}

impl AtomicParam {
    pub fn new(initial: f32) -> Self {
        Self {
            value: AtomicU32::new(initial.to_bits()),
        }
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    pub fn set(&self, val: f32) {
        self.value.store(val.to_bits(), Ordering::Relaxed);
    }
}

/// Central registry for all automatable parameters across the DAW.
/// Supports the "Record All" toggle for live performance capturing.
#[derive(Debug, Default)]
pub struct AutomationRegistry {
    params: HashMap<String, Arc<AtomicParam>>,
    recording_all: bool,
}

impl AutomationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_param(&mut self, id: &str, initial_value: f32) -> Arc<AtomicParam> {
        let param = Arc::new(AtomicParam::new(initial_value));
        self.params.insert(id.to_string(), Arc::clone(&param));
        param
    }

    pub fn get_param(&self, id: &str) -> Option<Arc<AtomicParam>> {
        self.params.get(id).cloned()
    }

    pub fn set_recording_all(&mut self, recording: bool) {
        self.recording_all = recording;
    }

    pub fn is_recording_all(&self) -> bool {
        self.recording_all
    }
}
