//! Comprehensive verification test suite for Turn #33 (Sprint Review):
//! Stage View (Live Performance Matrix) Two-Tier UX, Stage Launch Quantization,
//! Per-Track Independent Clip Launching/Stopping, Tap Tempo Engine, Panic Killswitch,
//! 1-Click Pro View Switchers, and Milestone 33 Lock-Free ParamBus Live Automation Bridge.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, StageLaunchQuantize};
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn33_stage_launch_quantize_resolutions_and_boundaries() {
        assert_eq!(StageLaunchQuantize::Bar1.name(), "1 Bar (4b)");
        assert_eq!(StageLaunchQuantize::Bar1.step_beats(), 4.0);
        assert_eq!(StageLaunchQuantize::Bar1.next_boundary(0.0), 0.0);
        assert_eq!(StageLaunchQuantize::Bar1.next_boundary(1.2), 4.0);
        assert_eq!(StageLaunchQuantize::Bar1.next_boundary(4.0), 4.0);
        assert_eq!(StageLaunchQuantize::Bar1.next_boundary(4.1), 8.0);

        assert_eq!(StageLaunchQuantize::BarHalf.name(), "1/2 Bar (2b)");
        assert_eq!(StageLaunchQuantize::BarHalf.step_beats(), 2.0);
        assert_eq!(StageLaunchQuantize::BarHalf.next_boundary(0.0), 0.0);
        assert_eq!(StageLaunchQuantize::BarHalf.next_boundary(0.5), 2.0);
        assert_eq!(StageLaunchQuantize::BarHalf.next_boundary(2.0), 2.0);
        assert_eq!(StageLaunchQuantize::BarHalf.next_boundary(2.1), 4.0);

        assert_eq!(StageLaunchQuantize::Beat1.name(), "1 Beat (1b)");
        assert_eq!(StageLaunchQuantize::Beat1.step_beats(), 1.0);
        assert_eq!(StageLaunchQuantize::Beat1.next_boundary(0.2), 1.0);
        assert_eq!(StageLaunchQuantize::Beat1.next_boundary(1.0), 1.0);
        assert_eq!(StageLaunchQuantize::Beat1.next_boundary(1.05), 2.0);

        assert_eq!(StageLaunchQuantize::Instant.name(), "Instant (0b)");
        assert_eq!(StageLaunchQuantize::Instant.step_beats(), 0.0);
        assert_eq!(StageLaunchQuantize::Instant.next_boundary(3.456), 3.456);
    }

    #[test]
    fn test_turn33_stage_scene_launch_and_stop_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.active_scene_idx, None);
        assert!(view.tracks.iter().all(|t| t.active_clip_idx.is_none()));

        // Launch Scene 2 (Drop)
        let success = view.launch_scene(2);
        assert!(success);
        assert_eq!(view.active_scene_idx, Some(2));
        assert!(view.top_bar_state.is_playing);
        assert_eq!(view.playhead_beat, 32.0); // 2 * 16.0
        assert!(!view.panic_triggered);

        // Verify all tracks now reflect clip 2
        for tr in &view.tracks {
            assert_eq!(tr.active_clip_idx, Some(2));
        }

        // Stop scene
        view.stop_scene();
        assert_eq!(view.active_scene_idx, None);
        assert!(!view.top_bar_state.is_playing);
        for tr in &view.tracks {
            assert_eq!(tr.active_clip_idx, None);
        }

        // Out of bounds scene index fails gracefully
        assert!(!view.launch_scene(10));
    }

    #[test]
    fn test_turn33_stage_per_track_independent_clip_launch_and_stop() {
        let mut view = AwardWinningGuiView::new();

        // Launch clip 0 on track 1 (Kick)
        assert!(view.launch_track_clip(0, 0));
        assert_eq!(view.tracks[0].active_clip_idx, Some(0));
        assert_eq!(view.selected_track_idx, 0);
        assert!(view.top_bar_state.is_playing);
        // Scene should be None since not all tracks are playing clip 0
        assert_eq!(view.active_scene_idx, None);

        // Launch clip 1 on track 1 (Snare)
        assert!(view.launch_track_clip(1, 1));
        assert_eq!(view.tracks[1].active_clip_idx, Some(1));
        assert_eq!(view.tracks[0].active_clip_idx, Some(0));

        // Re-clicking active clip toggles it off
        assert!(view.launch_track_clip(1, 1));
        assert_eq!(view.tracks[1].active_clip_idx, None);
        assert_eq!(view.tracks[0].active_clip_idx, Some(0));

        // Stop track 0 clip specifically
        assert!(view.stop_track_clip(0));
        assert_eq!(view.tracks[0].active_clip_idx, None);

        // Stop all clips killswitch
        view.launch_scene(1);
        assert_eq!(view.active_scene_idx, Some(1));
        view.stop_all_clips();
        assert_eq!(view.active_scene_idx, None);
        assert!(!view.top_bar_state.is_playing);
        for tr in &view.tracks {
            assert_eq!(tr.active_clip_idx, None);
        }
    }

    #[test]
    fn test_turn33_stage_tap_tempo_and_panic_killswitch() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.bpm = 120.0;

        // First tap records baseline
        view.tap_tempo();
        assert_eq!(view.top_bar_state.bpm, 120.0);
        assert!(view.last_tap_time.is_some());

        // Simulate second tap 0.5s later (120 BPM)
        view.last_tap_time = Some(std::time::Instant::now() - std::time::Duration::from_millis(500));
        view.tap_tempo();
        assert!((view.top_bar_state.bpm - 120.0).abs() < 1.0);

        // Simulate second tap 0.4s later (150 BPM)
        view.last_tap_time = Some(std::time::Instant::now() - std::time::Duration::from_millis(400));
        view.tap_tempo();
        assert!((view.top_bar_state.bpm - 150.0).abs() < 1.0);

        // Panic killswitch
        view.top_bar_state.is_playing = true;
        view.tracks[0].is_armed = true;
        view.tracks[0].active_clip_idx = Some(0);
        view.active_scene_idx = Some(0);

        view.trigger_panic();
        assert!(!view.top_bar_state.is_playing);
        assert!(view.panic_triggered);
        assert_eq!(view.active_scene_idx, None);
        assert!(!view.tracks[0].is_armed);
        assert_eq!(view.tracks[0].active_clip_idx, None);
    }

    #[test]
    fn test_turn33_stage_milestone33_parambus_dispatch_and_sync() {
        let mut view = AwardWinningGuiView::new();
        let mut param_bus = ParamBus::new();
        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();
        let project = ProjectConfig::default();

        // Register ParamIds in ParamBus
        let panic_pid = ParamId(9996);
        let scene_pid = ParamId(9997);
        param_bus.register(panic_pid, 0.0);
        param_bus.register(scene_pid, -1.0);

        for tr in &view.tracks {
            let clip_pid = ParamId(tr.id as u32 * 1000 + 204);
            param_bus.register(clip_pid, -1.0);
        }

        // 1. Initial sync (idle state)
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 0.0, false);
        assert_eq!(param_bus.get(panic_pid), Some(0.0));
        assert_eq!(param_bus.get(scene_pid), Some(-1.0));

        // 2. Launch Scene 3 (Outro)
        view.launch_scene(3);
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 48.0, false);
        assert_eq!(param_bus.get(scene_pid), Some(3.0));
        for tr in &view.tracks {
            let clip_pid = ParamId(tr.id as u32 * 1000 + 204);
            assert_eq!(param_bus.get(clip_pid), Some(3.0));
        }

        // 3. Trigger Panic
        view.trigger_panic();
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 48.0, false);
        assert_eq!(param_bus.get(panic_pid), Some(1.0));
        assert_eq!(param_bus.get(scene_pid), Some(-1.0));
        for tr in &view.tracks {
            let clip_pid = ParamId(tr.id as u32 * 1000 + 204);
            assert_eq!(param_bus.get(clip_pid), Some(-1.0));
        }
    }

    #[test]
    fn test_turn33_stage_track_selection_and_device_name() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.selected_track_idx, 4);

        // Select track 0 (Kick)
        assert!(view.select_track_for_stage(0));
        assert_eq!(view.selected_track_idx, 0);
        assert_eq!(view.device_rack_state.device_name, "Kick");
        assert_eq!(view.inspector_state.target_name, "Kick");

        // Select track 5 (Vocals)
        assert!(view.select_track_for_stage(5));
        assert_eq!(view.selected_track_idx, 5);
        assert_eq!(view.device_rack_state.device_name, "Vocals");
        assert_eq!(view.inspector_state.target_name, "Vocals");

        // Out of bounds selection returns false
        assert!(!view.select_track_for_stage(999));
    }
}
