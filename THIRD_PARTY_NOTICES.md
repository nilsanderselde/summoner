# Third-Party Notices and Licensing Audit (Summoner DAW)

## Overview & License Compatibility

Summoner is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0-or-later)**.
This document provides a compliance audit of third-party libraries, audio plugin SDKs, driver APIs, and crates used across the workspace.

---

## Plugin SDKs and Audio API Compliance

### 1. Steinberg VST3 SDK
- **License**: MIT since VST3 SDK 3.8 (earlier versions: GPLv3 or proprietary Steinberg agreement). Use SDK >= 3.8 only.
- **Compatibility**: AGPL-3.0 is compatible with GPLv3. The VST3 hosting code (`summoner_dsp::vst3_host`) and export code (`summoner_dsp::plugin_export`) comply with Steinberg's open-source terms without requiring proprietary licensing.

### 2. CLAP (Clever Audio Plug-in)
- **License**: MIT License.
- **Compatibility**: Permissive MIT license allows bidirectional embedding, hosting (`summoner_dsp::clap_host`), and plugin export (`summoner_dsp::plugin_export`) under AGPL-3.0.

### 3. Apple AudioUnit (AUv2 / AUv3)
- **License**: Apple CoreAudio SDK / macOS System Frameworks.
- **Compatibility**: Dynamically linked system APIs on macOS (`AudioToolbox.framework`, `CoreAudio.framework`). Standard operating system linking exception applies.

### 4. Linux LV2
- **License**: ISC / MIT.
- **Compatibility**: Permissive licensing allows full dynamic loading and URID mapping without GPL/AGPL conflicts.

### 5. Steinberg ASIO
- **License**: Proprietary Steinberg ASIO SDK.
- **Compliance Architecture**: The ASIO SDK is available under GPLv3 (compatible with AGPL-3.0) or a proprietary agreement. CPAL's `asio` feature needs the SDK at *build* time (`CPAL_ASIO_DIR`); the SDK is never committed to this repository. Binaries built with `asio` enabled are distributed under AGPL-3.0 only and must be built by the release pipeline from a developer-supplied SDK copy (see audit item AIO-1). The ASIO name/logo are Steinberg trademarks.

---

## Core Dependencies Audit

| Crate / Component | Upstream License | Usage in Summoner | AGPL-3.0 Compatibility Status |
|---|---|---|---|
| `cpal` | Apache-2.0 / MIT | Audio device driver backend | Fully compatible |
| `midir` | MIT | Hardware MIDI I/O | Fully compatible |
| `egui` / `eframe` | MIT / Apache-2.0 | GUI rendering and widgets | Fully compatible |
| `wgpu` | MIT / Apache-2.0 | Not yet a dependency (planned, GUI-1); pulled in via eframe backend when enabled | Fully compatible |
| `symphonia` | MPL-2.0 | Audio codec decoding (FLAC, MP3, WAV, OGG) | Compatible via file-level MPL copyleft isolation |
| `hound` | Apache-2.0 | WAV reading and writing | Fully compatible |
| `rubato` | MIT | Not yet a dependency (planned, AIO-6 sample-rate conversion) | Fully compatible |
| `parking_lot` | Apache-2.0 / MIT | Low-latency synchronization primitives | Fully compatible |
| `crossbeam-channel` | Apache-2.0 / MIT | Lock-free audio-to-GUI communication | Fully compatible |
| `serde` / `serde_json` | Apache-2.0 / MIT | State serialization and project schema | Fully compatible |
| `blake3` | Apache-2.0 / CC0 | Checksum and content-addressed storage | Fully compatible |
| `ureq` | MIT / Apache-2.0 | Optional cloud session sync & updates | Fully compatible |
| `git2` / `libgit2` | GPL-2.0 with Linking Exception | Optional version control integration | Fully compatible |

---

## Audio Content & Factory Assets

- **Factory Presets & Wavetables**: Authored in-house for Summoner. All bundled factory presets, impulse responses, and wavetables are released under the Creative Commons Zero (CC0 1.0 Universal) dedication.
- **Demonstration Projects**: Bundled project files and recordings are provided royalty-free for educational and demonstration purposes.

---

## Planned Dependencies - Licence Decisions (audit PLT-8)

| Component | Licence | Decision |
|---|---|---|
| Ableton Link SDK | GPL-2.0-or-later (open source) | Compatible with AGPL-3.0; use the open-source SDK, keep notices (MID-7) |
| LAME (MP3 encode) | LGPL-2.1 | Dynamic linking only, or use a pure-Rust encoder; MP3 patents expired 2017 (IOX-1) |
| FFmpeg | LGPL-2.1+ (GPL if built with GPL parts) | Use LGPL build, dynamic linking, no `--enable-gpl`/`--enable-nonfree`; or use OS decoders (IOX-5) |
| Opus / libopus | BSD-3-Clause | Compatible (IOX-1) |
| FLAC encoder (libFLAC) | BSD-3-Clause | Compatible (IOX-1) |
| ONNX Runtime (ort) | MIT | Compatible. Model weights carry their own licences (e.g. Demucs MIT) - record per model (NET-2) |
| btleplug | MIT / Apache-2.0 / BSD-3 | Compatible (AIO-10) |
| webrtc-rs | MIT / Apache-2.0 | Compatible (NET-1) |
| lilv / LV2 | ISC | Compatible (PLG-4) |
| Apple frameworks | System | Linked dynamically; no redistribution |
| AAX SDK | Proprietary (Avid) | **Not supported**; cannot be combined with AGPL-3.0 |
| VST2 SDK | Proprietary (Steinberg, discontinued) | **Not supported** |
