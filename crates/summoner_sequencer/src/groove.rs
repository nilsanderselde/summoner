// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Groove quantization engine supporting Funk, House, HipHop, and BossaNova feel templates.

use summoner_project::schema::TrackerStepConfig;

/// Groove template profiles for humanized micro-timing, swing, and velocity accenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrooveTemplate {
    Funk,
    House,
    HipHop,
    BossaNova,
}

/// Apply groove quantization template accents and micro-shifts to a slice of step configs.
pub fn apply_groove_quantize(steps: &mut [TrackerStepConfig], template: GrooveTemplate, amount: f32) {
    let amount = amount.clamp(0.0, 1.0);
    if amount == 0.0 || steps.is_empty() {
        return;
    }

    for (idx, step) in steps.iter_mut().enumerate() {
        if !step.active {
            continue;
        }

        match template {
            GrooveTemplate::Funk => {
                // Funk accent: push beats 2 & 4, add 16th-note swing
                if idx % 4 == 1 || idx % 4 == 3 {
                    step.swing = 0.5 * amount;
                    step.micro_shift = (5.0 * amount) as i32;
                }
                if idx % 4 == 1 {
                    step.velocity = (step.velocity + 0.15 * amount).min(1.0);
                }
            }
            GrooveTemplate::House => {
                // House accent: 4-on-the-floor kick emphasis, offbeat 16th micro-shift
                if idx % 4 == 0 {
                    step.velocity = (step.velocity + 0.20 * amount).min(1.0);
                    step.micro_shift = 0;
                } else if idx % 2 == 1 {
                    step.micro_shift = (3.0 * amount) as i32;
                    step.velocity = (step.velocity * (1.0 - 0.1 * amount)).max(0.1);
                }
            }
            GrooveTemplate::HipHop => {
                // HipHop accent: heavy 16th swing & laid-back snare micro-shift
                if idx % 2 == 1 {
                    step.swing = 0.65 * amount;
                    step.micro_shift = (12.0 * amount) as i32; // Dragged snare feel
                }
                if idx % 4 == 2 {
                    step.velocity = (step.velocity + 0.25 * amount).min(1.0);
                }
            }
            GrooveTemplate::BossaNova => {
                // BossaNova accent: syncopated cross-rhythm micro-timing
                if idx % 3 == 0 || idx % 5 == 0 {
                    step.velocity = (step.velocity + 0.18 * amount).min(1.0);
                    step.micro_shift = (-4.0 * amount) as i32; // Anticipated push
                } else {
                    step.micro_shift = (6.0 * amount) as i32;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groove_quantize_templates() {
        let mut steps = vec![
            TrackerStepConfig { note: 60.0, velocity: 0.7, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
            TrackerStepConfig { note: 62.0, velocity: 0.7, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
            TrackerStepConfig { note: 64.0, velocity: 0.7, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
            TrackerStepConfig { note: 65.0, velocity: 0.7, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
        ];

        apply_groove_quantize(&mut steps, GrooveTemplate::HipHop, 1.0);
        assert!(steps[1].swing > 0.0);
        assert!(steps[1].micro_shift > 0);

        apply_groove_quantize(&mut steps, GrooveTemplate::House, 1.0);
        assert!(steps[0].velocity > 0.7);
    }
}
