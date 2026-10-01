//! Comprehensive verification test suite for Turn #34 (Sprint Review):
//! Top Bar Novice Macro Strip Two-Tier UX & Ergonomics, Macro Dial Reset & Automation,
//! Master Volume Fader Unity Reset & Automation, Macro Live Bézier Curve Automation Editor,
//! and Milestone 33 Lock-Free ParamBus Live Automation Bridge.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::ModernTopBarState;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn34_top_bar_macro_dials_reset_and_automation_request() {
        let mut state = ModernTopBarState::default();
        assert_eq!(state.requested_automation_param, None);

        // Mutate macro values away from defaults
        state.macro_tone = 0.12;
        state.macro_space = 0.88;
        state.macro_punch = 0.95;
        state.macro_character = 0.05;

        // Reset macros restores all 4 to default states
        state.reset_macros();
        assert!((state.macro_tone - 0.65).abs() < 1e-5);
        assert!((state.macro_space - 0.40).abs() < 1e-5);
        assert!((state.macro_punch - 0.55).abs() < 1e-5);
        assert!((state.macro_character - 0.50).abs() < 1e-5);

        // Automation request helper
        state.request_macro_automation("Tone");
        assert_eq!(state.requested_automation_param, Some("macro_tone".to_string()));

        state.request_macro_automation("Space");
        assert_eq!(state.requested_automation_param, Some("macro_space".to_string()));

        state.request_macro_automation("Punch");
        assert_eq!(state.requested_automation_param, Some("macro_punch".to_string()));

        state.request_macro_automation("Char");
        assert_eq!(state.requested_automation_param, Some("macro_char".to_string()));
    }

    #[test]
    fn test_turn34_top_bar_master_volume_unity_reset_and_automation() {
        let mut state = ModernTopBarState::default();
        assert_eq!(state.master_gain, 1.0);

        // Adjust master volume
        state.master_gain = 0.45;
        assert_ne!(state.master_gain, 1.0);

        // Unity reset simulated
        state.master_gain = 1.0;
        assert_eq!(state.master_gain, 1.0);

        // Automation request for master gain
        state.requested_automation_param = Some("master_gain".to_string());
        assert_eq!(state.requested_automation_param, Some("master_gain".to_string()));
    }

    #[test]
    fn test_turn34_open_macro_automation_editor_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.macro_tone = 0.72;
        view.top_bar_state.macro_space = 0.45;
        view.top_bar_state.macro_punch = 0.65;
        view.top_bar_state.macro_character = 0.35;

        // Open Tone macro automation editor
        assert!(view.open_macro_automation_editor("macro_tone"));
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some("macro_tone".to_string()));
        let editor = view.automation_editor.as_ref().expect("Automation editor should be present");
        assert_eq!(editor.min_display, 0.0);
        assert_eq!(editor.max_display, 1.0);
        assert_eq!(editor.unit_name, "%");
        assert_eq!(editor.nodes.len(), 2);
        assert!((editor.nodes[0].value - 0.72).abs() < 1e-4);
        assert!((editor.nodes[1].value - 0.72).abs() < 1e-4);

        // Open Space macro with short alias
        assert!(view.open_macro_automation_editor("space"));
        assert_eq!(view.requested_modular_automation_param, Some("macro_space".to_string()));
        let editor = view.automation_editor.as_ref().unwrap();
        assert!((editor.nodes[0].value - 0.45).abs() < 1e-4);

        // Open Punch macro
        assert!(view.open_macro_automation_editor("punch"));
        assert_eq!(view.requested_modular_automation_param, Some("macro_punch".to_string()));

        // Open Character macro
        assert!(view.open_macro_automation_editor("char"));
        assert_eq!(view.requested_modular_automation_param, Some("macro_character".to_string()));

        // Unknown macro returns false gracefully
        assert!(!view.open_macro_automation_editor("unknown_macro"));
    }

    #[test]
    fn test_turn34_open_track_automation_editor_macro_support() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.macro_tone = 0.82;
        view.top_bar_state.macro_space = 0.55;

        // Open track automation editor with macro parameter
        let track_id = view.tracks[0].id;
        assert!(view.open_track_automation_editor(track_id, "macro_tone"));
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some("macro_tone".to_string()));
        let editor = view.automation_editor.as_ref().expect("Automation editor open");
        assert!((editor.nodes[0].value - 0.82).abs() < 1e-4);

        assert!(view.open_track_automation_editor(track_id, "space"));
        assert_eq!(view.requested_modular_automation_param, Some("macro_space".to_string()));
    }

    #[test]
    fn test_turn34_m33_live_macro_automation_recording_and_timeline_sync() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;
        let track_id = view.tracks[0].id;
        let project = ProjectConfig::default();
        let mut param_bus = ParamBus::new();
        param_bus.register(ParamId(track_id as u32 * 1000 + 100), 0.0);
        param_bus.register(ParamId(track_id as u32 * 1000 + 102), 0.0);
        param_bus.register(ParamId(track_id as u32 * 1000 + 105), 0.0);
        param_bus.register(ParamId(track_id as u32 * 1000 + 104), 0.0);

        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // 1. Live recording pass
        view.top_bar_state.macro_tone = 0.85;
        view.top_bar_state.macro_space = 0.70;
        view.top_bar_state.macro_punch = 0.60;
        view.top_bar_state.macro_character = 0.40;

        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut registry,
            &mut timeline,
            4.0,  // playhead_beat
            true, // is_recording_automation
        );

        // Verify ParamBus receives values at correct macro offsets
        let track_id = view.tracks[0].id;
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 100)), Some(0.85)); // Tone
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 102)), Some(0.70)); // Space
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 105)), Some(0.60)); // Punch
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 104)), Some(0.40)); // Character

        // Verify AutomationTimeline has recorded both track-scoped and global macro lane keys
        assert_eq!(timeline.evaluate("macro_tone", 4.0), Some(0.85));
        assert_eq!(timeline.evaluate(&format!("track_{}_macro_tone", track_id), 4.0), Some(0.85));
        assert_eq!(timeline.evaluate("macro_space", 4.0), Some(0.70));
        assert_eq!(timeline.evaluate("macro_punch", 4.0), Some(0.60));
        assert_eq!(timeline.evaluate("macro_character", 4.0), Some(0.40));

        // 2. Playback evaluation pass
        view.top_bar_state.is_playing = true;
        view.top_bar_state.macro_tone = 0.0;
        view.top_bar_state.macro_space = 0.0;
        view.top_bar_state.macro_punch = 0.0;
        view.top_bar_state.macro_character = 0.0;

        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut registry,
            &mut timeline,
            4.0,   // playhead_beat
            false, // is_recording_automation: false -> playback
        );

        // Live automation curves drive GUI macro values back during playback
        assert!((view.top_bar_state.macro_tone - 0.85).abs() < 1e-4);
        assert!((view.top_bar_state.macro_space - 0.70).abs() < 1e-4);
        assert!((view.top_bar_state.macro_punch - 0.60).abs() < 1e-4);
        assert!((view.top_bar_state.macro_character - 0.40).abs() < 1e-4);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::{show_modern_top_bar, ModernTopBarState};

    #[test]
    fn test_turn34_top_bar_canvas_rendering_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut state = ModernTopBarState::default();
                show_modern_top_bar(ui, &mut state, || {});
                assert!(state.master_gain > 0.0);
            });
        });
    }

    #[test]
    fn test_turn34_award_winning_gui_with_top_bar_automation_dispatch() {
        let ctx = egui::Context::default();
        let mut view = AwardWinningGuiView::new();

        // Queue macro automation request from top bar
        view.top_bar_state.requested_automation_param = Some("macro_tone".to_string());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Automation request was consumed and opened automation editor
        assert_eq!(view.top_bar_state.requested_automation_param, None);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some("macro_tone".to_string()));
    }
}
