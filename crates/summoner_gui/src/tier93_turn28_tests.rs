// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #28 Integration & Regression Tests:
//! 1. Arranger track header 1-click Pro view launchers ([🎹], [∿], [🎚], [📈]) and live Bézier parameter automation opening.
//! 2. Arranger track volume pill double-click reset to unity gain (1.0 = 0.0 dB).
//! 3. Extended track parameter automation support ("gain", "pan", "mute", "solo", "cutoff").
//! 4. Global modal rendering of live parameter automation and modular DSP catalog across all views (Arranger, Mixer, Stage).
//! 5. Stage / Live Performance Matrix scene launching (`launch_scene`), scene stopping (`stop_scene`), and panic killswitch (`trigger_panic`).
//! 6. Modern Top Bar 5-tab switcher integration (`ModernViewTab::Performance`).
//! 7. Headless egui Context rendering across all views and modals without panic.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::ModernViewTab;

    #[test]
    fn test_turn28_arranger_volume_pill_double_click_unity_reset() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;

        // Alter track gain away from unity
        view.tracks[0].gain = 0.42;
        assert_eq!(view.tracks[0].gain, 0.42);

        // Double-click unity reset
        assert!(view.reset_track_gain(0));
        assert_eq!(view.tracks[0].gain, 1.0);
        assert_eq!(view.inspector_state.gain_db, 0.0);

        // Out of bounds index safety
        assert!(!view.reset_track_gain(999));
    }

    #[test]
    fn test_turn28_extended_track_automation_parameters() {
        let mut view = AwardWinningGuiView::new();
        let track_id = view.tracks[0].id;

        // 1. Gain automation
        assert!(view.open_track_automation_editor(track_id, "gain"));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_gain", track_id))
        );
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.min_display, 0.0);
        assert_eq!(editor.max_display, 1.5);

        // 2. Pan automation
        assert!(view.open_track_automation_editor(track_id, "pan"));
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_pan", track_id))
        );
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.min_display, -1.0);
        assert_eq!(editor.max_display, 1.0);

        // 3. Mute automation
        assert!(view.open_track_automation_editor(track_id, "mute"));
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_mute", track_id))
        );
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.min_display, 0.0);
        assert_eq!(editor.max_display, 1.0);

        // 4. Solo automation
        assert!(view.open_track_automation_editor(track_id, "solo"));
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_solo", track_id))
        );

        // 5. Filter cutoff automation
        assert!(view.open_track_automation_editor(track_id, "cutoff"));
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_cutoff", track_id))
        );
        let editor = view.automation_editor.as_ref().unwrap();
        assert_eq!(editor.min_display, 20.0);
        assert_eq!(editor.max_display, 20000.0);

        // 6. Invalid parameter returns false
        assert!(!view.open_track_automation_editor(track_id, "unknown_param"));
        // Invalid track ID returns false
        assert!(!view.open_track_automation_editor(9999, "gain"));
    }

    #[test]
    fn test_turn28_stage_matrix_scene_launch_and_panic_killswitch() {
        let mut view = AwardWinningGuiView::new();

        // 1. Initial stage state
        assert_eq!(view.active_scene_idx, None);
        assert!(!view.top_bar_state.is_playing);
        assert!(!view.panic_triggered);

        // 2. Launch Scene 0 (1 Intro)
        assert!(view.launch_scene(0));
        assert_eq!(view.active_scene_idx, Some(0));
        assert!(view.top_bar_state.is_playing);
        assert!(view.last_synced_is_playing);
        assert_eq!(view.playhead_beat, 0.0);
        assert!(!view.panic_triggered);

        // 3. Launch Scene 2 (3 Drop)
        assert!(view.launch_scene(2));
        assert_eq!(view.active_scene_idx, Some(2));
        assert_eq!(view.playhead_beat, 32.0);

        // 4. Stop Scene
        view.stop_scene();
        assert_eq!(view.active_scene_idx, None);
        assert!(!view.top_bar_state.is_playing);

        // 5. Arm some tracks and trigger panic killswitch
        view.tracks[0].is_armed = true;
        view.tracks[1].is_armed = true;
        view.top_bar_state.is_playing = true;

        view.trigger_panic();
        assert!(!view.top_bar_state.is_playing);
        assert!(!view.last_synced_is_playing);
        assert!(view.panic_triggered);
        assert!(!view.tracks[0].is_armed);
        assert!(!view.tracks[1].is_armed);

        // 6. Out of bounds scene index safety
        assert!(!view.launch_scene(4));
        assert!(!view.launch_scene(99));
    }

    #[test]
    fn test_turn28_modern_top_bar_5_tabs_switcher() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Arranger);

        // Switch through all 5 view tabs
        view.top_bar_state.active_tab = ModernViewTab::PianoRoll;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::PianoRoll);

        view.top_bar_state.active_tab = ModernViewTab::Modular;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Modular);

        view.top_bar_state.active_tab = ModernViewTab::Mixer;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Mixer);

        view.top_bar_state.active_tab = ModernViewTab::Performance;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Performance);

        view.top_bar_state.active_tab = ModernViewTab::Arranger;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Arranger);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::ModernViewTab;

    #[test]
    fn test_turn28_global_modal_rendering_across_tabs() {
        let mut view = AwardWinningGuiView::new();
        let track_id = view.tracks[0].id;

        // Open live Bézier parameter automation
        assert!(view.open_track_automation_editor(track_id, "gain"));
        assert!(view.show_automation_editor_window);

        let ctx = egui::Context::default();

        // Render on Arranger tab with modal open
        view.top_bar_state.active_tab = ModernViewTab::Arranger;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Render on Mixer tab with modal open
        view.top_bar_state.active_tab = ModernViewTab::Mixer;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Render on Stage tab with modal open
        view.top_bar_state.active_tab = ModernViewTab::Performance;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Close modular automation editor
        view.close_modular_automation_editor();
        assert!(!view.show_automation_editor_window);
    }

    #[test]
    fn test_turn28_arranger_and_stage_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Pro Mode Arranger with 1-click Pro launchers
        view.top_bar_state.is_pro_mode = true;
        view.top_bar_state.active_tab = ModernViewTab::Arranger;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 2. Stage View live performance matrix
        view.top_bar_state.active_tab = ModernViewTab::Performance;
        view.launch_scene(1);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        view.stop_scene();

        // 3. Trigger Panic in Stage View
        view.trigger_panic();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }
}
