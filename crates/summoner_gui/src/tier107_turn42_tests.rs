// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #42 / Turn #9 (Sprint Review) Verification Suite:
//! - Multi-Node Track DSP Device Chain Automation Bridge (Milestone 33)
//! - Multi-device visual model in TrackVisualData (device_names, device_kinds, ensure_devices, primary getters)
//! - ParamBus multi-node slot atomic addressing: ParamId(track_id * 1000 + slot * 20 + p_i)
//! - Track automation editor multi-node lane key parsing: "node_{slot}_{param}" -> "track_{tid}_node_{slot}_{param}"
//! - Timeline evaluation with multi-node slot lane keys and fallback to standard lane keys
//! - Arranger track header and Mixer channel strip DSP device insert chips in Pro mode

#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn42_pure_parambus_multi_node_slot_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 2u32;

        // Slot 0 (e.g. Synth): offset track_id * 1000 + 0 * 20 + p_i
        let slot0_param0 = ParamId(track_id * 1000 + 0 * 20 + 0);
        let slot0_param1 = ParamId(track_id * 1000 + 0 * 20 + 1);

        // Slot 1 (e.g. Filter): offset track_id * 1000 + 1 * 20 + p_i
        let slot1_param0 = ParamId(track_id * 1000 + 1 * 20 + 0);
        let slot1_param1 = ParamId(track_id * 1000 + 1 * 20 + 1);

        // Slot 2 (e.g. Delay/Reverb): offset track_id * 1000 + 2 * 20 + p_i
        let slot2_param0 = ParamId(track_id * 1000 + 2 * 20 + 0);

        // Pro compatibility offset: track_id * 1000 + 500 + p_i
        let pro_comp_pid = ParamId(track_id * 1000 + 500 + 0);

        bus.register(slot0_param0, 440.0);
        bus.register(slot0_param1, 0.7);
        bus.register(slot1_param0, 1200.0);
        bus.register(slot1_param1, 0.4);
        bus.register(slot2_param0, 0.25);
        bus.register(pro_comp_pid, 440.0);

        assert_eq!(bus.get(slot0_param0), Some(440.0));
        assert_eq!(bus.get(slot1_param0), Some(1200.0));
        assert_eq!(bus.get(slot2_param0), Some(0.25));
        assert_eq!(bus.get(pro_comp_pid), Some(440.0));

        // Atomic dispatch to slot 1
        bus.set(slot1_param0, 2400.0);
        assert_eq!(bus.get(slot1_param0), Some(2400.0));
        // Slot 0 and Pro comp untouched
        assert_eq!(bus.get(slot0_param0), Some(440.0));
        assert_eq!(bus.get(pro_comp_pid), Some(440.0));
    }

    #[test]
    fn test_turn42_pure_multi_node_automation_lane_key_parsing() {
        let track_id = 4u64;

        // Verify parsing helper matching open_track_automation_editor
        let parse_param = |raw: &str| -> (Option<usize>, String, String) {
            if let Some(rest) = raw.strip_prefix("node_") {
                if let Some((slot_str, p_id)) = rest.split_once('_') {
                    if let Ok(slot) = slot_str.parse::<usize>() {
                        let lane_key = format!("track_{}_node_{}_{}", track_id, slot, p_id);
                        return (Some(slot), p_id.to_string(), lane_key);
                    }
                }
            }
            (None, raw.to_string(), format!("track_{}_{}", track_id, raw))
        };

        // Standard parameter (no slot)
        let (slot, param, lane_key) = parse_param("gain");
        assert_eq!(slot, None);
        assert_eq!(param, "gain");
        assert_eq!(lane_key, "track_4_gain");

        // Multi-node slot 0 parameter
        let (slot, param, lane_key) = parse_param("node_0_cutoff");
        assert_eq!(slot, Some(0));
        assert_eq!(param, "cutoff");
        assert_eq!(lane_key, "track_4_node_0_cutoff");

        // Multi-node slot 2 parameter
        let (slot, param, lane_key) = parse_param("node_2_feedback");
        assert_eq!(slot, Some(2));
        assert_eq!(param, "feedback");
        assert_eq!(lane_key, "track_4_node_2_feedback");

        // Multi-node slot with complex name containing underscores
        let (slot, param, lane_key) = parse_param("node_1_stereo_spread");
        assert_eq!(slot, Some(1));
        assert_eq!(param, "stereo_spread");
        assert_eq!(lane_key, "track_4_node_1_stereo_spread");
    }

    #[test]
    fn test_turn42_pure_timeline_evaluation_with_slot_keys() {
        let mut timeline = AutomationTimeline::default();
        let track_id = 1u64;
        let slot_idx = 1usize;
        let slot_lane_key = format!("track_{}_node_{}_cutoff", track_id, slot_idx);
        let fallback_lane_key = format!("track_{}_cutoff", track_id);

        // Register automation lane for the slot device
        let mut points = Vec::new();
        points.push(AutomationPoint {
            beat: 0.0,
            value: 400.0,
            interp: Interpolation::Linear,
        });
        points.push(AutomationPoint {
            beat: 4.0,
            value: 2000.0,
            interp: Interpolation::Linear,
        });
        timeline.lanes.insert(
            slot_lane_key.clone(),
            AutomationLane {
                param_id: slot_lane_key.clone(),
                curve: AutomationCurve { points },
            },
        );

        // Evaluate at beat 2.0 (midway)
        let eval_slot = timeline
            .evaluate(&slot_lane_key, 2.0)
            .or_else(|| timeline.evaluate(&fallback_lane_key, 2.0));
        assert_eq!(eval_slot, Some(1200.0));

        // When evaluating at beat 0.0
        let eval_start = timeline
            .evaluate(&slot_lane_key, 0.0)
            .or_else(|| timeline.evaluate(&fallback_lane_key, 0.0));
        assert_eq!(eval_start, Some(400.0));

        // When slot lane is absent but fallback exists
        let mut fallback_points = Vec::new();
        fallback_points.push(AutomationPoint {
            beat: 0.0,
            value: 0.8,
            interp: Interpolation::Linear,
        });
        let fallback_pan_key = format!("track_{}_pan", track_id);
        timeline.lanes.insert(
            fallback_pan_key.clone(),
            AutomationLane {
                param_id: fallback_pan_key.clone(),
                curve: AutomationCurve { points: fallback_points },
            },
        );

        let unmapped_slot_key = format!("track_{}_node_0_pan", track_id);
        let eval_pan = timeline
            .evaluate(&unmapped_slot_key, 1.0)
            .or_else(|| timeline.evaluate(&fallback_pan_key, 1.0));
        assert_eq!(eval_pan, Some(0.8));
    }

    #[test]
    fn test_turn42_pure_project_track_nodes_multi_device_reconciliation() {
        use summoner_project::{NodeConfig, TrackConfig};

        let track = TrackConfig {
            id: 10,
            name: "Bass Lead".to_string(),
            nodes: vec![
                NodeConfig {
                    kind: "AnalogSynthNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
                NodeConfig {
                    kind: "LadderFilterNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
                NodeConfig {
                    kind: "ChorusNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
            ],
            ..TrackConfig::default()
        };

        // Extract device kinds and display names as AwardWinningGuiView::sync_with_project does
        let device_kinds: Vec<String> = track.nodes.iter().map(|n| n.kind.clone()).collect();
        let device_names: Vec<String> = track
            .nodes
            .iter()
            .map(|n| {
                let s = n.kind.trim_end_matches("Node");
                s.to_string()
            })
            .collect();

        assert_eq!(device_kinds.len(), 3);
        assert_eq!(device_kinds[0], "AnalogSynthNode");
        assert_eq!(device_kinds[1], "LadderFilterNode");
        assert_eq!(device_kinds[2], "ChorusNode");

        assert_eq!(device_names[0], "AnalogSynth");
        assert_eq!(device_names[1], "LadderFilter");
        assert_eq!(device_names[2], "Chorus");
    }
}

#[cfg(all(test, feature = "gui"))]
mod gui_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, TrackVisualData};

    #[test]
    fn test_turn42_track_visual_data_devices_and_helpers() {
        let mut track = TrackVisualData {
            id: 5,
            name: "Lead Synth".to_string(),
            color_rgb: [168, 85, 247],
            is_audio: false,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            clip_start_beat: 0.0,
            clip_length_beats: 16.0,
            clips: Vec::new(),
            active_clip_idx: None,
            device_names: Vec::new(),
            device_kinds: Vec::new(),
        };

        // Fallback when empty
        assert_eq!(track.primary_device_name(), "Synth");
        assert_eq!(track.primary_device_kind(), "AetherSynth");

        // ensure_devices populates default
        track.ensure_devices();
        assert_eq!(track.device_names.len(), 1);
        assert_eq!(track.device_kinds.len(), 1);
        assert_eq!(track.primary_device_name(), "AetherSynth");
        assert_eq!(track.primary_device_kind(), "AetherSynth");

        // Mutate with multi-device chain
        track.device_names = vec!["SuperSaw".to_string(), "Overdrive".to_string()];
        track.device_kinds = vec!["WavetableSynthNode".to_string(), "OverdriveNode".to_string()];
        assert_eq!(track.primary_device_name(), "SuperSaw");
        assert_eq!(track.primary_device_kind(), "WavetableSynthNode");
    }

    #[test]
    fn test_turn42_award_winning_gui_view_multi_node_automation_editor_open() {
        let mut view = AwardWinningGuiView::new();

        // Ensure default tracks have device chains populated
        assert!(!view.tracks.is_empty());
        assert!(!view.tracks[0].device_names.is_empty());

        // Open automation editor for standard track parameter
        let res_gain = view.open_track_automation_editor(view.tracks[0].id, "gain");
        assert!(res_gain);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some(&format!("track_{}_gain", view.tracks[0].id)[..]));

        // Open automation editor for multi-node slot parameter (e.g. node_0_cutoff)
        let res_slot = view.open_track_automation_editor(view.tracks[0].id, "node_0_cutoff");
        assert!(res_slot);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some(&format!("track_{}_node_0_cutoff", view.tracks[0].id)[..]));
    }
}
