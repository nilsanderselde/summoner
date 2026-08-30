// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Cadence Path Resolver & Harmonic Progression Transition Graph (Milestone 14).
//!
//! Implements graph-based harmonic progression tree search, voice-leading cost optimization
//! (Tymoczko L1/L2 metric with parallel-fifth avoidance), circle-of-fifths root motion distance
//! calculations, and lock-free parameter dispatch into `CadenceBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in audio loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::cmp::Ordering as CmpOrdering;
use summoner_core::cadence_bus::{CadenceBus, CadenceResolutionType, ChordQuality};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Functional harmonic role in tonal cadences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HarmonicFunction {
    /// Rest and tonal foundation (I, vi, iii).
    #[default]
    Tonic,
    /// Motion away from tonic (IV, ii, viio/V).
    Subdominant,
    /// Active tension demanding resolution (V, V7, viio).
    Dominant,
    /// Chromatic preparation (Neapolitan bII, Augmented 6th, Secondary Dominant).
    PreDominant,
}

/// A node representing a chord state within the harmonic transition graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarmonicNode {
    /// Scale degree or semitone root relative to tonic (0 = Tonic / C).
    pub root_step: i32,
    /// Chord quality.
    pub quality: ChordQuality,
    /// Roman numeral / chord symbol label.
    pub roman_numeral: String,
    /// Functional harmonic role.
    pub function: HarmonicFunction,
    /// Inherent harmonic tension level [0.0 = total rest ..= 1.0 = maximum tension].
    pub tension: f32,
}

impl HarmonicNode {
    pub fn new(
        root_step: i32,
        quality: ChordQuality,
        roman_numeral: impl Into<String>,
        function: HarmonicFunction,
        tension: f32,
    ) -> Self {
        Self {
            root_step: root_step.rem_euclid(12),
            quality,
            roman_numeral: roman_numeral.into(),
            function,
            tension: tension.clamp(0.0, 1.0),
        }
    }
}

/// A directed edge in the harmonic transition graph representing musical chord motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CadenceEdge {
    pub from_node_idx: usize,
    pub to_node_idx: usize,
    /// Base transition weight/cost (lower cost = more natural/idiomatic progression).
    pub cost: f32,
    /// Type of cadence formed by this transition.
    pub cadence_type: CadenceResolutionType,
    /// Harmonic tension change delta.
    pub tension_delta: f32,
}

/// A discrete step along a resolved cadence path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarmonicPathStep {
    pub node_idx: usize,
    pub root_step: i32,
    pub quality: ChordQuality,
    pub roman_numeral: String,
    pub tension: f32,
    pub voice_leading_cost: f32,
    pub cadence_type: CadenceResolutionType,
    pub voicing: [i32; 4],
}

/// Graph of harmonic chord states and directed transitions for tonal cadence resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CadenceGraph {
    pub nodes: Vec<HarmonicNode>,
    pub edges: Vec<CadenceEdge>,
}

impl Default for CadenceGraph {
    fn default() -> Self {
        Self::build_tonal_diatonic_graph()
    }
}

