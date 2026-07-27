// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::pattern::PatternClip;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationStrategy {
    Ratchet,
    Euclidean,
    Markov2ndOrder,
    CellularAutomata(u8),
}

pub struct GenerativeEngine {
    seed: u64,
}

impl Default for GenerativeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerativeEngine {
    pub fn new() -> Self {
        Self {
            seed: 0x876543219ABCDEF0,
        }
    }

    fn next_prng(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.seed >> 33) as f32 / 2147483648.0
    }

    /// Bjorklund algorithm for Euclidean rhythms: distribute `pulses` evenly over `steps`.
    pub fn euclidean_rhythm(pulses: u32, steps: u32) -> Vec<bool> {
        if steps == 0 {
            return Vec::new();
        }
        let pulses = pulses.min(steps);
        let mut pattern = vec![false; steps as usize];
        let mut count = 0;
        for i in 0..steps {
            count += pulses;
            if count >= steps {
                count -= steps;
                pattern[i as usize] = true;
            }
        }
        pattern
    }

    /// 1D Cellular Automata rhythm generator (e.g. Rule 90, Rule 30).
    pub fn cellular_automata_rhythm(rule: u8, initial_state: &[bool]) -> Vec<bool> {
        let len = initial_state.len();
        if len == 0 {
            return Vec::new();
        }
        let mut next = vec![false; len];
        for i in 0..len {
            let left = initial_state[(i + len - 1) % len];
            let center = initial_state[i];
            let right = initial_state[(i + 1) % len];
            
            let neighborhood = ((left as u8) << 2) | ((center as u8) << 1) | (right as u8);
            next[i] = (rule & (1 << neighborhood)) != 0;
        }
        next
    }

    /// Mutates an existing pattern clip using generative algorithms.
    pub fn mutate_pattern(
        &mut self,
        source: &PatternClip,
        strategy: MutationStrategy,
        mutation_amount: f32,
    ) -> PatternClip {
        let mut mutated = source.clone();
        mutated.name = format!("{}_mutated", source.name);

        match strategy {
            MutationStrategy::Ratchet => {
                for step in &mut mutated.steps {
                    if step.active && self.next_prng() < mutation_amount {
                        step.ratchet = if self.next_prng() > 0.5 { 2 } else { 4 };
                    }
                }
            }
            MutationStrategy::Euclidean => {
                let rhythm = Self::euclidean_rhythm(
                    (mutated.steps.len() as f32 * mutation_amount).max(1.0) as u32,
                    mutated.steps.len() as u32,
                );
                for (idx, active) in rhythm.into_iter().enumerate() {
                    if idx < mutated.steps.len() {
                        mutated.steps[idx].active = active;
                    }
                }
            }
            MutationStrategy::Markov2ndOrder => {
                let active_notes: Vec<f32> = mutated.steps.iter().filter(|s| s.active).map(|s| s.note).collect();
                if active_notes.len() >= 3 {
                    // Build 2nd order transition matrix
                    let mut transitions: HashMap<(u32, u32), Vec<f32>> = HashMap::new();
                    for win in active_notes.windows(3) {
                        let k = (win[0] as u32, win[1] as u32);
                        transitions.entry(k).or_default().push(win[2]);
                    }

                    for i in 2..mutated.steps.len() {
                        if mutated.steps[i].active && self.next_prng() < mutation_amount {
                            let k = (mutated.steps[i - 2].note as u32, mutated.steps[i - 1].note as u32);
                            if let Some(next_options) = transitions.get(&k) {
                                let idx = (self.next_prng() * next_options.len() as f32) as usize % next_options.len();
                                mutated.steps[i].note = next_options[idx];
                            }
                        }
                    }
                }
            }
            MutationStrategy::CellularAutomata(rule) => {
                let init: Vec<bool> = mutated.steps.iter().map(|s| s.active).collect();
                let ca = Self::cellular_automata_rhythm(rule, &init);
                for (idx, active) in ca.into_iter().enumerate() {
                    if idx < mutated.steps.len() {
                        mutated.steps[idx].active = active;
                    }
                }
            }
        }

        mutated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_rhythm_generation() {
        let pattern = GenerativeEngine::euclidean_rhythm(3, 8);
        assert_eq!(pattern.len(), 8);
        let pulse_count = pattern.iter().filter(|&&b| b).count();
        assert_eq!(pulse_count, 3);
    }

    #[test]
    fn test_cellular_automata_rule90() {
        let init = vec![false, false, true, false, false];
        let next = GenerativeEngine::cellular_automata_rhythm(90, &init);
        assert_eq!(next.len(), 5);
        assert_eq!(next, vec![false, true, false, true, false]);
    }
}
