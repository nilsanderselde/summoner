// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Sidechain Routing Matrix & Multi-Bus Lock-Free Event Router.
//!
//! Provides zero-allocation real-time auxiliary bus sidechain routing,
//! multi-track control event multiplexing, and sample-accurate modulation dispatching.

use crate::audio::Sample;
use crate::param_bus::ParamId;
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Maximum number of supported sidechain auxiliary audio buses.
pub const MAX_AUX_BUSES: usize = 32;

/// Maximum number of tracks in the sidechain matrix.
pub const MAX_TRACKS: usize = 64;

/// Maximum number of sample-accurate control events per block.
pub const MAX_EVENTS_PER_BLOCK: usize = 256;

/// Sidechain bus identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SidechainBusId(pub u16);

/// Type of routing connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidechainTapPoint {
    /// Pre-fader track tap.
    PreFader,
    /// Post-fader track tap.
    PostFader,
    /// Direct node output port tap.
    NodeDirect,
}

/// A single sidechain routing connection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidechainRoute {
    /// Source track index (0..MAX_TRACKS).
    pub source_track: usize,
    /// Destination track index (0..MAX_TRACKS).
    pub dest_track: usize,
    /// Destination auxiliary bus index (0..MAX_AUX_BUSES).
    pub aux_bus_index: usize,
    /// Send level (linear gain [0.0, 4.0]).
    pub send_gain: f32,
    /// Tap point (Pre/Post fader).
    pub tap_point: SidechainTapPoint,
    /// Mute active state.
    pub muted: bool,
    /// Solo active state.
    pub solo: bool,
    /// Highpass filter cutoff for sidechain detector in Hz (0.0 = bypassed).
    pub detector_hpf_hz: f32,
    /// Lowpass filter cutoff for sidechain detector in Hz (0.0 = bypassed).
    pub detector_lpf_hz: f32,
}

impl Default for SidechainRoute {
    fn default() -> Self {
        Self {
            source_track: 0,
            dest_track: 0,
            aux_bus_index: 0,
            send_gain: 1.0,
            tap_point: SidechainTapPoint::PostFader,
            muted: false,
            solo: false,
            detector_hpf_hz: 0.0,
            detector_lpf_hz: 0.0,
        }
    }
}

/// Sample-accurate lock-free control event for modulation and sidechain triggering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlEvent {
    /// Sample frame offset within the current processing block (0..block_size).
    pub frame_offset: u32,
    /// Target parameter ID.
    pub target_param: ParamId,
    /// Value payload (normalized or engineering units).
    pub value: f32,
    /// Source track or bus index.
    pub source_id: u16,
}

impl Default for ControlEvent {
    fn default() -> Self {
        Self {
            frame_offset: 0,
            target_param: ParamId(0),
            value: 0.0,
            source_id: 0,
        }
    }
}

/// Routing table snapshot for lock-free atomic hot-swapping.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SidechainConfig {
    pub routes: Vec<SidechainRoute>,
    pub generation: u64,
}

/// Multi-Bus Lock-Free Event Ring Router for real-time inter-track modulation dispatching.
#[derive(Debug)]
pub struct MultiBusEventRouter {
    events: [ControlEvent; MAX_EVENTS_PER_BLOCK],
    event_count: AtomicU32,
}

impl Default for MultiBusEventRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiBusEventRouter {
    pub fn new() -> Self {
        Self {
            events: [ControlEvent::default(); MAX_EVENTS_PER_BLOCK],
            event_count: AtomicU32::new(0),
        }
    }

