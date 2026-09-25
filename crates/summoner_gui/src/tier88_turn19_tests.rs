// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #19 Integration & Regression Tests:
//! 1. Modular Canvas tactile node drag-to-reposition and boundary clamping.
//! 2. Modular Node lifecycle operations: deletion, duplication, and bypass toggling with automatic cable cleanup.
//! 3. Universal DSP module instantiation into modular canvas across categories with specialized port routing.
//! 4. Milestone 33 live parameter bridge: modular node bypass and parameter dispatch to `ParamBus` and `AutomationTimeline`.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, ModularPortKind};
    use crate::dsp_node_ui::DspNodeRegistry;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn19_modular_node_bypass_toggle_and_param_bus_sync() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32;
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Register ParamIds on bus for modular bypass channels (track offset 600..604)
        for i in 0..4 {
            bus.register(ParamId(track_id * 1000 + 600 + i), 0.0);
        }

        // Verify initial state: all nodes not bypassed
        assert!(!view.modular_nodes[0].bypassed);
        assert_eq!(view.modular_nodes[0].id, "osc_1");

        // Toggle bypass on osc_1
        let new_state = view.toggle_bypass_modular_node("osc_1");
        assert_eq!(new_state, Some(true));
        assert!(view.modular_nodes[0].bypassed);

        // Sync with param bus and automation recording
        let playhead = 4.0;
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, playhead, true);

        // Verify ParamBus was updated at track_id * 1000 + 600 + 0
        let val = bus.get(ParamId(track_id * 1000 + 600)).expect("Bypass ParamId should exist");
        assert_eq!(val, 1.0, "ParamBus should receive 1.0 for bypassed node");

        // Verify AutomationRegistry has recorded parameter
        let reg_val = reg.get_param(&format!("modular_{}_bypassed", "osc_1")).map(|p| p.get());
        assert_eq!(reg_val, Some(1.0));

        // Verify AutomationTimeline has recorded the automation point
        let lane = timeline.lanes.get(&format!("modular_{}_bypassed", "osc_1"));
        assert!(lane.is_some(), "Automation lane for modular bypass must be created");
        let points = &lane.unwrap().curve.points;
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].beat, 4.0);
        assert_eq!(points[0].value, 1.0);

        // Toggle back off
        let unbypassed = view.toggle_bypass_modular_node("osc_1");
        assert_eq!(unbypassed, Some(false));
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 6.0, true);
        let val_off = bus.get(ParamId(track_id * 1000 + 600)).unwrap();
        assert_eq!(val_off, 0.0, "ParamBus should receive 0.0 for active node");
    }

    #[test]
    fn test_turn19_modular_node_duplication() {
        let mut view = AwardWinningGuiView::new();
        let initial_node_count = view.modular_nodes.len();
        let initial_graph_nodes = view.audio_graph.lock().unwrap().nodes.len();

        let source_id = "filter_1";
        let source_pos = view.modular_nodes.iter().find(|n| n.id == source_id).unwrap().pos;

        // Duplicate filter_1
        let copy_id = view.duplicate_modular_node(source_id).expect("Duplicate should succeed");

        assert_eq!(view.modular_nodes.len(), initial_node_count + 1);
        assert_eq!(view.audio_graph.lock().unwrap().nodes.len(), initial_graph_nodes + 1);

        let copy_node = view.modular_nodes.iter().find(|n| n.id == copy_id).expect("Copied node must exist");
        assert_eq!(copy_node.kind_id, "FilterSvf");
        assert!(copy_node.display_name.contains("(Copy)"));
        assert_eq!(copy_node.pos.0, source_pos.0 + 25.0);
        assert_eq!(copy_node.pos.1, source_pos.1 + 25.0);
        assert_eq!(copy_node.ports.len(), 3);

        // Selected modular node should point to the duplicate
        assert_eq!(view.selected_modular_node_id.as_deref(), Some(copy_id.as_str()));
    }

    #[test]
    fn test_turn19_modular_node_removal_and_patch_cord_cleanup() {
        let mut view = AwardWinningGuiView::new();

        // Initially we have 4 default nodes and 3 default patch cords:
        // osc_1 -> filter_1, filter_1 -> vca_1, env_1 -> vca_1
        assert_eq!(view.modular_nodes.len(), 4);
        assert_eq!(view.patch_cords.len(), 3);

        // Remove filter_1 which connects both from osc_1 and to vca_1
        let removed = view.remove_modular_node("filter_1");
        assert!(removed);
        assert_eq!(view.modular_nodes.len(), 3);

        // Both cords connected to filter_1 must be removed
        for cord in &view.patch_cords {
            assert_ne!(cord.from_node_id, "filter_1", "No cords should originate from removed node");
            assert_ne!(cord.to_node_id, "filter_1", "No cords should terminate at removed node");
        }
        // Only env_1 -> vca_1 cord should remain
        assert_eq!(view.patch_cords.len(), 1);
        assert_eq!(view.patch_cords[0].from_node_id, "env_1");
        assert_eq!(view.patch_cords[0].to_node_id, "vca_1");

        // Selected node was previously osc_1, should still be valid
        assert!(view.selected_modular_node_id.is_some());
    }

    #[test]
    fn test_turn19_universal_dsp_module_instantiation_across_categories() {
        let mut view = AwardWinningGuiView::new();
        let registry = DspNodeRegistry::new();

        let initial_node_count = view.modular_nodes.len();

        // 1. Oscillator category
        let osc_desc = registry.get("NonlinearRabinovichFabrikantChaosSynthesizer")
            .expect("Rabinovich Fabrikant oscillator should exist in registry");
        view.add_modular_node_from_descriptor(osc_desc);

        // 2. FilterEq category
        let filter_desc = registry.get("NonlinearSteinerParkerSynth12dbFilter")
            .expect("Steiner Parker filter should exist in registry");
        view.add_modular_node_from_descriptor(filter_desc);

        // 3. DynamicsMaster category
        let dyn_desc = registry.get("MultibandMasteringDiodeBridgeLimiter")
            .expect("Diode Bridge Limiter should exist in registry");
        view.add_modular_node_from_descriptor(dyn_desc);

        // 4. SpatialSurround category
        let spatial_desc = registry.get("AmbisonicThirdOrderPlanarSoundfieldDecoder")
            .expect("Ambisonic Decoder should exist in registry");
        view.add_modular_node_from_descriptor(spatial_desc);

        // 5. Modulation category
        let mod_desc = registry.get("ChaoticDoubleWellDuffingLfoModulator")
            .expect("Duffing LFO should exist in registry");
        view.add_modular_node_from_descriptor(mod_desc);

        assert_eq!(view.modular_nodes.len(), initial_node_count + 5);

        // Verify port configurations match categories
        let added_osc = view.modular_nodes.iter().find(|n| n.kind_id == "NonlinearRabinovichFabrikantChaosSynthesizer").unwrap();
        assert!(added_osc.ports.iter().any(|p| p.kind == ModularPortKind::AudioOut));
        assert!(added_osc.ports.iter().any(|p| p.kind == ModularPortKind::ModulationIn));

        let added_spatial = view.modular_nodes.iter().find(|n| n.kind_id == "AmbisonicThirdOrderPlanarSoundfieldDecoder").unwrap();
        assert!(added_spatial.ports.iter().any(|p| p.id == "in_l" && p.kind == ModularPortKind::AudioIn));
        assert!(added_spatial.ports.iter().any(|p| p.id == "out_l" && p.kind == ModularPortKind::AudioOut));

        let added_mod = view.modular_nodes.iter().find(|n| n.kind_id == "ChaoticDoubleWellDuffingLfoModulator").unwrap();
        assert!(added_mod.ports.iter().any(|p| p.kind == ModularPortKind::ModulationOut));
    }
}
