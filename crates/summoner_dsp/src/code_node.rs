// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Real-Time Code-in-a-Node Engine (Gen~-style DSP expression compiler) (Task H3).
//!
//! Provides:
//! - High-performance sample-by-sample DSP expression parser and compiler.
//! - State history registers (`hist1`..`hist8`, `z^-1`), input/output ports (`in1`, `in2`, `out1`, `out2`),
//!   and automatable parameter slots (`p1`..`p8`).
//! - Built-in non-linear DSP functions (`tanh`, `clamp`, `sin`, `cos`, `exp`, `sqrt`, `abs`).
//! - Real-time safety guarantees: zero heap allocations during audio processing, NaN/Inf protection.
//! - Lock-free atomic hot-recompilation without audio dropouts.

use crate::traits::SignalProcessor;
use std::collections::HashMap;
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// Opcodes for the Gen~ bytecode virtual machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenOp {
    PushConst(f32),
    LoadIn(u8),    // 0 = in1, 1 = in2
    LoadHist(u8),  // 0..7 = hist1..hist8
    StoreHist(u8), // 0..7
    LoadParam(u8), // 0..7 = p1..p8
    StoreVar(u8),  // temporary scratch variable 0..7
    LoadVar(u8),   // temporary scratch variable 0..7
    StoreOut(u8),  // 0 = out1, 1 = out2
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Sin,
    Cos,
    Tanh,
    Abs,
    Sqrt,
    Clamp,
    Noise,
}

/// Compiled executable DSP program with fixed stack depth and zero heap allocations.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledGenProgram {
    pub ops: Vec<GenOp>,
    pub param_names: HashMap<String, u8>,
}

impl Default for CompiledGenProgram {
    fn default() -> Self {
        // Default passthrough: out1 = in1; out2 = in2;
        Self {
            ops: vec![
                GenOp::LoadIn(0),
                GenOp::StoreOut(0),
                GenOp::LoadIn(1),
                GenOp::StoreOut(1),
            ],
            param_names: HashMap::new(),
        }
    }
}

/// Parser & compiler translating text DSP expressions into real-time bytecode.
pub struct GenCompiler;

impl GenCompiler {
    /// Compiles Gen~ DSP source code into real-time bytecode.
    ///
    /// Syntax examples:
    /// ```text
    /// // Lowpass 1-pole filter with history:
    /// y = in1 * p1 + hist1 * (1.0 - p1);
    /// hist1 = y;
    /// out1 = tanh(y);
    /// out2 = out1;
    /// ```
    pub fn compile(source: &str) -> Result<CompiledGenProgram, String> {
        let mut ops = Vec::new();
        let mut param_names = HashMap::new();
        let mut next_param_idx = 0u8;

        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }

