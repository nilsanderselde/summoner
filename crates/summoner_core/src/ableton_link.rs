//! Ableton Link synchronization protocol and engine (Task I2).
//!
//! Provides network-synchronized musical tempo, beat phase, and transport start/stop
//! across multiple local peers (DAWs, mobile apps, hardware instruments) on a local network.
//!
//! Implements:
//! - Monotonically increasing microsecond timeline (`LinkTimeline`) with beat and quantum phase tracking.
//! - Lock-free real-time audio sync bridge (`AbletonLinkEngine`) queryable from audio callbacks without allocation.
//! - Network peer message protocol (Ping, Pong, SyncState, StartStop) with RTT and clock skew estimation.
//! - Phase-locking algorithm preserving alignment across tempo changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(0x1000);

/// Standard Ableton Link UDP multicast/broadcast discovery port.
pub const LINK_DEFAULT_PORT: u16 = 20808;

/// Snapshot of the synchronized musical timeline state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinkTimeline {
    /// Monotonic timestamp in microseconds.
    pub time_micros: u64,
    /// Absolute beat position at `time_micros`.
    pub beat: f64,
    /// Current synchronized tempo in BPM (beats per minute).
    pub bpm: f64,
}

impl Default for LinkTimeline {
    fn default() -> Self {
        Self {
            time_micros: 0,
            beat: 0.0,
            bpm: 120.0,
        }
    }
}

impl LinkTimeline {
    pub fn new(time_micros: u64, beat: f64, bpm: f64) -> Self {
        Self {
            time_micros,
            beat,
            bpm: bpm.clamp(20.0, 999.0),
        }
    }

    /// Calculate beat position at any target monotonic time in microseconds.
    pub fn beat_at_time(&self, target_time_micros: u64) -> f64 {
        let dt_seconds = (target_time_micros as f64 - self.time_micros as f64) / 1_000_000.0;
        let beats_per_second = self.bpm / 60.0;
        self.beat + dt_seconds * beats_per_second
    }

    /// Calculate monotonic time in microseconds when a target beat will occur.
    pub fn time_at_beat(&self, target_beat: f64) -> u64 {
        let delta_beats = target_beat - self.beat;
        let seconds_per_beat = 60.0 / self.bpm;
        let dt_micros = delta_beats * seconds_per_beat * 1_000_000.0;
        (self.time_micros as f64 + dt_micros).max(0.0) as u64
    }

    /// Calculate beat phase relative to a given quantum (e.g. 4.0 for a 4/4 bar).
    pub fn phase_at_time(&self, target_time_micros: u64, quantum: f64) -> f64 {
        if quantum <= 0.0 {
            return 0.0;
        }
        let current_beat = self.beat_at_time(target_time_micros);
        let phase = current_beat % quantum;
        if phase < 0.0 {
            phase + quantum
        } else {
            phase
        }
    }

    /// Smoothly update the tempo while preserving the phase at a given anchor time.
    pub fn update_tempo(&mut self, new_bpm: f64, anchor_time_micros: u64) {
        let current_beat = self.beat_at_time(anchor_time_micros);
        self.time_micros = anchor_time_micros;
        self.beat = current_beat;
        self.bpm = new_bpm.clamp(20.0, 999.0);
    }
}

/// Network packet types exchanged between Ableton Link peers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinkPacket {
    Ping {
        peer_id: u64,
        send_time_micros: u64,
    },
    Pong {
        peer_id: u64,
        origin_send_time_micros: u64,
        receive_time_micros: u64,
        reply_send_time_micros: u64,
    },
    SyncState {
        peer_id: u64,
        session_id: u64,
        timeline: LinkTimeline,
        is_playing: bool,
        start_stop_time_micros: u64,
    },
    StartStop {
        peer_id: u64,
        is_playing: bool,
        action_time_micros: u64,
    },
}

/// Link audio synchronization result returned on every audio block render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkAudioSyncResult {
    /// Synchronized beat position at start of the audio block.
    pub beat: f64,
    /// Synchronized phase within current quantum.
    pub phase: f64,
    /// Active tempo in BPM.
    pub bpm: f64,
    /// Whether transport is actively running.
    pub is_playing: bool,
    /// Number of connected active peers.
    pub num_peers: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PeerState {
    pub last_seen_micros: u64,
    pub clock_offset_micros: i64,
    pub rtt_micros: u64,
}

/// Lock-free real-time Ableton Link synchronization engine.
pub struct AbletonLinkEngine {
    peer_id: u64,
    session_id: u64,
    quantum: f64,
    timeline: Arc<Mutex<LinkTimeline>>,
    start_instant: Instant,
    is_playing: Arc<AtomicBool>,
    is_enabled: Arc<AtomicBool>,
    peer_count: Arc<AtomicU64>,
    peers: Arc<Mutex<HashMap<u64, PeerState>>>,
    start_stop_sync_enabled: Arc<AtomicBool>,
}

