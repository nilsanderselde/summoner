// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #43 / Turn #10 Verification Suite:
//! - Master Bus DSP Device Insert Processing Chain & Lifecycle (Milestone 33)
//! - Console Mixer Multi-Slot Insert Chips (Slots 0, 1 / [+ FX]) with 1-click select & secondary-click automation
//! - Master Bus Mixer Insert Chips (Master EQ, Limiter) with 1-click select & secondary-click automation
//! - Modular Canvas "🔗 Track Chain ➔ Modular" DAG Synchronization
//! - Stage Performance Matrix Compact DSP Device Badges ([🎛 {d_short}])
//! - Lock-free ParamBus Atomic Channel Addressing for Master DSP: ParamId(9000 + slot * 20 + p_i)
//! - Real-time Master Device Automation Curve Evaluation & Dispatch

#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn43_pure_parambus_master_dsp_slot_offsets() {
        let mut bus = ParamBus::new();

        // Master Bus DSP slots use ParamId(9000 + slot * 20 + p_i)
        // Slot 0 (e.g. Master EQ): 9000..9019
        let master_slot0_cutoff = ParamId(9000 + 0 * 20 + 0);
        let master_slot0_gain = ParamId(9000 + 0 * 20 + 1);

        // Slot 1 (e.g. True Peak Limiter): 9020..9039
        let master_slot1_threshold = ParamId(9000 + 1 * 20 + 0);
        let master_slot1_release = ParamId(9000 + 1 * 20 + 1);

        // Track 1 DSP Slot 0: ParamId(1000 + 0 * 20 + 0) = 1000
        let track1_slot0_cutoff = ParamId(1000 + 0 * 20 + 0);

        // Master Gain: ParamId(9999), Master Mono: ParamId(9998)
        let master_gain_pid = ParamId(9999);
        let master_mono_pid = ParamId(9998);

        bus.register(master_slot0_cutoff, 1000.0);
        bus.register(master_slot0_gain, 0.0);
        bus.register(master_slot1_threshold, -0.5);
        bus.register(master_slot1_release, 100.0);
        bus.register(track1_slot0_cutoff, 440.0);
        bus.register(master_gain_pid, 1.0);
        bus.register(master_mono_pid, 0.0);

        assert_eq!(bus.get(master_slot0_cutoff), Some(1000.0));
        assert_eq!(bus.get(master_slot1_threshold), Some(-0.5));
        assert_eq!(bus.get(track1_slot0_cutoff), Some(440.0));
        assert_eq!(bus.get(master_gain_pid), Some(1.0));
        assert_eq!(bus.get(master_mono_pid), Some(0.0));

        // Atomic lock-free dispatch to Master EQ and Limiter
        bus.set(master_slot0_cutoff, 2500.0);
        bus.set(master_slot1_threshold, -1.2);

        assert_eq!(bus.get(master_slot0_cutoff), Some(2500.0));
        assert_eq!(bus.get(master_slot1_threshold), Some(-1.2));

        // Verify isolation: track params and master gain remain unaltered
        assert_eq!(bus.get(track1_slot0_cutoff), Some(440.0));
        assert_eq!(bus.get(master_gain_pid), Some(1.0));
        assert_eq!(bus.get(master_mono_pid), Some(0.0));
    }

    #[test]
    fn test_turn43_pure_master_node_automation_lane_key_parsing() {
        // Parsing logic matching open_track_automation_editor & sync_with_param_bus
        let parse_param_lane = |raw: &str, track_id: u64| -> (Option<usize>, String, String, bool) {
            if raw == "master_gain" {
                return (None, "master_gain".to_string(), "master_gain".to_string(), true);
            }
            if let Some(rest) = raw.strip_prefix("master_node_") {
                if let Some((slot_str, p_id)) = rest.split_once('_') {
                    if let Ok(slot) = slot_str.parse::<usize>() {
                        let lane_key = format!("master_node_{}_{}", slot, p_id);
                        return (Some(slot), p_id.to_string(), lane_key, true);
                    }
                }
            }
            if let Some(rest) = raw.strip_prefix("node_") {
                if let Some((slot_str, p_id)) = rest.split_once('_') {
                    if let Ok(slot) = slot_str.parse::<usize>() {
                        let lane_key = format!("track_{}_node_{}_{}", track_id, slot, p_id);
                        return (Some(slot), p_id.to_string(), lane_key, false);
                    }
                }
            }
            (None, raw.to_string(), format!("track_{}_{}", track_id, raw), false)
        };

        // Master Gain
        let (slot, param, lane_key, is_master) = parse_param_lane("master_gain", 0);
        assert_eq!(slot, None);
        assert_eq!(param, "master_gain");
        assert_eq!(lane_key, "master_gain");
        assert!(is_master);

        // Master Node Slot 0 Cutoff
        let (slot, param, lane_key, is_master) = parse_param_lane("master_node_0_cutoff", 0);
        assert_eq!(slot, Some(0));
        assert_eq!(param, "cutoff");
        assert_eq!(lane_key, "master_node_0_cutoff");
        assert!(is_master);

        // Master Node Slot 1 Threshold
        let (slot, param, lane_key, is_master) = parse_param_lane("master_node_1_threshold", 0);
        assert_eq!(slot, Some(1));
        assert_eq!(param, "threshold");
        assert_eq!(lane_key, "master_node_1_threshold");
        assert!(is_master);

        // Track 3 Node Slot 1 Resonance
        let (slot, param, lane_key, is_master) = parse_param_lane("node_1_resonance", 3);
        assert_eq!(slot, Some(1));
        assert_eq!(param, "resonance");
        assert_eq!(lane_key, "track_3_node_1_resonance");
        assert!(!is_master);
    }

    #[test]
    fn test_turn43_pure_master_device_timeline_evaluation() {
        let mut timeline = AutomationTimeline::default();

        // Create an automation lane for Master EQ Cutoff: "master_node_0_cutoff"
        let mut points = Vec::new();
        points.push(AutomationPoint {
            beat: 0.0,
            value: 500.0,
            interp: Interpolation::Linear,
        });
        points.push(AutomationPoint {
            beat: 4.0,
            value: 3000.0,
            interp: Interpolation::Linear,
        });

        timeline.lanes.insert(
            "master_node_0_cutoff".to_string(),
            AutomationLane {
                param_id: "master_node_0_cutoff".to_string(),
                curve: AutomationCurve { points },
            },
        );

        // Evaluate at beat 0.0, 2.0, 4.0
        let val_0 = timeline.evaluate("master_node_0_cutoff", 0.0).unwrap_or(0.0);
        let val_2 = timeline.evaluate("master_node_0_cutoff", 2.0).unwrap_or(0.0);
        let val_4 = timeline.evaluate("master_node_0_cutoff", 4.0).unwrap_or(0.0);

        assert!((val_0 - 500.0).abs() < 1e-3);
        assert!((val_2 - 1750.0).abs() < 1e-3);
        assert!((val_4 - 3000.0).abs() < 1e-3);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, TrackVisualData};
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn43_gui_track_visual_data_multi_device_helpers() {
        let mut tr = TrackVisualData {
            id: 1,
            name: "Lead Synth".to_string(),
            color_rgb: [56, 189, 248],
            is_audio: true,
            gain: 0.9,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: true,
            clip_start_beat: 0.0,
            clip_length_beats: 8.0,
            clips: Vec::new(),
            active_clip_idx: None,
            device_names: vec!["AetherSynth".to_string(), "FilterPro".to_string(), "AnalogDelay".to_string()],
            device_kinds: vec!["AetherSynth".to_string(), "FilterPro".to_string(), "AnalogDelay".to_string()],
        };

        assert_eq!(tr.device_count(), 3);
        assert_eq!(tr.device_at(0), Some(("AetherSynth", "AetherSynth")));
        assert_eq!(tr.device_at(1), Some(("FilterPro", "FilterPro")));
        assert_eq!(tr.device_at(2), Some(("AnalogDelay", "AnalogDelay")));
        assert_eq!(tr.device_at(3), None);

        assert_eq!(tr.device_name_at(0), Some("AetherSynth"));
        assert_eq!(tr.device_name_at(1), Some("FilterPro"));
        assert_eq!(tr.device_name_at(2), Some("AnalogDelay"));
        assert_eq!(tr.device_name_at(3), None);

        assert_eq!(tr.device_kind_at(0), Some("AetherSynth"));
        assert_eq!(tr.device_kind_at(1), Some("FilterPro"));
        assert_eq!(tr.device_kind_at(2), Some("AnalogDelay"));
        assert_eq!(tr.device_kind_at(3), None);

        // Mutate devices and verify
        tr.device_names.pop();
        tr.device_kinds.pop();
        assert_eq!(tr.device_count(), 2);
        assert_eq!(tr.device_name_at(2), None);
    }

    #[test]
    fn test_turn43_gui_master_chain_defaults_and_selection() {
        let mut view = AwardWinningGuiView::new();

        // Verify Master Bus devices initialized
        assert_eq!(view.master_chain_devices.len(), 2);
        assert_eq!(view.master_chain_devices[0].kind, "ParametricEq");
        assert_eq!(view.master_chain_devices[0].display_name, "Master EQ");
        assert_eq!(view.master_chain_devices[1].kind, "TruePeakLimiter");
        assert_eq!(view.master_chain_devices[1].display_name, "True Peak Limiter");
        assert_eq!(view.selected_master_chain_idx, 0);

        // Selection
        assert!(view.select_master_node(1));
        assert_eq!(view.selected_master_chain_idx, 1);
        assert!(!view.select_master_node(99)); // Out of bounds returns false

        // Toggle bypass
        assert!(view.toggle_master_node_bypass(1));
        assert!(view.master_chain_devices[1].is_bypassed);
        assert!(view.toggle_master_node_bypass(1));
        assert!(!view.master_chain_devices[1].is_bypassed);

        // Add node to master
        let new_idx = view.add_dsp_node_to_master("AnalogCompressor");
        assert_eq!(view.master_chain_devices.len(), 3);
        assert_eq!(view.selected_master_chain_idx, new_idx);
        assert_eq!(view.master_chain_devices[new_idx].kind, "AnalogCompressor");

        // Move node in master
        assert!(view.move_dsp_node_in_master(2, 1));
        assert_eq!(view.master_chain_devices[1].kind, "AnalogCompressor");
        assert_eq!(view.selected_master_chain_idx, 1);

        // Remove node from master
        assert!(view.remove_dsp_node_from_master(1));
        assert_eq!(view.master_chain_devices.len(), 2);
    }

    #[test]
    fn test_turn43_gui_sync_track_chain_to_modular() {
        let mut view = AwardWinningGuiView::new();

        // Configure track 0 with 3-node device chain
        view.tracks[0].device_names = vec!["AetherSynth".to_string(), "ResonantFilter".to_string(), "DigitalDelay".to_string()];
        view.tracks[0].device_kinds = vec!["AetherSynth".to_string(), "ResonantFilter".to_string(), "DigitalDelay".to_string()];
        view.select_track(0);
        assert_eq!(view.device_rack_state.chain_devices.len(), 3);

        // Sync track chain to modular canvas
        assert_eq!(view.sync_track_chain_to_modular(), 3);

        // Modular canvas should now contain 3 nodes corresponding to the track's chain
        assert_eq!(view.modular_nodes.len(), 3);
        assert_eq!(view.modular_nodes[0].display_name, "1. AetherSynth");
        assert_eq!(view.modular_nodes[1].display_name, "2. ResonantFilter");
        assert_eq!(view.modular_nodes[2].display_name, "3. DigitalDelay");

        // Sequential audio patch cords should connect node 0 -> node 1 and node 1 -> node 2
        assert_eq!(view.patch_cords.len(), 2);
        assert_eq!(view.patch_cords[0].from_node_id, view.modular_nodes[0].id);
        assert_eq!(view.patch_cords[0].to_node_id, view.modular_nodes[1].id);
        assert!(view.patch_cords[0].is_audio);
        assert_eq!(view.patch_cords[1].from_node_id, view.modular_nodes[1].id);
        assert_eq!(view.patch_cords[1].to_node_id, view.modular_nodes[2].id);
        assert!(view.patch_cords[1].is_audio);
    }

    #[test]
    fn test_turn43_gui_open_master_device_automation_editor() {
        let mut view = AwardWinningGuiView::new();

        // Open master device automation editor for slot 0 cutoff
        assert!(view.open_master_device_automation_editor(0, "cutoff"));
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_node_0_cutoff"));
        assert!(view.automation_editor.is_some());
        assert_eq!(view.selected_master_chain_idx, 0);

        // Open master device automation editor for slot 1 threshold
        assert!(view.open_master_device_automation_editor(1, "threshold"));
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_node_1_threshold"));
        assert!(view.automation_editor.is_some());
        assert_eq!(view.selected_master_chain_idx, 1);
    }

    #[test]
    fn test_turn43_gui_parambus_master_device_live_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let project = summoner_project::schema::ProjectConfig::default();
        let mut param_bus = ParamBus::new();
        let mut automation_registry = AutomationRegistry::new();
        let mut automation_timeline = AutomationTimeline::default();

        // Set up ParamBus registration for Master EQ Cutoff (slot 0, param 0)
        let master_eq_cutoff_pid = ParamId(9000 + 0 * 20 + 0);
        param_bus.register(master_eq_cutoff_pid, 1000.0);

        // Add automation curve to timeline for "master_node_0_cutoff"
        let mut points = Vec::new();
        points.push(AutomationPoint {
            beat: 0.0,
            value: 800.0,
            interp: Interpolation::Linear,
        });
        points.push(AutomationPoint {
            beat: 4.0,
            value: 4800.0,
            interp: Interpolation::Linear,
        });
        automation_timeline.lanes.insert(
            "master_node_0_cutoff".to_string(),
            AutomationLane {
                param_id: "master_node_0_cutoff".to_string(),
                curve: AutomationCurve { points },
            },
        );

        // Sync at beat 0.0
        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut automation_registry,
            &mut automation_timeline,
            0.0,
            false,
        );
        let val_at_0 = param_bus.get(master_eq_cutoff_pid).unwrap();
        assert!((val_at_0 - 800.0).abs() < 1e-3);

        // Sync at beat 2.0
        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut automation_registry,
            &mut automation_timeline,
            2.0,
            false,
        );
        let val_at_2 = param_bus.get(master_eq_cutoff_pid).unwrap();
        assert!((val_at_2 - 2800.0).abs() < 1e-3);

        // Sync at beat 4.0
        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut automation_registry,
            &mut automation_timeline,
            4.0,
            false,
        );
        let val_at_4 = param_bus.get(master_eq_cutoff_pid).unwrap();
        assert!((val_at_4 - 4800.0).abs() < 1e-3);
    }
}
