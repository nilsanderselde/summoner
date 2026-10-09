#![allow(clippy::all)]

//! Test suite for Turn #120 (Sprint Turn #46) — Analog Diode-Ring Modulator & Quadrature Frequency Doubler HUD
//! and Psychoacoustic Shaped Noise Dither & Bit-Depth Truncation Radar HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::ring_mod_view::{
        RingModPreset, RingModulatorView, RingWaveformShape,
        RING_MOD_PUCK_HIT_RADIUS,
    };
    use crate::views::dither_quantizer_view::{
        DitherMode, DitherPreset, DitherQuantizerView,
        DITHER_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn120_pure_ring_modulator_initialization_and_physics() {
        let mut view = RingModulatorView::new();
        assert_eq!(view.preset, RingModPreset::DalekRoboticVoice);
        assert_eq!(RING_MOD_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (RingModPreset::DalekRoboticVoice, "Dalek Robotic Voice (30Hz Sine)", 30.0, RingWaveformShape::Sine, 1.00, 0.20, 0.0),
            (RingModPreset::ChimeBellHarmonics, "Chime Bell Harmonics (440Hz Tri)", 440.0, RingWaveformShape::Triangle, 0.85, 0.10, 45.0),
            (RingModPreset::TremoloWarmSub, "Tremolo Warm Sub (5.5Hz Sine)", 5.5, RingWaveformShape::Sine, 0.35, 0.00, 0.0),
            (RingModPreset::AlienQuadratureGlitch, "Alien Quadrature Glitch (1850Hz Sqr)", 1850.0, RingWaveformShape::Square, 1.00, 0.65, 90.0),
        ];

        for (p, exp_name, exp_freq, exp_wave, exp_depth, exp_asym, exp_phase) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (freq, wave, depth, asym, phase) = p.nominal_parameters();
            assert_eq!(freq, exp_freq);
            assert_eq!(wave, exp_wave);
            assert_eq!(depth, exp_depth);
            assert_eq!(asym, exp_asym);
            assert_eq!(phase, exp_phase);

            assert_eq!(view.carrier_freq_hz, exp_freq);
            assert_eq!(view.carrier_waveform, exp_wave);
            assert_eq!(view.mod_depth, exp_depth);
            assert_eq!(view.diode_asymmetry, exp_asym);
            assert_eq!(view.phase_deg, exp_phase);
        }

        // Frequency mapping normalization round-trip
        let test_freqs = [0.1, 1.0, 30.0, 440.0, 1000.0, 8000.0];
        for f in test_freqs {
            let norm = RingModulatorView::freq_to_normalized(f);
            assert!(norm >= 0.0 && norm <= 1.0);
            let back = RingModulatorView::normalized_to_freq(norm);
            assert!((back - f).abs() / f < 0.01, "Freq round-trip mismatch: {} vs {}", back, f);
        }

        // Depth normalization round-trip
        let norm_d = RingModulatorView::depth_to_normalized(0.65);
        let back_d = RingModulatorView::normalized_to_depth(norm_d);
        assert!((back_d - 0.65).abs() < 1e-4);

        // Carrier Waveform Evaluations across all shapes
        view.diode_asymmetry = 0.0;
        view.phase_deg = 0.0;

        view.carrier_waveform = RingWaveformShape::Sine;
        assert!((view.evaluate_carrier_waveform(0.0)).abs() < 1e-4);
        assert!((view.evaluate_carrier_waveform(0.25) - 1.0).abs() < 1e-4);
        assert!((view.evaluate_carrier_waveform(0.75) + 1.0).abs() < 1e-4);

        view.carrier_waveform = RingWaveformShape::Triangle;
        assert!((view.evaluate_carrier_waveform(0.0) - 1.0).abs() < 1e-4);
        assert!((view.evaluate_carrier_waveform(0.5) + 1.0).abs() < 1e-4);

        view.carrier_waveform = RingWaveformShape::Square;
        assert_eq!(view.evaluate_carrier_waveform(0.2), 1.0);
        assert_eq!(view.evaluate_carrier_waveform(0.8), -1.0);

        // Sideband calculation
        view.carrier_freq_hz = 500.0;
        let (upper, lower) = view.calculate_sidebands(100.0);
        assert_eq!(upper, 600.0);
        assert_eq!(lower, 400.0);

        // Advance running phase
        view.carrier_freq_hz = 100.0;
        view.advance_time(0.005);
        assert!((view.phase_accum - 0.5).abs() < 1e-4);

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("ANALOG DIODE-RING MODULATOR HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn120_pure_dither_quantizer_initialization_and_physics() {
        let mut view = DitherQuantizerView::new();
        assert_eq!(view.preset, DitherPreset::RedBookCd16BitTpdf);
        assert_eq!(DITHER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (DitherPreset::RedBookCd16BitTpdf, "Red Book CD 16-Bit TPDF (-98 dB)", 16, DitherMode::Tpdf, 1.0, 12345),
            (DitherPreset::PsychoacousticShaped24Bit, "Psychoacoustic 24-Bit Shaped (-146 dB)", 24, DitherMode::NoiseShaped, 1.5, 54321),
            (DitherPreset::VintageSampler12Bit, "Vintage Sampler 12-Bit Crunch (-74 dB)", 12, DitherMode::NoiseShaped, 0.75, 9999),
            (DitherPreset::LofiChiptune8BitRaw, "Lo-Fi Chiptune 8-Bit Raw (-50 dB)", 8, DitherMode::None, 0.0, 0),
        ];

        for (p, exp_name, exp_depth, exp_mode, exp_shaping, exp_seed) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (depth, mode, shaping, seed) = p.nominal_parameters();
            assert_eq!(depth, exp_depth);
            assert_eq!(mode, exp_mode);
            assert_eq!(shaping, exp_shaping);
            assert_eq!(seed, exp_seed);

            assert_eq!(view.bit_depth, exp_depth);
            assert_eq!(view.dither_mode, exp_mode);
            assert_eq!(view.shaping_intensity, exp_shaping);
            assert_eq!(view.prng_seed, exp_seed);
        }

        // Bit depth normalization round-trip
        let depths = [8, 12, 16, 20, 24];
        for d in depths {
            let norm = DitherQuantizerView::bit_depth_to_normalized(d);
            assert!(norm >= 0.0 && norm <= 1.0);
            let back = DitherQuantizerView::normalized_to_bit_depth(norm);
            assert_eq!(back, d);
        }

        // Shaping normalization round-trip
        let norm_s = DitherQuantizerView::shaping_to_normalized(1.25);
        let back_s = DitherQuantizerView::normalized_to_shaping(norm_s);
        assert!((back_s - 1.25).abs() < 1e-4);

        // Theoretical SNR and Quantization levels
        view.bit_depth = 16;
        assert!((view.theoretical_snr_db() - 98.08).abs() < 0.1);
        assert_eq!(view.quantization_steps(), 65536);

        view.bit_depth = 24;
        assert!((view.theoretical_snr_db() - 146.24).abs() < 0.1);
        assert_eq!(view.quantization_steps(), 16777216);

        view.bit_depth = 8;
        assert!((view.theoretical_snr_db() - 49.92).abs() < 0.1);
        assert_eq!(view.quantization_steps(), 256);

        // Dither sample evaluation
        view.bit_depth = 16;
        view.dither_mode = DitherMode::Tpdf;
        let dithered = view.quantize_sample(0.5);
        assert!(dithered >= -1.0 && dithered <= 1.0);

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("PSYCHOACOUSTIC SHAPED DITHER HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::ring_mod_view::RingModulatorView;
    use crate::views::dither_quantizer_view::DitherQuantizerView;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn120_headless_render_views() {
        let ctx = egui::Context::default();
        let mut rm_view = RingModulatorView::new();
        let mut dq_view = DitherQuantizerView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rm_view.show(ui);
                dq_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn120_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test Ring Mod HUD Lifecycle
        assert!(!aw_view.is_ring_mod_hud_open());
        aw_view.open_ring_mod_hud();
        assert!(aw_view.is_ring_mod_hud_open());
        aw_view.sync_ring_mod_hud_state();
        assert!(aw_view.device_rack_state.node_param_values.contains_key("carrier_freq_hz"));
        assert!(aw_view.inspector_state.node_param_values.contains_key("mod_depth"));
        aw_view.close_ring_mod_hud();
        assert!(!aw_view.is_ring_mod_hud_open());

        // Test Dither Quantizer HUD Lifecycle
        assert!(!aw_view.is_dither_quantizer_hud_open());
        aw_view.open_dither_quantizer_hud();
        assert!(aw_view.is_dither_quantizer_hud_open());
        aw_view.sync_dither_quantizer_hud_state();
        assert!(aw_view.device_rack_state.node_param_values.contains_key("bit_depth"));
        assert!(aw_view.inspector_state.node_param_values.contains_key("shaping_intensity"));
        aw_view.close_dither_quantizer_hud();
        assert!(!aw_view.is_dither_quantizer_hud_open());
    }

    #[test]
    fn test_turn120_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_ring_mod = actions.iter().any(|a| a.action_id == "open_ring_mod_hud");
        let has_dither = actions.iter().any(|a| a.action_id == "open_dither_quantizer_hud");

        assert!(has_ring_mod, "open_ring_mod_hud must be registered in command palette");
        assert!(has_dither, "open_dither_quantizer_hud must be registered in command palette");
    }
}
