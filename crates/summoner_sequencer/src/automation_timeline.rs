use crate::automation::AutomationRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interpolation {
    Linear,
    Exponential,
    Bezier(f32, f32), // Control points
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AutomationPoint {
    pub beat: f64,
    pub value: f32,
    pub interp: Interpolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationCurve {
    pub points: Vec<AutomationPoint>,
}

impl AutomationCurve {
    pub fn new(mut points: Vec<AutomationPoint>) -> Self {
        points.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap());
        Self { points }
    }

    pub fn evaluate_at_beat(&self, beat: f64) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if beat <= self.points.first().unwrap().beat {
            return self.points.first().unwrap().value;
        }
        if beat >= self.points.last().unwrap().beat {
            return self.points.last().unwrap().value;
        }

        let mut p0 = &self.points[0];
        let mut p1 = &self.points[0];
        for i in 0..self.points.len() - 1 {
            if beat >= self.points[i].beat && beat < self.points[i + 1].beat {
                p0 = &self.points[i];
                p1 = &self.points[i + 1];
                break;
            }
        }

        let t = ((beat - p0.beat) / (p1.beat - p0.beat)) as f32;

        match p0.interp {
            Interpolation::Linear => p0.value + t * (p1.value - p0.value),
            Interpolation::Exponential => {
                let v0 = p0.value.max(0.0001);
                let v1 = p1.value.max(0.0001);
                v0 * (v1 / v0).powf(t)
            }
            Interpolation::Bezier(c1, c2) => {
                let u = 1.0 - t;
                let t2 = t * t;
                let u2 = u * u;
                let u3 = u2 * u;
                let t3 = t2 * t;
                u3 * p0.value + 3.0 * u2 * t * c1 + 3.0 * u * t2 * c2 + t3 * p1.value
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    pub param_id: String,
    pub curve: AutomationCurve,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AutomationTimeline {
    pub lanes: HashMap<String, AutomationLane>,
}

impl AutomationTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_lane(&mut self, lane: AutomationLane) {
        self.lanes.insert(lane.param_id.clone(), lane);
    }

    pub fn evaluate(&self, param_id: &str, beat: f64) -> Option<f32> {
        self.lanes.get(param_id).map(|lane| lane.curve.evaluate_at_beat(beat))
    }

    /// Evaluates all automation lanes at the specified beat position and updates registered parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use summoner_sequencer::automation_timeline::{AutomationTimeline, AutomationLane, AutomationCurve, AutomationPoint, Interpolation};
    /// use summoner_sequencer::automation::AutomationRegistry;
    ///
    /// let registry = AutomationRegistry::new();
    /// let curve = AutomationCurve::new(vec![
    ///     AutomationPoint { beat: 0.0, value: 0.0, interp: Interpolation::Linear },
    ///     AutomationPoint { beat: 4.0, value: 1.0, interp: Interpolation::Linear },
    /// ]);
    /// let mut timeline = AutomationTimeline::new();
    /// timeline.add_lane(AutomationLane { param_id: "cutoff".to_string(), curve });
    /// timeline.apply_beat(&registry, 2.0);
    /// ```
    pub fn apply_beat(&self, registry: &AutomationRegistry, beat: f64) {
        for (param_id, lane) in &self.lanes {
            if let Some(param) = registry.get_param(param_id) {
                let val = lane.curve.evaluate_at_beat(beat);
                param.set(val);
            }
        }
    }

    pub fn record_beat(&mut self, registry: &mut AutomationRegistry, beat: f64) {
        if registry.is_recording_all() {
            let dirty = registry.snapshot_dirty_params(0 /* unused frame */);
            for (param_id, value) in dirty {
                let lane = self.lanes.entry(param_id.clone()).or_insert_with(|| AutomationLane {
                    param_id: param_id.clone(),
                    curve: AutomationCurve { points: Vec::new() },
                });
                
                // Keep it sorted
                let point = AutomationPoint {
                    beat,
                    value,
                    interp: Interpolation::Linear,
                };
                
                match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&beat).unwrap()) {
                    Ok(idx) => lane.curve.points[idx] = point,
                    Err(idx) => lane.curve.points.insert(idx, point),
                }
            }
        }
    }

    pub fn to_configs(&self, track_id: u64, sample_rate: u32, bpm: f64) -> Vec<summoner_project::schema::AutomationLaneConfig> {
        let sec_per_beat = 60.0 / bpm.max(1.0);
        self.lanes.values().map(|lane| {
            let events = lane.curve.points.iter().map(|p| {
                let frame = (p.beat * sec_per_beat * sample_rate as f64) as u64;
                summoner_project::schema::AutomationEventConfig {
                    frame,
                    value: p.value,
                }
            }).collect();

            summoner_project::schema::AutomationLaneConfig {
                param_id: lane.param_id.clone(),
                track_id,
                events,
            }
        }).collect()
    }

    pub fn from_configs(configs: &[summoner_project::schema::AutomationLaneConfig], sample_rate: u32, bpm: f64) -> Self {
        let sec_per_beat = 60.0 / bpm.max(1.0);
        let mut timeline = Self::new();
        for config in configs {
            let points = config.events.iter().map(|evt| {
                let beat = (evt.frame as f64 / sample_rate as f64) / sec_per_beat;
                AutomationPoint {
                    beat,
                    value: evt.value,
                    interp: Interpolation::Linear,
                }
            }).collect();

            timeline.add_lane(AutomationLane {
                param_id: config.param_id.clone(),
                curve: AutomationCurve::new(points),
            });
        }
        timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_timeline_evaluate_and_apply() {
        let registry = AutomationRegistry::new();
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 0.0, value: 0.0, interp: Interpolation::Linear },
            AutomationPoint { beat: 4.0, value: 1.0, interp: Interpolation::Linear },
        ]);
        let mut timeline = AutomationTimeline::new();
        timeline.add_lane(AutomationLane { param_id: "cutoff".to_string(), curve });

        let val_mid = timeline.evaluate("cutoff", 2.0).unwrap();
        assert!((val_mid - 0.5).abs() < 1e-4);

        timeline.apply_beat(&registry, 2.0);
    }

    #[test]
    fn test_automation_record_all() {
        let mut registry = AutomationRegistry::new();
        let param = registry.register_param("cutoff", 0.0);
        
        let mut timeline = AutomationTimeline::new();
        
        registry.start_record_all();
        
        // Simulating parameter sweeping 0 -> 1 over 100 frames (let's say 4 beats)
        for i in 0..=100 {
            let beat = (i as f64 / 100.0) * 4.0;
            let value = i as f32 / 100.0;
            param.set(value);
            timeline.record_beat(&mut registry, beat);
        }
        
        registry.stop_record_all();
        
        let val_mid = timeline.evaluate("cutoff", 2.0).unwrap();
        assert!((val_mid - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_automation_timeline_to_from_configs() {
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 0.0, value: 0.0, interp: Interpolation::Linear },
            AutomationPoint { beat: 4.0, value: 1.0, interp: Interpolation::Linear },
        ]);
        let mut timeline = AutomationTimeline::new();
        timeline.add_lane(AutomationLane { param_id: "cutoff".to_string(), curve });

        let configs = timeline.to_configs(1, 44100, 120.0);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].events.len(), 2);

        let restored = AutomationTimeline::from_configs(&configs, 44100, 120.0);
        let val_mid = restored.evaluate("cutoff", 2.0).unwrap();
        assert!((val_mid - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_automation_toml_round_trip() {
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 0.0, value: 0.1, interp: Interpolation::Linear },
            AutomationPoint { beat: 1.0, value: 0.3, interp: Interpolation::Linear },
            AutomationPoint { beat: 2.0, value: 0.5, interp: Interpolation::Linear },
            AutomationPoint { beat: 3.0, value: 0.7, interp: Interpolation::Linear },
            AutomationPoint { beat: 4.0, value: 0.9, interp: Interpolation::Linear },
        ]);
        let mut timeline = AutomationTimeline::new();
        timeline.add_lane(AutomationLane { param_id: "res".to_string(), curve });

        let configs = timeline.to_configs(0, 44100, 120.0);
        let restored = AutomationTimeline::from_configs(&configs, 44100, 120.0);

        for beat in [0.0, 1.0, 2.0, 3.0, 4.0] {
            let orig = timeline.evaluate("res", beat).unwrap();
            let rest = restored.evaluate("res", beat).unwrap();
            assert!((orig - rest).abs() < 1e-4, "Mismatch at beat {}: orig={}, rest={}", beat, orig, rest);
        }
    }

    #[test]
    fn test_automation_beat_frame_conversion_120bpm() {
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 1.0, value: 1.0, interp: Interpolation::Linear },
        ]);
        let mut timeline = AutomationTimeline::new();
        timeline.add_lane(AutomationLane { param_id: "gain".to_string(), curve });

        let configs = timeline.to_configs(0, 44100, 120.0);
        assert_eq!(configs[0].events[0].frame, 22050);

        let restored = AutomationTimeline::from_configs(&configs, 44100, 120.0);
        let point_beat = restored.lanes.get("gain").unwrap().curve.points[0].beat;
        assert!((point_beat - 1.0).abs() < 1e-5);
    }
}

