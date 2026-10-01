#![cfg(feature = "gui")]

// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 116 Test Suite — Turn #52 (Sprint Review Turn #18):
//! 2D Latent Timbre Morphing Orb & Spectral Trajectory HUD (Milestone 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_morph_orb_view::{
        NeuralMorphOrbView, NeuralOrbitMode, NeuralTimbrePreset, MAX_MORPH_COORD,
        MIN_MORPH_COORD, NEURAL_ORB_PUCK_HIT_RADIUS, NUM_TIMBRE_ANCHORS,
    };

    #[test]
    fn test_turn52_neural_morph_orb_view_initialization() {
        let view = NeuralMorphOrbView::new();

        assert_eq!(view.preset, NeuralTimbrePreset::VocalToCelloMorph);
        assert!((view.morph_x - (-0.35)).abs() < 1e-4);
        assert!((view.morph_y - 0.70).abs() < 1e-4);
        assert_eq!(view.orbit_mode, NeuralOrbitMode::ManualStatic);
        assert_eq!(view.anchor_weights.len(), NUM_TIMBRE_ANCHORS);

        let sum: f32 = view.anchor_weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "Weights must sum to 1.0, got {}", sum);

        assert!(view.spectral_centroid_hz > 100.0 && view.spectral_centroid_hz < 12000.0);
        assert!(view.harmonic_warmth >= 0.0 && view.harmonic_warmth <= 1.0);
        assert!(view.flux_entropy >= 0.0 && view.flux_entropy <= 1.0);
    }

    #[test]
    fn test_turn52_neural_morph_presets() {
        let mut view = NeuralMorphOrbView::new();

        let presets = [
            (NeuralTimbrePreset::VocalToCelloMorph, -0.35, 0.70),
            (NeuralTimbrePreset::GlassToSubBass, 0.25, -0.65),
            (NeuralTimbrePreset::AnalogToQuantumParticle, 0.05, -0.55),
            (NeuralTimbrePreset::SitarToBrassHyperHybrid, -0.25, 0.40),
            (NeuralTimbrePreset::CinematicTimbreSwarm, 0.00, 0.00),
            (NeuralTimbrePreset::CyberneticIndustrial, 0.65, -0.45),
        ];

        for (p, exp_x, exp_y) in presets {
            view.set_preset(p);
            assert_eq!(view.preset, p);
            assert!((view.morph_x - exp_x).abs() < 1e-3, "Preset {:?} X mismatch", p);
            assert!((view.morph_y - exp_y).abs() < 1e-3, "Preset {:?} Y mismatch", p);

            let sum: f32 = view.anchor_weights.iter().sum();
            assert!((sum - 1.0).abs() < 1e-4, "Preset {:?} weights sum mismatch", p);
        }
    }

    #[test]
    fn test_turn52_rbf_anchor_attraction_weights_sum_to_one() {
        let mut view = NeuralMorphOrbView::new();
        let test_coords = [
            (-1.0, -1.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (-0.8, 0.5),
            (0.4, -0.9),
            (0.9, 0.2),
            (-0.3, -0.7),
        ];

        for (x, y) in test_coords {
            view.set_morph_coords(x, y);
            let sum: f32 = view.anchor_weights.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-4,
                "Coords ({}, {}) weights sum must be 1.0, got {}",
                x,
                y,
                sum
            );
            for (i, &w) in view.anchor_weights.iter().enumerate() {
                assert!(w >= 0.0 && w <= 1.0, "Weight {} out of bounds: {}", i, w);
            }
        }
    }

    #[test]
    fn test_turn52_coordinate_clamping_and_normalization() {
        let mut view = NeuralMorphOrbView::new();

        view.set_morph_coords(-5.0, 10.0);
        assert_eq!(view.morph_x, MIN_MORPH_COORD);
        assert_eq!(view.morph_y, MAX_MORPH_COORD);

        view.set_morph_coords(2.5, -3.0);
        assert_eq!(view.morph_x, MAX_MORPH_COORD);
        assert_eq!(view.morph_y, MIN_MORPH_COORD);
    }

    #[test]
    fn test_turn52_automated_orbit_trajectories() {
        let mut view = NeuralMorphOrbView::new();
        let modes = [
            NeuralOrbitMode::LissajousFigure8,
            NeuralOrbitMode::EllipticalVortex,
            NeuralOrbitMode::BrownianDrift,
            NeuralOrbitMode::EnvelopeReactive,
        ];

        for mode in modes {
            view.orbit_mode = mode;
            view.orbit_speed_hz = 0.50;
            let init_x = view.morph_x;
            let init_y = view.morph_y;

            // Advance by 0.5 seconds
            view.advance_time(0.5);

            assert!(
                (view.morph_x - init_x).abs() > 1e-4 || (view.morph_y - init_y).abs() > 1e-4,
                "Trajectory mode {:?} failed to move morph coordinates",
                mode
            );
            assert!(view.morph_x >= MIN_MORPH_COORD && view.morph_x <= MAX_MORPH_COORD);
            assert!(view.morph_y >= MIN_MORPH_COORD && view.morph_y <= MAX_MORPH_COORD);
        }
    }

    #[test]
    fn test_turn52_touch_target_bounds_wcag_aaa() {
        // WCAG AAA requires touch targets to be at least 44x44pt
        assert!(
            NEURAL_ORB_PUCK_HIT_RADIUS >= 22.0,
            "Puck hit radius must be >= 22pt (diameter >= 44pt)"
        );
    }

    #[test]
    fn test_turn52_deterministic_ascii_snapshot() {
        let view = NeuralMorphOrbView::new();
        let ascii = view.render_snapshot_ascii();

        assert!(!ascii.is_empty());
        assert!(ascii.contains("NEURAL TIMBRE ORB HUD"));
        assert!(ascii.contains("VOCAL-CELLO"));
        assert!(ascii.contains("Morph:"));
        assert!(ascii.contains("Tilt:"));
        assert!(ascii.contains("Centroid:"));
        assert!(ascii.contains("Warmth:"));
        assert!(ascii.contains('@')); // Active puck indicator
    }
}

