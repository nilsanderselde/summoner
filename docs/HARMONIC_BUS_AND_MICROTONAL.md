# Summoner DAW — Global Harmonic Bus & Microtonal Systems

This document describes the **Global Harmonic Bus**, microtonal tuning engine ($N$-EDO), and Scala (`.scl`/`.kbm`) integration in `summoner_harmony`.

---

## 1. Global Harmonic Bus Overview

The **Global Harmonic Bus** (`HarmonicContext`) provides a system-wide reactive context for pitch, key, scale, and tuning across Summoner DAW.

Instead of tracks functioning in isolated pitch spaces, sequencers and synthesis engines subscribe to the Global Harmonic Bus to achieve real-time pitch alignment, dynamic scale snapping, and microtonal harmony.

```
┌──────────────────────────────────────────────────────────────┐
│                    HarmonicContext                           │
│  Tuning: N-EDO / Scala .scl   | Root: 440 Hz (A4 = note 69)  │
│  Scale: Major / Minor / Custom | Key Snap Mode: Reactive     │
└──────────────┬───────────────────────────────┬───────────────┘
               │                               │
               ▼                               ▼
    ┌────────────────────┐          ┌────────────────────┐
    │ Track 1 Sequencer  │          │ Track 2 Sequencer  │
    │  (Scale Snapping)  │          │ (N-EDO Microtonal) │
    └────────────────────┘          └────────────────────┘
```

---

## 2. Tuning Engine: 12-TET & N-EDO Microtonality

### Equal Division of the Octave ($N$-EDO)

Summoner supports standard 12-TET (12 Equal Division of the Octave) as well as arbitrary $N$-EDO microtonal systems (e.g., 19-EDO, 22-EDO, 31-EDO, 53-EDO).

For an $N$-EDO system, the pitch frequency $f$ for step index $k$ relative to root frequency $f_0$ is calculated as:

$$f(k) = f_0 \cdot 2^{\frac{k - k_0}{N}}$$

Where:
- $N$ is the number of divisions per octave (`tuning_edo`).
- $k$ is the note/step index.
- $k_0$ is the reference key index (default $69$ for $A_4$).
- $f_0$ is the reference root frequency (default $440.0 \text{ Hz}$).

---

## 3. Scala (`.scl`) & Key Mapping (`.kbm`) Support

Summoner parses standard Scala tuning definition files (`.scl`) and keyboard mapping files (`.kbm`) via `summoner_harmony::edo::ScalaTuning`.

### Scala (`.scl`) Format Parsing

```scl
! custom_19_edo.scl
19 Equal Division of the Octave
 19
!
 63.15789
 126.31579
 189.47368
 252.63158
 ...
 2/1
```

### Usage in Code

```rust
use summoner_harmony::bus::HarmonicContext;
use summoner_harmony::edo::EdoTuning;
use summoner_harmony::scale::Scale;

// Create a 19-EDO tuning bus centered at A4 = 440 Hz
let bus = HarmonicContext::new(EdoTuning::new(19), 60, Scale::major_12_tet());

// Convert note index 69.0 to Hz frequency
let freq = bus.freq_from_note(69.0);
```

---

## 4. Reactive Scale Snapping

Sequencers and MIDI controllers can query `snap_to_scale` to quantize incoming note pitches to the active scale:

- **Strict Quantize:** Snaps off-scale notes to the nearest valid degree in the scale.
- **Microtonal Quantize:** Preserves microtonal pitch offsets while mapping to nearest scale steps.
- **Harmonic Modulation:** Adjusting the scale or root note on the Global Harmonic Bus updates pitch resolution across all listening sequencers in real time.
