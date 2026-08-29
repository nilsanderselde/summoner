// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Cadence-Aware Chord Progression Generation & Harmonic Tension Analyzer (Milestone 11).
//!
//! Implements real-time harmonic tension tracking, interval roughness/dissonance calculation,
//! tonal distance on the circle of fifths, cadence resolution prediction, and lock-free
//! dispatch into `ParamBus` and sequencer automation lanes.

use crate::bus::HarmonicContext;
use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Cadence classification for harmonic progressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CadenceType {
    /// Authentic cadence (V -> I / V7 -> I): Strongest resolution.
    #[default]
    Authentic = 0,
    /// Plagal cadence (IV -> I): "Amen" church resolution.
    Plagal = 1,
    /// Deceptive cadence (V -> vi): Interrupted resolution.
    Deceptive = 2,
    /// Half cadence (I -> V / ii -> V): Open tension demanding resolution.
    Half = 3,
}

impl CadenceType {
    pub fn label(&self) -> &'static str {
        match self {
            CadenceType::Authentic => "AUTHENTIC (V -> I)",
            CadenceType::Plagal => "PLAGAL (IV -> I)",
            CadenceType::Deceptive => "DECEPTIVE (V -> vi)",
            CadenceType::Half => "HALF (I -> V)",
        }
    }

    pub fn to_f32(&self) -> f32 {
        *self as u8 as f32
    }
}

/// Chord definition holding root degree, scale intervals, and descriptive name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chord {
    pub name: String,
    pub root_step: i32,
    pub intervals: Vec<i32>,
}

impl Chord {
    pub fn new(name: impl Into<String>, root_step: i32, intervals: Vec<i32>) -> Self {
        Self {
            name: name.into(),
            root_step,
            intervals,
        }
    }

    /// Calculate frequency for each chord note within current harmonic context.
    pub fn frequencies(&self, context: &HarmonicContext) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| {
                let note_step = self.root_step + interval;
                context.scale_degree_freq(note_step as f64)
            })
            .collect()
    }
}

/// Real-time harmonic tension report containing dissonance, circle-of-fifths distance,
/// and predicted cadence resolution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HarmonicTension {
    /// Sensory dissonance / roughness computed from interval collisions (0.0 to 1.0).
    pub roughness: f32,
    /// Tonal distance from the tonic on the Circle of Fifths (0.0 to 1.0).
    pub tonal_distance: f32,
    /// Consolidated harmonic tension score (0.0 = total rest/tonic, 1.0 = maximum tension/climax).
    pub total_tension: f32,
    /// Predicted cadence resolution type.
    pub predicted_cadence: CadenceType,
    /// Urgency of resolution (0.0 to 1.0).
    pub resolution_urgency: f32,
}

impl Default for HarmonicTension {
    fn default() -> Self {
        Self {
            roughness: 0.0,
            tonal_distance: 0.0,
            total_tension: 0.0,
            predicted_cadence: CadenceType::Authentic,
            resolution_urgency: 0.0,
        }
    }
}

/// Real-time harmonic tension tracking analyzer.
#[derive(Debug, Clone, Copy, Default)]
pub struct HarmonicTensionAnalyzer;

