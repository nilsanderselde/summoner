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

    #[test]
    fn test_modular_routing_canvas_nodes_and_cords() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.modular_mode, crate::views::award_winning_gui_view::ModularCanvasMode::PatchCords);
        assert_eq!(view.modular_nodes.len(), 4);
        assert_eq!(view.patch_cords.len(), 3);

        // Verify audio vs CV color coding on ports
        let osc = &view.modular_nodes[0];
        assert_eq!(osc.id, "osc_1");
        assert_eq!(osc.ports[0].kind, crate::views::award_winning_gui_view::ModularPortKind::ModulationIn);
        assert_eq!(osc.ports[0].kind.color_rgb(), (245, 158, 11)); // Amber for CV
        assert_eq!(osc.ports[2].kind, crate::views::award_winning_gui_view::ModularPortKind::AudioOut);
        assert_eq!(osc.ports[2].kind.color_rgb(), (56, 189, 248)); // Cyan for Audio

        // Test adding a dynamic DSP module from DspNodeRegistry
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let desc = registry.get("FilterLadder").expect("FilterLadder must exist");
        view.add_modular_node_from_descriptor(desc);

        assert_eq!(view.modular_nodes.len(), 5);
        let last_node = view.modular_nodes.last().unwrap();
        assert_eq!(last_node.kind_id, "FilterLadder");
        assert_eq!(view.selected_modular_node_id.as_deref(), Some(last_node.id.as_str()));
        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("FilterLadder"));
        assert_eq!(view.inspector_state.selected_node_kind.as_deref(), Some("FilterLadder"));
    }

    #[test]
    fn test_modular_routing_matrix_and_pro_view_integration() {
        let mut view = AwardWinningGuiView::new();
        view.modular_mode = crate::views::award_winning_gui_view::ModularCanvasMode::RoutingMatrix;
        assert_eq!(view.modular_mode.label(), "▦ Routing Matrix");

        // Verify patch matrix integration
        assert!(view.patch_matrix.sources.len() >= 6);
        assert!(view.patch_matrix.destinations.len() >= 6);
        assert!(view.patch_matrix.is_connected("lfo1", "cutoff"));

        // Toggle connection
        view.patch_matrix.toggle_connection("lfo2", "resonance");
        assert!(view.patch_matrix.is_connected("lfo2", "resonance"));
    }

    #[test]
    fn test_modular_canvas_rack_and_inspector_synchronization() {
        let mut view = AwardWinningGuiView::new();

        // Mutate parameter in rack
        view.device_rack_state.node_param_values.insert("cutoff".to_string(), 0.77);
        view.device_rack_state.selected_node_kind = Some("ReverbPlate".to_string());
        view.device_rack_state.device_name = "Algorithmic Plate Reverb".to_string();

        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Inspector must be synchronized with device rack
        assert_eq!(view.inspector_state.selected_node_kind.as_deref(), Some("ReverbPlate"));
        assert_eq!(view.inspector_state.target_name, "Algorithmic Plate Reverb");
        assert_eq!(view.inspector_state.node_param_values.get("cutoff"), Some(&0.77));
    }

    #[test]
    fn test_dsp_node_registry_300_plus_modules_completeness() {
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 305, "Registry should contain >= 305 descriptors, found {}", registry.list_all().len());

        let inventory = crate::dsp_node_ui::DspNodeRegistry::inventory();
        assert!(inventory.len() >= 305, "Inventory should contain >= 305 entries, found {}", inventory.len());

        // Verify key newly registered DSP modules
        let limiter = registry.get("TruePeakLimiter").expect("TruePeakLimiter must exist");
        assert_eq!(limiter.category, crate::dsp_node_ui::DspNodeCategory::DynamicsMaster);
        assert!(limiter.params.len() >= 5);

        let nam = registry.get("NamAmpNode").expect("NamAmpNode must exist");
        assert_eq!(nam.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);
        assert!(nam.params.len() >= 4);

        let koto = registry.get("Koto").expect("Koto must exist");
        assert_eq!(koto.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert!(koto.params.len() >= 5);

        let free_reed = registry.get("FreeReedNode").expect("FreeReedNode must exist");
        assert_eq!(free_reed.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert!(free_reed.params.len() >= 5);

        let cloud = registry.get("GranularCloudNode").expect("GranularCloudNode must exist");
        assert_eq!(cloud.category, crate::dsp_node_ui::DspNodeCategory::Oscillator);
        assert!(cloud.params.len() >= 6);

        let room = registry.get("SpatialRoomNode").expect("SpatialRoomNode must exist");
        assert_eq!(room.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);
        assert!(room.params.len() >= 7);

        let panner = registry.get("ContinuousSpatialPannerDoppler3D").expect("ContinuousSpatialPannerDoppler3D must exist");
        assert_eq!(panner.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);
        assert!(panner.params.len() >= 5);
    }

    #[test]
    fn test_modular_canvas_category_specific_port_creation() {
        let mut view = AwardWinningGuiView::new();
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();

        // 1. Add Koto (AcousticPhysicalModel / Synth source)
        let koto_desc = registry.get("Koto").expect("Koto must exist");
        view.add_modular_node_from_descriptor(koto_desc);
        let koto_node = view.modular_nodes.last().unwrap();
        assert_eq!(koto_node.kind_id, "Koto");
        let koto_port_ids: Vec<_> = koto_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(koto_port_ids.contains(&"voct"));
        assert!(koto_port_ids.contains(&"gate"));
        assert!(koto_port_ids.contains(&"out"));
        let koto_id = koto_node.id.clone();

        // 2. Add SpatialRoomNode (SpatialSurround)
        let room_desc = registry.get("SpatialRoomNode").expect("SpatialRoomNode must exist");
        view.add_modular_node_from_descriptor(room_desc);
        let room_node = view.modular_nodes.last().unwrap();
        assert_eq!(room_node.kind_id, "SpatialRoomNode");
        let room_port_ids: Vec<_> = room_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(room_port_ids.contains(&"in_l"));
        assert!(room_port_ids.contains(&"in_r"));
        assert!(room_port_ids.contains(&"pos_cv"));
        assert!(room_port_ids.contains(&"out_l"));
        assert!(room_port_ids.contains(&"out_r"));
        let room_id = room_node.id.clone();

        // 3. Add TruePeakLimiter (DynamicsMaster)
        let limiter_desc = registry.get("TruePeakLimiter").expect("TruePeakLimiter must exist");
        view.add_modular_node_from_descriptor(limiter_desc);
        let limiter_node = view.modular_nodes.last().unwrap();
        assert_eq!(limiter_node.kind_id, "TruePeakLimiter");
        let lim_port_ids: Vec<_> = limiter_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(lim_port_ids.contains(&"in_l"));
        assert!(lim_port_ids.contains(&"in_r"));
        assert!(lim_port_ids.contains(&"sidechain"));
        assert!(lim_port_ids.contains(&"out_l"));
        assert!(lim_port_ids.contains(&"out_r"));

        // 4. Connect Koto Audio Out to SpatialRoomNode In L
        view.patch_cords.push(crate::views::award_winning_gui_view::ModularPatchCord {
            from_node_id: koto_id.clone(),
            from_port_id: "out".into(),
            to_node_id: room_id.clone(),
            to_port_id: "in_l".into(),
            is_audio: true,
            intensity: 1.0,
        });

        assert_eq!(view.patch_cords.len(), 4); // 3 defaults + 1 new
        let last_cord = view.patch_cords.last().unwrap();
        assert_eq!(last_cord.from_node_id, koto_id);
        assert_eq!(last_cord.to_node_id, room_id);
        assert!(last_cord.is_audio);
    }
}
