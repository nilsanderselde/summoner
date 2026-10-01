#![allow(clippy::all)]

//! Test suite for Turn #54 (Turn #20) — Physical Modeling Bowed String & Shakuhachi Flute HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };
    use crate::views::bowed_string_view::{
        BowedStringView, StringMaterial, MIN_BOW_SPEED_MPS, MAX_BOW_SPEED_MPS, MIN_BOW_FORCE_N, MAX_BOW_FORCE_N,
    };
    use crate::views::shakuhachi_view::{
        ShakuhachiView, ShakuhachiLengthType, MIN_JET_VELOCITY_MPS, MAX_JET_VELOCITY_MPS,
    };

    #[test]
    fn test_turn54_pure_bowed_string_materials_and_parameters() {
        let view = BowedStringView::new();
        assert_eq!(view.material, StringMaterial::SyntheticCore);
        assert!((view.bow_speed_mps - 0.45).abs() < 1e-4);
        assert!((view.bow_force_n - 1.25).abs() < 1e-4);
        assert!((view.bridge_proximity_beta - 0.12).abs() < 1e-4);

        // Test StringMaterial densities and friction
        let (mu_s, mu_d) = StringMaterial::SteelCore.friction_coefficients();
        assert!(mu_s > mu_d);
        assert!(StringMaterial::TungstenHeavy.linear_mass_density_g_m() > StringMaterial::GutCore.linear_mass_density_g_m());
        assert_eq!(MIN_BOW_SPEED_MPS, 0.01);
        assert_eq!(MAX_BOW_SPEED_MPS, 2.0);
        assert_eq!(MIN_BOW_FORCE_N, 0.05);
        assert_eq!(MAX_BOW_FORCE_N, 5.0);

        // Test coordinate normalization roundtrips
        let norm_spd = BowedStringView::speed_to_normalized(1.0);
        let recon_spd = BowedStringView::normalized_to_speed(norm_spd);
        assert!((recon_spd - 1.0).abs() < 1e-3);

        let norm_frc = BowedStringView::force_to_normalized(2.5);
        let recon_frc = BowedStringView::normalized_to_force(norm_frc);
        assert!((recon_frc - 2.5).abs() < 1e-3);

        // Schelleng limits test
        let (f_min, f_max) = view.schelleng_limits();
        assert!(f_min > 0.0);
        assert!(f_max > f_min);
    }

    #[test]
    fn test_turn54_pure_shakuhachi_flute_lengths_and_acoustics() {
        let mut view = ShakuhachiView::new();
        assert_eq!(view.length_type, ShakuhachiLengthType::IchishakuHassun);
        assert!((view.jet_velocity_mps - 19.0).abs() < 1e-4);
        assert!((view.utaguchi_angle_deg - 38.0).abs() < 1e-4);

        // Length switching
        view.set_length_type(ShakuhachiLengthType::SanShakuKyotaku);
        assert_eq!(view.length_type, ShakuhachiLengthType::SanShakuKyotaku);
        assert!((view.length_type.nominal_length_cm() - 90.9).abs() < 1e-2);
        assert!((view.jet_velocity_mps - 11.0).abs() < 1e-3);
        assert_eq!(MIN_JET_VELOCITY_MPS, 4.0);
        assert_eq!(MAX_JET_VELOCITY_MPS, 42.0);

        // Normalization roundtrips
        let norm_v = ShakuhachiView::velocity_to_normalized(20.0);
        let recon_v = ShakuhachiView::normalized_to_velocity(norm_v);
        assert!((recon_v - 20.0).abs() < 1e-3);

        let norm_a = ShakuhachiView::angle_to_normalized(45.0);
        let recon_a = ShakuhachiView::normalized_to_angle(norm_a);
        assert!((recon_a - 45.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn54_pure_parambus_slot_arithmetic_and_channel_isolation() {
        let mut bus = ParamBus::new();

        // Track 2, Slot 0: Bowed String
        let pid_bow_spd = ParamId(2 * 1000 + 0 * 20 + 0);
        let pid_bow_frc = ParamId(2 * 1000 + 0 * 20 + 1);
        let pid_bow_prx = ParamId(2 * 1000 + 0 * 20 + 2);

        // Track 2, Slot 1: Shakuhachi
        let pid_shak_jet = ParamId(2 * 1000 + 1 * 20 + 0);
        let pid_shak_ang = ParamId(2 * 1000 + 1 * 20 + 1);
        let pid_shak_mer = ParamId(2 * 1000 + 1 * 20 + 2);

        bus.register(pid_bow_spd, 0.45);
        bus.register(pid_bow_frc, 1.25);
        bus.register(pid_bow_prx, 0.12);

        bus.register(pid_shak_jet, 19.0);
        bus.register(pid_shak_ang, 38.0);
        bus.register(pid_shak_mer, 0.0);

        // Mutate Bowed String parameters
        bus.set(pid_bow_spd, 0.85);
        bus.set(pid_bow_frc, 2.50);

        // Mutate Shakuhachi parameters
        bus.set(pid_shak_jet, 24.5);
        bus.set(pid_shak_mer, -120.0);

        assert_eq!(bus.get(pid_bow_spd), Some(0.85));
        assert_eq!(bus.get(pid_bow_frc), Some(2.50));
        assert_eq!(bus.get(pid_bow_prx), Some(0.12));

        assert_eq!(bus.get(pid_shak_jet), Some(24.5));
        assert_eq!(bus.get(pid_shak_ang), Some(38.0));
        assert_eq!(bus.get(pid_shak_mer), Some(-120.0));
    }

    #[test]
    fn test_turn54_pure_timeline_evaluation_physical_modeling_curves() {
        let mut timeline = AutomationTimeline::new();

        let lane_bow = "track_1_node_0_bow_speed_mps".to_string();
        let lane_shak = "track_1_node_1_jet_velocity_mps".to_string();

        timeline.lanes.insert(
            lane_bow.clone(),
            AutomationLane {
                param_id: lane_bow.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 0.20, interp: Interpolation::Linear },
                        AutomationPoint { beat: 4.0, value: 1.80, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        timeline.lanes.insert(
            lane_shak.clone(),
            AutomationLane {
                param_id: lane_shak.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 10.0, interp: Interpolation::Linear },
                        AutomationPoint { beat: 4.0, value: 30.0, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        assert!((timeline.evaluate(&lane_bow, 0.0).unwrap() - 0.20).abs() < 1e-4);
        assert!((timeline.evaluate(&lane_bow, 2.0).unwrap() - 1.00).abs() < 1e-4);
        assert!((timeline.evaluate(&lane_bow, 4.0).unwrap() - 1.80).abs() < 1e-4);

        assert!((timeline.evaluate(&lane_shak, 0.0).unwrap() - 10.0).abs() < 1e-4);
        assert!((timeline.evaluate(&lane_shak, 2.0).unwrap() - 20.0).abs() < 1e-4);
        assert!((timeline.evaluate(&lane_shak, 4.0).unwrap() - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_turn54_pure_ascii_snapshots() {
        let b_view = BowedStringView::new();
        let b_lines = b_view.render_ascii(60, 16);
        assert_eq!(b_lines.len(), 16);
        assert!(b_lines[0].starts_with('+'));
        assert!(b_lines.iter().any(|l| l.contains('~') || l.contains('O')));

        let s_view = ShakuhachiView::new();
        let s_lines = s_view.render_ascii(60, 16);
        assert_eq!(s_lines.len(), 16);
        assert!(s_lines[0].starts_with('+'));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use std::sync::Arc;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn54_gui_bowed_string_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_bowed_string_hud_open());

        view.open_bowed_string_hud();
        assert!(view.is_bowed_string_hud_open());

        view.close_bowed_string_hud();
        assert!(!view.is_bowed_string_hud_open());
    }

    #[test]
    fn test_turn54_gui_shakuhachi_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_shakuhachi_hud_open());

        view.open_shakuhachi_hud();
        assert!(view.is_shakuhachi_hud_open());

        view.close_shakuhachi_hud();
        assert!(!view.is_shakuhachi_hud_open());
    }

    #[test]
    fn test_turn54_gui_inspector_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.inspector_state.requested_open_bowed_string_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_bowed_string_hud_open());
        assert!(!view.inspector_state.requested_open_bowed_string_hud);
        view.close_bowed_string_hud();

        view.inspector_state.requested_open_shakuhachi_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_shakuhachi_hud_open());
        assert!(!view.inspector_state.requested_open_shakuhachi_hud);
    }

    #[test]
    fn test_turn54_gui_device_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_bowed_string_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_bowed_string_hud_open());
        assert!(!view.device_rack_state.requested_open_bowed_string_hud);
        view.close_bowed_string_hud();

        view.device_rack_state.requested_open_shakuhachi_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_shakuhachi_hud_open());
        assert!(!view.device_rack_state.requested_open_shakuhachi_hud);
    }

    #[test]
    fn test_turn54_gui_live_parambus_sync() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0; // Track 1
        let mut bus = ParamBus::new();

        let track_id = 1;
        let slot_idx = 0;
        let pid_spd = ParamId(track_id * 1000 + slot_idx * 20 + 0);
        let pid_frc = ParamId(track_id * 1000 + slot_idx * 20 + 1);

        bus.register(pid_spd, 0.1);
        bus.register(pid_frc, 0.1);

        let bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&bus));

        view.bowed_string_view.bow_speed_mps = 1.45;
        view.bowed_string_view.bow_force_n = 3.20;
        view.sync_bowed_string_hud_state();

        assert_eq!(bus.get(pid_spd), Some(1.45));
        assert_eq!(bus.get(pid_frc), Some(3.20));

        let pid_jet = ParamId(track_id * 1000 + slot_idx * 20 + 0);
        let pid_ang = ParamId(track_id * 1000 + slot_idx * 20 + 1);
        view.shakuhachi_view.jet_velocity_mps = 28.5;
        view.shakuhachi_view.utaguchi_angle_deg = 42.0;
        view.sync_shakuhachi_hud_state();

        assert_eq!(bus.get(pid_jet), Some(28.5));
        assert_eq!(bus.get(pid_ang), Some(42.0));
    }

    #[test]
    fn test_turn54_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let has_bowed = palette.actions.iter().any(|a| a.action_id == "open_bowed_string_hud");
        let has_shak = palette.actions.iter().any(|a| a.action_id == "open_shakuhachi_hud");

        assert!(has_bowed, "open_bowed_string_hud should be registered in CommandPalette");
        assert!(has_shak, "open_shakuhachi_hud should be registered in CommandPalette");
    }

    #[test]
    fn test_turn54_gui_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_bowed_string_hud();
        view.open_shakuhachi_hud();

        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(!output.shapes.is_empty(), "egui output shapes should be generated for open HUDs");
    }
}
