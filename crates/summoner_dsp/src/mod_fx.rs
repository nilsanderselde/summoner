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

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct EffectChorus { pub depth: f32, pub feedback: f32 }
impl EffectChorus { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectChorus {
    fn name(&self) -> &str { "EffectChorus" }
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}

pub struct EffectFlanger { pub depth: f32, pub feedback: f32 }
impl EffectFlanger { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectFlanger {
    fn name(&self) -> &str { "EffectFlanger" }
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}

pub struct EffectPhaser { pub depth: f32, pub feedback: f32 }
impl EffectPhaser { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectPhaser {
    fn name(&self) -> &str { "EffectPhaser" }
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}
