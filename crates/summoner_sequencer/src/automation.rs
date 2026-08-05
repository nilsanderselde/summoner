use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub const MAX_AUTOMATION_EVENTS: usize = 4096;

/// Fixed-capacity automation event for zero-allocation recording on realtime audio path.
#[derive(Debug, Clone, Copy)]
pub struct AutomationEvent {
    pub frame: u64,
    pub value: f32,
    pub param_id: [u8; 64],
    pub param_id_len: usize,
}

impl Default for AutomationEvent {
    fn default() -> Self {
        Self {
            frame: 0,
            value: 0.0,
            param_id: [0u8; 64],
            param_id_len: 0,
        }
    }
}

impl AutomationEvent {
    pub fn new(frame: u64, id: &str, value: f32) -> Self {
        let mut param_id = [0u8; 64];
        let bytes = id.as_bytes();
        let len = bytes.len().min(64);
        param_id[..len].copy_from_slice(&bytes[..len]);
        Self {
            frame,
            value,
            param_id,
            param_id_len: len,
        }
    }

    pub fn param_id(&self) -> &str {
        std::str::from_utf8(&self.param_id[..self.param_id_len]).unwrap_or("")
    }
}

/// Stack array buffer holding up to 4096 automation events without heap allocation.
#[derive(Debug)]
pub struct AutomationEventBuffer {
    pub events: [AutomationEvent; MAX_AUTOMATION_EVENTS],
    pub count: usize,
}

impl Default for AutomationEventBuffer {
    fn default() -> Self {
        Self {
            events: [AutomationEvent::default(); MAX_AUTOMATION_EVENTS],
            count: 0,
        }
    }
}

impl AutomationEventBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: AutomationEvent) -> bool {
        if self.count < MAX_AUTOMATION_EVENTS {
            self.events[self.count] = event;
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn as_slice(&self) -> &[AutomationEvent] {
        &self.events[..self.count]
    }
}

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutomationMode {
    #[default]
    Read,
    Write,
    Touch,
    Latch,
}

/// Central registry for all automatable parameters across the DAW.
/// Supports the "Record All" toggle for live performance capturing.
#[derive(Debug, Default)]
pub struct AutomationRegistry {
    params: HashMap<String, Arc<AtomicParam>>,
    last_snapshotted: HashMap<String, f32>,
    recording_all: bool,
    mode: AutomationMode,
    latched_params: HashMap<String, f32>,
}

impl AutomationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mode(&mut self, mode: AutomationMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> AutomationMode {
        self.mode
    }

    pub fn register_param(&mut self, id: &str, initial_value: f32) -> Arc<AtomicParam> {
        let param = Arc::new(AtomicParam::new(initial_value));
        self.params.insert(id.to_string(), Arc::clone(&param));
        param
    }

    pub fn get_param(&self, id: &str) -> Option<Arc<AtomicParam>> {
        self.params.get(id).cloned()
    }

    pub fn start_record_all(&mut self) {
        self.recording_all = true;
        self.mode = AutomationMode::Write;
        // Seed the last seen values so we don't dump everything on first frame
        self.last_snapshotted.clear();
        self.latched_params.clear();
        for (id, param) in &self.params {
            self.last_snapshotted.insert(id.clone(), param.get());
        }
    }

    pub fn stop_record_all(&mut self) {
        self.recording_all = false;
        self.mode = AutomationMode::Read;
        self.latched_params.clear();
    }

    pub fn is_recording_all(&self) -> bool {
        self.recording_all
    }

    pub fn set(&self, id: &str, value: f32) {
        if let Some(param) = self.params.get(id) {
            param.set(value);
        }
    }

    pub fn snapshot_dirty_params(&mut self, _frame: u64) -> Vec<(String, f32)> {
        let mut dirty = Vec::new();
        for (id, param) in &self.params {
            let val = param.get();
            let last_val = self.last_snapshotted.get(id).copied().unwrap_or(0.0);
            if (val - last_val).abs() > 1e-5 {
                dirty.push((id.clone(), val));
                self.last_snapshotted.insert(id.clone(), val);
            }
        }
        dirty
    }

    pub fn snapshot_dirty_events(&mut self, frame: u64, buffer: &mut AutomationEventBuffer) {
        buffer.clear();
        for (id, param) in &self.params {
            let val = param.get();
            let last_val = self.last_snapshotted.get(id).copied().unwrap_or(0.0);
            if (val - last_val).abs() > 1e-5 {
                buffer.push(AutomationEvent::new(frame, id, val));
                self.last_snapshotted.insert(id.clone(), val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_event_buffer_stack_array() {
        let mut registry = AutomationRegistry::new();
        let param = registry.register_param("cutoff", 100.0);

        let mut buffer = AutomationEventBuffer::new();
        registry.start_record_all();

        param.set(250.0);
        registry.snapshot_dirty_events(100, &mut buffer);

        assert_eq!(buffer.count, 1);
        assert_eq!(buffer.as_slice()[0].param_id(), "cutoff");
        assert_eq!(buffer.as_slice()[0].value, 250.0);
        assert_eq!(buffer.as_slice()[0].frame, 100);
    }
}
