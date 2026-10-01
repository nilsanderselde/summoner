// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #40 / Turn #7 Verification Suite:
//! - Multi-Node Track DSP Device Chain model in Modern Device Rack
//! - Adding, removing, reordering, selecting, and bypassing devices in the rack chain
//! - Bidirectional synchronization between project TrackConfig nodes and GUI Device Rack chain
//! - Atomic live ParamBus dispatch for multi-node device parameters (Pro offset: track_id * 1000 + 500 + p_i)
//! - AwardWinningGuiView multi-node chain management methods (add, remove, move, select)

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn40_pure_param_bus_multi_node_pro_channel_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 2u32;

        // Pro node parameters mapped to track_id * 1000 + 500 + p_i
        let cutoff_pid = ParamId(track_id * 1000 + 500 + 0);
        let res_pid = ParamId(track_id * 1000 + 500 + 1);
        let drive_pid = ParamId(track_id * 1000 + 500 + 2);

        bus.register(cutoff_pid, 1000.0);
        bus.register(res_pid, 0.707);
        bus.register(drive_pid, 1.2);

        assert_eq!(bus.get(cutoff_pid), Some(1000.0));
        assert_eq!(bus.get(res_pid), Some(0.707));
        assert_eq!(bus.get(drive_pid), Some(1.2));

        // Real-time zero allocation atomic dispatch from GUI rack slider
        bus.set(cutoff_pid, 2400.0);
        assert_eq!(bus.get(cutoff_pid), Some(2400.0));
    }
}

#[cfg(all(test, feature = "gui"))]
mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack_with_context, ModernDeviceRackState};
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn40_pure_device_chain_initialization_and_ensure_chain() {
        let mut state = ModernDeviceRackState::default();
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
    fn test_turn40_pure_device_chain_add_remove_reorder() {
        let mut state = ModernDeviceRackState::default();
        state.ensure_chain(); // [0: OscillatorNode]

        // Add Filter and Chorus
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

        // Remove device 0 (OscillatorNode)
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
    }

    #[test]
    fn test_turn40_pure_device_chain_bypass_toggle() {
        let mut state = ModernDeviceRackState::default();
        state.ensure_chain();
        state.add_device("ReverbNode", "Algorithmic Reverb");

        assert!(!state.chain_devices[0].is_bypassed);
        assert!(!state.chain_devices[1].is_bypassed);

        // Toggle bypass on Reverb
        state.toggle_device_bypass(1);
        assert!(state.chain_devices[1].is_bypassed);
        assert!(!state.chain_devices[0].is_bypassed);

        // Toggle again to un-bypass
        state.toggle_device_bypass(1);
        assert!(!state.chain_devices[1].is_bypassed);
    }

    #[test]
    fn test_turn40_award_winning_gui_view_node_chain_manipulation() {
        let mut view = AwardWinningGuiView::default();
        assert!(!view.tracks.is_empty());

        // Add a node to active track
        let added1 = view.add_dsp_node_to_track("FilterLadder");
        assert!(added1);
        assert_eq!(view.device_rack_state.chain_devices.len(), 2);
        assert_eq!(view.selected_node_idx, 1);
        assert_eq!(view.device_rack_state.selected_chain_idx, 1);

        // Add another node
        let added2 = view.add_dsp_node_to_track("EffectChorus");
        assert!(added2);
        assert_eq!(view.device_rack_state.chain_devices.len(), 3);
        assert_eq!(view.selected_node_idx, 2);

        // Select node 0
        let sel_res = view.select_track_node(0);
        assert!(sel_res);
        assert_eq!(view.selected_node_idx, 0);
        assert_eq!(view.device_rack_state.selected_chain_idx, 0);

        // Move node 2 to index 1
        view.move_dsp_node(2, 1);
        assert_eq!(view.device_rack_state.chain_devices[1].kind, "EffectChorus");
        assert_eq!(view.device_rack_state.chain_devices[2].kind, "FilterLadder");

        // Remove node at index 1
        view.remove_dsp_node_from_track(1);
        assert_eq!(view.device_rack_state.chain_devices.len(), 2);
        assert_eq!(view.device_rack_state.chain_devices[1].kind, "FilterLadder");
    }

    #[test]
    fn test_turn40_award_winning_gui_view_sync_with_project_multi_node_chain() {
        let mut view = AwardWinningGuiView::default();
        let mut project = summoner_project::schema::ProjectConfig::default();
        let mut playhead = 0.0f64;
        let mut transport = false;
        let mut sel_track = None;

        let tr = summoner_project::schema::TrackConfig {
            id: 1,
            name: "Lead Synth".to_string(),
            gain: 1.0,
            pan: 0.0,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "OscillatorNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("waveform".to_string(), 1.0),
                        ("detune".to_string(), 0.05),
                    ]),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "LadderFilterNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("cutoff".to_string(), 0.65),
                        ("resonance".to_string(), 0.45),
                    ]),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "DelayNode".to_string(),
                    params: std::collections::HashMap::from([
                        ("time".to_string(), 0.35),
                        ("feedback".to_string(), 0.5),
                    ]),
                    plugin_state: None,
                },
            ],
            ..Default::default()
        };
        project.tracks.push(tr);

        // Sync project to GUI view
        view.sync_with_project(&mut project, &mut playhead, &mut transport, &mut sel_track);

        // Device rack should have reconciled 3 chain devices
        assert_eq!(view.device_rack_state.chain_devices.len(), 3);
        assert_eq!(view.device_rack_state.chain_devices[0].kind, "OscillatorNode");
        assert_eq!(view.device_rack_state.chain_devices[1].kind, "LadderFilterNode");
        assert_eq!(view.device_rack_state.chain_devices[2].kind, "DelayNode");

        // Selecting node 1 updates active parameters
        view.select_track_node(1);
        view.sync_with_project(&mut project, &mut playhead, &mut transport, &mut sel_track);

        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("LadderFilterNode"));
        assert_eq!(view.device_rack_state.cutoff, 0.65);
        assert_eq!(view.device_rack_state.resonance, 0.45);
    }

    #[test]
    fn test_turn40_modern_device_rack_rendering_with_chain() {
        let mut state = ModernDeviceRackState::default();
        state.ensure_chain();
        state.add_device("LadderFilterNode", "Ladder Filter");
        state.add_device("ReverbNode", "Algorithmic Reverb");

        let mut bus = ParamBus::new();
        let track_id = 1u64;

        // Register ParamBus pro parameters
        for p_i in 0..16 {
            bus.register(ParamId(track_id as u32 * 1000 + 500 + p_i), 0.5);
            bus.register(ParamId(track_id as u32 * 1000 + p_i), 0.5);
        }

        // Test rendering collapsed drawer
        state.is_minimized = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), track_id);
            });
        });

        // Test rendering expanded drawer with chain bar
        state.is_minimized = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), track_id);
            });
        });

        assert_eq!(state.chain_devices.len(), 3);
        assert_eq!(state.selected_chain_idx, 2);
    }
}
