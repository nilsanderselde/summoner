#![allow(clippy::all)]

//! Test suite for Turn #107 (Sprint Turn #33 — Sprint Review) — Mastering Multi-Bus Parallel Dynamic
//! Transient/Body Saturator & Harmonic Coloration HUD and Mastering Linear-Phase Stereo Mid/Side
//! Dynamic Punch & Multiband Mono-Bass Focuser HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::parallel_transient_saturator_view::{
        ParallelTransientSaturatorView, SaturatorMode, SATURATOR_PUCK_HIT_RADIUS,
    };
    use crate::views::midside_focuser_view::{
        MidSideFocuserView, MidSideProfileType, MIDSIDE_FOCUSER_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn107_pure_parallel_transient_saturator_initialization_and_kinematics() {
        let mut view = ParallelTransientSaturatorView::new();
        assert_eq!(view.saturator_mode, SaturatorMode::TriodeTubeEvenOdd);
        assert_eq!(SATURATOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Modes and nominal values
        let modes = [
            (SaturatorMode::TriodeTubeEvenOdd, "TRIODE TUBE"),
            (SaturatorMode::TapeHysteresisFlux, "TAPE FLUX"),
            (SaturatorMode::FetTransientPunch, "FET PUNCH"),
            (SaturatorMode::GermaniumDiodeGrit, "GERMANIUM DIODE"),
            (SaturatorMode::CleanLinearDynamics, "CLEAN LINEAR"),
        ];

        for (m, expected_name) in modes {
            view.set_saturator_mode(m);
            assert_eq!(view.saturator_mode, m);
            assert_eq!(m.mode_name(), expected_name);
            assert_eq!(m.name(), expected_name);
            assert!(m.nominal_drive_db() >= 0.0);
            assert!(m.nominal_blend() >= 0.0 && m.nominal_blend() <= 1.0);
        }

        // Normalization round-trips for Blend [0.0 ..= 1.0]
        let blend_test = 0.72;
        let blend_norm = ParallelTransientSaturatorView::blend_to_normalized(blend_test);
        let blend_back = ParallelTransientSaturatorView::normalized_to_blend(blend_norm);
        assert!((blend_back - blend_test).abs() < 1e-3);

        // Normalization round-trips for Transient Drive [0.0 ..= 24.0 dB]
        let drive_test = 14.8;
        let drive_norm = ParallelTransientSaturatorView::drive_to_normalized(drive_test);
        let drive_back = ParallelTransientSaturatorView::normalized_to_drive(drive_norm);
        assert!((drive_back - drive_test).abs() < 1e-3);

        // Normalization round-trips for Body Sustain Drive [0.0 ..= 24.0 dB]
        let body_test = 8.4;
        let body_norm = ParallelTransientSaturatorView::body_drive_to_normalized(body_test);
        let body_back = ParallelTransientSaturatorView::normalized_to_body_drive(body_norm);
        assert!((body_back - body_test).abs() < 1e-3);

        // Simulation update and harmonic profile
        view.set_saturator_mode(SaturatorMode::TriodeTubeEvenOdd);
        view.transient_drive_db = 12.0;
        view.blend_ratio = 0.65;
        view.update_saturation_simulation();
        assert!(view.thd_percent > 0.05);
        assert!(view.crest_factor_db >= 3.0 && view.crest_factor_db <= 20.0);
        assert_eq!(view.harmonic_profile[0], 1.0); // Fundamental
        for &h in &view.harmonic_profile[1..] {
            assert!(h >= 0.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(40.0, 40.0, 400.0, 300.0);
        let puck_x = canvas.x + view.saturation_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.saturation_puck_pos.1) * canvas.height;

        assert!(view.hit_test_saturator_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_saturator_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_saturator_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_saturator_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING PARALLEL TRANSIENT / BODY SATURATOR HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn107_pure_midside_focuser_initialization_and_kinematics() {
        let mut view = MidSideFocuserView::new();
        assert_eq!(view.profile, MidSideProfileType::ClubSubTightener);
        assert_eq!(MIDSIDE_FOCUSER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profiles and nominal values
        let profiles = [
            (MidSideProfileType::ClubSubTightener, "CLUB SUB TIGHTENER (120Hz)", 120.0, 4.0, 1.35, 512),
            (MidSideProfileType::VinylMasterMonofier, "VINYL MASTER ELLIPTICAL MONO", 150.0, 2.0, 0.95, 1024),
            (MidSideProfileType::AcousticLiveFocus, "ACOUSTIC LIVE MID-FOCUS", 80.0, 1.5, 1.10, 256),
            (MidSideProfileType::DynamicEdmSlam, "DYNAMIC EDM STEREO SLAM", 140.0, 6.5, 1.60, 512),
            (MidSideProfileType::BroadcastClaritySafe, "BROADCAST CLARITY SAFE", 100.0, 2.5, 1.00, 512),
        ];

        for (p, expected_name, exp_cutoff, exp_punch, exp_width, exp_taps) in profiles {
            view.set_profile(p);
            assert_eq!(view.profile, p);
            assert_eq!(p.profile_name(), expected_name);
            assert_eq!(p.name(), expected_name);
            assert_eq!(p.nominal_mono_cutoff_hz(), exp_cutoff);
            assert_eq!(p.nominal_mid_punch_db(), exp_punch);
            assert_eq!(p.nominal_side_width(), exp_width);
            assert_eq!(p.nominal_fir_taps(), exp_taps);
        }

        // Normalization round-trips for Mono Cutoff [20.0 ..= 300.0 Hz]
        let cutoff_test = 135.0;
        let cutoff_norm = MidSideFocuserView::cutoff_to_normalized(cutoff_test);
        let cutoff_back = MidSideFocuserView::normalized_to_cutoff(cutoff_norm);
        assert!((cutoff_back - cutoff_test).abs() < 1e-1);

        // Normalization round-trips for Mid Punch Boost [0.0 ..= 12.0 dB]
        let punch_test = 5.25;
        let punch_norm = MidSideFocuserView::punch_to_normalized(punch_test);
        let punch_back = MidSideFocuserView::normalized_to_punch(punch_norm);
        assert!((punch_back - punch_test).abs() < 1e-3);

        // Normalization round-trips for Side Width Ratio [0.0 ..= 2.0]
        let width_test = 1.45;
        let width_norm = MidSideFocuserView::width_to_normalized(width_test);
        let width_back = MidSideFocuserView::normalized_to_width(width_norm);
        assert!((width_back - width_test).abs() < 1e-3);

        // Simulation update and 4-band Mid/Side energy distribution
        view.set_profile(MidSideProfileType::DynamicEdmSlam);
        view.update_midside_simulation();
        assert!(view.stereo_correlation >= -1.0 && view.stereo_correlation <= 1.0);
        for &energy in &view.band_energies {
            assert!(energy >= 0.1 && energy <= 1.5);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(50.0, 50.0, 400.0, 300.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_focuser_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_focuser_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_focuser_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_focuser_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING STEREO MID/SIDE DYNAMIC PUNCH & MONO-BASS HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn107_gui_parallel_saturator_and_midside_focuser_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Parallel Transient Saturator Modal Lifecycle
        assert!(!view.is_parallel_transient_saturator_hud_open());
        view.open_parallel_transient_saturator_hud();
        assert!(view.is_parallel_transient_saturator_hud_open());
        view.close_parallel_transient_saturator_hud();
        assert!(!view.is_parallel_transient_saturator_hud_open());

        // 2. Mid/Side Focuser Modal Lifecycle
        assert!(!view.is_midside_focuser_hud_open());
        view.open_midside_focuser_hud();
        assert!(view.is_midside_focuser_hud_open());
        view.close_midside_focuser_hud();
        assert!(!view.is_midside_focuser_hud_open());

        // 3. Parallel Transient Saturator Parameter Sync Bridge
        view.parallel_transient_saturator_view.blend_ratio = 0.68;
        view.parallel_transient_saturator_view.transient_drive_db = 9.5;
        view.parallel_transient_saturator_view.body_sustain_drive_db = 5.2;
        view.parallel_transient_saturator_view.thd_percent = 2.45;
        view.parallel_transient_saturator_view.crest_factor_db = 11.2;
        view.sync_parallel_transient_saturator_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("blend_ratio").copied(),
            Some(0.68)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("transient_drive_db").copied(),
            Some(9.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("body_sustain_drive_db").copied(),
            Some(5.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("blend").copied(),
            Some(0.68)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("drive").copied(),
            Some(9.5)
        );

        // 4. Mid/Side Focuser Parameter Sync Bridge
        view.midside_focuser_view.mono_cutoff_hz = 145.0;
        view.midside_focuser_view.mid_punch_boost_db = 6.2;
        view.midside_focuser_view.side_width_ratio = 1.55;
        view.midside_focuser_view.stereo_correlation = 0.88;
        view.sync_midside_focuser_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("mono_cutoff_hz").copied(),
            Some(145.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("mid_punch_boost_db").copied(),
            Some(6.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("side_width_ratio").copied(),
            Some(1.55)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("cutoff_hz").copied(),
            Some(145.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("mid_punch").copied(),
            Some(6.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("width").copied(),
            Some(1.55)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_parallel_transient_saturator_modal_window(ui);
                view.show_midside_focuser_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let sat_action = palette.actions.iter().find(|a| a.action_id == "open_parallel_transient_saturator_hud");
        assert!(sat_action.is_some(), "open_parallel_transient_saturator_hud should be registered");
        assert_eq!(sat_action.unwrap().category, "Dynamics & Saturation");

        let ms_action = palette.actions.iter().find(|a| a.action_id == "open_midside_focuser_hud");
        assert!(ms_action.is_some(), "open_midside_focuser_hud should be registered");
        assert_eq!(ms_action.unwrap().category, "Spatial & Surround");
    }
}
