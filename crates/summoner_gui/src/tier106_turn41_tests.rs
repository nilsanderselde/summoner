// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #41 / Turn #8 Verification Suite:
//! - Multi-Node Track DSP Device Chain model in Modern Inspector
//! - Interactive Device Chain strip in inspector (add, remove, reorder, bypass, select)
//! - Bidirectional synchronization between project TrackConfig nodes, GUI Device Rack, and Inspector
//! - Atomic live ParamBus multi-node parameter dispatch (Pro offset: track_id * 1000 + 500 + p_i, slot: track_id * 1000 + node_idx * 20 + p_i)
//! - AwardWinningGuiView multi-node chain lockstep between device rack and inspector

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn41_pure_inspector_parambus_multi_node_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 3u32;
        let node_idx = 2u32;

        // Pro node parameters mapped to track_id * 1000 + 500 + p_i
        let pro_cutoff_pid = ParamId(track_id * 1000 + 500 + 0);
        let pro_res_pid = ParamId(track_id * 1000 + 500 + 1);

        // Multi-node slot offset: track_id * 1000 + node_idx * 20 + p_i
        let slot_cutoff_pid = ParamId(track_id * 1000 + node_idx * 20 + 0);
        let slot_res_pid = ParamId(track_id * 1000 + node_idx * 20 + 1);

        bus.register(pro_cutoff_pid, 880.0);
        bus.register(pro_res_pid, 0.5);
        bus.register(slot_cutoff_pid, 880.0);
        bus.register(slot_res_pid, 0.5);

        assert_eq!(bus.get(pro_cutoff_pid), Some(880.0));
        assert_eq!(bus.get(slot_cutoff_pid), Some(880.0));

        // Atomic zero-allocation dispatch
        bus.set(pro_cutoff_pid, 1760.0);
        bus.set(slot_cutoff_pid, 1760.0);

        assert_eq!(bus.get(pro_cutoff_pid), Some(1760.0));
        assert_eq!(bus.get(slot_cutoff_pid), Some(1760.0));
    }
}

