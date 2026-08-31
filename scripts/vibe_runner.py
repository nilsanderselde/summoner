#!/usr/bin/env python3
"""
Summoner DAW — Token-Optimized GUI & DSP Architecture Runner
Orchestrates incremental egui implementation with universal DSP parameter exposure.
"""

import sys
from vibe_core import run_vibe_loop

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

def build_prompt(latest_roadmap, step_num):
    return (
        f"You are the Lead Systems & GUI Architect for Summoner DAW (Rust + egui).\n"
        f"Your mission is to deliver a production-grade, intuitive GUI in `crates/summoner_gui` that matches the feature-completeness of the CLI, exposes EVERY DSP module, and remains accessible for novices while providing deep control for power users.\n\n"

        f"### 🛑 TOKEN ECONOMY & ATOMIC TURN RULES (CRITICAL)\n"
        f"1. **Do NOT generate new UI images** if reference mockups already exist in `design_docs/`.\n"
        f"2. **Work in One Discrete Atomic Slice per turn** (e.g., DSP Rack View, Node Inspector, Patch Matrix, Macro Header, Preset Bar). Do NOT try to build the entire app in one turn.\n"
        f"3. **Fast Verification**: Run `cargo check -p summoner_gui` and `cargo test -p summoner_gui` first. Only run full workspace tests when crossing crate boundaries.\n"
        f"4. **No Dead Ends**: Commit working increments immediately.\n\n"

        f"### 🎛️ CORE UX & ARCHITECTURE DIRECTIVES\n"
        f"**1. Zero CLI Left Behind (Expose ALL DSP Modules):**\n"
        f"- Query `crates/summoner_dsp` and `crates/summoner_core` to inventory all DSP nodes (oscillators, filters, envelopes, modulators, effects, master chain).\n"
        f"- Implement a universal `DspNodeUi` trait or parameter reflection view so EVERY node parameter is rendered with tactile egui controls (sliders, rotary knobs, toggles, dropdowns).\n"
        f"- No DSP function, routing option, or audio parameter should be accessible only via CLI.\n\n"

        f"**2. Two-Tier UX (Novice Simplicity + Pro Depth):**\n"
        f"- **Novice View (Top-Level Macro Strip & Presets):** Clean header with quick preset browsing, master gain, play/pause transport, and high-level macro knobs (e.g., Tone, Space, Punch).\n"
        f"- **Pro View (Collapsible Racks, Node Inspector, & Routing Matrix):** Click-to-expand modular rack drawer, surgical parameter inspectors, real-time signal scopes, and modular routing patch cords.\n\n"

        f"**3. egui Design & Theming:**\n"
        f"- Dark, modern, high-contrast palette with responsive spacing.\n"
        f"- Smooth drag-value knobs, color-coded audio vs. modulation signal lines, and clear visual hierarchy.\n"
        f"- Real-time audio engine state bindings without heap allocations on the audio thread.\n\n"

        f"### 📋 EXECUTION PRIORITY FOR TURN #{step_num}\n"
        f"1. Check the current compile status: `cargo check -p summoner_gui`.\n"
        f"2. Check existing GUI files in `crates/summoner_gui/src/` to identify missing DSP module views or broken layout bindings.\n"
        f"3. Implement the next focused GUI component (e.g., register unmapped DSP nodes into the rack/inspector UI).\n"
        f"4. Verify the build passes `cargo check -p summoner_gui`.\n"
        f"5. If complete, make a clean git commit with a clear summary message."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_runner_last.log",
        runner_title="Summoner DAW Progressive GUI & DSP Runner"
    )