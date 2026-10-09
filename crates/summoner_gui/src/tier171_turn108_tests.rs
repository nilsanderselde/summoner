#![allow(clippy::all)]

//! Test suite for Turn #108 (Sprint Turn #34) — Mastering Multi-Band Linear-Phase Dynamic
//! Transient Clipper & Oversampled Ceiling HUD and Mastering Multi-Stage Magnetic Tape Flux
//! Saturation & High-Frequency Hysteresis HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::transient_clipper_view::{
        TransientClipperView, ClipperProfileType, CLIPPER_PUCK_HIT_RADIUS,
    };
    use crate::views::tape_flux_master_view::{
        TapeFluxMasterView, TapeFormulation, TAPE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn108_pure_transient_clipper_initialization_and_kinematics() {
        let mut view = TransientClipperView::new();
        assert_eq!(view.profile_type, ClipperProfileType::LinearPhase4BandMaster);
        assert_eq!(CLIPPER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profiles and nominal values
        let profiles = [
            (ClipperProfileType::LinearPhase4BandMaster, "4-BAND LINEAR-PHASE MASTER CLIPPER", 6.0, 3.5, 8, -0.1),
            (ClipperProfileType::Oversampled16xPcmCeiling, "16x OVERSAMPLED TRUE-PEAK CEILING", 8.5, 1.5, 16, -0.3),
            (ClipperProfileType::TapeSoftSaturatingClipper, "TAPE SOFT-SATURATING KNEE", 4.5, 8.0, 4, -0.2),
            (ClipperProfileType::HardPunchEDMClipper, "HARD PUNCH BRICKWALL CLIPPER", 11.0, 0.0, 8, 0.0),
            (ClipperProfileType::BroadcastMultiStageClipper, "BROADCAST MULTI-STAGE PEAK CONTROL", 5.0, 4.0, 16, -0.5),
        ];

        for (p, expected_name, nom_drive, nom_knee, nom_os, nom_ceil) in profiles {
            view.set_profile(p);
            assert_eq!(view.profile_type, p);
            assert_eq!(p.profile_name(), expected_name);
            assert_eq!(p.name(), expected_name);
            assert_eq!(p.nominal_drive_db(), nom_drive);
            assert_eq!(p.nominal_knee_db(), nom_knee);
            assert_eq!(p.nominal_oversampling_factor(), nom_os);
            assert_eq!(p.nominal_ceiling_dbfs(), nom_ceil);
        }

        // Normalization round-trips for Clipping Drive [-6.0 ..= 18.0 dB]
        let drive_test = 7.5;
        let drive_norm = TransientClipperView::drive_to_normalized(drive_test);
        let drive_back = TransientClipperView::normalized_to_drive(drive_norm);
        assert!((drive_back - drive_test).abs() < 1e-3);

        // Normalization round-trips for Knee Softness [0.0 ..= 12.0 dB]
        let knee_test = 4.2;
        let knee_norm = TransientClipperView::knee_to_normalized(knee_test);
        let knee_back = TransientClipperView::normalized_to_knee(knee_norm);
        assert!((knee_back - knee_test).abs() < 1e-3);

        // Normalization round-trips for Ceiling [-12.0 ..= 0.0 dBFS]
        let ceil_test = -0.5;
        let ceil_norm = TransientClipperView::ceiling_to_normalized(ceil_test);
        let ceil_back = TransientClipperView::normalized_to_ceiling(ceil_norm);
        assert!((ceil_back - ceil_test).abs() < 1e-3);

        // Simulation update and transfer curve points
        view.set_profile(ClipperProfileType::LinearPhase4BandMaster);
        view.clip_drive_db = 9.0;
        view.knee_softness_db = 2.5;
        view.update_clipper_simulation();

        assert!(view.crest_factor_reduction_db >= 0.0);
        assert!(view.thd_distortion_pct >= 0.1);
        for pt in &view.transfer_curve_pts {
            assert!(pt.0 >= -1.0 && pt.0 <= 1.0);
            assert!(pt.1 >= -1.0 && pt.1 <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(30.0, 30.0, 380.0, 260.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_clipper_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_clipper_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_clipper_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_clipper_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING MULTI-BAND DYNAMIC TRANSIENT CLIPPER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn108_pure_tape_flux_master_initialization_and_kinematics() {
        let mut view = TapeFluxMasterView::new();
        assert_eq!(view.formulation, TapeFormulation::StuderA800MasterTape);
        assert_eq!(TAPE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Formulations and nominal values
        let formulations = [
            (TapeFormulation::Ampex456GrandMaster, "AMPEX 456 GRAND MASTER (+6)", 6.0, 1.5, 15.0, 55.0, 0.85),
            (TapeFormulation::StuderA800MasterTape, "STUDER A800 MASTER (+9)", 9.0, 3.0, 30.0, 42.0, 0.25),
            (TapeFormulation::QuantegyGP9, "QUANTEGY GP9 (+9 MAX)", 11.5, 4.5, 30.0, 38.0, 0.35),
            (TapeFormulation::CassetteTypeIVMetal, "CASSETTE TYPE IV METAL", 3.5, 0.5, 3.75, 95.0, 1.95),
            (TapeFormulation::VintageTubeTape1958, "VINTAGE TUBE TAPE (1958)", 4.0, -1.5, 15.0, 68.0, 2.40),
        ];

        for (f, expected_name, nom_drive, nom_bias, nom_speed, nom_bump, nom_thd) in formulations {
            view.set_formulation(f);
            assert_eq!(view.formulation, f);
            assert_eq!(f.formulation_name(), expected_name);
            assert_eq!(f.name(), expected_name);
            assert_eq!(f.nominal_flux_drive_db(), nom_drive);
            assert_eq!(f.nominal_bias_trim_db(), nom_bias);
            assert_eq!(f.nominal_ips_speed(), nom_speed);
            assert_eq!(f.nominal_head_bump_hz(), nom_bump);
            assert_eq!(f.nominal_thd_pct(), nom_thd);
        }

        // Normalization round-trips for Flux Drive [-6.0 ..= 18.0 dB]
        let drive_test = 8.5;
        let drive_norm = TapeFluxMasterView::drive_to_normalized(drive_test);
        let drive_back = TapeFluxMasterView::normalized_to_drive(drive_norm);
        assert!((drive_back - drive_test).abs() < 1e-3);

        // Normalization round-trips for Bias Trim [-6.0 ..= 6.0 dB]
        let bias_test = 2.4;
        let bias_norm = TapeFluxMasterView::bias_to_normalized(bias_test);
        let bias_back = TapeFluxMasterView::normalized_to_bias(bias_norm);
        assert!((bias_back - bias_test).abs() < 1e-3);

        // Normalization round-trips for Tape Speed [3.75 ..= 30.0 ips]
        let speed_test = 15.0;
        let speed_norm = TapeFluxMasterView::speed_to_normalized(speed_test);
        let speed_back = TapeFluxMasterView::normalized_to_speed(speed_norm);
        assert!((speed_back - speed_test).abs() < 1e-3);

        // Simulation update and hysteresis loop points
        view.set_formulation(TapeFormulation::StuderA800MasterTape);
        view.flux_drive_db = 10.0;
        view.bias_trim_db = 2.0;
        view.update_hysteresis_simulation();

        assert!(view.thd_distortion_pct > 0.05);
        assert!(view.hf_compression_pct >= 5.0 && view.hf_compression_pct <= 95.0);
        for pt in &view.hysteresis_loop_pts {
            assert!(pt.0 >= -1.0 && pt.0 <= 1.0);
            assert!(pt.1 >= -1.0 && pt.1 <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(30.0, 30.0, 380.0, 260.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_tape_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_tape_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_tape_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_tape_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING MAGNETIC TAPE FLUX SATURATION & HYSTERESIS HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn108_gui_transient_clipper_and_tape_flux_master_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Transient Clipper Modal Lifecycle
        assert!(!view.is_transient_clipper_hud_open());
        view.open_transient_clipper_hud();
        assert!(view.is_transient_clipper_hud_open());
        view.close_transient_clipper_hud();
        assert!(!view.is_transient_clipper_hud_open());

        // 2. Tape Flux Master Modal Lifecycle
        assert!(!view.is_tape_flux_master_hud_open());
        view.open_tape_flux_master_hud();
        assert!(view.is_tape_flux_master_hud_open());
        view.close_tape_flux_master_hud();
        assert!(!view.is_tape_flux_master_hud_open());

        // 3. Transient Clipper Parameter Sync Bridge
        view.transient_clipper_view.clip_drive_db = 7.5;
        view.transient_clipper_view.knee_softness_db = 4.0;
        view.transient_clipper_view.ceiling_dbfs = -0.2;
        view.transient_clipper_view.crest_factor_reduction_db = 3.8;
        view.transient_clipper_view.oversampling_factor = 16;
        view.sync_transient_clipper_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("clip_drive_db").copied(),
            Some(7.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("knee_softness_db").copied(),
            Some(4.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("ceiling_dbfs").copied(),
            Some(-0.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("oversampling_order").copied(),
            Some(16.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("threshold_db").copied(),
            Some(7.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("clip_hardness").copied(),
            Some(4.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("ceiling_db").copied(),
            Some(-0.2)
        );

        // 4. Tape Flux Master Parameter Sync Bridge
        view.tape_flux_master_view.flux_drive_db = 10.5;
        view.tape_flux_master_view.bias_trim_db = 2.2;
        view.tape_flux_master_view.tape_speed_ips = 30.0;
        view.tape_flux_master_view.thd_distortion_pct = 0.45;
        view.tape_flux_master_view.hf_compression_pct = 40.0;
        view.sync_tape_flux_master_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("flux_drive_db").copied(),
            Some(10.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("bias_trim_db").copied(),
            Some(2.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("tape_speed_ips").copied(),
            Some(30.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("tape_flux_nanoweber").copied(),
            Some(10.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("bias_hf_saturation").copied(),
            Some(2.2)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_transient_clipper_modal_window(ui);
                view.show_tape_flux_master_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let clip_action = palette.actions.iter().find(|a| a.action_id == "open_transient_clipper_hud");
        assert!(clip_action.is_some(), "open_transient_clipper_hud should be registered");
        assert_eq!(clip_action.unwrap().category, "Mastering & Dynamics");

        let tape_action = palette.actions.iter().find(|a| a.action_id == "open_tape_flux_master_hud");
        assert!(tape_action.is_some(), "open_tape_flux_master_hud should be registered");
        assert_eq!(tape_action.unwrap().category, "Dynamics & Saturation");
    }
}