#[cfg(test)]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::neural_morph_orb_view::NeuralMorphOrbView;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn52_headless_egui_render_without_panic() {
        let ctx = egui::Context::default();
        let mut view = NeuralMorphOrbView::new();

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn52_modal_lifecycle_open_and_close() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_neural_morph_hud_open());

        view.open_neural_morph_hud();
        assert!(view.is_neural_morph_hud_open());

        view.close_neural_morph_hud();
        assert!(!view.is_neural_morph_hud_open());
    }

    #[test]
    fn test_turn52_live_parambus_atomic_slot_sync() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0; // Track 1 (id: 1, slot: 0)
        let mut bus = ParamBus::new();

        let track_id = 1_u32;
        let slot = 0_u32;
        let base_pid = track_id * 1000 + slot * 20;

        for offset in 0..6 {
            bus.register(ParamId(base_pid + offset), 0.0);
        }

        view.live_param_bus = Some(std::sync::Arc::new(bus));
        view.neural_morph_orb_view.set_morph_coords(0.65, -0.45);
        view.sync_neural_morph_hud_state();

        let live_bus = view.live_param_bus.as_ref().unwrap();
        assert!((live_bus.get(ParamId(base_pid)).unwrap() - 0.65).abs() < 1e-3);
        assert!((live_bus.get(ParamId(base_pid + 1)).unwrap() - (-0.45)).abs() < 1e-3);
        assert!(live_bus.get(ParamId(base_pid + 2)).is_some());
        assert!(live_bus.get(ParamId(base_pid + 3)).is_some());
        assert!(live_bus.get(ParamId(base_pid + 4)).is_some());
        assert!(live_bus.get(ParamId(base_pid + 5)).is_some());
    }

    #[test]
    fn test_turn52_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_neural_morph_hud_open());

        view.inspector_state.requested_open_neural_morph_hud = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_neural_morph_hud_open());
        assert!(!view.inspector_state.requested_open_neural_morph_hud);
    }

    #[test]
    fn test_turn52_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let action = palette.actions.iter().find(|a| a.action_id == "open_neural_morph_hud");

        assert!(
            action.is_some(),
            "Command palette must contain open_neural_morph_hud"
        );
        let act = action.unwrap();
        assert_eq!(act.category, "Neural AI & Resynthesis");
        assert!(act.label.contains("Neural Timbre Morphing"));
    }
}
