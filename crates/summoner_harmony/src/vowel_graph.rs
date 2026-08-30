// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Phonetic Vowel Trajectory Interpolator & Dynamic Formant Coordinate Router (Milestone 15).
//!
//! Implements real-time International Phonetic Alphabet (IPA) vowel formant chart coordinate space
//! ($F_1/F_2/F_3/F_4$ mapping for /i/, /e/, /a/, /o/, /u/, /ə/, etc.) with Catmull-Rom cubic spline
//! trajectory smoothing, 44-cylinder cross-sectional area function synthesis, and lock-free
//! parameter dispatch into `FormantBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in audio loops under `AllocGuard`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use std::fmt;
use summoner_core::formant_bus::{FormantBus, IpaVowel};
use summoner_core::param_bus::{ParamBus, ParamId};

mod serde_cylinder_array {
    use super::*;

    pub fn serialize<S>(data: &[f32; 44], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(44))?;
        for elem in data.iter() {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; 44], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor;
        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = [f32; 44];
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of 44 f32 elements")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = [2.5f32; 44];
                let mut idx = 0;
                while let Some(val) = seq.next_element()? {
                    if idx < 44 {
                        arr[idx] = val;
                        idx += 1;
                    }
                }
                Ok(arr)
            }
        }
        deserializer.deserialize_seq(ArrayVisitor)
    }
}

/// Vowel formant coordinate entry in multi-dimensional acoustic and articulatory space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VowelNode {
    pub vowel: IpaVowel,
    pub symbol: String,
    pub formants_hz: [f32; 4],
    pub bandwidths_hz: [f32; 4],
    pub tongue_position: f32, // [0.0 = back ..= 1.0 = front]
    pub tongue_height: f32,   // [0.0 = open/low ..= 1.0 = close/high]
    pub lip_opening: f32,     // [0.1 = closed/rounded ..= 2.0 = spread/open]
    pub velum_opening: f32,   // [0.0 = oral ..= 1.0 = nasalized]
}

impl VowelNode {
    pub fn new(
        vowel: IpaVowel,
        formants_hz: [f32; 4],
        bandwidths_hz: [f32; 4],
        tongue_position: f32,
        tongue_height: f32,
        lip_opening: f32,
        velum_opening: f32,
    ) -> Self {
        Self {
            vowel,
            symbol: vowel.symbol().to_string(),
            formants_hz,
            bandwidths_hz,
            tongue_position: tongue_position.clamp(0.0, 1.0),
            tongue_height: tongue_height.clamp(0.0, 1.0),
            lip_opening: lip_opening.clamp(0.1, 2.5),
            velum_opening: velum_opening.clamp(0.0, 1.0),
        }
    }

    /// Normalized coordinate in IPA vowel chart (X: Backness 0.0=Back, 1.0=Front; Y: Height 0.0=Open, 1.0=Close).
    pub fn chart_coordinates(&self) -> (f32, f32) {
        let f1 = self.formants_hz[0].clamp(200.0, 950.0);
        let f2 = self.formants_hz[1].clamp(600.0, 2600.0);

        // F2 mapped to Backness (600 Hz = Back / 0.0, 2600 Hz = Front / 1.0)
        let x_backness = ((f2 - 600.0) / (2600.0 - 600.0)).clamp(0.0, 1.0);
        // F1 mapped to Height (900 Hz = Open / 0.0, 250 Hz = Close / 1.0)
        let y_height = ((900.0 - f1) / (900.0 - 250.0)).clamp(0.0, 1.0);

        (x_backness, y_height)
    }
}

/// Instantaneous interpolated state along a phonetic vowel trajectory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormantTrajectoryPoint {
    pub formants_hz: [f32; 4],
    pub bandwidths_hz: [f32; 4],
    pub tongue_position: f32,
    pub tongue_height: f32,
    pub lip_opening: f32,
    pub velum_opening: f32,
    pub nearest_vowel: IpaVowel,
    #[serde(with = "serde_cylinder_array")]
    pub cylinder_areas: [f32; 44],
}

