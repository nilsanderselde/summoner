#![allow(clippy::all)]

//! Test suite for Turn #122 (Sprint Turn #48) — Chromatic Autocorrelation Pitch Tuner & Strobe Phase Calibration HUD
//! and GM Multi-Pad Drum Machine & Per-Pad ADSR/VCA Synthesis HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::chromatic_tuner_view::{
        ChromaticTunerPreset, ChromaticTunerView, CHROMATIC_TUNER_PUCK_HIT_RADIUS,
    };
    use crate::views::drum_machine_view::{
        DrumMachinePadView, DrumMachinePreset, DrumPadSlotConfig, DRUM_MACHINE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn122_pure_chromatic_tuner_initialization_and_physics() {
        let mut view = ChromaticTunerView::new();
        assert_eq!(view.preset, ChromaticTunerPreset::ConcertStandard440Hz);
        assert_eq!(CHROMATIC_TUNER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                ChromaticTunerPreset::ConcertStandard440Hz,
                "ISO Concert Standard (440.0 Hz)",
                440.0,
                2.0,
                0.70,
                1.00,
            ),
            (
                ChromaticTunerPreset::BaroqueChamber415Hz,
                "Baroque Chamber (415.0 Hz)",
                415.0,
                3.0,
                0.65,
                0.85,
            ),
            (
                ChromaticTunerPreset::OrchestralBright442Hz,
                "Orchestral Symphonic (442.0 Hz)",
                442.0,
                1.5,
                0.75,
                1.15,
            ),
            (
                ChromaticTunerPreset::VerdiNatural432Hz,
                "Verdi Scientific (432.0 Hz)",
                432.0,
                2.5,
                0.68,
                0.95,
            ),
        ];

        for (p, exp_name, exp_a4, exp_tol, exp_thresh, exp_speed) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (a4, tol, thresh, speed) = p.nominal_parameters();
            assert_eq!(a4, exp_a4);
            assert_eq!(tol, exp_tol);
            assert_eq!(thresh, exp_thresh);
            assert_eq!(speed, exp_speed);

            assert_eq!(view.reference_a4_hz, exp_a4);
            assert_eq!(view.cents_tolerance, exp_tol);
            assert_eq!(view.autocorr_threshold, exp_thresh);
            assert_eq!(view.strobe_speed, exp_speed);
        }

        // Reference A4 normalization round-trip
        for a4 in [410.0, 415.0, 432.0, 440.0, 442.0, 450.0] {
            let norm = ChromaticTunerView::reference_a4_to_normalized(a4);
            assert!((0.0..=1.0).contains(&norm));
            let back = ChromaticTunerView::normalized_to_reference_a4(norm);
            assert!((back - a4).abs() < 1e-2);
        }

        // Cents normalization round-trip
        for c in [-50.0, -25.0, 0.0, 15.0, 50.0] {
            let norm = ChromaticTunerView::cents_to_normalized(c);
            assert!((0.0..=1.0).contains(&norm));
            let back = ChromaticTunerView::normalized_to_cents(norm);
            assert!((back - c).abs() < 1e-2);
        }

        // Autocorrelation pitch detection on synthetic A4 (440 Hz) sine wave
        view.set_preset(ChromaticTunerPreset::ConcertStandard440Hz);
        let sr = 44100.0;
        let mut buf = vec![0.0f32; 2048];
        for (i, sample) in buf.iter_mut().enumerate() {
            *sample = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sr).sin();
        }
        assert!(view.detect_pitch_from_buffer(&buf, sr));
        assert_eq!(view.detected_midi_note, 69);
        assert_eq!(view.detected_note_name, "A4");
        assert!((view.detected_pitch_hz - 440.0).abs() < 5.0);

        // Lock tolerance evaluation
        view.cents_deviation = 1.0;
        view.cents_tolerance = 2.0;
        assert!(view.is_in_tune_locked());
        view.cents_deviation = 12.5;
        assert!(!view.is_in_tune_locked());

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("CHROMATIC AUTOCORRELATION TUNER & STROBE HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn122_pure_drum_machine_initialization_and_physics() {
        let mut view = DrumMachinePadView::new();
        assert_eq!(view.preset, DrumMachinePreset::Analog808SubKit);
        assert_eq!(DRUM_MACHINE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                DrumMachinePreset::Analog808SubKit,
                "Analog 808 Sub Kit (Deep Decay)",
                0.90,
                1.35,
                -2.0,
                2.0,
                0.85,
            ),
            (
                DrumMachinePreset::AcousticStudioKit,
                "Acoustic Studio Maple Kit (Natural)",
                0.85,
                1.00,
                0.0,
                4.5,
                0.95,
            ),
            (
                DrumMachinePreset::CyberpunkIndustrialKit,
                "Cyberpunk Industrial Kit (+3.5 st)",
                0.95,
                0.75,
                3.5,
                1.0,
                0.70,
            ),
            (
                DrumMachinePreset::LoFiVinylDustKit,
                "Lo-Fi Vinyl Boom-Bap Kit (-3.0 st)",
                0.82,
                0.65,
                -3.0,
                6.0,
                0.90,
            ),
        ];

        for (p, exp_name, exp_master, exp_decay, exp_pitch, exp_attack, exp_vel) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (master, decay, pitch, attack, vel) = p.nominal_parameters();
            assert_eq!(master, exp_master);
            assert_eq!(decay, exp_decay);
            assert_eq!(pitch, exp_pitch);
            assert_eq!(attack, exp_attack);
            assert_eq!(vel, exp_vel);

            assert_eq!(view.master_gain, exp_master);
            assert_eq!(view.decay_scale, exp_decay);
            assert_eq!(view.pitch_semitones, exp_pitch);
            assert_eq!(view.punch_attack_ms, exp_attack);
            assert_eq!(view.velocity_sensitivity, exp_vel);
        }

        // Decay & Pitch normalization round-trip
        for d in [0.20, 0.75, 1.00, 1.50, 2.50] {
            let norm = DrumMachinePadView::decay_to_normalized(d);
            assert!((0.0..=1.0).contains(&norm));
            let back = DrumMachinePadView::normalized_to_decay(norm);
            assert!((back - d).abs() < 1e-2);
        }
        for p in [-12.0, -3.0, 0.0, 4.0, 12.0] {
            let norm = DrumMachinePadView::pitch_to_normalized(p);
            assert!((0.0..=1.0).contains(&norm));
            let back = DrumMachinePadView::normalized_to_pitch(norm);
            assert!((back - p).abs() < 1e-2);
        }

        // Pitch ratio math
        view.pitch_semitones = 0.0;
        assert!((view.pitch_ratio() - 1.0).abs() < 1e-5);
        view.pitch_semitones = 12.0;
        assert!((view.pitch_ratio() - 2.0).abs() < 1e-5);
        view.pitch_semitones = -12.0;
        assert!((view.pitch_ratio() - 0.5).abs() < 1e-5);

        // Pad effective amplitude & mute
        let mut pad = DrumPadSlotConfig::new("Kick", 36, 0.90, 0.50);
        pad.last_velocity = 127;
        assert!((pad.effective_amplitude(1.0) - 0.90).abs() < 1e-3);
        pad.muted = true;
        assert_eq!(pad.effective_amplitude(1.0), 0.0);

        // Envelope evaluation bounds
        for t in [0.0, 0.02, 0.25, 0.50, 1.0] {
            let env = view.evaluate_pad_envelope(t);
            assert!((0.0..=1.0).contains(&env));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("GM DRUM MACHINE & PER-PAD ADSR/VCA HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::chromatic_tuner_view::ChromaticTunerView;
    use crate::views::drum_machine_view::DrumMachinePadView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn122_headless_render_views() {
        let ctx = egui::Context::default();
        let mut ct_view = ChromaticTunerView::new();
        let mut dm_view = DrumMachinePadView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ct_view.show(ui);
                dm_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn122_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test Chromatic Tuner HUD Lifecycle
        assert!(!aw_view.is_chromatic_tuner_hud_open());
        aw_view.open_chromatic_tuner_hud();
        assert!(aw_view.is_chromatic_tuner_hud_open());
        aw_view.sync_chromatic_tuner_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("reference_a4_hz"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("cents_deviation"));
        aw_view.close_chromatic_tuner_hud();
        assert!(!aw_view.is_chromatic_tuner_hud_open());

        // Test Drum Machine HUD Lifecycle
        assert!(!aw_view.is_drum_machine_hud_open());
        aw_view.open_drum_machine_hud();
        assert!(aw_view.is_drum_machine_hud_open());
        aw_view.sync_drum_machine_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("decay_scale"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("pitch_semitones"));
        aw_view.close_drum_machine_hud();
        assert!(!aw_view.is_drum_machine_hud_open());
    }

    #[test]
    fn test_turn122_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_tuner = actions
            .iter()
            .any(|a| a.action_id == "open_chromatic_tuner_hud");
        let has_drums = actions
            .iter()
            .any(|a| a.action_id == "open_drum_machine_hud");

        assert!(
            has_tuner,
            "open_chromatic_tuner_hud must be registered in command palette"
        );
        assert!(
            has_drums,
            "open_drum_machine_hud must be registered in command palette"
        );
    }
}