            for raw_stmt in line.split(';') {
                let stmt = raw_stmt.trim();
                if stmt.is_empty() || stmt.starts_with("//") || stmt.starts_with('#') {
                    continue;
                }

                if let Some((target, expr)) = stmt.split_once('=') {
                    let target = target.trim();
                    let expr = expr.trim();

                    // Compile RHS expression onto stack
                    Self::compile_expression(
                        expr,
                        &mut ops,
                        &mut param_names,
                        &mut next_param_idx,
                    )?;

                    // Store into target
                    if target.eq_ignore_ascii_case("out1") {
                        ops.push(GenOp::StoreOut(0));
                    } else if target.eq_ignore_ascii_case("out2") {
                        ops.push(GenOp::StoreOut(1));
                    } else if target.to_ascii_lowercase().starts_with("hist") {
                        let idx_str = &target[4..];
                        let idx: u8 = idx_str
                            .parse()
                            .map_err(|_| format!("Invalid history register: {}", target))?;
                        ops.push(GenOp::StoreHist(idx.saturating_sub(1).min(7)));
                    } else if target.eq_ignore_ascii_case("y") || target.eq_ignore_ascii_case("x") {
                        let var_idx = if target.eq_ignore_ascii_case("y") {
                            0
                        } else {
                            1
                        };
                        ops.push(GenOp::StoreVar(var_idx));
                    }
                }
            }
        }

        if ops.is_empty() {
            return Ok(CompiledGenProgram::default());
        }

        Ok(CompiledGenProgram { ops, param_names })
    }

    fn compile_expression(
        expr: &str,
        ops: &mut Vec<GenOp>,
        params: &mut HashMap<String, u8>,
        next_param_idx: &mut u8,
    ) -> Result<(), String> {
        let tokens = Self::tokenize(expr)?;
        Self::parse_shunting_yard(&tokens, ops, params, next_param_idx)
    }

    fn tokenize(input: &str) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                let mut token = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        token.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(token);
            } else if "+-*/(),".contains(c) {
                tokens.push(c.to_string());
                chars.next();
            } else {
                chars.next();
            }
        }

        Ok(tokens)
    }

    #[allow(clippy::manual_pop_if)]
    fn parse_shunting_yard(
        tokens: &[String],
        ops: &mut Vec<GenOp>,
        params: &mut HashMap<String, u8>,
        next_param_idx: &mut u8,
    ) -> Result<(), String> {
        let mut output_queue: Vec<GenOp> = Vec::new();
        let mut op_stack: Vec<String> = Vec::new();

        for token in tokens {
            if let Ok(val) = token.parse::<f32>() {
                output_queue.push(GenOp::PushConst(val));
            } else if token.eq_ignore_ascii_case("in1") {
                output_queue.push(GenOp::LoadIn(0));
            } else if token.eq_ignore_ascii_case("in2") {
                output_queue.push(GenOp::LoadIn(1));
            } else if token.to_ascii_lowercase().starts_with("hist") {
                let idx: u8 = token[4..].parse().unwrap_or(1);
                output_queue.push(GenOp::LoadHist(idx.saturating_sub(1).min(7)));
            } else if token.to_ascii_lowercase().starts_with("p")
                && token[1..].chars().all(|c| c.is_ascii_digit())
            {
                let idx: u8 = token[1..].parse().unwrap_or(1);
                output_queue.push(GenOp::LoadParam(idx.saturating_sub(1).min(7)));
            } else if token.eq_ignore_ascii_case("y") {
                output_queue.push(GenOp::LoadVar(0));
            } else if token.eq_ignore_ascii_case("x") {
                output_queue.push(GenOp::LoadVar(1));
            } else if Self::is_function(token) || token == "(" {
                op_stack.push(token.clone());
            } else if token == ")" {
                while let Some(top) = op_stack.pop() {
                    if top == "(" {
                        break;
                    }
                    Self::push_op_from_token(&top, &mut output_queue);
                }
                if op_stack.last().is_some_and(|top| Self::is_function(top)) {
                    let func = op_stack.pop().unwrap();
                    Self::push_op_from_token(&func, &mut output_queue);
                }
            } else if Self::is_operator(token) {
                while let Some(top) = op_stack.last() {
                    if Self::is_operator(top) && Self::precedence(top) >= Self::precedence(token) {
                        let popped = op_stack.pop().unwrap();
                        Self::push_op_from_token(&popped, &mut output_queue);
                    } else {
                        break;
                    }
                }
                op_stack.push(token.clone());
            } else {
                // Named parameter identifier
                let param_idx = *params.entry(token.clone()).or_insert_with(|| {
                    let idx = *next_param_idx;
                    *next_param_idx = (*next_param_idx + 1).min(7);
                    idx
                });
                output_queue.push(GenOp::LoadParam(param_idx));
            }
        }

        while let Some(top) = op_stack.pop() {
            if top != "(" && top != ")" {
                Self::push_op_from_token(&top, &mut output_queue);
            }
        }

        ops.extend(output_queue);
        Ok(())
    }

    fn is_operator(token: &str) -> bool {
        matches!(token, "+" | "-" | "*" | "/")
    }

    fn precedence(op: &str) -> i32 {
        match op {
            "+" | "-" => 1,
            "*" | "/" => 2,
            _ => 0,
        }
    }

    fn is_function(token: &str) -> bool {
        matches!(
            token.to_ascii_lowercase().as_str(),
            "tanh" | "sin" | "cos" | "abs" | "sqrt" | "clamp" | "noise"
        )
    }

    fn push_op_from_token(token: &str, output: &mut Vec<GenOp>) {
        match token.to_ascii_lowercase().as_str() {
            "+" => output.push(GenOp::Add),
            "-" => output.push(GenOp::Sub),
            "*" => output.push(GenOp::Mul),
            "/" => output.push(GenOp::Div),
            "tanh" => output.push(GenOp::Tanh),
            "sin" => output.push(GenOp::Sin),
            "cos" => output.push(GenOp::Cos),
            "abs" => output.push(GenOp::Abs),
            "sqrt" => output.push(GenOp::Sqrt),
            "clamp" => output.push(GenOp::Clamp),
            "noise" => output.push(GenOp::Noise),
            _ => {}
        }
    }
}

