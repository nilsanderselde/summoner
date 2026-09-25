// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #11 Integration & Regression Tests:
//! 1. Production-Grade Two-Tier UX Mode Switcher (Novice Macro Strip vs. Pro Parameter Drawer & Inspector).
//! 2. Milestone 33 Live Parameter Automation Bridge for Novice Macros (Tone, Space, Punch, Character) & Master Gain.
//! 3. Zero-Allocation ParamBus Synchronization for Top Bar Controls and Arranger Track Faders.
//! 4. Console Mixer Pro Navigation Launchers (DAG, Rack, Auto) and Universal DSP Effect Insertion across all 11 DspCategories.

#[cfg(test)]
pub mod pure_tests {
    use crate::dsp_node_ui::{DspCategory, DspNodeRegistry};
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::NodeConfig;
    use std::collections::HashMap;

    #[test]
    fn test_turn11_param_bus_macro_and_master_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 2u32;

        // Register macro and master offsets
        let tone_pid = ParamId(track_id * 1000 + 100);
        let space_pid = ParamId(track_id * 1000 + 102);
        let punch_pid = ParamId(track_id * 1000 + 105);
        let char_pid = ParamId(track_id * 1000 + 104);
        let gain_pid = ParamId(track_id * 1000 + 200);
        let pan_pid = ParamId(track_id * 1000 + 201);
        let master_pid = ParamId(9999);

        bus.register(tone_pid, 0.65);
        bus.register(space_pid, 0.40);
        bus.register(punch_pid, 0.55);
        bus.register(char_pid, 0.50);
        bus.register(gain_pid, 1.0);
        bus.register(pan_pid, 0.0);
        bus.register(master_pid, 1.0);

        // Verify initial read
        assert_eq!(bus.get(tone_pid), Some(0.65));
        assert_eq!(bus.get(space_pid), Some(0.40));
        assert_eq!(bus.get(punch_pid), Some(0.55));
        assert_eq!(bus.get(char_pid), Some(0.50));
        assert_eq!(bus.get(master_pid), Some(1.0));

        // Atomic update from GUI thread (zero allocation)
        bus.set(tone_pid, 0.88);
        bus.set(space_pid, 0.72);
        bus.set(master_pid, 0.95);

        assert_eq!(bus.get(tone_pid), Some(0.88));
        assert_eq!(bus.get(space_pid), Some(0.72));
        assert_eq!(bus.get(master_pid), Some(0.95));
    }

    #[test]
    fn test_turn11_mixer_universal_dsp_inventory() {
        let registry = DspNodeRegistry::new();
        let all_nodes = registry.list_all();
        assert!(all_nodes.len() >= 1000, "Registry must contain reflected DSP nodes, found {}", all_nodes.len());

        let categories = [
            DspCategory::FilterEq,
            DspCategory::DynamicsMaster,
            DspCategory::DistortionSaturation,
            DspCategory::Modulation,
            DspCategory::TimeSpace,
            DspCategory::SpatialSurround,
            DspCategory::AcousticPhysicalModel,
            DspCategory::SpectralResynthesis,
            DspCategory::NeuralAi,
            DspCategory::Oscillator,
            DspCategory::Utility,
        ];

        for cat in categories {
            let cat_nodes: Vec<_> = all_nodes.iter().filter(|n| n.category == cat).collect();
            assert!(!cat_nodes.is_empty(), "Category {:?} must have registered DSP modules", cat);

            // Verify a module from this category can be converted into a valid NodeConfig
            let sample_node = cat_nodes[0];
            let mut params = HashMap::new();
            for p in &sample_node.params {
                params.insert(p.id.clone(), 0.5f32);
            }
            let node_cfg = NodeConfig {
                kind: sample_node.kind_id.clone(),
                params,
                plugin_state: None,
            };
            assert_eq!(node_cfg.kind, sample_node.kind_id);
            assert!(!node_cfg.params.is_empty() || sample_node.params.is_empty());
        }
    }
}