impl HarmonicTensionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate circle of fifths distance from tonic (0..=11 pitch class) to chord root.
    pub fn circle_of_fifths_distance(root_pc: u8, tonic_pc: u8) -> f32 {
        let diff = (root_pc as i32 - tonic_pc as i32).rem_euclid(12);
        // Circle of fifths step positions: [0, 7, 2, 9, 4, 11, 6, 1, 8, 3, 10, 5]
        let fifths_steps = match diff {
            0 => 0,
            7 => 1,
            2 => 2,
            9 => 3,
            4 => 4,
            11 => 5,
            6 => 6,
            1 => 5,
            8 => 4,
            3 => 3,
            10 => 2,
            5 => 1,
            _ => 0,
        };
        fifths_steps as f32 / 6.0 // Normalized 0.0 to 1.0
    }

    /// Evaluate interval roughness between pairs of notes.
    /// High roughness: Minor 2nd (1), Major 7th (11), Tritone (6), Minor 9th (13).
    /// Low roughness: Unison (0), Octave (12), Perfect 5th (7), Perfect 4th (5), Major 3rd (4).
    pub fn interval_roughness(interval: u8) -> f32 {
        let semitones = interval % 12;
        match semitones {
            0 => 0.00,  // Unison / Octave
            7 => 0.05,  // Perfect 5th
            5 => 0.10,  // Perfect 4th
            4 => 0.20,  // Major 3rd
            3 => 0.25,  // Minor 3rd
            9 => 0.20,  // Major 6th
            8 => 0.30,  // Minor 6th
            2 => 0.45,  // Major 2nd
            10 => 0.50, // Minor 7th
            6 => 0.85,  // Tritone
            11 => 0.90, // Major 7th
            1 => 1.00,  // Minor 2nd
            _ => 0.0,
        }
    }

    /// Analyze real-time harmonic tension from active MIDI note set.
    pub fn analyze_notes(notes: &[u8], root_note: u16) -> HarmonicTension {
        if notes.is_empty() {
            return HarmonicTension::default();
        }

        let tonic_pc = (root_note % 12) as u8;
        let mut pcs: Vec<u8> = notes.iter().map(|&n| n % 12).collect();
        pcs.sort();
        pcs.dedup();

        if pcs.len() == 1 {
            let dist = Self::circle_of_fifths_distance(pcs[0], tonic_pc);
            return HarmonicTension {
                roughness: 0.0,
                tonal_distance: dist,
                total_tension: dist * 0.4,
                predicted_cadence: CadenceType::Authentic,
                resolution_urgency: dist * 0.3,
            };
        }

        // Compute pairwise roughness
        let mut total_roughness = 0.0f32;
        let mut pairs_count = 0;
        for i in 0..notes.len() {
            for j in (i + 1)..notes.len() {
                let interval = (notes[i] as i32 - notes[j] as i32).unsigned_abs() as u8;
                total_roughness += Self::interval_roughness(interval);
                pairs_count += 1;
            }
        }

        let avg_roughness = if pairs_count > 0 {
            (total_roughness / pairs_count as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Determine effective root and circle-of-fifths distance
        let chord_root_pc = pcs[0];
        let tonal_dist = Self::circle_of_fifths_distance(chord_root_pc, tonic_pc);

        let total_tension = (avg_roughness * 0.60 + tonal_dist * 0.40).clamp(0.0, 1.0);

        // Predict cadence tendency based on pitch class contents
        let predicted_cadence = if pcs.contains(&((tonic_pc + 7) % 12)) && (pcs.contains(&((tonic_pc + 11) % 12)) || pcs.contains(&((tonic_pc + 2) % 12))) {
            // Dominant V or V7 contains Leading tone -> Authentic cadence
            CadenceType::Authentic
        } else if pcs.contains(&((tonic_pc + 5) % 12)) && pcs.contains(&((tonic_pc + 9) % 12)) {
            // Subdominant IV -> Plagal cadence
            CadenceType::Plagal
        } else if pcs.contains(&((tonic_pc + 7) % 12)) && pcs.contains(&((tonic_pc + 9) % 12)) {
            // V -> vi Deceptive cadence
            CadenceType::Deceptive
        } else if pcs.contains(&((tonic_pc + 2) % 12)) || pcs.contains(&((tonic_pc + 4) % 12)) {
            // ii or I -> Half cadence towards V
            CadenceType::Half
        } else {
            CadenceType::Authentic
        };

        let urgency = if predicted_cadence == CadenceType::Authentic {
            total_tension.max(0.65)
        } else {
            total_tension * 0.8
        };

        HarmonicTension {
            roughness: avg_roughness,
            tonal_distance: tonal_dist,
            total_tension,
            predicted_cadence,
            resolution_urgency: urgency,
        }
    }
}

/// Cadence-aware progression engine querying `HarmonicContext`.
#[derive(Debug, Default, Clone)]
pub struct CadenceEngine {
    pub tension_analyzer: HarmonicTensionAnalyzer,
}

impl CadenceEngine {
    pub fn new() -> Self {
        Self {
            tension_analyzer: HarmonicTensionAnalyzer::new(),
        }
    }

    /// Generate a full chord progression matching specified cadence type.
    pub fn generate_progression(context: &HarmonicContext, cadence: CadenceType) -> Vec<Chord> {
        let root = context.root_note as i32;

        match cadence {
            CadenceType::Authentic => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ],
            CadenceType::Plagal => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ],
            CadenceType::Deceptive => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ],
            CadenceType::Half => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("ii", root + 2, vec![0, 3, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
            ],
        }
    }

    /// Suggest harmonically compatible next chords based on current chord.
    pub fn suggest_next_chords(current: &Chord, context: &HarmonicContext) -> Vec<Chord> {
        let root = context.root_note as i32;
        if current.name.contains("I") {
            vec![
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ]
        } else if current.name.contains("V") {
            vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ]
        } else {
            vec![
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ]
        }
    }

    /// Analyze harmonic tension and dispatch atomically into `ParamBus`.
    pub fn dispatch_to_param_bus(
        &self,
        tension: &HarmonicTension,
        param_bus: &ParamBus,
        tension_param_id: ParamId,
        cadence_param_id: ParamId,
        urgency_param_id: ParamId,
    ) {
        param_bus.set(tension_param_id, tension.total_tension);
        param_bus.set(cadence_param_id, tension.predicted_cadence.to_f32());
        param_bus.set(urgency_param_id, tension.resolution_urgency);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::HarmonicContext;

    #[test]
    fn test_cadence_generation() {
        let ctx = HarmonicContext::default();
        let auth_prog = CadenceEngine::generate_progression(&ctx, CadenceType::Authentic);

        assert_eq!(auth_prog.len(), 4);
        assert_eq!(auth_prog[0].name, "I");
        assert_eq!(auth_prog[3].name, "I");

        let freqs = auth_prog[0].frequencies(&ctx);
        assert_eq!(freqs.len(), 3);
        assert!(freqs[0] > 0.0);
    }

    #[test]
    fn test_chord_suggestions() {
        let ctx = HarmonicContext::default();
        let tonic = Chord::new("I", 0, vec![0, 4, 7]);
        let suggestions = CadenceEngine::suggest_next_chords(&tonic, &ctx);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|c| c.name == "V"));
    }

    #[test]
    fn test_harmonic_tension_analyzer_dissonance_and_resolution() {
        // C Major triad: C4(60), E4(64), G4(67) -> low tension
        let c_maj = [60, 64, 67];
        let t_cmaj = HarmonicTensionAnalyzer::analyze_notes(&c_maj, 0);
        assert!(
            t_cmaj.total_tension < 0.35,
            "C Major tonic triad should have low harmonic tension: got {}",
            t_cmaj.total_tension
        );

        // G7 Dominant chord: G4(67), B4(71), D5(74), F5(77) -> High tension (Tritone B-F)
        let g7 = [67, 71, 74, 77];
        let t_g7 = HarmonicTensionAnalyzer::analyze_notes(&g7, 0);
        assert!(
            t_g7.total_tension > t_cmaj.total_tension,
            "G7 Dominant should have higher tension than C Major ({} vs {})",
            t_g7.total_tension,
            t_cmaj.total_tension
        );
        assert_eq!(t_g7.predicted_cadence, CadenceType::Authentic);
        assert!(t_g7.resolution_urgency >= 0.65);
    }

    #[test]
    fn test_cadence_engine_parambus_dispatch() {
        let mut param_bus = ParamBus::new();
        let p_tension = param_bus.register(ParamId(20), 0.0);
        let p_cadence = param_bus.register(ParamId(21), 0.0);
        let p_urgency = param_bus.register(ParamId(22), 0.0);

        let tension = HarmonicTension {
            roughness: 0.75,
            tonal_distance: 0.50,
            total_tension: 0.65,
            predicted_cadence: CadenceType::Authentic,
            resolution_urgency: 0.85,
        };

        let engine = CadenceEngine::new();
        engine.dispatch_to_param_bus(
            &tension,
            &param_bus,
            ParamId(20),
            ParamId(21),
            ParamId(22),
        );

        assert_eq!(p_tension.get(), 0.65);
        assert_eq!(p_cadence.get(), 0.0); // Authentic = 0.0
        assert_eq!(p_urgency.get(), 0.85);
    }
}
