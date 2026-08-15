#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe GUI Designer (Real-Time JSON Streamed)
Runs agy in autonomous loop focusing solely on GUI design, UX/UI, layouts,
pixel coordinate math, cross-OS compatibility, and visual testing.
"""

from vibe_core import run_vibe_loop

def build_prompt(latest_roadmap):
    return (
        f"Identify and read the latest roadmap file in local/ ({latest_roadmap}) as authoritative. "
        f"Additionally, you MUST read the GUI architectural rules defined in local/rules/GUI_RULES.md (routed from local/DESIGN.md) to ensure compliance with the project's vector UI and component standards. "
        f"Your SOLE purpose is GUI DESIGN, UI layout, and UX behavior for the next logical GUI step. "
        f"To overcome the AI obstacle of 'visualizing' the final result, you MUST employ a Vision Agent approach and adhere to the following rules: "
        f"1. MENTAL & VISUAL WIREFRAMING: Before writing code, draw a spatial map/wireframe of the target layout. Establish exact layout dimensions and visual hierarchy. "
        f"2. EXPLICIT SPATIAL MATH: Calculate paddings, margins, component dimensions, and coordinates explicitly. Ensure elements do not overlap and flex/grid boundaries are strictly defined. "
        f"3. CROSS-OS COMPATIBILITY: Factor in differences across Windows, macOS, and Linux (e.g., DPI scaling, system fonts, text sizes, scrollbar width, window frame differences). "
        f"4. UX & ERGONOMICS: Ensure minimum hit targets (at least 44x44 points for touch/click), define clear hover/active states, and maintain strong visual contrast ratios (WCAG AA/AAA). "
        f"5. VISION AGENT & SCREENSHOT FEEDBACK LOOP: Employ a Vision Agent (delegate to multimodal subagent / browser subagent / vision review tasks) to visually inspect component renders, screenshots, or visual layout dumps. "
        f"   - Render or capture screenshots of the updated UI/widgets (e.g. into `scratch/renders/` or via headless browser / visual test export). "
        f"   - Have the vision agent inspect the render for visual defects: text clipping, uneven padding, misaligned grid lines, contrast issues, or clipped buttons. "
        f"   - Use the vision subagent's feedback to make precision adjustments to the code until the render passes visual audit. "
        f"6. COMPONENT ISOLATION: Build and modify UI components so they are modular and testable outside of the main monolithic application state if possible. "
        f"7. VERIFICATION: Write layout unit tests (`cargo test --workspace --features gui`). Verify compiler cleanliness (`cargo check`, `cargo clippy`). "
        f"8. FAN-OUT / DELEGATION: You must heavily utilize subagents (including specialized vision subagents) to fan out your work. Delegate modular tasks concurrently to accelerate the design process. "
        f"Edit the code, verify compile/syntax checks, run visual audits, and commit all changes with a clear, detailed commit message. "
        f"If all tasks in {latest_roadmap} are complete, create a new roadmap file local/ROADMAP_YYYYMMDD_HHMMSS.md containing the next phase of GUI-focused milestones before finishing. Do not add unrelated backend features."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_designer_last.log",
        runner_title="Summoner Autonomous Vibe GUI Designer"
    )
