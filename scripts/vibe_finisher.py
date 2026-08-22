#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe Finisher (Real-Time JSON Streamed)
Runs agy in an autonomous loop focusing on:
1. GUI Polish & Ergonomics (visual design, layout math, cross-OS scaling, visual feedback loop).
2. Audio Output Quality Control & DSP Correctness (pure sine wave rendering, phase continuity,
   sample clamping/quantization, channel interleaving, zero allocation on audio thread).
3. End-to-end integration, bug fixing, test suite verification, and intermittent commits.
"""

from vibe_core import run_vibe_loop

def build_prompt(latest_roadmap):
    return (
        f"Read and adhere strictly to the latest roadmap file in local/ ({latest_roadmap}) as authoritative. "
        f"Review older roadmaps in local/ if needed for historical context. "
        f"Your mission as VIBE-FINISHER is twofold: GUI Polish & High-Priority Audio Output Quality Control.\n\n"
        f"1. GUI POLISH & ERGONOMICS (like vibe-designer):\n"
        f"   - Focus on GUI design, UX/UI layout, pixel coordinate math, minimum hit targets (>=44x44pt), high contrast (WCAG AA/AAA).\n"
        f"   - Factor in cross-OS UI scaling (Windows, macOS, Linux), DPI scaling, and system fonts.\n"
        f"   - Visual Feedback Loop: visually inspect GUI widgets via headless renders in `scratch/renders/` using `view_file` or multimodal subagents, eliminating clipping, misalignment, and contrast issues.\n\n"
        f"2. AUDIO OUTPUT QUALITY CONTROL & DSP INTEGRITY (CRITICAL PRIORITY):\n"
        f"   - Verify and fix audio rendering (e.g., rendering `simple_track.toml` or single/multi-cycle sine wave projects must produce a clean, pure, noiseless sine wave without glitching or buzzing).\n"
        f"   - Phase Accumulator State: Ensure oscillators maintain continuous mutable phase state (`phase: f32`/`f64`) across buffer calls rather than deriving phase solely from local buffer indices `i`. Wrap phase with modulo `% (2.0 * PI)` (or `% 1.0`). Maintain strict phase continuity across buffer chunks.\n"
        f"   - Float-to-PCM Quantization & Clamping: When writing 16-bit PCM WAV, clamp samples to [-1.0, 1.0] before scaling and rounding: `(sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16`. Ensure little-endian byte ordering.\n"
        f"   - Channel Layout & Interleaving: Ensure planar DSP buffers (`[Vec<f32>; 2]`) are properly interleaved (`[L0, R0, L1, R1, ...]`) when written to stereo WAV streams.\n"
        f"   - Crate & Writer Flushing: Ensure WAV encoder format specs match sample types and `finalize()` is called to update header chunk lengths.\n"
        f"   - Zero Allocations: Do not allocate memory on the audio processing thread.\n"
        f"   - Audio Verification: Render short test sine waves and analyze/verify sample values or waveform data to guarantee purity and zero distortion.\n\n"
        f"3. ROADMAP CONTINUITY & INTERMITTENT VERIFIED COMMITS:\n"
        f"   - Do NOT leave behind these audio polishing and quality control directives when transitioning or generating new roadmaps.\n"
        f"   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace --features gui` to ensure 0 warnings and 100% test pass rate.\n"
        f"   - Tool Usage: When calling `write_to_file` or `replace_file_content` on workspace/source files (e.g. `crates/...`), NEVER pass `ArtifactMetadata` (leave it omitted/empty), as `ArtifactMetadata` is strictly for brain artifacts.\n"
        f"   - Commit all changes intermittently with clear, detailed commit messages immediately after verifying each fix/milestone so work is never lost if quota runs out.\n"
        f"   - Once all tasks in {latest_roadmap} are complete, create the next `local/ROADMAP_YYYYMMDD_HHMMSS.md` preserving all GUI and audio quality mandates."
    )

if __name__ == "__main__":
    run_vibe_loop(
        build_prompt_fn=build_prompt,
        log_file_name="vibe_finisher_last.log",
        runner_title="Summoner Autonomous Vibe Finisher"
    )
