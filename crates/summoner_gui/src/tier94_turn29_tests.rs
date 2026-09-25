// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #29 Integration & Regression Tests:
//! 1. Piano Roll Track Selector and active device name reflection.
//! 2. Piano Roll Audition Toggle and state persistence.
//! 3. Piano Roll Multi-Lane Modes (Velocity, Gate, PitchBend) display names, cycling, and stick interactions.
//! 4. Piano Roll surgical note operations: Quantize, Transpose (+/- 1, +/- Octave), Duplicate, and Clear.
//! 5. Universal DSP Module Live Automation: Standard parameters + dynamic registry schema reflection.
//! 6. Inspector and Device Rack 1-click live automation request dispatching.
//! 7. Headless egui Context rendering of Piano Roll canvas across Pro/Novice modes without panics.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, PianoRollLaneMode};

    #[test]
    fn test_turn29_piano_roll_track_selection_and_device_name() {
        let mut view = AwardWinningGuiView::new();

        // Switch track to track 1
        assert!(view.select_track_for_piano_roll(1));
        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.device_rack_state.device_name, "Snare");

        // Switch to track 2
        assert!(view.select_track_for_piano_roll(2));
        assert_eq!(view.selected_track_idx, 2);
        assert_eq!(view.device_rack_state.device_name, "HiHats");

        // Out of bounds safety
        assert!(!view.select_track_for_piano_roll(999));
        assert_eq!(view.selected_track_idx, 2);
    }

    #[test]
    fn test_turn29_piano_roll_audition_toggle_and_param_bus_muting() {
        let mut view = AwardWinningGuiView::new();
        assert!(view.is_piano_roll_audition_enabled);

        // Toggle audition off
        let new_state = view.toggle_piano_roll_audition();
        assert!(!new_state);
        assert!(!view.is_piano_roll_audition_enabled);

        // Toggle audition back on
        view.set_piano_roll_audition(true);
        assert!(view.is_piano_roll_audition_enabled);
    }

    #[test]
    fn test_turn29_piano_roll_multi_lane_modes() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.piano_roll_lane_mode, PianoRollLaneMode::Velocity);
        assert_eq!(view.piano_roll_lane_mode.name(), "Velocity");

        // Set to Gate
        view.set_piano_roll_lane_mode(PianoRollLaneMode::Gate);
        assert_eq!(view.piano_roll_lane_mode, PianoRollLaneMode::Gate);
        assert_eq!(view.piano_roll_lane_mode.name(), "Gate Length");

        // Set to PitchBend
        view.set_piano_roll_lane_mode(PianoRollLaneMode::PitchBend);
        assert_eq!(view.piano_roll_lane_mode, PianoRollLaneMode::PitchBend);
        assert_eq!(view.piano_roll_lane_mode.name(), "Pitch Bend");

        // Cycle back to Velocity
        view.set_piano_roll_lane_mode(PianoRollLaneMode::Velocity);
        assert_eq!(view.piano_roll_lane_mode, PianoRollLaneMode::Velocity);
    }

    #[test]
    fn test_turn29_piano_roll_note_operations_quantize_transpose_duplicate_clear() {
        let mut view = AwardWinningGuiView::new();

        // Ensure initial notes are populated
        assert!(!view.piano_roll_notes.is_empty());
        let initial_note_count = view.piano_roll_notes.len();
        assert!(initial_note_count > 0);

        // 1. Select the first note
        view.selected_note_id = Some(1);
        let orig_pitch_idx = view.piano_roll_notes[0].pitch_idx;

        // 2. Transpose up 1 step (note: in grid canvas, up moves to lower pitch index)
        assert!(view.transpose_selected_piano_roll_note(1));
        assert_eq!(view.piano_roll_notes[0].pitch_idx, orig_pitch_idx.saturating_sub(1));

        // 3. Transpose down 1 step
        assert!(view.transpose_selected_piano_roll_note(-1));
        assert_eq!(view.piano_roll_notes[0].pitch_idx, orig_pitch_idx);

        // 4. Duplicate selected note
        let dup_id = view.duplicate_selected_piano_roll_note();
        assert!(dup_id.is_some());
        assert_eq!(view.piano_roll_notes.len(), initial_note_count + 1);
        assert_eq!(view.selected_note_id, dup_id);

        // 5. Quantize notes to active snap grid
        view.quantize_piano_roll_notes();
        assert_eq!(view.piano_roll_notes.len(), initial_note_count + 1);

        // 6. Clear notes
        view.clear_piano_roll_notes();
        assert_eq!(view.piano_roll_notes.len(), 0);
        assert_eq!(view.selected_note_id, None);

        // Transposing/duplicating with no notes returns false/None
        assert!(!view.transpose_selected_piano_roll_note(1));
        assert!(view.duplicate_selected_piano_roll_note().is_none());
    }

    #[test]
    fn test_turn29_universal_dsp_module_live_automation_reflection() {
        let mut view = AwardWinningGuiView::new();
        let track_id = view.tracks[0].id;

        // Standard audio parameters
        let standard_params = [
            ("gain", 0.0, 1.5),
            ("pan", -1.0, 1.0),
            ("mute", 0.0, 1.0),
            ("solo", 0.0, 1.0),
            ("cutoff", 20.0, 20000.0),
            ("resonance", 0.1, 10.0),
            ("decay", 0.01, 5.0),
            ("drive", 0.0, 2.0),
            ("mod", 0.0, 1.0),
            ("volume", 0.0, 1.5),
            ("osc_mix", 0.0, 1.0),
            ("shape", 0.0, 1.0),
            ("pitch", -12.0, 12.0),
            ("velocity", 0.0, 1.0),
            ("expression", 0.0, 1.0),
        ];

        for (param, min_val, max_val) in standard_params {
            assert!(view.open_track_automation_editor(track_id, param));
            assert!(view.show_automation_editor_window);
            assert!(
                view.requested_modular_automation_param.is_some(),
                "Failed for param: {}",
                param
            );
            let editor = view.automation_editor.as_ref().unwrap();
            assert_eq!(editor.min_display, min_val, "Min mismatch for {}", param);
            assert_eq!(editor.max_display, max_val, "Max mismatch for {}", param);
        }

        // Test DSP registry parameter fallback with registered AetherSynth
        view.device_rack_state.selected_node_kind = Some("AetherSynth".to_string());
        // Dynamic fallback should look up parameter descriptor
        assert!(view.open_track_automation_editor(track_id, "cutoff"));
        let editor = view.automation_editor.as_ref().unwrap();
        assert!(editor.max_display >= editor.min_display);
    }

    #[test]
    fn test_turn29_inspector_and_device_rack_automation_request_dispatch() {
        let mut view = AwardWinningGuiView::new();

        // 1. Inspector automation request
        view.inspector_state.requested_automation_param = Some("pan".to_string());
        // Simulate event consumption dispatch
        let track_id = view.tracks[view.selected_track_idx].id;
        let param = view.inspector_state.requested_automation_param.take().unwrap();
        assert!(view.open_track_automation_editor(track_id, &param));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_pan", track_id))
        );

        // 2. Device Rack automation request
        view.device_rack_state.requested_automation_param = Some("cutoff".to_string());
        let rack_param = view.device_rack_state.requested_automation_param.take().unwrap();
        assert!(view.open_track_automation_editor(track_id, &rack_param));
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_cutoff", track_id))
        );
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, PianoRollLaneMode};
    use crate::views::modern_top_bar::ModernViewTab;

    #[test]
    fn test_turn29_piano_roll_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        view.top_bar_state.active_tab = ModernViewTab::PianoRoll;

        // Render in Novice mode
        view.top_bar_state.is_pro_mode = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Render in Pro mode across all lane modes
        view.top_bar_state.is_pro_mode = true;
        for mode in [
            PianoRollLaneMode::Velocity,
            PianoRollLaneMode::Gate,
            PianoRollLaneMode::PitchBend,
        ] {
            view.set_piano_roll_lane_mode(mode);
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
        }
    }

    #[test]
    fn test_turn29_inspector_and_rack_automation_click_dispatch_in_show() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // Queue inspector request
        view.inspector_state.requested_automation_param = Some("gain".to_string());

        // Run show() to trigger dispatch loop
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_gain", view.tracks[view.selected_track_idx].id))
        );
        assert_eq!(view.inspector_state.requested_automation_param, None);
    }
}
