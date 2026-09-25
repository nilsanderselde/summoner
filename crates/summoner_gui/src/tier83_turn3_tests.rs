// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Tier 83 (Turn #3) System & GUI Tests:
//! - Dynamic `summoner_core::graph::NodeGraph` dynamic node insertion & patch cord wiring
//! - Modular canvas real-time block rendering through lock-free schedule
//! - Arranger clip split/slice/crossfade visual editing ergonomics

#[cfg(test)]
pub mod pure_tests {
    use summoner_project::schema::{SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_sequence_split_clip_pure() {
        let mut seq = SequenceConfig {
            start_beat: 4.0,
            step_division: 0.25,
            clip_color: Some([26, 140, 255]),
            clip_name: Some("Guitar Riff".to_string()),
            name: "Guitar Riff".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig { note: 60.0, ..Default::default() },
                TrackerStepConfig { note: 62.0, ..Default::default() },
                TrackerStepConfig { note: 64.0, ..Default::default() },
                TrackerStepConfig { note: 65.0, ..Default::default() },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            ..Default::default()
        };

        // Split at beat 0.5 (2 steps)
        let step_idx = (0.5 / seq.step_division).round() as usize;
        let remaining_steps = seq.steps.split_off(step_idx);
        let mut new_seq = seq.clone();
        new_seq.steps = remaining_steps;
        new_seq.start_beat = seq.start_beat + (step_idx as f64 * seq.step_division);

        assert_eq!(seq.steps.len(), 2);
        assert_eq!(new_seq.steps.len(), 2);
        assert_eq!(new_seq.start_beat, 4.5);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::{
        ArrangerClipVisual, ArrangerToolMode, AwardWinningGuiView, TrackVisualData,
    };
    use summoner_core::node::ProcessContext;
    use summoner_project::schema::{SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_modular_canvas_initial_graph_wiring() {
        let view = AwardWinningGuiView::new();
        let graph_arc = view.audio_graph.clone();
        let graph = graph_arc.lock().unwrap();

        // Initial setup should have 4 nodes (OscSaw, FilterSvf, EnvAdsr, VcaNode)
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(view.modular_nodes.len(), 4);

        // Initial patch cords: (osc->filter), (filter->vca), (env->vca)
        assert_eq!(graph.edges.len(), 3);
        assert_eq!(view.patch_cords.len(), 3);

        // Schedule must be compiled without cycles
        let schedule = graph.schedule.load();
        assert!(!schedule.has_cycle, "Modular canvas graph must be acyclic");
        assert_eq!(schedule.evaluation_order.len(), 4);
    }

    #[test]
    fn test_modular_canvas_add_node_dynamic_insertion() {
        let mut view = AwardWinningGuiView::new();
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let desc = registry
            .get("AetherSynth")
            .expect("AetherSynth descriptor should exist");

        view.add_modular_node_from_descriptor(desc);

        assert_eq!(view.modular_nodes.len(), 5);
        let last_node = view.modular_nodes.last().unwrap();
        assert_eq!(last_node.kind_id, "AetherSynth");
        assert!(last_node.graph_node_idx.is_some());

        let graph = view.audio_graph.lock().unwrap();
        assert_eq!(graph.nodes.len(), 5);
        let schedule = graph.schedule.load();
        assert!(!schedule.has_cycle);
    }

    #[test]
    fn test_modular_canvas_connect_and_disconnect_patch_cords() {
        let mut view = AwardWinningGuiView::new();

        // Initially 3 edges
        {
            let g = view.audio_graph.lock().unwrap();
            assert_eq!(g.edges.len(), 3);
        }

        // Connect osc_1 out to vca_1 audio directly
        view.connect_patch_cord_in_graph("osc_1", "out", "vca_1", "audio");
        {
            let g = view.audio_graph.lock().unwrap();
            assert_eq!(g.edges.len(), 4);
            let sched = g.schedule.load();
            assert!(!sched.has_cycle);
        }

        // Disconnect the newly added cord
        view.disconnect_patch_cord_in_graph("osc_1", "out", "vca_1", "audio");
        {
            let g = view.audio_graph.lock().unwrap();
            assert_eq!(g.edges.len(), 3);
        }
    }

    #[test]
    fn test_modular_canvas_live_audio_rendering() {
        let view = AwardWinningGuiView::new();
        let ctx = ProcessContext::new(44100, 120.0, 0);

        let mut out_l = vec![0.0f32; 256];
        let mut out_r = vec![0.0f32; 256];

        view.process_modular_graph(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        // SineOscillatorNode at index 0 connected through filters and VCAs produces sound
        let has_signal = out_l.iter().any(|&s| s.abs() > 1e-6) || out_r.iter().any(|&s| s.abs() > 1e-6);
        assert!(has_signal, "Modular audio graph should render live audio");
    }

    #[test]
    fn test_track_visual_data_split_clip_at_beat() {
        let mut track = TrackVisualData {
            id: 10,
            name: "Lead Synth".to_string(),
            color_rgb: [56, 189, 248],
            is_audio: false,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: true,
            clip_start_beat: 0.0,
            clip_length_beats: 16.0,
            clips: Vec::new(),
            active_clip_idx: None,
        };

        // Split at beat 6.0
        let success = track.split_clip_at_beat(1, 6.0);
        assert!(success, "Splitting clip within bounds must succeed");
        assert_eq!(track.clips.len(), 2);

        // First clip: 0.0 .. 6.0
        assert_eq!(track.clips[0].start_beat, 0.0);
        assert_eq!(track.clips[0].length_beats, 6.0);
        assert!(track.clips[0].fade_out > 0.0);

        // Second clip: 6.0 .. 16.0 (length 10.0)
        assert_eq!(track.clips[1].start_beat, 6.0);
        assert_eq!(track.clips[1].length_beats, 10.0);
        assert!(track.clips[1].fade_in > 0.0);
        assert_eq!(track.clips[1].name, "Lead Synth (2)");
    }

    #[test]
    fn test_track_visual_data_crossfade_detection() {
        let track = TrackVisualData {
            id: 11,
            name: "Ambient Drone".to_string(),
            color_rgb: [16, 185, 129],
            is_audio: true,
            gain: 0.9,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            clip_start_beat: 0.0,
            clip_length_beats: 8.0,
            clips: vec![
                ArrangerClipVisual {
                    id: 1,
                    name: "Drone A".to_string(),
                    start_beat: 0.0,
                    length_beats: 8.0,
                    fade_in: 0.0,
                    fade_out: 1.0,
                    gain: 1.0,
                    is_selected: false,
                },
                ArrangerClipVisual {
                    id: 2,
                    name: "Drone B".to_string(),
                    start_beat: 6.0, // overlaps Drone A by 2 beats (6.0 .. 8.0)
                    length_beats: 8.0,
                    fade_in: 1.0,
                    fade_out: 0.0,
                    gain: 1.0,
                    is_selected: false,
                },
            ],
            active_clip_idx: None,
        };

        let xfades = track.detect_crossfades();
        assert_eq!(xfades.len(), 1);
        let (start, end, c1, c2) = xfades[0];
        assert_eq!(start, 6.0);
        assert_eq!(end, 8.0);
        assert_eq!(c1, 1);
        assert_eq!(c2, 2);
    }

    #[test]
    fn test_arranger_split_clip_at_returns_second_half() {
        let mut seq = SequenceConfig {
            start_beat: 4.0,
            step_division: 0.25,
            clip_color: Some([26, 140, 255]),
            clip_name: Some("Guitar Riff".to_string()),
            name: "Guitar Riff".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig { note: 60.0, ..Default::default() },
                TrackerStepConfig { note: 62.0, ..Default::default() },
                TrackerStepConfig { note: 64.0, ..Default::default() },
                TrackerStepConfig { note: 65.0, ..Default::default() },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            ..Default::default()
        };

        let second_half = crate::views::arranger::split_clip_at(&mut seq, 0.5);
        assert!(second_half.is_some());
        let part2 = second_half.unwrap();

        // First half should have 2 steps (beat 4.0 .. 4.5)
        assert_eq!(seq.steps.len(), 2);
        assert_eq!(seq.start_beat, 4.0);

        // Second half should have remaining 2 steps (starts at beat 4.5)
        assert_eq!(part2.steps.len(), 2);
        assert_eq!(part2.start_beat, 4.5);
        assert_eq!(part2.clip_name.as_deref(), Some("Guitar Riff (Part 2)"));
    }

    #[test]
    fn test_arranger_tool_mode_selection() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.arranger_tool_mode, ArrangerToolMode::Pointer);

        view.arranger_tool_mode = ArrangerToolMode::Slice;
        assert_eq!(view.arranger_tool_mode, ArrangerToolMode::Slice);

        view.arranger_tool_mode = ArrangerToolMode::Fade;
        assert_eq!(view.arranger_tool_mode, ArrangerToolMode::Fade);
    }
}