impl CadenceGraph {
    /// Creates an empty harmonic graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Builds a comprehensive tonal diatonic & chromatic cadence transition graph.
    pub fn build_tonal_diatonic_graph() -> Self {
        let mut graph = Self::new();

        // Diatonic major scale nodes (C Major reference: root 0 = C)
        let idx_i = graph.add_node(HarmonicNode::new(0, ChordQuality::Major, "I", HarmonicFunction::Tonic, 0.05));
        let idx_ii = graph.add_node(HarmonicNode::new(2, ChordQuality::Minor, "ii", HarmonicFunction::Subdominant, 0.40));
        let idx_ii7 = graph.add_node(HarmonicNode::new(2, ChordQuality::Minor7, "ii7", HarmonicFunction::Subdominant, 0.50));
        let idx_iii = graph.add_node(HarmonicNode::new(4, ChordQuality::Minor, "iii", HarmonicFunction::Tonic, 0.30));
        let idx_iv = graph.add_node(HarmonicNode::new(5, ChordQuality::Major, "IV", HarmonicFunction::Subdominant, 0.35));
        let idx_v = graph.add_node(HarmonicNode::new(7, ChordQuality::Major, "V", HarmonicFunction::Dominant, 0.75));
        let idx_v7 = graph.add_node(HarmonicNode::new(7, ChordQuality::Dominant7, "V7", HarmonicFunction::Dominant, 0.90));
        let idx_vi = graph.add_node(HarmonicNode::new(9, ChordQuality::Minor, "vi", HarmonicFunction::Tonic, 0.25));
        let idx_viio = graph.add_node(HarmonicNode::new(11, ChordQuality::Diminished, "vii°", HarmonicFunction::Dominant, 0.85));

        // Chromatic & secondary dominant nodes
        let idx_v_v = graph.add_node(HarmonicNode::new(2, ChordQuality::Dominant7, "V7/V", HarmonicFunction::PreDominant, 0.80));
        let idx_neapolitan = graph.add_node(HarmonicNode::new(1, ChordQuality::Major, "N6 (bII)", HarmonicFunction::PreDominant, 0.70));
        let idx_ger6 = graph.add_node(HarmonicNode::new(8, ChordQuality::Dominant7, "Ger+6", HarmonicFunction::PreDominant, 0.88));
        let idx_iv_min = graph.add_node(HarmonicNode::new(5, ChordQuality::Minor, "iv (minor)", HarmonicFunction::Subdominant, 0.60));

        // Authentic cadences (Dominant -> Tonic)
        graph.add_edge(idx_v, idx_i, 1.0, CadenceResolutionType::Authentic, -0.70);
        graph.add_edge(idx_v7, idx_i, 0.8, CadenceResolutionType::Authentic, -0.85);
        graph.add_edge(idx_viio, idx_i, 1.2, CadenceResolutionType::Authentic, -0.80);

        // Plagal cadences (Subdominant -> Tonic)
        graph.add_edge(idx_iv, idx_i, 1.4, CadenceResolutionType::Plagal, -0.30);
        graph.add_edge(idx_iv_min, idx_i, 1.2, CadenceResolutionType::Plagal, -0.55);
        graph.add_edge(idx_ii, idx_i, 2.0, CadenceResolutionType::Plagal, -0.35);

        // Deceptive cadences (Dominant -> Submediant / Subdominant)
        graph.add_edge(idx_v, idx_vi, 1.8, CadenceResolutionType::Deceptive, -0.50);
        graph.add_edge(idx_v7, idx_vi, 1.6, CadenceResolutionType::Deceptive, -0.65);
        graph.add_edge(idx_v, idx_iv, 2.5, CadenceResolutionType::Deceptive, -0.40);

        // Half cadences (Tonic/Subdominant -> Dominant)
        graph.add_edge(idx_i, idx_v, 1.5, CadenceResolutionType::Half, 0.70);
        graph.add_edge(idx_iv, idx_v, 1.1, CadenceResolutionType::Half, 0.40);
        graph.add_edge(idx_ii, idx_v, 1.0, CadenceResolutionType::Half, 0.35);
        graph.add_edge(idx_ii7, idx_v7, 0.9, CadenceResolutionType::Half, 0.40);
        graph.add_edge(idx_vi, idx_v, 1.8, CadenceResolutionType::Half, 0.50);

        // Subdominant preparation (Tonic -> Subdominant)
        graph.add_edge(idx_i, idx_iv, 1.2, CadenceResolutionType::Half, 0.30);
        graph.add_edge(idx_i, idx_ii, 1.3, CadenceResolutionType::Half, 0.35);
        graph.add_edge(idx_i, idx_vi, 1.4, CadenceResolutionType::Half, 0.20);
        graph.add_edge(idx_i, idx_iii, 1.6, CadenceResolutionType::Half, 0.25);
        graph.add_edge(idx_iii, idx_vi, 1.1, CadenceResolutionType::Half, -0.05);
        graph.add_edge(idx_iii, idx_iv, 1.3, CadenceResolutionType::Half, 0.05);
        graph.add_edge(idx_vi, idx_ii, 1.1, CadenceResolutionType::Half, 0.15);
        graph.add_edge(idx_vi, idx_iv, 1.2, CadenceResolutionType::Half, 0.10);
        graph.add_edge(idx_iv, idx_ii, 1.3, CadenceResolutionType::Half, 0.05);

        // Chromatic pre-dominant routing
        graph.add_edge(idx_i, idx_neapolitan, 2.2, CadenceResolutionType::Half, 0.65);
        graph.add_edge(idx_neapolitan, idx_v7, 1.1, CadenceResolutionType::Half, 0.20);
        graph.add_edge(idx_iv, idx_ger6, 1.5, CadenceResolutionType::Half, 0.53);
        graph.add_edge(idx_ger6, idx_v7, 1.0, CadenceResolutionType::Half, 0.02);
        graph.add_edge(idx_ii, idx_v_v, 1.3, CadenceResolutionType::Half, 0.40);
        graph.add_edge(idx_v_v, idx_v7, 0.9, CadenceResolutionType::Modulatory, 0.10);
        graph.add_edge(idx_iv, idx_iv_min, 1.2, CadenceResolutionType::Half, 0.25);

        graph
    }

