// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::pattern::PatternClip;
use std::collections::HashMap;

/// Second-order Markov Chain note transition matrix.
#[derive(Debug, Clone, Default)]
pub struct MarkovChain2 {
    pub transitions: HashMap<(u8, u8), Vec<(u8, f32)>>,
}

impl MarkovChain2 {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
        }
    }

    pub fn train(&mut self, sequence: &[u8]) {
        if sequence.len() < 3 {
            return;
        }
        let mut counts: HashMap<(u8, u8), HashMap<u8, u32>> = HashMap::new();
        for win in sequence.windows(3) {
            let key = (win[0], win[1]);
            *counts.entry(key).or_default().entry(win[2]).or_insert(0) += 1;
        }

        self.transitions.clear();
        for (key, next_counts) in counts {
            let total: u32 = next_counts.values().sum();
            let mut cumulative = 0.0;
            let mut list = Vec::new();
            for (val, count) in next_counts {
                cumulative += count as f32 / total as f32;
                list.push((val, cumulative));
            }
            self.transitions.insert(key, list);
        }
    }

    pub fn generate(&self, seed: (u8, u8), length: usize, rng_seed: u64) -> Vec<u8> {
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length);
        result.push(seed.0);
        if length > 1 {
            result.push(seed.1);
        }

        let mut current_prng = rng_seed;
        let mut prng_next = || {
            current_prng = current_prng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (current_prng >> 33) as f32 / 2147483648.0
        };

        while result.len() < length {
            let n = result.len();
            let key = (result[n - 2], result[n - 1]);
            if let Some(list) = self.transitions.get(&key) {
                let r = prng_next();
                let mut picked = list.last().map(|(val, _)| *val).unwrap_or(0);
                for (val, cum) in list {
                    if r <= *cum {
                        picked = *val;
                        break;
                    }
                }
                result.push(picked);
            } else {
                let r = prng_next();
                if !self.transitions.is_empty() {
                    let keys: Vec<&(u8, u8)> = self.transitions.keys().collect();
                    let idx = (r * keys.len() as f32) as usize % keys.len();
                    result.push(keys[idx].0);
                } else {
                    result.push(seed.0);
                }
            }
        }

        result
    }
}

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
        Self::cellular_automata_multi_gen(initial_state, rule, 1)
    }

    /// 1D Cellular Automata rhythm generator over multiple generations using two fixed 128-bool ping-pong buffers.
    pub fn cellular_automata_multi_gen(initial: &[bool], rule: u8, generations: usize) -> Vec<bool> {
        let mut ping = [false; 128];
        let mut pong = [false; 128];

        let len = initial.len().min(128);
        if len == 0 {
            return Vec::new();
        }

        for i in 0..len {
            ping[i] = initial[i];
        }

        let mut use_ping = true;
        for _ in 0..generations {
            let (src, dst) = if use_ping { (&ping, &mut pong) } else { (&pong, &mut ping) };
            for i in 0..len {
                let left = src[(i + len - 1) % len];
                let center = src[i];
                let right = src[(i + 1) % len];

                let neighborhood = ((left as u8) << 2) | ((center as u8) << 1) | (right as u8);
                dst[i] = (rule & (1 << neighborhood)) != 0;
            }
            use_ping = !use_ping;
        }

        let active_buf = if use_ping { &ping } else { &pong };
        active_buf[..len].to_vec()
    }

    /// Mutates a sequence of pitch values using a 2nd order Markov chain.
    pub fn mutate_sequence_markov2(sequence: &[u8], steps: usize, rng_seed: u64) -> Vec<u8> {
        let mut model = MarkovChain2::new();
        model.train(sequence);
        let seed = if sequence.len() >= 2 {
            (sequence[0], sequence[1])
        } else if !sequence.is_empty() {
            (sequence[0], sequence[0])
        } else {
            (60, 60)
        };
        model.generate(seed, steps, rng_seed)
    }

    /// Applies a boolean rhythm mask to step configs.
    pub fn apply_rhythm_to_sequence(rhythm: &[bool], steps: &mut [summoner_project::schema::TrackerStepConfig]) {
        for (idx, &active) in rhythm.iter().enumerate() {
            if idx < steps.len() {
                steps[idx].gate = if active { 0.8 } else { 0.0 };
            }
        }
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
                let active_notes: Vec<u8> = mutated.steps.iter().filter(|s| s.active).map(|s| s.note as u8).collect();
                if active_notes.len() >= 3 {
                    let gen_notes = Self::mutate_sequence_markov2(&active_notes, mutated.steps.len(), self.seed);
                    for (i, &note) in gen_notes.iter().enumerate() {
                        if i < mutated.steps.len() {
                            mutated.steps[i].note = note as f32;
                        }
                    }
                }
            }
            MutationStrategy::CellularAutomata(rule) => {
                let init: Vec<bool> = mutated.steps.iter().map(|s| s.active).collect();
                let ca = Self::cellular_automata_multi_gen(&init, rule, 4);
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

    #[test]
    fn test_markov2_train_and_generate() {
        let mut model = MarkovChain2::new();
        let seq = vec![60, 62, 64, 60, 62, 65, 60, 62, 64];
        model.train(&seq);
        assert!(model.transitions.contains_key(&(60, 62)));

        let generated = model.generate((60, 62), 16, 42);
        assert_eq!(generated.len(), 16);
        assert_eq!(generated[0], 60);
        assert_eq!(generated[1], 62);
    }

    #[test]
    fn test_markov2_deterministic() {
        let seq = vec![60, 62, 64, 60, 62, 64, 60, 62, 64];
        let gen1 = GenerativeEngine::mutate_sequence_markov2(&seq, 10, 12345);
        let gen2 = GenerativeEngine::mutate_sequence_markov2(&seq, 10, 12345);
        assert_eq!(gen1, gen2);
    }

    #[test]
    fn test_rule30_deterministic() {
        let init = vec![false, false, true, false, false, false];
        let gen1 = GenerativeEngine::cellular_automata_multi_gen(&init, 30, 4);
        let gen2 = GenerativeEngine::cellular_automata_multi_gen(&init, 30, 4);
        assert_eq!(gen1, gen2);
    }

    #[test]
    fn test_cellular_automata_output_length_preserved() {
        let init = vec![true, false, true, false, true, true, false];
        let out = GenerativeEngine::cellular_automata_multi_gen(&init, 30, 8);
        assert_eq!(out.len(), init.len());
    }
}