    /// Push a sample-accurate control event into the block buffer without heap allocation.
    #[inline]
    pub fn push_event(&mut self, event: ControlEvent) -> bool {
        let count = self.event_count.load(Ordering::Relaxed) as usize;
        if count < MAX_EVENTS_PER_BLOCK {
            self.events[count] = event;
            self.event_count.store((count + 1) as u32, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Read active events for the current block. Safe on real-time audio threads.
    #[inline]
    pub fn get_events(&self) -> &[ControlEvent] {
        let count = (self.event_count.load(Ordering::Acquire) as usize).min(MAX_EVENTS_PER_BLOCK);
        &self.events[..count]
    }

    /// Clear event buffer at start of new block.
    #[inline]
    pub fn clear(&mut self) {
        self.event_count.store(0, Ordering::Release);
    }
}

/// Dynamic Sidechain Routing Matrix.
/// Coordinates sidechain bus routing across multiple tracks with zero heap allocations in streaming loops.
pub struct SidechainMatrix {
    pub config: Arc<ArcSwap<SidechainConfig>>,
    current_generation: u64,
    pub event_router: MultiBusEventRouter,
    // Pre-allocated auxiliary bus accumulator buffers: aux_buffers[bus_idx][sample_idx]
    aux_buffers: Vec<Vec<Sample>>,
    max_block_size: usize,
    // Biquad filter states for sidechain detector filters: [bus_idx]
    hpf_states: [f32; MAX_AUX_BUSES],
    lpf_states: [f32; MAX_AUX_BUSES],
}

impl std::fmt::Debug for SidechainMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidechainMatrix")
            .field("current_generation", &self.current_generation)
            .field("max_block_size", &self.max_block_size)
            .finish()
    }
}

impl SidechainMatrix {
    /// Create a new sidechain matrix with pre-allocated auxiliary buffers.
    pub fn new(max_block_size: usize) -> Self {
        let initial_config = Arc::new(SidechainConfig::default());
        let mut aux_buffers = Vec::with_capacity(MAX_AUX_BUSES);
        for _ in 0..MAX_AUX_BUSES {
            aux_buffers.push(vec![0.0f32; max_block_size]);
        }

        Self {
            config: Arc::new(ArcSwap::new(initial_config)),
            current_generation: 0,
            event_router: MultiBusEventRouter::new(),
            aux_buffers,
            max_block_size,
            hpf_states: [0.0f32; MAX_AUX_BUSES],
            lpf_states: [0.0f32; MAX_AUX_BUSES],
        }
    }

    /// Add or update a sidechain route atomically (called from UI/engine management threads).
    pub fn add_route(&mut self, route: SidechainRoute) {
        let current = self.config.load();
        let mut new_routes = current.routes.clone();

        // Update if existing route matches source, dest, and aux bus
        if let Some(existing) = new_routes.iter_mut().find(|r| {
            r.source_track == route.source_track
                && r.dest_track == route.dest_track
                && r.aux_bus_index == route.aux_bus_index
        }) {
            *existing = route;
        } else {
            new_routes.push(route);
        }

        let new_gen = current.generation + 1;
        self.config.store(Arc::new(SidechainConfig {
            routes: new_routes,
            generation: new_gen,
        }));
    }

    /// Remove a route atomically by source, destination, and bus indices.
    pub fn remove_route(&mut self, source_track: usize, dest_track: usize, aux_bus_index: usize) {
        let current = self.config.load();
        let mut new_routes = current.routes.clone();
        new_routes.retain(|r| {
            !(r.source_track == source_track
                && r.dest_track == dest_track
                && r.aux_bus_index == aux_bus_index)
        });

        let new_gen = current.generation + 1;
        self.config.store(Arc::new(SidechainConfig {
            routes: new_routes,
            generation: new_gen,
        }));
    }

    /// Get a pre-allocated auxiliary bus buffer slice for a destination track.
    #[inline]
    pub fn get_aux_bus_slice(&self, aux_bus_index: usize, num_samples: usize) -> &[Sample] {
        if aux_bus_index < self.aux_buffers.len() {
            let len = num_samples.min(self.aux_buffers[aux_bus_index].len());
            &self.aux_buffers[aux_bus_index][..len]
        } else {
            &[]
        }
    }

