#!/usr/bin/env python3
"""
Summoner DAW — Department Fan-Out Vibe Runner
Autonomous orchestration loop for agy CLI. Dispatches tasks to specialized
departments (Audio/DSP, GUI/Vision, Engine/Core, QA/Verification) with zero token waste.
"""

import os
import sys
from datetime import datetime
from vibe_core import run_vibe_loop

# Ensure stdout and stderr use UTF-8 on Windows
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

def build_prompt(latest_roadmap):
    current_ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    next_roadmap = f"local/ROADMAP_{current_ts}.md"
    
    return (
        f"You are the Lead Systems Architect and Engineering Director for Summoner DAW.\n"
        f"Read authoritative roadmap `{latest_roadmap}` and identify the next incomplete task.\n\n"
        
        f"### 1. DEPARTMENT FAN-OUT & SCOPE ROUTING\n"
        f"Identify the domain tag of the current task and follow ONLY its departmental mandate:\n"
        f"- [AUDIO / DSP]: Focus on DSP correctness, zero heap allocation in audio loops (`AllocGuard`), "
        f"continuous phase state accumulators (`phase % (2.0 * PI)`), float-to-PCM sample clamping "
        f"`[-1.0, 1.0]`, planar-to-interleaved stereo buffering, and WAV header finalization. "
        f"Refer to `rules/DSP_RULES.md` if present.\n"
        f"- [GUI / VISION]: Focus on `egui` native UI, layout math (8pt grid, >=44x44pt hit targets), "
        f"WCAG AA/AAA contrast, and cross-OS scaling. Render headless snapshots to `scratch/renders/` "
        f"and use `view_file` to visually verify layout alignment and contrast before completing. "
        f"Refer to `rules/GUI_RULES.md` if present.\n"
        f"- [CORE / ENGINE]: Focus on TOML parsing, lock-free buffers (`ParamBus`), `NodeGraph` DAG "
        f"topological sorting, and deterministic event dispatching. Refer to `rules/CORE_RULES.md`.\n"
        f"- [QA / VERIFICATION]: Run workspace audits, golden WAV regression tests, and cargo test suites.\n\n"
        
        f"### 2. LOCAL SKILLS & TOOL DIRECTIVES\n"
        f"- Utilize relevant Rust coding skills and patterns from `.agents/skills/`.\n"
        f"- NEVER pass `ArtifactMetadata` when editing workspace code files via `write_to_file` or `replace_file_content`.\n"
        f"- Run `cargo check --workspace` and `cargo test --workspace --features gui` to verify changes.\n"
        f"- Perform intermittent `git commit` operations immediately after verifying each unit of progress.\n\n"
        
        f"### 3. ROADMAP TRANSITION\n"
        f"Once all tasks in `{latest_roadmap}` are completed and verified, generate the next consolidated "
        f"milestone file at `{next_roadmap}` preserving all architectural invariants."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_runner_last.log",
        runner_title="Summoner DAW Autonomous Engineering Runner"
    )