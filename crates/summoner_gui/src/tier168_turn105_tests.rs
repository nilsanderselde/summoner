#![allow(clippy::all)]

//! Test suite for Turn #105 (Sprint Turn #31) — Mastering Linear-Phase Multiband Upward Dynamic De-Compressor
//! and Mastering Dynamic Stereo Width & Bass Mono HUDs Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::multiband_decompressor_view::{
        MultibandDecompressorView, DecompressorPreset, DECOMPRESSOR_PUCK_HIT_RADIUS,
    };
    use crate::views::dynamic_stereo_width_view::{
        DynamicStereoWidthView, StereoWidthProfile, STEREO_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn105_pure_multiband_decompressor_initialization_and_kinematics() {
        let mut view = MultibandDecompressorView::new();
        assert_eq!(view.selected_band, 1);
        assert_eq!(view.linear_phase_fir_taps, 1024);
        assert_eq!(DECOMPRESSOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Presets and their nominal names
        let presets = [
            (DecompressorPreset::MasterDynRestoration, "Master Dynamic Rescue"),
            (DecompressorPreset::DrumTransientRescue, "Drum Transients"),
            (DecompressorPreset::SlapBassPunch, "Slap Bass Punch"),
            (DecompressorPreset::VocalAirRestoration, "Vocal Air Expand"),
            (DecompressorPreset::OrchestralOpen, "Orchestral Open"),
        ];

        for (p, expected_name) in presets {
            view.set_preset(p);
            assert_eq!(view.preset, p);
            assert_eq!(p.name(), expected_name);
            assert!(p.default_threshold_db() <= 0.0);
            assert!(p.default_ratio() >= 1.0);
            assert!(p.default_range_db() > 0.0);
            let xo = p.default_crossovers_hz();
            assert!(xo[0] < xo[1] && xo[1] < xo[2]);
        }

        // Active band switching
        for b in 0..4 {
            view.set_active_band(b);
            assert_eq!(view.selected_band, b);
        }

        // Normalization round-trips for Threshold [-60.0 ..= 0.0 dB]
        let th_test = -22.5;
        let th_norm = MultibandDecompressorView::threshold_to_normalized(th_test);
        let th_back = MultibandDecompressorView::normalized_to_threshold(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Normalization round-trips for Ratio [1.0 ..= 4.0]
        let rt_test = 2.75;
        let rt_norm = MultibandDecompressorView::ratio_to_normalized(rt_test);
        let rt_back = MultibandDecompressorView::normalized_to_ratio(rt_norm);
        assert!((rt_back - rt_test).abs() < 1e-3);

        // Normalization round-trips for Range [0.0 ..= 18.0 dB]
        let rg_test = 8.5;
        let rg_norm = MultibandDecompressorView::range_to_normalized(rg_test);
        let rg_back = MultibandDecompressorView::normalized_to_range(rg_norm);
        assert!((rg_back - rg_test).abs() < 1e-3);

        // Evaluate upward expansion transfer curves
        view.set_preset(DecompressorPreset::MasterDynRestoration);
        view.set_active_band(1);
        let out_below = view.evaluate_expansion_curve(1, -40.0);
        let out_above = view.evaluate_expansion_curve(1, -10.0);
        assert_eq!(out_below, -40.0); // 1:1 below threshold (-18.0 dB)
        assert!(out_above > -10.0);   // Upward boost above threshold

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(40.0, 40.0, 360.0, 280.0);
        view.set_active_band(1);
        let puck_x = canvas.x + view.decompressor_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.decompressor_puck_pos.1) * canvas.height;

        assert!(view.hit_test_decompressor_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_decompressor_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_decompressor_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_decompressor_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING LINEAR-PHASE MULTIBAND DE-COMPRESSOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn105_pure_dynamic_stereo_width_initialization_and_kinematics() {
        let mut view = DynamicStereoWidthView::new();
        assert_eq!(view.elliptical_bass_hz, 120.0);
        assert_eq!(view.side_width_ratio, 1.25);
        assert_eq!(STEREO_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profiles and their nominal names
        let profiles = [
            (StereoWidthProfile::BroadcastMasteringClean, "BROADCAST MASTER (CLEAN)"),
            (StereoWidthProfile::ClubVinylPressing, "CLUB VINYL CUT (180Hz MONO)"),
            (StereoWidthProfile::CinematicSuperWide, "CINEMATIC SUPER WIDE"),
            (StereoWidthProfile::AcousticNaturalDepth, "ACOUSTIC NATURAL DEPTH"),
            (StereoWidthProfile::EDMPolyrhythmicHyperWidth, "EDM HYPER WIDTH (200%)"),
        ];

        for (prof, expected_name) in profiles {
            view.set_profile(prof);
            assert_eq!(view.profile, prof);
            assert_eq!(prof.name(), expected_name);
            assert!(prof.nominal_bass_cutoff_hz() >= 40.0);
            assert!(prof.nominal_width_ratio() > 0.0);
            assert!(prof.nominal_phase_correlation() > 0.0);
        }

        // Normalization round-trips for Elliptical Bass Cutoff [40.0 ..= 300.0 Hz]
        let bass_test = 145.0;
        let bass_norm = DynamicStereoWidthView::elliptical_bass_to_normalized(bass_test);
        let bass_back = DynamicStereoWidthView::normalized_to_elliptical_bass(bass_norm);
        assert!((bass_back - bass_test).abs() < 1e-3);

        // Normalization round-trips for Side Width Ratio [0.0 ..= 2.5]
        let width_test = 1.75;
        let width_norm = DynamicStereoWidthView::side_width_to_normalized(width_test);
        let width_back = DynamicStereoWidthView::normalized_to_side_width(width_norm);
        assert!((width_back - width_test).abs() < 1e-3);

        // Verify band widths and side levels
        view.set_profile(StereoWidthProfile::BroadcastMasteringClean);
        assert_eq!(view.band_widths[0], 0.0); // Sub collapsed to mono because bass cutoff > 60Hz
        assert!(view.band_widths[1] > 0.0);
        assert!(view.band_widths[2] > 0.0);
        assert!(view.band_widths[3] > 0.0);
        assert!(view.phase_correlation > 0.0);
        assert!(view.mono_compatibility > 0.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(50.0, 50.0, 400.0, 300.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_stereo_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_stereo_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_stereo_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_stereo_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING DYNAMIC STEREO WIDTH & BASS MONO HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn105_gui_multiband_decompressor_and_stereo_width_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Multiband De-Compressor Modal Lifecycle
        assert!(!view.is_multiband_decompressor_hud_open());
        view.open_multiband_decompressor_hud();
        assert!(view.is_multiband_decompressor_hud_open());
        view.close_multiband_decompressor_hud();
        assert!(!view.is_multiband_decompressor_hud_open());

        // 2. Dynamic Stereo Width Modal Lifecycle
        assert!(!view.is_dynamic_stereo_width_hud_open());
        view.open_dynamic_stereo_width_hud();
        assert!(view.is_dynamic_stereo_width_hud_open());
        view.close_dynamic_stereo_width_hud();
        assert!(!view.is_dynamic_stereo_width_hud_open());

        // 3. Multiband De-Compressor Parameter Sync Bridge
        view.multiband_decompressor_view.threshold_db[view.multiband_decompressor_view.selected_band] = -21.0;
        view.multiband_decompressor_view.ratio[view.multiband_decompressor_view.selected_band] = 2.4;
        view.multiband_decompressor_view.range_db[view.multiband_decompressor_view.selected_band] = 7.5;
        view.sync_multiband_decompressor_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("threshold_db").copied(),
            Some(-21.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("ratio").copied(),
            Some(2.4)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("range_db").copied(),
            Some(7.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("threshold").copied(),
            Some(-21.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("ratio").copied(),
            Some(2.4)
        );

        // 4. Dynamic Stereo Width Parameter Sync Bridge
        view.dynamic_stereo_width_view.elliptical_bass_hz = 150.0;
        view.dynamic_stereo_width_view.side_width_ratio = 1.65;
        view.sync_dynamic_stereo_width_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("elliptical_bass_hz").copied(),
            Some(150.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("side_width_ratio").copied(),
            Some(1.65)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("bass_cutoff_hz").copied(),
            Some(150.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("width").copied(),
            Some(1.65)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_multiband_decompressor_modal_window(ui);
                view.show_dynamic_stereo_width_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let actions = palette.available_actions();
        let decomp_action = actions.iter().find(|a| a.action_id == "open_multiband_decompressor_hud");
        assert!(decomp_action.is_some(), "open_multiband_decompressor_hud should be registered");
        assert_eq!(decomp_action.unwrap().category, "Mastering & Dynamics");

        let stereo_action = actions.iter().find(|a| a.action_id == "open_dynamic_stereo_width_hud");
        assert!(stereo_action.is_some(), "open_dynamic_stereo_width_hud should be registered");
        assert_eq!(stereo_action.unwrap().category, "Spatial & Surround");
    }
}
