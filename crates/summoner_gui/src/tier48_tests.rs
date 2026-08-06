// Summoner DAW - Tier 48 High-DPI Advanced Workflow & Modular GUI Component System Unit Tests (Steps 1281-1300)

#[cfg(test)]
mod tests {
    use crate::docking_layout::{DockPreset, DockingLayoutManager};
    use crate::layout_math::{OperatingSystem, Rect, SpatialLayoutCalculator};
    use crate::oscilloscope_view::{OscilloscopeMath, OscilloscopeMode, OscilloscopeView};
    use crate::patch_matrix::PatchMatrixView;
    use crate::touch_controls::{KnobState, SliderOrientation, SliderState, MIN_HIT_TARGET_PT};
    use crate::transport_bar::{TimeSignature, TransportBarView};

    #[test]
    fn test_step_1281_spatial_math_layout_and_cross_os_padding() {
        let calc = SpatialLayoutCalculator::for_os(OperatingSystem::Windows);
        assert_eq!(calc.config().scrollbar_padding_px, 17.0);
        assert_eq!(calc.config().dpi_scale, 1.25);
        assert_eq!(calc.config().min_hit_target_pt, 44.0);

        let mac_calc = SpatialLayoutCalculator::for_os(OperatingSystem::MacOS);
        assert_eq!(mac_calc.config().scrollbar_padding_px, 0.0);

        let linux_calc = SpatialLayoutCalculator::for_os(OperatingSystem::Linux);
        assert_eq!(linux_calc.config().scrollbar_padding_px, 14.0);

        let rect_a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect_b = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert!(rect_a.intersects(&rect_b));

        let non_overlapping = Rect::new(150.0, 150.0, 50.0, 50.0);
        assert!(!rect_a.intersects(&non_overlapping));

        let clamped = calc.ensure_min_hit_target(Rect::new(0.0, 0.0, 30.0, 20.0));
        assert!(clamped.width >= 44.0);
        assert!(clamped.height >= 44.0);
    }

    #[test]
    fn test_step_1281_docking_layout_manager_presets_and_splits() {
        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let mut dock = DockingLayoutManager::new(DockPreset::DefaultTiled, viewport);
        assert_eq!(dock.preset(), DockPreset::DefaultTiled);
        assert!(dock.root_node().is_some());

        dock.load_preset(DockPreset::DualMonitor);
        assert_eq!(dock.preset(), DockPreset::DualMonitor);
        assert!(dock.root_node().is_some());

        dock.load_preset(DockPreset::SingleFocus);
        assert_eq!(dock.preset(), DockPreset::SingleFocus);
        assert!(dock.root_node().is_some());

        let layout = dock.compute_layout();
        assert!(!layout.panels.is_empty());
    }

    #[test]
    fn test_step_1282_touch_control_widgets_and_hit_targets() {
        let mut knob = KnobState::new(1000.0, 20.0, 20000.0).with_label("Cutoff");
        assert_eq!(knob.value, 1000.0);

        knob.update_from_drag_delta(-10.0, false);
        assert!(knob.value != 1000.0);

        knob.reset_to_default();
        assert_eq!(knob.value, 1000.0);

        let mut slider = SliderState::new(0.0, -60.0, 12.0, SliderOrientation::Vertical)
            .with_label("Master Volume");
        slider.set_normalized(0.75);
        assert!(slider.value > -60.0 && slider.value < 12.0);
        assert_eq!(MIN_HIT_TARGET_PT, 44.0);
    }

    #[test]
    fn test_step_1283_oscilloscope_visualizer_and_phase_correlation() {
        let mut osc = OscilloscopeView::new();
        assert_eq!(osc.config.mode, OscilloscopeMode::Stereo);
        assert_eq!(osc.config.zoom_level, 1.0);

        let left_ch = vec![0.5f32; 512];
        let right_ch = vec![0.5f32; 512];
        osc.update_main_audio(&left_ch, &right_ch);

        assert!((osc.phase_correlation() - 1.0).abs() < 1e-4);

        // Anti-phase signals -> correlation -1.0
        let inv_right = vec![-0.5f32; 512];
        let anti_corr = OscilloscopeMath::calculate_phase_correlation(&left_ch, &inv_right);
        assert!((anti_corr - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_step_1284_patch_matrix_routing_and_pin_connections() {
        let mut matrix = PatchMatrixView::with_default_nodes();
        assert_eq!(matrix.connections.len(), 3); // Default initial connections

        // Connect LFO1 to Filter Cutoff
        matrix.connect("lfo1", "cutoff", 0.75);
        assert!(matrix.is_connected("lfo1", "cutoff"));
        assert_eq!(matrix.get_intensity("lfo1", "cutoff"), 0.75);

        // Toggle connection -> disconnect
        matrix.toggle_connection("lfo1", "cutoff");
        assert!(!matrix.is_connected("lfo1", "cutoff"));

        // Connect again and verify route list
        matrix.connect("midi_mod", "fx_wet", 0.5);
        let active = matrix.get_active_routes();
        assert!(active
            .iter()
            .any(|r| r.source_id == "midi_mod" && r.dest_id == "fx_wet"));
    }

    #[test]
    fn test_step_1285_transport_bar_and_bpm_tap_calculation() {
        let mut transport = TransportBarView::new();
        assert!(!transport.is_playing);
        assert!(!transport.is_recording);
        assert_eq!(transport.bpm, 120.0);

        transport.toggle_play();
        assert!(transport.is_playing);

        transport.toggle_record();
        assert!(transport.is_recording);

        // Tap tempo test
        transport.tap_bpm(1000.0);
        transport.tap_bpm(1500.0); // 500ms interval = 120 BPM
        assert!(transport.bpm >= 110.0 && transport.bpm <= 130.0);

        transport.time_signature = TimeSignature::SevenEight;
        assert_eq!(transport.time_signature.to_string(), "7/8");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_step_1288_tier48_gui_app_and_command_palette_integration() {
        use crate::app::SummonerApp;
        use crate::command_palette::CommandPalette;
        use std::sync::Arc;
        use summoner_core::param_bus::ParamBus;
        use summoner_project::create_default_project;

        let proj = create_default_project("Tier48 GUI Test Session");
        let bus = Arc::new(ParamBus::new());
        let app = SummonerApp::new(proj, bus);

        // Verify Tier 48 components initialized inside app
        assert_eq!(app.docking_manager.preset(), DockPreset::DefaultTiled);
        assert_eq!(app.transport_bar.bpm, 120.0);

        // Test command palette action IDs
        let cp = CommandPalette::new();
        let tier48_actions = [
            "open_docking_layout",
            "open_patch_matrix",
            "open_oscilloscope",
            "toggle_transport_bar",
            "open_touch_controls",
        ];

        for action_id in &tier48_actions {
            assert!(
                cp.actions.iter().any(|a| a.action_id == *action_id),
                "Command palette missing Tier 48 action: {}",
                action_id
            );
        }
    }
}
