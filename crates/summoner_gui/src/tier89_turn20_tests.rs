// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #20 Integration & Regression Tests:
//! 1. Modular Canvas patch cord attenuation & attenuversion (-100% to +100%).
//! 2. Modular Patch Cord lifecycle: selection, removal with graph edge cleanup, and automatic selection index shifting.
//! 3. Milestone 33 live parameter bridge: modular cable intensity lockstep dispatch to `ParamBus` and `AutomationTimeline`.
//! 4. Modular Canvas headless egui rendering with interactive attenuverter pucks and cable toolbar strip.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn20_patch_cord_attenuation_and_attenuversion() {
        let mut view = AwardWinningGuiView::new();

        // Initial default cords check
        assert_eq!(view.patch_cords.len(), 3);
        assert_eq!(view.patch_cords[0].intensity, 1.0);
        assert_eq!(view.patch_cords[1].intensity, 1.0);
        assert_eq!(view.patch_cords[2].intensity, 1.0);

        // Adjust cord intensity
        let ok = view.set_patch_cord_intensity(0, 0.45);
        assert!(ok);
        assert_eq!(view.patch_cords[0].intensity, 0.45);

        // Clamping bounds check (-1.0 to 1.0)
        view.set_patch_cord_intensity(0, 3.5);
        assert_eq!(view.patch_cords[0].intensity, 1.0);

        view.set_patch_cord_intensity(0, -5.0);
        assert_eq!(view.patch_cords[0].intensity, -1.0);

        // Polarity inversion toggle (+ / -)
        view.set_patch_cord_intensity(1, 0.75);
        let inverted = view.toggle_patch_cord_polarity(1);
        assert_eq!(inverted, Some(-0.75));
        assert_eq!(view.patch_cords[1].intensity, -0.75);

        let reinverted = view.toggle_patch_cord_polarity(1);
        assert_eq!(reinverted, Some(0.75));
        assert_eq!(view.patch_cords[1].intensity, 0.75);

        // Out of bounds safety
        assert!(!view.set_patch_cord_intensity(99, 0.5));
        assert_eq!(view.toggle_patch_cord_polarity(99), None);
    }

    #[test]
    fn test_turn20_patch_cord_removal_and_selection_shift() {
        let mut view = AwardWinningGuiView::new();

        assert_eq!(view.patch_cords.len(), 3);
        assert_eq!(view.selected_patch_cord_idx, None);

        // Select cord at index 1: filter_1:lp -> vca_1:audio
        view.selected_patch_cord_idx = Some(1);
        {
            let sel_cord = view.selected_patch_cord().expect("Selected cord must exist");
            assert_eq!(sel_cord.from_node_id, "filter_1");
            assert_eq!(sel_cord.to_node_id, "vca_1");
        }

        // Remove cord at index 0 (osc_1 -> filter_1)
        let removed = view.remove_patch_cord_by_index(0);
        assert!(removed);
        assert_eq!(view.patch_cords.len(), 2);

        // Verify selected_patch_cord_idx automatically shifted from 1 to 0
        assert_eq!(view.selected_patch_cord_idx, Some(0));
        {
            let sel_cord = view.selected_patch_cord().expect("Selected cord must still exist");
            assert_eq!(sel_cord.from_node_id, "filter_1");
            assert_eq!(sel_cord.to_node_id, "vca_1");
        }

        // Remove the currently selected cord (now at index 0)
        let removed_sel = view.remove_patch_cord_by_index(0);
        assert!(removed_sel);
        assert_eq!(view.patch_cords.len(), 1);

        // Verify selected_patch_cord_idx was cleared to None
        assert_eq!(view.selected_patch_cord_idx, None);
        assert!(view.selected_patch_cord().is_none());
    }

    #[test]
    fn test_turn20_m33_patch_cord_intensity_param_bus_and_automation_bridge() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1) as u32;
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // Register ParamIds on bus for patch cords (track offset 700..703)
        for i in 0..4 {
            bus.register(ParamId(track_id * 1000 + 700 + i), 1.0);
        }

        // Modify cord intensities
        view.set_patch_cord_intensity(0, 0.80);
        view.set_patch_cord_intensity(1, -0.60);
        view.set_patch_cord_intensity(2, 0.0);

        // Sync with ParamBus and live automation recording at beat 12.0
        let playhead = 12.0;
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, playhead, true);

        // Verify ParamBus receives exact values without audio-thread allocation
        let val0 = bus.get(ParamId(track_id * 1000 + 700 + 0)).expect("Cord 0 ParamId must exist");
        let val1 = bus.get(ParamId(track_id * 1000 + 700 + 1)).expect("Cord 1 ParamId must exist");
        let val2 = bus.get(ParamId(track_id * 1000 + 700 + 2)).expect("Cord 2 ParamId must exist");

        assert_eq!(val0, 0.80);
        assert_eq!(val1, -0.60);
        assert_eq!(val2, 0.0);

        // Verify AutomationRegistry has recorded parameters
        let key1 = format!(
            "modular_cord_{}_{}_{}_{}_intensity",
            view.patch_cords[1].from_node_id,
            view.patch_cords[1].from_port_id,
            view.patch_cords[1].to_node_id,
            view.patch_cords[1].to_port_id
        );
        let reg_val1 = reg.get_param(&key1).map(|p| p.get());
        assert_eq!(reg_val1, Some(-0.60));

        // Verify AutomationTimeline has recorded the automation curve point
        let lane1 = timeline.lanes.get(&key1).expect("Automation lane for cord 1 must exist");
        assert_eq!(lane1.curve.points.len(), 1);
        assert_eq!(lane1.curve.points[0].beat, 12.0);
        assert_eq!(lane1.curve.points[0].value, -0.60);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_turn20_modular_canvas_headless_rendering_and_cord_inspector() {
        let mut view = AwardWinningGuiView::new();
        let ctx = eframe::egui::Context::default();

        // 1. Initial render without selected cord
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 2. Select cord at index 0 and render with active cable toolbar strip & attenuverter puck
        view.selected_patch_cord_idx = Some(0);
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.selected_patch_cord_idx, Some(0));
        assert!(view.selected_patch_cord().is_some());
    }
}
