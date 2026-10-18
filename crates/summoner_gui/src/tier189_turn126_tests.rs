#![allow(clippy::all)]

//! Test suite for Turn #126 (Sprint Turn #52) — Aether Subtractive & Glitch Composite Synthesizer HUD
//! and Polyphase 12-Tap Half-Band FIR Anti-Aliasing Oversampler HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::aether_synth_view::{
        AetherSynthPreset, AetherSynthView, CompositeSynthVoiceMode, AETHER_SYNTH_PUCK_HIT_RADIUS,
    };
    use crate::views::polyphase_oversampler_view::{
        OversampleFactorMode, PolyphaseOversamplerPreset, PolyphaseOversamplerView, FIR_TAPS_COUNT,
        HALF_BAND_FIR_COEFFS, OVERSAMPLER_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn126_pure_aether_synth_initialization_and_composite_physics() {
        let mut view = AetherSynthView::new();
        assert_eq!(view.preset, AetherSynthPreset::AetherLeadClassic);
        assert_eq!(AETHER_SYNTH_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                AetherSynthPreset::AetherLeadClassic,
                "Aether Classic Lead (Saw/Pulse 50% + 1.2 kHz Ladder)",
                0.50,
                1200.0,
                1.00,
                2.0,
                0.60,
                1.00,
                0.15,
                CompositeSynthVoiceMode::AetherSubtractive,
            ),
            (
                AetherSynthPreset::GlitchAetherChopper,
                "Glitch Aether Chopper (Tube Drive + 8 Hz GlitchGate)",
                0.65,
                2500.0,
                1.50,
                8.0,
                0.75,
                0.60,
                0.75,
                CompositeSynthVoiceMode::GlitchAetherMachine,
            ),
            (
                AetherSynthPreset::CyberpunkWavefoldSub,
                "Cyberpunk Wavefold Sub (600 Hz SVF + 4x Wavefolder)",
                0.30,
                600.0,
                1.20,
                4.0,
                0.35,
                1.00,
                0.85,
                CompositeSynthVoiceMode::CyberpunkSubSynth,
            ),
            (
                AetherSynthPreset::AtmosphericHorizonPad,
                "Atmospheric Horizon Pad (Detuned +3c + 0.5 Hz LFO)",
                0.50,
                800.0,
                0.71,
                0.5,
                0.45,
                1.00,
                0.05,
                CompositeSynthVoiceMode::AtmosphericPadSynth,
            ),
        ];

        for (p, exp_name, exp_mix, exp_cut, exp_res, exp_lfo, exp_env, exp_gate, exp_drv, exp_mode) in
            presets
        {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (m, c, r, l, e, g, d, v_mode) = p.nominal_parameters();
            assert_eq!(m, exp_mix);
            assert_eq!(c, exp_cut);
            assert_eq!(r, exp_res);
            assert_eq!(l, exp_lfo);
            assert_eq!(e, exp_env);
            assert_eq!(g, exp_gate);
            assert_eq!(d, exp_drv);
            assert_eq!(v_mode, exp_mode);

            assert!((view.osc_mix - exp_mix).abs() < 1e-4);
            assert!((view.filter_cutoff_hz - exp_cut).abs() < 1e-2);
            assert!((view.filter_resonance - exp_res).abs() < 1e-4);
            assert!((view.lfo_rate_hz - exp_lfo).abs() < 1e-4);
            assert!((view.filter_env_depth - exp_env).abs() < 1e-4);
            assert!((view.glitch_gate_duty - exp_gate).abs() < 1e-4);
            assert!((view.distortion_drive - exp_drv).abs() < 1e-4);
            assert_eq!(view.voice_mode, exp_mode);
        }

        // Voice modes metadata
        let voice_modes = [
            CompositeSynthVoiceMode::AetherSubtractive,
            CompositeSynthVoiceMode::GlitchAetherMachine,
            CompositeSynthVoiceMode::CyberpunkSubSynth,
            CompositeSynthVoiceMode::AtmosphericPadSynth,
            CompositeSynthVoiceMode::GlitchPercussionSynth,
        ];
        for vm in voice_modes {
            assert!(!vm.label().is_empty());
            assert!(!vm.short_tag().is_empty());
        }

        // Mix and Logarithmic Cutoff normalization round-trip
        for m in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = AetherSynthView::mix_to_normalized(m);
            assert!((0.0..=1.0).contains(&norm));
            let back = AetherSynthView::normalized_to_mix(norm);
            assert!((back - m).abs() < 1e-4);
        }
        for f in [20.0, 120.0, 1200.0, 5000.0, 20000.0] {
            let norm = AetherSynthView::cutoff_to_normalized(f);
            assert!((0.0..=1.0).contains(&norm));
            let back = AetherSynthView::normalized_to_cutoff(norm);
            assert!((back - f).abs() / f < 1e-3);
        }

        // PWM duty cycle and filter envelope modulation math
        view.set_preset(AetherSynthPreset::AetherLeadClassic);
        let pwm_peak = view.compute_pwm_duty_cycle(0.25); // sin(pi/2) = 1.0 -> 0.5 + 0.4 = 0.90
        assert!((pwm_peak - 0.90).abs() < 1e-4);
        let pwm_trough = view.compute_pwm_duty_cycle(0.75); // sin(3pi/2) = -1.0 -> 0.5 - 0.4 = 0.10
        assert!((pwm_trough - 0.10).abs() < 1e-4);

        let mod_cut = view.compute_modulated_cutoff_hz(1.0);
        // 1200.0 + 0.60 * 4000.0 = 3600.0 Hz
        assert!((mod_cut - 3600.0).abs() < 1e-2);

        // Composite waveform evaluation
        view.bypassed = true;
        assert!((view.evaluate_composite_waveform(0.5) - 0.5).abs() < 1e-5);
        view.bypassed = false;
        for frac in [0.0, 0.20, 0.45, 0.70, 0.95] {
            let y = view.evaluate_composite_waveform(frac);
            assert!((0.05..=0.95).contains(&y));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("AETHER SUBTRACTIVE & GLITCH COMPOSITE SYNTH SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn126_pure_polyphase_oversampler_initialization_and_fir_physics() {
        let mut view = PolyphaseOversamplerView::new();
        assert_eq!(view.preset, PolyphaseOversamplerPreset::Realtime4xHalfBand);
        assert_eq!(OVERSAMPLER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target
        assert_eq!(FIR_TAPS_COUNT, 12);
        assert_eq!(HALF_BAND_FIR_COEFFS.len(), 12);

        // Verify symmetric half-band FIR coefficients sum to 1.0
        let coeff_sum: f32 = HALF_BAND_FIR_COEFFS.iter().sum();
        assert!((coeff_sum - 1.0).abs() < 1e-4);
        for i in 0..6 {
            assert!((HALF_BAND_FIR_COEFFS[i] - HALF_BAND_FIR_COEFFS[11 - i]).abs() < 1e-6);
        }

        // Presets and expected parameters
        let presets = [
            (
                PolyphaseOversamplerPreset::Realtime4xHalfBand,
                "Real-Time 4x Half-Band FIR (192 kHz / 84 dB Stopband)",
                2.5,
                84.0,
                0.50,
                1.00,
                OversampleFactorMode::Factor4x,
            ),
            (
                PolyphaseOversamplerPreset::Mastering8xUltraClean,
                "Mastering 8x Ultra-Clean (384 kHz / 108 dB Stopband)",
                4.0,
                108.0,
                0.48,
                1.00,
                OversampleFactorMode::Factor8x,
            ),
            (
                PolyphaseOversamplerPreset::AggressiveWavefold8xGuard,
                "Aggressive Wavefolder 8x Guard (9.5x Drive / 114 dB)",
                9.5,
                114.0,
                0.45,
                0.90,
                OversampleFactorMode::Factor8x,
            ),
            (
                PolyphaseOversamplerPreset::LowLatency2xTracking,
                "Low-Latency 2x Tracking (96 kHz / 64 dB Stopband)",
                1.8,
                64.0,
                0.52,
                1.00,
                OversampleFactorMode::Factor2x,
            ),
        ];

        for (p, exp_name, exp_drv, exp_stop, exp_cut, exp_wet, exp_mode) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (d, s, c, w, m) = p.nominal_parameters();
            assert_eq!(d, exp_drv);
            assert_eq!(s, exp_stop);
            assert_eq!(c, exp_cut);
            assert_eq!(w, exp_wet);
            assert_eq!(m, exp_mode);

            assert!((view.nonlinear_drive - exp_drv).abs() < 1e-4);
            assert!((view.stopband_attenuation_db - exp_stop).abs() < 1e-4);
            assert!((view.cutoff_nyquist_fraction - exp_cut).abs() < 1e-4);
            assert!((view.wet_mix - exp_wet).abs() < 1e-4);
            assert_eq!(view.factor_mode, exp_mode);
        }

        // Factor modes metadata and internal sample rates
        view.base_sample_rate = 48000.0;
        view.factor_mode = OversampleFactorMode::Factor2x;
        assert_eq!(view.factor_mode.factor(), 2);
        assert!((view.internal_sample_rate_hz() - 96000.0).abs() < 1e-2);
        view.factor_mode = OversampleFactorMode::Factor4x;
        assert_eq!(view.factor_mode.factor(), 4);
        assert!((view.internal_sample_rate_hz() - 192000.0).abs() < 1e-2);
        view.factor_mode = OversampleFactorMode::Factor8x;
        assert_eq!(view.factor_mode.factor(), 8);
        assert!((view.internal_sample_rate_hz() - 384000.0).abs() < 1e-2);

        // Drive & Stopband dB normalization round-trip
        for d in [1.0, 3.5, 6.5, 12.0] {
            let norm = PolyphaseOversamplerView::drive_to_normalized(d);
            assert!((0.0..=1.0).contains(&norm));
            let back = PolyphaseOversamplerView::normalized_to_drive(norm);
            assert!((back - d).abs() < 1e-3);
        }
        for s in [48.0, 72.0, 96.0, 120.0] {
            let norm = PolyphaseOversamplerView::stopband_to_normalized(s);
            assert!((0.0..=1.0).contains(&norm));
            let back = PolyphaseOversamplerView::normalized_to_stopband(norm);
            assert!((back - s).abs() < 1e-3);
        }

        // Oversampled zero-stuffing + FIR decimation round-trip evaluation
        view.set_preset(PolyphaseOversamplerPreset::Realtime4xHalfBand);
        view.bypassed = true;
        assert!((view.evaluate_oversampled_roundtrip(0.5) - 0.5).abs() < 1e-6);
        view.bypassed = false;
        let out_sample = view.evaluate_oversampled_roundtrip(0.5);
        assert!(out_sample.is_finite());

        // FIR magnitude response evaluation
        let dc_mag = view.evaluate_fir_magnitude_response(0.0);
        let nyq_mag = view.evaluate_fir_magnitude_response(1.0);
        assert!(dc_mag > nyq_mag); // lowpass filter passes DC and attenuates Nyquist

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("POLYPHASE 12-TAP HALF-BAND FIR OVERSAMPLER SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::aether_synth_view::AetherSynthView;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::polyphase_oversampler_view::PolyphaseOversamplerView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn126_headless_render_views() {
        let ctx = egui::Context::default();
        let mut aether_view = AetherSynthView::new();
        let mut oversampler_view = PolyphaseOversamplerView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                aether_view.show(ui);
                oversampler_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn126_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test Aether Composite Synth HUD Lifecycle
        assert!(!aw_view.is_aether_synth_hud_open());
        aw_view.open_aether_synth_hud();
        assert!(aw_view.is_aether_synth_hud_open());
        aw_view.sync_aether_synth_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("aether_osc_mix"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("aether_cutoff_hz"));
        aw_view.close_aether_synth_hud();
        assert!(!aw_view.is_aether_synth_hud_open());

        // Test Polyphase 12-Tap FIR Oversampler HUD Lifecycle
        assert!(!aw_view.is_polyphase_oversampler_hud_open());
        aw_view.open_polyphase_oversampler_hud();
        assert!(aw_view.is_polyphase_oversampler_hud_open());
        aw_view.sync_polyphase_oversampler_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("oversample_drive"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("oversample_stopband_db"));
        aw_view.close_polyphase_oversampler_hud();
        assert!(!aw_view.is_polyphase_oversampler_hud_open());
    }

    #[test]
    fn test_turn126_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_aether = actions
            .iter()
            .any(|a| a.action_id == "open_aether_synth_hud");
        let has_oversampler = actions
            .iter()
            .any(|a| a.action_id == "open_polyphase_oversampler_hud");

        assert!(
            has_aether,
            "open_aether_synth_hud must be registered in command palette"
        );
        assert!(
            has_oversampler,
            "open_polyphase_oversampler_hud must be registered in command palette"
        );
    }
}
