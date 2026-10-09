#![allow(clippy::all)]

//! Test suite for Turn #111 (Sprint Turn #37) — Psychoacoustic Dynamic Stereo
//! Transient Unwrapper & Spatial Depth Decorrelator HUD and Master Bus Multi-Point
//! Spectral Dynamics Unmasker & Sidechain Collision Heatmap HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::transient_unwrapper_view::{
        TransientUnwrapperView, UnwrapperMode, TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_unmasker_view::{
        SpectralUnmaskerView, UnmaskerRouting, UNMASKER_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn111_pure_transient_unwrapper_initialization_and_kinematics() {
        let mut view = TransientUnwrapperView::new();
        assert_eq!(view.mode, UnwrapperMode::WideStereoExpansion);
        assert_eq!(TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Modes and expected values
        let modes = [
            (UnwrapperMode::WideStereoExpansion, "WIDE STEREO EXPANSION", 140.0, 4.5, 90.0),
            (UnwrapperMode::BinauralDepthExtraction, "BINAURAL DEPTH EXTRACTION", 110.0, 8.2, 120.0),
            (UnwrapperMode::DrumTransientDecomb, "DRUM TRANSIENT DE-COMB", 100.0, 1.8, 60.0),
            (UnwrapperMode::MasteringSpatialUnwrap, "MASTERING SPATIAL UNWRAP", 125.0, 3.2, 140.0),
            (UnwrapperMode::MicrotonalPhaseDecorrelate, "POLY DECORRELATE", 160.0, 12.0, 80.0),
        ];

        for (m, expected_name, exp_width, exp_delay, exp_crossover) in modes {
            view.set_mode(m);
            assert_eq!(m.name(), expected_name);
            assert_eq!(m.default_width_pct(), exp_width);
            assert_eq!(m.default_decorrelation_delay_ms(), exp_delay);
            assert_eq!(m.default_mono_crossover_hz(), exp_crossover);
            assert_eq!(view.spatial_width_pct, exp_width);
            assert_eq!(view.decorrelation_delay_ms, exp_delay);
            assert_eq!(view.mono_crossover_hz, exp_crossover);
        }

        // Normalization round-trips for Width [0.0 ..= 200.0 %]
        let width_test = 145.5;
        let width_norm = TransientUnwrapperView::width_to_normalized(width_test);
        let width_back = TransientUnwrapperView::normalized_to_width(width_norm);
        assert!((width_back - width_test).abs() < 1e-3);

        // Normalization round-trips for Delay [0.0 ..= 25.0 ms]
        let delay_test = 6.75;
        let delay_norm = TransientUnwrapperView::delay_to_normalized(delay_test);
        let delay_back = TransientUnwrapperView::normalized_to_delay(delay_norm);
        assert!((delay_back - delay_test).abs() < 1e-3);

        // Normalization round-trips for Angle [-90.0 ..= 90.0 deg]
        let angle_test = 22.5;
        let angle_norm = TransientUnwrapperView::angle_to_normalized(angle_test);
        let angle_back = TransientUnwrapperView::normalized_to_angle(angle_norm);
        assert!((angle_back - angle_test).abs() < 1e-3);

        // Normalization round-trips for Sensitivity [0.0 ..= 1.0]
        let sens_test = 0.85;
        let sens_norm = TransientUnwrapperView::sensitivity_to_normalized(sens_test);
        let sens_back = TransientUnwrapperView::normalized_to_sensitivity(sens_norm);
        assert!((sens_back - sens_test).abs() < 1e-3);

        // Simulation update & IACC model math
        view.spatial_width_pct = 150.0;
        view.decorrelation_delay_ms = 5.0;
        view.transient_sensitivity = 0.80;
        view.update_psychoacoustic_simulation();
        assert!(view.iacc_correlation < 1.0);
        assert!(view.side_channel_gain_db > 0.0);

        // Stereo spread Lissajous evaluation
        let (l, r) = view.evaluate_stereo_spread(0.5);
        assert!(l.abs() > 0.0);
        assert!(r.abs() > 0.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.unwrapper_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.unwrapper_puck_pos.1) * canvas.height;

        assert!(view.hit_test_unwrapper_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_unwrapper_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_unwrapper_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_unwrapper_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PSYCHOACOUSTIC DYNAMIC STEREO TRANSIENT UNWRAPPER & DECORRELATOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn111_pure_spectral_unmasker_initialization_and_kinematics() {
        let mut view = SpectralUnmaskerView::new();
        assert_eq!(view.routing, UnmaskerRouting::KickVsBass);
        assert_eq!(UNMASKER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Routings and expected values
        let routings = [
            (UnmaskerRouting::KickVsBass, "KICK vs BASS", 68.4, 3.5, 5.2),
            (UnmaskerRouting::VocalVsSynth, "VOCAL vs SYNTH", 1850.0, 1.8, 4.0),
            (UnmaskerRouting::SnareVsGuitar, "SNARE vs GUITAR", 240.0, 2.2, 3.5),
            (UnmaskerRouting::DialogVsBgm, "DIALOG vs BGM", 1200.0, 1.2, 6.5),
            (UnmaskerRouting::CustomBus, "CUSTOM BUS", 500.0, 2.0, 4.5),
        ];

        for (r, expected_name, exp_freq, exp_q, exp_red) in routings {
            view.routing = r;
            assert_eq!(r.name(), expected_name);
            assert_eq!(r.target_center_freq_hz(), exp_freq);
            assert_eq!(r.target_q_factor(), exp_q);
            assert_eq!(r.nominal_reduction_db(), exp_red);
        }

        // Normalization round-trips for Frequency [20.0 ..= 20000.0 Hz] (logarithmic)
        let freq_test = 1250.0;
        let freq_norm = SpectralUnmaskerView::freq_to_normalized(freq_test);
        let freq_back = SpectralUnmaskerView::normalized_to_freq(freq_norm);
        assert!((freq_back - freq_test).abs() < 1.0);

        // Normalization round-trips for Depth [0.0 ..= 18.0 dB]
        let depth_test = 8.5;
        let depth_norm = SpectralUnmaskerView::depth_to_normalized(depth_test);
        let depth_back = SpectralUnmaskerView::normalized_to_depth(depth_norm);
        assert!((depth_back - depth_test).abs() < 1e-3);

        // Normalization round-trips for Sensitivity [0.0 ..= 100.0 %]
        let sens_test = 65.0;
        let sens_norm = SpectralUnmaskerView::sensitivity_to_normalized(sens_test);
        let sens_back = SpectralUnmaskerView::normalized_to_sensitivity(sens_norm);
        assert!((sens_back - sens_test).abs() < 1e-3);

        // Simulation update & filter response calculations
        view.collision_freq_hz = 180.0;
        view.reduction_depth_db = 6.0;
        view.sensitivity_pct = 80.0;
        view.update_unmasking_calculations();
        assert!(view.collision_intensity_score > 0.0);
        assert!(view.clarity_gain_pct > 70.0);

        // Evaluate filter response (bell notch carve)
        let notch_center = view.evaluate_unmasking_filter_response(180.0);
        let notch_away = view.evaluate_unmasking_filter_response(2000.0);
        assert!(notch_center < notch_away); // deeper cut at resonant center

        // Evaluate collision bins
        let (coll, gr) = view.evaluate_collision_bin(16, 32);
        assert!(coll >= 0.0);
        assert!(gr <= 0.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.unmasker_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.unmasker_puck_pos.1) * canvas.height;

        assert!(view.hit_test_unmasker_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_unmasker_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_unmasker_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_unmasker_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTER BUS MULTI-POINT SPECTRAL UNMASKER & SIDECHAIN COLLISION HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn111_gui_unwrapper_and_unmasker_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Transient Unwrapper Modal Lifecycle
        assert!(!view.is_transient_unwrapper_hud_open());
        view.open_transient_unwrapper_hud();
        assert!(view.is_transient_unwrapper_hud_open());
        view.close_transient_unwrapper_hud();
        assert!(!view.is_transient_unwrapper_hud_open());

        // 2. Spectral Unmasker Modal Lifecycle
        assert!(!view.is_spectral_unmasker_hud_open());
        view.open_spectral_unmasker_hud();
        assert!(view.is_spectral_unmasker_hud_open());
        view.close_spectral_unmasker_hud();
        assert!(!view.is_spectral_unmasker_hud_open());

        // 3. Transient Unwrapper Parameter Sync Bridge
        view.transient_unwrapper_view.spatial_width_pct = 145.0;
        view.transient_unwrapper_view.decorrelation_delay_ms = 5.2;
        view.transient_unwrapper_view.unwrap_angle_deg = 18.0;
        view.transient_unwrapper_view.mono_crossover_hz = 110.0;
        view.transient_unwrapper_view.transient_sensitivity = 0.78;
        view.sync_transient_unwrapper_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("spatial_width_pct").copied(),
            Some(145.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("decorrelation_delay_ms").copied(),
            Some(5.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("spatial_width_pct").copied(),
            Some(145.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("decorrelation_delay_ms").copied(),
            Some(5.2)
        );

        // 4. Spectral Unmasker Parameter Sync Bridge
        view.spectral_unmasker_view.collision_freq_hz = 1450.0;
        view.spectral_unmasker_view.reduction_depth_db = 6.5;
        view.spectral_unmasker_view.sensitivity_pct = 82.0;
        view.spectral_unmasker_view.clarity_gain_pct = 95.2;
        view.spectral_unmasker_view.attack_ms = 5.0;
        view.spectral_unmasker_view.release_ms = 70.0;
        view.sync_spectral_unmasker_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("collision_freq_hz").copied(),
            Some(1450.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("reduction_depth_db").copied(),
            Some(6.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("collision_freq_hz").copied(),
            Some(1450.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("reduction_depth_db").copied(),
            Some(6.5)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_transient_unwrapper_modal_window(ui);
                view.show_spectral_unmasker_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let tu_action = palette.actions.iter().find(|a| a.action_id == "open_transient_unwrapper_hud");
        assert!(tu_action.is_some(), "open_transient_unwrapper_hud should be registered");
        assert_eq!(tu_action.unwrap().category, "Spatial & Binaural");

        let su_action = palette.actions.iter().find(|a| a.action_id == "open_spectral_unmasker_hud");
        assert!(su_action.is_some(), "open_spectral_unmasker_hud should be registered");
        assert_eq!(su_action.unwrap().category, "Spectral & Synthesis");
    }
}
