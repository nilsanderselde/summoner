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

//! Lock-free real-time parameter exchange bus.
//!
//! Provides a registry of atomic parameters that can be safely updated from
//! a GUI or CLI thread while being concurrently read by the real-time audio thread
//! without locking or allocation.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// A lightweight, copyable identifier for a registered parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamId(pub u32);

/// A single atomic floating-point parameter.
#[derive(Debug)]
pub struct AtomicParam {
    /// Parameter identifier.
    pub id: ParamId,
    value: AtomicU32,
}

impl AtomicParam {
    /// Create a new atomic parameter with an initial f32 value.
    pub fn new(id: ParamId, initial: f32) -> Arc<Self> {
        Arc::new(Self {
            id,
            value: AtomicU32::new(initial.to_bits()),
        })
    }

    /// Read the current parameter value atomically. Safe for real-time threads.
    #[inline]
    pub fn get(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// Set the parameter value atomically. Safe to call from UI/CLI threads.
    #[inline]
    pub fn set(&self, v: f32) {
        self.value.store(v.to_bits(), Ordering::Relaxed);
    }
}

/// A pre-allocated, fixed registry of parameters.
/// Built at project load time and passed to both the UI and the audio engine.
#[derive(Debug, Default)]
pub struct ParamBus {
    params: Vec<Arc<AtomicParam>>,
}

impl ParamBus {
    /// Create an empty parameter bus.
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// Register a new parameter. Returns the Arc to the atomic container.
    /// This should only be called during setup, NOT on the real-time audio thread.
    pub fn register(&mut self, id: ParamId, initial: f32) -> Arc<AtomicParam> {
        // Ensure vector has enough capacity if index is based on ParamId(u32)
        // Since ParamId is typically dense, we can treat it as an index.
        let idx = id.0 as usize;
        if idx >= self.params.len() {
            self.params.resize_with(idx + 1, || {
                Arc::new(AtomicParam {
                    id: ParamId(0), // Dummy id for padded elements
                    value: AtomicU32::new(0.0f32.to_bits()),
                })
            });
        }
        let param = AtomicParam::new(id, initial);
        self.params[idx] = param.clone();
        param
    }

    /// Get the current value of a parameter. Returns None if unregistered.
    #[inline]
    pub fn get(&self, id: ParamId) -> Option<f32> {
        self.params.get(id.0 as usize).map(|p| p.get())
    }

    /// Set a parameter value by ID. Panics if the parameter is not registered.
    #[inline]
    pub fn set(&self, id: ParamId, v: f32) {
        self.params[id.0 as usize].set(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_param_bus_concurrent_access() {
        let mut bus = ParamBus::new();
        let id1 = ParamId(0);
        let id2 = ParamId(1);
        let id3 = ParamId(5); // Non-contiguous

        let param1 = bus.register(id1, 10.0);
        let param2 = bus.register(id2, 20.0);
        let param3 = bus.register(id3, 30.0);

        assert_eq!(bus.get(id1), Some(10.0));
        assert_eq!(bus.get(id2), Some(20.0));
        assert_eq!(bus.get(id3), Some(30.0));
        assert_eq!(bus.get(ParamId(2)).unwrap(), 0.0); // Dummy padding

        // Simulate GUI thread writing
        let t = thread::spawn({
            let param1_clone = param1.clone();
            let param2_clone = param2.clone();
            move || {
                param1_clone.set(15.0);
                param2_clone.set(25.0);
            }
        });

        t.join().unwrap();

        // Simulate Audio thread reading
        assert_eq!(bus.get(id1), Some(15.0));
        assert_eq!(bus.get(id2), Some(25.0));

        // Test writing via the bus index
        bus.set(id3, 35.0);
        assert_eq!(param3.get(), 35.0);
    }
}