    /// Adds a harmonic node to the graph and returns its index.
    pub fn add_node(&mut self, node: HarmonicNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Adds a directed edge between nodes.
    pub fn add_edge(
        &mut self,
        from_node_idx: usize,
        to_node_idx: usize,
        cost: f32,
        cadence_type: CadenceResolutionType,
        tension_delta: f32,
    ) {
        self.edges.push(CadenceEdge {
            from_node_idx,
            to_node_idx,
            cost,
            cadence_type,
            tension_delta,
        });
    }

    /// Finds node index matching root step and quality.
    pub fn find_node(&self, root_step: i32, quality: ChordQuality) -> Option<usize> {
        let r = root_step.rem_euclid(12);
        self.nodes.iter().position(|n| n.root_step == r && n.quality == quality)
    }

    /// Computes circle-of-fifths distance in semitones (0 to 6 fifths).
    pub fn circle_of_fifths_distance(root_a: i32, root_b: i32) -> u32 {
        let diff = (root_b - root_a).rem_euclid(12);
        // (diff * 7) mod 12 gives fifths count
        let fifths = (diff * 7) % 12;
        fifths.min(12 - fifths) as u32
    }
}

/// Dijkstra node entry for path search.
#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f32,
    node: usize,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Reverse for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(CmpOrdering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

/// Voice-Leading Optimizer for 4-part SATB keyboard voicings.
#[derive(Debug, Clone, Default)]
pub struct VoiceLeadingOptimizer;

impl VoiceLeadingOptimizer {
    /// Generates standard 4-voice closed voicing for a root step and chord quality.
    pub fn generate_initial_voicing(root_step: i32, quality: ChordQuality) -> [i32; 4] {
        let root = root_step.rem_euclid(12);
        let bass = 36 + root; // Bass in C2-B2 octave

        let (third, fifth, seventh) = match quality {
            ChordQuality::Major => (4, 7, 12),
            ChordQuality::Minor => (3, 7, 12),
            ChordQuality::Dominant7 => (4, 7, 10),
            ChordQuality::Major7 => (4, 7, 11),
            ChordQuality::Minor7 => (3, 7, 10),
            ChordQuality::Diminished => (3, 6, 12),
            ChordQuality::HalfDiminished => (3, 6, 10),
            ChordQuality::Augmented => (4, 8, 12),
            ChordQuality::Suspended4 => (5, 7, 12),
            ChordQuality::Suspended2 => (2, 7, 12),
            ChordQuality::Diminished7 => (3, 6, 9),
            ChordQuality::AlteredDominant => (4, 8, 10),
        };

        let tenor = 48 + root;
        let alto = 48 + root + third;
        let soprano = 48 + root + fifth;

        [bass, tenor, alto, soprano.min(72 + seventh)]
    }

    /// Computes voice-leading cost between two voicings including smooth motion and parallel fifths penalty.
    pub fn compute_voice_leading_cost(prev: &[i32; 4], next: &[i32; 4]) -> f32 {
        let mut total_displacement = 0.0f32;
        for i in 0..4 {
            let diff = (next[i] - prev[i]).abs() as f32;
            total_displacement += diff;
        }

        // Penalty for parallel fifths / octaves
        let mut parallel_penalty = 0.0f32;
        for i in 0..3 {
            for j in (i + 1)..4 {
                let interval_prev = (prev[j] - prev[i]).rem_euclid(12);
                let interval_next = (next[j] - next[i]).rem_euclid(12);
                let motion_i = next[i] - prev[i];
                let motion_j = next[j] - prev[j];

                // If both voices move in same direction into octave (0) or fifth (7)
                if motion_i != 0 && motion_i == motion_j && (interval_next == 0 || interval_next == 7) && interval_prev == interval_next {
                    parallel_penalty += 4.0;
                }
            }
        }

        total_displacement + parallel_penalty
    }

    /// Finds the smoothest voice-leading transition from `prev_voicing` to target chord.
    pub fn optimize_next_voicing(prev_voicing: &[i32; 4], next_root: i32, next_quality: ChordQuality) -> ([i32; 4], f32) {
        let r = next_root.rem_euclid(12);
        let bass = 36 + r;

        let chord_pitch_classes = match next_quality {
            ChordQuality::Major => [r, (r + 4) % 12, (r + 7) % 12, (r + 4) % 12],
            ChordQuality::Minor => [r, (r + 3) % 12, (r + 7) % 12, (r + 3) % 12],
            ChordQuality::Dominant7 => [r, (r + 4) % 12, (r + 7) % 12, (r + 10) % 12],
            ChordQuality::Major7 => [r, (r + 4) % 12, (r + 7) % 12, (r + 11) % 12],
            ChordQuality::Minor7 => [r, (r + 3) % 12, (r + 7) % 12, (r + 10) % 12],
            ChordQuality::Diminished => [r, (r + 3) % 12, (r + 6) % 12, (r + 3) % 12],
            ChordQuality::HalfDiminished => [r, (r + 3) % 12, (r + 6) % 12, (r + 10) % 12],
            ChordQuality::Augmented => [r, (r + 4) % 12, (r + 8) % 12, (r + 4) % 12],
            ChordQuality::Suspended4 => [r, (r + 5) % 12, (r + 7) % 12, (r + 5) % 12],
            ChordQuality::Suspended2 => [r, (r + 2) % 12, (r + 7) % 12, (r + 2) % 12],
            ChordQuality::Diminished7 => [r, (r + 3) % 12, (r + 6) % 12, (r + 9) % 12],
            ChordQuality::AlteredDominant => [r, (r + 4) % 12, (r + 8) % 12, (r + 10) % 12],
        };

        let mut best_voicing = [bass, 48 + chord_pitch_classes[0], 48 + chord_pitch_classes[1], 48 + chord_pitch_classes[2]];
        let mut min_cost = f32::MAX;

        // Try octave permutations for upper 3 voices to minimize distance to prev_voicing
        for o1 in 4..=6 {
            let v1 = o1 * 12 + chord_pitch_classes[1];
            for o2 in 4..=6 {
                let v2 = o2 * 12 + chord_pitch_classes[2];
                for o3 in 4..=6 {
                    let v3 = o3 * 12 + chord_pitch_classes[3];
                    if v1 <= v2 && v2 <= v3 {
                        let candidate = [bass, v1, v2, v3];
                        let cost = Self::compute_voice_leading_cost(prev_voicing, &candidate);
                        if cost < min_cost {
                            min_cost = cost;
                            best_voicing = candidate;
                        }
                    }
                }
            }
        }

        (best_voicing, min_cost)
    }
}

/// Dynamic Cadence Path Resolver engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CadencePathResolver {
    pub graph: CadenceGraph,
}

impl Default for CadencePathResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CadencePathResolver {
    /// Creates a new resolver with default diatonic cadence transition graph.
    pub fn new() -> Self {
        Self {
            graph: CadenceGraph::build_tonal_diatonic_graph(),
        }
    }

