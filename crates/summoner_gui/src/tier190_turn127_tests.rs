#![allow(clippy::all)]

//! Test suite for Turn #127 (Sprint Turn #53) — 16-Voice SIMD 4-Wide Polyphonic Hermite Wavetable Oscillator HUD
//! and 3-Band SVF Crossover Multiband Dynamics Compressor HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::multiband_compressor_view::{
        MultibandCompBandSlot, MultibandCompressorPreset, MultibandCompressorView,
        MULTIBAND_COMP_PUCK_HIT_RADIUS,
    };
    use crate::views::simd_poly_wavetable_view::{
        SimdPolyWavetablePreset, SimdPolyWavetableView, WavetableFrameShape, MAX_SIMD_POLY_VOICES,
        SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS, WAVETABLE_FRAME_SIZE,
    };

    #[test]
    fn test_turn127_pure_simd_poly_wavetable_initialization_and_hermite_physics() {
        let mut view = SimdPolyWavetableView::new();
        assert_eq!(view.preset, SimdPolyWavetablePreset::AnalogSawSquarePad);
        assert_eq!(SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target
        assert_eq!(WAVETABLE_FRAME_SIZE, 2048);
        assert_eq!(MAX_SIMD_POLY_VOICES, 16);

        // Presets and expected parameters
        let presets = [
            (
                SimdPolyWavetablePreset::AnalogSawSquarePad,
                "Analog Saw-to-Square Pad (35% Morph / 220 Hz / 8 SIMD Voices)",
                0.35,
                220.0,
                8,
                0.08,
                0.9992,
                8.5,
                WavetableFrameShape::HarmonicSawtooth,
                WavetableFrameShape::HollowSquare,
            ),
            (
                SimdPolyWavetablePreset::CrystallineChebyshevChoir,
                "Crystalline Chebyshev Bell Choir (68% Morph / 440 Hz / 12 Voices)",
                0.68,
                440.0,
                12,
                0.25,
                0.9995,
                12.0,
                WavetableFrameShape::PureSine,
                WavetableFrameShape::ChebyshevFoldedHarmonic,
            ),
            (
                SimdPolyWavetablePreset::FormantSyncHyperLead,
                "Formant Sync Hyper Lead (82% Morph / 880 Hz / 4 SIMD Voices)",
                0.82,
                880.0,
                4,
                0.45,
                0.9980,
                18.0,
                WavetableFrameShape::HarmonicSawtooth,
                WavetableFrameShape::FormantSyncSweep,
            ),
            (
                SimdPolyWavetablePreset::SubHarmonic16VoiceStack,
                "Sub-Harmonic 16-Voice Unison Stack (22% Morph / 65.4 Hz / 16 Voices)",
                0.22,
                65.4,
                16,
                0.15,
                0.9990,
                6.0,
                WavetableFrameShape::PureSine,
                WavetableFrameShape::HarmonicSawtooth,
            ),
        ];

        for (p, exp_name, exp_m, exp_f, exp_v, exp_a, exp_r, exp_d, exp_ta, exp_tb) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (m, f, v, a, r, d, ta, tb) = p.nominal_parameters();
            assert_eq!(m, exp_m);
            assert_eq!(f, exp_f);
            assert_eq!(v, exp_v);
            assert_eq!(a, exp_a);
            assert_eq!(r, exp_r);
            assert_eq!(d, exp_d);
            assert_eq!(ta, exp_ta);
            assert_eq!(tb, exp_tb);

            assert!((view.morph - exp_m).abs() < 1e-4);
            assert!((view.fundamental_freq_hz - exp_f).abs() < 1e-2);
            assert_eq!(view.active_voices, exp_v);
            assert!((view.attack_rate - exp_a).abs() < 1e-4);
            assert!((view.release_decay_rate - exp_r).abs() < 1e-4);
            assert!((view.unison_detune_cents - exp_d).abs() < 1e-4);
            assert_eq!(view.table_a, exp_ta);
            assert_eq!(view.table_b, exp_tb);
        }

        // Frame shape evaluation & metadata
        let shapes = [
            WavetableFrameShape::PureSine,
            WavetableFrameShape::HarmonicSawtooth,
            WavetableFrameShape::HollowSquare,
            WavetableFrameShape::ChebyshevFoldedHarmonic,
            WavetableFrameShape::FormantSyncSweep,
        ];
        for sh in shapes {
            assert!(!sh.label().is_empty());
            assert!(!sh.short_tag().is_empty());
            for ph in [0.0, 0.25, 0.5, 0.75] {
                let s = sh.sample_at_phase(ph);
                assert!((-1.0..=1.0).contains(&s));
            }
        }

        // Morph & Logarithmic Fundamental Hz normalization round-trip
        for m in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = SimdPolyWavetableView::morph_to_normalized(m);
            assert!((0.0..=1.0).contains(&norm));
            let back = SimdPolyWavetableView::normalized_to_morph(norm);
            assert!((back - m).abs() < 1e-4);
        }
        for f in [20.0, 110.0, 440.0, 2000.0, 8000.0] {
            let norm = SimdPolyWavetableView::freq_to_normalized(f);
            assert!((0.0..=1.0).contains(&norm));
            let back = SimdPolyWavetableView::normalized_to_freq(norm);
            assert!((back - f).abs() / f < 1e-3);
        }

        // SIMD 4-wide batch breakdown math
        view.active_voices = 14;
        assert_eq!(view.simd_batch_breakdown(), (3, 2)); // 3 * 4 + 2 = 14
        view.active_voices = 16;
        assert_eq!(view.simd_batch_breakdown(), (4, 0));

        // Hermite vs Nyquist anti-aliasing blend: (1.0 - dt * 2048 * 1.5).clamp(0.0, 1.0)
        view.sample_rate = 48000;
        view.fundamental_freq_hz = 20.0; // low pitch -> high Hermite cubic weight
        assert!(view.compute_hermite_aa_blend() > 0.0);
        view.fundamental_freq_hz = 16000.0; // high pitch -> Nyquist linear fallback (0.0 cubic weight)
        assert_eq!(view.compute_hermite_aa_blend(), 0.0);

        // Morphed sample evaluation
        view.set_preset(SimdPolyWavetablePreset::AnalogSawSquarePad);
        view.bypassed = true;
        assert_eq!(view.evaluate_hermite_morphed_sample(0.25), 0.0);
        view.bypassed = false;
        let sample = view.evaluate_hermite_morphed_sample(0.25);
        assert!((-1.0..=1.0).contains(&sample));

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("SIMD 4-WIDE POLYPHONIC WAVETABLE OSCILLATOR SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn127_pure_multiband_compressor_initialization_and_svf_crossover_physics() {
        let mut view = MultibandCompressorView::new();
        assert_eq!(
            view.preset,
            MultibandCompressorPreset::StudioThreeBandGlue
        );
        assert_eq!(MULTIBAND_COMP_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                MultibandCompressorPreset::StudioThreeBandGlue,
                "Studio 3-Band Glue (200 Hz / 2.0 kHz SVF Crossover)",
                200.0,
                2000.0,
                -18.0,
                3.0,
                -16.0,
                2.5,
                -14.0,
                2.0,
                -6.0,
            ),
            (
                MultibandCompressorPreset::TightSubCrispAir,
                "Tight Sub & Crisp Air (140 Hz / 3.8 kHz Mastering)",
                140.0,
                3800.0,
                -22.0,
                4.5,
                -15.0,
                2.0,
                -16.0,
                3.0,
                -5.0,
            ),
            (
                MultibandCompressorPreset::BroadcastMidrangeControl,
                "Broadcast Midrange Control (260 Hz / 2.8 kHz Vocal Focus)",
                260.0,
                2800.0,
                -20.0,
                3.5,
                -19.0,
                4.0,
                -15.0,
                2.5,
                -4.5,
            ),
            (
                MultibandCompressorPreset::AggressiveElectronicPunch,
                "Aggressive Electronic Punch (180 Hz / 2.4 kHz High-Ratio)",
                180.0,
                2400.0,
                -24.0,
                6.0,
                -21.0,
                4.5,
                -18.0,
                5.0,
                -3.0,
            ),
        ];

        for (p, exp_name, exp_lm, exp_mh, exp_lt, exp_lr, exp_mt, exp_mr, exp_ht, exp_hr, exp_in) in
            presets
        {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (lm, mh, low_slot, mid_slot, high_slot, in_db) = p.nominal_parameters();
            assert_eq!(lm, exp_lm);
            assert_eq!(mh, exp_mh);
            assert_eq!(low_slot.threshold_db, exp_lt);
            assert_eq!(low_slot.ratio, exp_lr);
            assert_eq!(mid_slot.threshold_db, exp_mt);
            assert_eq!(mid_slot.ratio, exp_mr);
            assert_eq!(high_slot.threshold_db, exp_ht);
            assert_eq!(high_slot.ratio, exp_hr);
            assert_eq!(in_db, exp_in);

            assert!((view.low_mid_freq_hz - exp_lm).abs() < 1e-2);
            assert!((view.mid_high_freq_hz - exp_mh).abs() < 1e-2);
            assert!((view.low_band.threshold_db - exp_lt).abs() < 1e-4);
            assert!((view.low_band.ratio - exp_lr).abs() < 1e-4);
            assert!((view.mid_band.threshold_db - exp_mt).abs() < 1e-4);
            assert!((view.mid_band.ratio - exp_mr).abs() < 1e-4);
            assert!((view.high_band.threshold_db - exp_ht).abs() < 1e-4);
            assert!((view.high_band.ratio - exp_hr).abs() < 1e-4);
            assert!((view.test_input_level_db - exp_in).abs() < 1e-4);
        }

        // Logarithmic Low-Mid & Mid-High crossover normalization round-trip
        for lm in [60.0, 150.0, 350.0, 800.0] {
            let norm = MultibandCompressorView::low_mid_to_normalized(lm);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandCompressorView::normalized_to_low_mid(norm);
            assert!((back - lm).abs() / lm < 1e-3);
        }
        for mh in [1000.0, 2500.0, 5000.0, 10000.0] {
            let norm = MultibandCompressorView::mid_high_to_normalized(mh);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandCompressorView::normalized_to_mid_high(norm);
            assert!((back - mh).abs() / mh < 1e-3);
        }

        // Gain reduction math & composite response evaluation
        let slot = MultibandCompBandSlot::new(-18.0, 3.0, 10.0, 100.0, 0.0);
        assert_eq!(slot.evaluate_gain_reduction_db(-30.0), 0.0);
        let gr = slot.evaluate_gain_reduction_db(-6.0);
        // Overshoot = 12 dB, ratio = 3:1 -> GR = -12 * (1 - 1/3) = -8.0 dB
        assert!((gr - (-8.0)).abs() < 1e-4);

        view.set_preset(MultibandCompressorPreset::StudioThreeBandGlue);
        let (gr_l, gr_m, gr_h) = view.evaluate_three_band_gain_reduction_db();
        assert!(gr_l <= 0.0 && gr_m <= 0.0 && gr_h <= 0.0);

        view.bypassed = true;
        assert_eq!(view.evaluate_composite_response_db_at_freq(1000.0), 0.0);
        view.bypassed = false;
        for f in [50.0, 200.0, 1000.0, 3000.0, 12000.0] {
            let resp = view.evaluate_composite_response_db_at_freq(f);
            assert!((-24.0..=12.0).contains(&resp));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("3-BAND SVF MULTIBAND COMPRESSOR HUD SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::multiband_compressor_view::MultibandCompressorView;
    use crate::views::simd_poly_wavetable_view::SimdPolyWavetableView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn127_headless_render_views() {
        let ctx = egui::Context::default();
        let mut poly_wt_view = SimdPolyWavetableView::new();
        let mut mb_comp_view = MultibandCompressorView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                poly_wt_view.show(ui);
                mb_comp_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn127_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test SIMD 4-Wide Polyphonic Wavetable HUD Lifecycle
        assert!(!aw_view.is_simd_poly_wavetable_hud_open());
        aw_view.open_simd_poly_wavetable_hud();
        assert!(aw_view.is_simd_poly_wavetable_hud_open());
        aw_view.sync_simd_poly_wavetable_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("poly_wavetable_morph"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("poly_wavetable_freq_hz"));
        aw_view.close_simd_poly_wavetable_hud();
        assert!(!aw_view.is_simd_poly_wavetable_hud_open());

        // Test 3-Band Multiband Dynamics Compressor HUD Lifecycle
        assert!(!aw_view.is_multiband_compressor_hud_open());
        aw_view.open_multiband_compressor_hud();
        assert!(aw_view.is_multiband_compressor_hud_open());
        aw_view.sync_multiband_compressor_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("mb_comp_low_mid_hz"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("mb_comp_mid_high_hz"));
        aw_view.close_multiband_compressor_hud();
        assert!(!aw_view.is_multiband_compressor_hud_open());
    }

    #[test]
    fn test_turn127_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_poly_wt = actions
            .iter()
            .any(|a| a.action_id == "open_simd_poly_wavetable_hud");
        let has_mb_comp = actions
            .iter()
            .any(|a| a.action_id == "open_multiband_compressor_hud");

        assert!(
            has_poly_wt,
            "open_simd_poly_wavetable_hud must be registered in command palette"
        );
        assert!(
            has_mb_comp,
            "open_multiband_compressor_hud must be registered in command palette"
        );
    }
}
