// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Global Harmonic Bus reactive context for pitch, microtonal tuning, and real-time tension tracking.

use crate::cadence::{CadenceType, HarmonicTension, HarmonicTensionAnalyzer};
use crate::edo::EdoTuning;
use crate::scale::Scale;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Global Harmonic Context providing microtonal tuning, scale quantization,
/// active pitch registry, and real-time tension analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarmonicContext {
    pub tuning: EdoTuning,
    pub root_note: u16,
    pub scale: Scale,
    pub active_notes: Vec<u8>,
}

impl HarmonicContext {
    pub fn new(tuning: EdoTuning, root_note: u16, scale: Scale) -> Self {
        Self {
            tuning,
            root_note,
            scale,
            active_notes: Vec::new(),
        }
    }

    /// Add a note to active notes set.
    pub fn push_note_on(&mut self, note: u8) {
        if !self.active_notes.contains(&note) {
            self.active_notes.push(note);
        }
    }

    /// Remove a note from active notes set.
    pub fn push_note_off(&mut self, note: u8) {
        self.active_notes.retain(|&n| n != note);
    }

    /// Clear all active notes.
    pub fn clear_notes(&mut self) {
        self.active_notes.clear();
    }

    /// Analyze active chord label.
    pub fn analyze_active_chord(&self) -> String {
        if self.active_notes.is_empty() {
            return "Silence".to_string();
        }
        let mut pcs: Vec<u8> = self.active_notes.iter().map(|n| n % 12).collect();
        pcs.sort();
        pcs.dedup();

        if pcs == vec![0, 4, 7] || (pcs.contains(&0) && pcs.contains(&4) && pcs.contains(&7)) {
            "C Major".to_string()
        } else if pcs == vec![0, 3, 7] || (pcs.contains(&0) && pcs.contains(&3) && pcs.contains(&7)) {
            "C Minor".to_string()
        } else if pcs.contains(&7) && pcs.contains(&11) && pcs.contains(&2) {
            "G Major (V)".to_string()
        } else if pcs.contains(&5) && pcs.contains(&9) && pcs.contains(&0) {
            "F Major (IV)".to_string()
        } else {
            format!("Chord({:?})", pcs)
        }
    }

    /// Perform real-time harmonic tension analysis on currently active notes.
    pub fn analyze_current_tension(&self) -> HarmonicTension {
        HarmonicTensionAnalyzer::analyze_notes(&self.active_notes, self.root_note)
    }

    /// Suggest harmonically compatible next chord notes based on active harmony.
    pub fn suggest_next_chord_notes(&self) -> Vec<u8> {
        let current_label = self.analyze_active_chord();
        let tension = self.analyze_current_tension();

        if tension.predicted_cadence == CadenceType::Authentic || current_label.contains("Major (V)") {
            vec![60, 64, 67] // Resolve to Tonic C Major (I)
        } else if current_label.contains("Major") || current_label == "Silence" {
            vec![67, 71, 74] // Build tension towards G Major (V)
        } else {
            vec![65, 69, 72] // F Major (IV)
        }
    }

    /// Resolve note index to frequency in Hz using current tuning context.
    #[inline]
    pub fn freq_from_note(&self, note: f64) -> f64 {
        self.tuning.note_to_freq(note)
    }

    /// Snap continuous note index to closest scale step in context.
    #[inline]
    pub fn snap_to_scale(&self, note: f64) -> f64 {
        self.scale.snap_note(note, self.root_note, &self.tuning)
    }

    /// Calculate frequency for a scale degree with scale quantization.
    #[inline]
    pub fn scale_degree_freq(&self, degree: f64) -> f64 {
        let snapped_note = self.snap_to_scale(degree);
        self.freq_from_note(snapped_note)
    }

    /// Dispatch real-time tension metrics into `ParamBus` atomically.
    pub fn dispatch_to_param_bus(
        &self,
        bus: &ParamBus,
        tension_id: ParamId,
        cadence_id: ParamId,
        urgency_id: ParamId,
    ) {
        let tension = self.analyze_current_tension();
        bus.set(tension_id, tension.total_tension);
        bus.set(cadence_id, tension.predicted_cadence.to_f32());
        bus.set(urgency_id, tension.resolution_urgency);
    }
}

impl Default for HarmonicContext {
    fn default() -> Self {
        Self {
            tuning: EdoTuning::standard_12_tet(),
            root_note: 0, // C
            scale: Scale::major_12_tet(),
            active_notes: Vec::new(),
        }
    }
}

/// Thread-safe Lock-Free Harmonic Bus Atomic Bridge for audio thread queries.
#[derive(Debug)]
pub struct HarmonicBusBridge {
    pub root_note: AtomicU32,
    pub tension_score: AtomicU32,
    pub cadence_type: AtomicU32,
}

impl Default for HarmonicBusBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonicBusBridge {
    pub fn new() -> Self {
        Self {
            root_note: AtomicU32::new(0),
            tension_score: AtomicU32::new(0.0f32.to_bits()),
            cadence_type: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn update_from_tension(&self, root: u16, tension: &HarmonicTension) {
        self.root_note.store(root as u32, Ordering::Relaxed);
        self.tension_score.store(tension.total_tension.to_bits(), Ordering::Relaxed);
        self.cadence_type.store(tension.predicted_cadence as u32, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_tension(&self) -> f32 {
        f32::from_bits(self.tension_score.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn get_cadence(&self) -> CadenceType {
        match self.cadence_type.load(Ordering::Relaxed) {
            1 => CadenceType::Plagal,
            2 => CadenceType::Deceptive,
            3 => CadenceType::Half,
            _ => CadenceType::Authentic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_notes_and_suggestions() {
        let mut ctx = HarmonicContext::default();
        ctx.push_note_on(60);
        ctx.push_note_on(64);
        ctx.push_note_on(67);

        assert_eq!(ctx.analyze_active_chord(), "C Major");
        let suggestion = ctx.suggest_next_chord_notes();
        assert!(!suggestion.is_empty());
    }

    #[test]
    fn test_harmonic_context_tension_dispatch() {
        let mut ctx = HarmonicContext::default();
        ctx.push_note_on(67); // G
        ctx.push_note_on(71); // B
        ctx.push_note_on(74); // D
        ctx.push_note_on(77); // F

        let mut param_bus = ParamBus::new();
        let p_tens = param_bus.register(ParamId(100), 0.0);
        let p_cad = param_bus.register(ParamId(101), 0.0);
        let p_urg = param_bus.register(ParamId(102), 0.0);

        ctx.dispatch_to_param_bus(&param_bus, ParamId(100), ParamId(101), ParamId(102));

        assert!(p_tens.get() > 0.3);
        assert_eq!(p_cad.get(), 0.0); // Authentic cadence predicted
        assert!(p_urg.get() >= 0.65);

        let bridge = HarmonicBusBridge::new();
        let tension = ctx.analyze_current_tension();
        bridge.update_from_tension(ctx.root_note, &tension);
        assert_eq!(bridge.get_cadence(), CadenceType::Authentic);
        assert!((bridge.get_tension() - tension.total_tension).abs() < 1e-6);
    }
}