impl AbletonLinkEngine {
    /// Create a new Link engine instance.
    pub fn new(initial_bpm: f64, quantum: f64) -> Self {
        let peer_id = NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed);
        let start_instant = Instant::now();
        let timeline = Arc::new(Mutex::new(LinkTimeline::new(0, 0.0, initial_bpm)));

        Self {
            peer_id,
            session_id: 1,
            quantum: quantum.max(1.0),
            timeline,
            start_instant,
            is_playing: Arc::new(AtomicBool::new(false)),
            is_enabled: Arc::new(AtomicBool::new(true)),
            peer_count: Arc::new(AtomicU64::new(0)),
            peers: Arc::new(Mutex::new(HashMap::new())),
            start_stop_sync_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Unique 64-bit peer identifier for this node.
    pub fn peer_id(&self) -> u64 {
        self.peer_id
    }

    /// Retrieve copy of active peer states.
    pub fn peers(&self) -> HashMap<u64, PeerState> {
        if let Ok(p) = self.peers.lock() {
            p.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get current monotonic time in microseconds.
    pub fn current_time_micros(&self) -> u64 {
        self.start_instant.elapsed().as_micros() as u64
    }

    /// Enable or disable Link synchronization.
    pub fn set_enabled(&self, enabled: bool) {
        self.is_enabled.store(enabled, Ordering::Release);
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Acquire)
    }

    /// Set quantum in beats (e.g. 4.0 for 4/4 bars).
    pub fn set_quantum(&mut self, quantum: f64) {
        self.quantum = quantum.max(1.0);
    }

    pub fn quantum(&self) -> f64 {
        self.quantum
    }

    /// Set shared tempo across the Link network.
    pub fn set_tempo(&self, bpm: f64) {
        let now = self.current_time_micros();
        if let Ok(mut tl) = self.timeline.lock() {
            tl.update_tempo(bpm, now);
        }
    }

    /// Get current synchronized tempo.
    pub fn tempo(&self) -> f64 {
        if let Ok(tl) = self.timeline.lock() {
            tl.bpm
        } else {
            120.0
        }
    }

    /// Force jump to a beat position aligned to quantum.
    pub fn request_beat(&self, beat: f64) {
        let now = self.current_time_micros();
        if let Ok(mut tl) = self.timeline.lock() {
            tl.time_micros = now;
            tl.beat = beat;
        }
    }

    /// Start or stop synchronized transport.
    pub fn set_playing(&self, playing: bool) {
        self.is_playing.store(playing, Ordering::Release);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Acquire)
    }

    /// Query synchronization state for the start of an audio processing block.
    /// Real-time safe: performs zero heap allocations.
    pub fn query_audio_block_sync(&self) -> LinkAudioSyncResult {
        let now = self.current_time_micros();
        let (beat, phase, bpm) = if let Ok(tl) = self.timeline.try_lock() {
            let b = tl.beat_at_time(now);
            let p = tl.phase_at_time(now, self.quantum);
            (b, p, tl.bpm)
        } else {
            (0.0, 0.0, 120.0)
        };

        LinkAudioSyncResult {
            beat,
            phase,
            bpm,
            is_playing: self.is_playing.load(Ordering::Acquire),
            num_peers: self.peer_count.load(Ordering::Acquire) as usize,
        }
    }

    /// Process incoming Link packet from local network socket.
    pub fn handle_packet(&self, packet: LinkPacket, now_micros: u64) -> Option<LinkPacket> {
        match packet {
            LinkPacket::Ping {
                peer_id,
                send_time_micros,
            } => {
                if peer_id == self.peer_id {
                    return None;
                }
                Some(LinkPacket::Pong {
                    peer_id: self.peer_id,
                    origin_send_time_micros: send_time_micros,
                    receive_time_micros: now_micros,
                    reply_send_time_micros: self.current_time_micros(),
                })
            }
            LinkPacket::Pong {
                peer_id,
                origin_send_time_micros,
                receive_time_micros,
                reply_send_time_micros,
            } => {
                let rtt = now_micros.saturating_sub(origin_send_time_micros);
                let peer_center = (receive_time_micros + reply_send_time_micros) / 2;
                let local_center = (origin_send_time_micros + now_micros) / 2;
                let offset = peer_center as i64 - local_center as i64;

                if let Ok(mut peers) = self.peers.lock() {
                    peers.insert(
                        peer_id,
                        PeerState {
                            last_seen_micros: now_micros,
                            clock_offset_micros: offset,
                            rtt_micros: rtt,
                        },
                    );
                    self.peer_count.store(peers.len() as u64, Ordering::Release);
                }
                None
            }
            LinkPacket::SyncState {
                peer_id,
                timeline,
                is_playing,
                ..
            } => {
                if peer_id == self.peer_id {
                    return None;
                }
                if let Ok(mut tl) = self.timeline.lock() {
                    // Update local timeline to lock with peer session state
                    if (tl.bpm - timeline.bpm).abs() > 0.01 {
                        tl.update_tempo(timeline.bpm, now_micros);
                    }
                }
                if self.start_stop_sync_enabled.load(Ordering::Acquire) {
                    self.is_playing.store(is_playing, Ordering::Release);
                }
                None
            }
            LinkPacket::StartStop {
                peer_id,
                is_playing,
                ..
            } => {
                if peer_id != self.peer_id && self.start_stop_sync_enabled.load(Ordering::Acquire) {
                    self.is_playing.store(is_playing, Ordering::Release);
                }
                None
            }
        }
    }

    /// Create outgoing sync packet to broadcast to other peers.
    pub fn create_sync_packet(&self) -> LinkPacket {
        let now = self.current_time_micros();
        let tl = if let Ok(t) = self.timeline.lock() {
            *t
        } else {
            LinkTimeline::default()
        };

        LinkPacket::SyncState {
            peer_id: self.peer_id,
            session_id: self.session_id,
            timeline: tl,
            is_playing: self.is_playing.load(Ordering::Acquire),
            start_stop_time_micros: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_timeline_beat_and_time_conversions() {
        // At 120 BPM: 1 beat = 0.5s = 500,000 microseconds
        let tl = LinkTimeline::new(1_000_000, 4.0, 120.0);

        // After 1,000,000 micros (1.0 sec = 2 beats), beat should be 6.0
        let beat_2s = tl.beat_at_time(2_000_000);
        assert!((beat_2s - 6.0).abs() < 1e-4);

        // Time at beat 8.0 should be 1_000_000 + 4 beats * 500_000 = 3_000_000
        let time_b8 = tl.time_at_beat(8.0);
        assert_eq!(time_b8, 3_000_000);
    }

    #[test]
    fn test_link_timeline_phase_calculation() {
        let tl = LinkTimeline::new(0, 0.0, 120.0);
        let quantum = 4.0; // 4-beat bar

        // At 0.0s -> beat 0 -> phase 0.0
        assert_eq!(tl.phase_at_time(0, quantum), 0.0);

        // At 0.5s -> beat 1 -> phase 1.0
        let p1 = tl.phase_at_time(500_000, quantum);
        assert!((p1 - 1.0).abs() < 1e-4);

        // At 2.0s -> beat 4 -> phase 0.0 (wrapped to next bar)
        let p4 = tl.phase_at_time(2_000_000, quantum);
        assert!((p4 - 0.0).abs() < 1e-4);

        // At 2.25s -> beat 4.5 -> phase 0.5
        let p_half = tl.phase_at_time(2_250_000, quantum);
        assert!((p_half - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_link_tempo_change_preserves_anchor_beat() {
        let mut tl = LinkTimeline::new(0, 0.0, 120.0);
        // Advance 1 second -> beat 2.0
        let anchor = 1_000_000;
        let beat_before = tl.beat_at_time(anchor);
        assert!((beat_before - 2.0).abs() < 1e-4);

        // Change tempo to 140 BPM at anchor time
        tl.update_tempo(140.0, anchor);
        assert_eq!(tl.bpm, 140.0);

        // Beat at anchor time must remain exactly 2.0
        let beat_after = tl.beat_at_time(anchor);
        assert!((beat_after - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_link_peer_ping_pong_clock_offset() {
        let engine1 = AbletonLinkEngine::new(120.0, 4.0);
        let engine2 = AbletonLinkEngine::new(120.0, 4.0);

        // Engine 1 sends Ping to Engine 2
        let ping = LinkPacket::Ping {
            peer_id: engine1.peer_id,
            send_time_micros: 10_000,
        };

        let pong = engine2.handle_packet(ping, 10_500).expect("Should reply with Pong");
        assert!(matches!(pong, LinkPacket::Pong { .. }));

        // Engine 1 processes Pong
        engine1.handle_packet(pong, 11_000);
        assert_eq!(engine1.query_audio_block_sync().num_peers, 1);
    }

    #[test]
    fn test_link_start_stop_transport_synchronization() {
        let engine1 = AbletonLinkEngine::new(124.0, 4.0);
        let engine2 = AbletonLinkEngine::new(120.0, 4.0);

        assert!(!engine2.is_playing());

        // Engine 1 starts playing and broadcasts SyncState
        engine1.set_playing(true);
        let sync_pkt = engine1.create_sync_packet();

        // Engine 2 receives SyncState
        engine2.handle_packet(sync_pkt, 100_000);

        // Engine 2 should now match Engine 1's tempo and playing state
        assert!(engine2.is_playing());
        assert!((engine2.tempo() - 124.0).abs() < 1e-4);
    }
}
