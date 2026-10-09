#![allow(clippy::all)]

//! Test suite for Turn #94 (Sprint Turn #20) — Dynamic Resonant Auto-Wah Filter HUD
//! and 5-Vowel Phonetic Formant Filter Resonator HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::auto_wah_view::{
        AutoWahView, WAH_PUCK_HIT_RADIUS,
    };
    use crate::views::formant_filter_view::{
        FormantFilterView, FormantTrajectoryMode, STANDARD_VOWELS, FORMANT_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn94_pure_auto_wah_initialization_and_kinematics() {
        let view = AutoWahView::new();
        assert!((view.sensitivity - 0.75).abs() < 1e-3);
        assert!((view.base_freq_hz - 350.0).abs() < 1e-3);
        assert!((view.sweep_range_octaves - 3.5).abs() < 1e-3);
        assert!((view.resonance_q - 6.5).abs() < 1e-3);
        assert!((view.attack_ms - 8.0).abs() < 1e-3);
        assert!((view.release_ms - 120.0).abs() < 1e-3);
        assert!((view.mix - 0.85).abs() < 1e-3);
        assert_eq!(WAH_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Coordinate normalizations & round-trips for Sensitivity [0.0..1.0]
        let sens_test = 0.85;
        let sens_norm = AutoWahView::sensitivity_to_normalized(sens_test);
        let sens_back = AutoWahView::normalized_to_sensitivity(sens_norm);
        assert!((sens_back - sens_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Resonance Q [0.5..25.0]
        let q_test = 14.5;
        let q_norm = AutoWahView::resonance_to_normalized(q_test);
        let q_back = AutoWahView::normalized_to_resonance(q_norm);
        assert!((q_back - q_test).abs() < 1e-3);

        // Filter response magnitude curve (32 points)
        let curve = view.evaluate_filter_magnitude_response();
        assert_eq!(curve.len(), 32);
        for &mag in &curve {
            assert!(mag >= 0.0 && mag <= 1.0);
        }

        // Wah puck hit testing
        let canvas = Rect::new(20.0, 100.0, 360.0, 210.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("AUTO-WAH HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn94_pure_formant_filter_initialization_and_kinematics() {
        let mut view = FormantFilterView::new();
        assert_eq!(view.trajectory_mode, FormantTrajectoryMode::Static);
        assert_eq!(view.active_vowel_idx, 0);
        assert!((view.resonance_q - 8.0).abs() < 1e-3);
        assert!((view.peak_boost_db - 9.0).abs() < 1e-3);
        assert!((view.vocal_tract_scale - 1.0).abs() < 1e-3);
        assert!((view.drive_warmth_db - 3.0).abs() < 1e-3);
        assert!((view.dry_wet_pct - 100.0).abs() < 1e-3);
        assert_eq!(FORMANT_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Trajectory modes & metadata
        let modes = [
            (FormantTrajectoryMode::Static, "STATIC"),
            (FormantTrajectoryMode::CircularLfo, "CIRCLE LFO"),
            (FormantTrajectoryMode::Figure8Lfo, "FIG-8"),
            (FormantTrajectoryMode::EnvelopeFollower, "ENV FOLLOWER"),
        ];
        for (mode, label) in modes {
            view.set_trajectory_mode(mode);
            assert_eq!(view.trajectory_mode, mode);
            assert_eq!(mode.label(), label);
        }

        // Vowel definitions & selection
        assert_eq!(STANDARD_VOWELS.len(), 5);
        for i in 0..5 {
            view.set_vowel(i);
            assert_eq!(view.active_vowel_idx, i);
            assert_eq!(view.morph_pos, STANDARD_VOWELS[i].pad_norm_pos);
        }

        // Coordinate normalizations & round-trips for Resonance Q [1.0..25.0]
        let q_test = 16.0;
        let q_norm = FormantFilterView::resonance_to_normalized(q_test);
        let q_back = FormantFilterView::normalized_to_resonance(q_norm);
        assert!((q_back - q_test).abs() < 1e-3);

        // Boost [0.0..18.0 dB]
        let b_test = 12.0;
        let b_norm = FormantFilterView::boost_to_normalized(b_test);
        let b_back = FormantFilterView::normalized_to_boost(b_norm);
        assert!((b_back - b_test).abs() < 1e-3);

        // Vocal tract scale [0.6..1.4]
        let sc_test = 1.2;
        let sc_norm = FormantFilterView::scale_to_normalized(sc_test);
        let sc_back = FormantFilterView::normalized_to_scale(sc_norm);
        assert!((sc_back - sc_test).abs() < 1e-3);

        // Throat warmth [0.0..12.0 dB]
        let w_test = 6.0;
        let w_norm = FormantFilterView::warmth_to_normalized(w_test);
        let w_back = FormantFilterView::normalized_to_warmth(w_norm);
        assert!((w_back - w_test).abs() < 1e-3);

        // Dry/Wet [0.0..100.0 %]
        let dw_test = 75.0;
        let dw_norm = FormantFilterView::dry_wet_to_normalized(dw_test);
        let dw_back = FormantFilterView::normalized_to_dry_wet(dw_norm);
        assert!((dw_back - dw_test).abs() < 1e-3);

        // Formant frequency response evaluation across spectrum (dB scale)
        let r_floor = view.evaluate_frequency_response(200.0);
        let r_peak = view.evaluate_frequency_response(800.0);
        let r_high = view.evaluate_frequency_response(5000.0);
        assert!(r_floor.is_finite());
        assert!(r_peak.is_finite());
        assert!(r_high.is_finite());
        assert!(r_peak > r_floor);

        // Morph puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let puck_x = canvas.x + view.morph_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.morph_pos.1) * canvas.height;
        assert!(view.hit_test_vowel_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_vowel_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PHONETIC FORMANT FILTER HUD"));
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
    fn test_turn94_gui_modal_window_lifecycles_auto_wah_and_formant_filter() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Dynamic Resonant Auto-Wah Filter HUD
        assert!(!view.is_auto_wah_hud_open());
        view.open_auto_wah_hud();
        assert!(view.is_auto_wah_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_auto_wah_hud();
        assert!(!view.is_auto_wah_hud_open());

        // 2. 5-Vowel Phonetic Formant Filter Resonator HUD
        assert!(!view.is_formant_filter_hud_open());
        view.open_formant_filter_hud();
        assert!(view.is_formant_filter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_formant_filter_hud();
        assert!(!view.is_formant_filter_hud_open());
    }

    #[test]
    fn test_turn94_gui_lock_free_param_bus_dispatch_auto_wah_and_formant_filter() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Auto-Wah (offsets 0..=6) and Formant Filter (offsets 0..=6)
        for offset in 0..=6 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Auto-Wah sync
        view.auto_wah_view.sensitivity = 0.90;
        view.auto_wah_view.base_freq_hz = 450.0;
        view.auto_wah_view.sweep_range_octaves = 4.2;
        view.auto_wah_view.resonance_q = 9.0;
        view.auto_wah_view.attack_ms = 8.0;
        view.auto_wah_view.release_ms = 90.0;
        view.auto_wah_view.mix = 0.85;

        view.sync_auto_wah_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("sensitivity"), Some(&0.90));
        assert_eq!(view.device_rack_state.node_param_values.get("base_freq_hz"), Some(&450.0));
        assert_eq!(view.device_rack_state.node_param_values.get("sweep_range_octaves"), Some(&4.2));
        assert_eq!(view.device_rack_state.node_param_values.get("resonance_q"), Some(&9.0));
        assert_eq!(view.device_rack_state.node_param_values.get("attack_ms"), Some(&8.0));
        assert_eq!(view.device_rack_state.node_param_values.get("release_ms"), Some(&90.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mix"), Some(&0.85));

        let pid_sens = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_res = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        assert_eq!(arc_bus.get(pid_sens), Some(0.90));
        assert_eq!(arc_bus.get(pid_res), Some(9.0));

        // Test Formant Filter sync
        view.formant_filter_view.resonance_q = 15.0;
        view.formant_filter_view.peak_boost_db = 12.0;
        view.formant_filter_view.vocal_tract_scale = 1.15;
        view.formant_filter_view.drive_warmth_db = 5.0;
        view.formant_filter_view.dry_wet_pct = 80.0;
        view.formant_filter_view.morph_pos = (0.70, 0.50);

        view.sync_formant_filter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("resonance_q"), Some(&15.0));
        assert_eq!(view.device_rack_state.node_param_values.get("peak_boost_db"), Some(&12.0));
        assert_eq!(view.device_rack_state.node_param_values.get("vocal_tract_scale"), Some(&1.15));
        assert_eq!(view.device_rack_state.node_param_values.get("drive_warmth_db"), Some(&5.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet_pct"), Some(&80.0));
        assert_eq!(view.device_rack_state.node_param_values.get("morph_pos_x"), Some(&0.70));
        assert_eq!(view.device_rack_state.node_param_values.get("morph_pos_y"), Some(&0.50));

        let pid_q = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_mx = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 5);
        let pid_my = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 6);
        assert_eq!(arc_bus.get(pid_q), Some(15.0));
        assert_eq!(arc_bus.get(pid_mx), Some(0.70));
        assert_eq!(arc_bus.get(pid_my), Some(0.50));
    }

    #[test]
    fn test_turn94_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Auto-Wah trigger propagation from rack
        view.device_rack_state.requested_open_auto_wah_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_auto_wah_hud);
        assert!(view.is_auto_wah_hud_open());
        view.close_auto_wah_hud();

        // 2. Formant Filter trigger propagation from inspector
        view.inspector_state.requested_open_formant_filter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_formant_filter_hud);
        assert!(view.is_formant_filter_hud_open());
        view.close_formant_filter_hud();
    }

    #[test]
    fn test_turn94_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let wah_action = palette.actions.iter().find(|a| a.action_id == "open_auto_wah_hud");
        assert!(wah_action.is_some(), "open_auto_wah_hud must be registered in command palette");
        let wah = wah_action.unwrap();
        assert_eq!(wah.category, "Filter / EQ");

        let formant_action = palette.actions.iter().find(|a| a.action_id == "open_formant_filter_hud");
        assert!(formant_action.is_some(), "open_formant_filter_hud must be registered in command palette");
        let formant = formant_action.unwrap();
        assert_eq!(formant.category, "Filter / EQ");
    }
}
