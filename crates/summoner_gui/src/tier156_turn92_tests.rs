#![allow(clippy::all)]

//! Test suite for Turn #92 (Sprint Turn #18 — Sprint Review) — Vacuum Tube Triode Bias HUD
//! and Analog Reel-to-Reel Tape Saturation & Magnetic Hysteresis HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::tube_bias_view::{
        TubeBiasView, TubeTopology, TUBE_PUCK_HIT_RADIUS, NUM_HARMONICS,
    };
    use crate::views::tape_saturation_view::{
        TapeSaturationView, TapeViewSpeed, TAPE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn92_pure_tube_bias_initialization_and_kinematics() {
        let mut view = TubeBiasView::new();
        assert_eq!(view.topology, TubeTopology::Triode12AX7);
        assert!((view.bias_voltage_v - (-1.85)).abs() < 1e-3);
        assert!((view.plate_voltage_v - 250.0).abs() < 1e-3);
        assert!((view.drive_warmth_db - 8.5).abs() < 1e-3);
        assert!((view.sag_compression_pct - 35.0).abs() < 1e-3);
        assert!((view.asymmetry_balance_pct - 65.0).abs() < 1e-3);
        assert!((view.dry_wet_pct - 100.0).abs() < 1e-3);
        assert_eq!(TUBE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target
        assert_eq!(NUM_HARMONICS, 5);

        // Topology switching & labels
        let topologies = [
            (TubeTopology::Triode12AX7, "12AX7 TRIODE (VINTAGE WARMTH)"),
            (TubeTopology::PentodeEL34, "EL34 PENTODE (BRITISH PUNCH)"),
            (TubeTopology::BeamTetrode6L6, "6L6 BEAM TETRODE (AMERICAN HEADROOM)"),
            (TubeTopology::CleanModern, "CLEAN MODERN (TRANSPARENT SAT)"),
        ];
        for (topo, label) in topologies {
            view.set_topology(topo);
            assert_eq!(view.topology, topo);
            assert_eq!(topo.label(), label);
        }

        // Coordinate normalizations & round-trips for Grid Bias [-4.0..0.0 V]
        let bias_test = -2.75;
        let bias_norm = TubeBiasView::bias_to_normalized(bias_test);
        let bias_back = TubeBiasView::normalized_to_bias(bias_norm);
        assert!((bias_back - bias_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Drive Warmth [0.0..24.0 dB]
        let drive_test = 14.5;
        let drive_norm = TubeBiasView::drive_to_normalized(drive_test);
        let drive_back = TubeBiasView::normalized_to_drive(drive_norm);
        assert!((drive_back - drive_test).abs() < 1e-3);

        // Harmonic distortion spectrum & THD calculation
        let spectrum = view.calculate_harmonic_spectrum();
        assert_eq!(spectrum.len(), 5);
        assert_eq!(spectrum[0], 0.0); // Fundamental f0 is 0 dB
        let thd = view.calculate_thd_pct();
        assert!(thd > 0.0 && thd <= 45.0);

        // Non-linear transfer sample evaluation
        let sat_pos = TubeBiasView::tube_transfer_sample(0.8, -1.85, 8.5, 65.0);
        let sat_neg = TubeBiasView::tube_transfer_sample(-0.8, -1.85, 8.5, 65.0);
        assert!(sat_pos > 0.0 && sat_pos <= 1.0);
        assert!(sat_neg < 0.0 && sat_neg >= -1.0);

        // Q-point puck hit testing
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);
        let puck_x = canvas.x + view.q_point_norm.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.q_point_norm.1) * canvas.height;
        assert!(view.hit_test_bias_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_bias_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("TUBE BIAS"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn92_pure_tape_saturation_initialization_and_kinematics() {
        let mut view = TapeSaturationView::new();
        assert_eq!(view.speed, TapeViewSpeed::Ips15);
        assert!((view.drive - 2.0).abs() < 1e-3);
        assert!((view.saturation - 0.6).abs() < 1e-3);
        assert!((view.bias - 0.0).abs() < 1e-3);
        assert!((view.head_bump - 0.5).abs() < 1e-3);
        assert!((view.wow_flutter - 0.15).abs() < 1e-3);
        assert!((view.hysteresis - 0.5).abs() < 1e-3);
        assert!((view.tone - 0.85).abs() < 1e-3);
        assert!((view.output_gain - 1.0).abs() < 1e-3);
        assert!((view.dry_wet - 1.0).abs() < 1e-3);
        assert_eq!(TAPE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Tape speed modes & metadata
        let speeds = [
            (TapeViewSpeed::Ips7_5, "7.5 IPS (VINTAGE)", 7.5, 50.0),
            (TapeViewSpeed::Ips15, "15 IPS (STUDIO)", 15.0, 80.0),
            (TapeViewSpeed::Ips30, "30 IPS (MASTER)", 30.0, 120.0),
        ];
        for (spd, label, ips, head_bump_freq) in speeds {
            view.set_speed(spd);
            assert_eq!(view.speed, spd);
            assert_eq!(spd.label(), label);
            assert!((spd.ips_value() - ips).abs() < 1e-3);
            assert!((spd.head_bump_freq() - head_bump_freq).abs() < 1e-3);
        }

        // Coordinate normalizations & round-trips for Drive [0.1..10.0x]
        let drv_test = 4.5;
        let drv_norm = TapeSaturationView::drive_to_normalized(drv_test);
        let drv_back = TapeSaturationView::normalized_to_drive(drv_norm);
        assert!((drv_back - drv_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Saturation [0.0..1.0]
        let sat_test = 0.75;
        let sat_norm = TapeSaturationView::saturation_to_normalized(sat_test);
        let sat_back = TapeSaturationView::normalized_to_saturation(sat_norm);
        assert!((sat_back - sat_test).abs() < 1e-3);

        // Hysteresis curve calculation (128 points)
        let curve = view.compute_hysteresis_points(128);
        assert_eq!(curve.len(), 128);
        for &(h, b) in &curve {
            assert!(h >= -2.0 && h <= 2.0);
            assert!(b >= -2.0 && b <= 2.0);
        }

        // Tape puck hit testing
        let canvas = Rect::new(16.0, 48.0, 360.0, 400.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_tape_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_tape_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ANALOG TAPE SATURATION & HYSTERESIS HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn92_gui_modal_window_lifecycles_tube_bias_and_tape_saturation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Vacuum Tube Bias HUD
        assert!(!view.is_tube_bias_hud_open());
        view.open_tube_bias_hud();
        assert!(view.is_tube_bias_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_tube_bias_hud();
        assert!(!view.is_tube_bias_hud_open());

        // 2. Analog Tape Saturation HUD
        assert!(!view.is_tape_saturation_hud_open());
        view.open_tape_saturation_hud();
        assert!(view.is_tape_saturation_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_tape_saturation_hud();
        assert!(!view.is_tape_saturation_hud_open());
    }

    #[test]
    fn test_turn92_gui_lock_free_param_bus_dispatch_tube_bias_and_tape_saturation() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Tube Bias (offsets 0..=5) and Tape Saturation (offsets 0..=7)
        for offset in 0..=7 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Tube Bias sync
        view.tube_bias_view.bias_voltage_v = -2.25;
        view.tube_bias_view.plate_voltage_v = 300.0;
        view.tube_bias_view.drive_warmth_db = 12.0;
        view.tube_bias_view.sag_compression_pct = 40.0;
        view.tube_bias_view.asymmetry_balance_pct = 70.0;
        view.tube_bias_view.dry_wet_pct = 90.0;

        view.sync_tube_bias_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("bias_voltage_v"), Some(&-2.25));
        assert_eq!(view.device_rack_state.node_param_values.get("plate_voltage_v"), Some(&300.0));
        assert_eq!(view.device_rack_state.node_param_values.get("drive_warmth_db"), Some(&12.0));
        assert_eq!(view.device_rack_state.node_param_values.get("sag_compression_pct"), Some(&40.0));
        assert_eq!(view.device_rack_state.node_param_values.get("asymmetry_balance_pct"), Some(&70.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet_pct"), Some(&90.0));

        let pid_bias = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_plate = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_bias), Some(-2.25));
        assert_eq!(arc_bus.get(pid_plate), Some(300.0));

        // Test Tape Saturation sync
        view.tape_saturation_view.drive = 3.5;
        view.tape_saturation_view.saturation = 0.8;
        view.tape_saturation_view.bias = 0.25;
        view.tape_saturation_view.head_bump = 0.65;
        view.tape_saturation_view.wow_flutter = 0.2;
        view.tape_saturation_view.hysteresis = 0.6;
        view.tape_saturation_view.tone = 0.75;
        view.tape_saturation_view.dry_wet = 85.0;

        view.sync_tape_saturation_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("drive"), Some(&3.5));
        assert_eq!(view.device_rack_state.node_param_values.get("saturation"), Some(&0.8));
        assert_eq!(view.device_rack_state.node_param_values.get("bias"), Some(&0.25));
        assert_eq!(view.device_rack_state.node_param_values.get("head_bump"), Some(&0.65));
        assert_eq!(view.device_rack_state.node_param_values.get("wow_flutter"), Some(&0.2));
        assert_eq!(view.device_rack_state.node_param_values.get("hysteresis"), Some(&0.6));
        assert_eq!(view.device_rack_state.node_param_values.get("tone"), Some(&0.75));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet"), Some(&85.0));

        let pid_drv = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_sat = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_drv), Some(3.5));
        assert_eq!(arc_bus.get(pid_sat), Some(0.8));
    }

    #[test]
    fn test_turn92_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Tube Bias trigger propagation from rack
        view.device_rack_state.requested_open_tube_bias_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_tube_bias_hud);
        assert!(view.is_tube_bias_hud_open());
        view.close_tube_bias_hud();

        // 2. Tape Saturation trigger propagation from inspector
        view.inspector_state.requested_open_tape_saturation_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_tape_saturation_hud);
        assert!(view.is_tape_saturation_hud_open());
        view.close_tape_saturation_hud();
    }

    #[test]
    fn test_turn92_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let tube_action = palette.actions.iter().find(|a| a.action_id == "open_tube_bias_hud");
        assert!(tube_action.is_some(), "open_tube_bias_hud must be registered in command palette");
        let tube = tube_action.unwrap();
        assert_eq!(tube.category, "Distortion / Saturation");

        let tape_action = palette.actions.iter().find(|a| a.action_id == "open_tape_saturation_hud");
        assert!(tape_action.is_some(), "open_tape_saturation_hud must be registered in command palette");
        let tape = tape_action.unwrap();
        assert_eq!(tape.category, "Distortion / Saturation");
    }
}
