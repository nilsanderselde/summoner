//! Capability inventory (audit item FND-1).
//!
//! Records, for every externally-facing capability, whether it is backed by a real
//! OS / library / network implementation or only modelled in-process. The registry is
//! the single source of truth used by `summon capabilities` and by the release gate
//! (`--deny-simulated`). When a capability becomes real, flip its [`Fidelity`] here in
//! the same commit that lands the real backend.

use serde::{Deserialize, Serialize};

/// How closely an implementation corresponds to the real thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Fidelity {
    /// Backed by a real OS API, library, protocol stack or file format implementation.
    Real,
    /// Data structures / protocol logic exist and are unit-tested, but no OS, network
    /// or runtime backend is connected. Cannot be used end-to-end.
    Modelled,
    /// Produces stand-in results (hard-coded parameters, placeholder frames, mock
    /// processors) that look like the real thing. Must never ship as a default path.
    Simulated,
}

impl Fidelity {
    pub fn as_str(self) -> &'static str {
        match self {
            Fidelity::Real => "real",
            Fidelity::Modelled => "modelled",
            Fidelity::Simulated => "simulated",
        }
    }
}

/// One entry of the capability inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Capability {
    pub id: &'static str,
    pub name: &'static str,
    pub fidelity: Fidelity,
    /// Audit work item that will make this capability real (empty if already real).
    pub audit_item: &'static str,
    pub note: &'static str,
}

const fn cap(
    id: &'static str,
    name: &'static str,
    fidelity: Fidelity,
    audit_item: &'static str,
    note: &'static str,
) -> Capability {
    Capability {
        id,
        name,
        fidelity,
        audit_item,
        note,
    }
}

use Fidelity::{Modelled, Real, Simulated};

/// The full inventory. Keep sorted by `id`.
pub const CAPABILITIES: &[Capability] = &[
    cap(
        "audio.cpal_io",
        "Audio I/O via cpal (WASAPI/ALSA/CoreAudio)",
        Real,
        "",
        "ASIO/JACK/hot-plug pending AIO-1..4",
    ),
    cap(
        "audio.decode",
        "Audio decoding (WAV/FLAC/MP3/OGG via symphonia/hound/claxon)",
        Real,
        "",
        "",
    ),
    cap(
        "audio.encode_compressed",
        "MP3/AAC/Opus/FLAC encoding",
        Modelled,
        "IOX-1",
        "No encoder crates; WAV only",
    ),
    cap(
        "audio.sample_rate_conversion",
        "Sample-rate conversion",
        Modelled,
        "AIO-6",
        "No resampler dependency",
    ),
    cap(
        "ble_midi.backend",
        "Bluetooth LE MIDI transport",
        Modelled,
        "AIO-10",
        "No OS BLE calls",
    ),
    cap(
        "clap.host",
        "CLAP plugin hosting",
        Modelled,
        "PLG-2",
        "No library entry call",
    ),
    cap(
        "cloud.render",
        "Distributed cloud render",
        Modelled,
        "NET-5",
        "",
    ),
    cap(
        "link.ableton",
        "Ableton Link network sync",
        Modelled,
        "MID-7",
        "",
    ),
    cap(
        "midi.io",
        "MIDI I/O via midir",
        Real,
        "",
        "Virtual ports on Windows pending AIO-8",
    ),
    cap(
        "midi2.transport",
        "MIDI 2.0 UMP over OS transports",
        Modelled,
        "AIO-9",
        "",
    ),
    cap(
        "mobile.audio",
        "AAudio / iOS AudioUnit mobile drivers",
        Modelled,
        "NET-4",
        "",
    ),
    cap(
        "neural.onnx",
        "ONNX Runtime neural separation",
        Modelled,
        "NET-2",
        "No ort dependency",
    ),
    cap(
        "plugin.au_host",
        "AudioUnit hosting (macOS)",
        Modelled,
        "PLG-3",
        "",
    ),
    cap(
        "plugin.editor_window",
        "Native plugin editor embedding",
        Modelled,
        "PLG-5",
        "Window-type enum only",
    ),
    cap(
        "plugin.lv2_host",
        "LV2 plugin hosting (Linux)",
        Modelled,
        "PLG-4",
        "",
    ),
    cap(
        "plugin.vst3_host",
        "VST3 plugin hosting",
        Simulated,
        "PLG-1",
        "Hard-coded Gain/Cutoff mock processor",
    ),
    cap(
        "video.decode",
        "Video decode and filmstrips",
        Simulated,
        "IOX-5",
        "Placeholder thumbnails",
    ),
    cap(
        "wasm.runtime",
        "WebAssembly headless renderer",
        Modelled,
        "NET-3",
        "No wasm toolchain",
    ),
    cap(
        "webrtc.mesh",
        "WebRTC audio mesh",
        Modelled,
        "NET-1",
        "No network transport",
    ),
];

/// All capabilities whose fidelity is not [`Fidelity::Real`].
pub fn not_real() -> impl Iterator<Item = &'static Capability> {
    CAPABILITIES.iter().filter(|c| c.fidelity != Fidelity::Real)
}

/// All capabilities that are [`Fidelity::Simulated`].
pub fn simulated() -> impl Iterator<Item = &'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|c| c.fidelity == Fidelity::Simulated)
}

/// Render the inventory as an aligned text table.
pub fn render_table() -> String {
    let mut out = String::new();
    for c in CAPABILITIES {
        out.push_str(&format!(
            "{:<10} {:<34} {:<8} {}\n",
            c.fidelity.as_str(),
            c.id,
            c.audit_item,
            c.name
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_unique_and_sorted() {
        for w in CAPABILITIES.windows(2) {
            assert!(
                w[0].id < w[1].id,
                "{} must sort before {}",
                w[0].id,
                w[1].id
            );
        }
    }

    #[test]
    fn non_real_entries_reference_an_audit_item() {
        for c in not_real() {
            assert!(!c.audit_item.is_empty(), "{} lacks an audit item", c.id);
        }
    }

    #[test]
    fn real_entries_have_no_audit_item() {
        for c in CAPABILITIES.iter().filter(|c| c.fidelity == Fidelity::Real) {
            assert!(
                c.audit_item.is_empty(),
                "{} is real; clear audit_item",
                c.id
            );
        }
    }

    #[test]
    fn table_lists_every_capability() {
        assert_eq!(render_table().lines().count(), CAPABILITIES.len());
    }

    #[test]
    fn vst3_host_is_flagged_simulated_until_real_backend_lands() {
        // The source still contains the mock processor; keep this in sync (PLG-1).
        let src = include_str!("../../summoner_dsp/src/vst3_host.rs");
        let has_mock = src.contains("_mock_active: true");
        let flagged = simulated().any(|c| c.id == "plugin.vst3_host");
        assert_eq!(has_mock, flagged, "vst3_host mock and registry disagree");
    }
}
