// Summoner DAW - Tier 50 GUI Milestones Unit Test Suite (Steps 1321-1330)

#[cfg(test)]
mod tests {
    use crate::layout_math::OperatingSystem;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::dpi_scale_panel::DpiScalePanelView;
    use crate::views::keybinding_editor::{KeyModifiers, KeyShortcut, KeybindingEditorView};
    use crate::views::live_macro_rack::{
        LiveMacroRackView, LiveXyPadState, MIN_XY_PAD_SIZE, PUCK_HIT_RADIUS, PUCK_RADIUS,
    };
    use crate::views::meter_bridge_view::{ChannelMeterState, MeterBridgeView, DB_MAX, DB_MIN};
    use crate::views::spectrogram_3d_view::{
        Spectrogram3DView, FREQ_MARKERS, NUM_FFT_BINS, NUM_HISTORY_SLICES,
    };

    #[test]
    fn test_step_1321_1326_live_macro_rack_xy_pad_coordinate_mapping() {
        let pad_rect = (100.0, 100.0, 200.0, 200.0); // (x, y, w, h)

        // Center normalized (0.5, 0.5) -> Canvas (200.0, 200.0)
        let (cx, cy) = LiveXyPadState::normalized_to_canvas(0.5, 0.5, pad_rect);
        assert!((cx - 200.0).abs() < 1e-4);
        assert!((cy - 200.0).abs() < 1e-4);

        // Top-Left normalized (0.0, 1.0) -> Canvas (100.0, 100.0)
        let (tl_x, tl_y) = LiveXyPadState::normalized_to_canvas(0.0, 1.0, pad_rect);
        assert!((tl_x - 100.0).abs() < 1e-4);
        assert!((tl_y - 100.0).abs() < 1e-4);

        // Bottom-Right normalized (1.0, 0.0) -> Canvas (300.0, 300.0)
        let (br_x, br_y) = LiveXyPadState::normalized_to_canvas(1.0, 0.0, pad_rect);
        assert!((br_x - 300.0).abs() < 1e-4);
        assert!((br_y - 300.0).abs() < 1e-4);

        // Roundtrip canvas to normalized
        let (norm_x, norm_y) = LiveXyPadState::canvas_to_normalized(200.0, 200.0, pad_rect);
        assert!((norm_x - 0.5).abs() < 1e-4);
        assert!((norm_y - 0.5).abs() < 1e-4);

        let (norm_tl_x, norm_tl_y) = LiveXyPadState::canvas_to_normalized(100.0, 100.0, pad_rect);
        assert!((norm_tl_x - 0.0).abs() < 1e-4);
        assert!((norm_tl_y - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_step_1321_1326_live_macro_rack_hit_target_bounds() {
        assert_eq!(MIN_XY_PAD_SIZE, 160.0);
        assert_eq!(PUCK_RADIUS, 14.0);
        assert_eq!(PUCK_HIT_RADIUS, 22.0); // 44x44pt hit target

        let puck_center = (200.0, 200.0);

        // Click directly on center -> hit
        assert!(LiveXyPadState::is_puck_hit(puck_center, (200.0, 200.0)));

        // Click within 22pt radius -> hit
        assert!(LiveXyPadState::is_puck_hit(
            puck_center,
            (200.0 + 15.0, 200.0)
        ));
        assert!(LiveXyPadState::is_puck_hit(
            puck_center,
            (200.0, 200.0 - 21.9)
        ));

        // Click outside 22pt radius -> miss
        assert!(!LiveXyPadState::is_puck_hit(
            puck_center,
            (200.0 + 22.5, 200.0)
        ));
        assert!(!LiveXyPadState::is_puck_hit(
            puck_center,
            (200.0, 200.0 + 25.0)
        ));

        let mut rack = LiveMacroRackView::new();
        let hit_rect = rack.pad_left.puck_hit_bounds((0.0, 0.0, 200.0, 200.0));
        assert!(hit_rect.width >= MIN_HIT_TARGET_PT);
        assert!(hit_rect.height >= MIN_HIT_TARGET_PT);

        // Test spring to center physics
        rack.pad_left.spring_to_center = true;
        rack.pad_left.set_pos(0.9, 0.1);
        rack.pad_left.is_dragging = false;
        rack.pad_left.update_spring();
        assert!(rack.pad_left.x_val < 0.9);
        assert!(rack.pad_left.y_val > 0.1);
    }

    #[test]
    fn test_step_1322_spectrogram_3d_projection_and_color_mapping() {
        assert_eq!(NUM_FFT_BINS, 64);
        assert_eq!(NUM_HISTORY_SLICES, 32);
        assert_eq!(FREQ_MARKERS.len(), 5);

        let view = Spectrogram3DView::new();
        assert_eq!(view.slices.len(), NUM_HISTORY_SLICES);

        // Test 3D Projection coordinate math
        let center = (200.0, 150.0);
        let (px, py) =
            Spectrogram3DView::project_3d_point(0.5, 0.5, 0.0, center, 300.0, 200.0, 0.0, 0.0, 1.0);
        // At 0 yaw & 0 pitch, centered point maps to center
        assert!((px - 200.0).abs() < 1e-4);
        assert!((py - 150.0).abs() < 1e-4);

        // Magnitude elevates Y in screen space (decreases screen py)
        let (_, py_elevated) =
            Spectrogram3DView::project_3d_point(0.5, 0.5, 1.0, center, 300.0, 200.0, 0.0, 0.0, 1.0);
        assert!(py_elevated < py);

        // Test spectral color ramp boundaries
        let c_low = Spectrogram3DView::magnitude_to_rgb(0.05);
        assert!(c_low.2 > c_low.0); // Low magnitude is predominantly deep blue/purple

        let c_mid = Spectrogram3DView::magnitude_to_rgb(0.40);
        assert!(c_mid.1 > 100); // Mid magnitude has strong green/cyan component

        let c_high = Spectrogram3DView::magnitude_to_rgb(0.95);
        assert_eq!(c_high.0, 255); // High magnitude has maximum red/hot pink saturation
    }

    #[test]
    fn test_step_1323_1327_keybinding_conflict_detection_and_modifiers() {
        let mut editor = KeybindingEditorView::new(OperatingSystem::Windows);

        // Initially configured with zero conflicts
        let initial_conflicts = editor.detect_conflicts();
        assert_eq!(initial_conflicts.len(), 0);

        // Intentionally introduce duplicate conflict: assign Space to Toggle Record
        editor.assign_shortcut(
            "transport_record",
            Some(KeyShortcut::new("Space", KeyModifiers::none())),
            false,
        );

        let conflicts = editor.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].shortcut_display, "Space");
        assert_eq!(conflicts[0].conflicting_action_ids.len(), 2);
        assert!(conflicts[0]
            .conflicting_action_ids
            .contains(&"transport_play_pause".to_string()));
        assert!(conflicts[0]
            .conflicting_action_ids
            .contains(&"transport_record".to_string()));

        // Reset all defaults resolves conflict
        editor.reset_all_defaults();
        assert_eq!(editor.detect_conflicts().len(), 0);

        // Test Cross-OS modifier string formatting
        let win_mod = KeyModifiers::ctrl_shift();
        assert_eq!(
            win_mod.display_string(OperatingSystem::Windows),
            "Ctrl+Shift+"
        );
        assert_eq!(
            win_mod.display_string(OperatingSystem::Linux),
            "Ctrl+Shift+"
        );

        let mac_mod = KeyModifiers {
            meta: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(
            mac_mod.display_string(OperatingSystem::MacOS),
            "⇧ Shift+⌘ Cmd+"
        );
    }

    #[test]
    fn test_step_1324_meter_bridge_db_math_and_peak_hold() {
        assert_eq!(DB_MIN, -60.0);
        assert_eq!(DB_MAX, 6.0);

        // Test dB to fraction mapping
        assert_eq!(MeterBridgeView::db_to_fraction(-60.0), 0.0);
        assert_eq!(MeterBridgeView::db_to_fraction(6.0), 1.0);
        let frac_0db = MeterBridgeView::db_to_fraction(0.0);
        assert!((frac_0db - (60.0 / 66.0)).abs() < 1e-4);

        // Test Peak Hold Decay and Clipping
        let mut ch = ChannelMeterState::new(1, "Test Track");
        assert!(!ch.clipped_l);

        // Feed +1.0 dBFS signal -> clips
        ch.update_levels(1.0, 1.0, -6.0, -6.0, 0.5);
        assert!(ch.clipped_l);
        assert!(ch.clipped_r);
        assert_eq!(ch.peak_hold_l_db, 1.0);

        // Reset clip
        ch.reset_clip();
        assert!(!ch.clipped_l);

        // Lower level -> peak hold decays gradually
        ch.update_levels(-10.0, -10.0, -14.0, -14.0, 0.5);
        assert!((ch.peak_hold_l_db - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_step_1325_dpi_scaling_panel_metrics_and_compliance() {
        let mut panel = DpiScalePanelView::new(OperatingSystem::Windows);
        assert_eq!(panel.detected_dpi, 120.0);
        assert_eq!(panel.detected_scale, 1.25);
        assert!(panel.is_touch_target_compliant());

        // Test physical touch target scaling (44pt * 1.25 = 55.0px)
        assert_eq!(panel.physical_touch_target_px(), 55.0);

        // Apply 2.0x preset
        panel.apply_preset(2.0);
        assert_eq!(panel.effective_scale(), 2.0);
        assert_eq!(panel.physical_touch_target_px(), 88.0);
        assert!(panel.is_touch_target_compliant());

        // Test macOS Retina defaults
        let mac_panel = DpiScalePanelView::new(OperatingSystem::MacOS);
        assert_eq!(mac_panel.detected_scale, 2.00);
    }
}
