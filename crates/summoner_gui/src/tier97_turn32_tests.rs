//! Comprehensive verification test suite for Turn #32:
//! Arranger Pro Top Toolbar, Two-Tier Ergonomics, Snap Grid Resolutions,
//! Clip Duplicate / Delete / Quantize / Move / Loop Operations, and 1-Click Pro View Switchers.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{
        ArrangerClipVisual, ArrangerSnapResolution, AwardWinningGuiView, TrackVisualData,
    };

    #[test]
    fn test_turn32_arranger_snap_resolution_step_and_snapping() {
        assert_eq!(ArrangerSnapResolution::Bar1.name(), "1 Bar (4b)");
        assert_eq!(ArrangerSnapResolution::Bar1.step_beats(), 4.0);
        assert_eq!(ArrangerSnapResolution::Bar1.snap(3.8), 4.0);
        assert_eq!(ArrangerSnapResolution::Bar1.snap(1.9), 0.0);
        assert_eq!(ArrangerSnapResolution::Bar1.snap(2.1), 4.0);

        assert_eq!(ArrangerSnapResolution::BeatQuarter.name(), "1/4 Beat (1b)");
        assert_eq!(ArrangerSnapResolution::BeatQuarter.step_beats(), 1.0);
        assert_eq!(ArrangerSnapResolution::BeatQuarter.snap(2.3), 2.0);
        assert_eq!(ArrangerSnapResolution::BeatQuarter.snap(2.7), 3.0);

        assert_eq!(ArrangerSnapResolution::BeatEighth.name(), "1/8 Beat (0.5b)");
        assert_eq!(ArrangerSnapResolution::BeatEighth.step_beats(), 0.5);
        assert_eq!(ArrangerSnapResolution::BeatEighth.snap(1.2), 1.0);
        assert_eq!(ArrangerSnapResolution::BeatEighth.snap(1.3), 1.5);

        assert_eq!(ArrangerSnapResolution::BeatSixteenth.name(), "1/16 Beat (0.25b)");
        assert_eq!(ArrangerSnapResolution::BeatSixteenth.step_beats(), 0.25);
        assert_eq!(ArrangerSnapResolution::BeatSixteenth.snap(0.6), 0.5);
        assert_eq!(ArrangerSnapResolution::BeatSixteenth.snap(0.7), 0.75);

        assert_eq!(ArrangerSnapResolution::Free.name(), "Off (Free)");
        assert_eq!(ArrangerSnapResolution::Free.step_beats(), 0.0);
        assert_eq!(ArrangerSnapResolution::Free.snap(1.2345), 1.2345);
    }

    #[test]
    fn test_turn32_arranger_track_visual_data_clip_duplicate() {
        let mut track = TrackVisualData {
            id: 1,
            name: "Lead Synth".to_string(),
            color_rgb: [100, 150, 250],
            is_audio: false,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            clip_start_beat: 0.0,
            clip_length_beats: 4.0,
            clips: vec![ArrangerClipVisual {
                id: 1,
                name: "Lead Synth".to_string(),
                start_beat: 0.0,
                length_beats: 4.0,
                fade_in: 0.1,
                fade_out: 0.1,
                gain: 1.0,
                is_selected: false,
            }],
        };

        let new_id = track.duplicate_clip(1);
        assert_eq!(new_id, Some(2));
        assert_eq!(track.clips.len(), 2);
        assert_eq!(track.clips[1].id, 2);
        assert_eq!(track.clips[1].name, "Lead Synth (Copy)");
        assert_eq!(track.clips[1].start_beat, 4.0);
        assert_eq!(track.clips[1].length_beats, 4.0);

        // Duplicating non-existent clip returns None
        assert_eq!(track.duplicate_clip(999), None);
    }

    #[test]
    fn test_turn32_arranger_track_visual_data_clip_delete() {
        let mut track = TrackVisualData {
            id: 1,
            name: "Bass".to_string(),
            color_rgb: [200, 100, 50],
            is_audio: false,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            clip_start_beat: 0.0,
            clip_length_beats: 4.0,
            clips: vec![
                ArrangerClipVisual {
                    id: 1,
                    name: "Bass A".to_string(),
                    start_beat: 0.0,
                    length_beats: 4.0,
                    fade_in: 0.0,
                    fade_out: 0.0,
                    gain: 1.0,
                    is_selected: false,
                },
                ArrangerClipVisual {
                    id: 2,
                    name: "Bass B".to_string(),
                    start_beat: 4.0,
                    length_beats: 4.0,
                    fade_in: 0.0,
                    fade_out: 0.0,
                    gain: 1.0,
                    is_selected: false,
                },
            ],
        };

        assert!(track.delete_clip(1));
        assert_eq!(track.clips.len(), 1);
        assert_eq!(track.clips[0].id, 2);

        // Deleting non-existent returns false
        assert!(!track.delete_clip(999));

        // Deleting final clip resets clip_length_beats
        assert!(track.delete_clip(2));
        assert!(track.clips.is_empty());
        assert_eq!(track.clip_length_beats, 0.0);
    }

    #[test]
    fn test_turn32_arranger_track_visual_data_clip_quantize_and_move() {
        let mut track = TrackVisualData {
            id: 1,
            name: "Drums".to_string(),
            color_rgb: [250, 100, 100],
            is_audio: true,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            clip_start_beat: 0.0,
            clip_length_beats: 4.0,
            clips: vec![ArrangerClipVisual {
                id: 1,
                name: "Drum Loop".to_string(),
                start_beat: 1.23,
                length_beats: 4.0,
                fade_in: 0.0,
                fade_out: 0.0,
                gain: 1.0,
                is_selected: false,
            }],
        };

        // Move clip by 1.5 beats
        assert!(track.move_clip(1, 1.5));
        assert!((track.clips[0].start_beat - 2.73).abs() < 1e-4);

        // Quantize to 1.0 beat grid
        assert!(track.quantize_clip(1, 1.0));
        assert_eq!(track.clips[0].start_beat, 3.0);

        // Move clip negative beyond 0.0 clamps at 0.0
        assert!(track.move_clip(1, -10.0));
        assert_eq!(track.clips[0].start_beat, 0.0);
    }

    #[test]
    fn test_turn32_arranger_track_selection_and_device_name() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.master_gain = 0.85;

        // Select track 2 via select_track_for_arranger
        let res = view.select_track_for_arranger(2);
        assert!(res);
        assert_eq!(view.selected_track_idx, 2);
        assert_eq!(view.device_rack_state.device_name, view.tracks[2].name);
        assert_eq!(view.inspector_state.target_name, view.tracks[2].name);
        // Master gain is preserved and not clobbered
        assert_eq!(view.top_bar_state.master_gain, 0.85);

        // Selecting out-of-bounds returns false
        assert!(!view.select_track_for_arranger(999));
    }

    #[test]
    fn test_turn32_arranger_clip_selection_duplicate_quantize_and_loop() {
        let mut view = AwardWinningGuiView::new();
        view.select_track_for_arranger(0);
        view.tracks[0].ensure_clips();
        let clip_id = view.tracks[0].clips[0].id;

        // Select clip
        assert!(view.select_clip(0, clip_id));
        assert_eq!(view.selected_clip, Some((0, clip_id)));
        assert!(view.tracks[0].clips[0].is_selected);

        // Set loop to clip
        assert!(view.set_loop_to_selected_clip());
        assert_eq!(view.loop_start_beat, view.tracks[0].clips[0].start_beat);
        assert_eq!(
            view.loop_end_beat,
            view.tracks[0].clips[0].start_beat + view.tracks[0].clips[0].length_beats
        );

        // Duplicate selected clip
        let dup_res = view.duplicate_selected_clip();
        assert!(dup_res.is_some());
        let (t_idx, new_c_id) = dup_res.unwrap();
        assert_eq!(t_idx, 0);
        assert_eq!(view.selected_clip, Some((0, new_c_id)));
        assert_eq!(view.tracks[0].clips.len(), 2);

        // Quantize selected clip
        view.arranger_snap_grid = ArrangerSnapResolution::Bar1;
        assert!(view.quantize_selected_clip());

        // Delete selected clip
        assert!(view.delete_selected_clip());
        assert_eq!(view.selected_clip, None);
        assert_eq!(view.tracks[0].clips.len(), 1);

        // Deselect clip clears selection
        view.select_clip(0, clip_id);
        assert_eq!(view.selected_clip, Some((0, clip_id)));
        view.deselect_clip();
        assert_eq!(view.selected_clip, None);
        assert!(!view.tracks[0].clips[0].is_selected);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;

    #[test]
    fn test_turn32_arranger_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.tracks.len() >= 4);
    }
}
