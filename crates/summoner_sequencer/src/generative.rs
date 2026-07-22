// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use crate::pattern::PatternClip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationStrategy {
    Ratchet,
    Euclidean,
    Markov,
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
            MutationStrategy::Markov => {
                let notes: Vec<f32> = mutated.steps.iter().filter(|s| s.active).map(|s| s.note).collect();
                if !notes.is_empty() {
                    for step in &mut mutated.steps {
                        if step.active && self.next_prng() < mutation_amount {
                            let note_idx = (self.next_prng() * notes.len() as f32) as usize % notes.len();
                            step.note = notes[note_idx];
                        }
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
}

