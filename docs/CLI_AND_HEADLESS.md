# Summoner DAW — CLI & Headless Operation

This document covers command-line flags, offline rendering daemon commands, the **UDP OSC Remote Control Server**, and **MIDI Clock Synchronization** interfaces provided by `summon`.

---

## 1. CLI Commands & Headless Daemon

The CLI binary (`summon`) provides full access to the audio engine, allowing offline rendering, script automation, and headless cloud execution without running a GUI.

### Command Reference

```bash
# Play a session file or preset live
summon play <SESSION_OR_PRESET_PATH> [--bpm <BPM>] [--sample-rate <SR>] [--midi-clock-out <DEVICE>]

# Render a session file offline to WAV/FLAC (bit-exact)
summon render <SESSION_PATH> --output <OUTPUT_WAV_PATH> [--duration <SECONDS>]

# Launch headless daemon for RPC / server integration
summon daemon --port 8000

# Print CLI version and build target info
summon version
```

### CLI Arguments & Options

- `--bpm <FLOAT>`: Override session BPM.
- `--sample-rate <INT>`: Set target audio sample rate ($44100$, $48000$, $96000$).
- `--midi-clock-out <STRING>`: Enable MIDI Clock Output (24 PPQN) to specified hardware MIDI port.
- `--preset <PATH>`: Load a specific `.preset.toml` device file into the session runner.

---

## 2. UDP OSC Remote Control Server

Summoner includes an embedded **UDP Open Sound Control (OSC) Server** listening on UDP port `8000` (configurable) for remote playback and parameter control.

### Supported OSC Messages

| OSC Address | Arguments | Description |
| :--- | :--- | :--- |
| `/play` | None | Start session transport playback. |
| `/stop` | None | Stop session transport playback and reset playhead to zero. |
| `/bpm` | `[float]` | Update transport tempo in BPM dynamically. |
| `/param` | `[string, float]` | Set parameter by ID (e.g., `"/param", "cutoff", 2400.0`). |

---

## 3. MIDI Clock Synchronization

Summoner provides hardware-grade MIDI Clock synchronization in `summoner_core::midi_clock`:

- **24 PPQN Generator:** Transmits `0xF8` MIDI Clock bytes every $1/24$th of a quarter note beat during playback.
- **Start / Stop / Continue:** Emits `0xFA` (Start), `0xFC` (Stop), and `0xFB` (Continue) System Real-Time messages.
- **Receiver / External Sync:** Listens to incoming MIDI clock streams to slave transport tempo.
