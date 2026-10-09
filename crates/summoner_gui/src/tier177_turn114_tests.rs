#![allow(clippy::all)]

//! Test suite for Turn #114 (Sprint Turn #40) — Acoustic Waveguide Brass Horn Flare
//! Reflection & Webster Horn Impedance HUD and Acoustic Glottal Pulse Flow & Vocal
//! Fold Vibration Phonation HUD Convergence (Milestones 15, 28, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::glottal_pulse_view::{
        GlottalPulseView, GlottalVoicingMode, GLOTTAL_PUCK_HIT_RADIUS,
    };
    use crate::views::horn_reflection_view::{
        HornReflectionView, HornFlarePreset, HornMuteType, HORN_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn114_pure_glottal_pulse_initialization_and_kinematics() {
        let mut view = GlottalPulseView::new();
        assert_eq!(view.mode, GlottalVoicingMode::Modal);
        assert_eq!(GLOTTAL_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Voicing modes and expected values
        let modes = [
            (GlottalVoicingMode::Modal, "Modal Phonation (Chest / Standard Voice)", 0.60, 2.0, 0.04, 0.015, 0.03),
            (GlottalVoicingMode::Falsetto, "Falsetto Phonation (Head / Thin Closure)", 0.75, 1.5, 0.08, 0.010, 0.02),
            (GlottalVoicingMode::Creaky, "Creaky Phonation (Vocal Fry / Explosive Pulses)", 0.35, 3.5, 0.02, 0.050, 0.10),
            (GlottalVoicingMode::Breathy, "Breathy Phonation (Incomplete Closure)", 0.85, 1.2, 0.35, 0.025, 0.06),
            (GlottalVoicingMode::Whisper, "Whispered Phonation (Unvoiced Pure Aspiration)", 0.95, 1.0, 0.85, 0.060, 0.12),
        ];

        for (m, expected_name, exp_oq, exp_sq, exp_asp, exp_jit, exp_shim) in modes {
            view.set_mode(m);
            assert_eq!(m.name(), expected_name);
            let (oq, sq, asp, jit, shim) = m.nominal_parameters();
            assert_eq!(oq, exp_oq);
            assert_eq!(sq, exp_sq);
            assert_eq!(asp, exp_asp);
            assert_eq!(jit, exp_jit);
            assert_eq!(shim, exp_shim);
            assert_eq!(view.open_quotient, exp_oq);
            assert_eq!(view.speed_quotient, exp_sq);
            assert_eq!(view.aspiration_level, exp_asp);
            assert_eq!(view.jitter, exp_jit);
            assert_eq!(view.shimmer, exp_shim);
        }

        // Normalization round-trips for Open Quotient [0.20 ..= 0.95]
        let oq_test = 0.58;
        let oq_norm = GlottalPulseView::open_quotient_to_normalized(oq_test);
        let oq_back = GlottalPulseView::normalized_to_open_quotient(oq_norm);
        assert!((oq_back - oq_test).abs() < 1e-4);

        // Normalization round-trips for Speed Quotient [0.80 ..= 4.00]
        let sq_test = 2.45;
        let sq_norm = GlottalPulseView::speed_quotient_to_normalized(sq_test);
        let sq_back = GlottalPulseView::normalized_to_speed_quotient(sq_norm);
        assert!((sq_back - sq_test).abs() < 1e-4);

        // Waveform physiological evaluation across glottal cycle
        let ug_open = view.evaluate_volume_velocity(0.3);
        let ug_closed = view.evaluate_volume_velocity(0.98);
        assert!(ug_open > 0.0);
        assert_eq!(ug_closed, 0.0);

        let dug_open = view.evaluate_flow_derivative(0.2);
        assert!(dug_open.is_finite());

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_glottal_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_glottal_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_glottal_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_glottal_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PHYSICAL MODELING ROSENBERG & LF GLOTTAL FLOW"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn114_pure_horn_reflection_initialization_and_acoustics() {
        let mut view = HornReflectionView::new();
        assert_eq!(view.preset, HornFlarePreset::Trumpet);
        assert_eq!(view.mute_type, HornMuteType::Open);
        assert_eq!(HORN_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Preset profiles and expected values
        let presets = [
            (HornFlarePreset::Trumpet, "Bb Trumpet Bell Flare (~1200 Hz Cutoff)", 1200.0, 0.72, 0.065, 0.985),
            (HornFlarePreset::FrenchHorn, "F French Horn Hyperbolic Bell (~700 Hz Cutoff)", 700.0, 0.50, 0.155, 0.975),
            (HornFlarePreset::Trombone, "Bb Tenor Trombone Bessel Flare (~850 Hz Cutoff)", 850.0, 0.62, 0.110, 0.980),
            (HornFlarePreset::Tuba, "Eb Concert Tuba Massive Bell (~400 Hz Cutoff)", 400.0, 0.45, 0.250, 0.965),
            (HornFlarePreset::Custom, "Custom User Flared Bell Geometry", 1000.0, 0.60, 0.100, 0.980),
        ];

        for (p, expected_name, exp_fc, exp_flare, exp_rbell, exp_damp) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), expected_name);
            let (fc, flare, r_bell, damp) = p.nominal_parameters();
            assert_eq!(fc, exp_fc);
            assert_eq!(flare, exp_flare);
            assert_eq!(r_bell, exp_rbell);
            assert_eq!(damp, exp_damp);
            assert_eq!(view.cutoff_hz, exp_fc);
            assert_eq!(view.flare_coefficient, exp_flare);
            assert_eq!(view.bell_radius_m, exp_rbell);
            assert_eq!(view.reflection_damping, exp_damp);
        }

        // Mute types
        let mutes = [
            (HornMuteType::Open, "Open Bell (Unmuted Free Radiation)"),
            (HornMuteType::Straight, "Straight Mute (Nasal Resonance & Low Cut)"),
            (HornMuteType::Harmon, "Harmon Mute (Stem-Out Metallic Buzz & Notch)"),
            (HornMuteType::Cup, "Cup Mute (Mellow Rolloff & Acoustic Damping)"),
        ];

        for (m, expected_name) in mutes {
            view.set_mute(m);
            assert_eq!(m.name(), expected_name);
            assert_eq!(view.mute_type, m);
        }

        // Normalization round-trips for Cutoff [200.0 ..= 3000.0 Hz] (log-scale)
        let fc_test = 880.0;
        let fc_norm = HornReflectionView::cutoff_to_normalized(fc_test);
        let fc_back = HornReflectionView::normalized_to_cutoff(fc_norm);
        assert!((fc_back - fc_test).abs() < 1e-2);

        // Normalization round-trips for Flare [0.20 ..= 2.00]
        let flare_test = 1.35;
        let flare_norm = HornReflectionView::flare_to_normalized(flare_test);
        let flare_back = HornReflectionView::normalized_to_flare(flare_norm);
        assert!((flare_back - flare_test).abs() < 1e-4);

        // Acoustic transmission and reflection evaluation
        let t_sub = view.evaluate_radiation_response_db(150.0);
        let t_pass = view.evaluate_radiation_response_db(2500.0);
        assert!(t_sub < t_pass); // High frequencies radiate more efficiently

        let r_sub = view.evaluate_reflection_magnitude(150.0);
        let r_pass = view.evaluate_reflection_magnitude(2500.0);
        assert!(r_sub > r_pass); // Low frequencies reflect back into the waveguide

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_horn_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_horn_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_horn_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_horn_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PHYSICAL MODELING ACOUSTIC HORN BELL FLARE"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn114_gui_horn_and_glottal_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Horn Reflection Modal Lifecycle
        assert!(!view.is_horn_reflection_hud_open());
        view.open_horn_reflection_hud();
        assert!(view.is_horn_reflection_hud_open());
        view.close_horn_reflection_hud();
        assert!(!view.is_horn_reflection_hud_open());

        // 2. Glottal Pulse Modal Lifecycle
        assert!(!view.is_glottal_pulse_hud_open());
        view.open_glottal_pulse_hud();
        assert!(view.is_glottal_pulse_hud_open());
        view.close_glottal_pulse_hud();
        assert!(!view.is_glottal_pulse_hud_open());

        // 3. Horn Reflection Parameter Sync Bridge
        view.horn_reflection_view.cutoff_hz = 950.0;
        view.horn_reflection_view.flare_coefficient = 1.15;
        view.horn_reflection_view.bell_radius_m = 0.14;
        view.sync_horn_reflection_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("cutoff_hz").copied(),
            Some(950.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("flare_coefficient").copied(),
            Some(1.15)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("cutoff_hz").copied(),
            Some(950.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("flare_coefficient").copied(),
            Some(1.15)
        );

        // 4. Glottal Pulse Parameter Sync Bridge
        view.glottal_pulse_view.open_quotient = 0.55;
        view.glottal_pulse_view.speed_quotient = 2.80;
        view.glottal_pulse_view.aspiration_level = 0.12;
        view.sync_glottal_pulse_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("open_quotient").copied(),
            Some(0.55)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("speed_quotient").copied(),
            Some(2.80)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("open_quotient").copied(),
            Some(0.55)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("speed_quotient").copied(),
            Some(2.80)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_horn_reflection_modal_window(ui);
                view.show_glottal_pulse_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let horn_action = palette.actions.iter().find(|a| a.action_id == "open_horn_reflection_hud");
        assert!(horn_action.is_some(), "open_horn_reflection_hud should be registered");
        assert_eq!(horn_action.unwrap().category, "Physical Modeling");

        let glottal_action = palette.actions.iter().find(|a| a.action_id == "open_glottal_pulse_hud");
        assert!(glottal_action.is_some(), "open_glottal_pulse_hud should be registered");
        assert_eq!(glottal_action.unwrap().category, "Physical Modeling");
    }
}
