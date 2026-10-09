#![allow(clippy::all)]

//! Test suite for Turn #109 (Sprint Turn #35) — Linear-Phase Dynamic Transient Crossover
//! De-Clicker & Vinyl Restoration HUD and Dynamic Transient Sidechain Expander/Gate &
//! Envelope Hysteresis Curve HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::transient_declicker_view::{
        TransientDeclickerView, VinylRestorationMode, DECLICKER_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_gate_view::{
        TransientGateView, GateMode, GATE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn109_pure_transient_declicker_initialization_and_kinematics() {
        let mut view = TransientDeclickerView::new();
        assert_eq!(view.mode, VinylRestorationMode::VinylMicrogroove);
        assert_eq!(DECLICKER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Modes and expected values
        let modes = [
            (VinylRestorationMode::VinylMicrogroove, "VINYL 33/45", -18.5, 1.20, 2800.0),
            (VinylRestorationMode::Shellac78Rpm, "78 RPM SHELLAC", -12.0, 2.80, 1200.0),
            (VinylRestorationMode::DigitalClicks, "DIGITAL CLICKS", -24.0, 0.45, 6000.0),
            (VinylRestorationMode::ThumpAndPlop, "THUMP & PLOP", -8.5, 4.50, 250.0),
            (VinylRestorationMode::TapeDropout, "TAPE DROPOUT", -15.0, 3.20, 1800.0),
        ];

        for (m, expected_name, def_thresh, def_width, def_xover) in modes {
            view.mode = m;
            assert_eq!(m.mode_name(), expected_name);
            assert_eq!(m.name(), expected_name);
            assert_eq!(m.default_threshold_db(), def_thresh);
            assert_eq!(m.default_click_width_ms(), def_width);
            assert_eq!(m.linear_phase_crossover_hz(), def_xover);
        }

        // Normalization round-trips for Click Width [0.05 ..= 5.00 ms]
        let width_test = 2.15;
        let width_norm = TransientDeclickerView::width_to_normalized(width_test);
        let width_back = TransientDeclickerView::normalized_to_width(width_norm);
        assert!((width_back - width_test).abs() < 1e-3);

        // Normalization round-trips for Threshold [-48.0 ..= 0.0 dB]
        let thresh_test = -22.5;
        let thresh_norm = TransientDeclickerView::threshold_to_normalized(thresh_test);
        let thresh_back = TransientDeclickerView::normalized_to_threshold(thresh_norm);
        assert!((thresh_back - thresh_test).abs() < 1e-3);

        // Normalization round-trips for Crackle Density [0.0 ..= 100.0 %]
        let density_test = 72.5;
        let density_norm = TransientDeclickerView::density_to_normalized(density_test);
        let density_back = TransientDeclickerView::normalized_to_density(density_norm);
        assert!((density_back - density_test).abs() < 1e-3);

        // Simulation update and SNR improvements
        view.threshold_db = -20.0;
        view.click_width_ms = 1.5;
        view.crackle_density_pct = 50.0;
        view.update_restoration_simulation();
        assert!(view.repair_rate_per_sec >= 1.0);
        assert!(view.snr_improvement_db >= 1.0);

        // Synthetic waveform repair evaluation
        let (damaged, repaired) = view.evaluate_waveform_repair(0.50);
        assert!(damaged != repaired); // Click defect altered

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.declicker_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.declicker_puck_pos.1) * canvas.height;

        assert!(view.hit_test_declicker_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_declicker_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_declicker_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_declicker_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("LINEAR-PHASE DYNAMIC TRANSIENT DE-CLICKER & VINYL RESTORATION HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn109_pure_transient_gate_initialization_and_kinematics() {
        let mut view = TransientGateView::new();
        assert_eq!(view.mode, GateMode::FastPercussiveSnare);
        assert_eq!(GATE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Modes and expected names
        let modes = [
            (GateMode::FastPercussiveSnare, "FAST SNARE / TRANSIENT"),
            (GateMode::VocalBreathSmoothing, "VOCAL BREATH SMOOTHING"),
            (GateMode::BassSubDucking, "BASS SUB DUCKING"),
            (GateMode::HardNoiseGate, "HARD NOISE GATE"),
        ];

        for (m, expected_name) in modes {
            view.mode = m;
            assert_eq!(m.mode_name(), expected_name);
            assert_eq!(m.name(), expected_name);
        }

        // Normalization round-trips for Threshold dB [-80.0 ..= 0.0 dB]
        let db_test = -36.5;
        let db_norm = TransientGateView::db_to_normalized(db_test);
        let db_back = TransientGateView::normalized_to_db(db_norm);
        assert!((db_back - db_test).abs() < 1e-3);

        // Normalization round-trips for Hysteresis dB [0.0 ..= 24.0 dB]
        let hyst_test = 8.4;
        let hyst_norm = TransientGateView::hysteresis_to_normalized(hyst_test);
        let hyst_back = TransientGateView::normalized_to_hysteresis(hyst_norm);
        assert!((hyst_back - hyst_test).abs() < 1e-3);

        // Normalization round-trips for Attack [0.1 ..= 100.0 ms]
        let att_test = 3.5;
        let att_norm = TransientGateView::attack_to_normalized(att_test);
        let att_back = TransientGateView::normalized_to_attack(att_norm);
        assert!((att_back - att_test).abs() < 1e-3);

        // Normalization round-trips for Release [5.0 ..= 2000.0 ms]
        let rel_test = 250.0;
        let rel_norm = TransientGateView::release_to_normalized(rel_test);
        let rel_back = TransientGateView::normalized_to_release(rel_norm);
        assert!((rel_back - rel_test).abs() < 1e-3);

        // Close threshold calculation
        view.open_threshold_db = -30.0;
        view.hysteresis_db = 6.0;
        assert_eq!(view.close_threshold_db(), -36.0);

        // Transfer gain evaluations
        let gain_open = view.evaluate_transfer_gain(-20.0, true);
        assert_eq!(gain_open, -20.0); // 0 dB attenuation when above threshold

        let gain_closed = view.evaluate_transfer_gain(-35.0, false);
        assert!(gain_closed < -35.0); // attenuated by expansion slope below open threshold
        let gain_floor = view.evaluate_transfer_gain(-60.0, false);
        assert_eq!(gain_floor, view.range_floor_db); // clamped by range floor

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 100.0, 440.0, 230.0);
        let puck_x = canvas.x + view.gate_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.gate_puck_pos.1) * canvas.height;

        assert!(view.hit_test_gate_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_gate_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_gate_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_gate_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DYNAMIC TRANSIENT SIDECHAIN EXPANDER/GATE & HYSTERESIS HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn109_gui_transient_declicker_and_gate_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Transient De-Clicker Modal Lifecycle
        assert!(!view.is_transient_declicker_hud_open());
        view.open_transient_declicker_hud();
        assert!(view.is_transient_declicker_hud_open());
        view.close_transient_declicker_hud();
        assert!(!view.is_transient_declicker_hud_open());

        // 2. Transient Gate Modal Lifecycle
        assert!(!view.is_transient_gate_hud_open());
        view.open_transient_gate_hud();
        assert!(view.is_transient_gate_hud_open());
        view.close_transient_gate_hud();
        assert!(!view.is_transient_gate_hud_open());

        // 3. Transient De-Clicker Parameter Sync Bridge
        view.transient_declicker_view.threshold_db = -19.5;
        view.transient_declicker_view.click_width_ms = 1.35;
        view.transient_declicker_view.crackle_density_pct = 75.0;
        view.transient_declicker_view.snr_improvement_db = 15.5;
        view.transient_declicker_view.repair_rate_per_sec = 160.0;
        view.sync_transient_declicker_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("threshold_db").copied(),
            Some(-19.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("click_width_ms").copied(),
            Some(1.35)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("crackle_density_pct").copied(),
            Some(75.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("threshold_db").copied(),
            Some(-19.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("click_width_ms").copied(),
            Some(1.35)
        );

        // 4. Transient Gate Parameter Sync Bridge
        view.transient_gate_view.open_threshold_db = -28.0;
        view.transient_gate_view.hysteresis_db = 7.5;
        view.transient_gate_view.attack_ms = 1.5;
        view.transient_gate_view.release_ms = 120.0;
        view.transient_gate_view.range_floor_db = -60.0;
        view.transient_gate_view.sidechain_hpf_hz = 150.0;
        view.sync_transient_gate_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("open_threshold_db").copied(),
            Some(-28.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("hysteresis_db").copied(),
            Some(7.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("attack_ms").copied(),
            Some(1.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("release_ms").copied(),
            Some(120.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("open_threshold_db").copied(),
            Some(-28.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("hysteresis_db").copied(),
            Some(7.5)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_transient_declicker_modal_window(ui);
                view.show_transient_gate_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let declick_action = palette.actions.iter().find(|a| a.action_id == "open_transient_declicker_hud");
        assert!(declick_action.is_some(), "open_transient_declicker_hud should be registered");
        assert_eq!(declick_action.unwrap().category, "Restoration & Dynamics");

        let gate_action = palette.actions.iter().find(|a| a.action_id == "open_transient_gate_hud");
        assert!(gate_action.is_some(), "open_transient_gate_hud should be registered");
        assert_eq!(gate_action.unwrap().category, "Dynamics & Saturation");
    }
}
