#![allow(clippy::all)]

//! Test suite for Turn #121 (Sprint Turn #47) — Physical Modeling Air-Jet Reed & Utaguchi Flue Vortex Aerodynamics HUD
//! and Neural ONNX 4-Stem Spectral Mask Decomposition & Multi-Track Router HUD Convergence (Milestones 25, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::air_reed_view::{
        AirReedPreset, AirReedView, AIR_REED_PUCK_HIT_RADIUS,
    };
    use crate::views::stem_separator_view::{
        StemChannelConfig, StemSeparatorPreset, StemSeparatorView,
        STEM_SEPARATOR_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn121_pure_air_reed_initialization_and_physics() {
        let mut view = AirReedView::new();
        assert_eq!(view.preset, AirReedPreset::HonkyokuRoMeditation);
        assert_eq!(AIR_REED_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (AirReedPreset::HonkyokuRoMeditation, "Honkyoku Ro Meditation (800 Pa)", 800.0, 0.0, 10.0, 1.45, 0.15, 0.00),
            (AirReedPreset::DeepMeriShadowBend, "Deep Meri Shadow Bend (-200c)", 520.0, -200.0, 14.0, 1.15, 0.28, 0.10),
            (AirReedPreset::KariOverblownKan, "Kari Overblown Kan (2200 Pa)", 2200.0, 120.0, 6.5, 2.10, 0.12, 0.25),
            (AirReedPreset::MuraiIkiExplosiveBurst, "Murai-Iki Explosive Burst (3600 Pa)", 3600.0, -45.0, 8.0, 2.65, 0.45, 0.95),
        ];

        for (p, exp_name, exp_press, exp_cents, exp_dist, exp_sharp, exp_turb, exp_murai) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (press, cents, dist, sharp, turb, murai) = p.nominal_parameters();
            assert_eq!(press, exp_press);
            assert_eq!(cents, exp_cents);
            assert_eq!(dist, exp_dist);
            assert_eq!(sharp, exp_sharp);
            assert_eq!(turb, exp_turb);
            assert_eq!(murai, exp_murai);

            assert_eq!(view.blowing_pressure_pa, exp_press);
            assert_eq!(view.meri_kari_cents, exp_cents);
            assert_eq!(view.jet_distance_mm, exp_dist);
            assert_eq!(view.edge_sharpness, exp_sharp);
            assert_eq!(view.turbulence_gain, exp_turb);
            assert_eq!(view.murai_iki_intensity, exp_murai);
        }

        // Pressure normalization round-trip
        for p in [50.0, 500.0, 800.0, 2000.0, 4000.0] {
            let norm = AirReedView::pressure_to_normalized(p);
            assert!(norm >= 0.0 && norm <= 1.0);
            let back = AirReedView::normalized_to_pressure(norm);
            assert!((back - p).abs() < 1e-2);
        }

        // Meri/Kari normalization round-trip
        for c in [-300.0, -150.0, 0.0, 100.0, 200.0] {
            let norm = AirReedView::meri_kari_to_normalized(c);
            assert!(norm >= 0.0 && norm <= 1.0);
            let back = AirReedView::normalized_to_meri_kari(norm);
            assert!((back - c).abs() < 1e-2);
        }

        // Bernoulli jet velocity matching summoner_dsp::air_reed
        view.blowing_pressure_pa = 800.0;
        assert!((view.jet_velocity_mps() - 36.45).abs() < 1.0);

        // Pitch multiplier matching 2^(cents/1200)
        view.meri_kari_cents = 0.0;
        assert!((view.pitch_multiplier() - 1.0).abs() < 1e-5);
        view.meri_kari_cents = -300.0;
        assert!((view.pitch_multiplier() - 0.84089).abs() < 1e-3);
        view.meri_kari_cents = 200.0;
        assert!((view.pitch_multiplier() - 1.12246).abs() < 1e-3);

        // Utaguchi splitting function bounds
        for disp in [-3.0, -1.0, 0.0, 1.0, 3.0] {
            let split = view.evaluate_utaguchi_splitting(disp);
            assert!((-1.0..=1.0).contains(&split));
        }

        // Jet delay samples
        let delay = view.calculate_jet_delay_samples();
        assert!(delay >= 1.0 && delay <= 4094.0);

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("AIR-JET REED & UTAGUCHI FLUE VORTEX HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn121_pure_stem_separator_initialization_and_physics() {
        let mut view = StemSeparatorView::new();
        assert_eq!(view.preset, StemSeparatorPreset::BalancedFull4Stem);
        assert_eq!(STEM_SEPARATOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (StemSeparatorPreset::BalancedFull4Stem, "Balanced Full 4-Stem (0 dB Flat)", [0.0, 0.0, 0.0, 0.0], 0.50, 0.75),
            (StemSeparatorPreset::AcapellaMelodyFocus, "Acapella & Melody Focus (+3 dB Lead)", [-18.0, -18.0, 3.0, -12.0], 0.68, 0.92),
            (StemSeparatorPreset::DrumAndBassFoundation, "Drum & Bass Foundation (Rhythm Solo)", [2.0, 2.5, -24.0, -20.0], 0.32, 0.85),
            (StemSeparatorPreset::InstrumentalBackingKaraoke, "Instrumental Backing (-36 dB Lead)", [0.5, 0.5, -36.0, 1.0], 0.45, 0.88),
        ];

        for (p, exp_name, exp_gains, exp_tilt, exp_sharp) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (gains, tilt, sharp) = p.nominal_parameters();
            assert_eq!(gains, exp_gains);
            assert_eq!(tilt, exp_tilt);
            assert_eq!(sharp, exp_sharp);

            assert_eq!(view.spectral_tilt, exp_tilt);
            assert_eq!(view.mask_sharpness, exp_sharp);
            for i in 0..4 {
                assert_eq!(view.stems[i].gain_db, exp_gains[i]);
            }
        }

        // Puck update round-trip
        view.update_from_puck(0.35, 0.82);
        assert!((view.spectral_tilt - 0.35).abs() < 1e-4);
        assert!((view.mask_sharpness - 0.82).abs() < 1e-4);

        // Stem linear gain & mute evaluation
        let mut ch = StemChannelConfig::new("drums", 6.0, 0);
        assert!((ch.linear_gain() - 1.995).abs() < 0.01);
        ch.is_muted = true;
        assert_eq!(ch.linear_gain(), 0.0);

        // Spectral mask evaluation across frequency bins
        view.set_preset(StemSeparatorPreset::BalancedFull4Stem);
        let low_masks = view.evaluate_stem_spectral_masks(0.05);
        let mid_masks = view.evaluate_stem_spectral_masks(0.50);
        let high_masks = view.evaluate_stem_spectral_masks(0.95);
        assert!(low_masks[1] > low_masks[2]); // Bass > Melody at very low bin
        assert!(mid_masks[2] > mid_masks[1]); // Melody > Bass at mid bin
        assert!(high_masks[0] > high_masks[1]); // Drums transient > Bass at high bin

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("NEURAL ONNX 4-STEM SEPARATOR & ROUTER HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::air_reed_view::AirReedView;
    use crate::views::stem_separator_view::StemSeparatorView;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn121_headless_render_views() {
        let ctx = egui::Context::default();
        let mut ar_view = AirReedView::new();
        let mut ss_view = StemSeparatorView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ar_view.show(ui);
                ss_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn121_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test Air-Reed HUD Lifecycle
        assert!(!aw_view.is_air_reed_hud_open());
        aw_view.open_air_reed_hud();
        assert!(aw_view.is_air_reed_hud_open());
        aw_view.sync_air_reed_hud_state();
        assert!(aw_view.device_rack_state.node_param_values.contains_key("blowing_pressure_pa"));
        assert!(aw_view.inspector_state.node_param_values.contains_key("meri_kari_cents"));
        aw_view.close_air_reed_hud();
        assert!(!aw_view.is_air_reed_hud_open());

        // Test Stem Separator HUD Lifecycle
        assert!(!aw_view.is_stem_separator_hud_open());
        aw_view.open_stem_separator_hud();
        assert!(aw_view.is_stem_separator_hud_open());
        aw_view.sync_stem_separator_hud_state();
        assert!(aw_view.device_rack_state.node_param_values.contains_key("spectral_tilt"));
        assert!(aw_view.inspector_state.node_param_values.contains_key("mask_sharpness"));
        aw_view.close_stem_separator_hud();
        assert!(!aw_view.is_stem_separator_hud_open());
    }

    #[test]
    fn test_turn121_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_air_reed = actions.iter().any(|a| a.action_id == "open_air_reed_hud");
        let has_stem_sep = actions.iter().any(|a| a.action_id == "open_stem_separator_hud");

        assert!(has_air_reed, "open_air_reed_hud must be registered in command palette");
        assert!(has_stem_sep, "open_stem_separator_hud must be registered in command palette");
    }
}
