#![allow(clippy::all)]

//! Test suite for Turn #87 (Sprint Turn #13) — Neural Speech-to-Singing Prosody & Intonation Transformer HUD
//! and Neural Polyphonic Choir Formant Morpher HUD Convergence (Milestones 33 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_speech_to_singing_view::{
        NeuralSpeechToSingingView, VocalModel, SINGING_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_choir_formant_view::{
        ChoirEnsembleType, NeuralChoirFormantView, CHOIR_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn87_pure_neural_speech_to_singing_initialization_and_kinematics() {
        let mut view = NeuralSpeechToSingingView::new();
        assert_eq!(view.vocal_model, VocalModel::BelCantoOperaTenor);
        assert!((view.target_pitch_hz - 220.0).abs() < 1e-4);
        assert!((view.vibrato_depth_cents - 45.0).abs() < 1e-4);
        assert!((view.vibrato_rate_hz - 5.8).abs() < 1e-4);
        assert!((view.formant_shift_semitones - 0.0).abs() < 1e-4);
        assert!((view.f0_confidence - 0.985).abs() < 1e-4);
        assert_eq!(SINGING_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Model profiles & nominal parameters
        let models = [
            (VocalModel::BelCantoOperaTenor, 220.0, 45.0, 5.8, "BEL CANTO OPERA"),
            (VocalModel::PopModernVocalist, 440.0, 15.0, 6.2, "MODERN POP LEAD"),
            (VocalModel::ChoralPolyphonicChoir, 330.0, 25.0, 5.0, "POLYPHONIC CHOIR"),
            (VocalModel::MicrotonalMaqamRetune, 293.66, 30.0, 4.5, "MICROTONAL MAQAM"),
            (VocalModel::ExperimentalFormantMorph, 164.81, 60.0, 7.0, "NEURAL VOCODER"),
        ];
        for (m, pitch, vib, rate, name) in models {
            view.set_vocal_model(m);
            assert_eq!(view.vocal_model, m);
            assert!((view.target_pitch_hz - pitch).abs() < 1e-2);
            assert!((view.vibrato_depth_cents - vib).abs() < 1e-4);
            assert!((view.vibrato_rate_hz - rate).abs() < 1e-4);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips
        let pitch_test = 440.0;
        let pitch_norm = NeuralSpeechToSingingView::pitch_to_normalized(pitch_test);
        let pitch_back = NeuralSpeechToSingingView::normalized_to_pitch(pitch_norm);
        assert!((pitch_back - pitch_test).abs() < 1e-2);

        let vib_test = 65.0;
        let vib_norm = NeuralSpeechToSingingView::vibrato_to_normalized(vib_test);
        let vib_back = NeuralSpeechToSingingView::normalized_to_vibrato(vib_norm);
        assert!((vib_back - vib_test).abs() < 1e-3);

        // Vocal synthesis simulation update & confidence check
        view.formant_shift_semitones = 3.5;
        view.update_vocal_synthesis();
        assert!(view.f0_confidence < 0.985);
        assert!(view.formant_envelope[0] > 0.0);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL SPEECH-TO-SINGING SYNTHESIS & RETUNING HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn87_pure_neural_choir_formant_initialization_and_simulation() {
        let mut view = NeuralChoirFormantView::new();
        assert_eq!(view.ensemble, ChoirEnsembleType::ClassicalSATB);
        assert!((view.formant_f1_hz - 650.0).abs() < 1e-4);
        assert!((view.formant_f2_hz - 1400.0).abs() < 1e-4);
        assert_eq!(view.voice_count, 32);
        assert!((view.neural_morph_blend - 0.88).abs() < 1e-4);
        assert_eq!(CHOIR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Ensemble profiles & nominal parameters
        let ensembles = [
            (ChoirEnsembleType::ClassicalSATB, 650.0, 1400.0, 32, "CLASSICAL SATB CHOIR"),
            (ChoirEnsembleType::BulgarianWomensChoir, 780.0, 1950.0, 16, "BULGARIAN FOLK CHOIR"),
            (ChoirEnsembleType::GregorianMonasticChant, 450.0, 950.0, 24, "GREGORIAN MONASTIC"),
            (ChoirEnsembleType::ContemporaryVocalEnsemble, 580.0, 1700.0, 8, "CONTEMPORARY ENSEMBLE"),
            (ChoirEnsembleType::GospelChoirWallOfSound, 720.0, 1600.0, 48, "GOSPEL WALL OF SOUND"),
        ];
        for (ens, f1, f2, count, name) in ensembles {
            view.set_ensemble(ens);
            assert_eq!(view.ensemble, ens);
            assert!((view.formant_f1_hz - f1).abs() < 1e-4);
            assert!((view.formant_f2_hz - f2).abs() < 1e-4);
            assert_eq!(view.voice_count, count);
            assert_eq!(ens.ensemble_name(), name);
        }

        // Coordinate normalizations & round-trips
        let f1_test = 500.0;
        let f1_norm = NeuralChoirFormantView::f1_to_normalized(f1_test);
        let f1_back = NeuralChoirFormantView::normalized_to_f1(f1_norm);
        assert!((f1_back - f1_test).abs() < 1e-3);

        let f2_test = 1800.0;
        let f2_norm = NeuralChoirFormantView::f2_to_normalized(f2_test);
        let f2_back = NeuralChoirFormantView::normalized_to_f2(f2_norm);
        assert!((f2_back - f2_test).abs() < 1e-3);

        // Simulation update & formant peaks check
        view.neural_morph_blend = 0.95;
        view.update_choir_simulation();
        assert!(view.voice_formant_peaks[0] > 0.0);
        assert!(view.voice_formant_peaks[1] > 0.0);
        assert!(view.formant_f3_hz >= 1800.0);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL POLYPHONIC CHOIR FORMANT MORPHER HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{
        show_modern_device_rack_with_context, ModernDeviceRackState,
    };
    use crate::views::modern_inspector::{
        show_modern_inspector_with_context, ModernInspectorState,
    };
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn87_gui_award_winning_view_speech_and_choir_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Neural Speech-to-Singing Synthesis HUD
        view.open_neural_speech_to_singing_hud();
        assert!(view.is_neural_speech_to_singing_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_speech_to_singing_hud();
        assert!(!view.is_neural_speech_to_singing_hud_open());

        // 2. Neural Choir Formant Morpher HUD
        view.open_neural_choir_formant_hud();
        assert!(view.is_neural_choir_formant_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_choir_formant_hud();
        assert!(!view.is_neural_choir_formant_hud_open());
    }

    #[test]
    fn test_turn87_gui_lock_free_param_bus_dispatch_speech_and_choir() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Speech (offsets 0..=4) and Choir (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Speech-to-Singing sync
        view.neural_speech_to_singing_view.target_pitch_hz = 330.0;
        view.neural_speech_to_singing_view.vibrato_depth_cents = 55.0;
        view.neural_speech_to_singing_view.vibrato_rate_hz = 6.0;
        view.neural_speech_to_singing_view.formant_shift_semitones = 2.0;
        view.neural_speech_to_singing_view.f0_confidence = 0.96;

        view.sync_neural_speech_to_singing_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("target_pitch_hz"), Some(&330.0));
        assert_eq!(view.device_rack_state.node_param_values.get("vibrato_depth_cents"), Some(&55.0));
        assert_eq!(view.device_rack_state.node_param_values.get("vibrato_rate_hz"), Some(&6.0));
        assert_eq!(view.inspector_state.node_param_values.get("formant_shift_semitones"), Some(&2.0));
        assert_eq!(view.inspector_state.node_param_values.get("f0_confidence"), Some(&0.96));

        let pid_pitch = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_vib = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_pitch), Some(330.0));
        assert_eq!(arc_bus.get(pid_vib), Some(55.0));

        // Test Choir Formant Morpher sync
        view.neural_choir_formant_view.formant_f1_hz = 720.0;
        view.neural_choir_formant_view.formant_f2_hz = 1850.0;
        view.neural_choir_formant_view.formant_f3_hz = 3000.0;
        view.neural_choir_formant_view.voice_count = 24;
        view.neural_choir_formant_view.neural_morph_blend = 0.92;
        view.neural_choir_formant_view.tract_scale = 1.05;

        view.sync_neural_choir_formant_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("formant_f1_hz"), Some(&720.0));
        assert_eq!(view.device_rack_state.node_param_values.get("formant_f2_hz"), Some(&1850.0));
        assert_eq!(view.device_rack_state.node_param_values.get("voice_count"), Some(&24.0));
        assert_eq!(view.inspector_state.node_param_values.get("neural_morph_blend"), Some(&0.92));
        assert_eq!(view.inspector_state.node_param_values.get("tract_scale"), Some(&1.05));

        let pid_f1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_f2 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_f1), Some(720.0));
        assert_eq!(arc_bus.get(pid_f2), Some(1850.0));
    }

    #[test]
    fn test_turn87_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Speech-to-Singing trigger propagation from rack
        view.device_rack_state.requested_open_neural_speech_to_singing_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_speech_to_singing_hud);
        assert!(view.is_neural_speech_to_singing_hud_open());
        view.close_neural_speech_to_singing_hud();

        // 2. Choir Formant trigger propagation from inspector
        view.inspector_state.requested_open_neural_choir_formant_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_choir_formant_hud);
        assert!(view.is_neural_choir_formant_hud_open());
        view.close_neural_choir_formant_hud();
    }

    #[test]
    fn test_turn87_gui_modern_device_rack_and_inspector_headless_render() {
        let ctx = egui::Context::default();
        let mut rack_state = ModernDeviceRackState::default();
        let mut inspector_state = ModernInspectorState::default();

        rack_state.selected_node_kind = Some("NeuralSpeechToSingingView".to_string());
        rack_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "NeuralSpeechToSingingView".to_string(),
            display_name: "Speech To Singing".to_string(),
            is_bypassed: false,
        }];

        inspector_state.selected_node_kind = Some("NeuralChoirFormantView".to_string());
        inspector_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "NeuralChoirFormantView".to_string(),
            display_name: "Neural Choir Formant".to_string(),
            is_bypassed: false,
        }];

        let bus = ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(
                    ui,
                    &mut rack_state,
                    None,
                    Some(&bus),
                    1,
                );
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });

        assert_eq!(rack_state.chain_devices.len(), 1);
        assert_eq!(inspector_state.chain_devices.len(), 1);
    }

    #[test]
    fn test_turn87_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let speech_act = palette.actions.iter().find(|a| a.action_id == "open_neural_speech_to_singing_hud");
        assert!(speech_act.is_some());
        assert_eq!(speech_act.unwrap().category, "Synthesis");

        let choir_act = palette.actions.iter().find(|a| a.action_id == "open_neural_choir_formant_hud");
        assert!(choir_act.is_some());
        assert_eq!(choir_act.unwrap().category, "Synthesis");
    }
}
