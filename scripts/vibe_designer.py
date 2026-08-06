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
        f"Your SOLE purpose is GUI DESIGN, UI layout, and UX behavior for the next logical GUI step. "
        f"To overcome the AI obstacle of 'visualizing' the final result, you MUST adhere to the following rules: "
        f"1. MENTAL WIREFRAMING: Before writing any code, draw a text-based/ASCII wireframe of the target layout in your thought process to establish a robust spatial map. "
        f"2. EXPLICIT SPATIAL MATH: Calculate paddings, margins, component dimensions, and coordinates explicitly. Do not guess; ensure elements will not overlap and flex/grid boundaries are strictly defined. "
        f"3. CROSS-OS COMPATIBILITY: Factor in differences across Windows, macOS, and Linux (e.g., DPI scaling, system fonts, text sizes, scrollbar width, window frame differences). "
        f"4. UX & ERGONOMICS: Ensure minimum hit targets (e.g., at least 44x44 points for touch/click), define clear hover/active states, and maintain strong visual contrast ratios. "
        f"5. COMPONENT ISOLATION: Build and modify UI components so they are modular and testable outside of the main monolithic application state if possible. "
        f"6. VERIFICATION: If the framework allows, write layout unit tests. Otherwise, explain exactly how a human should visually test the new layout. "
        f"7. FAN-OUT / DELEGATION: You must heavily utilize subagents to fan out your work. Delegate modular tasks (e.g., building isolated widgets, prototyping specific layout components, or writing isolated tests) to subagents concurrently to accelerate the design process. "
        f"Edit the code, verify compile/syntax checks, and commit all changes with a detailed commit message. "
        f"If all GUI tasks in {latest_roadmap} are complete, do not add unrelated backend features."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_designer_last.log",
        runner_title="Summoner Autonomous Vibe GUI Designer"
    )
