#![allow(clippy::all)]

//! Test suite for Turn #117 (Sprint Turn #43) — Physical Modeling Organ Windchest Aerodynamics &
//! Dynamic Bellows Reservoir HUD and Vinyl Turntable Inertial Motor Brake Tape Stop & Stutter Chopper HUD
//! Convergence (Milestones 14, 24, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::windchest_reservoir_view::{
        WindchestPreset, WindchestReservoirView, WINDCHEST_PUCK_HIT_RADIUS,
        NUM_PRESSURE_PLOT_POINTS, MMH2O_TO_PASCALS, AIR_DENSITY_RHO,
    };
    use crate::views::tape_stop_glitch_view::{
        TapeStopPreset, TapeStopGlitchView, TAPESTOP_PUCK_HIT_RADIUS,
        NUM_CURVE_PLOT_POINTS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn117_pure_windchest_reservoir_initialization_and_physics() {
        let mut view = WindchestReservoirView::new();
        assert_eq!(view.preset, WindchestPreset::BaroqueSchnitger);
        assert_eq!(WINDCHEST_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (WindchestPreset::BaroqueSchnitger, "Baroque Schnitger (Wedge Bellows, 1690)", 68.0, 0.012, 0.18, 4.5, 0.15),
            (WindchestPreset::RomanticCavailleColl, "Romantic Cavaillé-Coll (Barker Lever, 1880)", 115.0, 0.005, 0.08, 5.8, 0.08),
            (WindchestPreset::EnglishCathedral, "English Cathedral (Heavy Regulator, 1895)", 95.0, 0.003, 0.05, 3.8, 0.10),
            (WindchestPreset::ModernChamberPortative, "Modern Portative (Direct Hand Bellows)", 48.0, 0.035, 0.32, 6.2, 0.22),
        ];

        for (p, exp_name, exp_pres, exp_sag, exp_rec, exp_rate, exp_depth) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (pres, sag, rec, rate, depth) = p.nominal_parameters();
            assert_eq!(pres, exp_pres);
            assert_eq!(sag, exp_sag);
            assert_eq!(rec, exp_rec);
            assert_eq!(rate, exp_rate);
            assert_eq!(depth, exp_depth);

            assert_eq!(view.nominal_pressure_mmh2o, exp_pres);
            assert_eq!(view.sag_coefficient, exp_sag);
            assert_eq!(view.recovery_time_sec, exp_rec);
            assert_eq!(view.tremulant_rate_hz, exp_rate);
            assert_eq!(view.tremulant_depth, exp_depth);
            assert!(view.dynamic_pressure_mmh2o > 0.0);
            assert!(view.dynamic_pressure_pascals > 0.0);
            assert!(view.flue_jet_velocity_mps > 0.0);
            assert!(view.reservoir_bellows_sag_pct >= 0.0 && view.reservoir_bellows_sag_pct <= 100.0);
        }

        // Normalization round-trips for Pressure [40.0 ..= 160.0 mmH2O]
        let pres_test = 82.5;
        let pres_norm = WindchestReservoirView::pressure_to_normalized(pres_test);
        let pres_back = WindchestReservoirView::normalized_to_pressure(pres_norm);
        assert!((pres_back - pres_test).abs() < 1e-4);

        // Normalization round-trips for Sag Compliance [0.001 ..= 0.050]
        let sag_test = 0.024;
        let sag_norm = WindchestReservoirView::sag_to_normalized(sag_test);
        let sag_back = WindchestReservoirView::normalized_to_sag(sag_norm);
        assert!((sag_back - sag_test).abs() < 1e-5);

        // Flue jet velocity physics validation: v = sqrt(2 * P_pascals / rho)
        let p_pascal = view.dynamic_pressure_mmh2o * MMH2O_TO_PASCALS;
        let expected_v = (2.0 * p_pascal / AIR_DENSITY_RHO).sqrt();
        assert!((view.flue_jet_velocity_mps - expected_v).abs() < 0.01);

        // Dynamic pressure curve evaluation (64 points)
        let curve = view.evaluate_pressure_curve();
        assert_eq!(curve.len(), NUM_PRESSURE_PLOT_POINTS);
        for &p in &curve {
            assert!(p > 0.0 && p <= 200.0);
        }

        // Tremulant toggle test
        view.tremulant_enabled = true;
        view.update_physics();
        assert!(view.dynamic_pressure_mmh2o > 0.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 340.0, 200.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_windchest_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_windchest_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_windchest_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_windchest_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ORGAN WINDCHEST AERODYNAMICS & BELLOWS RESERVOIR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn117_pure_tape_stop_glitch_initialization_and_physics() {
        let mut view = TapeStopGlitchView::new();
        assert_eq!(view.preset, TapeStopPreset::DirectDriveVinylBrake);
        assert_eq!(TAPESTOP_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (TapeStopPreset::DirectDriveVinylBrake, "Direct Drive Vinyl Brake (SL-1200 Classic)", 650.0, 350.0, 16.0, 0.50, 1.8),
            (TapeStopPreset::FastTapeSlamStop, "Fast Tape Slam Stop (Reel-to-Reel Pinch)", 120.0, 80.0, 32.0, 0.70, 0.9),
            (TapeStopPreset::SlowDubSpinDown, "Slow Dub Spin Down (Belt-Drive Momentum)", 1450.0, 750.0, 8.0, 0.35, 2.4),
            (TapeStopPreset::HyperStutterGlitch, "Hyper Stutter Glitch (Buffer Override)", 250.0, 100.0, 48.0, 0.85, 1.2),
        ];

        for (p, exp_name, exp_brk, exp_strt, exp_div, exp_pw, exp_crv) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (brk, strt, div, pw, crv) = p.nominal_parameters();
            assert_eq!(brk, exp_brk);
            assert_eq!(strt, exp_strt);
            assert_eq!(div, exp_div);
            assert_eq!(pw, exp_pw);
            assert_eq!(crv, exp_crv);

            assert_eq!(view.brake_duration_ms, exp_brk);
            assert_eq!(view.start_duration_ms, exp_strt);
            assert_eq!(view.stutter_division, exp_div);
            assert_eq!(view.stutter_pulse_width, exp_pw);
            assert_eq!(view.deceleration_curvature, exp_crv);
            assert!(view.effective_stutter_ms > 0.0);
        }

        // Normalization round-trips for Brake Duration [20.0 ..= 2000.0 ms]
        let brk_test = 820.0;
        let brk_norm = TapeStopGlitchView::brake_to_normalized(brk_test);
        let brk_back = TapeStopGlitchView::normalized_to_brake(brk_norm);
        assert!((brk_back - brk_test).abs() < 1e-3);

        // Normalization round-trips for Division [1.0 ..= 64.0]
        let div_test = 24.0;
        let div_norm = TapeStopGlitchView::division_to_normalized(div_test);
        let div_back = TapeStopGlitchView::normalized_to_division(div_norm);
        assert!((div_back - div_test).abs() < 1e-4);

        // Brake trigger and pitch dive physics
        view.trigger_brake();
        assert!(view.is_stopped);
        assert_eq!(view.current_speed_ratio, 0.0);
        assert_eq!(view.pitch_drop_semitones, -96.0);

        view.trigger_start();
        assert!(!view.is_stopped);
        assert_eq!(view.current_speed_ratio, 1.0);
        assert_eq!(view.pitch_drop_semitones, 0.0);

        // Half speed gives exactly -12 semitones (one octave)
        view.current_speed_ratio = 0.5;
        view.update_physics();
        assert!((view.pitch_drop_semitones - (-12.0)).abs() < 1e-4);

        // Deceleration curve evaluation (64 points)
        let curve = view.evaluate_deceleration_curve();
        assert_eq!(curve.len(), NUM_CURVE_PLOT_POINTS);
        assert_eq!(curve[0], 1.0); // full speed at t=0
        assert_eq!(curve[NUM_CURVE_PLOT_POINTS - 1], 0.0); // complete stop at t=1
        for &v in &curve {
            assert!(v >= 0.0 && v <= 1.0);
        }

        // Stutter toggle
        assert!(!view.stutter_enabled);
        view.toggle_stutter();
        assert!(view.stutter_enabled);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 340.0, 200.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_tapestop_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_tapestop_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_tapestop_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_tapestop_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("VINYL TURNTABLE TAPE STOP & STUTTER CHOPPER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn117_gui_windchest_and_tapestop_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Windchest Reservoir Modal Lifecycle
        assert!(!view.is_windchest_reservoir_hud_open());
        view.open_windchest_reservoir_hud();
        assert!(view.is_windchest_reservoir_hud_open());
        view.close_windchest_reservoir_hud();
        assert!(!view.is_windchest_reservoir_hud_open());

        // 2. Tape Stop Glitch Modal Lifecycle
        assert!(!view.is_tape_stop_glitch_hud_open());
        view.open_tape_stop_glitch_hud();
        assert!(view.is_tape_stop_glitch_hud_open());
        view.close_tape_stop_glitch_hud();
        assert!(!view.is_tape_stop_glitch_hud_open());

        // 3. Windchest Parameter Sync Bridge
        view.windchest_reservoir_view.nominal_pressure_mmh2o = 110.0;
        view.windchest_reservoir_view.sag_coefficient = 0.008;
        view.windchest_reservoir_view.recovery_time_sec = 0.12;
        view.windchest_reservoir_view.tremulant_depth = 0.18;
        view.sync_windchest_reservoir_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("nominal_pressure_mmh2o").copied(),
            Some(110.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("sag_coefficient").copied(),
            Some(0.008)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("nominal_pressure_mmh2o").copied(),
            Some(110.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("sag_coefficient").copied(),
            Some(0.008)
        );

        // 4. Tape Stop Parameter Sync Bridge
        view.tape_stop_glitch_view.brake_duration_ms = 450.0;
        view.tape_stop_glitch_view.start_duration_ms = 220.0;
        view.tape_stop_glitch_view.stutter_division = 32.0;
        view.tape_stop_glitch_view.current_speed_ratio = 0.75;
        view.sync_tape_stop_glitch_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("brake_duration_ms").copied(),
            Some(450.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("stutter_division").copied(),
            Some(32.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("brake_duration_ms").copied(),
            Some(450.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("stutter_division").copied(),
            Some(32.0)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_windchest_reservoir_modal_window(ui);
                view.show_tape_stop_glitch_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let wind_action = palette.actions.iter().find(|a| a.action_id == "open_windchest_reservoir_hud");
        assert!(wind_action.is_some(), "open_windchest_reservoir_hud should be registered");
        assert_eq!(wind_action.unwrap().category, "Physical Modeling");

        let tapestop_action = palette.actions.iter().find(|a| a.action_id == "open_tape_stop_glitch_hud");
        assert!(tapestop_action.is_some(), "open_tape_stop_glitch_hud should be registered");
        assert_eq!(tapestop_action.unwrap().category, "Creative & Glitch");
    }
}