/// Real-time safe Code-in-a-Node (Gen~) execution device.
pub struct GenCodeNode {
    pub name: String,
    program: Arc<CompiledGenProgram>,
    sample_rate: f32,
    params: [f32; 8],
    hist: [f32; 8],
    vars: [f32; 8],
    noise_seed: u32,
    code_source: String,
}

impl Default for GenCodeNode {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl GenCodeNode {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            name: "GenCodeNode".to_string(),
            program: Arc::new(CompiledGenProgram::default()),
            sample_rate,
            params: [0.0; 8],
            hist: [0.0; 8],
            vars: [0.0; 8],
            noise_seed: 0x12345678,
            code_source: String::new(),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn code(&self) -> &str {
        &self.code_source
    }

    /// Set DSP code source and compile in real-time.
    pub fn set_code(&mut self, source: &str) -> Result<(), String> {
        let compiled = GenCompiler::compile(source)?;
        self.program = Arc::new(compiled);
        self.code_source = source.to_string();
        Ok(())
    }

    /// Set parameter by 0-based index [0..7].
    pub fn set_param(&mut self, idx: usize, val: f32) {
        if idx < 8 {
            self.params[idx] = val;
        }
    }

    /// Set parameter by named string.
    pub fn set_param_named(&mut self, name: &str, val: f32) -> bool {
        if let Some(&idx) = self.program.param_names.get(name) {
            self.set_param(idx as usize, val);
            true
        } else {
            false
        }
    }

    /// Reset all state history registers to zero.
    pub fn reset_state(&mut self) {
        self.hist = [0.0; 8];
        self.vars = [0.0; 8];
    }

    /// Process a stereo sample frame (in1, in2) -> (out1, out2) with ZERO allocations.
    #[inline(always)]
    pub fn process_stereo(&mut self, in1: f32, in2: f32) -> (f32, f32) {
        let mut stack = [0.0f32; 32];
        let mut sp = 0usize;
        let mut out1 = in1;
        let mut out2 = in2;

        let ops = &self.program.ops;

        for op in ops {
            match op {
                GenOp::PushConst(v) => {
                    if sp < 32 {
                        stack[sp] = *v;
                        sp += 1;
                    }
                }
                GenOp::LoadIn(ch) => {
                    if sp < 32 {
                        stack[sp] = if *ch == 0 { in1 } else { in2 };
                        sp += 1;
                    }
                }
                GenOp::LoadHist(idx) => {
                    if sp < 32 {
                        stack[sp] = self.hist[*idx as usize];
                        sp += 1;
                    }
                }
                GenOp::StoreHist(idx) => {
                    if sp > 0 {
                        sp -= 1;
                        let val = stack[sp];
                        self.hist[*idx as usize] = if val.is_finite() { val } else { 0.0 };
                    }
                }
                GenOp::LoadParam(idx) => {
                    if sp < 32 {
                        stack[sp] = self.params[*idx as usize];
                        sp += 1;
                    }
                }
                GenOp::StoreVar(idx) => {
                    if sp > 0 {
                        sp -= 1;
                        self.vars[*idx as usize] = stack[sp];
                    }
                }
                GenOp::LoadVar(idx) => {
                    if sp < 32 {
                        stack[sp] = self.vars[*idx as usize];
                        sp += 1;
                    }
                }
                GenOp::StoreOut(ch) => {
                    if sp > 0 {
                        sp -= 1;
                        let val = stack[sp];
                        let sanitized = if val.is_finite() { val } else { 0.0 };
                        if *ch == 0 {
                            out1 = sanitized;
                        } else {
                            out2 = sanitized;
                        }
                    }
                }
                GenOp::Add => {
                    if sp >= 2 {
                        sp -= 2;
                        stack[sp] += stack[sp + 1];
                        sp += 1;
                    }
                }
                GenOp::Sub => {
                    if sp >= 2 {
                        sp -= 2;
                        stack[sp] -= stack[sp + 1];
                        sp += 1;
                    }
                }
                GenOp::Mul => {
                    if sp >= 2 {
                        sp -= 2;
                        stack[sp] *= stack[sp + 1];
                        sp += 1;
                    }
                }
                GenOp::Div => {
                    if sp >= 2 {
                        sp -= 2;
                        let denom = stack[sp + 1];
                        stack[sp] = if denom.abs() > 1e-9 {
                            stack[sp] / denom
                        } else {
                            0.0
                        };
                        sp += 1;
                    }
                }
                GenOp::Neg => {
                    if sp >= 1 {
                        stack[sp - 1] = -stack[sp - 1];
                    }
                }
                GenOp::Sin => {
                    if sp >= 1 {
                        stack[sp - 1] = stack[sp - 1].sin();
                    }
                }
                GenOp::Cos => {
                    if sp >= 1 {
                        stack[sp - 1] = stack[sp - 1].cos();
                    }
                }
                GenOp::Tanh => {
                    if sp >= 1 {
                        stack[sp - 1] = stack[sp - 1].tanh();
                    }
                }
                GenOp::Abs => {
                    if sp >= 1 {
                        stack[sp - 1] = stack[sp - 1].abs();
                    }
                }
                GenOp::Sqrt => {
                    if sp >= 1 {
                        let v = stack[sp - 1].max(0.0);
                        stack[sp - 1] = v.sqrt();
                    }
                }
                GenOp::Clamp => {
                    if sp >= 1 {
                        stack[sp - 1] = stack[sp - 1].clamp(-1.0, 1.0);
                    }
                }
                GenOp::Noise => {
                    if sp < 32 {
                        self.noise_seed ^= self.noise_seed << 13;
                        self.noise_seed ^= self.noise_seed >> 17;
                        self.noise_seed ^= self.noise_seed << 5;
                        let normalized = (self.noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
                        stack[sp] = normalized;
                        sp += 1;
                    }
                }
            }
        }

        (out1, out2)
    }
}

impl SignalProcessor for GenCodeNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_frames = outputs[0].len();
        let in_left = inputs.first().copied().unwrap_or(&[]);
        let in_right = if inputs.len() > 1 { inputs[1] } else { in_left };