    /// Process and route all sidechain signals for the current audio block.
    /// Zero heap allocations guaranteed under `AllocGuard`.
    pub fn process_block(
        &mut self,
        track_outputs: &[&[Sample]],
        num_samples: usize,
        sample_rate: u32,
    ) {
        let samples_to_process = num_samples.min(self.max_block_size);

        // 1. Clear auxiliary accumulation buffers
        for bus in self.aux_buffers.iter_mut() {
            for s in bus[..samples_to_process].iter_mut() {
                *s = 0.0;
            }
        }

        // 2. Fetch lock-free atomic routing snapshot
        let config = self.config.load();

        // 3. Accumulate routed audio from source tracks to destination aux buses
        for route in config.routes.iter() {
            if route.muted || route.source_track >= track_outputs.len() || route.aux_bus_index >= MAX_AUX_BUSES {
                continue;
            }

            let src_buf = track_outputs[route.source_track];
            let aux_buf = &mut self.aux_buffers[route.aux_bus_index];
            let gain = route.send_gain;

            let count = samples_to_process.min(src_buf.len());

            for i in 0..count {
                let mut sample = src_buf[i] * gain;

                // Simple 1-pole highpass detector filter if configured
                if route.detector_hpf_hz > 10.0 {
                    let rc = 1.0 / (2.0 * std::f32::consts::PI * route.detector_hpf_hz);
                    let dt = 1.0 / (sample_rate as f32);
                    let alpha = rc / (rc + dt);
                    let prev = self.hpf_states[route.aux_bus_index];
                    let filtered = alpha * (prev + sample);
                    self.hpf_states[route.aux_bus_index] = sample;
                    sample = filtered;
                }

                // Simple 1-pole lowpass detector filter if configured
                if route.detector_lpf_hz > 10.0 && route.detector_lpf_hz < (sample_rate as f32 * 0.49) {
                    let dt = 1.0 / (sample_rate as f32);
                    let rc = 1.0 / (2.0 * std::f32::consts::PI * route.detector_lpf_hz);
                    let alpha = dt / (rc + dt);
                    self.lpf_states[route.aux_bus_index] += alpha * (sample - self.lpf_states[route.aux_bus_index]);
                    sample = self.lpf_states[route.aux_bus_index];
                }

                aux_buf[i] += sample;
            }
        }

        // 4. Sample clamping on accumulated aux buses
        for bus in self.aux_buffers.iter_mut() {
            for s in bus[..samples_to_process].iter_mut() {
                *s = s.clamp(-4.0, 4.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_sidechain_matrix_routing_and_zero_alloc() {
        let mut matrix = SidechainMatrix::new(512);

        let route1 = SidechainRoute {
            source_track: 0,
            dest_track: 1,
            aux_bus_index: 0,
            send_gain: 0.8,
            tap_point: SidechainTapPoint::PostFader,
            muted: false,
            solo: false,
            detector_hpf_hz: 0.0,
            detector_lpf_hz: 0.0,
        };

        let route2 = SidechainRoute {
            source_track: 2,
            dest_track: 1,
            aux_bus_index: 0,
            send_gain: 0.5,
            tap_point: SidechainTapPoint::PostFader,
            muted: false,
            solo: false,
            detector_hpf_hz: 0.0,
            detector_lpf_hz: 0.0,
        };

        matrix.add_route(route1);
        matrix.add_route(route2);

        let track0 = [0.5f32; 128];
        let track1 = [0.0f32; 128];
        let track2 = [0.2f32; 128];

        let track_refs: [&[Sample]; 3] = [&track0[..], &track1[..], &track2[..]];

        // Verify zero heap allocation during routing execution
        let _guard = AllocGuard::new();

        matrix.process_block(&track_refs, 128, 48000);

        drop(_guard);

        let aux0 = matrix.get_aux_bus_slice(0, 128);
        assert_eq!(aux0.len(), 128);

        // Expected output: 0.5 * 0.8 + 0.2 * 0.5 = 0.4 + 0.1 = 0.5
        let first_sample = aux0[0];
        assert!((first_sample - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_multibus_event_router_sample_accuracy() {
        let mut router = MultiBusEventRouter::new();

        let ev1 = ControlEvent {
            frame_offset: 12,
            target_param: ParamId(42),
            value: 0.75,
            source_id: 1,
        };

        let ev2 = ControlEvent {
            frame_offset: 64,
            target_param: ParamId(88),
            value: 1.0,
            source_id: 2,
        };

        assert!(router.push_event(ev1));
        assert!(router.push_event(ev2));

        let events = router.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].frame_offset, 12);
        assert_eq!(events[0].value, 0.75);
        assert_eq!(events[1].frame_offset, 64);
        assert_eq!(events[1].value, 1.0);

        router.clear();
        assert_eq!(router.get_events().len(), 0);
    }
}
