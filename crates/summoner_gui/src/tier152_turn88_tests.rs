#![allow(clippy::all)]

//! Test suite for Turn #88 (Sprint Turn #14) — Multi-Stage Neural Vocoder Formant Morpher HUD
//! and Neural Wavetable Morphing Synthesizer HUD Convergence (Milestones 33 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_vocoder_morph_view::{
        NeuralVocoderMorphView, VocoderMorphMode, NEURAL_VOCODER_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_wavetable_view::{
        LatentArchitecture, NeuralWavetableView, NEURAL_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn88_pure_neural_vocoder_morph_initialization_and_kinematics() {
        let mut view = NeuralVocoderMorphView::new();
        assert_eq!(view.mode, VocoderMorphMode::NeuralLpc16);
        assert!((view.formant_shift_semitones - 0.0).abs() < 1e-4);
        assert!((view.articulation_depth_pct - 82.5).abs() < 1e-4);
        assert!((view.carrier_harmonics_mix - 65.0).abs() < 1e-4);
        assert!((view.voicing_probability_pct - 94.2).abs() < 1e-4);
        assert_eq!(view.lpc_prediction_order, 16);
        assert_eq!(NEURAL_VOCODER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode profiles & names
        let modes = [
            (VocoderMorphMode::NeuralLpc16, "NEURAL LPC-16"),
            (VocoderMorphMode::PhoneticVowel, "PHONETIC VOWEL"),
            (VocoderMorphMode::RoboticCarrier, "ROBOTIC CARRIER"),
            (VocoderMorphMode::CepstralMorph, "CEPSTRAL MORPH"),
            (VocoderMorphMode::SpectralResynth, "SPECTRAL RESYNTH"),
        ];
        for (m, name) in modes {
            view.set_mode(m);
            assert_eq!(view.mode, m);
            assert_eq!(m.mode_name(), name);
        }

        // Coordinate normalizations & round-trips for F1
        let f1_test = 650.0;
        let f1_norm = NeuralVocoderMorphView::f1_freq_to_normalized(f1_test);
        let f1_back = NeuralVocoderMorphView::normalized_to_f1_freq(f1_norm);
        assert!((f1_back - f1_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for F2
        let f2_test = 1750.0;
        let f2_norm = NeuralVocoderMorphView::f2_freq_to_normalized(f2_test);
        let f2_back = NeuralVocoderMorphView::normalized_to_f2_freq(f2_norm);
        assert!((f2_back - f2_test).abs() < 1e-3);

        // Formant shift normalizations & round-trips
        let shift_test = -7.5;
        let shift_norm = NeuralVocoderMorphView::formant_shift_to_normalized(shift_test);
        let shift_back = NeuralVocoderMorphView::normalized_to_formant_shift(shift_norm);
        assert!((shift_back - shift_test).abs() < 1e-3);

        // Spectral envelope evaluation
        let env_mag = view.evaluate_spectral_envelope(1000.0);
        assert!(env_mag > 0.0);

        // Puck hit testing
        let canvas = Rect::new(100.0, 100.0, 200.0, 200.0);
        let puck_x = canvas.x + view.formant_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.formant_puck_pos.1) * canvas.height;
        assert!(view.hit_test_formant_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_formant_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTI-STAGE NEURAL VOCODER FORMANT MORPHER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn88_pure_neural_wavetable_initialization_and_simulation() {
        let mut view = NeuralWavetableView::new();
        assert_eq!(view.architecture, LatentArchitecture::VaeContinuous);
        assert!((view.latent_z.0 - 0.62).abs() < 1e-4);
        assert!((view.latent_z.1 - (-0.45)).abs() < 1e-4);
        assert!((view.latent_z.2 - 0.18).abs() < 1e-4);
        assert!((view.morph_speed_hz - 0.85).abs() < 1e-4);
        assert!((view.orbit_radius - 0.45).abs() < 1e-4);
        assert_eq!(NEURAL_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & nominal parameters
        let archs = [
            (LatentArchitecture::VaeContinuous, 0.85, 0.45, 98.4, "VAE CONTINUOUS"),
            (LatentArchitecture::TransformerDyn, 2.40, 0.75, 97.8, "TRANSFORMER DYN"),
            (LatentArchitecture::DiffusionRes, 0.35, 0.30, 99.6, "DIFFUSION RES"),
            (LatentArchitecture::Hypersphere4D, 1.20, 0.90, 98.9, "HYPERSPHERE 4D"),
            (LatentArchitecture::SpectralFlow, 0.65, 0.55, 99.2, "SPECTRAL FLOW"),
        ];
        for (arch, spd, rad, fid, name) in archs {
            view.set_architecture(arch);
            assert_eq!(view.architecture, arch);
            assert!((view.morph_speed_hz - spd).abs() < 1e-4);
            assert!((view.orbit_radius - rad).abs() < 1e-4);
            assert!((view.reconstruction_quality_pct - fid).abs() < 1e-4);
            assert_eq!(arch.architecture_name(), name);
        }

        // Coordinate normalizations & round-trips
        let coord_test = 1.35;
        let coord_norm = NeuralWavetableView::coord_to_normalized(coord_test);
        let coord_back = NeuralWavetableView::normalized_to_coord(coord_norm);
        assert!((coord_back - coord_test).abs() < 1e-3);

        let speed_test = 5.25;
        let speed_norm = NeuralWavetableView::speed_to_normalized(speed_test);
        let speed_back = NeuralWavetableView::normalized_to_speed(speed_norm);
        assert!((speed_back - speed_test).abs() < 1e-3);

        let radius_test = 1.15;
        let radius_norm = NeuralWavetableView::radius_to_normalized(radius_test);
        let radius_back = NeuralWavetableView::normalized_to_radius(radius_norm);
        assert!((radius_back - radius_test).abs() < 1e-3);

        // 3D perspective projection
        let proj = view.project_3d_latent(0.5, -0.2, 0.8, (100.0, 100.0), 50.0);
        assert!(proj.0 > 0.0 && proj.1 > 0.0);

        // Single-cycle sample evaluation
        let sample = view.evaluate_wavetable_sample(0.25);
        assert!(sample.abs() <= 1.0);

        // Harmonic overtone energy evaluation
        let harm = view.evaluate_harmonic_energy(3);
        assert!(harm > 0.0 && harm <= 1.0);

        // Puck hit testing
        let canvas = Rect::new(50.0, 50.0, 150.0, 150.0);
        let puck_x = canvas.x + view.latent_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.latent_puck_pos.1) * canvas.height;
        assert!(view.hit_test_latent_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_latent_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL WAVETABLE SYNTHESIS & 3D LATENT TRAJECTORY HUD"));
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
    fn test_turn88_gui_modal_window_lifecycles_vocoder_and_wavetable() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Neural Vocoder Morpher HUD
        assert!(!view.is_neural_vocoder_morph_hud_open());
        view.open_neural_vocoder_morph_hud();
        assert!(view.is_neural_vocoder_morph_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_vocoder_morph_hud();
        assert!(!view.is_neural_vocoder_morph_hud_open());

        // 2. Neural Wavetable Synthesizer HUD
        assert!(!view.is_neural_wavetable_hud_open());
        view.open_neural_wavetable_hud();
        assert!(view.is_neural_wavetable_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_wavetable_hud();
        assert!(!view.is_neural_wavetable_hud_open());
    }

    #[test]
    fn test_turn88_gui_lock_free_param_bus_dispatch_vocoder_and_wavetable() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Vocoder (offsets 0..=5) and Wavetable (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Vocoder Morpher sync
        let norm_f1 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::f1_freq_to_normalized(620.0);
        let norm_f2 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::f2_freq_to_normalized(1850.0);
        view.neural_vocoder_morph_view.formant_puck_pos = (norm_f1, norm_f2);
        view.neural_vocoder_morph_view.articulation_depth_pct = 88.0;
        view.neural_vocoder_morph_view.carrier_harmonics_mix = 70.0;
        view.neural_vocoder_morph_view.voicing_probability_pct = 95.0;
        view.neural_vocoder_morph_view.formant_shift_semitones = 3.0;

        view.sync_neural_vocoder_morph_hud_state();

        assert!((view.device_rack_state.node_param_values.get("f1_freq_hz").copied().unwrap_or(0.0) - 620.0).abs() < 1e-2);
        assert!((view.device_rack_state.node_param_values.get("f2_freq_hz").copied().unwrap_or(0.0) - 1850.0).abs() < 1e-2);
        assert_eq!(view.device_rack_state.node_param_values.get("articulation_depth_pct"), Some(&88.0));
        assert_eq!(view.inspector_state.node_param_values.get("carrier_harmonics_mix"), Some(&70.0));
        assert_eq!(view.inspector_state.node_param_values.get("voicing_probability_pct"), Some(&95.0));

        let pid_f1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_f2 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert!((arc_bus.get(pid_f1).unwrap_or(0.0) - 620.0).abs() < 1e-2);
        assert!((arc_bus.get(pid_f2).unwrap_or(0.0) - 1850.0).abs() < 1e-2);

        // Test Neural Wavetable Synthesizer sync
        view.neural_wavetable_view.latent_z = (1.25, -0.80, 0.45);
        view.neural_wavetable_view.morph_speed_hz = 3.20;
        view.neural_wavetable_view.orbit_radius = 0.85;
        view.neural_wavetable_view.spectral_entropy_bits = 4.10;

        view.sync_neural_wavetable_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("latent_z1"), Some(&1.25));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_z2"), Some(&-0.80));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_z3"), Some(&0.45));
        assert_eq!(view.inspector_state.node_param_values.get("morph_speed_hz"), Some(&3.20));
        assert_eq!(view.inspector_state.node_param_values.get("orbit_radius"), Some(&0.85));

        let pid_z1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_z2 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_z1), Some(1.25));
        assert_eq!(arc_bus.get(pid_z2), Some(-0.80));
    }

    #[test]
    fn test_turn88_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Vocoder Morpher trigger propagation from rack
        view.device_rack_state.requested_open_neural_vocoder_morph_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_vocoder_morph_hud);
        assert!(view.is_neural_vocoder_morph_hud_open());
        view.close_neural_vocoder_morph_hud();

        // 2. Neural Wavetable trigger propagation from inspector
        view.inspector_state.requested_open_neural_wavetable_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_wavetable_hud);
        assert!(view.is_neural_wavetable_hud_open());
        view.close_neural_wavetable_hud();
    }

    #[test]
    fn test_turn88_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let voc_action = palette.actions.iter().find(|a| a.action_id == "open_neural_vocoder_morph_hud");
        assert!(voc_action.is_some(), "open_neural_vocoder_morph_hud must be registered in command palette");
        let voc = voc_action.unwrap();
        assert_eq!(voc.category, "Synthesis");

        let wt_action = palette.actions.iter().find(|a| a.action_id == "open_neural_wavetable_hud");
        assert!(wt_action.is_some(), "open_neural_wavetable_hud must be registered in command palette");
        let wt = wt_action.unwrap();
        assert_eq!(wt.category, "Synthesis");
    }
}
