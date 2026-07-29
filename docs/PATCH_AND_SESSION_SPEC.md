# Summoner DAW — Session & Patch Schema Specification

> **Specification Version:** 1.0.0  
> **Format:** TOML Document / JSON Schema Compatible (`preset.schema.json`)

This document defines the schema for declarative session documents (`.toml`), device presets (`.preset.toml`), routing topologies, sequence step data, and automation lanes.

---

## 1. Complete Session Document Example (`.toml`)

```toml
name = "Generative Ambient Session"
tuning_file = "local/tunings/19edo.scl"

[transport]
sample_rate = 44100
bpm = 118.0
time_signature = "4/4"

[[tracks]]
id = 1
name = "Subtractive Lead"
channels = 2
gain = 0.8
pan = 0.0
muted = false
tuning_edo = 19
tuning_root_hz = 440.0

[[tracks.nodes]]
kind = "OscSaw"
params = { freq = 440.0 }

[[tracks.nodes]]
kind = "FilterLadder"
params = { cutoff = 1800.0, res = 0.5 }

[[tracks.connections]]
from = "0:0"
to = "1:0"

[tracks.sequence]
step_division = 0.25

[[tracks.sequence.steps]]
note = 60.0
velocity = 0.8
gate = 0.5
probability = 0.9
ratchet = 2
micro_shift = 0
active = true

[[automation_lanes]]
param_id = "track_1_filter_cutoff"

[[automation_lanes.events]]
frame = 0
value = 400.0

[[automation_lanes.events]]
frame = 44100
value = 3200.0
```

---

## 2. Device Preset Schema (`.preset.toml`)

Presets describe modular composite sub-graphs or multi-sample zone mappings.

### 1. Sampler Preset (`.preset.toml`)

```toml
name = "Grand Piano"
instrument_type = "SamplerDevice"
attack_sec = 0.005
release_sec = 0.4

[[regions]]
lokey = 0
hikey = 60
pitch_keycenter = 48
lovel = 0
hivel = 127
sample_path = "samples/piano_c3.flac"

[[regions]]
lokey = 61
hikey = 127
pitch_keycenter = 72
lovel = 0
hivel = 127
sample_path = "samples/piano_c5.flac"
```

### 2. Synthesizer Preset (`.preset.toml`)

```toml
name = "Aether Lead"
device_kind = "AetherSynth"

[params]
frequency = 440.0
cutoff = 2400.0
resonance = 0.6
attack = 0.01
release = 0.5
sub_osc_gain = 0.3
```

---

## 3. Polymetric Step Sequencer Schema

Step sequencers support non-standard meter and polymetric step divisions.

- **`step_division`**: Length of step in quarter note beats ($0.25 = \text{16th note}$).
- **`probability`**: Chance of note trigger ($0.0$ to $1.0$).
- **`ratchet`**: Number of sub-burst note triggers within the step duration.
- **`micro_shift`**: Micro-timing offset in ticks for humanization.

---

## 4. Key Schema Invariants

1. **Deterministic Execution:** Given identical session `.toml` documents, audio renders produce bit-identical WAV/FLAC output across platforms.
2. **Schema Validation:** Presets and session files validate against `local/preset.schema.json`.
3. **Channel Agnostic:** Track channels can be configured dynamically ($1$ mono, $2$ stereo, $4$ quad, or multi-bus surround).
