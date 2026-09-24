#!/usr/bin/env python3
"""
Summoner DAW — Token-Optimized GUI & DSP Architecture Runner
Orchestrates incremental egui implementation with universal DSP parameter exposure
and periodic consultation of PROGRESS.md for product shippability alignment.
"""

import sys
import os
import re
from vibe_core import run_vibe_loop, PROJECT_ROOT

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

SPRINT_INTERVAL = 3  # Consult and review PROGRESS.md every 3 turns

def get_progress_summary():
    """
    Extracts high-priority gaps and release readiness directives from PROGRESS.md.
    """
    progress_file = os.path.join(PROJECT_ROOT, "PROGRESS.md")
    if not os.path.exists(progress_file):
        return None

    try:
        with open(progress_file, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()
    except Exception:
        return None

    summary_items = []

    # Extract Scorecard or Executive Summary score
    m_score = re.search(r"\*\*Overall (?:Product )?Shippability Score:\s*([^\n\*]+)\*\*", content)
    score_str = m_score.group(1).strip() if m_score else "8.2 / 10"

    # Extract Critical Gaps
    gaps_match = re.search(r"## 5\.\s*Critical Gaps & Shippability Risks(.*?)(?:## 6|\Z)", content, re.DOTALL)
    if gaps_match:
        gaps_text = gaps_match.group(1)
        gap_headings = re.findall(r"###\s*(Gap \d+:[^\n]+)", gaps_text)
        for gh in gap_headings[:3]:  # Top 3 critical gaps
            summary_items.append(f"  * {gh}")

    return {
        "score": score_str,
        "gaps": summary_items,
        "path": "PROGRESS.md"
    }

def build_prompt(latest_roadmap, step_num):
    is_sprint_turn = (step_num % SPRINT_INTERVAL == 0 or step_num == 1)
    progress_info = get_progress_summary()

    sprint_guidance = ""
    if progress_info:
        gaps_formatted = "\n".join(progress_info["gaps"]) if progress_info["gaps"] else "  * Milestone 33: Real-Time Parameter Automation Bridge\n  * Codebase Ergonomics & Registry Modularization"
        if is_sprint_turn:
            sprint_guidance = (
                f"### 🧭 SPRINT ALIGNMENT & PROGRESS AUDIT (Turn #{step_num} — Sprint Review)\n"
                f"**PROGRESS.md is authoritative for product readiness and shippability goals (Current Score: {progress_info['score']}).**\n"
                f"- **Top Shippability Priorities Identified by Product Management:**\n"
                f"{gaps_formatted}\n"
                f"- **Sprint Action Items:**\n"
                f"  1. Read `PROGRESS.md` to ground your turn in user-facing shippability.\n"
                f"  2. Prioritize closing critical gaps (e.g. Milestone 33 live parameter bridge, reducing compile bottlenecks, or packaging factory presets) rather than accumulating unlinked DSP node sprawl.\n"
                f"  3. If this sprint completes an active roadmap milestone, resolves a gap, or alters module completeness, update `PROGRESS.md` accordingly.\n\n"
            )
        else:
            sprint_guidance = (
                f"### 🎯 PRODUCT SHIPPABILITY CONTEXT (PROGRESS.md — Score: {progress_info['score']})\n"
                f"- Active Target: v1.0 Production Readiness. Top priorities: Live parameter automation (M33) & UI ergonomics.\n"
                f"- Sprints consult and reconcile with `PROGRESS.md` every {SPRINT_INTERVAL} turns (Next sprint review: Turn #{step_num + (SPRINT_INTERVAL - (step_num % SPRINT_INTERVAL))}).\n\n"
            )

    return (
        f"You are the Lead Systems & GUI Architect for Summoner DAW (Rust + egui).\n"
        f"Your mission is to deliver a production-grade, intuitive GUI in `crates/summoner_gui` that matches the feature-completeness of the CLI, exposes EVERY DSP module, and remains accessible for novices while providing deep control for power users.\n\n"

        f"{sprint_guidance}"

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
        f"3. Implement the next focused GUI component (e.g., register unmapped DSP nodes into the rack/inspector UI, or advance Milestone 33 live parameter bridge).\n"
        f"4. Verify the build passes `cargo check -p summoner_gui`.\n"
        f"5. If complete, make a clean git commit with a clear summary message."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_runner_last.log",
        runner_title="Summoner DAW Progressive GUI & DSP Runner"
    )