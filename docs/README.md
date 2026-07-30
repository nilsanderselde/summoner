# Summoner DAW — Technical Documentation

Welcome to the official documentation for **Summoner DAW**, a headless-first, deterministic Digital Audio Workstation built in pure Rust (2021 edition) and licensed under AGPLv3.

---

## 🏛 Core Philosophy

1. **Determinism First:** Given identical project configurations, audio buffers, and seed states, `summon` MUST produce bit-identical rendered output across all operating systems.
2. **Real-time Audio Safety:** Zero heap allocation (`malloc`/`free`) and zero locking primitives (mutexes, sync channels) inside the DSP render loop (`process_block`).
3. **Headless Native:** The engine kernel is fully decoupled from rendering and UI. All functionality is accessible via the CLI daemon (`summon.exe`).
4. **Declarative State:** Complete session state is represented in human-readable, Git-friendly plain-text `.toml` documents.
5. **Integrated Version Control:** Every UI or CLI state mutation creates an atomic micro-commit via an embedded `libgit2` DAG engine, enabling non-destructive undo/redo and patch-to-PR workflows.

---

## 📚 Documentation Index

| Document | Description |
| :--- | :--- |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | High-level system architecture, crate topology, real-time audio thread rules, and content-addressed storage. |
| [DSP_PRIMITIVES_AND_PRESETS.md](./DSP_PRIMITIVES_AND_PRESETS.md) | Atomic DSP nodes (`OscSaw`, `FilterLadder`, `FilterComb`, etc.) and composite devices (`AetherSynth`, `Pluck`, `FmOperatorPair`, `SamplerDevice`). |
| [HARMONIC_BUS_AND_MICROTONAL.md](./HARMONIC_BUS_AND_MICROTONAL.md) | The reactive Global Harmonic Bus, 12-TET and arbitrary $N$-EDO microtonal tuning systems, and Scala (`.scl`/`.kbm`) integration. |
| [PATCH_AND_SESSION_SPEC.md](./PATCH_AND_SESSION_SPEC.md) | Complete TOML session document schema, track pipelines, step sequencer ratchets/probabilities, and preset specifications. |
| [VERSION_CONTROL_ENGINE.md](./VERSION_CONTROL_ENGINE.md) | The embedded `libgit2` micro-commit engine, DAG traversal for undo/redo, and patch export features. |
| [CLI_AND_HEADLESS.md](./CLI_AND_HEADLESS.md) | Headless daemon commands (`summon play`, `summon render`), UDP OSC remote server, and MIDI Clock synchronization. |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | Developer guide, coding standards, real-time safety guardrails, testing/doctests/fuzzing, and commit conventions. |

---

## 🚀 Quick Start

### Prerequisites
- **Rust Toolchain:** Stable 1.75+ with `cargo` and `rustc`.
- **C Compiler & Build Tools:** Required for native `libgit2` and audio driver backends.

### Building & Running

```bash
# Clone repository
git clone https://github.com/nilsanderselde/summoner.git
cd summoner

# Build release binaries
cargo build --release

# Run unit and doctests
make test

# Launch CLI playback
cargo run --bin summon -- play local/presets/freepats/freepats_gm.preset.toml

# Launch GPU-accelerated GUI
cargo run --bin summoner_gui
```

---

## 📄 License

Summoner DAW is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.
