#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe GUI Designer (Real-Time JSON Streamed)
Runs agy in autonomous loop focusing solely on GUI design, UX/UI, layouts,
pixel coordinate math, cross-OS compatibility, and visual testing.
"""

from vibe_core import run_vibe_loop

def build_prompt(latest_roadmap):
    return (
        f"Read roadmap {latest_roadmap} and GUI constraints in local/rules/GUI_RULES.md. "
        f"Your SOLE purpose is GUI DESIGN, UI layout, and UX behavior for the next logical step. "
        f"1. WIREFRAMING & MATH: Draw spatial maps before coding. Calculate paddings, margins, and exact hit targets (>=44x44pt). Ensure high contrast (WCAG AA). "
        f"2. CROSS-OS SCALING: Factor in DPI scaling, system fonts, and OS-specific window frames. "
        f"3. VISION FEEDBACK LOOP: You must visually inspect your work. "
        f"   - If no egui headless screenshot test harness exists, YOUR FIRST TASK is to build a minimal Rust test or Python script to render and capture the GUI component to a PNG in `scratch/renders/`. "
        f"   - Spawn a multimodal vision subagent (or use your native vision capabilities via `view_file` on the PNG) to inspect for: text clipping, uneven padding, misaligned grids, or contrast issues. "
        f"   - Iterate on code using this visual feedback until perfect. "
        f"4. FAN-OUT: Use subagents concurrently to accelerate prototyping or test writing. "
        f"5. INTERMITTENT VERIFIED COMMITS & TOOL RULES: Do NOT accumulate uncommitted changes. After implementing and visually/unit verifying each view, widget, or layout milestone, run tests/checks (`cargo check --workspace`, `cargo test --workspace --features gui`) and immediately `git commit` all modified files with a clear, detailed commit message describing the exact GUI changes. Commit intermittently after each unit of progress so uncommitted changes are never lost if quota runs out. When calling `write_to_file` on workspace/source files, NEVER pass `ArtifactMetadata`. "
        f"Once tasks in {latest_roadmap} are complete, create the next local/ROADMAP_YYYYMMDD_HHMMSS.md. Do not touch backend features."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_designer_last.log",
        runner_title="Summoner Autonomous Vibe GUI Designer"
    )