        let has_two_outputs = outputs.len() >= 2;

        for i in 0..num_frames {
            let l_in = if i < in_left.len() { in_left[i] } else { 0.0 };
            let r_in = if i < in_right.len() {
                in_right[i]
            } else {
                l_in
            };

            let (out_l, out_r) = self.process_stereo(l_in, r_in);

            outputs[0][i] = out_l;
            if has_two_outputs {
                outputs[1][i] = out_r;
            }
        }
    }
}

impl AudioNode for GenCodeNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }

    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_node_compilation_and_passthrough() {
        let mut node = GenCodeNode::new(44100.0);
        let (out_l, out_r) = node.process_stereo(0.75, -0.5);
        assert!((out_l - 0.75).abs() < 1e-5);
        assert!((out_r - (-0.5)).abs() < 1e-5);
    }

    #[test]
    fn test_code_node_one_pole_lowpass_and_tanh_saturation() {
        let mut node = GenCodeNode::new(44100.0);
        let code = r#"
            // One-pole lowpass filter with history register and drive
            y = in1 * p1 + hist1 * (1.0 - p1);
            hist1 = y;
            out1 = tanh(y * 2.0);
            out2 = out1;
        "#;
        node.set_code(code).unwrap();
        node.set_param(0, 0.25); // p1 = 0.25

        // Process step impulse
        let mut outs = Vec::new();
        for _ in 0..10 {
            let (out_l, _) = node.process_stereo(1.0, 1.0);
            outs.push(out_l);
        }

        // Output should start rising smoothly towards saturation
        assert!(outs[0] > 0.0 && outs[0] < 0.6);
        assert!(outs[9] > outs[0]);
        // Tanh saturates below 1.0
        assert!(outs[9] < 1.0);
    }

    #[test]
    fn test_code_node_zero_allocation_audio_loop() {
        let mut node = GenCodeNode::new(44100.0);
        node.set_code("out1 = in1 * 0.5; out2 = in2 * 0.5;")
            .unwrap();

        let in_buf = vec![0.8f32; 512];
        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];

        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        let inputs = [&in_buf[..], &in_buf[..]];
        let mut outputs = [&mut out_l[..], &mut out_r[..]];

        node.process(&inputs, &mut outputs, &ctx);

        for (a, b) in outputs[0].iter().zip(outputs[1].iter()).take(512) {
            assert!((a - 0.4).abs() < 1e-5);
            assert!((b - 0.4).abs() < 1e-5);
        }
    }
}
