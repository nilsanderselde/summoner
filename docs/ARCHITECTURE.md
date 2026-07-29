# Summoner DAW — System Architecture

This document details the software architecture, crate topology, real-time audio thread constraints, and asset storage model of **Summoner DAW**.

---

## 1. Core Principles

- **Determinism First:** Given identical project configurations, audio buffers, and seed states, `summon` MUST produce bit-identical rendered audio output across all systems and operating systems.
- **Real-Time Audio Safety:** Zero heap allocation (`malloc`/`free`) and zero locking primitives (mutexes, sync channels) inside the DSP render loop (`process_block`).
- **Headless Native:** The engine core is decoupled from rendering and UI. All synthesis, sequencing, and processing functionality is accessible via the CLI daemon (`summon.exe`).
- **Declarative State:** Complete session state is represented in human-readable, Git-friendly plain-text `.toml` files.

---

## 2. Workspace Topology & Crate Division

The project is structured as a modular Rust workspace (2021 edition):

```
summoner/
├── Cargo.toml
├── docs/                      # Open-source technical documentation
├── crates/
│   ├── summon                 # CLI binary & headless daemon launcher
│   ├── summoner_core          # Lock-free buffers, signal graph, AudioNode traits, MIDI Clock
│   ├── summoner_dsp           # SIMD DSP algorithms, PolyBLEP oscillators, filters, samplers
│   ├── summoner_harmony       # Global Harmonic Bus & N-EDO tuning systems
│   ├── summoner_project       # TOML parsing & libgit2 micro-commit DAG engine
│   └── summoner_gui           # GPU-accelerated UI (wgpu), NodeGraph editor & StageView
└── fuzz/                      # LibFuzzer targets for DSP algorithms
```

### Crate Responsibilities

- **`summon` (CLI App):** Binary wrapper providing command-line arguments parsing (`clap`), headless rendering daemon (`summon render`), live audio playback (`summon play`), UDP OSC server, and MIDI Clock outputs.
- **`summoner_core`:** Core abstractions including `AudioNode`, `AudioBuffer`, `SignalGraph`, lock-free atomic ring buffers (`AtomicRingBuffer`), and MIDI clock sync primitives.
- **`summoner_dsp`:** SIMD-optimized audio generation and effect primitives (`OscSaw`, `OscPulse`, `FilterLadder`, `FilterSVF`, `FilterComb`, `EnvADSR`), multi-sample streaming engine (`SamplerDevice`), and physical modeling synthesis.
- **`summoner_harmony`:** Engine-wide reactive harmonic context (`HarmonicContext`), scale definitions, $N$-EDO tuning calculations, and Scala (`.scl`/`.kbm`) parser.
- **`summoner_project`:** TOML project schema serialization/deserialization, patch management, and embedded `libgit2` version control engine for micro-commits and undo/redo history.
- **`summoner_gui`:** Cross-platform GUI powered by `wgpu`. Includes Arranger view, NodeGraph modular routing editor, StageView live performance grid, waveform cache, and shortcut key managers.

---

## 3. Real-Time Audio Execution Model

```
                    ┌─────────────────────────┐
                    │      GUI / CLI Layer    │
                    └────────────┬────────────┘
                                 │ Atomically Queued
                                 │ Parameter Commands
                                 ▼
┌──────────────────────────────────────────────────────────────┐
│                    Audio Render Thread                       │
│                                                              │
│  ┌──────────────────┐   ┌─────────────────┐   ┌───────────┐  │
│  │ Global Harmonic  │──►│ NodeGraph DSP   │──►│ Multi-Bus │  │
│  │ Context (N-EDO)  │   │ Evaluation Loop │   │ Master    │  │
│  └──────────────────┘   └─────────────────┘   └───────────┘  │
│          Zero Allocation | Lock-Free | Bit-Exact            │
└──────────────────────────────────────────────────────────────┘
```

### Real-Time Thread Constraints

1. **Zero Heap Allocation:** Allocating memory (`Box::new`, `Vec::push`, string operations) is prohibited inside `process_block`. All audio buffers, node state, and delay lines are pre-allocated during initial graph construction.
2. **Lock-Free Parameter Sync:** GUI-to-Audio parameter updates are transferred exclusively using atomic variables (`AtomicF32`, `AtomicU32`) or lock-free SPSC queues (`AtomicRingBuffer`).
3. **Graph Evaluation:** The DSP node graph is evaluated block-by-block using static buffer allocations. Topological execution order is pre-computed on graph updates, eliminating dynamic traversal overhead during audio rendering.

---

## 4. Content-Addressed Asset Storage

Audio assets, sample banks, and audio clips are referenced and indexed by their **BLAKE3 content hash**, rather than fragile relative file paths.

- **Sample Deduplication:** Identical audio files stored in different locations are recognized and loaded once based on content hash.
- **Project Portability:** Session files refer to sample content hashes, making project sharing across directories or machines robust against broken path references.
