#![allow(clippy::all)]

//! Test suite for Turn #89 (Sprint Turn #15) — Neural Phoneme Morphing Vocoder HUD
//! and Neural Blind Dereverberation & Deconvolution HUD Convergence (Milestones 33 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_phoneme_view::{
        NeuralPhonemeView, PhonemeModel, PHONEME_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_dereverb_view::{
        DereverbModel, NeuralDereverbView, DEREVERB_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn89_pure_neural_phoneme_initialization_and_kinematics() {
        let mut view = NeuralPhonemeView::new();
        assert_eq!(view.model, PhonemeModel::VowelFormantMorph);
        assert!((view.formant_f1_hz - 500.0).abs() < 1e-3);
        assert!((view.formant_f2_hz - 1800.0).abs() < 1e-3);
        assert!((view.vocal_tract_cm - 17.0).abs() < 1e-3);
        assert!((view.latent_style_weight - 0.80).abs() < 1e-3);
        assert_eq!(PHONEME_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Model profiles & names
        let models = [
            (PhonemeModel::VowelFormantMorph, "VOWEL MORPH"),
            (PhonemeModel::WhisperToVoicedTransfer, "WHISPER TRANSFER"),
            (PhonemeModel::RoboticVocoderCarrier, "ROBOT VOCODER"),
            (PhonemeModel::AlienFormantShift, "ALIEN SHIFT"),
            (PhonemeModel::LatentDiffusionInterpolate, "LATENT DIFFUSION"),
        ];
        for (m, name) in models {
            view.set_model(m);
            assert_eq!(view.model, m);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips for F1
        let f1_test = 620.0;
        let f1_norm = NeuralPhonemeView::f1_to_normalized(f1_test);
        let f1_back = NeuralPhonemeView::normalized_to_f1(f1_norm);
        assert!((f1_back - f1_test).abs() < 1e-2);

        // Coordinate normalizations & round-trips for F2
        let f2_test = 2100.0;
        let f2_norm = NeuralPhonemeView::f2_to_normalized(f2_test);
        let f2_back = NeuralPhonemeView::normalized_to_f2(f2_norm);
        assert!((f2_back - f2_test).abs() < 1e-2);

        // Tract length normalizations & round-trips
        let tract_test = 14.5;
        let tract_norm = NeuralPhonemeView::tract_to_normalized(tract_test);
        let tract_back = NeuralPhonemeView::normalized_to_tract(tract_norm);
        assert!((tract_back - tract_test).abs() < 1e-3);

        // Spectral envelope evaluation
        let mag_db = view.evaluate_spectral_envelope_db(1000.0);
        assert!(mag_db >= -36.0 && mag_db <= 12.0);

        // Puck hit testing
        let canvas = Rect::new(50.0, 50.0, 150.0, 150.0);
        let puck_x = canvas.x + view.phoneme_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.phoneme_puck_pos.1) * canvas.height;
        assert!(view.hit_test_phoneme_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_phoneme_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL AUDIO LATENT STYLE TRANSFER & PHONEME MORPHING VOCODER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn89_pure_neural_dereverb_initialization_and_simulation() {
        let mut view = NeuralDereverbView::new();
        assert_eq!(view.model, DereverbModel::NeuralSpectralMaskUNet);
        assert!((view.suppression_depth_db - 18.0).abs() < 1e-3);
        assert!((view.direct_to_reverberant_ratio_db - 6.0).abs() < 1e-3);
        assert!((view.early_reflection_cancel - 0.75).abs() < 1e-3);
        assert!((view.late_tail_suppression - 0.80).abs() < 1e-3);
        assert!((view.room_rt60_s - 1.5).abs() < 1e-3);
        assert!((view.neural_blend_pct - 85.0).abs() < 1e-3);
        assert_eq!(DEREVERB_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & nominal parameters
        let models = [
            (DereverbModel::NeuralSpectralMaskUNet, 18.0, 6.0, 1.5, "NEURAL SPECTRAL U-NET"),
            (DereverbModel::WeightedPredictionErrorWPE, 12.0, 3.0, 1.2, "STATISTICAL WPE (M-CH)"),
            (DereverbModel::DiffusionDeconvolution, 24.0, 12.0, 2.2, "DIFFUSION DECONVOLUTION"),
            (DereverbModel::CathedralAcousticHall, 30.0, -6.0, 4.8, "CATHEDRAL LONG-TAIL"),
            (DereverbModel::ConferenceRoomAutomix, 15.0, 8.0, 0.6, "CONFERENCE ROOM AUTOMIX"),
        ];
        for (m, dep, drr, rt, name) in models {
            view.set_model(m);
            assert_eq!(view.model, m);
            assert!((view.suppression_depth_db - dep).abs() < 1e-3);
            assert!((view.direct_to_reverberant_ratio_db - drr).abs() < 1e-3);
            assert!((view.room_rt60_s - rt).abs() < 1e-3);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips
        let depth_test = 22.5;
        let depth_norm = NeuralDereverbView::depth_to_normalized(depth_test);
        let depth_back = NeuralDereverbView::normalized_to_depth(depth_norm);
        assert!((depth_back - depth_test).abs() < 1e-3);

        let drr_test = 4.5;
        let drr_norm = NeuralDereverbView::drr_to_normalized(drr_test);
        let drr_back = NeuralDereverbView::normalized_to_drr(drr_norm);
        assert!((drr_back - drr_test).abs() < 1e-3);

        // Simulation curves
        assert_eq!(view.energy_decay_curve.len(), 16);
        assert_eq!(view.spectral_mask_bands.len(), 8);

        // Puck hit testing
        let canvas = Rect::new(40.0, 40.0, 160.0, 160.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_dereverb_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_dereverb_puck((puck_x + 55.0, puck_y + 55.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL ACOUSTIC BLIND DEREVERBERATION & DECONVOLUTION HUD"));
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
    fn test_turn89_gui_modal_window_lifecycles_phoneme_and_dereverb() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Neural Phoneme Morpher HUD
        assert!(!view.is_neural_phoneme_hud_open());
        view.open_neural_phoneme_hud();
        assert!(view.is_neural_phoneme_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_phoneme_hud();
        assert!(!view.is_neural_phoneme_hud_open());

        // 2. Neural Blind Dereverberation HUD
        assert!(!view.is_neural_dereverb_hud_open());
        view.open_neural_dereverb_hud();
        assert!(view.is_neural_dereverb_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_dereverb_hud();
        assert!(!view.is_neural_dereverb_hud_open());
    }

    #[test]
    fn test_turn89_gui_lock_free_param_bus_dispatch_phoneme_and_dereverb() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Phoneme (offsets 0..=5) and Dereverb (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Phoneme Morpher sync
        view.neural_phoneme_view.formant_f1_hz = 550.0;
        view.neural_phoneme_view.formant_f2_hz = 1950.0;
        view.neural_phoneme_view.formant_f3_hz = 2750.0;
        view.neural_phoneme_view.vocal_tract_cm = 16.5;
        view.neural_phoneme_view.latent_style_weight = 0.85;
        view.neural_phoneme_view.glottal_open_quotient = 0.58;

        view.sync_neural_phoneme_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("formant_f1_hz"), Some(&550.0));
        assert_eq!(view.device_rack_state.node_param_values.get("formant_f2_hz"), Some(&1950.0));
        assert_eq!(view.device_rack_state.node_param_values.get("formant_f3_hz"), Some(&2750.0));
        assert_eq!(view.device_rack_state.node_param_values.get("vocal_tract_cm"), Some(&16.5));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_style_weight"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("glottal_open_quotient"), Some(&0.58));

        let pid_f1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_f2 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_f1), Some(550.0));
        assert_eq!(arc_bus.get(pid_f2), Some(1950.0));

        // Test Neural Dereverberation sync
        view.neural_dereverb_view.suppression_depth_db = 21.0;
        view.neural_dereverb_view.direct_to_reverberant_ratio_db = 7.5;
        view.neural_dereverb_view.early_reflection_cancel = 0.82;
        view.neural_dereverb_view.late_tail_suppression = 0.88;
        view.neural_dereverb_view.room_rt60_s = 2.1;
        view.neural_dereverb_view.neural_blend_pct = 90.0;

        view.sync_neural_dereverb_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("suppression_depth_db"), Some(&21.0));
        assert_eq!(view.device_rack_state.node_param_values.get("direct_to_reverberant_ratio_db"), Some(&7.5));
        assert_eq!(view.device_rack_state.node_param_values.get("early_reflection_cancel"), Some(&0.82));
        assert_eq!(view.inspector_state.node_param_values.get("late_tail_suppression"), Some(&0.88));
        assert_eq!(view.inspector_state.node_param_values.get("room_rt60_s"), Some(&2.1));
        assert_eq!(view.inspector_state.node_param_values.get("neural_blend_pct"), Some(&90.0));

        let pid_depth = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_drr = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_depth), Some(21.0));
        assert_eq!(arc_bus.get(pid_drr), Some(7.5));
    }

    #[test]
    fn test_turn89_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Phoneme Morpher trigger propagation from rack
        view.device_rack_state.requested_open_neural_phoneme_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_phoneme_hud);
        assert!(view.is_neural_phoneme_hud_open());
        view.close_neural_phoneme_hud();

        // 2. Neural Dereverb trigger propagation from inspector
        view.inspector_state.requested_open_neural_dereverb_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_dereverb_hud);
        assert!(view.is_neural_dereverb_hud_open());
        view.close_neural_dereverb_hud();
    }

    #[test]
    fn test_turn89_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let phon_action = palette.actions.iter().find(|a| a.action_id == "open_neural_phoneme_hud");
        assert!(phon_action.is_some(), "open_neural_phoneme_hud must be registered in command palette");
        let phon = phon_action.unwrap();
        assert_eq!(phon.category, "Synthesis");

        let dereverb_action = palette.actions.iter().find(|a| a.action_id == "open_neural_dereverb_hud");
        assert!(dereverb_action.is_some(), "open_neural_dereverb_hud must be registered in command palette");
        let dr = dereverb_action.unwrap();
        assert_eq!(dr.category, "Restoration");
    }
}
