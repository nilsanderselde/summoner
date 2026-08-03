#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe Finisher (Real-Time JSON Streamed)
Runs agy in autonomous loop focusing on completing, testing, connecting components/APIs,
and rendering UI components correctly.
"""

from vibe_core import run_vibe_loop

def build_prompt(latest_roadmap):
    return (
        f"Identify and read the latest roadmap file in local/ ({latest_roadmap}) as authoritative. "
        f"If needed for context, review older roadmap files in local/. "
        f"Focus strictly on COMPLETING, TESTING, CONNECTING components/APIs, ensuring UI displays correctly, "
        f"fixing bugs, and tying up loose ends so everything works seamlessly end-to-end. "
        f"Do NOT add new speculative features; instead prioritize fixing incomplete wiring, broken UI layouts, "
        f"unhandled errors, and missing tests. "
        f"Edit the code, run tests/checks to verify that everything works as expected, "
        f"and commit all changes with a clear, detailed commit message. "
        f"If all existing tasks in {latest_roadmap} are verified complete and working, update the roadmap status accordingly."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_finisher_last.log",
        runner_title="Summoner Autonomous Vibe Finisher"
    )
