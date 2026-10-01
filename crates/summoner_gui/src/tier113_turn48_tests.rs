#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn48_pure_hurdy_gurdy_and_trompette_slot_arithmetic() {
        let mut bus = ParamBus::new();

        // Track 1, Slot 1 (Hurdy-Gurdy)
        let t1_s1_speed = ParamId(1 * 1000 + 1 * 20 + 0);
        let t1_s1_pulse = ParamId(1 * 1000 + 1 * 20 + 1);
        let t1_s1_press = ParamId(1 * 1000 + 1 * 20 + 2);
        let t1_s1_clear = ParamId(1 * 1000 + 1 * 20 + 3);
        let t1_s1_drone = ParamId(1 * 1000 + 1 * 20 + 4);
        let t1_s1_buzz = ParamId(1 * 1000 + 1 * 20 + 5);

        // Track 2, Slot 0 (Trompette Chien Bridge)
        let t2_s0_gap = ParamId(2 * 1000 + 0 * 20 + 0);
        let t2_s0_force = ParamId(2 * 1000 + 0 * 20 + 1);
        let t2_s0_buzz = ParamId(2 * 1000 + 0 * 20 + 2);

        bus.register(t1_s1_speed, 6.28);
        bus.register(t1_s1_pulse, 1.20);
        bus.register(t1_s1_press, 0.70);
        bus.register(t1_s1_clear, 0.35);
        bus.register(t1_s1_drone, 0.50);
        bus.register(t1_s1_buzz, 0.85);

        bus.register(t2_s0_gap, 0.40);
        bus.register(t2_s0_force, 4.50);
        bus.register(t2_s0_buzz, 0.90);

        // Mutate parameters
        bus.set(t1_s1_speed, 12.56);
        bus.set(t1_s1_pulse, 2.50);
        bus.set(t2_s0_gap, 0.65);
        bus.set(t2_s0_force, 7.00);

        assert_eq!(bus.get(t1_s1_speed), Some(12.56));
        assert_eq!(bus.get(t1_s1_pulse), Some(2.50));
        assert_eq!(bus.get(t1_s1_press), Some(0.70));
        assert_eq!(bus.get(t2_s0_gap), Some(0.65));
        assert_eq!(bus.get(t2_s0_force), Some(7.00));
        assert_eq!(bus.get(t2_s0_buzz), Some(0.90));
    }

    #[test]
    fn test_turn48_pure_hurdy_gurdy_and_trompette_timeline_evaluation() {
        let mut timeline = AutomationTimeline::new();

        let lane_speed = "track_1_node_1_crank_speed_rad_s".to_string();
        let lane_force = "track_2_node_0_strike_force".to_string();

        timeline.lanes.insert(
            lane_speed.clone(),
            AutomationLane {
                param_id: lane_speed.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 3.14, interp: Interpolation::Linear },
                        AutomationPoint { beat: 4.0, value: 12.56, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        timeline.lanes.insert(
            lane_force.clone(),
            AutomationLane {
                param_id: lane_force.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 1.0, interp: Interpolation::Linear },
                        AutomationPoint { beat: 2.0, value: 9.0, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        // Evaluate at beat 2.0
        let val_speed = timeline.lanes.get(&lane_speed).unwrap().curve.evaluate_at_beat(2.0);
        assert!((val_speed - 7.85).abs() < 1e-4);

        // Evaluate at beat 1.0
        let val_force = timeline.lanes.get(&lane_force).unwrap().curve.evaluate_at_beat(1.0);
        assert!((val_force - 5.0).abs() < 1e-4);
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use eframe::egui;
    use summoner_core::param_bus::ParamBus;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack_with_context, ModernDeviceRackState};
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn48_gui_award_winning_view_hurdy_and_trompette_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_hurdy_gurdy_hud_open());
        assert!(!view.is_trompette_hud_open());

        view.open_hurdy_gurdy_hud();
        assert!(view.is_hurdy_gurdy_hud_open());

        view.close_hurdy_gurdy_hud();
        assert!(!view.is_hurdy_gurdy_hud_open());

        view.open_trompette_hud();
        assert!(view.is_trompette_hud_open());

        view.close_trompette_hud();
        assert!(!view.is_trompette_hud_open());
    }

    #[test]
    fn test_turn48_gui_hurdy_gurdy_modal_show_and_sync() {
        let mut view = AwardWinningGuiView::new();
        view.open_hurdy_gurdy_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_hurdy_gurdy_hud_open());
    }

    #[test]
    fn test_turn48_gui_trompette_modal_show_and_sync() {
        let mut view = AwardWinningGuiView::new();
        view.open_trompette_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_trompette_hud_open());
    }

    #[test]
    fn test_turn48_gui_inspector_hud_triggers() {
        let mut inspector = ModernInspectorState::default();
        inspector.selected_node_kind = Some("FrenchHurdyGurdy".to_string());
        inspector.requested_open_hurdy_gurdy_hud = true;

        let mut app_view = AwardWinningGuiView::new();
        app_view.inspector_state = inspector;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app_view.show(ui);
            });
        });

        assert!(!app_view.inspector_state.requested_open_hurdy_gurdy_hud);
        assert!(app_view.is_hurdy_gurdy_hud_open());
    }

    #[test]
    fn test_turn48_gui_device_rack_hud_triggers() {
        let mut rack = ModernDeviceRackState::default();
        rack.selected_node_kind = Some("TrompetteChienBridge".to_string());
        rack.requested_open_trompette_hud = true;

        let mut app_view = AwardWinningGuiView::new();
        app_view.device_rack_state = rack;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app_view.show(ui);
            });
        });

        assert!(!app_view.device_rack_state.requested_open_trompette_hud);
        assert!(app_view.is_trompette_hud_open());
    }

    #[test]
    fn test_turn48_gui_device_rack_pro_drawer_hurdy_and_trompette_visualizers() {
        let mut rack = ModernDeviceRackState::default();
        rack.is_expanded_params = true;
        rack.add_device("FrenchHurdyGurdy", "French Vielle");
        rack.select_device(1);

        let bus = ParamBus::new();
        let ctx = egui::Context::default();

        // Render pro parameter drawer with FrenchHurdyGurdy
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack, None, Some(&bus), 1);
            });
        });

        // Render pro parameter drawer with TrompetteChienBridge
        rack.add_device("TrompetteChienBridge", "Trompette Chien");
        rack.select_device(2);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack, None, Some(&bus), 1);
            });
        });

        assert_eq!(rack.chain_devices.len(), 3);
    }

    #[test]
    fn test_turn48_gui_hud_live_param_bus_sync() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let t1_s0_speed = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 0);
        let t1_s0_pulse = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 1);
        let t1_s0_press = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 2);
        let t1_s0_clear = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 3);

        bus.register(t1_s0_speed, 6.28);
        bus.register(t1_s0_pulse, 1.0);
        bus.register(t1_s0_press, 0.5);
        bus.register(t1_s0_clear, 0.3);

        let bus = std::sync::Arc::new(bus);
        view.live_param_bus = Some(bus.clone());

        // Modify HurdyGurdyView state
        view.hurdy_gurdy_view.crank_speed_rad_s = 9.42;
        view.hurdy_gurdy_view.wrist_acceleration_pulse = 3.2;
        view.hurdy_gurdy_view.wheel_pressure = 0.85;
        view.hurdy_gurdy_view.chien_clearance_mm = 0.55;

        view.sync_hurdy_gurdy_hud_state();

        // Verify live synchronization to node_param_values and ParamBus
        assert_eq!(view.device_rack_state.node_param_values.get("crank_speed_rad_s"), Some(&9.42));
        assert_eq!(view.inspector_state.node_param_values.get("wrist_acceleration_pulse"), Some(&3.2));

        assert_eq!(bus.get(t1_s0_speed), Some(9.42));
        assert_eq!(bus.get(t1_s0_pulse), Some(3.2));
        assert_eq!(bus.get(t1_s0_press), Some(0.85));
        assert_eq!(bus.get(t1_s0_clear), Some(0.55));
    }

    #[test]
    fn test_turn48_gui_command_palette_hurdy_and_trompette_actions() {
        let mut palette = CommandPalette::new();
        palette.open();
        palette.search_query = "Hurdy".into();
        let filtered = palette.get_filtered_actions();
        assert!(filtered.iter().any(|a| a.action_id == "open_hurdy_gurdy_hud"));

        palette.search_query = "Trompette".into();
        let filtered2 = palette.get_filtered_actions();
        assert!(filtered2.iter().any(|a| a.action_id == "open_trompette_hud"));
    }
}
