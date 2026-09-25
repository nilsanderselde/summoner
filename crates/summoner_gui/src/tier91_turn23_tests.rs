// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #23 Integration & Regression Tests:
//! 1. Live Modular Parameter Automation Editor window lifecycle (open/close).
//! 2. Live quick curve shaping (Flat, Ramp Up, Ramp Down, Sine LFO, Exp Drop, S-Curve, Invert, Smooth).
//! 3. Milestone 33 lockstep live parameter evaluation and ParamBus dispatch during playback.
//! 4. Milestone 33 live patch cord intensity curve evaluation and replay.
//! 5. Bidirectional parameter synchronization between modular faceplates and Pro Inspector / Device Rack.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AutomationQuickShape, AwardWinningGuiView};
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn23_modular_automation_editor_open_and_close() {
        let mut view = AwardWinningGuiView::new();

        // 1. Initial state
        assert!(!view.show_automation_editor_window);
        assert!(view.automation_editor.is_none());
        assert!(view.requested_modular_automation_param.is_none());

        // 2. Open editor for filter_1 cutoff
        let ok = view.open_modular_automation_editor("filter_1", "cutoff");
        assert!(ok);
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param.as_deref(),
            Some("modular_filter_1_cutoff")
        );
        let editor = view.automation_editor.as_ref().expect("Editor must be initialized");
        assert_eq!(editor.min_display, 20.0);
        assert_eq!(editor.max_display, 20000.0);
        assert_eq!(editor.unit_name, "Hz");
        assert!(!editor.nodes.is_empty());

        // Initialized with current cutoff value (1200.0 Hz) normalized:
        // (1200 - 20) / (20000 - 20) = 1180 / 19980 ≈ 0.05906
        let norm_val = editor.nodes[0].value;
        assert!((norm_val - (1180.0 / 19980.0)).abs() < 1e-3);

        // 3. Close editor
        view.close_modular_automation_editor();
        assert!(!view.show_automation_editor_window);

        // 4. Invalid node/param safety
        assert!(!view.open_modular_automation_editor("invalid_node", "cutoff"));
        assert!(!view.open_modular_automation_editor("filter_1", "invalid_param"));
    }

    #[test]
    fn test_turn23_modular_automation_quick_shapes() {
        let mut view = AwardWinningGuiView::new();
        assert!(view.open_modular_automation_editor("osc_1", "freq"));

        // 1. Ramp Up: 0.0 to 1.0
        view.apply_automation_quick_shape(AutomationQuickShape::RampUp);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes.len(), 2);
        assert_eq!(editor.nodes[0].value, 0.0);
        assert_eq!(editor.nodes[1].value, 1.0);

        // 2. Invert: 1.0 to 0.0
        view.apply_automation_quick_shape(AutomationQuickShape::Invert);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes[0].value, 1.0);
        assert_eq!(editor.nodes[1].value, 0.0);

        // 3. Ramp Down: 1.0 to 0.0
        view.apply_automation_quick_shape(AutomationQuickShape::RampDown);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes[0].value, 1.0);
        assert_eq!(editor.nodes[1].value, 0.0);

        // 4. Sine LFO: 9 points oscillatory
        view.apply_automation_quick_shape(AutomationQuickShape::SineLfo);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes.len(), 9);
        // Middle point at 8 beats should be ~0.5
        assert!((editor.nodes[4].value - 0.5).abs() < 0.1);

        // 5. Flat
        view.apply_automation_quick_shape(AutomationQuickShape::Flat);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes.len(), 2);
        assert_eq!(editor.nodes[0].value, editor.nodes[1].value);

        // 6. S-Curve
        view.apply_automation_quick_shape(AutomationQuickShape::SCurve);
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.nodes.len(), 2);
        assert_eq!(editor.nodes[0].value, 0.0);
        assert_eq!(editor.nodes[1].value, 1.0);
    }

    #[test]
    fn test_turn23_m33_modular_parameter_curve_evaluation_and_param_bus_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = if !project.tracks.is_empty() {
            project.tracks[view.selected_track_idx].id as u32
        } else {
            view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32
        };
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // 1. Open editor and generate RampUp for filter_1 cutoff (20.0 to 20000.0 Hz)
        assert!(view.open_modular_automation_editor("filter_1", "cutoff"));
        view.apply_automation_quick_shape(AutomationQuickShape::RampUp);

        // 2. Pre-register all modular parameters in ParamBus
        for n_idx in 0..view.modular_nodes.len() {
            for p_idx in 0..8 {
                let pid = ParamId(track_id * 1000 + 500 + (n_idx as u32 * 16) + p_idx as u32);
                bus.register(pid, 0.0);
            }
        }
        let filter_n_idx = view.modular_nodes.iter().position(|n| n.id == "filter_1").unwrap();
        let cutoff_pid = ParamId(track_id * 1000 + 500 + (filter_n_idx as u32 * 16) + 0);

        // 3. Sync at beat 0.0 (editor curve syncs to timeline, then timeline evaluates at beat 0.0)
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 0.0, false);
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(20.0));
        assert_eq!(bus.get(cutoff_pid), Some(20.0));

        // 4. Advance playhead to beat 8.0 (halfway on 16-beat timeline -> 50% = 10010.0 Hz)
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 8.0, false);
        let cur_val = view.get_modular_node_param("filter_1", "cutoff").unwrap();
        assert!((cur_val - 10010.0).abs() < 10.0);
        assert!((bus.get(cutoff_pid).unwrap() - 10010.0).abs() < 10.0);

        // 5. Advance playhead to beat 16.0 (full scale -> 20000.0 Hz)
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 16.0, false);
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(20000.0));
        assert_eq!(bus.get(cutoff_pid), Some(20000.0));
    }

    #[test]
    fn test_turn23_m33_patch_cord_intensity_curve_evaluation_and_replay() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = if !project.tracks.is_empty() {
            project.tracks[view.selected_track_idx].id as u32
        } else {
            view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32
        };
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Target the first patch cord: osc_1:audio_out -> filter_1:audio_in
        let cord = &view.patch_cords[0];
        let cord_key = format!(
            "modular_cord_{}_{}_{}_{}_intensity",
            cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id
        );
        let cord_pid = ParamId(track_id * 1000 + 700 + 0);
        for c_idx in 0..view.patch_cords.len() {
            let pid = ParamId(track_id * 1000 + 700 + c_idx as u32);
            bus.register(pid, 0.0);
        }

        // Add automation curve: beat 0.0 = 0.2, beat 4.0 = 1.0
        let lane = AutomationLane {
            param_id: cord_key.clone(),
            curve: AutomationCurve {
                points: vec![
                    AutomationPoint {
                        beat: 0.0,
                        value: 0.2,
                        interp: Interpolation::Linear,
                    },
                    AutomationPoint {
                        beat: 4.0,
                        value: 1.0,
                        interp: Interpolation::Linear,
                    },
                ],
            },
        };
        timeline.lanes.insert(cord_key, lane);

        // Replay at beat 2.0 (midpoint -> 0.6)
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 2.0, false);
        let cord_intensity = view.patch_cords[0].intensity;
        assert!((cord_intensity - 0.6).abs() < 1e-3);
        assert!((bus.get(cord_pid).unwrap() - 0.6).abs() < 1e-3);
    }

    #[test]
    fn test_turn23_modular_node_selection_and_bidirectional_inspector_sync() {
        let mut view = AwardWinningGuiView::new();

        // 1. Programmatically select filter_1
        assert!(view.select_modular_node("filter_1"));
        assert_eq!(view.selected_modular_node_id.as_deref(), Some("filter_1"));
        assert_eq!(view.inspector_state.target_name, "Filter SVF");
        assert_eq!(view.inspector_state.node_param_values.get("cutoff"), Some(&1200.0));

        // 2. Modify parameter in Inspector map and call sync
        view.inspector_state.node_param_values.insert("cutoff".to_string(), 4800.0);
        view.sync_selected_modular_node_params();
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(4800.0));

        // 3. Modify parameter on modular faceplate and call sync
        view.set_modular_node_param("filter_1", "cutoff", 7500.0);
        view.sync_selected_modular_node_params();
        assert_eq!(view.inspector_state.node_param_values.get("cutoff"), Some(&7500.0));
        assert_eq!(view.device_rack_state.node_param_values.get("cutoff"), Some(&7500.0));
    }
}
