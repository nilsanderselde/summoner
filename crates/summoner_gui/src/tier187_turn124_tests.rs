#![allow(clippy::all)]

//! Test suite for Turn #124 (Sprint Turn #50) — AI Autonomous Mastering, Drum Replacer & Room Acoustic Matcher HUD
//! and Hardware Control Surface, CV/Gate Modular Output & DIN Sync24 Bridge HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::ai_autonomous_mastering_view::{
        AiAutonomousMasteringView, AiMasteringPreset, MasteringTargetCurve,
        AI_MASTERING_PUCK_HIT_RADIUS,
    };
    use crate::views::hardware_control_surface_view::{
        HardwareControlPreset, HardwareControlSurfaceView, HardwareControllerProtocol,
        HARDWARE_SURFACE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn124_pure_ai_autonomous_mastering_initialization_and_physics() {
        let mut view = AiAutonomousMasteringView::new();
        assert_eq!(view.preset, AiMasteringPreset::StreamingPop14Lufs);
        assert_eq!(AI_MASTERING_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                AiMasteringPreset::StreamingPop14Lufs,
                "Streaming Pop Standard (-14.0 LUFS / -1.0 dBFS)",
                -14.0,
                -1.0,
                MasteringTargetCurve::ModernPop,
                -18.0,
                0.65,
                1.20,
                0.45,
            ),
            (
                AiMasteringPreset::ClubFestival8Lufs,
                "Club & Festival Impact (-8.0 LUFS / -0.1 dBFS)",
                -8.0,
                -0.1,
                MasteringTargetCurve::ClubLoudness,
                -12.0,
                0.85,
                0.85,
                0.75,
            ),
            (
                AiMasteringPreset::WarmVinyl16Lufs,
                "Warm Analog Vinyl (-16.0 LUFS / -0.5 dBFS)",
                -16.0,
                -0.5,
                MasteringTargetCurve::WarmAnalog,
                -22.0,
                0.40,
                1.65,
                0.55,
            ),
            (
                AiMasteringPreset::BroadcastEbu23Lufs,
                "EBU R128 Broadcast (-23.0 LUFS / -1.5 dBFS)",
                -23.0,
                -1.5,
                MasteringTargetCurve::BrightAir,
                -24.0,
                0.25,
                1.10,
                0.30,
            ),
        ];

        for (p, exp_name, exp_lufs, exp_ceil, exp_curve, exp_dth, exp_dbl, exp_rt, exp_sub) in
            presets
        {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (lufs, ceil, curve, dth, dbl, rt, sub) = p.nominal_parameters();
            assert_eq!(lufs, exp_lufs);
            assert_eq!(ceil, exp_ceil);
            assert_eq!(curve, exp_curve);
            assert_eq!(dth, exp_dth);
            assert_eq!(dbl, exp_dbl);
            assert_eq!(rt, exp_rt);
            assert_eq!(sub, exp_sub);

            assert_eq!(view.target_lufs, exp_lufs);
            assert_eq!(view.ceiling_db, exp_ceil);
            assert_eq!(view.target_curve, exp_curve);
            assert_eq!(view.drum_threshold_db, exp_dth);
            assert_eq!(view.drum_blend, exp_dbl);
            assert_eq!(view.room_rt60_sec, exp_rt);
            assert_eq!(view.sub_bass_mix, exp_sub);
        }

        // LUFS & Ceiling normalization round-trip
        for l in [-24.0, -18.0, -14.0, -9.0, -6.0] {
            let norm = AiAutonomousMasteringView::lufs_to_normalized(l);
            assert!((0.0..=1.0).contains(&norm));
            let back = AiAutonomousMasteringView::normalized_to_lufs(norm);
            assert!((back - l).abs() < 1e-2);
        }
        for c in [-3.0, -1.5, -1.0, -0.3, 0.0] {
            let norm = AiAutonomousMasteringView::ceiling_to_normalized(c);
            assert!((0.0..=1.0).contains(&norm));
            let back = AiAutonomousMasteringView::normalized_to_ceiling(norm);
            assert!((back - c).abs() < 1e-2);
        }

        // Sample evaluation & brickwall ceiling limiting
        view.set_preset(AiMasteringPreset::StreamingPop14Lufs);
        view.bypassed = true;
        assert!((view.evaluate_mastered_sample(0.5, 0, -18.0) - 0.5).abs() < 1e-6);
        view.bypassed = false;
        let mastered = view.evaluate_mastered_sample(0.5, 0, -18.0);
        let ceiling_lin = 10.0_f32.powf(view.ceiling_db / 20.0);
        assert!(mastered.abs() <= ceiling_lin + 1e-5);

        // Spectral contour & room volume estimation
        for f in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let y = view.evaluate_spectral_contour(f);
            assert!((0.0..=1.0).contains(&y));
        }
        assert!((view.estimated_room_volume_m3() - view.room_rt60_sec * 80.0).abs() < 1e-3);

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("AI AUTONOMOUS MASTERING & DRUM REPLACER SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn124_pure_hardware_control_surface_initialization_and_physics() {
        let mut view = HardwareControlSurfaceView::new();
        assert_eq!(view.preset, HardwareControlPreset::Push3EurorackBridge);
        assert_eq!(HARDWARE_SURFACE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                HardwareControlPreset::Push3EurorackBridge,
                "Push 3 + Eurorack 1V/Oct CV/Gate Bridge",
                1.0,
                0.0,
                1.0,
                5.0,
                120.0,
                HardwareControllerProtocol::AbletonPush3Display,
                false,
            ),
            (
                HardwareControlPreset::McuMotorizedConsole,
                "Mackie MCU 9-Fader Motorized Console",
                0.0,
                -2.5,
                1.0,
                5.0,
                124.0,
                HardwareControllerProtocol::MackieControlMotorFaders,
                false,
            ),
            (
                HardwareControlPreset::VintageDinSync24Rig,
                "Vintage DIN Sync24 (24 PPQN) + 5V Gate Rig",
                2.0,
                4.5,
                1.2,
                8.0,
                128.0,
                HardwareControllerProtocol::LaunchpadProGridMatrix,
                false,
            ),
            (
                HardwareControlPreset::NksOscBidirectionalStage,
                "NI NKS 88-Key Light Guide + OSC Stage Rig",
                -1.0,
                1.2,
                1.0,
                5.0,
                110.0,
                HardwareControllerProtocol::KompleteKontrolNksLightGuide,
                true,
            ),
        ];

        for (p, exp_name, exp_cv, exp_off, exp_vpo, exp_gate, exp_bpm, exp_proto, exp_learn) in
            presets
        {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (cv, off, vpo, gate, bpm, proto, learn) = p.nominal_parameters();
            assert_eq!(cv, exp_cv);
            assert_eq!(off, exp_off);
            assert_eq!(vpo, exp_vpo);
            assert_eq!(gate, exp_gate);
            assert_eq!(bpm, exp_bpm);
            assert_eq!(proto, exp_proto);
            assert_eq!(learn, exp_learn);

            assert_eq!(view.cv_voltage, exp_cv);
            assert_eq!(view.clock_offset_ms, exp_off);
            assert_eq!(view.volts_per_octave, exp_vpo);
            assert_eq!(view.gate_high_voltage, exp_gate);
            assert_eq!(view.din_sync_bpm, exp_bpm);
            assert_eq!(view.protocol, exp_proto);
            assert_eq!(view.midi_learn_active, exp_learn);
        }

        // CV Voltage & Clock Offset normalization round-trip
        for v in [-5.0, -2.5, 0.0, 1.0, 5.0] {
            let norm = HardwareControlSurfaceView::cv_to_normalized(v);
            assert!((0.0..=1.0).contains(&norm));
            let back = HardwareControlSurfaceView::normalized_to_cv(norm);
            assert!((back - v).abs() < 1e-2);
        }
        for o in [-25.0, -10.0, 0.0, 5.0, 25.0] {
            let norm = HardwareControlSurfaceView::offset_to_normalized(o);
            assert!((0.0..=1.0).contains(&norm));
            let back = HardwareControlSurfaceView::normalized_to_offset(norm);
            assert!((back - o).abs() < 1e-2);
        }

        // 1V/Oct CV calculation and DIN Sync24 frequency
        view.set_preset(HardwareControlPreset::Push3EurorackBridge);
        assert!((view.note_to_cv_voltage(72.0) - 1.0).abs() < 1e-5); // C4 = +1.0V above C3 (60)
        assert!((view.normalized_dc_audio_level() - 0.2).abs() < 1e-5); // 1.0V / 5.0V = 0.2
        assert!((view.din_sync_pulse_hz() - 48.0).abs() < 1e-5); // 120 BPM = 2 Hz * 24 PPQN = 48 Hz

        for p in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let y = view.evaluate_sync_waveform(p);
            assert!((0.0..=1.0).contains(&y));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("HARDWARE CONTROL SURFACE, CV/GATE & DIN SYNC SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::ai_autonomous_mastering_view::AiAutonomousMasteringView;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::hardware_control_surface_view::HardwareControlSurfaceView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn124_headless_render_views() {
        let ctx = egui::Context::default();
        let mut am_view = AiAutonomousMasteringView::new();
        let mut hw_view = HardwareControlSurfaceView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                am_view.show(ui);
                hw_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn124_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test AI Autonomous Mastering HUD Lifecycle
        assert!(!aw_view.is_ai_autonomous_mastering_hud_open());
        aw_view.open_ai_autonomous_mastering_hud();
        assert!(aw_view.is_ai_autonomous_mastering_hud_open());
        aw_view.sync_ai_autonomous_mastering_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("target_lufs"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("ceiling_db"));
        aw_view.close_ai_autonomous_mastering_hud();
        assert!(!aw_view.is_ai_autonomous_mastering_hud_open());

        // Test Hardware Control Surface HUD Lifecycle
        assert!(!aw_view.is_hardware_control_surface_hud_open());
        aw_view.open_hardware_control_surface_hud();
        assert!(aw_view.is_hardware_control_surface_hud_open());
        aw_view.sync_hardware_control_surface_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("cv_voltage"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("clock_offset_ms"));
        aw_view.close_hardware_control_surface_hud();
        assert!(!aw_view.is_hardware_control_surface_hud_open());
    }

    #[test]
    fn test_turn124_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_ai_mastering = actions
            .iter()
            .any(|a| a.action_id == "open_ai_autonomous_mastering_hud");
        let has_hw_surface = actions
            .iter()
            .any(|a| a.action_id == "open_hardware_control_surface_hud");

        assert!(
            has_ai_mastering,
            "open_ai_autonomous_mastering_hud must be registered in command palette"
        );
        assert!(
            has_hw_surface,
            "open_hardware_control_surface_hud must be registered in command palette"
        );
    }
}
