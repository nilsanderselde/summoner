# Summoner DAW

[![Summoner CI](https://github.com/nilsanderselde/Summoner/actions/workflows/summoner-ci.yml/badge.svg)](https://github.com/nilsanderselde/Summoner/actions/workflows/summoner-ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**Summoner** is a deterministic, microtonal, headless-first Digital Audio Workstation (DAW) built in Rust. It combines low-latency real-time DSP, microtonal harmony engines (N-EDO, Scala `.scl`/`.kbm`), generative music sequencing, Git-backed project versioning, and an egui-based interface.

---

## Key Features

- **Headless-First Architecture**: Full CLI engine for rendering, pattern generation, auto-slicing, and preset loading (`summon play`, `summon render-wav`, `summon sfz-convert`, `summon generate-pattern`).
- **Low-Latency Audio & SIMD DSP**: Pre-allocated zero-alloc audio graph callbacks with SIMD-accelerated synthesis primitives (wide `f32x4`).
- **Microtonal & Harmonic Bus**: Dynamic N-EDO tuning systems, custom scales, pitch-class mapping, and real-time chord suggestion engines.
- **Git Micro-Commit Engine**: Native Git DAG tracking for every state mutation with built-in `undo`/`redo` and automated patch branch PR generation.
- **Generative Engines**: Higher-order Markov chains and 1D cellular automata (Rule 30, Rule 90) for rhythm and melody synthesis.
- **Modular GUI**: Cross-platform egui interface featuring Arranger, Node Graph Editor, Piano Roll, Macro Rack, Console Mixer, and Live Stage View.

---

## Quickstart

### Headless CLI

```bash
# Build the headless CLI
cargo build --release -p summon

# Initialize a new Summoner project
cargo run -p summon -- init my_project --bpm 120

# Render project to WAV
cargo run -p summon -- render-wav my_project/project.toml output.wav

# Play audio live
cargo run -p summon -- play my_project/project.toml
```

### Graphical Interface

```bash
# Launch GUI
cargo run -p summoner_gui --features gui
```

---

## Architecture

| Crate | Responsibilities |
|---|---|
| [`summoner_core`](file:///crates/summoner_core) | Core data structures, audio node traits, transport clock, memory allocator guard, voice pool. |
| [`summoner_dsp`](file:///crates/summoner_dsp) | SIMD DSP algorithms, oscillators, filters, modulators, sampler, granular synth, drum machine, effects. |
| [`summoner_harmony`](file:///crates/summoner_harmony) | Microtonal tuning (N-EDO, `.scl`), harmonic bus, chord suggestions, scale snapping. |
| [`summoner_project`](file:///crates/summoner_project) | Project schema, TOML serialization, Git micro-commit DAG, asset hashing. |
| [`summoner_sequencer`](file:///crates/summoner_sequencer) | Automation timeline, step sequencers, generative engines (Markov, Cellular Automata). |
| [`summoner_gui`](file:///crates/summoner_gui) | Graphical user interface views (Arranger, Node Graph, Piano Roll, Mixer, Stage View). |
| [`summon`](file:///crates/summon) | Headless CLI application and audio stream runtime. |

---

## License

Copyright (C) 2026 nilsanderselde.
Licensed under the [GNU Affero General Public License v3.0](https://www.gnu.org/licenses/agpl-3.0.html).
