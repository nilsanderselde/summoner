#![allow(clippy::all)]

//! Test suite for Turn #101 (Sprint Turn #27 - Sprint Review) — Tactile Transient Designer & Harmonic Punch HUD
//! and Mastering Multiband Upward Compressor (OTT) HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::transient_designer_view::{
        TransientMode, TransientDesignerView, TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS,
    };
    use crate::views::upward_compressor_view::{
        UpwardCompressionProfile, UpwardCompressorView, UPWARD_COMPRESSOR_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn101_pure_transient_designer_initialization_and_kinematics() {
        let mut view = TransientDesignerView::new();
        assert_eq!(view.mode, TransientMode::Broadband);
        assert!((view.attack_gain_db - 6.0).abs() < 1e-3);
        assert!((view.attack_length_ms - 25.0).abs() < 1e-3);
        assert!((view.sustain_gain_db - (-3.0)).abs() < 1e-3);
        assert!((view.sustain_length_ms - 350.0).abs() < 1e-3);
        assert!((view.punch_freq_hz - 90.0).abs() < 1e-3);
        assert!(view.soft_clip_enabled);
        assert_eq!(TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (TransientMode::Broadband, "Broadband"),
            (TransientMode::FrequencySplit, "Frequency Split"),
            (TransientMode::HarmonicPunch, "Harmonic Punch"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Gain [-24.0..+24.0 dB]
        let g_test = 8.5;
        let g_norm = TransientDesignerView::gain_to_normalized(g_test);
        let g_back = TransientDesignerView::normalized_to_gain(g_norm);
        assert!((g_back - g_test).abs() < 1e-3);

        // Attack Time [5.0..100.0 ms]
        let at_test = 42.0;
        let at_norm = TransientDesignerView::attack_time_to_normalized(at_test);
        let at_back = TransientDesignerView::normalized_to_attack_time(at_norm);
        assert!((at_back - at_test).abs() < 1e-3);

        // Sustain Time [50.0..1000.0 ms]
        let st_test = 420.0;
        let st_norm = TransientDesignerView::sustain_time_to_normalized(st_test);
        let st_back = TransientDesignerView::normalized_to_sustain_time(st_norm);
        assert!((st_back - st_test).abs() < 1e-3);

        // Punch Frequency [40.0..500.0 Hz]
        let pf_test = 120.0;
        let pf_norm = TransientDesignerView::punch_freq_to_normalized(pf_test);
        let pf_back = TransientDesignerView::normalized_to_punch_freq(pf_norm);
        assert!((pf_back - pf_test).abs() < 1e-3);

        // Output Gain [-12.0..+12.0 dB]
        let og_test = 2.5;
        let og_norm = TransientDesignerView::output_gain_to_normalized(og_test);
        let og_back = TransientDesignerView::normalized_to_output_gain(og_norm);
        assert!((og_back - og_test).abs() < 1e-3);

        // Envelope curve evaluations
        let env_start = view.evaluate_envelope_curve(0.0);
        let env_attack = view.evaluate_envelope_curve(0.05);
        let env_sustain = view.evaluate_envelope_curve(0.5);
        assert!(env_start >= 0.0);
        assert!(env_attack > 0.0);
        assert!(env_sustain >= 0.0);

        // Touch handle hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let atk_hx = canvas.x + (view.attack_handle_pos.0 * 0.35) * canvas.width;
        let atk_hy = canvas.y + (1.0 - view.attack_handle_pos.1) * canvas.height;
        assert!(view.hit_test_attack_handle((atk_hx, atk_hy), canvas));
        assert!(!view.hit_test_attack_handle((atk_hx + 50.0, atk_hy + 50.0), canvas));

        let sus_hx = canvas.x + (0.35 + view.sustain_handle_pos.0 * 0.65) * canvas.width;
        let sus_hy = canvas.y + (1.0 - view.sustain_handle_pos.1) * canvas.height;
        assert!(view.hit_test_sustain_handle((sus_hx, sus_hy), canvas));
        assert!(!view.hit_test_sustain_handle((sus_hx + 50.0, sus_hy + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("TRANSIENT DESIGNER"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn101_pure_upward_compressor_initialization_and_kinematics() {
        let mut view = UpwardCompressorView::new();
        assert_eq!(view.profile, UpwardCompressionProfile::LowLevelDetail);
        assert_eq!(view.selected_band_idx, 1);
        assert_eq!(view.bands.len(), 4);
        assert_eq!(UPWARD_COMPRESSOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profile switching & names
        let profiles = [
            (UpwardCompressionProfile::LowLevelDetail, "Low Level Detail"),
            (UpwardCompressionProfile::OttAggressive, "OTT Aggressive"),
            (UpwardCompressionProfile::BroadcastDensity, "Broadcast Density"),
            (UpwardCompressionProfile::VocalAirExtract, "Vocal Air Extract"),
            (UpwardCompressionProfile::LinearPhaseMaster, "Linear Phase Master"),
        ];

        for (profile, expected_name) in profiles {
            view.set_profile(profile);
            assert_eq!(view.profile, profile);
            assert_eq!(profile.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Threshold [-60.0..-10.0 dBFS]
        let th_test = -35.0;
        let th_norm = UpwardCompressorView::threshold_to_normalized(th_test);
        let th_back = UpwardCompressorView::normalized_to_threshold(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Boost [0.0..18.0 dB]
        let bst_test = 9.5;
        let bst_norm = UpwardCompressorView::boost_to_normalized(bst_test);
        let bst_back = UpwardCompressorView::normalized_to_boost(bst_norm);
        assert!((bst_back - bst_test).abs() < 1e-3);

        // Ratio [1.0..4.0]
        let r_test = 2.6;
        let r_norm = UpwardCompressorView::ratio_to_normalized(r_test);
        let r_back = UpwardCompressorView::normalized_to_ratio(r_norm);
        assert!((r_back - r_test).abs() < 1e-3);

        // Attack [1.0..100.0 ms]
        let atk_test = 18.0;
        let atk_norm = UpwardCompressorView::attack_to_normalized(atk_test);
        let atk_back = UpwardCompressorView::normalized_to_attack(atk_norm);
        assert!((atk_back - atk_test).abs() < 1e-3);

        // Release [10.0..500.0 ms]
        let rel_test = 140.0;
        let rel_norm = UpwardCompressorView::release_to_normalized(rel_test);
        let rel_back = UpwardCompressorView::normalized_to_release(rel_norm);
        assert!((rel_back - rel_test).abs() < 1e-3);

        // Transfer curve evaluations
        let above_thresh = view.evaluate_transfer_curve(-20.0, 1);
        assert!((above_thresh - (-20.0)).abs() < 1e-3); // Unity gain above threshold

        let below_thresh = view.evaluate_transfer_curve(-50.0, 1);
        assert!(below_thresh > -50.0); // Upward compression boosts signals below threshold

        // Touch puck hit testing
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.upward_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.upward_puck_pos.1) * canvas.height;
        assert!(view.hit_test_upward_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_upward_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("|"));
        assert!(snap.contains("-"));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::show_modern_inspector_with_context;

    #[test]
    fn test_turn101_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Tactile Transient Designer HUD
        assert!(!view.is_transient_designer_hud_open());
        view.open_transient_designer_hud();
        assert!(view.is_transient_designer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_transient_designer_hud();
        assert!(!view.is_transient_designer_hud_open());

        // 2. Mastering Multiband Upward Compressor (OTT) HUD
        assert!(!view.is_upward_compressor_hud_open());
        view.open_upward_compressor_hud();
        assert!(view.is_upward_compressor_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_upward_compressor_hud();
        assert!(!view.is_upward_compressor_hud_open());
    }

    #[test]
    fn test_turn101_gui_lock_free_param_bus_dispatch_transient_and_upward() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Transient Designer (offsets 0..=3) and Upward Compressor (offsets 0..=3)
        for offset in 0..=4 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Transient Designer sync
        view.transient_designer_view.attack_gain_db = 9.0;
        view.transient_designer_view.sustain_gain_db = -4.5;
        view.transient_designer_view.punch_freq_hz = 110.0;
        view.transient_designer_view.output_gain_db = 1.5;

        view.sync_transient_designer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("attack_gain_db"), Some(&9.0));
        assert_eq!(view.device_rack_state.node_param_values.get("sustain_gain_db"), Some(&-4.5));
        assert_eq!(view.device_rack_state.node_param_values.get("punch_freq_hz"), Some(&110.0));
        assert_eq!(view.device_rack_state.node_param_values.get("output_gain_db"), Some(&1.5));

        let pid_atk = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_sus = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_punch = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_out = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_atk).unwrap() - 9.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_sus).unwrap() - (-4.5)).abs() < 1e-3);
        assert!((arc_bus.get(pid_punch).unwrap() - 110.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_out).unwrap() - 1.5).abs() < 1e-3);

        // Test Upward Compressor sync
        view.upward_compressor_view.selected_band_idx = 1;
        view.upward_compressor_view.bands[1].threshold_dbfs = -38.0;
        view.upward_compressor_view.bands[1].max_boost_db = 12.5;
        view.upward_compressor_view.bands[1].ratio = 3.2;
        view.upward_compressor_view.bands[1].active_gain_boost_db = 7.0;

        view.sync_upward_compressor_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("threshold_dbfs"), Some(&-38.0));
        assert_eq!(view.device_rack_state.node_param_values.get("max_boost_db"), Some(&12.5));
        assert_eq!(view.device_rack_state.node_param_values.get("ratio"), Some(&3.2));
        assert_eq!(view.device_rack_state.node_param_values.get("active_gain_boost_db"), Some(&7.0));

        let pid_th = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_bst = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_rt = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_lft = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_th).unwrap() - (-38.0)).abs() < 1e-3);
        assert!((arc_bus.get(pid_bst).unwrap() - 12.5).abs() < 1e-3);
        assert!((arc_bus.get(pid_rt).unwrap() - 3.2).abs() < 1e-3);
        assert!((arc_bus.get(pid_lft).unwrap() - 7.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn101_gui_inspector_and_rack_trigger_buttons() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Modern Inspector Trigger Propagation for Transient Designer
        view.inspector_state.selected_node_kind = Some("TransientDesignerNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(
                    ui,
                    &mut view.inspector_state,
                    view.live_param_bus.as_deref(),
                    1,
                );
            });
        });

        view.inspector_state.requested_open_transient_designer_hud = true;
        assert!(!view.is_transient_designer_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_transient_designer_hud_open());
        view.close_transient_designer_hud();

        // 2. Modern Inspector Trigger Propagation for Upward Compressor
        view.inspector_state.selected_node_kind = Some("UpwardCompressorNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(
                    ui,
                    &mut view.inspector_state,
                    view.live_param_bus.as_deref(),
                    1,
                );
            });
        });

        view.inspector_state.requested_open_upward_compressor_hud = true;
        assert!(!view.is_upward_compressor_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_upward_compressor_hud_open());
        view.close_upward_compressor_hud();

        // 3. Modern Device Rack Trigger Propagation for Transient Designer
        view.device_rack_state.selected_node_kind = Some("TransientDesigner".to_string());
        view.device_rack_state.requested_open_transient_designer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_transient_designer_hud_open());
        view.close_transient_designer_hud();

        // 4. Modern Device Rack Trigger Propagation for Upward Compressor
        view.device_rack_state.selected_node_kind = Some("UpwardCompressor".to_string());
        view.device_rack_state.requested_open_upward_compressor_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_upward_compressor_hud_open());
        view.close_upward_compressor_hud();
    }
}
