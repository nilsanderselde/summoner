// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #26 Integration & Regression Tests:
//! 1. Console Mixer channel strip rotary pan pot interaction, bounds clamping, and double-click reset to Center (0.0).
//! 2. Channel strip and master fader double-click reset to unity gain (1.0 = 0.0 dB).
//! 3. Master bus mute toggle with gain preservation and unmute restoration.
//! 4. 1-Click Pro View Launchers and Bézier automation editor integration for track gain, track pan, and master volume.
//! 5. Milestone 33 live automation evaluation, timeline synchronization, and lock-free ParamBus dispatch.
//! 6. Headless egui Context rendering of the upgraded Console Mixer canvas in both Novice and Pro modes.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AutomationQuickShape, AwardWinningGuiView};
    use crate::views::modern_top_bar::ModernViewTab;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn26_mixer_pan_pot_drag_and_double_click_reset() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;

        // 1. Initial pan is 0.0 (Center)
        assert_eq!(view.tracks[0].pan, 0.0);

        // 2. Set pan to hard left (-1.0) and hard right (+1.0)
        assert!(view.set_track_pan(0, -0.65));
        assert_eq!(view.tracks[0].pan, -0.65);
        assert_eq!(view.inspector_state.pan_val, -0.65);

        // 3. Clamping behavior
        assert!(view.set_track_pan(0, 1.8));
        assert_eq!(view.tracks[0].pan, 1.0);

        assert!(view.set_track_pan(0, -3.2));
        assert_eq!(view.tracks[0].pan, -1.0);

        // 4. Double-click reset to center
        assert!(view.reset_track_pan(0));
        assert_eq!(view.tracks[0].pan, 0.0);
        assert_eq!(view.inspector_state.pan_val, 0.0);

        // 5. Out of bounds index safety
        assert!(!view.set_track_pan(999, 0.5));
        assert!(!view.reset_track_pan(999));
    }

    #[test]
    fn test_turn26_mixer_fader_unity_gain_reset_and_master() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;

        // 1. Track fader gain alteration & reset
        view.tracks[0].gain = 0.35;
        assert_eq!(view.tracks[0].gain, 0.35);

        assert!(view.reset_track_gain(0));
        assert_eq!(view.tracks[0].gain, 1.0);
        assert_eq!(view.inspector_state.gain_db, 0.0);

        // 2. Master fader gain alteration & reset
        view.top_bar_state.master_gain = 0.45;
        assert_eq!(view.top_bar_state.master_gain, 0.45);

        view.reset_master_gain();
        assert_eq!(view.top_bar_state.master_gain, 1.0);
        assert!(!view.is_master_muted);

        // 3. Out of bounds index safety
        assert!(!view.reset_track_gain(999));
    }

    #[test]
    fn test_turn26_mixer_master_mute_toggle() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.master_gain = 1.35;

        // 1. First toggle mutes master output
        let is_muted = view.toggle_master_mute();
        assert!(is_muted);
        assert!(view.is_master_muted);
        assert_eq!(view.top_bar_state.master_gain, 0.0);
        assert_eq!(view.unmuted_master_gain, 1.35);

        // 2. Second toggle restores previous gain
        let is_muted_again = view.toggle_master_mute();
        assert!(!is_muted_again);
        assert!(!view.is_master_muted);
        assert_eq!(view.top_bar_state.master_gain, 1.35);
    }

    #[test]
    fn test_turn26_mixer_pro_view_launchers_and_track_automation_editor() {
        let mut view = AwardWinningGuiView::new();

        // 1. Open track volume gain automation editor
        let ok_gain = view.open_track_automation_editor(1, "gain");
        assert!(ok_gain);
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param.as_deref(),
            Some("track_1_gain")
        );
        let editor_gain = view.automation_editor.as_ref().expect("Editor must exist");
        assert_eq!(editor_gain.min_display, 0.0);
        assert_eq!(editor_gain.max_display, 1.5);
        assert_eq!(editor_gain.unit_name, "x");
        assert_eq!(editor_gain.nodes.len(), 2);

        // Apply quick shape RampUp
        view.apply_automation_quick_shape(AutomationQuickShape::RampUp);
        let ed_after = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_after.nodes[0].value, 0.0);
        assert_eq!(ed_after.nodes[1].value, 1.0);

        // 2. Open track stereo pan automation editor
        let ok_pan = view.open_track_automation_editor(1, "pan");
        assert!(ok_pan);
        assert_eq!(
            view.requested_modular_automation_param.as_deref(),
            Some("track_1_pan")
        );
        let editor_pan = view.automation_editor.as_ref().expect("Editor must exist");
        assert_eq!(editor_pan.min_display, -1.0);
        assert_eq!(editor_pan.max_display, 1.0);
        assert_eq!(editor_pan.unit_name, "");

        // 3. Open master volume gain automation editor
        let ok_master = view.open_master_automation_editor();
        assert!(ok_master);
        assert_eq!(
            view.requested_modular_automation_param.as_deref(),
            Some("master_gain")
        );
        let editor_master = view.automation_editor.as_ref().expect("Editor must exist");
        assert_eq!(editor_master.min_display, 0.0);
        assert_eq!(editor_master.max_display, 2.0);

        // 4. Invalid track / param safety
        assert!(!view.open_track_automation_editor(999, "gain"));
        assert!(!view.open_track_automation_editor(1, "invalid_param"));
    }

    #[test]
    fn test_turn26_mixer_m33_live_pan_and_gain_automation_timeline_and_param_bus() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;
        let project = ProjectConfig::default();
        let mut param_bus = ParamBus::new();
        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        let track_id = 1;
        let pan_pid = ParamId(track_id * 1000 + 201);
        let gain_pid = ParamId(track_id * 1000 + 200);
        let master_pid = ParamId(9999);

        // Pre-register mixer and master parameters in ParamBus
        param_bus.register(pan_pid, 0.0);
        param_bus.register(gain_pid, 1.0);
        param_bus.register(master_pid, 1.0);

        // 1. Initial sync to register parameters into registry and bus
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 0.0, false);
        assert_eq!(param_bus.get(pan_pid), Some(0.0));
        assert_eq!(param_bus.get(gain_pid), Some(1.0));
        assert_eq!(param_bus.get(master_pid), Some(1.0));

        // 2. Open automation editor for track pan and apply RampUp curve
        view.open_track_automation_editor(track_id as u64, "pan");
        view.apply_automation_quick_shape(AutomationQuickShape::RampUp);

        // Sync with param bus to flush editor curve into timeline
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 0.0, false);

        let pan_lane_key = format!("track_{}_pan", track_id);
        let lane = timeline.lanes.get(&pan_lane_key).expect("Pan lane must exist");
        assert_eq!(lane.curve.points.len(), 2);
        assert_eq!(lane.curve.points[0].value, -1.0);
        assert_eq!(lane.curve.points[1].value, 1.0);

        // 3. Playback simulation: evaluate at beat 8.0 (midway -> 0.0) and beat 16.0 (end -> 1.0)
        view.top_bar_state.is_playing = true;
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 8.0, false);
        let pan_at_8 = view.tracks[0].pan;
        assert!(pan_at_8.abs() < 1e-3, "Pan at midway should be 0.0, got {}", pan_at_8);
        assert!((param_bus.get(pan_pid).unwrap() - 0.0).abs() < 1e-3);

        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 16.0, false);
        let pan_at_16 = view.tracks[0].pan;
        assert!((pan_at_16 - 1.0).abs() < 1e-3, "Pan at end should be 1.0, got {}", pan_at_16);
        assert!((param_bus.get(pan_pid).unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_turn26_mixer_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.active_tab = ModernViewTab::Mixer;
        view.top_bar_state.is_pro_mode = true;

        let ctx = eframe::egui::Context::default();
        let raw_input = eframe::egui::RawInput::default();

        // 1. Render Pro Mode Console Mixer Canvas
        let _ = ctx.run(raw_input.clone(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 2. Render Novice Mode Console Mixer Canvas
        view.top_bar_state.is_pro_mode = false;
        let _ = ctx.run(raw_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.tracks.len(), 8);
    }
}
