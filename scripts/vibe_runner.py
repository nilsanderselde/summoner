#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe Runner (Real-Time JSON Streamed)
Runs agy in autonomous loop focusing on implementing next roadmap steps.
"""

from vibe_core import run_vibe_loop

def build_prompt(latest_roadmap):
    return (
        f"Identify and read the latest roadmap file in local/ ({latest_roadmap}) as authoritative. "
        f"If needed for context, review older roadmap files in local/. "
        f"Proceed with implementing the next incomplete logical step on {latest_roadmap}, "
        f"edit the code, run tests to verify, and commit all changes intermittently with clear, detailed commit messages after each verified step. "
        f"When calling `write_to_file` on workspace/source files, NEVER pass `ArtifactMetadata`. "
        f"If all tasks in {latest_roadmap} are complete, create a new roadmap file "
        f"local/ROADMAP_YYYYMMDD_HHMMSS.md for subsequent steps before finishing."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_last.log",
        runner_title="Summoner Autonomous Vibe Runner"
    )
