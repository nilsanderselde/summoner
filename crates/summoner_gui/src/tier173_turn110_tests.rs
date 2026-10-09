#![allow(clippy::all)]

//! Test suite for Turn #110 (Sprint Turn #36) — Broadcast Mastering ITU-R BS.1770-4
//! Dialog Gating & Speech Normalization HUD and Multiband Lookahead Linear-Phase
//! Transient Reconstructor & Crest Factor Expander HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::dialog_gating_view::{
        DialogGatingView, DialogLoudnessStandard, DIALOG_GATING_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_reconstructor_view::{
        TransientReconstructorView, ReconstructProfileType, RECONSTRUCT_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn110_pure_dialog_gating_initialization_and_kinematics() {
        let mut view = DialogGatingView::new();
        assert_eq!(view.standard, DialogLoudnessStandard::EbuR128);
        assert_eq!(DIALOG_GATING_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Standards and expected values
        let standards = [
            (DialogLoudnessStandard::EbuR128, "EBU R128 (-23 LUFS)", -23.0, -1.0),
            (DialogLoudnessStandard::AtscA85, "ATSC A/85 (-24 LKFS)", -24.0, -2.0),
            (DialogLoudnessStandard::NetflixOtt, "Netflix OTT (-27 LUFS)", -27.0, -2.0),
            (DialogLoudnessStandard::StreamingMusic, "Streaming Music (-14 LUFS)", -14.0, -1.0),
            (DialogLoudnessStandard::PodcastSpeech, "Podcast Speech (-16 LUFS)", -16.0, -1.0),
        ];

        for (std, expected_name, exp_target, exp_tp) in standards {
            view.standard = std;
            assert_eq!(std.name(), expected_name);
            assert_eq!(std.target_integrated_lufs(), exp_target);
            assert_eq!(std.true_peak_ceiling_dbtp(), exp_tp);
        }

        // Normalization round-trips for LUFS [-40.0 ..= -10.0]
        let lufs_test = -23.5;
        let lufs_norm = DialogGatingView::lufs_to_normalized(lufs_test);
        let lufs_back = DialogGatingView::normalized_to_lufs(lufs_norm);
        assert!((lufs_back - lufs_test).abs() < 1e-3);

        // Normalization round-trips for VAD [0.0 ..= 100.0 %]
        let vad_test = 68.5;
        let vad_norm = DialogGatingView::vad_to_normalized(vad_test);
        let vad_back = DialogGatingView::normalized_to_vad(vad_norm);
        assert!((vad_back - vad_test).abs() < 1e-3);

        // Update gating calculations
        view.gated_integrated_lkfs = -22.5;
        view.ungated_integrated_lkfs = -25.0;
        view.vad_speech_activity_pct = 75.0;
        view.update_gating_calculations();
        assert!((view.gating_delta_lu - 2.5).abs() < 1e-3);
        assert!(view.dialog_anchored_lkfs < 0.0);

        // K-weighting filter response
        let resp_1k = view.evaluate_k_weighting_response_db(1000.0);
        let resp_10k = view.evaluate_k_weighting_response_db(10000.0);
        assert!(resp_10k > resp_1k); // High shelf boost above 1.5 kHz

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.dialog_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.dialog_puck_pos.1) * canvas.height;

        assert!(view.hit_test_dialog_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_dialog_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_dialog_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_dialog_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("BROADCAST MASTERING ITU-R BS.1770-4 DIALOG GATING HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn110_pure_transient_reconstructor_initialization_and_kinematics() {
        let mut view = TransientReconstructorView::new();
        assert_eq!(view.profile, ReconstructProfileType::PunchyMaster);
        assert_eq!(RECONSTRUCT_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profiles and expected names
        let profiles = [
            (ReconstructProfileType::PunchyMaster, "PUNCHY MASTER RESTORATION", 4.0, 3.5, 0.65, 512),
            (ReconstructProfileType::MicroTransientRestorer, "MICRO-TRANSIENT RESTORER", 2.5, 2.0, 0.85, 256),
            (ReconstructProfileType::DrumStemExpander, "DRUM STEM CREST EXPANDER", 6.0, 5.5, 0.50, 512),
            (ReconstructProfileType::AcousticGuitarExciter, "ACOUSTIC GUITAR PLUCK EXCITER", 3.0, 2.5, 0.75, 256),
            (ReconstructProfileType::TransparentAirCeiling, "TRANSPARENT AIR CEILING", 5.0, 1.8, 0.60, 1024),
        ];

        for (p, expected_name, def_la, def_crest, def_sens, def_taps) in profiles {
            view.set_profile(p);
            assert_eq!(p.profile_name(), expected_name);
            assert_eq!(p.name(), expected_name);
            assert_eq!(p.nominal_lookahead_ms(), def_la);
            assert_eq!(p.nominal_crest_boost_db(), def_crest);
            assert_eq!(p.nominal_sensitivity(), def_sens);
            assert_eq!(p.nominal_fir_taps(), def_taps);
            assert_eq!(view.lookahead_ms, def_la);
            assert_eq!(view.crest_factor_boost_db, def_crest);
            assert_eq!(view.transient_sensitivity, def_sens);
            assert_eq!(view.fir_taps, def_taps);
        }

        // Normalization round-trips for Lookahead [0.5 ..= 15.0 ms]
        let la_test = 5.25;
        let la_norm = TransientReconstructorView::lookahead_to_normalized(la_test);
        let la_back = TransientReconstructorView::normalized_to_lookahead(la_norm);
        assert!((la_back - la_test).abs() < 1e-3);

        // Normalization round-trips for Crest Boost [0.0 ..= 12.0 dB]
        let crest_test = 4.75;
        let crest_norm = TransientReconstructorView::crest_to_normalized(crest_test);
        let crest_back = TransientReconstructorView::normalized_to_crest(crest_norm);
        assert!((crest_back - crest_test).abs() < 1e-3);

        // Normalization round-trips for Sensitivity [0.0 ..= 1.0]
        let sens_test = 0.82;
        let sens_norm = TransientReconstructorView::sensitivity_to_normalized(sens_test);
        let sens_back = TransientReconstructorView::normalized_to_sensitivity(sens_norm);
        assert!((sens_back - sens_test).abs() < 1e-3);

        // Simulation update & 4-band gain evaluation
        view.lookahead_ms = 4.5;
        view.crest_factor_boost_db = 4.0;
        view.transient_sensitivity = 0.70;
        view.update_reconstruction_simulation();
        assert_eq!(view.band_gains_db.len(), 4);
        for &gain in &view.band_gains_db {
            assert!(gain > 0.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_reconstruct_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_reconstruct_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_reconstruct_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_reconstruct_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTIBAND LINEAR-PHASE TRANSIENT RECONSTRUCTOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn110_gui_dialog_gating_and_reconstructor_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Dialog Gating Modal Lifecycle
        assert!(!view.is_dialog_gating_hud_open());
        view.open_dialog_gating_hud();
        assert!(view.is_dialog_gating_hud_open());
        view.close_dialog_gating_hud();
        assert!(!view.is_dialog_gating_hud_open());

        // 2. Transient Reconstructor Modal Lifecycle
        assert!(!view.is_transient_reconstructor_hud_open());
        view.open_transient_reconstructor_hud();
        assert!(view.is_transient_reconstructor_hud_open());
        view.close_transient_reconstructor_hud();
        assert!(!view.is_transient_reconstructor_hud_open());

        // 3. Dialog Gating Parameter Sync Bridge
        view.dialog_gating_view.gated_integrated_lkfs = -23.4;
        view.dialog_gating_view.vad_speech_activity_pct = 72.0;
        view.dialog_gating_view.dialog_anchored_lkfs = -23.2;
        view.dialog_gating_view.gating_delta_lu = 1.8;
        view.dialog_gating_view.true_peak_max_dbtp = -1.15;
        view.sync_dialog_gating_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("gated_integrated_lkfs").copied(),
            Some(-23.4)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("vad_speech_activity_pct").copied(),
            Some(72.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("dialog_anchored_lkfs").copied(),
            Some(-23.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("gated_integrated_lkfs").copied(),
            Some(-23.4)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("vad_speech_activity_pct").copied(),
            Some(72.0)
        );

        // 4. Transient Reconstructor Parameter Sync Bridge
        view.transient_reconstructor_view.lookahead_ms = 4.2;
        view.transient_reconstructor_view.crest_factor_boost_db = 4.5;
        view.transient_reconstructor_view.transient_sensitivity = 0.72;
        view.transient_reconstructor_view.fir_taps = 512;
        view.transient_reconstructor_view.envelope_hold_ms = 14.0;
        view.sync_transient_reconstructor_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("lookahead_ms").copied(),
            Some(4.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("crest_factor_boost_db").copied(),
            Some(4.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("transient_sensitivity").copied(),
            Some(0.72)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("fir_taps").copied(),
            Some(512.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("lookahead_ms").copied(),
            Some(4.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("crest_factor_boost_db").copied(),
            Some(4.5)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_dialog_gating_modal_window(ui);
                view.show_transient_reconstructor_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let dg_action = palette.actions.iter().find(|a| a.action_id == "open_dialog_gating_hud");
        assert!(dg_action.is_some(), "open_dialog_gating_hud should be registered");
        assert_eq!(dg_action.unwrap().category, "Mastering & Dynamics");

        let tr_action = palette.actions.iter().find(|a| a.action_id == "open_transient_reconstructor_hud");
        assert!(tr_action.is_some(), "open_transient_reconstructor_hud should be registered");
        assert_eq!(tr_action.unwrap().category, "Dynamics & Saturation");
    }
}
