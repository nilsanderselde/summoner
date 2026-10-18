#![allow(clippy::all)]

//! Test suite for Turn #123 (Sprint Turn #49) — Real-Time Master Bus Live Session Recorder & Disk Capture HUD
//! and VST3 / CLAP / WebAssembly Plugin Host & Parameter Automapper HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::live_session_recorder_view::{
        LiveRecordDiskFormat, LiveRecorderPreset, LiveSessionRecorderView,
        LIVE_RECORDER_PUCK_HIT_RADIUS,
    };
    use crate::views::plugin_host_view::{
        HostedPluginFormat, PluginHostPreset, PluginHostView, PLUGIN_HOST_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn123_pure_live_session_recorder_initialization_and_physics() {
        let mut view = LiveSessionRecorderView::new();
        assert_eq!(view.preset, LiveRecorderPreset::MasterStereoWav44k);
        assert_eq!(LIVE_RECORDER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                LiveRecorderPreset::MasterStereoWav44k,
                "Master Stereo WAV (44.1 kHz / 16-bit)",
                0.0,
                -0.1,
                44100,
                LiveRecordDiskFormat::Wav16BitPcm,
                2.0,
            ),
            (
                LiveRecorderPreset::ArchivalFlac48k,
                "Archival Studio FLAC (48 kHz / 24-bit)",
                0.0,
                -0.3,
                48000,
                LiveRecordDiskFormat::FlacLossless24,
                3.5,
            ),
            (
                LiveRecorderPreset::BroadcastSafeMaster,
                "Broadcast Safe Master (-1.0 dBFS Ceiling)",
                -1.5,
                -1.0,
                48000,
                LiveRecordDiskFormat::Wav24BitPcm,
                1.5,
            ),
            (
                LiveRecorderPreset::ModularJamHeadroom,
                "Modular Live Jam (-6.0 dB Headroom)",
                -6.0,
                -0.5,
                44100,
                LiveRecordDiskFormat::Wav24BitPcm,
                4.0,
            ),
        ];

        for (p, exp_name, exp_trim, exp_ceil, exp_sr, exp_fmt, exp_tail) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (trim, ceil, sr, fmt, tail) = p.nominal_parameters();
            assert_eq!(trim, exp_trim);
            assert_eq!(ceil, exp_ceil);
            assert_eq!(sr, exp_sr);
            assert_eq!(fmt, exp_fmt);
            assert_eq!(tail, exp_tail);

            assert_eq!(view.input_trim_db, exp_trim);
            assert_eq!(view.safety_ceiling_db, exp_ceil);
            assert_eq!(view.sample_rate, exp_sr);
            assert_eq!(view.format, exp_fmt);
            assert_eq!(view.auto_tail_sec, exp_tail);
        }

        // Input Trim & Safety Ceiling normalization round-trip
        for t in [-18.0, -6.0, 0.0, 3.0, 6.0] {
            let norm = LiveSessionRecorderView::trim_to_normalized(t);
            assert!((0.0..=1.0).contains(&norm));
            let back = LiveSessionRecorderView::normalized_to_trim(norm);
            assert!((back - t).abs() < 1e-2);
        }
        for c in [-6.0, -3.0, -1.0, -0.3, 0.0] {
            let norm = LiveSessionRecorderView::ceiling_to_normalized(c);
            assert!((0.0..=1.0).contains(&norm));
            let back = LiveSessionRecorderView::normalized_to_ceiling(norm);
            assert!((back - c).abs() < 1e-2);
        }

        // PCM i16 quantization and safety brickwall ceiling
        view.set_preset(LiveRecorderPreset::MasterStereoWav44k);
        view.input_trim_db = 0.0;
        view.safety_ceiling_db = 0.0;
        assert_eq!(view.quantize_sample_i16(0.0), 0);
        assert!(view.quantize_sample_i16(1.0) >= 32766);
        assert!(view.quantize_sample_i16(-1.0) <= -32766);

        // Duration and file size estimation
        view.captured_frames = 44_100;
        view.sample_rate = 44_100;
        assert!((view.duration_seconds() - 1.0).abs() < 1e-5);
        assert_eq!(view.estimated_file_size_bytes(), 44 + 44_100 * 2 * 2);

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("MASTER LIVE SESSION RECORDER & DISK CAPTURE SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn123_pure_plugin_host_initialization_and_physics() {
        let mut view = PluginHostView::new();
        assert_eq!(view.preset, PluginHostPreset::ClapPolyMpeSynth);
        assert_eq!(PLUGIN_HOST_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                PluginHostPreset::ClapPolyMpeSynth,
                "CLAP MPE Poly Synth (Sample-Accurate)",
                1.00,
                2400.0,
                1.00,
                0.72,
                HostedPluginFormat::ClapMpeHost,
                0,
            ),
            (
                PluginHostPreset::Vst3MasteringChain,
                "VST3 Mastering Bus Chain (100% Wet)",
                1.05,
                8500.0,
                1.00,
                0.50,
                HostedPluginFormat::Vst3BundleHost,
                64,
            ),
            (
                PluginHostPreset::WasmSandboxedGranular,
                "Wasm Sandboxed Spectral FX (64 Pages)",
                0.95,
                1600.0,
                0.80,
                0.85,
                HostedPluginFormat::WasmSandboxHost,
                32,
            ),
            (
                PluginHostPreset::ParallelInsertBlend,
                "Parallel Insert Blend (50% Wet/Dry)",
                1.15,
                1200.0,
                0.50,
                0.45,
                HostedPluginFormat::ClapMpeHost,
                16,
            ),
        ];

        for (p, exp_name, exp_gain, exp_cut, exp_wet, exp_mpe, exp_fmt, exp_lat) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (gain, cut, wet, mpe, fmt, lat) = p.nominal_parameters();
            assert_eq!(gain, exp_gain);
            assert_eq!(cut, exp_cut);
            assert_eq!(wet, exp_wet);
            assert_eq!(mpe, exp_mpe);
            assert_eq!(fmt, exp_fmt);
            assert_eq!(lat, exp_lat);

            assert_eq!(view.plugin_gain, exp_gain);
            assert_eq!(view.cutoff_hz, exp_cut);
            assert_eq!(view.wet_mix, exp_wet);
            assert_eq!(view.mpe_timbre, exp_mpe);
            assert_eq!(view.format, exp_fmt);
            assert_eq!(view.latency_samples, exp_lat);
        }

        // Wet Mix & Cutoff log normalization round-trip
        for w in [0.0, 0.25, 0.50, 0.85, 1.0] {
            let norm = PluginHostView::mix_to_normalized(w);
            assert!((0.0..=1.0).contains(&norm));
            let back = PluginHostView::normalized_to_mix(norm);
            assert!((back - w).abs() < 1e-3);
        }
        for c in [20.0, 200.0, 1000.0, 2400.0, 20_000.0] {
            let norm = PluginHostView::cutoff_to_normalized(c);
            assert!((0.0..=1.0).contains(&norm));
            let back = PluginHostView::normalized_to_cutoff(norm);
            assert!((back - c).abs() / c < 1e-2);
        }

        // Sample evaluation & bypass behavior
        view.set_preset(PluginHostPreset::ParallelInsertBlend);
        view.bypassed = true;
        assert!((view.evaluate_hosted_sample(0.5) - 0.5).abs() < 1e-6);
        view.bypassed = false;
        let processed = view.evaluate_hosted_sample(0.5);
        assert!(processed > 0.0 && processed <= 1.0);

        // Response curve bounds
        for x in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let y = view.evaluate_response_curve(x);
            assert!((0.0..=1.0).contains(&y));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("VST3 / CLAP / WASM PLUGIN HOST & AUTOMAP SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::live_session_recorder_view::LiveSessionRecorderView;
    use crate::views::plugin_host_view::PluginHostView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn123_headless_render_views() {
        let ctx = egui::Context::default();
        let mut rec_view = LiveSessionRecorderView::new();
        let mut ph_view = PluginHostView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rec_view.show(ui);
                ph_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn123_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test Live Session Recorder HUD Lifecycle
        assert!(!aw_view.is_live_session_recorder_hud_open());
        aw_view.open_live_session_recorder_hud();
        assert!(aw_view.is_live_session_recorder_hud_open());
        aw_view.sync_live_session_recorder_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("input_trim_db"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("safety_ceiling_db"));
        aw_view.close_live_session_recorder_hud();
        assert!(!aw_view.is_live_session_recorder_hud_open());

        // Test Plugin Host HUD Lifecycle
        assert!(!aw_view.is_plugin_host_hud_open());
        aw_view.open_plugin_host_hud();
        assert!(aw_view.is_plugin_host_hud_open());
        aw_view.sync_plugin_host_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("wet_mix"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("cutoff_hz"));
        aw_view.close_plugin_host_hud();
        assert!(!aw_view.is_plugin_host_hud_open());
    }

    #[test]
    fn test_turn123_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_recorder = actions
            .iter()
            .any(|a| a.action_id == "open_live_session_recorder_hud");
        let has_plugin_host = actions
            .iter()
            .any(|a| a.action_id == "open_plugin_host_hud");

        assert!(
            has_recorder,
            "open_live_session_recorder_hud must be registered in command palette"
        );
        assert!(
            has_plugin_host,
            "open_plugin_host_hud must be registered in command palette"
        );
    }
}
