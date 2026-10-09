#![allow(clippy::all)]

//! Test suite for Turn #116 (Sprint Turn #42) — Japanese Satsuma Biwa Silk String & Sawari
//! Buzzing Bridge HUD and Discrete Schottky Diode Multi-Stage Wavefolder & Overdrive HUD
//! Convergence (Milestones 11, 18, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::biwa_sawari_view::{
        BiwaSawariPreset, BiwaSawariView, BIWA_PUCK_HIT_RADIUS, NUM_SAWARI_HARMONICS,
    };
    use crate::views::schottky_wavefolder_view::{
        SchottkyWavefolderPreset, SchottkyWavefolderView, SCHOTTKY_PUCK_HIT_RADIUS,
        NUM_WAVEFOLDER_HARMONICS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn116_pure_biwa_sawari_initialization_and_physics() {
        let mut view = BiwaSawariView::new();
        assert_eq!(view.preset, BiwaSawariPreset::SatsumaHeikeEpic);
        assert_eq!(BIWA_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (BiwaSawariPreset::SatsumaHeikeEpic, "Satsuma Biwa (Heike Epic Recitation)", 0.85, 0.40, 60.0, 0.85, 2.2, 0.65),
            (BiwaSawariPreset::ChikuzenDelicateLyric, "Chikuzen Biwa (Delicate Lyrical Silk)", 0.45, 0.18, 120.0, 0.70, 3.5, 0.25),
            (BiwaSawariPreset::GagakuCourtBiwa, "Gaku-Biwa (Imperial Court Orchestra)", 0.60, 0.08, 15.0, 0.60, 1.2, 0.35),
            (BiwaSawariPreset::ModernExperimentalPercussive, "Modern Biwa (Percussive Body Slap)", 0.95, 0.85, 350.0, 0.92, 0.8, 0.90),
        ];

        for (p, exp_name, exp_vel, exp_gap, exp_bend, exp_mul, exp_damp, exp_slap) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (vel, gap, bend, mul, damp, slap) = p.nominal_parameters();
            assert_eq!(vel, exp_vel);
            assert_eq!(gap, exp_gap);
            assert_eq!(bend, exp_bend);
            assert_eq!(mul, exp_mul);
            assert_eq!(damp, exp_damp);
            assert_eq!(slap, exp_slap);
            assert_eq!(view.bachi_plectrum_velocity, exp_vel);
            assert_eq!(view.sawari_buzz_clearance, exp_gap);
            assert_eq!(view.silk_string_tension_bend, exp_bend);
            assert_eq!(view.mulberry_casing_resonance, exp_mul);
            assert_eq!(view.fret_contact_damping, exp_damp);
            assert_eq!(view.slap_soundboard_transient, exp_slap);
            assert!(view.string_displacement_peak_mm > 0.0);
            assert!(view.sawari_buzz_intensity >= 0.0 && view.sawari_buzz_intensity <= 1.0);
        }

        // Normalization round-trips for Bachi Velocity [0.0 ..= 1.0]
        let vel_test = 0.62;
        let vel_norm = BiwaSawariView::bachi_velocity_to_normalized(vel_test);
        let vel_back = BiwaSawariView::normalized_to_bachi_velocity(vel_norm);
        assert!((vel_back - vel_test).abs() < 1e-4);

        // Normalization round-trips for Sawari Clearance [0.01 ..= 2.0 mm]
        let gap_test = 0.55;
        let gap_norm = BiwaSawariView::sawari_clearance_to_normalized(gap_test);
        let gap_back = BiwaSawariView::normalized_to_sawari_clearance(gap_norm);
        assert!((gap_back - gap_test).abs() < 1e-4);

        // String displacement waveform evaluation across phase [0.0 .. 1.0]
        let disp_mid = view.evaluate_string_displacement(0.25);
        let disp_barrier = view.evaluate_string_displacement(0.75);
        assert!(disp_mid > 0.0);
        assert!(disp_barrier >= -view.sawari_buzz_clearance - 0.5); // Constrained by sawari ridge

        // Sawari harmonic buzzing cascade across 8 harmonics
        let harmonics = view.evaluate_sawari_harmonics();
        assert_eq!(harmonics.len(), NUM_SAWARI_HARMONICS);
        for &h in &harmonics {
            assert!(h > 0.0 && h <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding target)
        let canvas = Rect::new(20.0, 20.0, 340.0, 200.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_biwa_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_biwa_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_biwa_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_biwa_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("JAPANESE SATSUMA BIWA SILK STRING & SAWARI BUZZING BRIDGE HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn116_pure_schottky_wavefolder_initialization_and_physics() {
        let mut view = SchottkyWavefolderView::new();
        assert_eq!(view.preset, SchottkyWavefolderPreset::BuchlaWestCoastHarmonics);
        assert_eq!(SCHOTTKY_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Preset profiles and expected values
        let presets = [
            (SchottkyWavefolderPreset::SubtleEvenFoldWarmth, "Subtle Even Fold Warmth (2-Stage Diode)", 10.0, 0.15, 2.0, 120.0, 0.50, -2.0),
            (SchottkyWavefolderPreset::BuchlaWestCoastHarmonics, "Buchla West-Coast Harmonics (3-Stage Classic)", 18.0, 0.0, 3.0, 80.0, 0.35, -4.0),
            (SchottkyWavefolderPreset::ScreamingDiodeCascade, "Screaming Diode Cascade (5-Stage Extreme)", 28.0, 0.05, 5.0, 220.0, 0.20, -7.5),
            (SchottkyWavefolderPreset::AsymmetricMetallicCrunch, "Asymmetric Metallic Crunch (Industrial)", 22.0, -0.45, 4.0, 450.0, 0.40, -5.0),
        ];

        for (p, exp_name, exp_drv, exp_bias, exp_stg, exp_cap, exp_knee, exp_trim) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (drv, bias, stg, cap, knee, trim) = p.nominal_parameters();
            assert_eq!(drv, exp_drv);
            assert_eq!(bias, exp_bias);
            assert_eq!(stg, exp_stg);
            assert_eq!(cap, exp_cap);
            assert_eq!(knee, exp_knee);
            assert_eq!(trim, exp_trim);
            assert_eq!(view.input_drive_gain, exp_drv);
            assert_eq!(view.fold_symmetry_bias, exp_bias);
            assert_eq!(view.folding_stages_count, exp_stg);
            assert_eq!(view.miller_junction_capacitance, exp_cap);
            assert_eq!(view.diode_saturation_softness, exp_knee);
            assert_eq!(view.output_trim_makeup, exp_trim);
            assert!(view.effective_fold_count >= 1);
            assert!(view.miller_cutoff_khz > 0.0);
            assert!(view.total_harmonic_density > 0.0);
        }

        // Normalization round-trips for Pre-Amp Drive [0.0 ..= 36.0 dB]
        let drv_test = 21.5;
        let drv_norm = SchottkyWavefolderView::drive_to_normalized(drv_test);
        let drv_back = SchottkyWavefolderView::normalized_to_drive(drv_norm);
        assert!((drv_back - drv_test).abs() < 1e-4);

        // Normalization round-trips for Symmetry Bias [-1.0 ..= 1.0]
        let bias_test = -0.35;
        let bias_norm = SchottkyWavefolderView::bias_to_normalized(bias_test);
        let bias_back = SchottkyWavefolderView::normalized_to_bias(bias_norm);
        assert!((bias_back - bias_test).abs() < 1e-4);

        // Transfer curve evaluation across Vin [-2.0 ..= +2.0 V]
        let v_zero = view.evaluate_transfer_curve(0.0);
        let v_pos = view.evaluate_transfer_curve(1.0);
        let v_neg = view.evaluate_transfer_curve(-1.0);
        assert!(v_zero.abs() <= 1.8);
        assert!(v_pos.abs() <= 1.8);
        assert!(v_neg.abs() <= 1.8);

        // Harmonic breakdown evaluation (H1..H9)
        let harmonics = view.evaluate_harmonics();
        assert_eq!(harmonics.len(), NUM_WAVEFOLDER_HARMONICS);
        assert_eq!(harmonics[0], 1.0); // Fundamental H1 normalized
        for &h in &harmonics {
            assert!(h > 0.0 && h <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 340.0, 200.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_schottky_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_schottky_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_schottky_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_schottky_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DISCRETE SCHOTTKY DIODE MULTI-STAGE WAVEFOLDER & OVERDRIVE HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn116_gui_biwa_and_schottky_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Biwa Sawari Modal Lifecycle
        assert!(!view.is_biwa_sawari_hud_open());
        view.open_biwa_sawari_hud();
        assert!(view.is_biwa_sawari_hud_open());
        view.close_biwa_sawari_hud();
        assert!(!view.is_biwa_sawari_hud_open());

        // 2. Schottky Wavefolder Modal Lifecycle
        assert!(!view.is_schottky_wavefolder_hud_open());
        view.open_schottky_wavefolder_hud();
        assert!(view.is_schottky_wavefolder_hud_open());
        view.close_schottky_wavefolder_hud();
        assert!(!view.is_schottky_wavefolder_hud_open());

        // 3. Biwa Sawari Parameter Sync Bridge
        view.biwa_sawari_view.bachi_plectrum_velocity = 0.90;
        view.biwa_sawari_view.sawari_buzz_clearance = 0.50;
        view.biwa_sawari_view.silk_string_tension_bend = 80.0;
        view.biwa_sawari_view.mulberry_casing_resonance = 0.82;
        view.sync_biwa_sawari_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("bachi_plectrum_velocity").copied(),
            Some(0.90)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("sawari_buzz_clearance").copied(),
            Some(0.50)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("bachi_plectrum_velocity").copied(),
            Some(0.90)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("sawari_buzz_clearance").copied(),
            Some(0.50)
        );

        // 4. Schottky Wavefolder Parameter Sync Bridge
        view.schottky_wavefolder_view.input_drive_gain = 24.0;
        view.schottky_wavefolder_view.fold_symmetry_bias = -0.25;
        view.schottky_wavefolder_view.folding_stages_count = 4.0;
        view.schottky_wavefolder_view.output_trim_makeup = -6.0;
        view.sync_schottky_wavefolder_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("input_drive_gain").copied(),
            Some(24.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("fold_symmetry_bias").copied(),
            Some(-0.25)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("input_drive_gain").copied(),
            Some(24.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("fold_symmetry_bias").copied(),
            Some(-0.25)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_biwa_sawari_modal_window(ui);
                view.show_schottky_wavefolder_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let biwa_action = palette.actions.iter().find(|a| a.action_id == "open_biwa_sawari_hud");
        assert!(biwa_action.is_some(), "open_biwa_sawari_hud should be registered");
        assert_eq!(biwa_action.unwrap().category, "Physical Modeling");

        let wavefold_action = palette.actions.iter().find(|a| a.action_id == "open_schottky_wavefolder_hud");
        assert!(wavefold_action.is_some(), "open_schottky_wavefolder_hud should be registered");
        assert_eq!(wavefold_action.unwrap().category, "Dynamics & Saturation");
    }
}