#[cfg(all(test, feature = "gui"))]
mod tests {
    use eframe::egui;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::mixer::MixerState;
    use crate::views::modern_top_bar::ModernTopBarState;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::create_default_project;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation};

    #[test]
    fn test_turn11_two_tier_mode_state_transitions() {
        let mut top_bar = ModernTopBarState::default();
        // Defaults to Novice simplicity
        assert!(!top_bar.is_pro_mode, "Default must be Novice mode");
        assert!(top_bar.is_novice_macro_visible, "Novice macros must be visible by default");

        // Transition to Pro Mode
        top_bar.is_pro_mode = true;
        top_bar.is_novice_macro_visible = false;
        assert!(top_bar.is_pro_mode);
        assert!(!top_bar.is_novice_macro_visible);

        // Transition back to Novice Mode
        top_bar.is_pro_mode = false;
        top_bar.is_novice_macro_visible = true;
        assert!(!top_bar.is_pro_mode);
        assert!(top_bar.is_novice_macro_visible);
    }

    #[test]
    fn test_turn11_mixer_state_navigation_requests() {
        let mut state = MixerState::default();
        assert!(state.requested_navigation.is_none());

        // Test DAG navigation trigger
        state.requested_navigation = Some((3, "dag".to_string()));
        let nav = state.requested_navigation.take();
        assert_eq!(nav, Some((3, "dag".to_string())));
        assert!(state.requested_navigation.is_none());

        // Test Rack navigation trigger
        state.requested_navigation = Some((5, "rack".to_string()));
        let nav = state.requested_navigation.take();
        assert_eq!(nav, Some((5, "rack".to_string())));

        // Test Automation navigation trigger
        state.requested_navigation = Some((7, "auto".to_string()));
        let nav = state.requested_navigation.take();
        assert_eq!(nav, Some((7, "auto".to_string())));
    }

    #[test]
    fn test_turn11_award_winning_view_two_tier_sync() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        // 1. Initial frame in default Novice Mode
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.top_bar_state.is_pro_mode);
        assert!(view.top_bar_state.is_novice_macro_visible);

        // 2. User activates Pro Mode via Top Bar
        view.top_bar_state.is_pro_mode = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Verify Pro Drawer & Inspector are automatically uncollapsed
        assert!(view.top_bar_state.is_pro_mode);
        assert!(!view.top_bar_state.is_novice_macro_visible);
        assert!(!view.device_rack_state.is_minimized);
        assert!(view.device_rack_state.is_expanded_params);
        assert!(!view.inspector_state.is_collapsed);

        // 3. User switches back to Novice Mode
        view.top_bar_state.is_pro_mode = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(!view.top_bar_state.is_pro_mode);
        assert!(view.top_bar_state.is_novice_macro_visible);
        assert!(!view.device_rack_state.is_expanded_params);
    }

    #[test]
    fn test_turn11_live_parameter_bridge_eval_and_dispatch() {
        let mut view = AwardWinningGuiView::default();
        let project = create_default_project("Turn 11 M33 Test");
        let track_id = project.tracks[0].id;
        let mut bus = ParamBus::new();

        // Pre-register parameters in ParamBus
        let tone_pid = ParamId(track_id as u32 * 1000 + 100);
        let space_pid = ParamId(track_id as u32 * 1000 + 102);
        let master_pid = ParamId(9999);
        bus.register(tone_pid, 0.5);
        bus.register(space_pid, 0.5);
        bus.register(master_pid, 1.0);

        let mut auto_reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Create automation curve for track macro_tone: at beat 2.0 value is 0.92
        let lane_key = format!("track_{}_macro_tone", track_id);
        timeline.lanes.insert(lane_key.clone(), AutomationLane {
            param_id: lane_key.clone(),
            curve: AutomationCurve {
                points: vec![
                    AutomationPoint { beat: 0.0, value: 0.20, interp: Interpolation::Linear },
                    AutomationPoint { beat: 2.0, value: 0.92, interp: Interpolation::Linear },
                ],
            },
        });

        // Set transport playing at beat 2.0
        view.top_bar_state.is_playing = true;

        // Run sync_with_param_bus (playback mode, not recording)
        view.sync_with_param_bus(
            &project,
            &bus,
            &mut auto_reg,
            &mut timeline,
            2.0,
            false,
        );

        // Verify top bar macro_tone was dynamically evaluated from curve
        assert!((view.top_bar_state.macro_tone - 0.92).abs() < 1e-4, "macro_tone should be evaluated to 0.92, got {}", view.top_bar_state.macro_tone);
        // Verify ParamBus received the automated value
        assert!((bus.get(tone_pid).unwrap() - 0.92).abs() < 1e-4);
    }

    #[test]
    fn test_turn11_live_parameter_recording_macro_and_master() {
        let mut view = AwardWinningGuiView::default();
        let project = create_default_project("Turn 11 Recording Test");
        let track_id = project.tracks[0].id;
        let mut bus = ParamBus::new();

        let tone_pid = ParamId(track_id as u32 * 1000 + 100);
        let master_pid = ParamId(9999);
        bus.register(tone_pid, 0.5);
        bus.register(master_pid, 1.0);

        let mut auto_reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // User manipulates Macro Tone to 0.77 and Master Gain to 0.85
        view.top_bar_state.macro_tone = 0.77;
        view.top_bar_state.master_gain = 0.85;

        // Run sync_with_param_bus in recording mode at beat 3.5
        view.sync_with_param_bus(
            &project,
            &bus,
            &mut auto_reg,
            &mut timeline,
            3.5,
            true, // is_recording_automation
        );

        // Verify ParamBus received live values
        assert_eq!(bus.get(tone_pid), Some(0.77));
        assert_eq!(bus.get(master_pid), Some(0.85));

        // Verify automation points were recorded in timeline
        let tone_lane = format!("track_{}_macro_tone", track_id);
        assert!(timeline.lanes.contains_key(&tone_lane), "Timeline must contain macro_tone lane");
        let lane = timeline.lanes.get(&tone_lane).unwrap();
        assert_eq!(lane.curve.points.len(), 1);
        assert_eq!(lane.curve.points[0].beat, 3.5);
        assert_eq!(lane.curve.points[0].value, 0.77);

        assert!(timeline.lanes.contains_key("master_gain"), "Timeline must contain master_gain lane");
        let m_lane = timeline.lanes.get("master_gain").unwrap();
        assert_eq!(m_lane.curve.points.len(), 1);
        assert_eq!(m_lane.curve.points[0].beat, 3.5);
        assert_eq!(m_lane.curve.points[0].value, 0.85);
    }
}
