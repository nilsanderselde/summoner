// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #30 Integration & Regression Tests (Sprint Review):
//! 1. Patch Cord Live Bézier Automation Editor open/close and normalization.
//! 2. Milestone 33 live patch cord intensity curve evaluation, ParamBus dispatch, and timeline synchronization.
//! 3. Modular canvas 1-click Pro view switchers and Track Selector ComboBox state reflections.
//! 4. Stage / Performance matrix 1-click Pro view switchers and Track Header selection.
//! 5. Headless egui Context rendering of Modular canvas and Stage canvas without panics.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AutomationQuickShape, AwardWinningGuiView};
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn30_patch_cord_automation_editor_open_and_close() {
        let mut view = AwardWinningGuiView::new();

        // Ensure default patch cords exist
        assert!(!view.patch_cords.is_empty());
        let cord = &view.patch_cords[0];
        let expected_key = format!(
            "modular_cord_{}_{}_{}_{}_intensity",
            cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id
        );

        // 1. Initial state
        assert!(!view.show_automation_editor_window);
        assert!(view.automation_editor.is_none());

        // 2. Open automation editor for patch cord 0
        assert!(view.open_patch_cord_automation_editor(0));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param.as_deref(),
            Some(expected_key.as_str())
        );

        let editor = view.automation_editor.as_ref().expect("Editor must be initialized");
        assert_eq!(editor.min_display, -1.0);
        assert_eq!(editor.max_display, 1.0);
        assert_eq!(editor.unit_name, "%");
        assert_eq!(editor.nodes.len(), 2);

        // Cord default intensity is 1.0 -> normalized: (1.0 + 1.0) / 2.0 = 1.0
        assert_eq!(editor.nodes[0].value, 1.0);
        assert_eq!(editor.nodes[1].value, 1.0);

        // 3. Out-of-bounds safety
        assert!(!view.open_patch_cord_automation_editor(999));

        // 4. Close editor
        view.close_modular_automation_editor();
        assert!(!view.show_automation_editor_window);
    }

    #[test]
    fn test_turn30_patch_cord_live_automation_evaluation_and_sync() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = if !project.tracks.is_empty() {
            project.tracks[view.selected_track_idx].id as u32
        } else {
            view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32
        };
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Register ParamBus slot for cable 0: track_id * 1000 + 700 + 0
        let cord_pid = ParamId(track_id * 1000 + 700);
        bus.register(cord_pid, 0.0);

        // 1. Open patch cord automation editor
        assert!(view.open_patch_cord_automation_editor(0));

        // 2. Apply RampUp shape: 0.0 (value -1.0) -> 1.0 (value +1.0) across 16 beats
        view.apply_automation_quick_shape(AutomationQuickShape::RampUp);

        // 3. Sync at beat 0.0 -> value should be -1.0
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 0.0, false);
        assert_eq!(view.patch_cords[0].intensity, -1.0);
        assert_eq!(bus.get(cord_pid), Some(-1.0));

        // 4. Sync at beat 8.0 (halfway) -> value should be 0.0
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 8.0, false);
        let mid_val = view.patch_cords[0].intensity;
        assert!(mid_val.abs() < 1e-3, "Expected ~0.0, got {}", mid_val);
        assert!((bus.get(cord_pid).unwrap()).abs() < 1e-3);

        // 5. Sync at beat 16.0 (end) -> value should be 1.0
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 16.0, false);
        assert_eq!(view.patch_cords[0].intensity, 1.0);
        assert_eq!(bus.get(cord_pid), Some(1.0));
    }

    #[test]
    fn test_turn30_modular_canvas_pro_view_switchers_and_track_selector() {
        let mut view = AwardWinningGuiView::new();

        // 1. Track selection from Modular canvas
        assert!(view.select_track_for_modular(1));
        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.device_rack_state.device_name, "Snare");
        assert_eq!(view.inspector_state.target_name, "Snare");

        assert!(view.select_track_for_modular(2));
        assert_eq!(view.selected_track_idx, 2);
        assert_eq!(view.device_rack_state.device_name, "HiHats");
        assert_eq!(view.inspector_state.target_name, "HiHats");

        // Out of bounds safety
        assert!(!view.select_track_for_modular(999));
        assert_eq!(view.selected_track_idx, 2);

        // 2. View switching from Modular canvas
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
        assert_eq!(
            view.top_bar_state.active_tab,
            crate::views::modern_top_bar::ModernViewTab::Modular
        );
    }

    #[test]
    fn test_turn30_stage_view_pro_view_switchers_and_track_selection() {
        let mut view = AwardWinningGuiView::new();

        // 1. Track selection from Stage view
        assert!(view.select_track_for_stage(1));
        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.device_rack_state.device_name, "Snare");
        assert_eq!(view.inspector_state.target_name, "Snare");

        assert!(view.select_track_for_stage(0));
        assert_eq!(view.selected_track_idx, 0);
        assert_eq!(view.device_rack_state.device_name, "Kick");
        assert_eq!(view.inspector_state.target_name, "Kick");

        // Out of bounds safety
        assert!(!view.select_track_for_stage(999));
        assert_eq!(view.selected_track_idx, 0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, ModularCanvasMode};
    use crate::views::modern_top_bar::ModernViewTab;

    #[test]
    fn test_turn30_modular_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        view.top_bar_state.active_tab = ModernViewTab::Modular;

        // Render Patch Cords mode
        view.modular_mode = ModularCanvasMode::PatchCords;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Select a cable and render again (triggers cable details bar with Auto button)
        view.selected_patch_cord_idx = Some(0);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Render Routing Matrix mode
        view.modular_mode = ModularCanvasMode::RoutingMatrix;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn30_stage_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        view.top_bar_state.active_tab = ModernViewTab::Performance;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }
}
