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
        f"We are actively REDESIGNING the GUI from the ground up, departing from the text-heavy roadmap for UI work.\n"
        f"Instead of relying on local minimums in `{latest_roadmap}`, you must work from design first principles and visual references.\n\n"
        
        f"### 1. VISION-DRIVEN GUI REDESIGN (HIGHEST PRIORITY)\n"
        f"The current GUI is terrible, confusing, and broken. Your goal is to build an award-winning UX.\n"
        f"- **Mockups:** If no official design images exist in `design_docs/`, use the `generate_image` tool to ideate and create a sleek, premium, modern UI mockup (e.g. `award_winning_daw_gui.png`).\n"
        f"- **Implementation:** Use your vision capabilities (`view_file` on images) to analyze these mockups and translate them directly into `egui` code in `crates/summoner_gui`.\n"
        f"- **Verification:** Render headless snapshots of your `egui` implementation to `scratch/renders/` and visually compare them against the mockups. Do NOT rely purely on text prompts for layout.\n"
        f"Refer to `rules/GUI_RULES.md` if present.\n\n"

        f"### 2. OTHER DEPARTMENTS (If GUI is blocked or complete)\n"
        f"For non-GUI tasks, you may fall back to the roadmap `{latest_roadmap}`:\n"
        f"- [AUDIO / DSP]: Focus on DSP correctness, zero heap allocation (`AllocGuard`).\n"
        f"- [CORE / ENGINE]: Focus on lock-free buffers, DAG sorting.\n\n"
        
        f"### 3. LOCAL SKILLS & TOOL DIRECTIVES\n"
        f"- Utilize relevant Rust coding skills and patterns from `.agents/skills/`.\n"
        f"- NEVER pass `ArtifactMetadata` when editing workspace code files via `write_to_file` or `replace_file_content`.\n"
        f"- Run `cargo check --workspace` and `cargo test --workspace --features gui` to verify changes.\n"
        f"- Perform intermittent `git commit` followed by `git push` operations after verifying progress.\n\n"
        
        f"### 4. ROADMAP TRANSITION & WISHLIST\n"
        f"If you complete tasks from the roadmap, generate the next consolidated milestone file at `{next_roadmap}`.\n"
        f"ONLY IF the GUI redesign is 100% complete and all roadmap tasks are finished, you may consult `local/WISHLIST.md` for 'nice-to-have' tasks. Do not prioritize wishlist items over a fully working GUI and DSP."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_runner_last.log",
        runner_title="Summoner DAW Autonomous Engineering Runner"
    )