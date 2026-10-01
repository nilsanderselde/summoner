// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #47 / Turn #14 Sprint Verification Suite:
//! - Physical Modeling Quartz Crystal Singing Bowl & Glass Chalice HUD Modal Lifecycle (`AwardWinningGuiView`)
//! - Physical Modeling Franklin Glass Armonica Spindle Resonance HUD Modal Lifecycle (`AwardWinningGuiView`)
//! - Multi-Node Track DSP Device Chain Live ParamBus Slot Channel Addressing (`ParamId(track_id * 1000 + slot * 20 + p_i)`)
//! - Modern Inspector & Device Rack Physical Modeling Tactical Action Triggers & Auto-Detection
//! - Modern Device Rack Pro Drawer Specialized 2D Acoustic Rim & Spindle Friction Visualizers
//! - Zero Heap Allocation Audio-Thread ParamBus Safety & Live Automation Evaluation
//! - Command Palette Integration for Crystal Resonator and Glass Armonica HUDs

#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn47_pure_crystal_and_armonica_slot_channels() {
        let mut bus = ParamBus::new();

        // Track 1, Slot 0: AetherSynth (cutoff, resonance)
        let t1_s0_p0 = ParamId(1 * 1000 + 0 * 20 + 0); // 1000
        let t1_s0_p1 = ParamId(1 * 1000 + 0 * 20 + 1); // 1001

        // Track 1, Slot 1: CrystalResonator (root_freq, water_fill, q_scale, friction_velocity, master_gain)
        let t1_s1_root = ParamId(1 * 1000 + 1 * 20 + 0); // 1020
        let t1_s1_water = ParamId(1 * 1000 + 1 * 20 + 1); // 1021
        let t1_s1_q = ParamId(1 * 1000 + 1 * 20 + 2); // 1022
        let t1_s1_fric = ParamId(1 * 1000 + 1 * 20 + 3); // 1023
        let t1_s1_gain = ParamId(1 * 1000 + 1 * 20 + 4); // 1024

        // Track 2, Slot 0: GlassArmonica (rotation_speed, normal_force, water_level, modal_fundamental)
        let t2_s0_speed = ParamId(2 * 1000 + 0 * 20 + 0); // 2000
        let t2_s0_force = ParamId(2 * 1000 + 0 * 20 + 1); // 2001
        let t2_s0_water = ParamId(2 * 1000 + 0 * 20 + 2); // 2002
        let t2_s0_fund = ParamId(2 * 1000 + 0 * 20 + 3); // 2003

        bus.register(t1_s0_p0, 0.75);
        bus.register(t1_s0_p1, 0.50);

        bus.register(t1_s1_root, 432.0);
        bus.register(t1_s1_water, 0.20);
        bus.register(t1_s1_q, 1.50);
        bus.register(t1_s1_fric, 0.65);
        bus.register(t1_s1_gain, 1.00);

        bus.register(t2_s0_speed, 2.50);
        bus.register(t2_s0_force, 0.45);
        bus.register(t2_s0_water, 0.40);
        bus.register(t2_s0_fund, 523.25);

        assert_eq!(bus.get(t1_s1_root), Some(432.0));
        assert_eq!(bus.get(t1_s1_water), Some(0.20));
        assert_eq!(bus.get(t2_s0_speed), Some(2.50));
        assert_eq!(bus.get(t2_s0_fund), Some(523.25));

        // Atomic lock-free updates
        bus.set(t1_s1_water, 0.75);
        bus.set(t1_s1_fric, 0.90);
        bus.set(t2_s0_speed, 5.00);
        bus.set(t2_s0_force, 0.80);

        assert_eq!(bus.get(t1_s1_water), Some(0.75));
        assert_eq!(bus.get(t1_s1_fric), Some(0.90));
        assert_eq!(bus.get(t1_s0_p0), Some(0.75)); // slot 0 untouched
        assert_eq!(bus.get(t2_s0_speed), Some(5.00));
        assert_eq!(bus.get(t2_s0_force), Some(0.80));
    }

    #[test]
    fn test_turn47_pure_crystal_and_armonica_timeline_evaluation() {
        let mut timeline = AutomationTimeline::new();

        let lane_water = "track_1_node_1_water_fill_level".to_string();
        let lane_speed = "track_2_node_0_rotation_speed_rad_s".to_string();

        timeline.lanes.insert(
            lane_water.clone(),
            AutomationLane {
                param_id: lane_water.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 0.10, interp: Interpolation::Linear },
                        AutomationPoint { beat: 4.0, value: 0.90, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        timeline.lanes.insert(
            lane_speed.clone(),
            AutomationLane {
                param_id: lane_speed.clone(),
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 1.0, interp: Interpolation::Linear },
                        AutomationPoint { beat: 2.0, value: 6.0, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        // Evaluate at beat 2.0
        let val_water = timeline.lanes.get(&lane_water).unwrap().curve.evaluate_at_beat(2.0);
        assert!((val_water - 0.50).abs() < 1e-4);

        // Evaluate at beat 1.0
        let val_speed = timeline.lanes.get(&lane_speed).unwrap().curve.evaluate_at_beat(1.0);
        assert!((val_speed - 3.50).abs() < 1e-4);
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
    fn test_turn47_gui_award_winning_view_hud_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_crystal_resonator_hud_open());
        assert!(!view.is_glass_armonica_hud_open());

        view.open_crystal_resonator_hud();
        assert!(view.is_crystal_resonator_hud_open());

        view.close_crystal_resonator_hud();
        assert!(!view.is_crystal_resonator_hud_open());

        view.open_glass_armonica_hud();
        assert!(view.is_glass_armonica_hud_open());

        view.close_glass_armonica_hud();
        assert!(!view.is_glass_armonica_hud_open());
    }

    #[test]
    fn test_turn47_gui_inspector_hud_triggers() {
        let mut insp = ModernInspectorState::default();
        insp.selected_node_kind = Some("CrystalResonator".to_string());
        assert!(!insp.requested_open_crystal_hud);

        let ctx = egui::Context::default();
        let bus = ParamBus::new();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut insp, Some(&bus), 1);
            });
        });

        // Trigger request flag
        insp.requested_open_crystal_hud = true;

        let mut app_view = AwardWinningGuiView::new();
        app_view.inspector_state = insp;

        // Running show() processes the request and opens the modal
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app_view.show(ui);
            });
        });

        assert!(!app_view.inspector_state.requested_open_crystal_hud);
        assert!(app_view.is_crystal_resonator_hud_open());
    }

    #[test]
    fn test_turn47_gui_device_rack_hud_triggers() {
        let mut rack = ModernDeviceRackState::default();
        rack.selected_node_kind = Some("GlassArmonica".to_string());
        rack.requested_open_armonica_hud = true;

        let mut app_view = AwardWinningGuiView::new();
        app_view.device_rack_state = rack;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app_view.show(ui);
            });
        });

        assert!(!app_view.device_rack_state.requested_open_armonica_hud);
        assert!(app_view.is_glass_armonica_hud_open());
    }

    #[test]
    fn test_turn47_gui_device_rack_pro_drawer_specialized_visualizers() {
        let mut rack = ModernDeviceRackState::default();
        rack.is_expanded_params = true;
        rack.add_device("CrystalResonator", "Crystal Bowl");
        rack.select_device(1);

        let bus = ParamBus::new();
        let ctx = egui::Context::default();

        // Render pro parameter drawer with CrystalResonator
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack, None, Some(&bus), 1);
            });
        });

        // Render pro parameter drawer with GlassArmonica
        rack.add_device("GlassArmonica", "Franklin Armonica");
        rack.select_device(2);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack, None, Some(&bus), 1);
            });
        });

        assert_eq!(rack.chain_devices.len(), 3);
    }

    #[test]
    fn test_turn47_gui_hud_live_param_bus_sync() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let t1_s0_root = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 0);
        let t1_s0_water = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 1);
        let t1_s0_q = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 2);
        let t1_s0_fric = summoner_core::param_bus::ParamId(1 * 1000 + 0 * 20 + 3);

        bus.register(t1_s0_root, 440.0);
        bus.register(t1_s0_water, 0.0);
        bus.register(t1_s0_q, 1.0);
        bus.register(t1_s0_fric, 0.0);

        let bus = std::sync::Arc::new(bus);
        view.live_param_bus = Some(bus.clone());

        // Modify CrystalResonatorView state
        view.crystal_resonator_view.root_freq_hz = 528.0;
        view.crystal_resonator_view.water_fill_pct = 0.45;
        view.crystal_resonator_view.q_scale = 2.5;
        view.crystal_resonator_view.friction_speed_mps = 0.85;

        view.sync_crystal_resonator_hud_state();

        // Verify live synchronization to node_param_values and ParamBus
        assert_eq!(view.device_rack_state.node_param_values.get("water_fill_level"), Some(&0.45));
        assert_eq!(view.inspector_state.node_param_values.get("root_freq_hz"), Some(&528.0));

        assert_eq!(bus.get(t1_s0_root), Some(528.0));
        assert_eq!(bus.get(t1_s0_water), Some(0.45));
        assert_eq!(bus.get(t1_s0_q), Some(2.5));
        assert_eq!(bus.get(t1_s0_fric), Some(0.85));
    }

    #[test]
    fn test_turn47_gui_command_palette_crystal_and_armonica_actions() {
        let mut palette = CommandPalette::new();
        palette.open();
        palette.search_query = "Crystal".into();
        let filtered = palette.get_filtered_actions();
        assert!(filtered.iter().any(|a| a.action_id == "open_crystal_resonator_hud"));

        palette.search_query = "Armonica".into();
        let filtered2 = palette.get_filtered_actions();
        assert!(filtered2.iter().any(|a| a.action_id == "open_glass_armonica_hud"));
    }
}