    /// Resolves the optimal cadence progression path from a starting chord to a target chord.
    pub fn resolve_cadence_path(
        &self,
        start_root: i32,
        start_quality: ChordQuality,
        target_root: i32,
        target_quality: ChordQuality,
    ) -> Vec<HarmonicPathStep> {
        let start_idx = self.graph.find_node(start_root, start_quality).unwrap_or(0);
        let target_idx = self.graph.find_node(target_root, target_quality).unwrap_or(0);

        if start_idx == target_idx {
            let node = &self.graph.nodes[start_idx];
            let voicing = VoiceLeadingOptimizer::generate_initial_voicing(node.root_step, node.quality);
            return vec![HarmonicPathStep {
                node_idx: start_idx,
                root_step: node.root_step,
                quality: node.quality,
                roman_numeral: node.roman_numeral.clone(),
                tension: node.tension,
                voice_leading_cost: 0.0,
                cadence_type: CadenceResolutionType::Authentic,
                voicing,
            }];
        }

        // Dijkstra shortest path search on graph edges
        let n = self.graph.nodes.len();
        let mut dist = vec![f32::INFINITY; n];
        let mut prev = vec![None; n];
        let mut prev_edge = vec![None; n];
        let mut heap = BinaryHeap::new();

        dist[start_idx] = 0.0;
        heap.push(State { cost: 0.0, node: start_idx });

        while let Some(State { cost, node }) = heap.pop() {
            if node == target_idx {
                break;
            }
            if cost > dist[node] {
                continue;
            }

            for edge in &self.graph.edges {
                if edge.from_node_idx == node {
                    let next_node = edge.to_node_idx;
                    let next_cost = cost + edge.cost;
                    if next_cost < dist[next_node] {
                        dist[next_node] = next_cost;
                        prev[next_node] = Some(node);
                        prev_edge[next_node] = Some(edge.clone());
                        heap.push(State { cost: next_cost, node: next_node });
                    }
                }
            }
        }

        // Reconstruct path indices
        let mut path_node_indices = Vec::new();
        let mut curr = target_idx;
        while let Some(p) = prev[curr] {
            path_node_indices.push(curr);
            curr = p;
        }
        path_node_indices.push(start_idx);
        path_node_indices.reverse();

        // Generate voicings and compute step metadata
        let mut steps = Vec::new();
        let mut curr_voicing = VoiceLeadingOptimizer::generate_initial_voicing(
            self.graph.nodes[path_node_indices[0]].root_step,
            self.graph.nodes[path_node_indices[0]].quality,
        );

        for (i, &node_idx) in path_node_indices.iter().enumerate() {
            let node = &self.graph.nodes[node_idx];
            let (voicing, vl_cost) = if i == 0 {
                (curr_voicing, 0.0)
            } else {
                let (opt_voicing, cost) = VoiceLeadingOptimizer::optimize_next_voicing(&curr_voicing, node.root_step, node.quality);
                curr_voicing = opt_voicing;
                (opt_voicing, cost)
            };

            let cadence_type = if i > 0 {
                prev_edge[node_idx].as_ref().map(|e| e.cadence_type).unwrap_or(CadenceResolutionType::Authentic)
            } else {
                CadenceResolutionType::Authentic
            };

            steps.push(HarmonicPathStep {
                node_idx,
                root_step: node.root_step,
                quality: node.quality,
                roman_numeral: node.roman_numeral.clone(),
                tension: node.tension,
                voice_leading_cost: vl_cost,
                cadence_type,
                voicing,
            });
        }

        steps
    }

