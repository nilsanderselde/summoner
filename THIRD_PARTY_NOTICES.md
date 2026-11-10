# Third-Party Notices and Licensing Audit (Summoner DAW)

## Overview & License Compatibility

Summoner is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0-or-later)**.
This document provides a compliance audit of third-party libraries, audio plugin SDKs, driver APIs, and crates used across the workspace.

---

## Plugin SDKs and Audio API Compliance

### 1. Steinberg VST3 SDK
- **License**: Dual-licensed under GPLv3 or proprietary Steinberg VST3 Developer Agreement.
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
- **Compliance Architecture**: Summoner does not bundle proprietary Steinberg ASIO headers in binary redistributions. ASIO support is isolated behind CPAL's optional feature flags and dynamically binds to registered ASIO driver DLLs via the Windows COM/registry ABI without redistributing SDK sources.

---

## Core Dependencies Audit

| Crate / Component | Upstream License | Usage in Summoner | AGPL-3.0 Compatibility Status |
|---|---|---|---|
| `cpal` | Apache-2.0 / MIT | Audio device driver backend | Fully compatible |
| `midir` | MIT | Hardware MIDI I/O | Fully compatible |
| `egui` / `eframe` | MIT / Apache-2.0 | GUI rendering and widgets | Fully compatible |
| `wgpu` | MIT / Apache-2.0 | WebGPU / DirectX / Metal / Vulkan rendering | Fully compatible |
| `symphonia` | MPL-2.0 | Audio codec decoding (FLAC, MP3, WAV, OGG) | Compatible via file-level MPL copyleft isolation |
| `hound` | Apache-2.0 | WAV reading and writing | Fully compatible |
| `rubato` | MIT | Sample rate conversion and time stretching | Fully compatible |
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
