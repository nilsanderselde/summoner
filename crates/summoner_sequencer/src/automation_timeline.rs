// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interpolation {
    Linear,
    Exponential,
    Bezier(f32, f32), // Control points
}

#[derive(Debug, Clone, Copy)]
pub struct AutomationPoint {
    pub beat: f64,
    pub value: f32,
    pub interp: Interpolation,
}

#[derive(Debug, Clone)]
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
            },
            Interpolation::Bezier(c1, c2) => {
                // Simplified cubic bezier over 1D value for ease of use
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
