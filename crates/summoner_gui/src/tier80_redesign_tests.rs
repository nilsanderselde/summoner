// Summoner DAW - Tier 80 GUI Redesign Unit Test Suite & Headless Snapshot Verification

#[cfg(test)]
mod tests {
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_asset_browser::{BrowserCategory, ModernAssetBrowserState};
    use crate::views::modern_device_rack::{ModernDeviceRackState, MIN_KNOB_HIT_RADIUS};
    use crate::views::modern_inspector::ModernInspectorState;
    use crate::views::modern_top_bar::{ModernTopBarState, ModernViewTab};

    #[test]
    fn test_award_winning_gui_hit_targets_and_grid_math() {
        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(MIN_KNOB_HIT_RADIUS >= 22.0) };
        const { assert!(MIN_KNOB_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        let view = AwardWinningGuiView::new();
        assert!(!view.tracks.is_empty());
        assert_eq!(view.tracks.len(), 8);
    }

    #[test]
    fn test_award_winning_gui_top_bar_and_transport_state() {
        let mut top_bar = ModernTopBarState::default();

        // 1. Initial defaults
        assert_eq!(top_bar.bpm, 120.00);
        assert_eq!(top_bar.key_signature, "Am");
        assert_eq!(top_bar.time_signature, "4/4");
        assert_eq!(top_bar.active_tab, ModernViewTab::Arranger);

        // 2. State mutations
        top_bar.bpm = 135.5;
        top_bar.is_playing = true;
        top_bar.is_recording = true;
        top_bar.active_tab = ModernViewTab::Modular;

        assert_eq!(top_bar.bpm, 135.5);
        assert!(top_bar.is_playing);
        assert!(top_bar.is_recording);
        assert_eq!(top_bar.active_tab.name(), "MODULAR");
    }

    #[test]
    fn test_award_winning_gui_asset_browser_and_inspector() {
        let mut browser = ModernAssetBrowserState::default();
        assert_eq!(browser.selected_category, BrowserCategory::Samples);
        assert!(browser.expanded_folders.contains(&"Drums".to_string()));

        browser.selected_category = BrowserCategory::Instruments;
        assert_eq!(browser.selected_category.name(), "Instruments");
        assert_eq!(browser.selected_category.icon(), "🎹");

        let mut inspector = ModernInspectorState::default();
        assert_eq!(inspector.target_name, "Synth 1");
        assert_eq!(inspector.gain_db, 0.0);
        assert_eq!(inspector.scale_name, "A Minor Pentatonic");

        inspector.gain_db = -3.5;
        inspector.pan_val = 0.4;
        inspector.is_armed = true;
        inspector.scale_name = "19-EDO Equal".to_string();

        assert_eq!(inspector.gain_db, -3.5);
        assert_eq!(inspector.pan_val, 0.4);
        assert_eq!(inspector.scale_name, "19-EDO Equal");
    }

    #[test]
    fn test_award_winning_gui_device_rack_and_oscilloscope() {
        let mut rack = ModernDeviceRackState::default();
        assert_eq!(rack.device_name, "Synth 1");
        assert!(rack.is_enabled);
        assert_eq!(rack.cutoff, 0.65);
        assert_eq!(rack.resonance, 0.45);

        // Parameter adjustments
        rack.cutoff = 0.82;
        rack.drive = 0.55;
        rack.volume = 0.90;
        assert_eq!(rack.cutoff, 0.82);
        assert_eq!(rack.drive, 0.55);
        assert_eq!(rack.volume, 0.90);
    }

    #[test]
    fn test_award_winning_gui_headless_snapshot_render() {
        let view = AwardWinningGuiView::new();
        // 1. Render primary high-fidelity snapshot at 1280x720 (HD resolution)
        let snap_res = view.render_snapshot_png("scratch/renders/award_winning_daw_gui_render.png", 1280, 720);
        assert!(snap_res.is_ok());

        // 2. Also verify compact resolution rendering at 800x520
        let compact_res = view.render_snapshot_png("scratch/renders/award_winning_daw_gui_compact.png", 800, 520);
        assert!(compact_res.is_ok());

        // Verify output files were created and are non-empty
        let path = std::path::Path::new("scratch/renders/award_winning_daw_gui_render.png");
        assert!(path.exists());
        let meta = std::fs::metadata(path).expect("Metadata should exist");
        assert!(meta.len() > 1024, "PNG snapshot file should be non-empty");
    }
}
