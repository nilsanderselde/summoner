#![allow(clippy::all)]

//! Test suite for Turn #104 (Sprint Turn #30) — Mastering Linear-Phase Dynamic Multiband Clipper HUD
//! and 4-Band Dynamic Expander & Noise Gate HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::multiband_clipper_view::{
        MultibandClipperView, ClipperCurveMode, NUM_CLIPPER_BANDS, CLIPPER_KNEE_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_expander_view::{
        MultibandExpanderView, ExpanderMode, NUM_EXPANDER_BANDS, EXPANDER_NODE_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn104_pure_multiband_clipper_initialization_and_kinematics() {
        let mut view = MultibandClipperView::new();
        assert_eq!(view.active_band_idx, 1);
        assert_eq!(view.oversampling_factor, 4);
        assert_eq!(CLIPPER_KNEE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target
        assert_eq!(view.bands.len(), NUM_CLIPPER_BANDS);

        // Curve mode names and switching
        let modes = [
            (ClipperCurveMode::SoftKneeCubic, "Soft-Knee Cubic"),
            (ClipperCurveMode::HyperbolicTanh, "Analog Tanh"),
            (ClipperCurveMode::HardBrickwall, "Hard Brickwall"),
            (ClipperCurveMode::QuinticSmooth, "Quintic Smooth"),
            (ClipperCurveMode::AsymmetricTube, "Asymmetric Tube"),
        ];

        for (m, expected_name) in modes {
            view.set_curve_mode(m);
            assert_eq!(view.curve_mode, m);
            assert_eq!(m.name(), expected_name);
        }

        // Active band switching
        for i in 0..NUM_CLIPPER_BANDS {
            view.set_active_band(i);
            assert_eq!(view.active_band_idx, i);
        }

        // Coordinate normalizations & round-trips for Threshold [-24.0 ..= 0.0 dB]
        let th_test = -14.5;
        let th_norm = MultibandClipperView::threshold_to_normalized(th_test);
        let th_back = MultibandClipperView::normalized_to_threshold(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Ceiling [-12.0 ..= 0.0 dB]
        let cl_test = -1.5;
        let cl_norm = MultibandClipperView::ceiling_to_normalized(cl_test);
        let cl_back = MultibandClipperView::normalized_to_ceiling(cl_norm);
        assert!((cl_back - cl_test).abs() < 1e-3);

        // Knee Width [0.0 ..= 12.0 dB]
        let kn_test = 6.0;
        let kn_norm = MultibandClipperView::knee_width_to_normalized(kn_test);
        let kn_back = MultibandClipperView::normalized_to_knee_width(kn_norm);
        assert!((kn_back - kn_test).abs() < 1e-3);

        // Drive [0.0 ..= 12.0 dB]
        let dr_test = 3.5;
        let dr_norm = MultibandClipperView::drive_to_normalized(dr_test);
        let dr_back = MultibandClipperView::normalized_to_drive(dr_norm);
        assert!((dr_back - dr_test).abs() < 1e-3);

        // Evaluate transfer curves across all modes
        for &(mode, _) in &modes {
            view.set_curve_mode(mode);
            let y_below = view.evaluate_transfer_curve(-20.0);
            let y_above = view.evaluate_transfer_curve(-1.0);
            assert!(y_below <= 0.0);
            assert!(y_above <= 0.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(50.0, 50.0, 400.0, 300.0);
        view.set_active_band(1);
        let puck_x = canvas.x + view.knee_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.knee_puck_pos.1) * canvas.height;

        assert!(view.hit_test_knee_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_knee_puck((puck_x + 15.0, puck_y), canvas));
        assert!(view.hit_test_knee_puck((puck_x, puck_y + 15.0), canvas));
        assert!(!view.hit_test_knee_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING DYNAMIC MULTIBAND CLIPPER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn104_pure_multiband_expander_initialization_and_kinematics() {
        let mut view = MultibandExpanderView::new();
        assert_eq!(view.active_band_idx, 1);
        assert_eq!(view.bands.len(), NUM_EXPANDER_BANDS);
        assert_eq!(EXPANDER_NODE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (ExpanderMode::DownwardExpansion, "Downward Expansion"),
            (ExpanderMode::UpwardExpansion, "Upward Expansion"),
            (ExpanderMode::NoiseGate, "Noise Gate"),
            (ExpanderMode::DynamicDucking, "Dynamic Ducking"),
        ];

        for (m, expected_name) in modes {
            view.set_mode(m);
            assert_eq!(view.mode, m);
            assert_eq!(m.name(), expected_name);
        }

        // Active band switching
        for i in 0..NUM_EXPANDER_BANDS {
            view.set_active_band(i);
            assert_eq!(view.active_band_idx, i);
        }

        // Coordinate normalizations & round-trips for Threshold [-60.0 ..= 0.0 dB]
        let th_test = -34.0;
        let th_norm = MultibandExpanderView::threshold_to_normalized(th_test);
        let th_back = MultibandExpanderView::normalized_to_threshold(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Ratio [1.0 ..= 10.0]
        let rt_test = 3.5;
        let rt_norm = MultibandExpanderView::ratio_to_normalized(rt_test);
        let rt_back = MultibandExpanderView::normalized_to_ratio(rt_norm);
        assert!((rt_back - rt_test).abs() < 1e-3);

        // Knee [0.0 ..= 18.0 dB]
        let kn_test = 8.5;
        let kn_norm = MultibandExpanderView::knee_to_normalized(kn_test);
        let kn_back = MultibandExpanderView::normalized_to_knee(kn_norm);
        assert!((kn_back - kn_test).abs() < 1e-3);

        // Dry/Wet [0.0 ..= 100.0 %]
        let dw_test = 75.0;
        let dw_norm = MultibandExpanderView::dry_wet_to_normalized(dw_test);
        let dw_back = MultibandExpanderView::normalized_to_dry_wet(dw_norm);
        assert!((dw_back - dw_test).abs() < 1e-3);

        // Transfer curve evaluations across all modes
        for &(mode, _) in &modes {
            view.set_mode(mode);
            let y_zero = MultibandExpanderView::evaluate_transfer_curve(0.0, -30.0, 2.0, 4.0, mode);
            let y_low = MultibandExpanderView::evaluate_transfer_curve(-50.0, -30.0, 2.0, 4.0, mode);
            assert!(y_zero.is_finite());
            assert!(y_low.is_finite());
        }

        // Hit testing: 3 crossover divider handles
        let canvas = Rect::new(20.0, 50.0, 400.0, 250.0);
        let min_log = (20.0_f32).ln();
        let max_log = (20000.0_f32).ln();
        for i in 0..3 {
            let freq = view.bands[i].crossover_freq_hz;
            let norm_x = (freq.ln() - min_log) / (max_log - min_log);
            let px = canvas.x + norm_x * canvas.width;
            let py = canvas.y + canvas.height * 0.5;
            assert_eq!(view.hit_test_crossover_node((px, py), canvas), Some(i));
        }

        // Dynamic transfer puck hit test
        assert!(view.hit_test_transfer_puck(
            {
                let band = &view.bands[view.active_band_idx];
                let nx = MultibandExpanderView::threshold_to_normalized(band.threshold_db);
                let ny = MultibandExpanderView::ratio_to_normalized(band.ratio);
                (canvas.x + nx * canvas.width, canvas.y + (1.0 - ny) * canvas.height)
            },
            canvas,
        ));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("4-BAND DYNAMIC EXPANDER & NOISE GATE HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn104_gui_multiband_clipper_and_expander_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Multiband Clipper Modal Lifecycle
        assert!(!view.is_multiband_clipper_hud_open());
        view.open_multiband_clipper_hud();
        assert!(view.is_multiband_clipper_hud_open());
        view.close_multiband_clipper_hud();
        assert!(!view.is_multiband_clipper_hud_open());

        // 2. Multiband Expander Modal Lifecycle
        assert!(!view.is_multiband_expander_hud_open());
        view.open_multiband_expander_hud();
        assert!(view.is_multiband_expander_hud_open());
        view.close_multiband_expander_hud();
        assert!(!view.is_multiband_expander_hud_open());

        // 3. Multiband Clipper Parameter Sync Bridge
        view.multiband_clipper_view.bands[view.multiband_clipper_view.active_band_idx].threshold_db = -8.5;
        view.multiband_clipper_view.bands[view.multiband_clipper_view.active_band_idx].ceiling_db = -1.2;
        view.multiband_clipper_view.knee_width_db = 5.5;
        view.sync_multiband_clipper_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("threshold_db").copied(),
            Some(-8.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("ceiling_db").copied(),
            Some(-1.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("knee_width_db").copied(),
            Some(5.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("threshold").copied(),
            Some(-8.5)
        );

        // 4. Multiband Expander Parameter Sync Bridge
        view.multiband_expander_view.bands[view.multiband_expander_view.active_band_idx].threshold_db = -42.0;
        view.multiband_expander_view.bands[view.multiband_expander_view.active_band_idx].ratio = 4.5;
        view.multiband_expander_view.bands[view.multiband_expander_view.active_band_idx].knee_db = 8.0;
        view.sync_multiband_expander_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("threshold_db").copied(),
            Some(-42.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("ratio").copied(),
            Some(4.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("knee_db").copied(),
            Some(8.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("ratio").copied(),
            Some(4.5)
        );
    }

    #[test]
    fn test_turn104_gui_trigger_propagation_from_inspector_and_rack() {
        let ctx = egui::Context::default();
        let mut view = AwardWinningGuiView::new();

        // 1. Inspector Trigger: Multiband Clipper
        view.inspector_state.selected_node_kind = Some("MultibandClipperNode".to_string());
        view.inspector_state.requested_open_multiband_clipper_hud = true;
        assert!(!view.is_multiband_clipper_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_clipper_hud_open());
        assert!(!view.inspector_state.requested_open_multiband_clipper_hud);
        view.close_multiband_clipper_hud();

        // 2. Inspector Trigger: Multiband Expander
        view.inspector_state.selected_node_kind = Some("MultibandExpanderNode".to_string());
        view.inspector_state.requested_open_multiband_expander_hud = true;
        assert!(!view.is_multiband_expander_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_expander_hud_open());
        assert!(!view.inspector_state.requested_open_multiband_expander_hud);
        view.close_multiband_expander_hud();

        // 3. Device Rack Trigger: Multiband Clipper
        view.device_rack_state.selected_node_kind = Some("MultibandClipper".to_string());
        view.device_rack_state.requested_open_multiband_clipper_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_clipper_hud_open());
        assert!(!view.device_rack_state.requested_open_multiband_clipper_hud);
        view.close_multiband_clipper_hud();

        // 4. Device Rack Trigger: Multiband Expander
        view.device_rack_state.selected_node_kind = Some("MultibandExpander".to_string());
        view.device_rack_state.requested_open_multiband_expander_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_expander_hud_open());
        assert!(!view.device_rack_state.requested_open_multiband_expander_hud);
        view.close_multiband_expander_hud();
    }

    #[test]
    fn test_turn104_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let target_ids = [
            "open_multiband_clipper_hud",
            "open_multiband_expander_hud",
        ];

        for action_id in target_ids {
            let found = palette.actions.iter().any(|a| a.action_id == action_id);
            assert!(found, "Command palette missing action: {}", action_id);
        }
    }
}