    /// Dispatches a single step along the resolved cadence path into `CadenceBus` and `ParamBus`.
    pub fn dispatch_step(
        &self,
        step: &HarmonicPathStep,
        step_index: usize,
        total_steps: usize,
        cadence_bus: &CadenceBus,
        param_bus: &ParamBus,
        base_param_id: ParamId,
    ) {
        let progress = if total_steps > 1 {
            (step_index as f32) / ((total_steps - 1) as f32)
        } else {
            1.0
        };

        cadence_bus.set_current_chord(step.root_step, step.quality);
        cadence_bus.set_harmonic_tension(step.tension);
        cadence_bus.set_voice_leading_cost(step.voice_leading_cost);
        cadence_bus.set_predicted_cadence(step.cadence_type);
        cadence_bus.set_path_status(progress, total_steps as u32);

        cadence_bus.dispatch_to_param_bus(param_bus, base_param_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_cadence_path_resolver_diatonic_ii_v_i() {
        let resolver = CadencePathResolver::new();
        // Resolve from ii (D minor, root 2) to I (C Major, root 0)
        let path = resolver.resolve_cadence_path(2, ChordQuality::Minor, 0, ChordQuality::Major);

        assert!(path.len() >= 2);
        assert_eq!(path[0].roman_numeral, "ii");
        assert_eq!(path.last().unwrap().roman_numeral, "I");

        // Intermediate step should be V or V7
        if path.len() == 3 {
            assert!(path[1].roman_numeral.contains('V'));
        }
    }

    #[test]
    fn test_cadence_path_resolver_neapolitan_to_tonic() {
        let resolver = CadencePathResolver::new();
        // Resolve from Neapolitan (Db Major, root 1) to I (C Major, root 0)
        let path = resolver.resolve_cadence_path(1, ChordQuality::Major, 0, ChordQuality::Major);

        assert!(path.len() >= 3);
        assert_eq!(path[0].roman_numeral, "N6 (bII)");
        assert_eq!(path.last().unwrap().roman_numeral, "I");
    }

    #[test]
    fn test_voice_leading_cost_optimization() {
        let v_init = VoiceLeadingOptimizer::generate_initial_voicing(7, ChordQuality::Dominant7); // G7
        let (v_next, cost) = VoiceLeadingOptimizer::optimize_next_voicing(&v_init, 0, ChordQuality::Major); // C

        assert!(cost.is_finite());
        assert!(cost > 0.0);
        // Bass note should be C2 (MIDI 36)
        assert_eq!(v_next[0], 36);
        // Upper voices should maintain close spacing
        assert!(v_next[1] >= 48);
        assert!(v_next[3] <= 76);
    }

    #[test]
    fn test_cadence_step_dispatch_zero_allocation() {
        let resolver = CadencePathResolver::new();
        let path = resolver.resolve_cadence_path(5, ChordQuality::Major, 0, ChordQuality::Major); // IV -> ... -> I
        let cadence_bus = CadenceBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..10 {
            param_bus.register(ParamId(200 + i), 0.0);
        }

        {
            let _guard = AllocGuard::new();
            for (idx, step) in path.iter().enumerate() {
                resolver.dispatch_step(step, idx, path.len(), &cadence_bus, &param_bus, ParamId(200));
            }
        }

        let snap = cadence_bus.snapshot();
        assert_eq!(snap.current_root_step, 0);
        assert_eq!(snap.current_quality, ChordQuality::Major);
        assert!((snap.path_progress - 1.0).abs() < 1e-4);
    }
}
