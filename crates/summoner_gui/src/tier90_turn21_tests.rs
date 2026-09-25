// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #21 Integration & Regression Tests:
//! 1. Modular Canvas faceplate tactile parameter lifecycle, reflection, and duplication.
//! 2. Modular Node knob parameter adjustment, bounds clamping, and reset to defaults.
//! 3. Milestone 33 live parameter bridge: lockstep dispatch of modular faceplate parameters to `ParamBus` and `AutomationTimeline`.
//! 4. Bidirectional Modular Routing Matrix synchronization between visual patch cords and crossbar matrix.
//! 5. Universal DSP module catalog instantiation parameter reflection.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn21_modular_node_params_lifecycle_and_duplication() {
        let mut view = AwardWinningGuiView::new();

        // 1. Verify default nodes have faceplate parameters initialized
        let osc = view.modular_nodes.iter().find(|n| n.id == "osc_1").expect("osc_1 must exist");
        assert_eq!(osc.params.len(), 2);
        assert_eq!(osc.params[0].id, "freq");
        assert_eq!(osc.params[0].value, 440.0);
        assert_eq!(osc.params[1].id, "shape");
        assert_eq!(osc.params[1].value, 0.5);

        let filter = view.modular_nodes.iter().find(|n| n.id == "filter_1").expect("filter_1 must exist");
        assert_eq!(filter.params.len(), 2);
        assert_eq!(filter.params[0].id, "cutoff");
        assert_eq!(filter.params[0].value, 1200.0);
        assert_eq!(filter.params[1].id, "resonance");
        assert_eq!(filter.params[1].value, 1.0);

        // 2. Adjust parameter value on osc_1
        let ok = view.set_modular_node_param("osc_1", "freq", 880.0);
        assert!(ok);
        assert_eq!(view.get_modular_node_param("osc_1", "freq"), Some(880.0));

        // 3. Duplicate osc_1 and ensure cloned node retains modified parameter values
        let dup_id = view.duplicate_modular_node("osc_1").expect("Duplication must succeed");
        let dup_node = view.modular_nodes.iter().find(|n| n.id == dup_id).expect("Duplicate node must exist");
        assert_eq!(dup_node.params.len(), 2);
        assert_eq!(dup_node.params[0].id, "freq");
        assert_eq!(dup_node.params[0].value, 880.0); // Retains adjusted value!
        assert_eq!(dup_node.params[1].id, "shape");
        assert_eq!(dup_node.params[1].value, 0.5);
    }

    #[test]
    fn test_turn21_modular_node_knob_adjustment_and_clamping() {
        let mut view = AwardWinningGuiView::new();

        // Bounds clamping tests for filter cutoff (20.0 to 20000.0)
        assert!(view.set_modular_node_param("filter_1", "cutoff", 5000.0));
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(5000.0));

        // Upper clamp
        view.set_modular_node_param("filter_1", "cutoff", 99999.0);
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(20000.0));

        // Lower clamp
        view.set_modular_node_param("filter_1", "cutoff", -500.0);
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(20.0));

        // Non-existent node or param safety
        assert!(!view.set_modular_node_param("non_existent_node", "cutoff", 100.0));
        assert!(!view.set_modular_node_param("filter_1", "invalid_param", 100.0));
        assert_eq!(view.get_modular_node_param("filter_1", "invalid_param"), None);
    }

    #[test]
    fn test_turn21_modular_node_knob_reset_to_default() {
        let mut view = AwardWinningGuiView::new();

        // Change filter cutoff away from default 1200.0
        view.set_modular_node_param("filter_1", "cutoff", 6400.0);
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(6400.0));

        // Reset to default
        let res = view.reset_modular_node_param("filter_1", "cutoff");
        assert_eq!(res, Some(1200.0));
        assert_eq!(view.get_modular_node_param("filter_1", "cutoff"), Some(1200.0));

        // Reset non-existent
        assert_eq!(view.reset_modular_node_param("filter_1", "foo"), None);
    }

    #[test]
    fn test_turn21_m33_modular_node_param_bus_and_automation_bridge() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32;
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Pre-allocate slots in ParamBus for modular node parameters
        for n_idx in 0..view.modular_nodes.len() {
            for p_idx in 0..8 {
                let pid = ParamId(track_id * 1000 + 500 + (n_idx as u32 * 16) + p_idx as u32);
                bus.register(pid, 0.0);
            }
        }

        // Adjust a faceplate parameter
        view.set_modular_node_param("osc_1", "freq", 523.25); // C5 note

        // Synchronize with ParamBus while recording automation at beat 4.0
        view.sync_with_param_bus(
            &project,
            &mut bus,
            &mut reg,
            &mut timeline,
            4.0,  // playhead_beat
            true, // is_recording_automation
        );

        // Verify ParamBus slot was updated
        // osc_1 is node index 0, freq is param index 0
        let freq_pid = ParamId(track_id * 1000 + 500);
        assert_eq!(bus.get(freq_pid), Some(523.25));

        // Verify AutomationRegistry has registered and updated key
        assert_eq!(reg.get_param("modular_osc_1_freq").map(|p| p.get()), Some(523.25));

        // Verify AutomationTimeline recorded the live point
        let lane = timeline.lanes.get("modular_osc_1_freq").expect("Lane must be recorded");
        assert_eq!(lane.curve.points.len(), 1);
        assert_eq!(lane.curve.points[0].beat, 4.0);
        assert_eq!(lane.curve.points[0].value, 523.25);
    }

    #[test]
    fn test_turn21_bidirectional_modular_routing_matrix_sync() {
        let mut view = AwardWinningGuiView::new();

        // 1. Initial sync to routing matrix
        view.sync_modular_to_routing_matrix();
        assert!(!view.patch_matrix.sources.is_empty());
        assert!(!view.patch_matrix.destinations.is_empty());
        assert_eq!(view.patch_matrix.connections.len(), view.patch_cords.len());

        // Verify default connection exists in matrix
        let osc_to_filter = ("osc_1:out".to_string(), "filter_1:in".to_string());
        let conn = view.patch_matrix.connections.get(&osc_to_filter).expect("Connection must exist in matrix");
        assert!(conn.active);
        assert_eq!(conn.intensity, 1.0);

        // 2. Add connection inside Routing Matrix (e.g. osc_1:out to vca_1:audio)
        view.patch_matrix.connect("osc_1:out", "vca_1:audio", 0.85);

        // 3. Sync routing matrix back to modular cords
        view.sync_routing_matrix_to_modular();

        let cord = view.patch_cords.iter().find(|c| {
            c.from_node_id == "osc_1" && c.from_port_id == "out" && c.to_node_id == "vca_1" && c.to_port_id == "audio"
        }).expect("New cord must have been created from matrix");
        assert!((cord.intensity - 0.85).abs() < 0.001);

        // 4. Disconnect in routing matrix and sync back
        view.patch_matrix.toggle_connection("osc_1:out", "vca_1:audio");
        view.sync_routing_matrix_to_modular();

        let cord_still_exists = view.patch_cords.iter().any(|c| {
            c.from_node_id == "osc_1" && c.from_port_id == "out" && c.to_node_id == "vca_1" && c.to_port_id == "audio"
        });
        assert!(!cord_still_exists);
    }

    #[test]
    fn test_turn21_universal_dsp_module_instantiation_params() {
        let mut view = AwardWinningGuiView::new();
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();

        // Instantiate MOOG Ladder Filter
        if let Some(desc) = registry.get("FilterMoogLadder") {
            let initial_len = view.modular_nodes.len();
            view.add_modular_node_from_descriptor(desc);
            assert_eq!(view.modular_nodes.len(), initial_len + 1);

            let added_node = view.modular_nodes.last().unwrap();
            assert_eq!(added_node.kind_id, "FilterMoogLadder");
            // Parameters must be populated from descriptor!
            assert!(!added_node.params.is_empty());
        }

        // Instantiate Stereo Chorus
        if let Some(desc) = registry.get("StereoChorus") {
            let initial_len = view.modular_nodes.len();
            view.add_modular_node_from_descriptor(desc);
            assert_eq!(view.modular_nodes.len(), initial_len + 1);

            let added_node = view.modular_nodes.last().unwrap();
            assert_eq!(added_node.kind_id, "StereoChorus");
            assert!(!added_node.params.is_empty());
        }
    }
}
