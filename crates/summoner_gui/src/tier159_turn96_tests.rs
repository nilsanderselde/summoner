#![allow(clippy::all)]

//! Test suite for Turn #96 (Sprint Turn #22) — Mastering True-Peak 8x Oversampled Limiter HUD
//! and Dual-Mono Variable-Mu Vacuum Tube Optical Compressor HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::oversampled_limiter_view::{
        DitherShapingCurve, LimiterProfile, OversampledLimiterView, LIMITER_PUCK_HIT_RADIUS,
    };
    use crate::views::vari_mu_master_view::{
        TubeProfile, VariMuMasterView, VARI_MU_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn96_pure_oversampled_limiter_initialization_and_kinematics() {
        let mut view = OversampledLimiterView::new();
        assert_eq!(view.profile, LimiterProfile::BroadcastEbuR128);
        assert!((view.ceiling_dbtp - (-1.0)).abs() < 1e-3);
        assert!((view.threshold_db - (-5.0)).abs() < 1e-3);
        assert!((view.release_ms - 85.0).abs() < 1e-3);
        assert!(view.auto_release);
        assert_eq!(view.dither_curve, DitherShapingCurve::ModifiedShibata);
        assert!(!view.is_16_bit_dither);
        assert_eq!(LIMITER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profile switching and default character metrics
        let profiles = [
            (LimiterProfile::TransparentClean, -0.3, -6.0, 50.0, "TRANSPARENT"),
            (LimiterProfile::WarmAnalogTape, -0.5, -8.5, 120.0, "WARM TAPE"),
            (LimiterProfile::PunchyTransient, -0.2, -7.2, 25.0, "PUNCHY SNAP"),
            (LimiterProfile::BroadcastEbuR128, -1.0, -5.0, 85.0, "BROADCAST EBU"),
            (LimiterProfile::AggressiveClubLoudness, -0.1, -12.0, 15.0, "CLUB LOUD"),
        ];

        for (prof, expected_ceil, expected_thresh, expected_rel, expected_name) in profiles {
            view.set_profile(prof);
            assert_eq!(view.profile, prof);
            assert_eq!(prof.name(), expected_name);
            assert!((view.ceiling_dbtp - expected_ceil).abs() < 1e-3);
            assert!((view.threshold_db - expected_thresh).abs() < 1e-3);
            assert!((view.release_ms - expected_rel).abs() < 1e-3);
        }

        // Coordinate normalizations & round-trips for Threshold [-18.0..0.0 dB]
        let th_test = -12.5;
        let th_norm = OversampledLimiterView::thresh_to_normalized(th_test);
        let th_back = OversampledLimiterView::normalized_to_thresh(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Ceiling [-6.0..0.0 dBTP]
        let c_test = -2.4;
        let c_norm = OversampledLimiterView::ceiling_to_normalized(c_test);
        let c_back = OversampledLimiterView::normalized_to_ceiling(c_norm);
        assert!((c_back - c_test).abs() < 1e-3);

        // Release Time [1.0..1000.0 ms]
        let rel_test = 65.0;
        let rel_norm = OversampledLimiterView::release_to_normalized(rel_test);
        let rel_back = OversampledLimiterView::normalized_to_release(rel_norm);
        assert!((rel_back - rel_test).abs() < 1e-2);

        // Sinc ISP evaluation
        let sinc_center = view.evaluate_sinc_isp(0.0);
        let sinc_side = view.evaluate_sinc_isp(1.0);
        assert!((sinc_center - 1.0).abs() < 1e-3);
        assert!(sinc_side.abs() < 0.2);

        // Dither shaping curves & SNR improvements
        let dither_curves = [
            (DitherShapingCurve::FlatTpdf, "FLAT TPDF", 0.0),
            (DitherShapingCurve::Lipshitz, "LIPSHITZ", 6.2),
            (DitherShapingCurve::EWeighted, "E-WEIGHTED", 11.5),
            (DitherShapingCurve::FWeighted, "F-WEIGHTED", 13.8),
            (DitherShapingCurve::ModifiedShibata, "MODIFIED SHIBATA", 16.2),
        ];

        for (curve, label, snr) in dither_curves {
            view.set_dither_curve(curve);
            assert_eq!(view.dither_curve, curve);
            assert_eq!(curve.label(), label);
            assert!((curve.snr_improvement_db() - snr).abs() < 1e-3);
            let curve_val = view.evaluate_noise_shaping_curve(4000.0);
            assert!(curve_val.is_finite());
        }

        // Limiter puck hit testing
        let canvas = Rect::new(30.0, 114.0, 360.0, 216.0);
        let puck_x = canvas.x + view.limiter_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.limiter_puck_pos.1) * canvas.height;
        assert!(view.hit_test_limiter_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_limiter_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("TRUE-PEAK 8x LIMITER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn96_pure_vari_mu_master_initialization_and_kinematics() {
        let mut view = VariMuMasterView::new();
        assert_eq!(view.profile, TubeProfile::Fairchild670Vintage);
        assert!((view.threshold_db - (-14.0)).abs() < 1e-3);
        assert!((view.input_drive_db - 4.5).abs() < 1e-3);
        assert!((view.makeup_gain_db - 3.0).abs() < 1e-3);
        assert!((view.stereo_link_pct - 100.0).abs() < 1e-3);
        assert!((view.tube_bias_v - (-6.5)).abs() < 1e-3);
        assert_eq!(VARI_MU_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profile switching and character definitions
        let profiles = [
            (TubeProfile::Fairchild670Vintage, 0.2, 300.0, 2.0, "FAIRCHILD 670"),
            (TubeProfile::ManleyVariableMu, 25.0, 400.0, 1.5, "MANLEY VARI-MU"),
            (TubeProfile::TeletronixLa2aOpto, 10.0, 1500.0, 3.0, "LA-2A OPTO"),
            (TubeProfile::Neve33609DiodeBridge, 3.0, 100.0, 2.5, "NEVE 33609"),
            (TubeProfile::PultecTubeFeedback, 15.0, 500.0, 1.8, "PULTEC TUBE SAT"),
        ];

        for (prof, att, rel, base_ratio, name) in profiles {
            view.set_profile(prof);
            assert_eq!(view.profile, prof);
            assert_eq!(prof.name(), name);
            assert!((prof.default_attack_ms() - att).abs() < 1e-3);
            assert!((prof.default_release_ms() - rel).abs() < 1e-3);
            assert!((prof.nominal_base_ratio() - base_ratio).abs() < 1e-3);
            assert!(!prof.harmonic_profile_name().is_empty());
        }

        // Coordinate normalizations & round-trips for Threshold [-40.0..+10.0 dB]
        let th_test = -22.5;
        let th_norm = VariMuMasterView::threshold_to_normalized(th_test);
        let th_back = VariMuMasterView::normalized_to_threshold(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Input Drive [-12.0..+24.0 dB]
        let drv_test = 8.5;
        let drv_norm = VariMuMasterView::drive_to_normalized(drv_test);
        let drv_back = VariMuMasterView::normalized_to_drive(drv_norm);
        assert!((drv_back - drv_test).abs() < 1e-3);

        // Stereo Link [0.0..100.0 %]
        let link_test = 75.0;
        let link_norm = VariMuMasterView::link_to_normalized(link_test);
        let link_back = VariMuMasterView::normalized_to_link(link_norm);
        assert!((link_back - link_test).abs() < 1e-3);

        // Non-linear transfer curve evaluation
        let out_uncompressed = view.evaluate_transfer_curve(-30.0);
        let out_compressed = view.evaluate_transfer_curve(0.0);
        assert!(out_uncompressed.is_finite());
        assert!(out_compressed.is_finite());
        assert!(out_compressed < 0.0 + view.input_drive_db + view.makeup_gain_db);

        // Tube simulation update
        view.update_tube_simulation();
        assert!(view.dynamic_ratio >= 1.2 && view.dynamic_ratio <= 10.0);
        assert!(view.gain_reduction_l_db >= 0.0);
        assert!(view.gain_reduction_r_db >= 0.0);
        assert!(view.thd_distortion_pct >= 0.01 && view.thd_distortion_pct <= 5.0);

        // Puck hit testing
        let canvas = Rect::new(30.0, 114.0, 360.0, 216.0);
        let puck_x = canvas.x + view.vari_mu_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.vari_mu_puck_pos.1) * canvas.height;
        assert!(view.hit_test_vari_mu_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_vari_mu_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("VARIABLE-MU TUBE MASTERING COMPRESSOR HUD"));
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
    fn test_turn96_gui_modal_window_lifecycles_limiter_and_vari_mu() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Mastering True-Peak 8x Oversampled Limiter HUD
        assert!(!view.is_oversampled_limiter_hud_open());
        view.open_oversampled_limiter_hud();
        assert!(view.is_oversampled_limiter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_oversampled_limiter_hud();
        assert!(!view.is_oversampled_limiter_hud_open());

        // 2. Mastering Dual-Mono Variable-Mu Vacuum Tube Optical Compressor HUD
        assert!(!view.is_vari_mu_master_hud_open());
        view.open_vari_mu_master_hud();
        assert!(view.is_vari_mu_master_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_vari_mu_master_hud();
        assert!(!view.is_vari_mu_master_hud_open());
    }

    #[test]
    fn test_turn96_gui_lock_free_param_bus_dispatch_limiter_and_vari_mu() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Limiter (offsets 0..=4) and Vari-Mu (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Limiter sync
        view.oversampled_limiter_view.ceiling_dbtp = -0.5;
        view.oversampled_limiter_view.threshold_db = -8.0;
        view.oversampled_limiter_view.release_ms = 45.0;
        view.oversampled_limiter_view.auto_release = true;
        view.oversampled_limiter_view.is_16_bit_dither = true;

        view.sync_oversampled_limiter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("ceiling_dbtp"), Some(&-0.5));
        assert_eq!(view.device_rack_state.node_param_values.get("threshold_db"), Some(&-8.0));
        assert_eq!(view.device_rack_state.node_param_values.get("release_ms"), Some(&45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("auto_release"), Some(&1.0));
        assert_eq!(view.device_rack_state.node_param_values.get("is_16_bit_dither"), Some(&1.0));

        let pid_ceil = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_thresh = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_rel = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        assert_eq!(arc_bus.get(pid_ceil), Some(-0.5));
        assert_eq!(arc_bus.get(pid_thresh), Some(-8.0));
        assert_eq!(arc_bus.get(pid_rel), Some(45.0));

        // Test Vari-Mu sync
        view.vari_mu_master_view.threshold_db = -18.0;
        view.vari_mu_master_view.input_drive_db = 6.0;
        view.vari_mu_master_view.makeup_gain_db = 4.0;
        view.vari_mu_master_view.stereo_link_pct = 80.0;
        view.vari_mu_master_view.tube_bias_v = -8.0;
        view.vari_mu_master_view.dynamic_ratio = 3.2;

        view.sync_vari_mu_master_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("threshold_db"), Some(&-18.0));
        assert_eq!(view.device_rack_state.node_param_values.get("input_drive_db"), Some(&6.0));
        assert_eq!(view.device_rack_state.node_param_values.get("makeup_gain_db"), Some(&4.0));
        assert_eq!(view.device_rack_state.node_param_values.get("stereo_link_pct"), Some(&80.0));
        assert_eq!(view.device_rack_state.node_param_values.get("tube_bias_v"), Some(&-8.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dynamic_ratio"), Some(&3.2));

        let pid_th = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_drv = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_mk = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_lk = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        assert_eq!(arc_bus.get(pid_th), Some(-18.0));
        assert_eq!(arc_bus.get(pid_drv), Some(6.0));
        assert_eq!(arc_bus.get(pid_mk), Some(4.0));
        assert_eq!(arc_bus.get(pid_lk), Some(80.0));
    }

    #[test]
    fn test_turn96_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Limiter trigger propagation from rack
        view.device_rack_state.requested_open_oversampled_limiter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_oversampled_limiter_hud);
        assert!(view.is_oversampled_limiter_hud_open());
        view.close_oversampled_limiter_hud();

        // 2. Vari-Mu trigger propagation from inspector
        view.inspector_state.requested_open_vari_mu_master_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_vari_mu_master_hud);
        assert!(view.is_vari_mu_master_hud_open());
        view.close_vari_mu_master_hud();
    }

    #[test]
    fn test_turn96_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let limiter_action = palette.actions.iter().find(|a| a.action_id == "open_oversampled_limiter_hud");
        assert!(limiter_action.is_some(), "open_oversampled_limiter_hud must be registered in command palette");
        let lim = limiter_action.unwrap();
        assert_eq!(lim.category, "Dynamics & Mastering");

        let vari_mu_action = palette.actions.iter().find(|a| a.action_id == "open_vari_mu_master_hud");
        assert!(vari_mu_action.is_some(), "open_vari_mu_master_hud must be registered in command palette");
        let vm = vari_mu_action.unwrap();
        assert_eq!(vm.category, "Dynamics & Mastering");
    }
}
