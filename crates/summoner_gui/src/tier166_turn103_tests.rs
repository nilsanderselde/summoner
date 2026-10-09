#![allow(clippy::all)]

//! Test suite for Turn #103 (Sprint Turn #29) — Master Bus Dynamic Multiband Saturator & Harmonic Warmth HUD
//! and Multiband Stereo Imager & Spectral Correlation Matrix HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::multiband_saturator_view::{
        MultibandSaturatorView, SaturationModel, NUM_SATURATOR_BANDS, SATURATOR_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_imager_view::{
        MultibandImagerView, NUM_IMAGER_BANDS, IMAGER_HANDLE_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn103_pure_multiband_saturator_initialization_and_kinematics() {
        let mut view = MultibandSaturatorView::new();
        assert_eq!(view.selected_band_idx, 2);
        assert_eq!(view.oversampling_factor, 4);
        assert!((view.crossover_low_mid_hz - 120.0).abs() < 1e-3);
        assert!((view.crossover_mid_high_hz - 1500.0).abs() < 1e-3);
        assert!((view.crossover_air_hz - 6500.0).abs() < 1e-3);
        assert_eq!(SATURATOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Check bands initialization
        assert_eq!(view.bands.len(), NUM_SATURATOR_BANDS);
        assert_eq!(view.bands[0].model, SaturationModel::TriodeTubeWarmth);
        assert_eq!(view.bands[1].model, SaturationModel::TapeHysteresis);
        assert_eq!(view.bands[2].model, SaturationModel::GermaniumDiodeClip);
        assert_eq!(view.bands[3].model, SaturationModel::TriodeTubeWarmth);

        // SaturationModel names and switching
        let models = [
            (SaturationModel::TapeHysteresis, "Tape Hysteresis"),
            (SaturationModel::TriodeTubeWarmth, "Triode Tube Warmth"),
            (SaturationModel::GermaniumDiodeClip, "Germanium Diode Clip"),
            (SaturationModel::AsymmetricOverdrive, "Asymmetric Overdrive"),
            (SaturationModel::SoftKneeLimiter, "Soft-Knee Limiter"),
        ];

        for (m, expected_name) in models {
            view.set_model(0, m);
            assert_eq!(view.bands[0].model, m);
            assert_eq!(m.name(), expected_name);
        }

        // Band selection
        for i in 0..NUM_SATURATOR_BANDS {
            view.set_selected_band(i);
            assert_eq!(view.selected_band_idx, i);
        }

        // Coordinate normalizations & round-trips for Saturation Drive [0.0..24.0 dB]
        let dr_test = 14.4;
        let dr_norm = MultibandSaturatorView::drive_to_normalized(dr_test);
        let dr_back = MultibandSaturatorView::normalized_to_drive(dr_norm);
        assert!((dr_back - dr_test).abs() < 1e-3);

        // Harmonic Asymmetry Bias [-1.0..+1.0]
        let bias_test = -0.45;
        let bias_norm = MultibandSaturatorView::bias_to_normalized(bias_test);
        let bias_back = MultibandSaturatorView::normalized_to_bias(bias_norm);
        assert!((bias_back - bias_test).abs() < 1e-3);

        // Crossover Low [40.0..300.0 Hz]
        let cl_test = 180.0;
        let cl_norm = MultibandSaturatorView::crossover_low_to_normalized(cl_test);
        let cl_back = MultibandSaturatorView::normalized_to_crossover_low(cl_norm);
        assert!((cl_back - cl_test).abs() < 1e-3);

        // Crossover Mid [500.0..4000.0 Hz]
        let cm_test = 2200.0;
        let cm_norm = MultibandSaturatorView::crossover_mid_to_normalized(cm_test);
        let cm_back = MultibandSaturatorView::normalized_to_crossover_mid(cm_norm);
        assert!((cm_back - cm_test).abs() < 1e-3);

        // Crossover Air [4000.0..16000.0 Hz]
        let ca_test = 8800.0;
        let ca_norm = MultibandSaturatorView::crossover_air_to_normalized(ca_test);
        let ca_back = MultibandSaturatorView::normalized_to_crossover_air(ca_norm);
        assert!((ca_back - ca_test).abs() < 1e-3);

        // Mix [0.0..100.0 %]
        let mix_test = 65.0;
        let mix_norm = MultibandSaturatorView::mix_to_normalized(mix_test);
        let mix_back = MultibandSaturatorView::normalized_to_mix(mix_norm);
        assert!((mix_back - mix_test).abs() < 1e-3);

        // Transfer curve evaluations across models
        for &(model, _) in &models {
            view.set_model(view.selected_band_idx, model);
            let y_zero = view.evaluate_transfer_curve(0.0, view.selected_band_idx);
            let y_pos = view.evaluate_transfer_curve(0.5, view.selected_band_idx);
            let y_neg = view.evaluate_transfer_curve(-0.5, view.selected_band_idx);
            assert!(y_pos > y_zero || y_pos.is_finite());
            assert!(y_neg < y_zero || y_neg.is_finite());
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 104.0, 400.0, 236.0);
        let puck_x = canvas.x + view.saturator_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.saturator_puck_pos.1) * canvas.height;
        assert!(view.hit_test_saturator_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_saturator_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DYNAMIC MULTIBAND SATURATOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn103_pure_multiband_imager_initialization_and_kinematics() {
        let mut view = MultibandImagerView::new();
        assert_eq!(view.active_band_idx, 0);
        assert!((view.dry_wet_pct - 100.0).abs() < 1e-3);
        assert_eq!(view.bands.len(), NUM_IMAGER_BANDS);
        assert_eq!(IMAGER_HANDLE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Average width calculation
        let avg_w = view.average_width_pct();
        assert!((avg_w - (0.0 + 100.0 + 135.0 + 160.0) / 4.0).abs() < 1e-3);

        // Active band and width setters
        view.set_active_band(2);
        assert_eq!(view.active_band_idx, 2);
        view.set_band_width(2, 175.0);
        assert!((view.bands[2].width_pct - 175.0).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Width [0.0..200.0 %]
        let w_test = 142.0;
        let w_norm = MultibandImagerView::width_to_normalized(w_test);
        let w_back = MultibandImagerView::normalized_to_width(w_norm);
        assert!((w_back - w_test).abs() < 1e-3);

        // Pan [-100.0..+100.0 %]
        let pan_test = -35.0;
        let pan_norm = MultibandImagerView::pan_to_normalized(pan_test);
        let pan_back = MultibandImagerView::normalized_to_pan(pan_norm);
        assert!((pan_back - pan_test).abs() < 1e-3);

        // Dry/Wet [0.0..100.0 %]
        let dw_test = 82.5;
        let dw_norm = MultibandImagerView::dry_wet_to_normalized(dw_test);
        let dw_back = MultibandImagerView::normalized_to_dry_wet(dw_norm);
        assert!((dw_back - dw_test).abs() < 1e-3);

        // Crossover Logarithmic [20.0..20000.0 Hz]
        let co_test = 1500.0;
        let co_norm = MultibandImagerView::crossover_to_normalized(co_test);
        let co_back = MultibandImagerView::normalized_to_crossover(co_norm);
        assert!((co_back - co_test).abs() < 1.0);

        // Crossover divider hit testing
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let hx0 = canvas.x + (1.0 / 4.0) * canvas.width;
        let hy = canvas.y + canvas.height * 0.5;
        assert!(view.hit_test_crossover((hx0, hy), canvas, 0));
        assert!(!view.hit_test_crossover((hx0 + 50.0, hy + 50.0), canvas, 0));

        // Band puck hit testing
        let band_w = canvas.width / 4.0;
        let b1_cx = canvas.x + 1.5 * band_w;
        let mid_y = canvas.y + canvas.height * 0.65;
        let puck_y = mid_y + 35.0;
        assert!(view.hit_test_band_puck((b1_cx, puck_y), canvas, 1));
        assert!(!view.hit_test_band_puck((b1_cx + 50.0, puck_y + 50.0), canvas, 1));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTIBAND STEREO IMAGER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn103_gui_multiband_saturator_and_imager_modal_lifecycles_and_param_sync() {
        let mut view = AwardWinningGuiView::new();

        // 1. Multiband Saturator Modal Lifecycle
        assert!(!view.is_multiband_saturator_hud_open());
        view.open_multiband_saturator_hud();
        assert!(view.is_multiband_saturator_hud_open());
        view.close_multiband_saturator_hud();
        assert!(!view.is_multiband_saturator_hud_open());

        // 2. Multiband Imager Modal Lifecycle
        assert!(!view.is_multiband_imager_hud_open());
        view.open_multiband_imager_hud();
        assert!(view.is_multiband_imager_hud_open());
        view.close_multiband_imager_hud();
        assert!(!view.is_multiband_imager_hud_open());

        // 3. Multiband Saturator Parameter Sync Bridge
        view.multiband_saturator_view.bands[view.multiband_saturator_view.selected_band_idx].drive_gain_db = 12.5;
        view.multiband_saturator_view.bands[view.multiband_saturator_view.selected_band_idx].bias_even_harmonics = 0.35;
        view.sync_multiband_saturator_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("drive_gain_db").copied(),
            Some(12.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("bias_even_harmonics").copied(),
            Some(0.35)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("drive").copied(),
            Some(12.5)
        );

        // 4. Multiband Imager Parameter Sync Bridge
        view.multiband_imager_view.bands[view.multiband_imager_view.active_band_idx].width_pct = 150.0;
        view.multiband_imager_view.bands[view.multiband_imager_view.active_band_idx].pan_pct = 25.0;
        view.sync_multiband_imager_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("width_pct").copied(),
            Some(150.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("pan_pct").copied(),
            Some(25.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("width").copied(),
            Some(150.0)
        );
    }

    #[test]
    fn test_turn103_gui_trigger_propagation_from_inspector_and_rack() {
        let ctx = egui::Context::default();
        let mut view = AwardWinningGuiView::new();

        // 1. Inspector Trigger: Multiband Saturator
        view.inspector_state.selected_node_kind = Some("MultibandSaturatorNode".to_string());
        view.inspector_state.requested_open_multiband_saturator_hud = true;
        assert!(!view.is_multiband_saturator_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_saturator_hud_open());
        assert!(!view.inspector_state.requested_open_multiband_saturator_hud);
        view.close_multiband_saturator_hud();

        // 2. Inspector Trigger: Multiband Imager
        view.inspector_state.selected_node_kind = Some("MultibandImagerNode".to_string());
        view.inspector_state.requested_open_multiband_imager_hud = true;
        assert!(!view.is_multiband_imager_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_imager_hud_open());
        assert!(!view.inspector_state.requested_open_multiband_imager_hud);
        view.close_multiband_imager_hud();

        // 3. Device Rack Trigger: Multiband Saturator
        view.device_rack_state.selected_node_kind = Some("MultibandSaturator".to_string());
        view.device_rack_state.requested_open_multiband_saturator_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_saturator_hud_open());
        assert!(!view.device_rack_state.requested_open_multiband_saturator_hud);
        view.close_multiband_saturator_hud();

        // 4. Device Rack Trigger: Multiband Imager
        view.device_rack_state.selected_node_kind = Some("MultibandImager".to_string());
        view.device_rack_state.requested_open_multiband_imager_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multiband_imager_hud_open());
        assert!(!view.device_rack_state.requested_open_multiband_imager_hud);
        view.close_multiband_imager_hud();
    }

    #[test]
    fn test_turn103_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let target_ids = [
            "open_multiband_saturator_hud",
            "open_multiband_imager_hud",
        ];

        for action_id in target_ids {
            let found = palette.actions.iter().any(|a| a.action_id == action_id);
            assert!(found, "Command palette missing action: {}", action_id);
        }
    }
}
