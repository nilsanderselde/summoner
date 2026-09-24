// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Milestone 33: Real-Time Parameter Automation Bridge & Multi-Track Audio Streaming Tests.

#[cfg(all(test, feature = "gui"))]
mod tests {
    use std::sync::Arc;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::create_default_project;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };
    use crate::app::SummonerApp;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;

    #[test]
    fn test_m33_param_bus_live_sync() {
        let project = create_default_project("M33 Sync Session");
        let mut bus = ParamBus::new();

        // Pre-register track 1 params
        for p in 0..64 {
            bus.register(ParamId(1000 + p), 0.0);
        }
        for p in 0..16 {
            bus.register(ParamId(1100 + p), 0.0);
        }

        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        let mut view = AwardWinningGuiView::default();
        view.selected_track_idx = 0; // Track ID 1

        // User tweaks dials in the GUI
        view.device_rack_state.cutoff = 0.88;
        view.device_rack_state.resonance = 0.72;
        view.device_rack_state.decay = 0.45;
        view.device_rack_state.drive = 0.63;
        view.device_rack_state.volume = 0.91;
        view.device_rack_state
            .node_param_values
            .insert("frequency".to_string(), 0.77);

        // Sync with ParamBus
        view.sync_with_param_bus(&project, &bus, &mut registry, &mut timeline, 0.0, false);

        // Verify standard parameters propagated to ParamBus
        assert_eq!(bus.get(ParamId(1100)), Some(0.88)); // Cutoff
        assert_eq!(bus.get(ParamId(1101)), Some(0.72)); // Resonance
        assert_eq!(bus.get(ParamId(1102)), Some(0.45)); // Decay
        assert_eq!(bus.get(ParamId(1105)), Some(0.63)); // Drive
        assert_eq!(bus.get(ParamId(1106)), Some(0.91)); // Volume

        // Verify registered in automation registry
        assert!(registry.get_param("track_1_cutoff").is_some());
        assert_eq!(registry.get_param("track_1_cutoff").unwrap().get(), 0.88);
    }

    #[test]
    fn test_m33_project_nodes_bidirectional_sync() {
        let mut project = create_default_project("M33 Project Sync");
        let mut view = AwardWinningGuiView::default();
        let mut playhead = 0.0;
        let mut playing = false;
        let mut selected_id = Some(1);

        // Initial sync
        view.sync_with_project(&mut project, &mut playhead, &mut playing, &mut selected_id);

        // Tweak GUI state
        view.device_rack_state.selected_node_kind = Some("FilterLadder".to_string());
        view.device_rack_state.cutoff = 0.82;
        view.device_rack_state.resonance = 0.64;
        view.device_rack_state
            .node_param_values
            .insert("drive".to_string(), 0.55);

        // Sync back to project
        view.sync_with_project(&mut project, &mut playhead, &mut playing, &mut selected_id);

        let track = &project.tracks[0];
        assert_eq!(track.nodes[0].kind, "FilterLadder");
        assert_eq!(track.nodes[0].params.get("cutoff"), Some(&0.82));
        assert_eq!(track.nodes[0].params.get("resonance"), Some(&0.64));
        assert_eq!(track.nodes[0].params.get("drive"), Some(&0.55));
    }

    #[test]
    fn test_m33_automation_timeline_playback_and_recording() {
        let project = create_default_project("M33 Auto Session");
        let bus = ParamBus::new();
        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // 1. Setup automation curve on track 1 cutoff: 0.0 at beat 0, 1.0 at beat 4.0
        let curve = AutomationCurve::new(vec![
            AutomationPoint {
                beat: 0.0,
                value: 0.2,
                interp: Interpolation::Linear,
            },
            AutomationPoint {
                beat: 4.0,
                value: 0.8,
                interp: Interpolation::Linear,
            },
        ]);
        timeline.add_lane(AutomationLane {
            param_id: "track_1_cutoff".to_string(),
            curve,
        });

        let mut view = AwardWinningGuiView::default();
        view.selected_track_idx = 0;
        view.top_bar_state.is_playing = true;

        // Evaluate at beat 2.0 (midpoint => 0.5)
        view.sync_with_param_bus(&project, &bus, &mut registry, &mut timeline, 2.0, false);
        let cutoff_val = view.device_rack_state.cutoff;
        assert!((cutoff_val - 0.5).abs() < 1e-4, "Expected ~0.5, got {}", cutoff_val);

        // 2. Automation recording test: User tweaks decay to 0.95 at beat 3.0
        view.device_rack_state.decay = 0.95;
        view.sync_with_param_bus(&project, &bus, &mut registry, &mut timeline, 3.0, true);

        // Timeline should now contain lane for track_1_decay at beat 3.0
        let recorded_decay = timeline.evaluate("track_1_decay", 3.0);
        assert_eq!(recorded_decay, Some(0.95));
    }

    #[test]
    fn test_m33_live_oscilloscope_buffer_streaming_and_alloc_guard() {
        let project = create_default_project("M33 Live Stream");
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        // Verify pre-registered ParamIds
        assert!(app.param_bus.get(ParamId(1000)).is_some());
        assert!(app.param_bus.get(ParamId(1100)).is_some());

        // Start transport
        app.transport_running = true;

        // Verify zero-allocation audio streaming into oscilloscope buffer
        let guard_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = AllocGuard::new();
            if let Some(osc) = app.oscilloscope_buffers.get(&1) {
                osc.write_sample(0.75);
                let samples = osc.read_all();
                assert!((samples[511] - 0.75).abs() < 1e-5);
            }
        }));
        assert!(guard_result.is_ok(), "Oscilloscope streaming violated AllocGuard zero-alloc constraint!");

        // Verify AwardWinningGuiView receives live oscilloscope data
        let osc = app.oscilloscope_buffers.get(&1).unwrap();
        app.award_winning_view.current_oscilloscope_samples = Some(osc.read_all().to_vec());
        assert!(app.award_winning_view.current_oscilloscope_samples.is_some());
        assert_eq!(app.award_winning_view.current_oscilloscope_samples.as_ref().unwrap().len(), 512);
    }
}
