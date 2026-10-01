#![allow(clippy::all)]

//! Test suite for Turn #61 (Turn #26 Deliverable) — Physical Modeling Waveguide Brass (Trumpet, French Horn, Trombone, Tuba, Flugelhorn, Lip-Reed Bernoulli Embouchure & 3-Valve Acoustic Bore) & Vocal Tract 44-Cylinder Area Function (9 Vowel Formants, Tongue Articulatory Space & Velum Nasalization) HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::waveguide_brass_view::{
        WaveguideBrassView, BrassInstrument, BRASS_PUCK_HIT_RADIUS,
    };
    use crate::views::vocal_tract_view::{
        VocalTractView, VOCAL_PUCK_HIT_RADIUS,
    };
    use summoner_dsp::vocal_tract::VowelPreset;

    #[test]
    fn test_turn61_pure_waveguide_brass_acoustic_physics_and_instruments() {
        let mut view = WaveguideBrassView::new();
        assert_eq!(view.instrument, BrassInstrument::TrumpetBb);
        assert!((view.lip_tension_hz - 233.08).abs() < 1e-2);
        assert!((view.blowing_pressure_kpa - 3.85).abs() < 1e-2);
        assert!((view.bore_length_m - 1.48).abs() < 1e-2);
        assert_eq!(view.valve_state, [false, false, false]);
        assert!(view.acoustic_impedance_score > 0.0 && view.acoustic_impedance_score <= 1.0);
        assert!(view.lip_aperture_mm > 0.0);
        assert!((view.bell_cutoff_hz - 1450.0).abs() < 1e-2);

        // Test instrument profile presets
        view.set_instrument(BrassInstrument::FrenchHornF);
        assert_eq!(view.instrument, BrassInstrument::FrenchHornF);
        assert!((view.bore_length_m - 3.75).abs() < 1e-2);
        assert!((view.bell_cutoff_hz - 820.0).abs() < 1e-2);

        view.set_instrument(BrassInstrument::TromboneBb);
        assert_eq!(view.instrument, BrassInstrument::TromboneBb);
        assert!((view.bore_length_m - 2.75).abs() < 1e-2);
        assert!((view.bell_cutoff_hz - 980.0).abs() < 1e-2);

        view.set_instrument(BrassInstrument::TubaEb);
        assert_eq!(view.instrument, BrassInstrument::TubaEb);
        assert!((view.bore_length_m - 5.40).abs() < 1e-2);
        assert!((view.bell_cutoff_hz - 420.0).abs() < 1e-2);

        view.set_instrument(BrassInstrument::FlugelhornBb);
        assert_eq!(view.instrument, BrassInstrument::FlugelhornBb);
        assert!((view.bore_length_m - 1.52).abs() < 1e-2);
        assert!((view.bell_cutoff_hz - 1180.0).abs() < 1e-2);

        // Test 3-Valve acoustic tube elongation
        view.set_instrument(BrassInstrument::TrumpetBb);
        let base_len = view.bore_length_m;
        view.valve_state[0] = true;
        view.update_physics_simulation();
        assert!(view.bore_length_m > base_len);

        view.valve_state[1] = true;
        view.update_physics_simulation();
        assert!(view.bore_length_m > base_len * 1.15);

        view.valve_state[2] = true;
        view.update_physics_simulation();
        assert!(view.bore_length_m > base_len * 1.30);

        // Normalization roundtrips
        let norm_ten = WaveguideBrassView::tension_to_normalized(440.0);
        let recon_ten = WaveguideBrassView::normalized_to_tension(norm_ten);
        assert!((recon_ten - 440.0).abs() < 1e-2);

        let norm_press = WaveguideBrassView::pressure_to_normalized(4.5);
        let recon_press = WaveguideBrassView::normalized_to_pressure(norm_press);
        assert!((recon_press - 4.5).abs() < 1e-3);

        // Hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let px = canvas.x + view.embouchure_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.embouchure_puck_pos.1) * canvas.height;
        assert!(view.hit_test_embouchure_puck((px, py), canvas));
        assert!(view.hit_test_embouchure_puck((px + BRASS_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_embouchure_puck((px + BRASS_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Bore profile & radiation reflection
        let r0 = view.evaluate_bore_profile(0.0);
        let r_mid = view.evaluate_bore_profile(0.5);
        let r_bell = view.evaluate_bore_profile(1.0);
        assert!(r0 < r_mid);
        assert!(r_mid < r_bell);
        assert!((r_bell - 1.0).abs() < 0.1);

        let refl_low = view.evaluate_radiation_reflection(200.0);
        let refl_high = view.evaluate_radiation_reflection(3000.0);
        assert!(refl_low > refl_high);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(!snap.is_empty());
        assert!(snap.contains('+'));
    }

    #[test]
    fn test_turn61_pure_vocal_tract_cylinders_and_formants() {
        let mut view = VocalTractView::new();
        assert_eq!(view.vowel_preset, VowelPreset::I);
        assert!((view.tongue_position - 0.75).abs() < 1e-2);
        assert!((view.tongue_height - 0.85).abs() < 1e-2);
        assert!((view.glottal_f0_hz - 140.0).abs() < 1e-2);
        assert_eq!(view.cylinder_areas.len(), 44);

        // Test vowel presets
        view.set_vowel_preset(VowelPreset::A);
        assert_eq!(view.vowel_preset, VowelPreset::A);
        assert!((view.tongue_position - 0.30).abs() < 1e-2);
        assert!((view.tongue_height - 0.15).abs() < 1e-2);

        view.set_vowel_preset(VowelPreset::U);
        assert_eq!(view.vowel_preset, VowelPreset::U);
        assert!((view.tongue_position - 0.10).abs() < 1e-2);
        assert!((view.tongue_height - 0.90).abs() < 1e-2);

        view.set_vowel_preset(VowelPreset::E);
        assert_eq!(view.vowel_preset, VowelPreset::E);

        view.set_vowel_preset(VowelPreset::Schwa);
        assert_eq!(view.vowel_preset, VowelPreset::Schwa);

        // Cylinder area properties
        for &area in &view.cylinder_areas {
            assert!(area > 0.0 && area <= 8.0);
        }

        // Hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_puck((px, py), canvas));
        assert!(view.hit_test_puck((px + VOCAL_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_puck((px + VOCAL_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(!snap.is_empty());
        assert!(snap.contains('+'));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use std::sync::Arc;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn61_gui_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Waveguide Brass HUD Lifecycle
        assert!(!view.is_waveguide_brass_hud_open());
        view.open_waveguide_brass_hud();
        assert!(view.is_waveguide_brass_hud_open());
        view.close_waveguide_brass_hud();
        assert!(!view.is_waveguide_brass_hud_open());

        // Vocal Tract HUD Lifecycle
        assert!(!view.is_vocal_tract_hud_open());
        view.open_vocal_tract_hud();
        assert!(view.is_vocal_tract_hud_open());
        view.close_vocal_tract_hud();
        assert!(!view.is_vocal_tract_hud_open());
    }

    #[test]
    fn test_turn61_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_waveguide_brass_hud();
        view.open_vocal_tract_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_waveguide_brass_hud_open());
        assert!(view.is_vocal_tract_hud_open());
    }

    #[test]
    fn test_turn61_gui_live_param_bus_atomic_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..6 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Waveguide Brass sync
        view.waveguide_brass_view.lip_tension_hz = 440.0;
        view.waveguide_brass_view.blowing_pressure_kpa = 5.2;
        view.waveguide_brass_view.bore_length_m = 2.2;
        view.waveguide_brass_view.bell_cutoff_hz = 1200.0;
        view.waveguide_brass_view.lip_aperture_mm = 1.1;
        view.waveguide_brass_view.acoustic_impedance_score = 0.88;

        view.sync_waveguide_brass_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("lip_tension_hz").copied(), Some(440.0));
        assert_eq!(view.device_rack_state.node_param_values.get("blowing_pressure_kpa").copied(), Some(5.2));
        assert_eq!(view.inspector_state.node_param_values.get("lip_tension_hz").copied(), Some(440.0));
        assert_eq!(view.inspector_state.node_param_values.get("blowing_pressure_kpa").copied(), Some(5.2));

        assert_eq!(arc_bus.get(ParamId(base)), Some(440.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(5.2));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(2.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(1200.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(1.1));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.88));

        // Test Vocal Tract sync
        view.vocal_tract_view.tongue_position = 0.60;
        view.vocal_tract_view.tongue_height = 0.70;
        view.vocal_tract_view.lip_opening = 1.4;
        view.vocal_tract_view.velum_opening = 0.25;
        view.vocal_tract_view.glottal_f0_hz = 220.0;
        view.vocal_tract_view.aspiration_level = 0.08;

        view.sync_vocal_tract_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("tongue_position").copied(), Some(0.60));
        assert_eq!(view.device_rack_state.node_param_values.get("tongue_height").copied(), Some(0.70));
        assert_eq!(view.inspector_state.node_param_values.get("tongue_position").copied(), Some(0.60));
        assert_eq!(view.inspector_state.node_param_values.get("tongue_height").copied(), Some(0.70));

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.60));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.70));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(1.4));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.25));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(220.0));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.08));
    }

    #[test]
    fn test_turn61_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger brass from inspector
        view.inspector_state.requested_open_waveguide_brass_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_waveguide_brass_hud_open());
        assert!(!view.inspector_state.requested_open_waveguide_brass_hud);
        view.close_waveguide_brass_hud();

        // Trigger vocal tract from rack
        view.device_rack_state.requested_open_vocal_tract_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_vocal_tract_hud_open());
        assert!(!view.device_rack_state.requested_open_vocal_tract_hud);
    }

    #[test]
    fn test_turn61_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let brass_action = palette.actions.iter().find(|a| a.action_id == "open_waveguide_brass_hud");
        assert!(brass_action.is_some());
        let brass = brass_action.unwrap();
        assert_eq!(brass.category, "Physical Modeling");
        assert!(brass.label.contains("Waveguide Brass"));

        let vocal_action = palette.actions.iter().find(|a| a.action_id == "open_vocal_tract_hud");
        assert!(vocal_action.is_some());
        let vocal = vocal_action.unwrap();
        assert_eq!(vocal.category, "Physical Modeling");
        assert!(vocal.label.contains("Vocal Tract"));
    }
}
