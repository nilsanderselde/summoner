#![allow(clippy::all)]

//! Test suite for Turn #112 (Sprint Turn #38) — Psychoacoustic Multi-Band
//! Spectral Transient De-Bleed & Spill Eliminator HUD and Multi-Band Transient
//! Spectral Reshaper & Formant Envelope HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::spectral_debleed_view::{
        SpectralDebleedView, DebleedMode, DEBLEED_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_reshaper_view::{
        SpectralReshaperView, ReshaperPreset, RESHAPER_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn112_pure_spectral_debleed_initialization_and_kinematics() {
        let mut view = SpectralDebleedView::new();
        assert_eq!(view.debleed_mode, DebleedMode::DrumKitHiHatBleed);
        assert_eq!(DEBLEED_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Modes and expected values
        let modes = [
            (DebleedMode::DrumKitHiHatBleed, "DRUM HI-HAT SPILL", -28.0, 2.4, 0.95, 0.90),
            (DebleedMode::VocalHeadphoneSpill, "VOCAL HP SPILL", -42.0, 1.8, 0.70, 0.98),
            (DebleedMode::LiveStageGuitarSpill, "STAGE GUITAR SPILL", -22.0, 3.2, 0.85, 0.82),
            (DebleedMode::AcousticPianoHammerDebleed, "PIANO HAMMER/DAMPER", -36.0, 1.5, 0.80, 0.94),
            (DebleedMode::OrchestralSectionIsolator, "ORCHESTRA ISOLATOR", -32.0, 2.0, 0.60, 0.88),
        ];

        for (m, expected_name, exp_thresh, exp_gamma, exp_pres, exp_coh) in modes {
            view.set_debleed_mode(m);
            assert_eq!(m.name(), expected_name);
            assert_eq!(m.mode_name(), expected_name);
            assert_eq!(m.nominal_thresh_db(), exp_thresh);
            assert_eq!(m.nominal_mask_sharpness(), exp_gamma);
            assert_eq!(m.nominal_transient_preserve(), exp_pres);
            assert_eq!(m.nominal_phase_coherence(), exp_coh);
            assert_eq!(view.separation_thresh_db, exp_thresh);
            assert_eq!(view.mask_sharpness_gamma, exp_gamma);
            assert_eq!(view.transient_preserve_pct, exp_pres);
            assert_eq!(view.phase_coherence_pct, exp_coh);
        }

        // Normalization round-trips for Threshold [-60.0 ..= 0.0 dBFS]
        let thresh_test = -34.5;
        let thresh_norm = SpectralDebleedView::thresh_to_normalized(thresh_test);
        let thresh_back = SpectralDebleedView::normalized_to_thresh(thresh_norm);
        assert!((thresh_back - thresh_test).abs() < 1e-3);

        // Normalization round-trips for Sharpness [0.5 ..= 4.0]
        let sharp_test = 2.75;
        let sharp_norm = SpectralDebleedView::sharpness_to_normalized(sharp_test);
        let sharp_back = SpectralDebleedView::normalized_to_sharpness(sharp_norm);
        assert!((sharp_back - sharp_test).abs() < 1e-3);

        // Spectral mask update & attenuation calculations
        view.separation_thresh_db = -30.0;
        view.mask_sharpness_gamma = 2.0;
        view.update_spectral_mask();
        for &att in &view.spectral_attenuations_db {
            assert!(att <= 0.0 && att >= -48.0);
        }
        for &leak in &view.leakage_energy_levels {
            assert!(leak >= 0.0 && leak <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_debleed_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_debleed_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_debleed_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_debleed_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PSYCHOACOUSTIC SPECTRAL TRANSIENT DE-BLEED & SPILL ELIMINATOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn112_pure_spectral_reshaper_initialization_and_kinematics() {
        let mut view = SpectralReshaperView::new();
        assert_eq!(view.preset, ReshaperPreset::DrumKitOverhead);
        assert_eq!(view.selected_band, 1);
        assert_eq!(RESHAPER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Presets and expected values
        let presets = [
            (ReshaperPreset::DrumKitOverhead, "DRUM KIT OVERHEAD"),
            (ReshaperPreset::SnareCloseMic, "SNARE CLOSE MIC"),
            (ReshaperPreset::AcousticGuitarSnap, "ACOUSTIC GUITAR SNAP"),
            (ReshaperPreset::VocalPlosiveTamer, "VOCAL PLOSIVE TAMER"),
            (ReshaperPreset::MasterPunch, "MASTER PUNCH"),
        ];

        for (p, expected_name) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), expected_name);
            assert_eq!(view.crossovers_hz, p.default_crossovers_hz());
            assert_eq!(view.attack_db, p.default_attack_db());
            assert_eq!(view.sustain_db, p.default_sustain_db());
            assert_eq!(view.debleed_thresh_db, p.default_debleed_thresh_db());
        }

        // Normalization round-trips for Attack Gain [-12.0 ..= +12.0 dB]
        let attack_test = 4.5;
        let attack_norm = SpectralReshaperView::attack_to_normalized(attack_test);
        let attack_back = SpectralReshaperView::normalized_to_attack(attack_norm);
        assert!((attack_back - attack_test).abs() < 1e-3);

        // Normalization round-trips for Sustain Gain [-12.0 ..= +12.0 dB]
        let sustain_test = -3.2;
        let sustain_norm = SpectralReshaperView::sustain_to_normalized(sustain_test);
        let sustain_back = SpectralReshaperView::normalized_to_sustain(sustain_norm);
        assert!((sustain_back - sustain_test).abs() < 1e-3);

        // Normalization round-trips for De-Bleed Threshold [-60.0 ..= 0.0 dB]
        let thresh_test = -25.0;
        let thresh_norm = SpectralReshaperView::thresh_to_normalized(thresh_test);
        let thresh_back = SpectralReshaperView::normalized_to_thresh(thresh_norm);
        assert!((thresh_back - thresh_test).abs() < 1e-3);

        // Frequency response evaluation across acoustic spectrum
        let gain_low = view.evaluate_frequency_response(100.0);
        let gain_mid = view.evaluate_frequency_response(1000.0);
        let gain_high = view.evaluate_frequency_response(10000.0);
        assert!(gain_low >= -12.0 && gain_low <= 12.0);
        assert!(gain_mid >= -12.0 && gain_mid <= 12.0);
        assert!(gain_high >= -12.0 && gain_high <= 12.0);

        // Simulation update & crest factor / isolation score
        view.update_dsp_metrics();
        assert!(view.transient_crest_factor_db >= 6.0 && view.transient_crest_factor_db <= 26.0);
        assert!(view.debleed_isolation_score >= 0.1 && view.debleed_isolation_score <= 1.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.band_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.band_puck_pos.1) * canvas.height;

        assert!(view.hit_test_band_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_band_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_band_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_band_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTI-BAND TRANSIENT SPECTRAL RESHAPER & FORMANT ENVELOPE HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn112_gui_debleed_and_reshaper_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Spectral De-Bleed Modal Lifecycle
        assert!(!view.is_spectral_debleed_hud_open());
        view.open_spectral_debleed_hud();
        assert!(view.is_spectral_debleed_hud_open());
        view.close_spectral_debleed_hud();
        assert!(!view.is_spectral_debleed_hud_open());

        // 2. Spectral Reshaper Modal Lifecycle
        assert!(!view.is_spectral_reshaper_hud_open());
        view.open_spectral_reshaper_hud();
        assert!(view.is_spectral_reshaper_hud_open());
        view.close_spectral_reshaper_hud();
        assert!(!view.is_spectral_reshaper_hud_open());

        // 3. Spectral De-Bleed Parameter Sync Bridge
        view.spectral_debleed_view.separation_thresh_db = -32.5;
        view.spectral_debleed_view.mask_sharpness_gamma = 2.8;
        view.spectral_debleed_view.transient_preserve_pct = 0.88;
        view.spectral_debleed_view.phase_coherence_pct = 0.92;
        view.sync_spectral_debleed_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("separation_thresh_db").copied(),
            Some(-32.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("mask_sharpness_gamma").copied(),
            Some(2.8)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("separation_thresh_db").copied(),
            Some(-32.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("mask_sharpness_gamma").copied(),
            Some(2.8)
        );

        // 4. Spectral Reshaper Parameter Sync Bridge
        view.spectral_reshaper_view.selected_band = 1;
        view.spectral_reshaper_view.attack_db[1] = 5.0;
        view.spectral_reshaper_view.sustain_db[1] = -2.5;
        view.spectral_reshaper_view.debleed_thresh_db[1] = -38.0;
        view.spectral_reshaper_view.debleed_isolation_score = 0.91;
        view.spectral_reshaper_view.transient_crest_factor_db = 18.2;
        view.sync_spectral_reshaper_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("attack_gain_db").copied(),
            Some(5.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("sustain_gain_db").copied(),
            Some(-2.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("attack_gain_db").copied(),
            Some(5.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("sustain_gain_db").copied(),
            Some(-2.5)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_spectral_debleed_modal_window(ui);
                view.show_spectral_reshaper_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let sd_action = palette.actions.iter().find(|a| a.action_id == "open_spectral_debleed_hud");
        assert!(sd_action.is_some(), "open_spectral_debleed_hud should be registered");
        assert_eq!(sd_action.unwrap().category, "Spectral & Synthesis");

        let sr_action = palette.actions.iter().find(|a| a.action_id == "open_spectral_reshaper_hud");
        assert!(sr_action.is_some(), "open_spectral_reshaper_hud should be registered");
        assert_eq!(sr_action.unwrap().category, "Spectral & Synthesis");
    }
}
