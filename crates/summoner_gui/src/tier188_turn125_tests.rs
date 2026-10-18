#![allow(clippy::all)]

//! Test suite for Turn #125 (Sprint Turn #51) — 8-Band Parametric Equalizer & Biquad Magnitude Response HUD
//! and Multi-Sample SFZ Keyzone & Velocity Layer Mapper HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::parametric_eq_view::{
        BiquadFilterMode, ParametricEqPreset, ParametricEqView, PARAMETRIC_EQ_PUCK_HIT_RADIUS,
    };
    use crate::views::sfz_sampler_view::{
        SfzLoopTopology, SfzRegionSlot, SfzSamplerPreset, SfzSamplerView,
        SFZ_SAMPLER_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn125_pure_parametric_eq_initialization_and_biquad_physics() {
        let mut view = ParametricEqView::new();
        assert_eq!(view.preset, ParametricEqPreset::StudioFlatReference);
        assert_eq!(PARAMETRIC_EQ_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected active band parameters
        let presets = [
            (
                ParametricEqPreset::StudioFlatReference,
                "Studio Flat Reference (8-Band 0.0 dB Unity)",
                3,
                1000.0,
                0.0,
                1.0,
                0.0,
            ),
            (
                ParametricEqPreset::VocalPresenceSilk,
                "Vocal Presence & Air Silk (+3.5 dB @ 2.5 kHz)",
                4,
                2500.0,
                3.5,
                1.25,
                -0.5,
            ),
            (
                ParametricEqPreset::SubBassPunchSculpt,
                "Sub-Bass Punch Sculpt (+4.5 dB @ 60 Hz Shelf)",
                0,
                60.0,
                4.5,
                0.71,
                -1.0,
            ),
            (
                ParametricEqPreset::SurgicalResonanceNotch,
                "Surgical Resonance Notch (-12.0 dB @ 4.0 kHz Q=6)",
                5,
                4000.0,
                -12.0,
                6.00,
                0.0,
            ),
        ];

        for (p, exp_name, exp_idx, exp_freq, exp_gain, exp_q, exp_trim) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            assert_eq!(view.selected_band_idx, exp_idx);
            assert!((view.active_freq_hz - exp_freq).abs() < 1e-2);
            assert!((view.active_gain_db - exp_gain).abs() < 1e-2);
            assert!((view.active_q - exp_q).abs() < 1e-2);
            assert!((view.output_trim_db - exp_trim).abs() < 1e-2);
        }

        // Logarithmic frequency & linear gain normalization round-trip
        for f in [20.0, 100.0, 1000.0, 5000.0, 20000.0] {
            let norm = ParametricEqView::freq_to_normalized(f);
            assert!((0.0..=1.0).contains(&norm));
            let back = ParametricEqView::normalized_to_freq(norm);
            assert!((back - f).abs() / f < 1e-3);
        }
        for g in [-18.0, -9.0, 0.0, 6.0, 18.0] {
            let norm = ParametricEqView::gain_to_normalized(g);
            assert!((0.0..=1.0).contains(&norm));
            let back = ParametricEqView::normalized_to_gain(norm);
            assert!((back - g).abs() < 1e-2);
        }

        // Verify RBJ biquad coefficients across all 7 BiquadFilterMode variants
        let modes = [
            BiquadFilterMode::Lowpass,
            BiquadFilterMode::Highpass,
            BiquadFilterMode::Bandpass,
            BiquadFilterMode::Notch,
            BiquadFilterMode::Peaking,
            BiquadFilterMode::LowShelf,
            BiquadFilterMode::HighShelf,
        ];
        for m in modes {
            view.bands[0].mode = m;
            view.bands[0].freq_hz = 1000.0;
            view.bands[0].gain_db = 4.0;
            view.bands[0].q_factor = 1.0;
            let (b0, b1, b2, a1, a2) = view.compute_biquad_coeffs(0);
            assert!(
                b0.is_finite() && b1.is_finite() && b2.is_finite() && a1.is_finite() && a2.is_finite()
            );
            assert!(!m.label().is_empty());
            assert!(!m.short_tag().is_empty());
        }

        // Magnitude response curve evaluation
        view.set_preset(ParametricEqPreset::VocalPresenceSilk);
        view.bypassed = true;
        assert!((view.evaluate_magnitude_db_at_freq(2500.0)).abs() < 1e-5);
        view.bypassed = false;
        let db_at_2500 = view.evaluate_magnitude_db_at_freq(2500.0);
        assert!(db_at_2500 > 1.0); // boosted presence band

        for frac in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let y = view.evaluate_response_curve_normalized(frac);
            assert!((0.0..=1.0).contains(&y));
        }

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("8-BAND PARAMETRIC EQ & BIQUAD RESPONSE SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }

    #[test]
    fn test_turn125_pure_sfz_sampler_initialization_and_keyzone_physics() {
        let mut view = SfzSamplerView::new();
        assert_eq!(view.preset, SfzSamplerPreset::ConcertGrandVelocityLayers);
        assert_eq!(SFZ_SAMPLER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (
                SfzSamplerPreset::ConcertGrandVelocityLayers,
                "Concert Grand Piano (Velocity Layers + LoopRelease)",
                0.25,
                0.55,
                60,
                0.0,
                15.0,
                0.85,
                SfzLoopTopology::LoopRelease,
            ),
            (
                SfzSamplerPreset::OrchestralStringsSustainLoop,
                "Orchestral Strings Ensemble (Continuous Loop)",
                0.20,
                0.70,
                60,
                2.0,
                45.0,
                0.65,
                SfzLoopTopology::LoopContinuous,
            ),
            (
                SfzSamplerPreset::AmbientPadPingPong,
                "Crystalline Ambient Pad (Bidirectional Ping-Pong)",
                0.15,
                0.75,
                64,
                -4.0,
                80.0,
                0.50,
                SfzLoopTopology::PingPong,
            ),
            (
                SfzSamplerPreset::VintageMpcChopsNoLoop,
                "Vintage MPC Keyzone Chops (One-Shot NoLoop)",
                0.00,
                1.00,
                48,
                0.0,
                2.0,
                1.00,
                SfzLoopTopology::NoLoop,
            ),
        ];

        for (p, exp_name, exp_start, exp_len, exp_root, exp_fine, exp_xfade, exp_vel, exp_mode) in
            presets
        {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (s, l, r, f, x, v, m) = p.nominal_parameters();
            assert_eq!(s, exp_start);
            assert_eq!(l, exp_len);
            assert_eq!(r, exp_root);
            assert_eq!(f, exp_fine);
            assert_eq!(x, exp_xfade);
            assert_eq!(v, exp_vel);
            assert_eq!(m, exp_mode);

            assert!((view.loop_start_norm - exp_start).abs() < 1e-4);
            assert!((view.loop_length_norm - exp_len).abs() < 1e-4);
            assert_eq!(view.root_keycenter, exp_root);
            assert!((view.fine_tune_cents - exp_fine).abs() < 1e-4);
            assert!((view.crossfade_ms - exp_xfade).abs() < 1e-4);
            assert!((view.velocity_tracking - exp_vel).abs() < 1e-4);
            assert_eq!(view.loop_mode, exp_mode);
        }

        // Loop Start & Loop Length normalization round-trip
        for s in [0.0, 0.20, 0.50, 0.85] {
            let norm = SfzSamplerView::loop_start_to_normalized(s);
            assert!((0.0..=1.0).contains(&norm));
            let back = SfzSamplerView::normalized_to_loop_start(norm);
            assert!((back - s).abs() < 1e-3);
        }
        for l in [0.05, 0.35, 0.75, 1.0] {
            let norm = SfzSamplerView::loop_length_to_normalized(l);
            assert!((0.0..=1.0).contains(&norm));
            let back = SfzSamplerView::normalized_to_loop_length(norm);
            assert!((back - l).abs() < 1e-3);
        }

        // Semitone pitch rate calculation (C4=60 -> 1.0x, C5=72 -> 2.0x)
        view.set_preset(SfzSamplerPreset::ConcertGrandVelocityLayers);
        assert!((view.compute_playback_rate(60) - 1.0).abs() < 1e-5);
        assert!((view.compute_playback_rate(72) - 2.0).abs() < 1e-5);
        assert!((view.compute_playback_rate(48) - 0.5).abs() < 1e-5);

        // Keyzone auto-mapping & velocity region matching
        view.regions = vec![
            SfzRegionSlot::new(0, 0, 36, 0, 127, SfzLoopTopology::Loop, "C2.wav"),
            SfzRegionSlot::new(0, 0, 60, 0, 127, SfzLoopTopology::Loop, "C4.wav"),
            SfzRegionSlot::new(0, 0, 84, 0, 127, SfzLoopTopology::Loop, "C6.wav"),
        ];
        view.auto_map_regions_by_keycenter();
        assert_eq!(view.regions[0].lokey, 0);
        assert_eq!(view.regions[0].hikey, 48);
        assert_eq!(view.regions[1].lokey, 49);
        assert_eq!(view.regions[1].hikey, 72);
        assert_eq!(view.regions[2].lokey, 73);
        assert_eq!(view.regions[2].hikey, 127);

        let matched = view.find_matching_region(60, 100).unwrap();
        assert_eq!(matched.sample_label, "C4.wav");

        // Touch puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 100.0);
        let (px, py) = view.puck_to_canvas_pos(&canvas);
        assert!(view.is_puck_hit(&canvas, (px, py)));
        assert!(!view.is_puck_hit(&canvas, (px + 50.0, py + 50.0)));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot();
        assert!(snap.contains("MULTI-SAMPLE SFZ KEYZONE & VELOCITY MAPPER SNAPSHOT"));
        assert!(snap.contains("[PASS: >=44pt Hit Target]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::parametric_eq_view::ParametricEqView;
    use crate::views::sfz_sampler_view::SfzSamplerView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn125_headless_render_views() {
        let ctx = egui::Context::default();
        let mut eq_view = ParametricEqView::new();
        let mut sfz_view = SfzSamplerView::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                eq_view.show(ui);
                sfz_view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn125_award_winning_gui_modal_lifecycles_and_parambus_sync() {
        let mut aw_view = AwardWinningGuiView::new();
        let bus = Arc::new(ParamBus::new());
        aw_view.live_param_bus = Some(bus.clone());

        // Test 8-Band Parametric EQ HUD Lifecycle
        assert!(!aw_view.is_parametric_eq_hud_open());
        aw_view.open_parametric_eq_hud();
        assert!(aw_view.is_parametric_eq_hud_open());
        aw_view.sync_parametric_eq_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("eq_freq_hz"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("eq_gain_db"));
        aw_view.close_parametric_eq_hud();
        assert!(!aw_view.is_parametric_eq_hud_open());

        // Test Multi-Sample SFZ Keyzone Mapper HUD Lifecycle
        assert!(!aw_view.is_sfz_sampler_hud_open());
        aw_view.open_sfz_sampler_hud();
        assert!(aw_view.is_sfz_sampler_hud_open());
        aw_view.sync_sfz_sampler_hud_state();
        assert!(aw_view
            .device_rack_state
            .node_param_values
            .contains_key("sfz_loop_start"));
        assert!(aw_view
            .inspector_state
            .node_param_values
            .contains_key("sfz_loop_length"));
        aw_view.close_sfz_sampler_hud();
        assert!(!aw_view.is_sfz_sampler_hud_open());
    }

    #[test]
    fn test_turn125_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_eq = actions
            .iter()
            .any(|a| a.action_id == "open_parametric_eq_hud");
        let has_sfz = actions
            .iter()
            .any(|a| a.action_id == "open_sfz_sampler_hud");

        assert!(
            has_eq,
            "open_parametric_eq_hud must be registered in command palette"
        );
        assert!(
            has_sfz,
            "open_sfz_sampler_hud must be registered in command palette"
        );
    }
}