/// Phonetic Vowel Graph & Catmull-Rom Trajectory Interpolator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VowelGraph {
    pub nodes: Vec<VowelNode>,
    pub waypoints: Vec<usize>, // Sequence of node indices forming active diphthong/morph trajectory
}

impl Default for VowelGraph {
    fn default() -> Self {
        Self::build_standard_ipa_graph()
    }
}

impl VowelGraph {
    /// Creates an empty vowel graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            waypoints: Vec::new(),
        }
    }

    /// Builds a comprehensive standard International Phonetic Alphabet (IPA) vowel space graph.
    pub fn build_standard_ipa_graph() -> Self {
        let mut graph = Self::new();

        // Standard cardinal & English/French vowels
        graph.add_node(VowelNode::new(
            IpaVowel::CloseFrontI,
            [270.0, 2290.0, 3010.0, 3500.0],
            [50.0, 90.0, 120.0, 150.0],
            0.92, 0.95, 1.25, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::CloseMidFrontE,
            [390.0, 2300.0, 2850.0, 3500.0],
            [60.0, 100.0, 130.0, 160.0],
            0.82, 0.75, 1.30, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::OpenMidFrontEps,
            [530.0, 1840.0, 2480.0, 3500.0],
            [70.0, 110.0, 140.0, 170.0],
            0.70, 0.55, 1.35, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::NearOpenFrontAsh,
            [660.0, 1720.0, 2410.0, 3500.0],
            [80.0, 120.0, 150.0, 180.0],
            0.60, 0.35, 1.45, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::OpenFrontA,
            [730.0, 1090.0, 2440.0, 3500.0],
            [90.0, 130.0, 160.0, 190.0],
            0.40, 0.15, 1.60, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::OpenBackA,
            [700.0, 1220.0, 2600.0, 3500.0],
            [85.0, 125.0, 155.0, 185.0],
            0.25, 0.15, 1.50, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::OpenMidBackO,
            [570.0, 840.0, 2410.0, 3500.0],
            [75.0, 115.0, 145.0, 175.0],
            0.20, 0.45, 0.70, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::CloseMidBackO,
            [460.0, 1100.0, 2500.0, 3500.0],
            [65.0, 105.0, 135.0, 165.0],
            0.15, 0.70, 0.50, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::CloseBackU,
            [300.0, 870.0, 2240.0, 3500.0],
            [55.0, 95.0, 125.0, 155.0],
            0.10, 0.90, 0.30, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::MidCentralSchwa,
            [500.0, 1500.0, 2500.0, 3500.0],
            [70.0, 110.0, 140.0, 170.0],
            0.50, 0.50, 1.00, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::CloseFrontY,
            [270.0, 1900.0, 2100.0, 3500.0],
            [50.0, 90.0, 120.0, 150.0],
            0.90, 0.95, 0.35, 0.0,
        ));
        graph.add_node(VowelNode::new(
            IpaVowel::CloseMidFrontOe,
            [400.0, 1600.0, 2250.0, 3500.0],
            [60.0, 100.0, 130.0, 160.0],
            0.75, 0.70, 0.45, 0.0,
        ));

        // Default waypoint trajectory: /i/ -> /e/ -> /a/ -> /o/ -> /u/
        graph.waypoints = vec![0, 1, 4, 7, 8];
        graph
    }

    /// Adds a vowel node to the graph and returns its index.
    pub fn add_node(&mut self, node: VowelNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Sets the trajectory waypoint node indices for smooth diphthong / morph transitions.
    pub fn set_waypoints(&mut self, waypoints: Vec<usize>) {
        if !waypoints.is_empty() {
            self.waypoints = waypoints;
        }
    }

    /// Finds vowel node index by IPA vowel enum.
    pub fn find_node(&self, vowel: IpaVowel) -> Option<usize> {
        self.nodes.iter().position(|n| n.vowel == vowel)
    }

    /// Evaluates 1D Catmull-Rom cubic spline interpolation between 4 control points $p_0, p_1, p_2, p_3$.
    #[inline]
    pub fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        0.5 * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
    }

    /// Evaluates smooth Catmull-Rom trajectory at normalized progress $t \in [0.0 ..= 1.0]$.
    pub fn evaluate_trajectory(&self, t: f32) -> FormantTrajectoryPoint {
        if self.nodes.is_empty() {
            return FormantTrajectoryPoint {
                formants_hz: [500.0, 1500.0, 2500.0, 3500.0],
                bandwidths_hz: [70.0, 110.0, 140.0, 170.0],
                tongue_position: 0.5,
                tongue_height: 0.5,
                lip_opening: 1.0,
                velum_opening: 0.0,
                nearest_vowel: IpaVowel::MidCentralSchwa,
                cylinder_areas: [2.5; 44],
            };
        }

        let num_w = self.waypoints.len();
        if num_w == 1 {
            let n = &self.nodes[self.waypoints[0].min(self.nodes.len() - 1)];
            let areas = Self::synthesize_cylinder_areas(n.tongue_position, n.tongue_height, n.lip_opening);
            return FormantTrajectoryPoint {
                formants_hz: n.formants_hz,
                bandwidths_hz: n.bandwidths_hz,
                tongue_position: n.tongue_position,
                tongue_height: n.tongue_height,
                lip_opening: n.lip_opening,
                velum_opening: n.velum_opening,
                nearest_vowel: n.vowel,
                cylinder_areas: areas,
            };
        }

        let clamped_t = t.clamp(0.0, 1.0);
        let num_segments = (num_w - 1) as f32;
        let segment_pos = clamped_t * num_segments;
        let seg_idx = (segment_pos.floor() as usize).min(num_w - 2);
        let seg_frac = segment_pos - seg_idx as f32;

        let i0 = if seg_idx == 0 { self.waypoints[0] } else { self.waypoints[seg_idx - 1] };
        let i1 = self.waypoints[seg_idx];
        let i2 = self.waypoints[seg_idx + 1];
        let i3 = if seg_idx + 2 < num_w { self.waypoints[seg_idx + 2] } else { self.waypoints[seg_idx + 1] };

        let n0 = &self.nodes[i0.min(self.nodes.len() - 1)];
        let n1 = &self.nodes[i1.min(self.nodes.len() - 1)];
        let n2 = &self.nodes[i2.min(self.nodes.len() - 1)];
        let n3 = &self.nodes[i3.min(self.nodes.len() - 1)];

        let mut formants_hz = [0.0f32; 4];
        let mut bandwidths_hz = [0.0f32; 4];
        for k in 0..4 {
            formants_hz[k] = Self::catmull_rom_1d(
                n0.formants_hz[k],
                n1.formants_hz[k],
                n2.formants_hz[k],
                n3.formants_hz[k],
                seg_frac,
            ).max(50.0);
            bandwidths_hz[k] = Self::catmull_rom_1d(
                n0.bandwidths_hz[k],
                n1.bandwidths_hz[k],
                n2.bandwidths_hz[k],
                n3.bandwidths_hz[k],
                seg_frac,
            ).max(20.0);
        }

        let tongue_pos = Self::catmull_rom_1d(n0.tongue_position, n1.tongue_position, n2.tongue_position, n3.tongue_position, seg_frac).clamp(0.0, 1.0);
        let tongue_height = Self::catmull_rom_1d(n0.tongue_height, n1.tongue_height, n2.tongue_height, n3.tongue_height, seg_frac).clamp(0.0, 1.0);
        let lip_opening = Self::catmull_rom_1d(n0.lip_opening, n1.lip_opening, n2.lip_opening, n3.lip_opening, seg_frac).clamp(0.1, 2.5);
        let velum_opening = Self::catmull_rom_1d(n0.velum_opening, n1.velum_opening, n2.velum_opening, n3.velum_opening, seg_frac).clamp(0.0, 1.0);

        let nearest_vowel = if seg_frac < 0.5 { n1.vowel } else { n2.vowel };
        let cylinder_areas = Self::synthesize_cylinder_areas(tongue_pos, tongue_height, lip_opening);

        FormantTrajectoryPoint {
            formants_hz,
            bandwidths_hz,
            tongue_position: tongue_pos,
            tongue_height,
            lip_opening,
            velum_opening,
            nearest_vowel,
            cylinder_areas,
        }
    }

    /// Synthesizes 44-cylinder cross-sectional areas from tongue position, height, and lip opening.
    pub fn synthesize_cylinder_areas(tongue_pos: f32, tongue_height: f32, lip_opening: f32) -> [f32; 44] {
        let mut areas = [2.5f32; 44];
        let center_idx = 12.0 + tongue_pos.clamp(0.0, 1.0) * 24.0;
        let width = 7.0;
        let min_area = 0.30 + (1.0 - tongue_height.clamp(0.0, 1.0)) * 3.0;

        for (i, val) in areas.iter_mut().enumerate() {
            let dist = (i as f32 - center_idx).abs();
            if dist < width {
                let factor = (1.0 - dist / width).powi(2);
                let constricted = 3.0 * (1.0 - factor) + min_area * factor;
                *val = constricted.max(0.15);
            } else if i < 12 {
                // Pharyngeal widening/taper
                let t = i as f32 / 12.0;
                *val = 3.5 * (1.0 - t) + 2.5 * t;
            } else {
                *val = 2.8;
            }
        }

        // Apply lip opening to mouth exit
        for (i, area) in areas.iter_mut().enumerate().skip(38).take(6) {
            let t = (i - 38) as f32 / 6.0;
            *area = (*area * (1.0 - t + lip_opening * t)).clamp(0.1, 8.0);
        }

        areas
    }

    /// Dispatches an evaluated trajectory point into `FormantBus` and `ParamBus`.
    pub fn dispatch_trajectory_point(
        &self,
        t: f32,
        formant_bus: &FormantBus,
        param_bus: &ParamBus,
        base_param_id: ParamId,
    ) {
        let pt = self.evaluate_trajectory(t);

        formant_bus.set_active_vowel(pt.nearest_vowel);
        formant_bus.set_formants(pt.formants_hz[0], pt.formants_hz[1], pt.formants_hz[2], pt.formants_hz[3]);
        formant_bus.set_articulators(pt.tongue_position, pt.tongue_height, pt.lip_opening, pt.velum_opening);
        formant_bus.set_trajectory_progress(t, true);

        formant_bus.dispatch_to_param_bus(param_bus, base_param_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_vowel_graph_catmull_rom_smoothing_and_trajectory() {
        let graph = VowelGraph::build_standard_ipa_graph();
        assert!(graph.nodes.len() >= 10);

        let pt0 = graph.evaluate_trajectory(0.0);
        let pt_mid = graph.evaluate_trajectory(0.5);
        let pt1 = graph.evaluate_trajectory(1.0);

        assert!(pt0.formants_hz[0] > 100.0);
        assert!(pt_mid.formants_hz[0] > 100.0);
        assert!(pt1.formants_hz[0] > 100.0);

        assert_eq!(pt0.nearest_vowel, IpaVowel::CloseFrontI);
        assert_eq!(pt1.nearest_vowel, IpaVowel::CloseBackU);
    }

    #[test]
    fn test_vowel_chart_coordinates_normalized() {
        let node_i = VowelNode::new(
            IpaVowel::CloseFrontI,
            [270.0, 2290.0, 3010.0, 3500.0],
            [50.0, 90.0, 120.0, 150.0],
            0.92, 0.95, 1.25, 0.0,
        );
        let (x, y) = node_i.chart_coordinates();
        assert!(x > 0.70, "Vowel /i/ should be front (high X)");
        assert!(y > 0.80, "Vowel /i/ should be close (high Y)");
    }

    #[test]
    fn test_vowel_dispatch_zero_allocation() {
        let graph = VowelGraph::build_standard_ipa_graph();
        let formant_bus = FormantBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..15 {
            param_bus.register(ParamId(500 + i), 0.0);
        }

        {
            let _guard = AllocGuard::new();
            for step in 0..=10 {
                let t = step as f32 / 10.0;
                graph.dispatch_trajectory_point(t, &formant_bus, &param_bus, ParamId(500));
            }
        }

        let snap = formant_bus.snapshot();
        assert_eq!(snap.active_vowel, IpaVowel::CloseBackU);
        assert!(snap.morph_active);
    }
}
