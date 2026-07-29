# Summoner DAW — DSP Primitives & Composite Devices

This document outlines the **Don't Repeat Yourself (DRY) DSP Architecture** in Summoner DAW, detailing atomic signal processing nodes, factory preset devices, and the dual Micro/Macro presentation system.

---

## 1. DRY DSP Architecture

Synthesizers, samplers, and effect units in Summoner DAW are not monolithic, hardcoded binaries. Instead, they are defined strictly as **modular sub-graphs dynamically constructed from a shared pool of atomic DSP primitives**.

Every instrument is a declarative graph configuration. The engine treats native factory synths and custom user patches identically.

```
                  ┌─────────────────────────────────────┐
                  │          Atomic Primitives          │
                  │  (OscSaw, FilterLadder, EnvADSR...) │
                  └──────────────────┬──────────────────┘
                                     │ Patch Connections
                                     ▼
                  ┌─────────────────────────────────────┐
                  │          Composite Device           │
                  │    (AetherSynth / Pluck / FM Pair)  │
                  └──────────┬──────────────────────┬───┘
                             │                      │
                             ▼                      ▼
                     ┌───────────────┐      ┌───────────────┐
                     │  Micro View   │      │  Macro View   │
                     │  (Node Graph) │      │  (Control UI) │
                     └───────────────┘      └───────────────┘
```

---

## 2. Atomic Oscillator & Processor Primitives

The core building blocks reside in `summoner_dsp`. All primitives implement the `AudioNode` trait with zero-allocation `process_block` routines.

### Oscillators (Audio-Rate Generators)

- **`OscSaw`**: Band-limited sawtooth wave generator using PolyBLEP anti-aliasing. Supports $V/\text{Oct}$ pitch modulation and hard phase sync.
- **`OscPulse`**: Square and pulse wave generator with modulatable pulse width (PWM) and anti-aliasing.
- **`OscSine`**: Pure sine wave generator for clean sub-bass and FM (Phase Modulation) operator synthesis.
- **`OscTriangle`**: Band-limited triangle wave for soft subtractive synthesis foundations.
- **`NoiseGen`**: Algorithmic noise generator producing White, Pink, and Brown noise using a non-allocating PRNG.

### Filters & Processors

- **`FilterLadder`**: 4-pole (24 dB/octave) Moog-style nonlinear ladder filter with saturating resonance control.
- **`FilterSVF`**: State Variable Filter outputting simultaneous Lowpass, Highpass, Bandpass, and Notch signals.
- **`FilterComb`**: Tuned delay line with feedback for Karplus-Strong physical modeling and flanging effects.

### Modulators & Utilities

- **`EnvADSR`**: Standard 4-stage exponential envelope generator driven by gate triggers ($0.0$ to $1.0$ unipolar output).
- **`LFO`**: Low-Frequency Oscillator with Sine, Triangle, Square, and Sample & Hold shapes (Hz or tempo-synced).
- **`MacroKnob`**: Virtual control node mapping GUI or MIDI CC parameters to multiple destination nodes simultaneously.
- **`MathAdd` / `MathMult`**: Signal summing mixers and ring modulators/amplification multipliers.
- **`VCA`**: Voltage-Controlled Amplifier for shaping audio rate signals with envelope control signals.

---

## 3. Composite Devices & Factory Presets

### 1. AetherSynth (`.preset.toml`)

- **Architecture:** Subtractive and FM dual-oscillator synthesizer.
- **Topology:** `OscSaw` (Osc 1) and `OscPulse` (Osc 2) -> `MathAdd` -> `FilterLadder` -> `VCA`.
- **Modulation:** `EnvADSR` 1 controls `VCA` amplitude. `EnvADSR` 2 controls `FilterLadder` cutoff. `LFO` modulates `OscPulse` PWM.

### 2. Pluck (`.preset.toml` — Physical Modeling)

- **Architecture:** Karplus-Strong plucked string acoustic simulator.
- **Topology:** Short `NoiseGen` burst (exciter) -> `FilterComb` (tuned to MIDI pitch feedback delay) -> `FilterSVF` lowpass inside feedback loop (simulates string damping over time).

### 3. FmOperatorPair (`.preset.toml`)

- **Architecture:** 2-operator Phase Modulation synthesizer.
- **Topology:** `OscSine` (Modulator) scaled by `EnvADSR` 1 -> Phase modulation input of `OscSine` (Carrier) -> `VCA` scaled by `EnvADSR` 2.

### 4. SamplerDevice (`SamplerDevice`)

- **Architecture:** Multi-sample keyzone/velocity-zone instrument engine.
- **Capabilities:** Supports WAV and FLAC sample playback, keycenter tracking, ADSR envelope shaping, and multi-region `.preset.toml` mapping.

---

## 4. Micro View vs. Macro View

Every composite device provides two complementary presentation interfaces in `summoner_gui`:

1. **Micro View:** Full modular canvas exposing the underlying node graph, allowing users to rewire signals, add modulation nodes, or inspect internal node outputs.
2. **Macro View:** Streamlined control panel presenting primary parameters (e.g., Cutoff, Resonance, Attack, Release, Osc Mix) as macro knobs.