#[cfg(all(test, feature = "gui"))]
mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn41_pure_inspector_chain_initialization_and_ensure_chain() {
        let mut state = ModernInspectorState::default();
        assert!(state.chain_devices.is_empty());

        // ensure_chain populates default synth device if none present
        state.ensure_chain();
        assert_eq!(state.chain_devices.len(), 1);
        assert_eq!(state.chain_devices[0].kind, "AetherSynth");
        assert_eq!(state.selected_chain_idx, 0);
        assert!(!state.chain_devices[0].is_bypassed);

        // Calling again preserves existing devices
        state.ensure_chain();
        assert_eq!(state.chain_devices.len(), 1);
    }

    #[test]
    fn test_turn41_pure_inspector_chain_add_remove_reorder() {
        let mut state = ModernInspectorState::default();
        state.ensure_chain();

        // Add Ladder Filter and Chorus
        state.add_device("LadderFilterNode", "Ladder Filter");
        assert_eq!(state.chain_devices.len(), 2);
        assert_eq!(state.selected_chain_idx, 1);
        assert_eq!(state.chain_devices[1].kind, "LadderFilterNode");

        state.add_device("ChorusNode", "Stereo Chorus");
        assert_eq!(state.chain_devices.len(), 3);
        assert_eq!(state.selected_chain_idx, 2);
        assert_eq!(state.chain_devices[2].kind, "ChorusNode");

        // Move Chorus from index 2 to index 1
        state.move_device(2, 1);
        assert_eq!(state.chain_devices[1].kind, "ChorusNode");
        assert_eq!(state.chain_devices[2].kind, "LadderFilterNode");
        assert_eq!(state.selected_chain_idx, 1);

        // Select device 0
        state.select_device(0);
        assert_eq!(state.selected_chain_idx, 0);

        // Remove device 0 (AetherSynth)
        state.remove_device(0);
        assert_eq!(state.chain_devices.len(), 2);
        assert_eq!(state.chain_devices[0].kind, "ChorusNode");
        assert_eq!(state.chain_devices[1].kind, "LadderFilterNode");
        assert_eq!(state.selected_chain_idx, 0);

        // Select device 1 then remove device 1 -> index clamps to 0
        state.select_device(1);
        assert_eq!(state.selected_chain_idx, 1);
        state.remove_device(1);
        assert_eq!(state.chain_devices.len(), 1);
        assert_eq!(state.selected_chain_idx, 0);

        // Cannot remove the only remaining device
        assert!(!state.remove_device(0));
        assert_eq!(state.chain_devices.len(), 1);
    }

    #[test]
    fn test_turn41_pure_inspector_chain_bypass_toggle() {
        let mut state = ModernInspectorState::default();
        state.ensure_chain();
        state.add_device("ReverbNode", "Algorithmic Reverb");

        assert!(!state.chain_devices[0].is_bypassed);
        assert!(!state.chain_devices[1].is_bypassed);
        assert!(!state.is_selected_bypassed());

        // Toggle bypass on active device (ReverbNode at index 1)
        state.toggle_device_bypass(1);
        assert!(state.chain_devices[1].is_bypassed);
        assert!(!state.chain_devices[0].is_bypassed);
        assert!(state.is_selected_bypassed());

        // Toggle again to un-bypass
        state.toggle_device_bypass(1);
        assert!(!state.chain_devices[1].is_bypassed);
        assert!(!state.is_selected_bypassed());
    }

    #[test]
    fn test_turn41_pure_inspector_active_device_queries() {
        let mut state = ModernInspectorState::default();
        state.ensure_chain();
        state.add_device("CombFilterNode", "Comb Filter");

        assert_eq!(state.selected_device_kind(), Some("CombFilterNode"));
        assert_eq!(state.selected_device_name(), Some("Comb Filter"));
        assert!(!state.is_selected_bypassed());

        state.select_device(0);
        assert_eq!(state.selected_device_kind(), Some("AetherSynth"));
        assert_eq!(state.selected_device_name(), Some("Synth 1"));
    }

    #[test]
    fn test_turn41_award_winning_gui_view_inspector_rack_sync() {
        let mut view = AwardWinningGuiView::default();
        assert!(!view.tracks.is_empty());

        // Add a node to active track
        let added1 = view.add_dsp_node_to_track("FilterLadder");
        assert!(added1);
        assert_eq!(view.device_rack_state.chain_devices.len(), 2);
        assert_eq!(view.inspector_state.chain_devices.len(), 2);
        assert_eq!(view.selected_node_idx, 1);
        assert_eq!(view.inspector_state.selected_chain_idx, 1);
        assert_eq!(view.device_rack_state.selected_chain_idx, 1);

        // Add another node
        let added2 = view.add_dsp_node_to_track("EffectChorus");
        assert!(added2);
        assert_eq!(view.device_rack_state.chain_devices.len(), 3);
        assert_eq!(view.inspector_state.chain_devices.len(), 3);
        assert_eq!(view.selected_node_idx, 2);
        assert_eq!(view.inspector_state.selected_chain_idx, 2);

        // Select node 0
        let sel_res = view.select_track_node(0);
        assert!(sel_res);
        assert_eq!(view.selected_node_idx, 0);
        assert_eq!(view.device_rack_state.selected_chain_idx, 0);
        assert_eq!(view.inspector_state.selected_chain_idx, 0);

        // Move node 2 to index 1
        view.move_dsp_node(2, 1);
        assert_eq!(view.device_rack_state.chain_devices[1].kind, "EffectChorus");
        assert_eq!(view.inspector_state.chain_devices[1].kind, "EffectChorus");

        // Remove node at index 1
        view.remove_dsp_node_from_track(1);
        assert_eq!(view.device_rack_state.chain_devices.len(), 2);
        assert_eq!(view.inspector_state.chain_devices.len(), 2);
    }

    #[test]
    fn test_turn41_modern_inspector_rendering_with_multi_node_chain() {
        let mut state = ModernInspectorState::default();
        state.ensure_chain();
        state.add_device("FilterLadder", "Ladder Filter");
        state.add_device("EffectReverb", "Algorithmic Reverb");

        let mut bus = ParamBus::new();
        let track_id = 1u64;

        // Register ParamBus parameters for all devices
        for p_i in 0..16 {
            bus.register(ParamId(track_id as u32 * 1000 + 500 + p_i), 0.5);
            bus.register(ParamId(track_id as u32 * 1000 + p_i), 0.5);
            for node_i in 0..3 {
                bus.register(ParamId(track_id as u32 * 1000 + node_i * 20 + p_i), 0.5);
            }
        }

        // Test rendering collapsed
        state.is_collapsed = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, Some(&bus), track_id);
            });
        });

        // Test rendering expanded with multi-device chain strip
        state.is_collapsed = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, Some(&bus), track_id);
            });
        });

        assert_eq!(state.chain_devices.len(), 3);
        assert_eq!(state.selected_chain_idx, 2);
        assert!(!state.node_param_values.is_empty());
    }

    #[test]
    fn test_turn41_inspector_sync_with_project_multi_node_chain() {
        let mut view = AwardWinningGuiView::default();
        let mut project = summoner_project::schema::ProjectConfig::default();
        let mut playhead = 0.0f64;
        let mut transport = false;
        let mut sel_track = None;

        let tr = summoner_project::schema::TrackConfig {
            id: 1,
            name: "Pluck Bass".to_string(),
            gain: 1.0,
            pan: 0.0,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "OscillatorNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("waveform".to_string(), 1.0),
                        ("detune".to_string(), 0.02),
                    ]),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "LadderFilterNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("cutoff".to_string(), 0.70),
                        ("resonance".to_string(), 0.50),
                    ]),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "ChorusNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("rate".to_string(), 0.4),
                        ("depth".to_string(), 0.6),
                    ]),
                    plugin_state: None,
                },
            ],
            ..Default::default()
        };
        project.tracks.push(tr);

        // Sync project to GUI view
        view.sync_with_project(&mut project, &mut playhead, &mut transport, &mut sel_track);

        // Both device rack and inspector should have reconciled 3 chain devices
        assert_eq!(view.device_rack_state.chain_devices.len(), 3);
        assert_eq!(view.inspector_state.chain_devices.len(), 3);
        assert_eq!(view.inspector_state.chain_devices[0].kind, "OscillatorNode");
        assert_eq!(view.inspector_state.chain_devices[1].kind, "LadderFilterNode");
        assert_eq!(view.inspector_state.chain_devices[2].kind, "ChorusNode");

        // Selecting node 1 in inspector reflects in view and project
        view.select_track_node(1);
        view.sync_with_project(&mut project, &mut playhead, &mut transport, &mut sel_track);

        assert_eq!(view.inspector_state.selected_chain_idx, 1);
        assert_eq!(view.device_rack_state.selected_chain_idx, 1);
        assert_eq!(view.selected_node_idx, 1);
    }
}
