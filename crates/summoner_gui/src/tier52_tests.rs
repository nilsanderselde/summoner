// Summoner DAW - Tier 52 GUI Milestones Unit Test Suite (Steps 1361-1370)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::bezier_automation_editor::{
        AutomationCurveType, BezierAutomationEditorView, NODE_HIT_RADIUS,
    };
    use crate::views::envelope_follower_view::{
        EnvelopeFollowerView, EnvelopeMode, SidechainSource,
    };
    use crate::views::isomorphic_tuning_keyboard::{
        IntervalCategory, IsomorphicTuningKeyboardView,
    };
    use crate::views::step_sequencer_matrix::{StepEditMode, StepSequencerMatrixView};
    use crate::views::transient_warp_editor::{TransientWarpEditorView, WARP_MARKER_HIT_RADIUS};

    #[test]
    fn test_step_1361_1366_transient_warp_marker_coordinates_and_hit_targets() {
        let mut editor = TransientWarpEditorView::new(44100 * 4, 44100, 120.0);
        assert_eq!(editor.total_samples, 44100 * 4);
        assert!(!editor.markers.is_empty());

        let canvas = Rect::new(0.0, 50.0, 800.0, 200.0);

        // 1. Coordinate transformations (sample <-> screen_x)
        let sample_pos = 44100; // 1 second in (Beat 2 at 120 BPM)
        let screen_x = editor.sample_to_screen_x(sample_pos, canvas);
        assert!((0.0..=800.0).contains(&screen_x));

        let roundtrip_sample = editor.screen_x_to_sample(screen_x, canvas);
        let sample_diff = (roundtrip_sample as i64 - sample_pos as i64).abs();
        assert!(
            sample_diff <= 100,
            "Sample roundtrip error too high: {sample_diff}"
        );

        // 2. Zoom & Visible Range
        editor.set_zoom(2.0);
        let (start, end) = editor.visible_sample_range();
        assert_eq!(end - start, (editor.total_samples as f32 / 2.0) as usize);

        // 3. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch box)
        const { assert!(WARP_MARKER_HIT_RADIUS >= 22.0) };
        const { assert!(WARP_MARKER_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // Test Hit Detection at marker 0
        let m0_x = editor.sample_to_screen_x(editor.markers[0].warped_sample_idx, canvas);
        let hit = editor.hit_test_marker((m0_x, canvas.y + 10.0), canvas);
        assert_eq!(hit, Some(0));

        // Test Miss Detection far away
        let miss = editor.hit_test_marker((m0_x + 60.0, canvas.y + 10.0), canvas);
        assert_ne!(miss, Some(0));

        // 4. Grid Snapping (1/16th note snap)
        let snap_test_sample = 22000;
        let snapped = editor.snap_sample_to_grid(snap_test_sample);
        assert_ne!(snapped, 0);

        // 5. Warp Marker Manipulation & Stretch Ratio
        let added_idx = editor.add_marker(50000, true);
        editor.move_marker(added_idx, 55000);
        assert_eq!(editor.markers[added_idx].warped_sample_idx, 55000);
        assert!(editor.markers[added_idx].stretch_ratio() > 1.0);
        assert!(editor.markers[added_idx].stretch_percentage() > 0.0);

        // 6. Reset Warp
        editor.reset_all_warp();
        assert_eq!(editor.markers[added_idx].warped_sample_idx, 50000);

        // 7. Marker Deletion
        assert!(editor.delete_marker(added_idx));

        // 8. Deterministic ASCII
        let ascii = editor.render_ascii(40);
        assert_eq!(ascii.len(), 40);
    }

    #[test]
    fn test_step_1362_step_sequencer_matrix_grid_and_touch_faders() {
        let mut seq = StepSequencerMatrixView::new(16, 128.0);
        assert_eq!(seq.num_steps, 16);
        assert_eq!(seq.lanes.len(), 6);

        let grid_origin = (110.0, 50.0);
        let cell_size = (MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT);

        // 1. Cell Minimum Hit Target Dimensions (>=44x44pt)
        assert!(cell_size.0 >= MIN_HIT_TARGET_PT);
        assert!(cell_size.1 >= MIN_HIT_TARGET_PT);

        // 2. Cell Bounding Box & Hit Testing
        let cell_rect = seq.calculate_cell_rect(1, 2, grid_origin, cell_size);
        assert_eq!(cell_rect.width, MIN_HIT_TARGET_PT);
        assert_eq!(cell_rect.height, MIN_HIT_TARGET_PT);

        let hit = seq.hit_test_cell(
            (cell_rect.x + 10.0, cell_rect.y + 10.0),
            grid_origin,
            cell_size,
        );
        assert_eq!(hit, Some((1, 2)));

        // 3. Step Active Toggling
        let initial_state = seq.lanes[0].steps[1].active;
        let new_state = seq.toggle_step(0, 1);
        assert_eq!(new_state, !initial_state);

        // 4. Per-Step Velocity & Probability Fader Controls
        seq.set_step_velocity(0, 1, 115);
        assert_eq!(seq.lanes[0].steps[1].velocity, 115);

        seq.set_step_probability(0, 1, 0.75);
        assert_eq!(seq.lanes[0].steps[1].probability, 0.75);

        seq.set_step_ratchet(0, 1, 3);
        assert_eq!(seq.lanes[0].steps[1].ratchet_count, 3);

        // 5. Playhead Advancement
        assert_eq!(seq.current_step, 0);
        seq.advance_step();
        assert_eq!(seq.current_step, 1);

        // 6. Mode Switcher
        seq.edit_mode = StepEditMode::Velocity;
        assert_eq!(seq.edit_mode, StepEditMode::Velocity);

        // 7. ASCII representation
        let ascii = seq.render_ascii();
        assert!(ascii.contains("Kick"));
        assert!(ascii.contains("Snare"));
    }

    #[test]
    fn test_step_1363_1367_isomorphic_microtonal_tuning_keyboard_intervals_and_bounds() {
        let mut kbd =
            IsomorphicTuningKeyboardView::new(19, 261.6255653, "C4", "19-EDO Equal Temperament");
        assert_eq!(kbd.edo_division, 19);
        assert_eq!(kbd.keys.len(), 28); // 7 cols x 4 rows

        let origin = (50.0, 50.0);

        // 1. Key Radius & Hit Target Compliance (Diameter >= 52pt > 44pt)
        assert!(kbd.key_radius_pt >= 24.0);
        assert!(kbd.key_radius_pt * 2.0 >= MIN_HIT_TARGET_PT);

        // 2. Hexagonal Center Calculations & Hit Testing
        let (cx, cy) =
            IsomorphicTuningKeyboardView::calculate_key_center(0, 0, origin, kbd.key_radius_pt);
        let hit = kbd.hit_test_key((cx, cy), origin);
        assert_eq!(hit, Some(0));

        let miss = kbd.hit_test_key((cx + 100.0, cy + 100.0), origin);
        assert_ne!(miss, Some(0));

        // 3. Microtonal Harmonic Intervals & Cents
        // Key (0, 0) should be Root Unison
        assert_eq!(kbd.keys[0].category, IntervalCategory::RootUnison);
        assert!(kbd.keys[0].is_root);
        assert!((kbd.keys[0].cents_from_root - 0.0).abs() < 0.001);
        assert!((kbd.keys[0].frequency_hz - 261.6255653).abs() < 0.01);

        // 4. Polyphonic Key Press Tracking
        assert!(kbd.pressed_keys.is_empty());
        kbd.press_key(0);
        assert!(kbd.keys[0].is_pressed);
        assert_eq!(kbd.pressed_keys.len(), 1);

        kbd.press_key(5);
        assert_eq!(kbd.pressed_keys.len(), 2);

        kbd.release_key(0);
        assert!(!kbd.keys[0].is_pressed);
        assert_eq!(kbd.pressed_keys.len(), 1);

        kbd.release_all();
        assert!(kbd.pressed_keys.is_empty());

        // 5. 31-EDO Microtonal Reconfiguration
        let kbd31 =
            IsomorphicTuningKeyboardView::new(31, 440.0, "A4", "31-EDO Quarter-Comma Meantone");
        assert_eq!(kbd31.edo_division, 31);
        let ascii31 = kbd31.render_ascii();
        assert!(ascii31.contains("31 EDO"));
    }

    #[test]
    fn test_step_1364_dynamic_envelope_follower_ball_physics_and_sidechain() {
        let mut env = EnvelopeFollowerView::new();
        assert_eq!(env.mode, EnvelopeMode::OptoBallistic);
        assert_eq!(env.sidechain_source, SidechainSource::Track1Kick);

        // 1. dB Normalization Math
        assert_eq!(EnvelopeFollowerView::db_to_norm(-60.0), 0.0);
        assert_eq!(EnvelopeFollowerView::db_to_norm(0.0), 1.0);
        assert_eq!(EnvelopeFollowerView::norm_to_db(0.0), -60.0);
        assert_eq!(EnvelopeFollowerView::norm_to_db(1.0), 0.0);

        // 2. Ball Physics Real-Time Step
        let initial_pos = env.physics.position_db;
        env.feed_input_sample(-6.0, 0.016); // 16ms frame step with -6dB signal
        assert!(env.physics.position_db > initial_pos);

        // Test Multiple Steps Converge toward Target
        for _ in 0..60 {
            env.feed_input_sample(-6.0, 0.016);
        }
        let diff = (env.physics.position_db - (-6.0)).abs();
        assert!(
            diff < 2.0,
            "Ball physics did not converge: pos={}",
            env.physics.position_db
        );

        // 3. Sidechain Source Switching
        env.sidechain_source = SidechainSource::Bus1Drums;
        assert_eq!(env.sidechain_source.display_name(), "Bus 1: Drum Group");

        // 4. ASCII Representation
        let ascii = env.render_ascii(30);
        assert!(ascii.contains('O'));
    }

    #[test]
    fn test_step_1365_tactile_bezier_automation_editor_pinch_zoom_and_curves() {
        let mut auto = BezierAutomationEditorView::new("Cutoff", "Hz", 20.0, 20000.0, 16.0);
        assert_eq!(auto.total_beats, 16.0);
        assert!(auto.nodes.len() >= 5);

        let canvas = Rect::new(0.0, 50.0, 800.0, 200.0);

        // 1. Coordinate Transformations & Pinch-to-Zoom Scaling
        let (sx, sy) = auto.time_value_to_screen(4.0, 0.5, canvas);
        assert!((0.0..=800.0).contains(&sx));
        assert!((50.0..=250.0).contains(&sy));

        let (roundtrip_time, roundtrip_val) = auto.screen_to_time_value((sx, sy), canvas);
        assert!((roundtrip_time - 4.0).abs() < 0.05);
        assert!((roundtrip_val - 0.5).abs() < 0.05);

        auto.apply_pinch_zoom(2.0, 1.0);
        assert_eq!(auto.zoom_x, 2.0);

        // 2. Minimum Hit Target (Radius >= 22pt -> 44x44pt bounding touch box)
        const { assert!(NODE_HIT_RADIUS >= 22.0) };
        const { assert!(NODE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // Hit testing node 0
        let (n0_x, n0_y) =
            auto.time_value_to_screen(auto.nodes[0].time_beats, auto.nodes[0].value, canvas);
        let hit = auto.hit_test_node((n0_x, n0_y), canvas);
        assert_eq!(hit, Some(0));

        // 3. Curve Interpolation Evaluation
        // Test Linear segment
        let lin_node_idx = auto
            .nodes
            .iter()
            .position(|n| matches!(n.curve, AutomationCurveType::Linear))
            .unwrap();
        let lin_start_t = auto.nodes[lin_node_idx].time_beats;
        let lin_end_t = auto.nodes[lin_node_idx + 1].time_beats;
        let mid_val = auto.evaluate_curve_at((lin_start_t + lin_end_t) * 0.5);
        let expected_mid =
            (auto.nodes[lin_node_idx].value + auto.nodes[lin_node_idx + 1].value) * 0.5;
        assert!((mid_val - expected_mid).abs() < 0.01);

        // Test Exponential & Bezier
        let exp_val = auto.evaluate_curve_at(2.0);
        assert!((0.0..=1.0).contains(&exp_val));

        // 4. Node Insertion & Deletion
        let initial_count = auto.nodes.len();
        let inserted_idx = auto.insert_node(6.0, 0.75);
        assert_eq!(auto.nodes.len(), initial_count + 1);

        let deleted = auto.delete_node(inserted_idx);
        assert!(deleted);
        assert_eq!(auto.nodes.len(), initial_count);

        // 5. Grid Snapping
        assert_eq!(auto.snap_beat(3.22), 3.25);
        assert_eq!(auto.snap_beat(3.38), 3.50);

        // 6. ASCII Rendering
        let ascii = auto.render_ascii(40, 8);
        assert!(ascii.contains('O'));
        assert!(ascii.contains('*'));
    }
}
