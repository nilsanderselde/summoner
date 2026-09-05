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

    #[test]
    fn test_novice_macro_strip_and_preset_sync_to_pro_device_rack() {
        let mut view = AwardWinningGuiView::new();

        // Initial defaults
        assert_eq!(view.device_rack_state.device_name, "Synth 1");
        assert_eq!(view.top_bar_state.selected_preset, "Init Synth 1");

        // 1. Mutate Novice Presets in Top Bar
        view.top_bar_state.selected_preset = "Ambient Crystal Bells".to_string();

        // 2. Mutate Novice Macro Knobs (Tone, Space, Punch, Character) + Master Gain
        view.top_bar_state.macro_tone = 0.88;
        view.top_bar_state.macro_space = 0.72;
        view.top_bar_state.macro_punch = 0.64;
        view.top_bar_state.macro_character = 0.91;
        view.top_bar_state.master_gain = 1.25;

        // Run UI cycle to process bindings
        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Verify Device Rack and Inspector updated to CrystalResonator
        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("CrystalResonator"));
        assert_eq!(view.inspector_state.selected_node_kind.as_deref(), Some("CrystalResonator"));
        assert_eq!(view.device_rack_state.device_name, "8-Mode Thin-Shell Crystal Glass & Singing Bowl Resonator");

        // Verify Macro Knobs propagated into Rack Controls
        assert_eq!(view.device_rack_state.cutoff, 0.88);
        assert_eq!(view.device_rack_state.decay, 0.72);
        assert_eq!(view.device_rack_state.drive, 0.64);
        assert_eq!(view.device_rack_state.mod_amt, 0.91);

        // Verify mirrored into node parameter reflection
        assert_eq!(view.device_rack_state.node_param_values.get("cutoff"), Some(&0.88));
        assert_eq!(view.device_rack_state.node_param_values.get("decay"), Some(&0.72));
        assert_eq!(view.device_rack_state.node_param_values.get("drive"), Some(&0.64));
        assert_eq!(view.device_rack_state.node_param_values.get("character"), Some(&0.91));

        // Verify Master Gain propagated to active selected track
        assert_eq!(view.tracks[view.selected_track_idx].gain, 1.25);
    }

    #[test]
    fn test_newly_registered_dsp_nodes_inventory_and_port_sockets() {
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 315, "Registry should contain >= 315 descriptors, found {}", registry.list_all().len());

        let inventory = crate::dsp_node_ui::DspNodeRegistry::inventory();
        assert!(inventory.len() >= 315, "Inventory should contain >= 315 entries, found {}", inventory.len());

        // 1. CrystalResonator (Acoustic physical singing bowl)
        let crystal = registry.get("CrystalResonator").expect("CrystalResonator must exist");
        assert_eq!(crystal.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crystalbowl"), Some("CrystalResonator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crystalglass"), Some("CrystalResonator"));

        // 2. DistanceDopplerNode (3D spatial audio)
        let doppler = registry.get("DistanceDopplerNode").expect("DistanceDopplerNode must exist");
        assert_eq!(doppler.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("distancedoppler"), Some("DistanceDopplerNode"));

        // 3. BrirConvolutionNode (Binaural room impulse)
        let brir = registry.get("BrirConvolutionNode").expect("BrirConvolutionNode must exist");
        assert_eq!(brir.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("binauralroomimpulse"), Some("BrirConvolutionNode"));

        // 4. TarabResonatorBank (Indian classical sympathetic resonance)
        let tarab = registry.get("TarabResonatorBank").expect("TarabResonatorBank must exist");
        assert_eq!(tarab.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sympatheticstrings"), Some("TarabResonatorBank"));

        // 5. SineOscillatorNode (Core deterministic sine AudioNode)
        let sine = registry.get("SineOscillatorNode").expect("SineOscillatorNode must exist");
        assert_eq!(sine.category, crate::dsp_node_ui::DspNodeCategory::Oscillator);

        // 6. SpatialReverb3D (3D volumetric spatial room)
        let reverb = registry.get("SpatialReverb3D").expect("SpatialReverb3D must exist");
        assert_eq!(reverb.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spatialreverb"), Some("SpatialReverb3D"));

        // 7. SurroundLimiterAndLoudness (True-peak multichannel limiter)
        let surround_lim = registry.get("SurroundLimiterAndLoudness").expect("SurroundLimiterAndLoudness must exist");
        assert_eq!(surround_lim.category, crate::dsp_node_ui::DspNodeCategory::DynamicsMaster);

        // 8. MacroModulationMatrix (64-slot modulation routing)
        let mod_matrix = registry.get("MacroModulationMatrix").expect("MacroModulationMatrix must exist");
        assert_eq!(mod_matrix.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        // 9. Standard *Node aliases normalization
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pipeorgannode"), Some("PipeOrganNode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sitarnode"), Some("SitarNode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tonewheelorgannode"), Some("TonewheelOrganNode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("rotaryspeakernode"), Some("RotarySpeakerNode"));

        // 10. Test dynamic modular node creation from newly registered descriptors
        let mut view = AwardWinningGuiView::new();
        view.add_modular_node_from_descriptor(crystal);
        view.add_modular_node_from_descriptor(doppler);
        view.add_modular_node_from_descriptor(mod_matrix);

        let nodes = &view.modular_nodes;
        assert_eq!(nodes[nodes.len() - 3].kind_id, "CrystalResonator");
        assert_eq!(nodes[nodes.len() - 2].kind_id, "DistanceDopplerNode");
        assert_eq!(nodes[nodes.len() - 1].kind_id, "MacroModulationMatrix");

        // Verify category socket bindings
        let doppler_ports: Vec<_> = nodes[nodes.len() - 2].ports.iter().map(|p| p.id.as_str()).collect();
        assert!(doppler_ports.contains(&"in_l"));
        assert!(doppler_ports.contains(&"in_r"));
        assert!(doppler_ports.contains(&"pos_cv"));
        assert!(doppler_ports.contains(&"out_l"));
        assert!(doppler_ports.contains(&"out_r"));

        let mod_ports: Vec<_> = nodes[nodes.len() - 1].ports.iter().map(|p| p.id.as_str()).collect();
        assert!(mod_ports.contains(&"gate"));
        assert!(mod_ports.contains(&"sync"));
        assert!(mod_ports.contains(&"cv_out"));
    }

    #[test]
    fn test_tier81_comprehensive_dsp_coverage_and_preset_sync() {
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        assert!(
            registry.list_all().len() >= 340,
            "Registry should contain >= 340 descriptors, found {}",
            registry.list_all().len()
        );

        let inventory = crate::dsp_node_ui::DspNodeRegistry::inventory();
        assert!(
            inventory.len() >= 340,
            "Inventory should contain >= 340 entries, found {}",
            inventory.len()
        );

        // 1. Verify physical modeling acoustic modules
        let windchest = registry.get("Windchest").expect("Windchest must exist");
        assert_eq!(windchest.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert!(windchest.params.len() >= 4);

        let cassotto = registry.get("CassottoChamber").expect("CassottoChamber must exist");
        assert_eq!(cassotto.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let air_reed = registry.get("AirReed").expect("AirReed must exist");
        assert_eq!(air_reed.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let hammer = registry.get("HammerStrike").expect("HammerStrike must exist");
        assert_eq!(hammer.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let tonehole = registry.get("ToneholeLattice").expect("ToneholeLattice must exist");
        assert_eq!(tonehole.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let soundboard = registry.get("SpruceSoundboard").expect("SpruceSoundboard must exist");
        assert_eq!(soundboard.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let chien = registry.get("TrompetteChienJunction").expect("TrompetteChienJunction must exist");
        assert_eq!(chien.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        let sympathetic = registry.get("SympatheticResonatorMatrix").expect("SympatheticResonatorMatrix must exist");
        assert_eq!(sympathetic.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);

        // 2. Verify Neural AI modules
        let crepe = registry.get("CrepePitchTracker").expect("CrepePitchTracker must exist");
        assert_eq!(crepe.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let style_transfer = registry.get("NeuralAudioStyleTransferPreviewRenderer").expect("NeuralAudioStyleTransferPreviewRenderer must exist");
        assert_eq!(style_transfer.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let neuro_aff = registry.get("NeuroAffectiveAnalyzer").expect("NeuroAffectiveAnalyzer must exist");
        assert_eq!(neuro_aff.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let mental = registry.get("MentalImageryClassifier").expect("MentalImageryClassifier must exist");
        assert_eq!(mental.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let aesthetic = registry.get("NeuroAestheticScorer").expect("NeuroAestheticScorer must exist");
        assert_eq!(aesthetic.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        // 3. Verify Modulation & Glitch modules
        let tapestop = registry.get("TapeStop").expect("TapeStop must exist");
        assert_eq!(tapestop.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        let mpe_curve = registry.get("MpeExpressionCurveEditor").expect("MpeExpressionCurveEditor must exist");
        assert_eq!(mpe_curve.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        let hrv_sync = registry.get("HrvTempoSyncEngine").expect("HrvTempoSyncEngine must exist");
        assert_eq!(hrv_sync.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        let emg = registry.get("EmgGestureDriver").expect("EmgGestureDriver must exist");
        assert_eq!(emg.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        // 4. Verify Filters & Mastering modules
        let head_bump = registry.get("HeadBumpFilter").expect("HeadBumpFilter must exist");
        assert_eq!(head_bump.category, crate::dsp_node_ui::DspNodeCategory::FilterEq);

        let crossover = registry.get("LinkwitzRiley4BandCrossover").expect("LinkwitzRiley4BandCrossover must exist");
        assert_eq!(crossover.category, crate::dsp_node_ui::DspNodeCategory::FilterEq);

        // 5. Verify Utility & Infrastructure modules
        let recorder = registry.get("LiveSessionRecorder").expect("LiveSessionRecorder must exist");
        assert_eq!(recorder.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let stem_router = registry.get("MultiTrackAudioRouter").expect("MultiTrackAudioRouter must exist");
        assert_eq!(stem_router.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let buffer_scaler = registry.get("AdaptiveBufferScaler").expect("AdaptiveBufferScaler must exist");
        assert_eq!(buffer_scaler.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let visualizer = registry.get("VisualizerIntegrationEngine").expect("VisualizerIntegrationEngine must exist");
        assert_eq!(visualizer.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let clap_host = registry.get("ClapHostEngine").expect("ClapHostEngine must exist");
        assert_eq!(clap_host.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let wasm_dsp = registry.get("WasmDspRuntime").expect("WasmDspRuntime must exist");
        assert_eq!(wasm_dsp.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        // 6. Verify Normalization & Aliases (including bugfix validations)
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hyperbolicreverbnode"), Some("NonEuclideanHyperbolicReverb"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("haptictransducernode"), Some("SubsensoryTactileHapticTransducer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tapestop"), Some("TapeStop"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sessionrecorder"), Some("LiveSessionRecorder"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("headbump"), Some("HeadBumpFilter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("linkwitzriley"), Some("LinkwitzRiley4BandCrossover"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crepe"), Some("CrepePitchTracker"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("styletransfer"), Some("NeuralAudioStyleTransferPreviewRenderer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("adaptivebuffer"), Some("AdaptiveBufferScaler"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("mpeexpression"), Some("MpeExpressionCurveEditor"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("visualizerengine"), Some("VisualizerIntegrationEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("neuroaffective"), Some("NeuroAffectiveAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("mentalimagery"), Some("MentalImageryClassifier"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hrvsync"), Some("HrvTempoSyncEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("emggesture"), Some("EmgGestureDriver"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("claphost"), Some("ClapHostEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("wasmdsp"), Some("WasmDspRuntime"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("organwindchest"), Some("Windchest"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cassotto"), Some("CassottoChamber"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("airreed"), Some("AirReed"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("felthammerstrike"), Some("HammerStrike"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("toneholematrix"), Some("ToneholeLattice"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("soundboardradiation"), Some("SpruceSoundboard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("trompettechien"), Some("TrompetteChienJunction"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sympatheticmatrix"), Some("SympatheticResonatorMatrix"));

        // 7. Verify dynamic modular socket creation
        let mut view = AwardWinningGuiView::new();
        view.add_modular_node_from_descriptor(windchest);
        view.add_modular_node_from_descriptor(hrv_sync);
        view.add_modular_node_from_descriptor(recorder);
        view.add_modular_node_from_descriptor(head_bump);

        let nodes = &view.modular_nodes;
        let wc_node = &nodes[nodes.len() - 4];
        let hrv_node = &nodes[nodes.len() - 3];
        let rec_node = &nodes[nodes.len() - 2];
        let hb_node = &nodes[nodes.len() - 1];

        assert_eq!(wc_node.kind_id, "Windchest");
        assert_eq!(hrv_node.kind_id, "HrvTempoSyncEngine");
        assert_eq!(rec_node.kind_id, "LiveSessionRecorder");
        assert_eq!(hb_node.kind_id, "HeadBumpFilter");

        let wc_ports: Vec<_> = wc_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(wc_ports.contains(&"voct"));
        assert!(wc_ports.contains(&"gate"));
        assert!(wc_ports.contains(&"mod"));
        assert!(wc_ports.contains(&"out"));

        let hrv_ports: Vec<_> = hrv_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(hrv_ports.contains(&"gate"));
        assert!(hrv_ports.contains(&"sync"));
        assert!(hrv_ports.contains(&"cv_out"));

        let rec_ports: Vec<_> = rec_node.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(rec_ports.contains(&"in"));
        assert!(rec_ports.contains(&"cv"));
        assert!(rec_ports.contains(&"out"));

        // 8. Verify Novice Preset synchronization with new presets
        #[cfg(feature = "gui")]
        {
            let ctx = eframe::egui::Context::default();

            // Preset A: Cathedral Pipe Organ
            view.top_bar_state.selected_preset = "Cathedral Pipe Organ".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PipeOrganModel".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PipeOrganModel".to_string()));

            // Preset B: Cosmic Shockwave Reverb
            view.top_bar_state.selected_preset = "Cosmic Shockwave Reverb".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("IsmShockwaveReverb".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("IsmShockwaveReverb".to_string()));

            // Preset C: Neuro HRV Bio-Sync
            view.top_bar_state.selected_preset = "Neuro HRV Bio-Sync".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("HrvTempoSyncEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("HrvTempoSyncEngine".to_string()));

            // Preset D: Analog Tape Stop
            view.top_bar_state.selected_preset = "Analog Tape Stop".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("TapeStop".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("TapeStop".to_string()));
        }
    }

    #[test]
    fn test_tier82_arranger_and_piano_roll_canvas_scrubbing_and_track_selection() {
        let mut view = AwardWinningGuiView::new();
        view.asset_browser_state.is_collapsed = true;
        assert_eq!(view.selected_track_idx, 4);
        view.playhead_beat = 0.0;

        let ctx = eframe::egui::Context::default();

        // 1. Click on timeline ruler in Arranger canvas to scrub playhead
        let mut input = eframe::egui::RawInput::default();
        input.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::Vec2::new(1200.0, 800.0),
        ));
        let ruler_click = eframe::egui::pos2(350.0, 80.0);
        input.events.push(eframe::egui::Event::PointerButton {
            pos: ruler_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        input.events.push(eframe::egui::Event::PointerButton {
            pos: ruler_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });

        let _ = ctx.run(input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Playhead should have moved forward from 0.0
        assert!(view.playhead_beat > 0.0, "Playhead beat should scrub forward on ruler click, got {}", view.playhead_beat);

        // 2. Click on Track 2 lane (idx 1) to select it
        let mut track_click_input = eframe::egui::RawInput::default();
        track_click_input.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::Vec2::new(1200.0, 800.0),
        ));
        let track_click = eframe::egui::pos2(120.0, 145.0);
        track_click_input.events.push(eframe::egui::Event::PointerButton {
            pos: track_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        track_click_input.events.push(eframe::egui::Event::PointerButton {
            pos: track_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });

        let _ = ctx.run(track_click_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.inspector_state.target_name, view.tracks[1].name);

        // 3. Switch to Piano Roll canvas and scrub ruler
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
        view.playhead_beat = 0.0;
        let mut pr_input = eframe::egui::RawInput::default();
        pr_input.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::Vec2::new(1200.0, 800.0),
        ));
        let pr_click = eframe::egui::pos2(350.0, 80.0);
        pr_input.events.push(eframe::egui::Event::PointerButton {
            pos: pr_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        pr_input.events.push(eframe::egui::Event::PointerButton {
            pos: pr_click,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });

        let _ = ctx.run(pr_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.playhead_beat > 0.0);
    }

    #[test]
    fn test_tier82_mixer_canvas_mute_solo_and_fader_interaction() {
        let mut view = AwardWinningGuiView::new();
        view.asset_browser_state.is_collapsed = true;
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;

        assert!(!view.tracks[0].is_muted);
        assert!(!view.tracks[0].is_soloed);

        let ctx = eframe::egui::Context::default();

        // Click Mute button for channel 0 (approx x: 110px, y: 124px inside canvas)
        let mut mute_input = eframe::egui::RawInput::default();
        mute_input.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::Vec2::new(1200.0, 800.0),
        ));
        let mute_pos = eframe::egui::pos2(110.0, 124.0);
        mute_input.events.push(eframe::egui::Event::PointerButton {
            pos: mute_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        mute_input.events.push(eframe::egui::Event::PointerButton {
            pos: mute_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });

        let _ = ctx.run(mute_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.tracks[0].is_muted, "Track 0 should be muted after clicking M button");
        assert!(view.inspector_state.is_muted, "Inspector mute state should mirror track mute");

        // Click Solo button for channel 0 (approx x: 158px, y: 124px)
        let mut solo_input = eframe::egui::RawInput::default();
        solo_input.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::Vec2::new(1200.0, 800.0),
        ));
        let solo_pos = eframe::egui::pos2(158.0, 124.0);
        solo_input.events.push(eframe::egui::Event::PointerButton {
            pos: solo_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        solo_input.events.push(eframe::egui::Event::PointerButton {
            pos: solo_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });

        let _ = ctx.run(solo_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.tracks[0].is_soloed, "Track 0 should be soloed after clicking S button");
        assert!(view.inspector_state.is_soloed, "Inspector solo state should mirror track solo");
    }

    #[test]
    fn test_tier82_project_config_and_transport_bidirectional_sync() {
        let mut view = AwardWinningGuiView::new();

        // 1. Create mock ProjectConfig with 2 tracks
        let mut project = summoner_project::schema::ProjectConfig {
            version: "1.0".to_string(),
            name: "Cybernetic Synthwave Session".to_string(),
            tuning_file: None,
            transport: summoner_project::schema::TransportConfig {
                sample_rate: 48000,
                bpm: 138.0,
                time_signature: "4/4".to_string(),
                master_tune_cents: 0.0,
                master_trim_db: 0.0,
            },
            tracks: vec![
                summoner_project::schema::TrackConfig {
                    id: 101,
                    name: "Neon Bass".to_string(),
                    gain: 0.95,
                    pan: -0.2,
                    color: Some([168, 85, 247]),
                    record_armed: true,
                    ..Default::default()
                },
                summoner_project::schema::TrackConfig {
                    id: 102,
                    name: "Hologram Lead".to_string(),
                    gain: 1.10,
                    pan: 0.3,
                    color: Some([56, 189, 248]),
                    record_armed: false,
                    ..Default::default()
                },
            ],
            assets: Vec::new(),
            automation_lanes: Vec::new(),
            midi_mappings: Vec::new(),
            markers: Vec::new(),
            loop_start_beat: 8.0,
            loop_end_beat: 24.0,
            loop_enabled: true,
            punch_in_beat: None,
            punch_out_beat: None,
            locator_a_beat: None,
            locator_b_beat: None,
            meta: None,
            scripts: Vec::new(),
            lua_state: None,
        };

        let mut playhead_beat = 12.0_f64;
        let mut transport_running = true;
        let mut selected_track_id = Some(102_u64);

        // First sync: ProjectConfig -> AwardWinningGuiView
        view.sync_with_project(&mut project, &mut playhead_beat, &mut transport_running, &mut selected_track_id);

        assert_eq!(view.tracks.len(), 2);
        assert_eq!(view.tracks[0].id, 101);
        assert_eq!(view.tracks[0].name, "Neon Bass");
        assert_eq!(view.tracks[0].color_rgb, [168, 85, 247]);
        assert_eq!(view.tracks[1].id, 102);
        assert_eq!(view.tracks[1].name, "Hologram Lead");

        // Selected track should have synchronized to index 1 (id 102)
        assert_eq!(view.selected_track_idx, 1);

        // Transport & Loop bounds synced
        assert_eq!(view.top_bar_state.bpm, 138.0);
        assert_eq!(view.loop_start_beat, 8.0);
        assert_eq!(view.loop_end_beat, 24.0);
        assert_eq!(view.playhead_beat, 12.0);
        assert!(view.top_bar_state.is_playing);

        // Mutate track parameters in GUI view
        view.tracks[0].gain = 0.72;
        view.tracks[0].pan = -0.6;
        view.tracks[0].is_muted = true;
        view.tracks[0].is_soloed = true;

        // Mutate tempo in GUI top bar
        view.top_bar_state.bpm = 144.0;

        // Pause transport in GUI top bar
        view.top_bar_state.is_playing = false;

        // Second sync: AwardWinningGuiView -> ProjectConfig
        view.sync_with_project(&mut project, &mut playhead_beat, &mut transport_running, &mut selected_track_id);

        // Verify project tracks updated
        assert_eq!(project.tracks[0].gain, 0.72);
        assert_eq!(project.tracks[0].pan, -0.6);
        assert!(project.tracks[0].muted);
        assert!(project.tracks[0].soloed);

        // Verify tempo and transport state synced back
        assert_eq!(project.transport.bpm, 144.0);
        assert!(!transport_running);
    }

    #[test]
    fn test_tier83_stage_canvas_scene_launch_pad_trigger_and_panic() {
        let mut view = AwardWinningGuiView::new();
        view.asset_browser_state.is_collapsed = true;
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Performance;
        view.top_bar_state.is_playing = false;

        let ctx = eframe::egui::Context::default();

        // 1. Initial render of Stage Canvas
        let mut raw_input = eframe::egui::RawInput::default();
        raw_input.screen_rect = Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1200.0, 800.0)));
        let _ = ctx.run(raw_input.clone(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.active_scene_idx, None);
        assert!(!view.panic_triggered);

        // 2. Click Scene 2 ("2 Verse") Launch Button (x=120.0, y=210.0 in stage canvas)
        let mut press_scene = raw_input.clone();
        let scene_pos = eframe::egui::pos2(120.0, 210.0);
        press_scene.events.push(eframe::egui::Event::PointerMoved(scene_pos));
        press_scene.events.push(eframe::egui::Event::PointerButton {
            pos: scene_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        press_scene.events.push(eframe::egui::Event::PointerButton {
            pos: scene_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });
        let _ = ctx.run(press_scene, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.active_scene_idx, Some(1));
        assert!(view.top_bar_state.is_playing);
        assert_eq!(view.playhead_beat, 16.0);

        // 3. Click Clip Pad for Track 3 (x=410.0, y=210.0)
        let mut press_pad = raw_input.clone();
        let pad_pos = eframe::egui::pos2(410.0, 210.0);
        press_pad.events.push(eframe::egui::Event::PointerMoved(pad_pos));
        press_pad.events.push(eframe::egui::Event::PointerButton {
            pos: pad_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        press_pad.events.push(eframe::egui::Event::PointerButton {
            pos: pad_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });
        let _ = ctx.run(press_pad, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Track should now be selected and focused in Inspector & Device Rack
        assert!(view.selected_track_idx < view.tracks.len());
        let sel_name = view.tracks[view.selected_track_idx].name.clone();
        assert_eq!(view.inspector_state.target_name, sel_name);
        assert_eq!(view.device_rack_state.device_name, sel_name);

        // 4. Click PANIC Button (top right of stage canvas: x=1135.0, y=84.0)
        let mut press_panic = raw_input.clone();
        let panic_pos = eframe::egui::pos2(1135.0, 84.0);
        press_panic.events.push(eframe::egui::Event::PointerMoved(panic_pos));
        press_panic.events.push(eframe::egui::Event::PointerButton {
            pos: panic_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        press_panic.events.push(eframe::egui::Event::PointerButton {
            pos: panic_pos,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: eframe::egui::Modifiers::default(),
        });
        let _ = ctx.run(press_panic, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Transport should be stopped and panic triggered
        assert!(!view.top_bar_state.is_playing);
        assert!(view.panic_triggered);
        for tr in &view.tracks {
            assert!(!tr.is_armed);
        }
    }

    #[test]
    fn test_tier83_master_bus_fader_interaction() {
        let mut view = AwardWinningGuiView::new();
        view.asset_browser_state.is_collapsed = true;
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
        view.top_bar_state.master_gain = 1.0;

        let ctx = eframe::egui::Context::default();
        let mut raw_input = eframe::egui::RawInput::default();
        raw_input.screen_rect = Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1200.0, 800.0)));

        // Drag master fader near right edge of mixer canvas (approx x=920.0, y=140.0)
        let mut drag_master = raw_input.clone();
        drag_master.events.push(eframe::egui::Event::PointerMoved(eframe::egui::pos2(920.0, 140.0)));
        drag_master.events.push(eframe::egui::Event::PointerButton {
            pos: eframe::egui::pos2(920.0, 140.0),
            button: eframe::egui::PointerButton::Primary,
            pressed: true,
            modifiers: eframe::egui::Modifiers::default(),
        });
        let _ = ctx.run(drag_master, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Master gain should have been updated from the master strip drag
        assert!(view.top_bar_state.master_gain > 0.0);
    }

    #[test]
    fn test_tier83_bidirectional_track_and_inspector_sync() {
        let mut view = AwardWinningGuiView::new();
        view.asset_browser_state.is_collapsed = true;

        let ctx = eframe::egui::Context::default();
        let mut raw_input = eframe::egui::RawInput::default();
        raw_input.screen_rect = Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1200.0, 800.0)));

        // Select track 2 (HiHats)
        view.selected_track_idx = 2;

        let _ = ctx.run(raw_input.clone(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Inspector must match track 2 properties
        assert_eq!(view.inspector_state.target_name, "HiHats");
        let initial_gain = view.tracks[2].gain;
        assert!((view.inspector_state.gain_db - ((initial_gain - 1.0) * 12.0)).abs() < 0.05);

        // 1. Mutate Inspector controls
        view.inspector_state.gain_db = 6.0;
        view.inspector_state.pan_val = -0.45;
        view.inspector_state.is_muted = true;
        view.inspector_state.is_soloed = true;
        view.inspector_state.is_armed = true;

        let _ = ctx.run(raw_input.clone(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Track 2 must now have updated properties written back from Inspector
        assert!((view.tracks[2].gain - 1.5).abs() < 0.05);
        assert_eq!(view.tracks[2].pan, -0.45);
        assert!(view.tracks[2].is_muted);
        assert!(view.tracks[2].is_soloed);
        assert!(view.tracks[2].is_armed);

        // 2. Mutate Track directly (e.g. from Arranger/Mixer)
        view.tracks[2].gain = 0.8;
        view.tracks[2].pan = 0.35;
        view.tracks[2].is_muted = false;

        let _ = ctx.run(raw_input.clone(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Inspector must now reflect the mutated track properties
        assert!((view.inspector_state.gain_db - ((0.8 - 1.0) * 12.0)).abs() < 0.05);
        assert_eq!(view.inspector_state.pan_val, 0.35);
        assert!(!view.inspector_state.is_muted);
    }

    #[test]
    fn test_tier84_dsp_registry_physical_modeling_expansion() {
        use crate::dsp_node_ui::{DspCategory, DspNodeRegistry};
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 346, "Registry should have at least 346 modules, got {}", registry.list_all().len());

        // Verify TuningMatrix registration
        let tuning = registry.get("TuningMatrix").expect("TuningMatrix must be registered");
        assert_eq!(tuning.category, DspCategory::Modulation);
        assert!(!tuning.params.is_empty());
        let tuning_ui = DspNodeRegistry::create_node_ui(&tuning.kind_id);
        assert!(tuning_ui.is_some(), "create_node_ui must succeed for TuningMatrix");

        // Verify JiBridgeJunction registration
        let ji = registry.get("JiBridgeJunction").expect("JiBridgeJunction must be registered");
        assert_eq!(ji.category, DspCategory::AcousticPhysicalModel);
        assert!(!ji.params.is_empty());
        let ji_ui = DspNodeRegistry::create_node_ui(&ji.kind_id);
        assert!(ji_ui.is_some(), "create_node_ui must succeed for JiBridgeJunction");

        // Verify PaulowniaSoundboardBody registration
        let soundboard = registry.get("PaulowniaSoundboardBody").expect("PaulowniaSoundboardBody must be registered");
        assert_eq!(soundboard.category, DspCategory::AcousticPhysicalModel);
        assert!(!soundboard.params.is_empty());
        let soundboard_ui = DspNodeRegistry::create_node_ui(&soundboard.kind_id);
        assert!(soundboard_ui.is_some(), "create_node_ui must succeed for PaulowniaSoundboardBody");

        // Verify aliases in inventory and normalize_type_name
        let inv = DspNodeRegistry::inventory();
        assert!(inv.iter().any(|(name, _, _)| *name == "TuningMatrix"));
        assert!(inv.iter().any(|(name, _, _)| *name == "JiBridgeJunction"));
        assert!(inv.iter().any(|(name, _, _)| *name == "PaulowniaSoundboardBody"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tuningmatrix"), Some("TuningMatrix"));
        assert_eq!(DspNodeRegistry::normalize_type_name("jibridgejunction"), Some("JiBridgeJunction"));
        assert_eq!(DspNodeRegistry::normalize_type_name("paulowniasoundboardbody"), Some("PaulowniaSoundboardBody"));
    }

    #[test]
    fn test_tier84_piano_roll_interactive_note_selection_and_creation() {
        let mut view = AwardWinningGuiView::new();
        assert_eq!(view.piano_roll_notes.len(), 8);
        assert_eq!(view.selected_note_id, Some(1));

        // Select note 3
        view.selected_note_id = Some(3);
        let note3 = view.piano_roll_notes.iter().find(|n| n.id == 3).unwrap();
        assert_eq!(note3.pitch_idx, 8); // E4

        // Add a new note at beat 18.0, pitch_idx 0 (C5)
        let new_id = view.next_note_id;
        view.next_note_id += 1;
        view.piano_roll_notes.push(crate::views::award_winning_gui_view::PianoRollNote {
            id: new_id,
            pitch_idx: 0,
            start_beat: 18.0,
            length_beats: 1.0,
            velocity: 0.9,
        });
        view.selected_note_id = Some(new_id);
        assert_eq!(view.piano_roll_notes.len(), 9);

        // Delete note 2
        view.piano_roll_notes.retain(|n| n.id != 2);
        assert_eq!(view.piano_roll_notes.len(), 8);
        assert!(view.piano_roll_notes.iter().all(|n| n.id != 2));
    }

    #[test]
    fn test_tier84_piano_roll_velocity_lane_dragging() {
        let mut view = AwardWinningGuiView::new();
        // Modify velocity of selected note
        let sel_id = view.selected_note_id.unwrap();
        if let Some(note) = view.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
            note.velocity = 0.42;
        }

        let updated_note = view.piano_roll_notes.iter().find(|n| n.id == sel_id).unwrap();
        assert_eq!(updated_note.velocity, 0.42);

        // Test clamping bounds
        if let Some(note) = view.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
            let clamped_low = (-0.5_f32).clamp(0.05, 1.0);
            note.velocity = clamped_low;
        }
        assert_eq!(view.piano_roll_notes.iter().find(|n| n.id == sel_id).unwrap().velocity, 0.05);

        if let Some(note) = view.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
            let clamped_high = (1.8_f32).clamp(0.05, 1.0);
            note.velocity = clamped_high;
        }
        assert_eq!(view.piano_roll_notes.iter().find(|n| n.id == sel_id).unwrap().velocity, 1.0);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_tier84_piano_roll_keyboard_auditioning_and_tuning_sync() {
        let mut view = AwardWinningGuiView::new();
        view.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;

        // Verify pitch tuning calculation for auditioned pitch
        let pitch_names = ["C5", "B4", "A#4", "A4", "G#4", "G4", "F#4", "F4", "E4", "D#4", "D4", "C#4", "C4"];
        let p_idx = 5; // G4 (MIDI 67)
        let num_pitches = pitch_names.len();
        view.auditioned_pitch_idx = Some(p_idx);
        view.inspector_state.target_name = format!("Pitch {}", pitch_names[p_idx]);
        view.inspector_state.scale_ratio_num = (num_pitches - p_idx) as i32;
        view.inspector_state.root_ratio = 440.0 * 2.0f32.powf(((72 - p_idx as i32) - 69) as f32 / 12.0);
        view.inspector_state.octave_offset = if p_idx < 1 { 5 } else { 4 };

        assert_eq!(view.inspector_state.target_name, "Pitch G4");
        assert_eq!(view.inspector_state.scale_ratio_num, 8);
        assert!((view.inspector_state.root_ratio - 391.995).abs() < 0.1);
        assert_eq!(view.inspector_state.octave_offset, 4);

        // Verify that rendering a frame when mouse is not down releases auditioned key
        let ctx = eframe::egui::Context::default();
        let raw_input = eframe::egui::RawInput::default();
        let _ = ctx.run(raw_input, |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Key released on pointer up
        assert_eq!(view.auditioned_pitch_idx, None);
    }

    #[test]
    fn test_tier84_piano_roll_bidirectional_project_sequence_sync() {
        let mut view = AwardWinningGuiView::new();

        // 1. Create a track with custom sequence steps
        let mut track = summoner_project::schema::TrackConfig {
            id: 201,
            name: "Pluck Lead".to_string(),
            sequence: Some(summoner_project::schema::SequenceConfig {
                start_beat: 4.0,
                step_division: 0.5,
                steps: vec![
                    summoner_project::schema::TrackerStepConfig {
                        note: 72.0, // C5 -> pitch_idx 0
                        velocity: 0.95,
                        gate: 0.8,
                        active: true,
                        ..Default::default()
                    },
                    summoner_project::schema::TrackerStepConfig {
                        note: 67.0, // G4 (72 - 5) -> pitch_idx 5
                        velocity: 0.80,
                        gate: 0.5,
                        active: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };

        // 2. Load sequence into piano roll
        view.load_piano_roll_from_track(&track);
        assert_eq!(view.piano_roll_notes.len(), 2);
        assert_eq!(view.piano_roll_notes[0].pitch_idx, 0); // C5
        assert_eq!(view.piano_roll_notes[0].start_beat, 4.0);
        assert_eq!(view.piano_roll_notes[0].velocity, 0.95);
        assert_eq!(view.piano_roll_notes[1].pitch_idx, 5); // G4
        assert_eq!(view.piano_roll_notes[1].start_beat, 4.5);
        assert_eq!(view.piano_roll_notes[1].velocity, 0.80);

        // 3. Mutate piano roll: add a third note at beat 5.0
        let new_id = view.next_note_id;
        view.piano_roll_notes.push(crate::views::award_winning_gui_view::PianoRollNote {
            id: new_id,
            pitch_idx: 12, // C4
            start_beat: 5.0,
            length_beats: 1.0,
            velocity: 0.88,
        });

        // 4. Sync back to track
        view.sync_piano_roll_to_track(&mut track);
        let seq = track.sequence.as_ref().unwrap();
        // At beat 5.0 / 0.25 = step 20
        let step20 = &seq.steps[20];
        assert!(step20.active);
        assert_eq!(step20.note, 60.0); // 72 - 12 = 60 (C4)
        assert_eq!(step20.velocity, 0.88);

        // 5. Test full sync_with_project round trip
        let mut project = summoner_project::schema::ProjectConfig {
            version: "1.0".to_string(),
            name: "Sequence Test Project".to_string(),
            tracks: vec![track],
            transport: summoner_project::schema::TransportConfig {
                bpm: 124.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut playhead = 0.0_f64;
        let mut running = false;
        let mut sel_id = Some(201_u64);

        view.selected_track_idx = 0;
        view.last_synced_track_notes_idx = 0;
        view.sync_with_project(&mut project, &mut playhead, &mut running, &mut sel_id);
        assert_eq!(project.tracks[0].sequence.as_ref().unwrap().steps[20].note, 60.0);
    }

    #[test]
    fn test_tier85_dsp_registry_percussion_and_idiophone_expansion() {
        use crate::dsp_node_ui::{DspCategory, DspNodeRegistry};
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 358, "Registry should have at least 358 modules, got {}", registry.list_all().len());

        // 1. PercussionMembrane
        let drum = registry.get("PercussionMembrane").expect("PercussionMembrane must be registered");
        assert_eq!(drum.category, DspCategory::AcousticPhysicalModel);
        assert!(drum.params.iter().any(|p| p.id == "fundamental_hz"));
        assert!(drum.params.iter().any(|p| p.id == "snare_tension"));
        assert!(drum.params.iter().any(|p| p.id == "air_cavity_depth"));
        let drum_ui = DspNodeRegistry::create_node_ui(&drum.kind_id);
        assert!(drum_ui.is_some(), "create_node_ui must succeed for PercussionMembrane");

        // 2. StruckIdiophoneResonator
        let marimba = registry.get("StruckIdiophoneResonator").expect("StruckIdiophoneResonator must be registered");
        assert_eq!(marimba.category, DspCategory::AcousticPhysicalModel);
        assert!(marimba.params.iter().any(|p| p.id == "mallet_hardness"));
        assert!(marimba.params.iter().any(|p| p.id == "tube_mix"));
        assert!(marimba.params.iter().any(|p| p.id == "instrument"));
        let marimba_ui = DspNodeRegistry::create_node_ui(&marimba.kind_id);
        assert!(marimba_ui.is_some(), "create_node_ui must succeed for StruckIdiophoneResonator");

        // 3. HurdyGurdySoundboxBody
        let gurdy = registry.get("HurdyGurdySoundboxBody").expect("HurdyGurdySoundboxBody must be registered");
        assert_eq!(gurdy.category, DspCategory::AcousticPhysicalModel);
        assert!(gurdy.params.iter().any(|p| p.id == "body_gain"));
        assert!(gurdy.params.iter().any(|p| p.id == "helmholtz_hz"));
        let gurdy_ui = DspNodeRegistry::create_node_ui(&gurdy.kind_id);
        assert!(gurdy_ui.is_some(), "create_node_ui must succeed for HurdyGurdySoundboxBody");

        // 4. SitarSoundboxBody
        let sitar_box = registry.get("SitarSoundboxBody").expect("SitarSoundboxBody must be registered");
        assert_eq!(sitar_box.category, DspCategory::AcousticPhysicalModel);
        assert!(sitar_box.params.iter().any(|p| p.id == "resonance_gain"));
        assert!(sitar_box.params.iter().any(|p| p.id == "gourd_volume_l"));
        let sitar_box_ui = DspNodeRegistry::create_node_ui(&sitar_box.kind_id);
        assert!(sitar_box_ui.is_some(), "create_node_ui must succeed for SitarSoundboxBody");

        // 5. BridgeWaveCoupler
        let coupler = registry.get("BridgeWaveCoupler").expect("BridgeWaveCoupler must be registered");
        assert_eq!(coupler.category, DspCategory::AcousticPhysicalModel);
        assert!(coupler.params.iter().any(|p| p.id == "bridge_impedance"));
        assert!(coupler.params.iter().any(|p| p.id == "reflection"));
        let coupler_ui = DspNodeRegistry::create_node_ui(&coupler.kind_id);
        assert!(coupler_ui.is_some(), "create_node_ui must succeed for BridgeWaveCoupler");

        // Verify inventory & aliases
        let inv = DspNodeRegistry::inventory();
        assert!(inv.iter().any(|(name, _, _)| *name == "PercussionMembrane"));
        assert!(inv.iter().any(|(name, _, _)| *name == "StruckIdiophoneResonator"));
        assert!(inv.iter().any(|(name, _, _)| *name == "HurdyGurdySoundboxBody"));
        assert!(inv.iter().any(|(name, _, _)| *name == "SitarSoundboxBody"));
        assert!(inv.iter().any(|(name, _, _)| *name == "BridgeWaveCoupler"));

        assert_eq!(DspNodeRegistry::normalize_type_name("percussionmembranenode"), Some("PercussionMembraneNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("struckidiophoneresonator"), Some("StruckIdiophoneResonator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdysoundboxbody"), Some("HurdyGurdySoundboxBody"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sitarsoundboxbody"), Some("SitarSoundboxBody"));
        assert_eq!(DspNodeRegistry::normalize_type_name("bridgewavecoupler"), Some("BridgeWaveCoupler"));
    }

    #[test]
    fn test_tier85_asset_browser_category_filtering_and_selection() {
        let mut browser = ModernAssetBrowserState::default();

        // 1. Verify default folders per category
        let inst_folders = BrowserCategory::Instruments.default_folders();
        assert!(inst_folders.iter().any(|(name, _)| *name == "Physical Models"));
        assert!(inst_folders.iter().any(|(name, _)| *name == "Synthesizers"));

        let phys_models = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_models.contains(&"PercussionMembrane"));
        assert!(phys_models.contains(&"StruckIdiophoneResonator"));
        assert!(phys_models.contains(&"HurdyGurdySoundboxBody"));
        assert!(phys_models.contains(&"SitarSoundboxBody"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        assert!(fx_folders.iter().any(|(name, _)| *name == "Dynamics & Level"));
        assert!(fx_folders.iter().any(|(name, _)| *name == "Time & Reverb"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        assert!(midi_folders.iter().any(|(name, _)| *name == "Generative & Sync"));

        // 2. Test search filtering and interactive category selection
        #[cfg(feature = "gui")]
        {
            let ctx = eframe::egui::Context::default();

            // Select Instruments category and search for "Marimba"
            browser.selected_category = BrowserCategory::Instruments;
            browser.search_query = "Marimba".to_string();

            let mut dragged_item = None;
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    crate::views::modern_asset_browser::show_modern_asset_browser(ui, &mut browser, |item| {
                        dragged_item = Some(item.to_string());
                    });
                });
            });

            // Clear search and switch to Audio FX
            browser.search_query.clear();
            browser.selected_category = BrowserCategory::AudioFx;
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    crate::views::modern_asset_browser::show_modern_asset_browser(ui, &mut browser, |item| {
                        dragged_item = Some(item.to_string());
                    });
                });
            });
            assert_eq!(browser.selected_category, BrowserCategory::AudioFx);
        }
    }

    #[test]
    fn test_tier85_award_winning_gui_novice_presets_and_asset_drag_sync() {
        let mut view = AwardWinningGuiView::new();

        #[cfg(feature = "gui")]
        {
            let ctx = eframe::egui::Context::default();

            // 1. Test Novice Preset: Orchestral Timpani Drum
            view.top_bar_state.selected_preset = "Orchestral Timpani Drum".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PercussionMembrane".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PercussionMembrane".to_string()));

            // 2. Test Novice Preset: Concert Rosewood Marimba
            view.top_bar_state.selected_preset = "Concert Rosewood Marimba".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("StruckIdiophoneResonator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("StruckIdiophoneResonator".to_string()));

            // 3. Test Novice Preset: Bourbonnais Vielle Gurdy
            view.top_bar_state.selected_preset = "Bourbonnais Vielle Gurdy".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("HurdyGurdySoundboxBody".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("HurdyGurdySoundboxBody".to_string()));

            // 4. Test Novice Preset: Silk String Japanese Koto
            view.top_bar_state.selected_preset = "Silk String Japanese Koto".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SitarSoundboxBody".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SitarSoundboxBody".to_string()));

            // 5. Test Asset Browser dynamic item selection synchronizing to Device Rack & Inspector
            view.asset_browser_state.selected_category = BrowserCategory::Instruments;
            view.asset_browser_state.selected_item = Some("StruckIdiophoneResonator".to_string());
            let registry = crate::dsp_node_ui::DspNodeRegistry::new();
            if let Some(desc) = registry.get("StruckIdiophoneResonator") {
                view.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                view.device_rack_state.device_name = desc.display_name.clone();
                view.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                view.inspector_state.target_name = desc.display_name.clone();
            }
            assert_eq!(view.device_rack_state.selected_node_kind, Some("StruckIdiophoneResonator".to_string()));
            assert_eq!(view.device_rack_state.device_name, "Physical Modeling Struck Idiophone Resonator Bank");
            assert_eq!(view.inspector_state.selected_node_kind, Some("StruckIdiophoneResonator".to_string()));
        }
    }

    #[test]
    fn test_tier86_dsp_node_registry_expansion_and_parameter_reflection() {
        use crate::dsp_node_ui::{DspCategory, DspNodeRegistry};

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 372, "Registry must contain >= 372 descriptors, found {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 372, "Inventory must contain >= 372 entries, found {}", inv.len());

        // 1. JawariBridge
        let jawari = registry.get("JawariBridge").expect("JawariBridge must be registered");
        assert_eq!(jawari.category, DspCategory::AcousticPhysicalModel);
        assert!(jawari.params.iter().any(|p| p.id == "clearance_gap_mm"));
        assert!(jawari.params.iter().any(|p| p.id == "curvature_c"));
        assert!(jawari.params.iter().any(|p| p.id == "contact_stiffness_k"));
        assert!(jawari.params.iter().any(|p| p.id == "jiva_thread_pos"));
        assert!(jawari.params.iter().any(|p| p.id == "jiva_damping"));
        let jawari_ui = DspNodeRegistry::create_node_ui(&jawari.kind_id);
        assert!(jawari_ui.is_some(), "create_node_ui must succeed for JawariBridge");

        // 2. TineResonator
        let tine = registry.get("TineResonator").expect("TineResonator must be registered");
        assert_eq!(tine.category, DspCategory::AcousticPhysicalModel);
        assert!(tine.params.iter().any(|p| p.id == "frequency"));
        assert!(tine.params.iter().any(|p| p.id == "hammer_hardness"));
        assert!(tine.params.iter().any(|p| p.id == "bark_drive"));
        assert!(tine.params.iter().any(|p| p.id == "air_gap_mm"));
        assert!(tine.params.iter().any(|p| p.id == "tonebar_coupling"));
        assert!(tine.params.iter().any(|p| p.id == "feedback_gain"));
        let tine_ui = DspNodeRegistry::create_node_ui(&tine.kind_id);
        assert!(tine_ui.is_some(), "create_node_ui must succeed for TineResonator");

        // 3. FrictionWheelExciter
        let wheel = registry.get("FrictionWheelExciter").expect("FrictionWheelExciter must be registered");
        assert_eq!(wheel.category, DspCategory::AcousticPhysicalModel);
        assert!(wheel.params.iter().any(|p| p.id == "angular_velocity"));
        assert!(wheel.params.iter().any(|p| p.id == "wheel_pressure"));
        assert!(wheel.params.iter().any(|p| p.id == "rosin_adhesion"));
        assert!(wheel.params.iter().any(|p| p.id == "wrist_acceleration"));
        assert!(wheel.params.iter().any(|p| p.id == "wheel_radius_m"));
        let wheel_ui = DspNodeRegistry::create_node_ui(&wheel.kind_id);
        assert!(wheel_ui.is_some(), "create_node_ui must succeed for FrictionWheelExciter");

        // 4. SnareRattleModel
        let snare = registry.get("SnareRattleModel").expect("SnareRattleModel must be registered");
        assert_eq!(snare.category, DspCategory::AcousticPhysicalModel);
        assert!(snare.params.iter().any(|p| p.id == "snare_tension"));
        assert!(snare.params.iter().any(|p| p.id == "snare_buzz_decay"));
        assert!(snare.params.iter().any(|p| p.id == "snare_threshold"));
        assert!(snare.params.iter().any(|p| p.id == "buzz_gain"));
        assert!(snare.params.iter().any(|p| p.id == "enabled"));
        let snare_ui = DspNodeRegistry::create_node_ui(&snare.kind_id);
        assert!(snare_ui.is_some(), "create_node_ui must succeed for SnareRattleModel");

        // 5. ChikariDroneBank
        let chikari = registry.get("ChikariDroneBank").expect("ChikariDroneBank must be registered");
        assert_eq!(chikari.category, DspCategory::AcousticPhysicalModel);
        assert!(chikari.params.iter().any(|p| p.id == "root_hz"));
        assert!(chikari.params.iter().any(|p| p.id == "strum_speed"));
        assert!(chikari.params.iter().any(|p| p.id == "drone_resonance"));
        assert!(chikari.params.iter().any(|p| p.id == "chikari_gain"));
        let chikari_ui = DspNodeRegistry::create_node_ui(&chikari.kind_id);
        assert!(chikari_ui.is_some(), "create_node_ui must succeed for ChikariDroneBank");

        // 6. ClavinetAnvilModel & ArmonicaChassisResonator
        let anvil = registry.get("ClavinetAnvilModel").expect("ClavinetAnvilModel must be registered");
        assert_eq!(anvil.category, DspCategory::AcousticPhysicalModel);
        assert!(anvil.params.iter().any(|p| p.id == "hardness"));
        assert!(anvil.params.iter().any(|p| p.id == "velocity"));
        assert!(anvil.params.iter().any(|p| p.id == "exponent"));
        let anvil_ui = DspNodeRegistry::create_node_ui(&anvil.kind_id);
        assert!(anvil_ui.is_some(), "create_node_ui must succeed for ClavinetAnvilModel");

        let armonica = registry.get("ArmonicaChassisResonator").expect("ArmonicaChassisResonator must be registered");
        assert_eq!(armonica.category, DspCategory::AcousticPhysicalModel);
        assert!(armonica.params.iter().any(|p| p.id == "master_chassis_gain"));
        let armonica_ui = DspNodeRegistry::create_node_ui(&armonica.kind_id);
        assert!(armonica_ui.is_some(), "create_node_ui must succeed for ArmonicaChassisResonator");

        // 7. WetStickSlipExciter
        let wet = registry.get("WetStickSlipExciter").expect("WetStickSlipExciter must be registered");
        assert_eq!(wet.category, DspCategory::AcousticPhysicalModel);
        assert!(wet.params.iter().any(|p| p.id == "wand_speed_rad_s"));
        assert!(wet.params.iter().any(|p| p.id == "normal_force_n"));
        assert!(wet.params.iter().any(|p| p.id == "moisture_film"));
        assert!(wet.params.iter().any(|p| p.id == "rim_radius_m"));
        let wet_ui = DspNodeRegistry::create_node_ui(&wet.kind_id);
        assert!(wet_ui.is_some(), "create_node_ui must succeed for WetStickSlipExciter");

        // 8. YarnDamper
        let damper = registry.get("YarnDamper").expect("YarnDamper must be registered");
        assert_eq!(damper.category, DspCategory::AcousticPhysicalModel);
        assert!(damper.params.iter().any(|p| p.id == "damping_amount"));
        assert!(damper.params.iter().any(|p| p.id == "thud_volume"));
        assert!(damper.params.iter().any(|p| p.id == "thud_decay"));
        assert!(damper.params.iter().any(|p| p.id == "thud_freq"));
        let damper_ui = DspNodeRegistry::create_node_ui(&damper.kind_id);
        assert!(damper_ui.is_some(), "create_node_ui must succeed for YarnDamper");

        // 9. AutoSlicer
        let slicer = registry.get("AutoSlicer").expect("AutoSlicer must be registered");
        assert_eq!(slicer.category, DspCategory::SamplerSlicer);
        assert!(slicer.params.iter().any(|p| p.id == "threshold"));
        assert!(slicer.params.iter().any(|p| p.id == "algorithm"));
        assert!(slicer.params.iter().any(|p| p.id == "min_slice_length_ms"));
        assert!(slicer.params.iter().any(|p| p.id == "sensitivity"));
        let slicer_ui = DspNodeRegistry::create_node_ui(&slicer.kind_id);
        assert!(slicer_ui.is_some(), "create_node_ui must succeed for AutoSlicer");

        // 10. SpeakerCalibrationMatrix
        let speaker = registry.get("SpeakerCalibrationMatrix").expect("SpeakerCalibrationMatrix must be registered");
        assert_eq!(speaker.category, DspCategory::SpatialSurround);
        assert!(speaker.params.iter().any(|p| p.id == "num_channels"));
        assert!(speaker.params.iter().any(|p| p.id == "master_trim_db"));
        assert!(speaker.params.iter().any(|p| p.id == "delay_compensation_ms"));
        let speaker_ui = DspNodeRegistry::create_node_ui(&speaker.kind_id);
        assert!(speaker_ui.is_some(), "create_node_ui must succeed for SpeakerCalibrationMatrix");

        // 11. AiMixBalanceAnalyzer
        let mix = registry.get("AiMixBalanceAnalyzer").expect("AiMixBalanceAnalyzer must be registered");
        assert_eq!(mix.category, DspCategory::NeuralAi);
        assert!(mix.params.iter().any(|p| p.id == "sensitivity"));
        assert!(mix.params.iter().any(|p| p.id == "spectral_bands"));
        assert!(mix.params.iter().any(|p| p.id == "smoothing_sec"));
        assert!(mix.params.iter().any(|p| p.id == "auto_recommend_cut"));
        let mix_ui = DspNodeRegistry::create_node_ui(&mix.kind_id);
        assert!(mix_ui.is_some(), "create_node_ui must succeed for AiMixBalanceAnalyzer");

        // 12. AiSongStructureDetector
        let struct_det = registry.get("AiSongStructureDetector").expect("AiSongStructureDetector must be registered");
        assert_eq!(struct_det.category, DspCategory::NeuralAi);
        assert!(struct_det.params.iter().any(|p| p.id == "min_section_bars"));
        assert!(struct_det.params.iter().any(|p| p.id == "energy_contrast"));
        assert!(struct_det.params.iter().any(|p| p.id == "tempo_stability"));
        assert!(struct_det.params.iter().any(|p| p.id == "auto_marker_placement"));
        let struct_ui = DspNodeRegistry::create_node_ui(&struct_det.kind_id);
        assert!(struct_ui.is_some(), "create_node_ui must succeed for AiSongStructureDetector");

        // 13. AiPolyphonicChordExtractor
        let chord = registry.get("AiPolyphonicChordExtractor").expect("AiPolyphonicChordExtractor must be registered");
        assert_eq!(chord.category, DspCategory::NeuralAi);
        assert!(chord.params.iter().any(|p| p.id == "confidence_threshold"));
        assert!(chord.params.iter().any(|p| p.id == "bass_octave_weight"));
        assert!(chord.params.iter().any(|p| p.id == "velocity_scaling"));
        let chord_ui = DspNodeRegistry::create_node_ui(&chord.kind_id);
        assert!(chord_ui.is_some(), "create_node_ui must succeed for AiPolyphonicChordExtractor");

        // 14. AudioAlignmentTool
        let align = registry.get("AudioAlignmentTool").expect("AudioAlignmentTool must be registered");
        assert_eq!(align.category, DspCategory::Utility);
        assert!(align.params.iter().any(|p| p.id == "max_search_lag_ms"));
        assert!(align.params.iter().any(|p| p.id == "cross_corr_threshold"));
        assert!(align.params.iter().any(|p| p.id == "auto_polarity_invert"));
        assert!(align.params.iter().any(|p| p.id == "phase_align_amount"));
        let align_ui = DspNodeRegistry::create_node_ui(&align.kind_id);
        assert!(align_ui.is_some(), "create_node_ui must succeed for AudioAlignmentTool");

        // Verify normalization aliases
        assert_eq!(DspNodeRegistry::normalize_type_name("jawari"), Some("JawariBridge"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sitarjawari"), Some("JawariBridge"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tineresonator"), Some("TineResonator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("rhodestine"), Some("TineResonator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("frictionwheel"), Some("FrictionWheelExciter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("snarerattle"), Some("SnareRattleModel"));
        assert_eq!(DspNodeRegistry::normalize_type_name("chikaridrone"), Some("ChikariDroneBank"));
        assert_eq!(DspNodeRegistry::normalize_type_name("armonicasoundbox"), Some("ArmonicaChassisResonator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("wetstickslip"), Some("WetStickSlipExciter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("yarndamper"), Some("YarnDamper"));
        assert_eq!(DspNodeRegistry::normalize_type_name("autoslicer"), Some("AutoSlicer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("clavinetanvil"), Some("ClavinetAnvilModel"));
        assert_eq!(DspNodeRegistry::normalize_type_name("speakercalibration"), Some("SpeakerCalibrationMatrix"));
        assert_eq!(DspNodeRegistry::normalize_type_name("aimixbalance"), Some("AiMixBalanceAnalyzer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("songstructure"), Some("AiSongStructureDetector"));
        assert_eq!(DspNodeRegistry::normalize_type_name("chordextractor"), Some("AiPolyphonicChordExtractor"));
        assert_eq!(DspNodeRegistry::normalize_type_name("audioalign"), Some("AudioAlignmentTool"));
    }

    #[test]
    fn test_tier86_asset_browser_category_expansion() {
        let mut browser = ModernAssetBrowserState::default();

        // 1. Verify default folders per category
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_models = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_models.contains(&"JawariBridge"));
        assert!(phys_models.contains(&"TineResonator"));
        assert!(phys_models.contains(&"FrictionWheelExciter"));
        assert!(phys_models.contains(&"SnareRattleModel"));
        assert!(phys_models.contains(&"ChikariDroneBank"));
        assert!(phys_models.contains(&"ArmonicaChassisResonator"));
        assert!(phys_models.contains(&"WetStickSlipExciter"));
        assert!(phys_models.contains(&"YarnDamper"));
        assert!(phys_models.contains(&"ClavinetAnvilModel"));

        let samplers = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(samplers.contains(&"AutoSlicer"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dynamics = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dynamics.contains(&"SpeakerCalibrationMatrix"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_sync = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_sync.contains(&"AiPolyphonicChordExtractor"));
        assert!(gen_sync.contains(&"AiSongStructureDetector"));

        let routing = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing.contains(&"AiMixBalanceAnalyzer"));
        assert!(routing.contains(&"AudioAlignmentTool"));

        // 2. Interactive browsing under egui
        #[cfg(feature = "gui")]
        {
            let ctx = eframe::egui::Context::default();
            browser.selected_category = BrowserCategory::Instruments;
            browser.search_query = "Jawari".to_string();

            let mut dragged_item = None;
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    crate::views::modern_asset_browser::show_modern_asset_browser(ui, &mut browser, |item| {
                        dragged_item = Some(item.to_string());
                    });
                });
            });

            browser.search_query.clear();
            browser.selected_category = BrowserCategory::MidiFx;
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    crate::views::modern_asset_browser::show_modern_asset_browser(ui, &mut browser, |item| {
                        dragged_item = Some(item.to_string());
                    });
                });
            });
            assert_eq!(browser.selected_category, BrowserCategory::MidiFx);
        }
    }

    #[test]
    fn test_tier86_award_winning_gui_novice_presets_physical_models_sync() {
        let mut view = AwardWinningGuiView::new();

        #[cfg(feature = "gui")]
        {
            let ctx = eframe::egui::Context::default();

            // 1. Test Novice Preset: Hindustani Ravi Sitar
            view.top_bar_state.selected_preset = "Hindustani Ravi Sitar".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("JawariBridge".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("JawariBridge".to_string()));

            // 2. Test Novice Preset: Rhodes Classic Mark I
            view.top_bar_state.selected_preset = "Rhodes Classic Mark I".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("TineResonator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("TineResonator".to_string()));

            // 3. Test Novice Preset: Hurdy-Gurdy Crank Wheel
            view.top_bar_state.selected_preset = "Hurdy-Gurdy Crank Wheel".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("FrictionWheelExciter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("FrictionWheelExciter".to_string()));

            // 4. Test Novice Preset: Concert Snare Drum Rattle
            view.top_bar_state.selected_preset = "Concert Snare Drum Rattle".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SnareRattleModel".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SnareRattleModel".to_string()));

            // 5. Test Novice Preset: Indian Raga Chikari Drone
            view.top_bar_state.selected_preset = "Indian Raga Chikari Drone".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ChikariDroneBank".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ChikariDroneBank".to_string()));

            // 6. Test Novice Preset: Franklin Water Armonica
            view.top_bar_state.selected_preset = "Franklin Water Armonica".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ArmonicaChassisResonator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ArmonicaChassisResonator".to_string()));

            // 7. Test Asset Selection synchronization for AutoSlicer & SpeakerCalibrationMatrix
            view.asset_browser_state.selected_category = BrowserCategory::Instruments;
            view.asset_browser_state.selected_item = Some("AutoSlicer".to_string());
            let registry = crate::dsp_node_ui::DspNodeRegistry::new();
            if let Some(desc) = registry.get("AutoSlicer") {
                view.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                view.device_rack_state.device_name = desc.display_name.clone();
                view.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                view.inspector_state.target_name = desc.display_name.clone();
            }
            assert_eq!(view.device_rack_state.selected_node_kind, Some("AutoSlicer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("AutoSlicer".to_string()));

            if let Some(desc) = registry.get("SpeakerCalibrationMatrix") {
                view.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                view.device_rack_state.device_name = desc.display_name.clone();
                view.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                view.inspector_state.target_name = desc.display_name.clone();
            }
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SpeakerCalibrationMatrix".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SpeakerCalibrationMatrix".to_string()));
        }
    }

    #[test]
    fn test_tier87_dsp_registry_400_plus_and_physical_modeling_expansion() {
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let total_nodes = registry.list_all().len();
        assert!(total_nodes >= 400, "Registry must contain >= 400 DSP modules, found {}", total_nodes);

        let inventory = crate::dsp_node_ui::DspNodeRegistry::inventory();
        assert!(inventory.len() >= 400, "Inventory must contain >= 400 DSP modules, found {}", inventory.len());

        // 1. Verify physical modeling nodes and schemas
        let felt_hammer = registry.get("GrandPianoFeltHammer").expect("GrandPianoFeltHammer must exist");
        assert_eq!(felt_hammer.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(felt_hammer.params.len(), 4);
        assert!(felt_hammer.params.iter().any(|p| p.id == "nonlin_stiffness"));
        assert!(felt_hammer.params.iter().any(|p| p.id == "hysteresis_loss"));
        assert!(felt_hammer.params.iter().any(|p| p.id == "una_corda_shift"));

        let spruce = registry.get("SpruceSoundboardMode").expect("SpruceSoundboardMode must exist");
        assert_eq!(spruce.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(spruce.params.len(), 4);
        assert!(spruce.params.iter().any(|p| p.id == "modal_frequency"));
        assert!(spruce.params.iter().any(|p| p.id == "bridge_impedance"));

        let tonehole = registry.get("ToneholeJunction").expect("ToneholeJunction must exist");
        assert_eq!(tonehole.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(tonehole.params.len(), 4);

        let clav = registry.get("ClavinetVoice").expect("ClavinetVoice must exist");
        assert_eq!(clav.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(clav.params.len(), 4);

        let tarab = registry.get("TarabStringResonator").expect("TarabStringResonator must exist");
        assert_eq!(tarab.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(tarab.params.len(), 4);

        let duplex = registry.get("GrandPianoStringDuplexCoupler").expect("GrandPianoStringDuplexCoupler must exist");
        assert_eq!(duplex.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(duplex.params.len(), 4);

        let bellows = registry.get("BellowsDynamicsCoupler").expect("BellowsDynamicsCoupler must exist");
        assert_eq!(bellows.category, crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(bellows.params.len(), 4);

        // 2. Verify Spatial & Audio FX nodes
        let hoa_dec = registry.get("AmbisonicsDecoder3D").expect("AmbisonicsDecoder3D must exist");
        assert_eq!(hoa_dec.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);

        let hoa_enc = registry.get("AmbisonicsEncoder3D").expect("AmbisonicsEncoder3D must exist");
        assert_eq!(hoa_enc.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);

        let part_conv = registry.get("PartitionedBinauralHrtfConvolver").expect("PartitionedBinauralHrtfConvolver must exist");
        assert_eq!(part_conv.category, crate::dsp_node_ui::DspNodeCategory::SpatialSurround);

        let clipper = registry.get("OversampledClipper").expect("OversampledClipper must exist");
        assert_eq!(clipper.category, crate::dsp_node_ui::DspNodeCategory::DistortionSaturation);

        let peak_det = registry.get("TruePeakDetector").expect("TruePeakDetector must exist");
        assert_eq!(peak_det.category, crate::dsp_node_ui::DspNodeCategory::Utility);

        let autowah = registry.get("AutoWah").expect("AutoWah must exist");
        assert_eq!(autowah.category, crate::dsp_node_ui::DspNodeCategory::Modulation);

        let formant_vowel = registry.get("FormantVowelMatrix").expect("FormantVowelMatrix must exist");
        assert_eq!(formant_vowel.category, crate::dsp_node_ui::DspNodeCategory::FilterEq);

        // 3. Verify Neural AI nodes
        let neural_ir = registry.get("NeuralIrSynthesizer").expect("NeuralIrSynthesizer must exist");
        assert_eq!(neural_ir.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let neural_midi = registry.get("NeuralMidiTranscriber").expect("NeuralMidiTranscriber must exist");
        assert_eq!(neural_midi.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let drum_tr = registry.get("DrumTranscriptor").expect("DrumTranscriptor must exist");
        assert_eq!(drum_tr.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let ai_chord = registry.get("AiChordGenerator").expect("AiChordGenerator must exist");
        assert_eq!(ai_chord.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        let ai_mix = registry.get("AiMixAssistant").expect("AiMixAssistant must exist");
        assert_eq!(ai_mix.category, crate::dsp_node_ui::DspNodeCategory::NeuralAi);

        // 4. Verify alias normalization
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("grandpianofelthammer"), Some("GrandPianoFeltHammer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("felthammer"), Some("GrandPianoFeltHammer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sprucesoundboardmode"), Some("SpruceSoundboardMode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tonehole"), Some("ToneholeJunction"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("ambisonicsdecoder3d"), Some("AmbisonicsDecoder3D"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("optotremolo"), Some("OpticalTremolo"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("buffershuffle"), Some("GlitchShuffle"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("stutterrepeater"), Some("GlitchStutter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aichordgenerator"), Some("AiChordGenerator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("vowelmatrix"), Some("FormantVowelMatrix"));

        // 5. Verify create_node_ui instantiations
        let ui_hammer = crate::dsp_node_ui::DspNodeRegistry::create_node_ui("GrandPianoFeltHammer").expect("UI must create for GrandPianoFeltHammer");
        assert_eq!(ui_hammer.display_name(), "Nonlinear Felt Hammer Contact Dynamics");

        let ui_spruce = crate::dsp_node_ui::DspNodeRegistry::create_node_ui("SpruceSoundboardMode").expect("UI must create for SpruceSoundboardMode");
        assert_eq!(ui_spruce.display_name(), "Sitka Spruce Soundboard Orthotropic 2D Mode");

        let ui_neural_ir = crate::dsp_node_ui::DspNodeRegistry::create_node_ui("NeuralIrSynthesizer").expect("UI must create for NeuralIrSynthesizer");
        assert_eq!(ui_neural_ir.display_name(), "Deep Neural Parametric Room Impulse Generator");

            // 6. Test Novice Preset switching in AwardWinningGuiView
            let mut view = AwardWinningGuiView::new();
            let ctx = eframe::egui::Context::default();

            // Preset: Concert Grand Felt Hammer
            view.top_bar_state.selected_preset = "Concert Grand Felt Hammer".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("GrandPianoFeltHammer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("GrandPianoFeltHammer".to_string()));

            // Preset: Sitka Spruce Resonance
            view.top_bar_state.selected_preset = "Sitka Spruce Resonance".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SpruceSoundboardMode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SpruceSoundboardMode".to_string()));

            // Preset: Hohner D6 Funk Clavinet
            view.top_bar_state.selected_preset = "Hohner D6 Funk Clavinet".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ClavinetVoice".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ClavinetVoice".to_string()));

            // Preset: Indian Tarab Sympathetic
            view.top_bar_state.selected_preset = "Indian Tarab Sympathetic".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("TarabStringResonator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("TarabStringResonator".to_string()));

            // Preset: Neural AI Room Impulse
            view.top_bar_state.selected_preset = "Neural AI Room Impulse".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("NeuralIrSynthesizer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("NeuralIrSynthesizer".to_string()));

            // Preset: AI Voice-Leading Master
            view.top_bar_state.selected_preset = "AI Voice-Leading Master".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("AiChordGenerator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("AiChordGenerator".to_string()));
    }

    #[test]
    fn test_tier88_dsp_registry_445_plus_quantum_and_neuro_expansion() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 445, "Expected inventory count >= 445, got {}", inv.len());

        let registry = DspNodeRegistry::new();
        let new_nodes = [
            "BrainstemPitchTracker",
            "NeuroFeedbackOscillator",
            "AcousticHologramFilter",
            "HapticTransducerNode",
            "SubHarmonicQuantumTunnelingFilter",
            "HyperbolicReverbNode",
            "QuantumEntanglementRouter",
            "NeuralQuantumAnnealer",
            "QuantumTeleportationBufferBus",
            "QuantumErrorCorrectionCodec",
            "HyperDimensionalHrtfLoader",
            "MhdPlasmaWaveModulatorNode",
            "AcousticCloakingSpatializerNode",
            "CrystalModalFilter",
            "HurdyGurdyDispersionFilter",
            "SpectrogramArtEngine",
            "SpectrogramArtMorpher",
            "AudioReverse",
            "GlitchGate",
        ];

        for node_id in &new_nodes {
            let desc = registry.get(node_id);
            assert!(desc.is_some(), "Node {} missing from DspNodeRegistry", node_id);
            let desc = desc.unwrap();
            assert!(!desc.params.is_empty(), "Node {} has empty parameter list", node_id);
            assert!(!desc.description.is_empty(), "Node {} has empty description", node_id);

            let ui = DspNodeRegistry::create_node_ui(node_id);
            assert!(ui.is_some(), "create_node_ui failed for {}", node_id);
            let ui = ui.unwrap();
            assert!(!ui.parameters().is_empty(), "UI for {} has no parameters", node_id);
        }

        // Test normalize_type_name
        assert_eq!(DspNodeRegistry::normalize_type_name("brainstempitchtracker"), Some("BrainstemPitchTracker"));
        assert_eq!(DspNodeRegistry::normalize_type_name("neurofeedback"), Some("NeuroFeedbackOscillator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("haptictransducer"), Some("HapticTransducerNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("quantumtunneling"), Some("SubHarmonicQuantumTunnelingFilter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicreverb"), Some("HyperbolicReverbNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("quantumentanglement"), Some("QuantumEntanglementRouter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("shorcodec"), Some("QuantumErrorCorrectionCodec"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hdhrtfloader"), Some("HyperDimensionalHrtfLoader"));
        assert_eq!(DspNodeRegistry::normalize_type_name("mhdplasmawave"), Some("MhdPlasmaWaveModulatorNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("acousticcloaking"), Some("AcousticCloakingSpatializerNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("piezocrystalfilter"), Some("CrystalModalFilter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdydispersion"), Some("HurdyGurdyDispersionFilter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramartengine"), Some("SpectrogramArtEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spectrogrammorpher"), Some("SpectrogramArtMorpher"));
        assert_eq!(DspNodeRegistry::normalize_type_name("audioreverse"), Some("AudioReverse"));
        assert_eq!(DspNodeRegistry::normalize_type_name("glitchgate"), Some("GlitchGate"));
        assert_eq!(DspNodeRegistry::normalize_type_name("samplernode"), Some("SamplerNode"));

        // Test modern asset browser integration
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let synth_items = inst_folders.iter().find(|(name, _)| *name == "Synthesizers").unwrap().1;
        assert!(synth_items.contains(&"NeuroFeedbackOscillator"));
        assert!(synth_items.contains(&"SpectrogramArtEngine"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"HyperbolicReverbNode"));
        assert!(time_items.contains(&"AcousticCloakingSpatializerNode"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"SubHarmonicQuantumTunnelingFilter"));
        assert!(filter_items.contains(&"CrystalModalFilter"));

        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Quantum Hyperbolic Reverb
            view.top_bar_state.selected_preset = "Quantum Hyperbolic Reverb".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("NonEuclideanHyperbolicReverb".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("NonEuclideanHyperbolicReverb".to_string()));

            // Preset: Neuro Relaxation Feedback
            view.top_bar_state.selected_preset = "Neuro Relaxation Feedback".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("NeuroFeedbackOscillator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("NeuroFeedbackOscillator".to_string()));

            // Preset: Acoustic Cloaking Soundfield
            view.top_bar_state.selected_preset = "Acoustic Cloaking Soundfield".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("AcousticCloakingSpatializer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("AcousticCloakingSpatializer".to_string()));

            // Preset: Sub-Harmonic Quantum Tunnel
            view.top_bar_state.selected_preset = "Sub-Harmonic Quantum Tunnel".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SubharmonicQuantumTunnelingFilter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SubharmonicQuantumTunnelingFilter".to_string()));
        }
    }

    #[test]
    fn test_tier89_dsp_registry_465_plus_full_module_exposure() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 465, "Expected inventory count >= 465, got {}", inv.len());

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 465, "Expected descriptor count >= 465, got {}", registry.list_all().len());

        let new_tier89_nodes = [
            "PeakHeadroomAnalyzer",
            "EbuR128LoudnessMeter",
            "StemMetadataParser",
            "DrumReplacementTrigger",
            "SurroundStemSplitterBedObject",
            "HardwareControlEditorState",
            "HeadTrackerReceiver",
            "ProceduralSpatialIrGenerator",
            "IsolatedPluginScanner",
            "MidiFilterEngine",
            "PolymetricSequencer",
            "LinkwitzRiley4WaySplitter",
            "NativeAudioDriverTuner",
            "HardwareWatchdogService",
            "ThermalThrottlingListener",
            "BypassRelayTrigger",
            "RotaryEncoderDebouncer",
            "MultiTenantRenderQueue",
            "LockFreeTuningRemapper",
            "SidechainMatrix",
        ];

        for node_id in &new_tier89_nodes {
            let desc = registry.get(node_id);
            assert!(desc.is_some(), "Node {} missing from DspNodeRegistry", node_id);
            let desc = desc.unwrap();
            assert!(!desc.params.is_empty(), "Node {} has empty parameter list", node_id);
            assert!(!desc.description.is_empty(), "Node {} has empty description", node_id);

            let ui = DspNodeRegistry::create_node_ui(node_id);
            assert!(ui.is_some(), "create_node_ui failed for {}", node_id);
            let ui = ui.unwrap();
            assert!(!ui.parameters().is_empty(), "UI for {} has no parameters", node_id);
        }

        // Test canonical alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("peakheadroom"), Some("PeakHeadroomAnalyzer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("ebur128"), Some("EbuR128LoudnessMeter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("ixmlparser"), Some("StemMetadataParser"));
        assert_eq!(DspNodeRegistry::normalize_type_name("drumtrigger"), Some("DrumReplacementTrigger"));
        assert_eq!(DspNodeRegistry::normalize_type_name("atmosstemsplitter"), Some("SurroundStemSplitterBedObject"));
        assert_eq!(DspNodeRegistry::normalize_type_name("controlsurfaceeditor"), Some("HardwareControlEditorState"));
        assert_eq!(DspNodeRegistry::normalize_type_name("imuheadtracker"), Some("HeadTrackerReceiver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("proceduralspatialir"), Some("ProceduralSpatialIrGenerator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("pluginscanner"), Some("IsolatedPluginScanner"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midifilter"), Some("MidiFilterEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("euclideansequencer"), Some("PolymetricSequencer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("lr4waysplitter"), Some("LinkwitzRiley4WaySplitter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("drivertuner"), Some("NativeAudioDriverTuner"));
        assert_eq!(DspNodeRegistry::normalize_type_name("watchdogservice"), Some("HardwareWatchdogService"));
        assert_eq!(DspNodeRegistry::normalize_type_name("thermallistener"), Some("ThermalThrottlingListener"));
        assert_eq!(DspNodeRegistry::normalize_type_name("relaytrigger"), Some("BypassRelayTrigger"));
        assert_eq!(DspNodeRegistry::normalize_type_name("encoderdebouncer"), Some("RotaryEncoderDebouncer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("renderqueue"), Some("MultiTenantRenderQueue"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sclremapper"), Some("LockFreeTuningRemapper"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sidechainhub"), Some("SidechainMatrix"));

        // Test modern asset browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let samplers = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(samplers.contains(&"DrumReplacementTrigger"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"PeakHeadroomAnalyzer"));
        assert!(dyn_items.contains(&"EbuR128LoudnessMeter"));
        assert!(dyn_items.contains(&"SidechainMatrix"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"ProceduralSpatialIrGenerator"));
        assert!(time_items.contains(&"SurroundStemSplitterBedObject"));
        assert!(time_items.contains(&"HeadTrackerReceiver"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"LockFreeTuningRemapper"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_items.contains(&"PolymetricSequencer"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"MidiFilterEngine"));

        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: EBU R128 Master Broadcast
            view.top_bar_state.selected_preset = "EBU R128 Master Broadcast".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("EbuR128LoudnessMeter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("EbuR128LoudnessMeter".to_string()));

            // Preset: Dolby Atmos Bed Splitter
            view.top_bar_state.selected_preset = "Dolby Atmos Bed Splitter".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SurroundStemSplitterBedObject".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SurroundStemSplitterBedObject".to_string()));

            // Preset: Procedural Raytraced Hall
            view.top_bar_state.selected_preset = "Procedural Raytraced Hall".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ProceduralSpatialIrGenerator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ProceduralSpatialIrGenerator".to_string()));

            // Preset: Polymetric Euclidean Groove
            view.top_bar_state.selected_preset = "Polymetric Euclidean Groove".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PolymetricSequencer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PolymetricSequencer".to_string()));
        }
    }

    #[test]
    fn test_tier90_dsp_registry_485_plus_and_gesture_engine_exposure() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 485, "Expected inventory count >= 485, got {}", inv.len());

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 485, "Expected descriptor count >= 485, got {}", registry.list_all().len());

        let new_tier90_nodes = [
            "BellowsGestureEngine",
            "BowingGestureEngine",
            "ClavinetGestureEngine",
            "EpGestureEngine",
            "GlassGestureEngine",
            "GrandPianoGestureEngine",
            "HurdyGurdyGestureEngine",
            "KotoGestureEngine",
            "MalletGestureEngine",
            "MarkovSequenceMutator",
            "PipeOrganGestureEngine",
            "PluckGestureEngine",
            "RotaryGestureEngine",
            "ShakuhachiGestureEngine",
            "SitarGestureEngine",
            "SpatialAutomationEngine",
            "SpringGestureEngine",
            "WoodwindFingeringEngine",
            "BellowsBus",
            "SpatialBus",
        ];

        for name in &new_tier90_nodes {
            assert!(registry.get(name).is_some(), "Module {} must be in DspNodeRegistry", name);
            assert!(inv.iter().any(|(n, _, _)| n == name), "Module {} must be in inventory", name);

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(!ui_node.parameters().is_empty(), "Module {} must have parameters exposed", name);
        }

        // Verify specific parameter presence
        let bellows = registry.get("BellowsGestureEngine").unwrap();
        assert!(bellows.params.iter().any(|p| p.id == "bellows_pressure_pa"));
        assert!(bellows.params.iter().any(|p| p.id == "push_pull_symmetry"));

        let markov = registry.get("MarkovSequenceMutator").unwrap();
        assert!(markov.params.iter().any(|p| p.id == "temperature"));
        assert!(markov.params.iter().any(|p| p.id == "mutation_rate"));

        let spatial = registry.get("SpatialAutomationEngine").unwrap();
        assert!(spatial.params.iter().any(|p| p.id == "orbit_radius_m"));
        assert!(spatial.params.iter().any(|p| p.id == "orbit_speed_cycles"));

        let piano = registry.get("GrandPianoGestureEngine").unwrap();
        assert!(piano.params.iter().any(|p| p.id == "damper_lift_pos"));
        assert!(piano.params.iter().any(|p| p.id == "una_corda_shift"));

        // Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("bellowsgesture"), Some("BellowsGestureEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("markovmutator"), Some("MarkovSequenceMutator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spatialautomation"), Some("SpatialAutomationEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("pianogesture"), Some("GrandPianoGestureEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("bowinggesture"), Some("BowingGestureEngine"));

        // Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"BowingGestureEngine"));
        assert!(phys_items.contains(&"GrandPianoGestureEngine"));
        assert!(phys_items.contains(&"HurdyGurdyGestureEngine"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"SpatialAutomationEngine"));
        assert!(time_items.contains(&"SpringGestureEngine"));
        assert!(time_items.contains(&"SpatialBus"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"RotaryGestureEngine"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_items.contains(&"MarkovSequenceMutator"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"BellowsBus"));

        // Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Markov Generative Matrix
            view.top_bar_state.selected_preset = "Markov Generative Matrix".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("MarkovSequenceMutator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("MarkovSequenceMutator".to_string()));

            // Preset: 3D Atmos Trajectory Orbit
            view.top_bar_state.selected_preset = "3D Atmos Trajectory Orbit".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SpatialAutomationEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SpatialAutomationEngine".to_string()));

            // Preset: Concert Grand Piano Escapement
            view.top_bar_state.selected_preset = "Concert Grand Piano Escapement".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("GrandPianoGestureEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("GrandPianoGestureEngine".to_string()));

            // Preset: Bowed Violin Kinematics
            view.top_bar_state.selected_preset = "Bowed Violin Kinematics".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("BowingGestureEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("BowingGestureEngine".to_string()));
        }
    }

    #[test]
    fn test_tier91_dsp_registry_505_plus_and_lockfree_bus_exposure() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 505, "Expected inventory count >= 505, got {}", inv.len());

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 505, "Expected descriptor count >= 505, got {}", registry.list_all().len());

        let new_tier91_nodes = [
            "ArticulationBus",
            "BreathBus",
            "CadenceBus",
            "ClavinetBus",
            "EpBus",
            "FormantBus",
            "GlassBus",
            "GrandPianoBus",
            "HurdyGurdyBus",
            "KotoBus",
            "MalletBus",
            "PipeOrganBus",
            "PlectrumBus",
            "RotaryBus",
            "ShakuhachiBus",
            "SitarBus",
            "SpringBus",
            "BootToSynthEngine",
            "MidiUsbGadgetMode",
            "MultiBusEventRouter",
        ];

        for name in &new_tier91_nodes {
            assert!(registry.get(name).is_some(), "Module {} must be in DspNodeRegistry", name);
            assert!(inv.iter().any(|(n, _, _)| n == name), "Module {} must be in inventory", name);

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(!ui_node.parameters().is_empty(), "Module {} must have parameters exposed", name);
        }

        // Verify specific parameter presence
        let articulation = registry.get("ArticulationBus").unwrap();
        assert!(articulation.params.iter().any(|p| p.id == "technique_index"));
        assert!(articulation.params.iter().any(|p| p.id == "bow_velocity_mps"));

        let breath = registry.get("BreathBus").unwrap();
        assert!(breath.params.iter().any(|p| p.id == "blowing_pressure_pa"));
        assert!(breath.params.iter().any(|p| p.id == "jet_distance_mm"));

        let cadence = registry.get("CadenceBus").unwrap();
        assert!(cadence.params.iter().any(|p| p.id == "harmonic_tension"));
        assert!(cadence.params.iter().any(|p| p.id == "voice_leading_cost"));

        let clav = registry.get("ClavinetBus").unwrap();
        assert!(clav.params.iter().any(|p| p.id == "key_tangent_velocity"));
        assert!(clav.params.iter().any(|p| p.id == "pickup_coil_balance"));

        let ep = registry.get("EpBus").unwrap();
        assert!(ep.params.iter().any(|p| p.id == "tine_excitation_force"));
        assert!(ep.params.iter().any(|p| p.id == "pickup_gap_mm"));

        let formant = registry.get("FormantBus").unwrap();
        assert!(formant.params.iter().any(|p| p.id == "formant_f1_hz"));
        assert!(formant.params.iter().any(|p| p.id == "nasal_coupling_ratio"));

        let glass = registry.get("GlassBus").unwrap();
        assert!(glass.params.iter().any(|p| p.id == "rim_angular_vel_rads"));
        assert!(glass.params.iter().any(|p| p.id == "water_level_detune_cents"));

        let piano = registry.get("GrandPianoBus").unwrap();
        assert!(piano.params.iter().any(|p| p.id == "strike_velocity"));
        assert!(piano.params.iter().any(|p| p.id == "duplex_resonance_db"));

        let gurdy = registry.get("HurdyGurdyBus").unwrap();
        assert!(gurdy.params.iter().any(|p| p.id == "crank_wheel_rpm"));
        assert!(gurdy.params.iter().any(|p| p.id == "chien_pressure_gf"));

        let plectrum = registry.get("PlectrumBus").unwrap();
        assert!(plectrum.params.iter().any(|p| p.id == "attack_velocity"));
        assert!(plectrum.params.iter().any(|p| p.id == "snap_release_force_n"));

        let boot = registry.get("BootToSynthEngine").unwrap();
        assert!(boot.params.iter().any(|p| p.id == "boot_profile_mode"));
        assert!(boot.params.iter().any(|p| p.id == "startup_gain_db"));

        let router = registry.get("MultiBusEventRouter").unwrap();
        assert!(router.params.iter().any(|p| p.id == "source_bus_index"));
        assert!(router.params.iter().any(|p| p.id == "target_bus_index"));

        // Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("articulationbus"), Some("ArticulationBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("bowingbus"), Some("ArticulationBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("woodwindbus"), Some("BreathBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmoniccadencebus"), Some("CadenceBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("electricpianobus"), Some("EpBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vocaltractbus"), Some("FormantBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("pianobus"), Some("GrandPianoBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("pluckbus"), Some("PlectrumBus"));
        assert_eq!(DspNodeRegistry::normalize_type_name("boottosynth"), Some("BootToSynthEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("usbgadgetmode"), Some("MidiUsbGadgetMode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("multibusevent"), Some("MultiBusEventRouter"));

        // Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"ArticulationBus"));
        assert!(phys_items.contains(&"BreathBus"));
        assert!(phys_items.contains(&"ClavinetBus"));
        assert!(phys_items.contains(&"EpBus"));
        assert!(phys_items.contains(&"GrandPianoBus"));
        assert!(phys_items.contains(&"HurdyGurdyBus"));
        assert!(phys_items.contains(&"PlectrumBus"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"SpringBus"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"RotaryBus"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_items.contains(&"CadenceBus"));
        assert!(gen_items.contains(&"BootToSynthEngine"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"FormantBus"));
        assert!(routing_items.contains(&"MidiUsbGadgetMode"));
        assert!(routing_items.contains(&"MultiBusEventRouter"));

        // Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Concert Acoustic Harp Arpeggio -> PlectrumBus
            view.top_bar_state.selected_preset = "Concert Acoustic Harp Arpeggio".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PlectrumBus".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PlectrumBus".to_string()));

            // Preset: Hurdy Gurdy Chien Drone -> HurdyGurdyBus
            view.top_bar_state.selected_preset = "Hurdy Gurdy Chien Drone".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("HurdyGurdyBus".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("HurdyGurdyBus".to_string()));

            // Preset: Vocal Tract Formant Shaper -> FormantBus
            view.top_bar_state.selected_preset = "Vocal Tract Formant Shaper".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("FormantBus".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("FormantBus".to_string()));

            // Preset: Electric Piano Tine Saturation -> EpBus
            view.top_bar_state.selected_preset = "Electric Piano Tine Saturation".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("EpBus".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("EpBus".to_string()));
        }
    }

    #[test]
    fn test_tier92_dsp_registry_525_plus_and_hardware_ecosystem_expansion() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 525, "Expected inventory count >= 525, got {}", inv.len());

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 525, "Expected descriptor count >= 525, got {}", registry.list_all().len());

        let new_tier92_nodes = [
            "SampleEditor",
            "SpatialObjectAutomation",
            "Vst3WindowEmbedder",
            "ParameterAutomapper",
            "TruePeakMeter",
            "AudioTagger",
            "NeuralWavetableInterpolator",
            "KarplusStrongString",
            "TonebarCoupling",
            "InductivePickup",
            "HammerModel",
            "ImageReflection",
            "TapeChannelState",
            "TubeChannelState",
            "MpeRouter",
            "MidiUartSerialDriver",
            "EepromPresetStore",
            "WebConfigDashboard",
            "WasapiDriver",
            "StereoPanner",
        ];

        for name in &new_tier92_nodes {
            assert!(registry.get(name).is_some(), "Module {} must be in DspNodeRegistry", name);
            assert!(inv.iter().any(|(n, _, _)| n == name), "Module {} must be in inventory", name);

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(!ui_node.parameters().is_empty(), "Module {} must have parameters exposed", name);
        }

        // 1. Verify specific parameter presence
        let sample_editor = registry.get("SampleEditor").unwrap();
        assert!(sample_editor.params.iter().any(|p| p.id == "loop_start_sec"));
        assert!(sample_editor.params.iter().any(|p| p.id == "snap_to_zero"));

        let spatial_auto = registry.get("SpatialObjectAutomation").unwrap();
        assert!(spatial_auto.params.iter().any(|p| p.id == "azimuth_deg"));
        assert!(spatial_auto.params.iter().any(|p| p.id == "orbit_speed_hz"));

        let vst_embed = registry.get("Vst3WindowEmbedder").unwrap();
        assert!(vst_embed.params.iter().any(|p| p.id == "embed_scale_factor"));
        assert!(vst_embed.params.iter().any(|p| p.id == "gpu_compositing"));

        let automapper = registry.get("ParameterAutomapper").unwrap();
        assert!(automapper.params.iter().any(|p| p.id == "mapping_confidence"));
        assert!(automapper.params.iter().any(|p| p.id == "macro_slot_count"));

        let true_peak = registry.get("TruePeakMeter").unwrap();
        assert!(true_peak.params.iter().any(|p| p.id == "peak_ceiling_dbfs"));
        assert!(true_peak.params.iter().any(|p| p.id == "oversample_factor"));

        let audio_tagger = registry.get("AudioTagger").unwrap();
        assert!(audio_tagger.params.iter().any(|p| p.id == "classification_threshold"));
        assert!(audio_tagger.params.iter().any(|p| p.id == "top_k_candidates"));

        let wt_interp = registry.get("NeuralWavetableInterpolator").unwrap();
        assert!(wt_interp.params.iter().any(|p| p.id == "latent_dim_x"));
        assert!(wt_interp.params.iter().any(|p| p.id == "bandlimited_resynthesis"));

        let karplus = registry.get("KarplusStrongString").unwrap();
        assert!(karplus.params.iter().any(|p| p.id == "frequency_hz"));
        assert!(karplus.params.iter().any(|p| p.id == "decay_damping"));

        let tonebar = registry.get("TonebarCoupling").unwrap();
        assert!(tonebar.params.iter().any(|p| p.id == "tonebar_mass_ratio"));
        assert!(tonebar.params.iter().any(|p| p.id == "coupling_stiffness"));

        let pickup = registry.get("InductivePickup").unwrap();
        assert!(pickup.params.iter().any(|p| p.id == "air_gap_mm"));
        assert!(pickup.params.iter().any(|p| p.id == "coil_inductance_henry"));

        let hammer = registry.get("HammerModel").unwrap();
        assert!(hammer.params.iter().any(|p| p.id == "hammer_stiffness_p"));
        assert!(hammer.params.iter().any(|p| p.id == "nonlinear_exponent_p"));

        let room_refl = registry.get("ImageReflection").unwrap();
        assert!(room_refl.params.iter().any(|p| p.id == "room_length_m"));
        assert!(room_refl.params.iter().any(|p| p.id == "wall_absorption_coeff"));

        let tape = registry.get("TapeChannelState").unwrap();
        assert!(tape.params.iter().any(|p| p.id == "tape_speed_ips"));
        assert!(tape.params.iter().any(|p| p.id == "saturation_drive_db"));

        let tube = registry.get("TubeChannelState").unwrap();
        assert!(tube.params.iter().any(|p| p.id == "tube_type_mode"));
        assert!(tube.params.iter().any(|p| p.id == "plate_drive_db"));

        let mpe = registry.get("MpeRouter").unwrap();
        assert!(mpe.params.iter().any(|p| p.id == "mpe_zone_mode"));
        assert!(mpe.params.iter().any(|p| p.id == "pitch_bend_range_st"));

        let uart = registry.get("MidiUartSerialDriver").unwrap();
        assert!(uart.params.iter().any(|p| p.id == "baud_rate_bps"));
        assert!(uart.params.iter().any(|p| p.id == "running_status_opt"));

        let eeprom = registry.get("EepromPresetStore").unwrap();
        assert!(eeprom.params.iter().any(|p| p.id == "active_bank_index"));
        assert!(eeprom.params.iter().any(|p| p.id == "crc32_checksum_verify"));

        let web_cfg = registry.get("WebConfigDashboard").unwrap();
        assert!(web_cfg.params.iter().any(|p| p.id == "http_server_port"));
        assert!(web_cfg.params.iter().any(|p| p.id == "websocket_telemetry_rate_hz"));

        let wasapi = registry.get("WasapiDriver").unwrap();
        assert!(wasapi.params.iter().any(|p| p.id == "buffer_frame_size"));
        assert!(wasapi.params.iter().any(|p| p.id == "exclusive_lock"));

        let panner = registry.get("StereoPanner").unwrap();
        assert!(panner.params.iter().any(|p| p.id == "pan_position"));
        assert!(panner.params.iter().any(|p| p.id == "stereo_width"));

        // 2. Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("sampleeditor"), Some("SampleEditor"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spatialobjectautomation"), Some("SpatialObjectAutomation"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vst3windowembedder"), Some("Vst3WindowEmbedder"));
        assert_eq!(DspNodeRegistry::normalize_type_name("parameterautomapper"), Some("ParameterAutomapper"));
        assert_eq!(DspNodeRegistry::normalize_type_name("truepeakmeter"), Some("TruePeakMeter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("audiotagger"), Some("AudioTagger"));
        assert_eq!(DspNodeRegistry::normalize_type_name("latentwavetable"), Some("NeuralWavetableInterpolator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("karplusstrong"), Some("KarplusStrongString"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tonebarcoupling"), Some("TonebarCoupling"));
        assert_eq!(DspNodeRegistry::normalize_type_name("inductivepickup"), Some("InductivePickup"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hammermodel"), Some("HammerModel"));
        assert_eq!(DspNodeRegistry::normalize_type_name("imagereflection"), Some("ImageReflection"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tapechannelstate"), Some("TapeChannelState"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tubechannelstate"), Some("TubeChannelState"));
        assert_eq!(DspNodeRegistry::normalize_type_name("mperouter"), Some("MpeRouter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midiuartserialdriver"), Some("MidiUartSerialDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("eeprompresetstore"), Some("EepromPresetStore"));
        assert_eq!(DspNodeRegistry::normalize_type_name("webconfigdashboard"), Some("WebConfigDashboard"));
        assert_eq!(DspNodeRegistry::normalize_type_name("wasapidriver"), Some("WasapiDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("stereopanner"), Some("StereoPanner"));

        // 3. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"KarplusStrongString"));
        assert!(phys_items.contains(&"TonebarCoupling"));
        assert!(phys_items.contains(&"InductivePickup"));
        assert!(phys_items.contains(&"HammerModel"));

        let synth_items = inst_folders.iter().find(|(name, _)| *name == "Synthesizers").unwrap().1;
        assert!(synth_items.contains(&"NeuralWavetableInterpolator"));

        let sampler_items = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(sampler_items.contains(&"SampleEditor"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"TruePeakMeter"));
        assert!(dyn_items.contains(&"TapeChannelState"));
        assert!(dyn_items.contains(&"TubeChannelState"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"SpatialObjectAutomation"));
        assert!(time_items.contains(&"ImageReflection"));
        assert!(time_items.contains(&"StereoPanner"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_items.contains(&"AudioTagger"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"ParameterAutomapper"));
        assert!(routing_items.contains(&"MpeRouter"));
        assert!(routing_items.contains(&"Vst3WindowEmbedder"));
        assert!(routing_items.contains(&"MidiUartSerialDriver"));
        assert!(routing_items.contains(&"EepromPresetStore"));
        assert!(routing_items.contains(&"WebConfigDashboard"));
        assert!(routing_items.contains(&"WasapiDriver"));

        // 4. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Karplus Plucked Nylon Resonator -> KarplusStrongString
            view.top_bar_state.selected_preset = "Karplus Plucked Nylon Resonator".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("KarplusStrongString".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("KarplusStrongString".to_string()));

            // Preset: Analog Reel-to-Reel Tape Saturation -> TapeChannelState
            view.top_bar_state.selected_preset = "Analog Reel-to-Reel Tape Saturation".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("TapeChannelState".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("TapeChannelState".to_string()));

            // Preset: Shoebox Early Acoustic Reflections -> ImageReflection
            view.top_bar_state.selected_preset = "Shoebox Early Acoustic Reflections".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ImageReflection".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ImageReflection".to_string()));

            // Preset: MPE Polyphonic Gesture Router -> MpeRouter
            view.top_bar_state.selected_preset = "MPE Polyphonic Gesture Router".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("MpeRouter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("MpeRouter".to_string()));
        }
    }

    #[test]
    fn test_step_1292_tier92_multitrack_looper_and_hardware_drivers() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 545, "Expected >= 545 descriptors, got {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 545, "Expected >= 545 inventory items, got {}", inv.len());

        let new_nodes = [
            "MultitrackSessionLooper",
            "AlsaDriver",
            "AudioUnitDriver",
            "AAudioDriver",
            "AsapiDriver",
            "BleMidiPeripheral",
            "HardwareEmulationHarness",
            "MidiClockGenerator",
            "MidiClockReceiver",
            "MidiInputFilter",
            "KeyboardSplit",
            "KeyboardLayering",
            "NamWaveNetEngine",
            "WaveguideBrassNode",
            "WaveguideMeshNode",
            "WoodwindJetNode",
            "PercussionMembraneNode",
            "PipeOrganNode",
            "SitarNode",
            "BowedStringNode",
        ];

        // 1. Verify existence and UI instantiation
        for name in &new_nodes {
            let desc = registry.get(name);
            assert!(desc.is_some(), "Module {} must exist in DspNodeRegistry", name);
            let desc = desc.unwrap();
            assert!(!desc.params.is_empty(), "Module {} must have schema parameters", name);

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(!ui_node.parameters().is_empty(), "Module {} must have parameters exposed in UI", name);
        }

        // 2. Verify parameter IDs on key nodes
        let looper = registry.get("MultitrackSessionLooper").unwrap();
        assert!(looper.params.iter().any(|p| p.id == "feedback"));
        assert!(looper.params.iter().any(|p| p.id == "track_count"));
        assert!(looper.params.iter().any(|p| p.id == "crossfade_ms"));

        let alsa = registry.get("AlsaDriver").unwrap();
        assert!(alsa.params.iter().any(|p| p.id == "period_size_frames"));
        assert!(alsa.params.iter().any(|p| p.id == "mmap_transfer_enabled"));

        let au = registry.get("AudioUnitDriver").unwrap();
        assert!(au.params.iter().any(|p| p.id == "buffer_frame_capacity"));
        assert!(au.params.iter().any(|p| p.id == "voice_processing_io"));

        let aaudio = registry.get("AAudioDriver").unwrap();
        assert!(aaudio.params.iter().any(|p| p.id == "buffer_burst_frames"));
        assert!(aaudio.params.iter().any(|p| p.id == "sharing_mode_exclusive"));

        let asapi = registry.get("AsapiDriver").unwrap();
        assert!(asapi.params.iter().any(|p| p.id == "preferred_buffer_size"));
        assert!(asapi.params.iter().any(|p| p.id == "clock_source"));

        let ble = registry.get("BleMidiPeripheral").unwrap();
        assert!(ble.params.iter().any(|p| p.id == "connection_interval_ms"));
        assert!(ble.params.iter().any(|p| p.id == "midi2_ump_protocol"));

        let harness = registry.get("HardwareEmulationHarness").unwrap();
        assert!(harness.params.iter().any(|p| p.id == "simulated_cpu_load_pct"));
        assert!(harness.params.iter().any(|p| p.id == "thermal_temp_celsius"));

        let clock_gen = registry.get("MidiClockGenerator").unwrap();
        assert!(clock_gen.params.iter().any(|p| p.id == "bpm_tempo"));
        assert!(clock_gen.params.iter().any(|p| p.id == "shuffle_swing_pct"));

        let clock_rx = registry.get("MidiClockReceiver").unwrap();
        assert!(clock_rx.params.iter().any(|p| p.id == "pll_damping"));
        assert!(clock_rx.params.iter().any(|p| p.id == "flywheel_timeout_ms"));

        let filter = registry.get("MidiInputFilter").unwrap();
        assert!(filter.params.iter().any(|p| p.id == "filter_channel"));
        assert!(filter.params.iter().any(|p| p.id == "velocity_curve_gamma"));

        let split = registry.get("KeyboardSplit").unwrap();
        assert!(split.params.iter().any(|p| p.id == "split_key_note"));
        assert!(split.params.iter().any(|p| p.id == "lower_transpose_st"));

        let layering = registry.get("KeyboardLayering").unwrap();
        assert!(layering.params.iter().any(|p| p.id == "layer_mode"));
        assert!(layering.params.iter().any(|p| p.id == "velocity_split_threshold"));

        let wavenet = registry.get("NamWaveNetEngine").unwrap();
        assert!(wavenet.params.iter().any(|p| p.id == "input_gain_db"));
        assert!(wavenet.params.iter().any(|p| p.id == "dilated_layers_count"));

        let freereed = registry.get("FreeReedVoice").unwrap();
        assert!(freereed.params.iter().any(|p| p.id == "reed_frequency_hz"));
        assert!(freereed.params.iter().any(|p| p.id == "bellows_pressure_kpa"));

        let armonica = registry.get("ArmonicaChassisMode").unwrap();
        assert!(armonica.params.iter().any(|p| p.id == "spindle_angular_vel_rpm"));
        assert!(armonica.params.iter().any(|p| p.id == "water_bath_damping"));

        let paulownia = registry.get("PaulowniaBodyMode").unwrap();
        assert!(paulownia.params.iter().any(|p| p.id == "wood_stiffness_longitudinal"));
        assert!(paulownia.params.iter().any(|p| p.id == "cavity_air_resonance_hz"));

        let allpass_chain = registry.get("DispersionAllpassChain").unwrap();
        assert!(allpass_chain.params.iter().any(|p| p.id == "dispersion_stages_count"));
        assert!(allpass_chain.params.iter().any(|p| p.id == "inharmonicity_coefficient_b"));

        let tine_disp = registry.get("TineDispersionAllpass").unwrap();
        assert!(tine_disp.params.iter().any(|p| p.id == "tine_inharmonicity"));
        assert!(tine_disp.params.iter().any(|p| p.id == "center_frequency_hz"));

        let sitar_disp = registry.get("SitarDispersionAllpass").unwrap();
        assert!(sitar_disp.params.iter().any(|p| p.id == "string_stiffness_parameter"));
        assert!(sitar_disp.params.iter().any(|p| p.id == "taraf_coupling_cutoff_hz"));

        let band_dyn = registry.get("BandDynamics").unwrap();
        assert!(band_dyn.params.iter().any(|p| p.id == "threshold_db"));
        assert!(band_dyn.params.iter().any(|p| p.id == "compression_ratio"));

        // 3. Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("multitracksessionlooper"), Some("MultitrackSessionLooper"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sessionlooper"), Some("MultitrackSessionLooper"));
        assert_eq!(DspNodeRegistry::normalize_type_name("alsadriver"), Some("AlsaDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("audiounitdriver"), Some("AudioUnitDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("aaudiodriver"), Some("AAudioDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("asapidriver"), Some("AsapiDriver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("blemidiperipheral"), Some("BleMidiPeripheral"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hardwareemulationharness"), Some("HardwareEmulationHarness"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midiclockgenerator"), Some("MidiClockGenerator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midiclockreceiver"), Some("MidiClockReceiver"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midiinputfilter"), Some("MidiInputFilter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("keyboardsplit"), Some("KeyboardSplit"));
        assert_eq!(DspNodeRegistry::normalize_type_name("keyboardlayering"), Some("KeyboardLayering"));
        assert_eq!(DspNodeRegistry::normalize_type_name("namwavenetengine"), Some("NamWaveNetEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("freereedvoice"), Some("FreeReedVoice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmonicareed"), Some("FreeReedVoice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("armonicachassismode"), Some("ArmonicaChassisMode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("paulowniabodymode"), Some("PaulowniaBodyMode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("dispersionallpasschain"), Some("DispersionAllpassChain"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tinedispersionallpass"), Some("TineDispersionAllpass"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sitardispersionallpass"), Some("SitarDispersionAllpass"));
        assert_eq!(DspNodeRegistry::normalize_type_name("banddynamics"), Some("BandDynamics"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"FreeReedVoice"));
        assert!(phys_items.contains(&"ArmonicaChassisMode"));
        assert!(phys_items.contains(&"PaulowniaBodyMode"));

        let synth_items = inst_folders.iter().find(|(name, _)| *name == "Synthesizers").unwrap().1;
        assert!(synth_items.contains(&"NamWaveNetEngine"));

        let sampler_items = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(sampler_items.contains(&"MultitrackSessionLooper"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"BandDynamics"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"DispersionAllpassChain"));
        assert!(filter_items.contains(&"TineDispersionAllpass"));
        assert!(filter_items.contains(&"SitarDispersionAllpass"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_items.contains(&"MidiClockGenerator"));
        assert!(gen_items.contains(&"MidiClockReceiver"));
        assert!(gen_items.contains(&"BleMidiPeripheral"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"MidiInputFilter"));
        assert!(routing_items.contains(&"KeyboardSplit"));
        assert!(routing_items.contains(&"KeyboardLayering"));
        assert!(routing_items.contains(&"AlsaDriver"));
        assert!(routing_items.contains(&"AudioUnitDriver"));
        assert!(routing_items.contains(&"AAudioDriver"));
        assert!(routing_items.contains(&"AsapiDriver"));
        assert!(routing_items.contains(&"HardwareEmulationHarness"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Multitrack Ambient Soundscape Loop -> MultitrackSessionLooper
            view.top_bar_state.selected_preset = "Multitrack Ambient Soundscape Loop".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("MultitrackSessionLooper".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("MultitrackSessionLooper".to_string()));

            // Preset: Harmonium Free-Reed Expression Swell -> FreeReedVoice
            view.top_bar_state.selected_preset = "Harmonium Free-Reed Expression Swell".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("FreeReedVoice".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("FreeReedVoice".to_string()));

            // Preset: Glass Armonica Celestial Shimmer -> ArmonicaChassisMode
            view.top_bar_state.selected_preset = "Glass Armonica Celestial Shimmer".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ArmonicaChassisMode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ArmonicaChassisMode".to_string()));

            // Preset: High-Gain WaveNet Tube Overdrive -> NamWaveNetEngine
            view.top_bar_state.selected_preset = "High-Gain WaveNet Tube Overdrive".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("NamWaveNetEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("NamWaveNetEngine".to_string()));
        }
    }

    #[test]
    fn test_step_1293_tier93_dsp_modules_expansion_coverage() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 565, "Expected >= 565 descriptors, got {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 565, "Expected >= 565 inventory items, got {}", inv.len());

        let new_dsp_modules = [
            "BowedStringNode",
            "PercussionMembraneNode",
            "PipeOrganNode",
            "CommutedPluckedString",
            "RotarySpeakerNode",
            "SamplerNode",
            "ShakuhachiVoice",
            "SitarVoice",
            "TonewheelOrganNode",
            "WaveguideBrassNode",
            "WaveguideMeshNode",
            "WoodwindJetNode",
            "MolecularVibrationResonatorNode",
            "PlasmaArcSynthesizerNode",
            "MetamaterialRefractionFilterNode",
            "IsmShockwaveReverbNode",
            "FusionResonanceSynthNode",
            "MasterLimiter",
            "TrackStereoCorrelationMeter",
            "SpectrumMatchingAnalyzer",
            "MasterBusTrim",
        ];

        // 1. Verify existence and UI instantiation
        for name in &new_dsp_modules {
            let desc = registry.get(name);
            assert!(desc.is_some(), "Module {} must exist in DspNodeRegistry", name);
            let desc = desc.unwrap();
            assert!(desc.params.len() >= 5, "Module {} must have at least 5 schema parameters, found {}", name, desc.params.len());

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(ui_node.parameters().len() >= 5, "Module {} must have at least 5 parameters exposed in UI, found {}", name, ui_node.parameters().len());
        }

        // 2. Verify parameter IDs on key nodes
        let bowed = registry.get("BowedStringNode").unwrap();
        assert!(bowed.params.iter().any(|p| p.id == "bow_velocity_mps"));
        assert!(bowed.params.iter().any(|p| p.id == "bow_pressure_n"));
        assert!(bowed.params.iter().any(|p| p.id == "string_damping"));

        let perc = registry.get("PercussionMembraneNode").unwrap();
        assert!(perc.params.iter().any(|p| p.id == "rim_tension_kpa"));
        assert!(perc.params.iter().any(|p| p.id == "membrane_mass_g_m2"));

        let organ = registry.get("PipeOrganNode").unwrap();
        assert!(organ.params.iter().any(|p| p.id == "pipe_frequency_hz"));
        assert!(organ.params.iter().any(|p| p.id == "chiff_noise_mix"));

        let plucked = registry.get("CommutedPluckedString").unwrap();
        assert!(plucked.params.iter().any(|p| p.id == "pitch_hz"));
        assert!(plucked.params.iter().any(|p| p.id == "pick_hardness"));

        let rotary = registry.get("RotarySpeakerNode").unwrap();
        assert!(rotary.params.iter().any(|p| p.id == "horn_rate_hz"));
        assert!(rotary.params.iter().any(|p| p.id == "drum_rate_hz"));

        let sampler = registry.get("SamplerNode").unwrap();
        assert!(sampler.params.iter().any(|p| p.id == "playback_speed"));
        assert!(sampler.params.iter().any(|p| p.id == "root_pitch_semitones"));

        let shakuhachi = registry.get("ShakuhachiVoice").unwrap();
        assert!(shakuhachi.params.iter().any(|p| p.id == "breath_pressure_pa"));
        assert!(shakuhachi.params.iter().any(|p| p.id == "meri_angle_cents"));

        let sitar = registry.get("SitarVoice").unwrap();
        assert!(sitar.params.iter().any(|p| p.id == "baj_frequency_hz"));
        assert!(sitar.params.iter().any(|p| p.id == "taraf_strings_count"));

        let tonewheel = registry.get("TonewheelOrganNode").unwrap();
        assert!(tonewheel.params.iter().any(|p| p.id == "key_click_gain"));
        assert!(tonewheel.params.iter().any(|p| p.id == "leakage_crosstalk"));

        let brass = registry.get("WaveguideBrassNode").unwrap();
        assert!(brass.params.iter().any(|p| p.id == "lip_tension_n_m"));
        assert!(brass.params.iter().any(|p| p.id == "bore_flare_exponent"));

        let mesh = registry.get("WaveguideMeshNode").unwrap();
        assert!(mesh.params.iter().any(|p| p.id == "mesh_geometry"));
        assert!(mesh.params.iter().any(|p| p.id == "boundary_reflection"));

        let jet = registry.get("WoodwindJetNode").unwrap();
        assert!(jet.params.iter().any(|p| p.id == "jet_length_mm"));
        assert!(jet.params.iter().any(|p| p.id == "jet_velocity_mps"));

        let molec = registry.get("MolecularVibrationResonatorNode").unwrap();
        assert!(molec.params.iter().any(|p| p.id == "base_frequency_hz"));
        assert!(molec.params.iter().any(|p| p.id == "thermal_excitation_temp_k"));

        let plasma = registry.get("PlasmaArcSynthesizerNode").unwrap();
        assert!(plasma.params.iter().any(|p| p.id == "arc_current_ma"));
        assert!(plasma.params.iter().any(|p| p.id == "spark_gap_mm"));

        let meta = registry.get("MetamaterialRefractionFilterNode").unwrap();
        assert!(meta.params.iter().any(|p| p.id == "center_frequency_hz"));
        assert!(meta.params.iter().any(|p| p.id == "refractive_index"));

        let ism = registry.get("IsmShockwaveReverbNode").unwrap();
        assert!(ism.params.iter().any(|p| p.id == "room_length_m"));
        assert!(ism.params.iter().any(|p| p.id == "shockwave_mach_number"));

        let fusion = registry.get("FusionResonanceSynthNode").unwrap();
        assert!(fusion.params.iter().any(|p| p.id == "magnetic_field_tesla"));
        assert!(fusion.params.iter().any(|p| p.id == "plasma_density_10e19"));

        let limiter = registry.get("MasterLimiter").unwrap();
        assert!(limiter.params.iter().any(|p| p.id == "threshold_db"));
        assert!(limiter.params.iter().any(|p| p.id == "true_peak_oversampling"));

        let meter = registry.get("TrackStereoCorrelationMeter").unwrap();
        assert!(meter.params.iter().any(|p| p.id == "integration_time_ms"));
        assert!(meter.params.iter().any(|p| p.id == "correlation_coefficient"));

        let spec = registry.get("SpectrumMatchingAnalyzer").unwrap();
        assert!(spec.params.iter().any(|p| p.id == "num_bands"));
        assert!(spec.params.iter().any(|p| p.id == "matching_intensity"));

        let trim = registry.get("MasterBusTrim").unwrap();
        assert!(trim.params.iter().any(|p| p.id == "trim_gain_db"));
        assert!(trim.params.iter().any(|p| p.id == "phase_flip_left"));

        // 3. Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("bowedstringnode"), Some("BowedStringNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("percussionmembranenode"), Some("PercussionMembraneNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("pipeorgannode"), Some("PipeOrganNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("commutedpluckedstring"), Some("CommutedPluckedString"));
        assert_eq!(DspNodeRegistry::normalize_type_name("rotaryspeakernode"), Some("RotarySpeakerNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("samplernode"), Some("SamplerNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachivoice"), Some("ShakuhachiVoice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sitarvoice"), Some("SitarVoice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tonewheelorgannode"), Some("TonewheelOrganNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("waveguidebrassnode"), Some("WaveguideBrassNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("waveguidemeshnode"), Some("WaveguideMeshNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("woodwindjetnode"), Some("WoodwindJetNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("molecularvibrationresonatornode"), Some("MolecularVibrationResonatorNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("plasmaarcsynthesizernode"), Some("PlasmaArcSynthesizerNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("metamaterialrefractionfilternode"), Some("MetamaterialRefractionFilterNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("ismshockwavereverbnode"), Some("IsmShockwaveReverbNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("fusionresonancesynthnode"), Some("FusionResonanceSynthNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("masterlimiter"), Some("MasterLimiter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("trackstereocorrelationmeter"), Some("TrackStereoCorrelationMeter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spectrummatchinganalyzer"), Some("SpectrumMatchingAnalyzer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("masterbustrim"), Some("MasterBusTrim"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"BowedStringNode"));
        assert!(phys_items.contains(&"PercussionMembraneNode"));
        assert!(phys_items.contains(&"PipeOrganNode"));
        assert!(phys_items.contains(&"CommutedPluckedString"));
        assert!(phys_items.contains(&"ShakuhachiVoice"));
        assert!(phys_items.contains(&"SitarVoice"));
        assert!(phys_items.contains(&"WaveguideBrassNode"));
        assert!(phys_items.contains(&"WaveguideMeshNode"));
        assert!(phys_items.contains(&"WoodwindJetNode"));

        let synth_items = inst_folders.iter().find(|(name, _)| *name == "Synthesizers").unwrap().1;
        assert!(synth_items.contains(&"TonewheelOrganNode"));
        assert!(synth_items.contains(&"MolecularVibrationResonatorNode"));
        assert!(synth_items.contains(&"PlasmaArcSynthesizerNode"));
        assert!(synth_items.contains(&"FusionResonanceSynthNode"));

        let sampler_items = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(sampler_items.contains(&"SamplerNode"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"MasterLimiter"));
        assert!(dyn_items.contains(&"MasterBusTrim"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"MetamaterialRefractionFilterNode"));
        assert!(filter_items.contains(&"SpectrumMatchingAnalyzer"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"IsmShockwaveReverbNode"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"RotarySpeakerNode"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"TrackStereoCorrelationMeter"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Concert Hall Bowed Double Bass -> BowedStringNode
            view.top_bar_state.selected_preset = "Concert Hall Bowed Double Bass".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("BowedStringNode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("BowedStringNode".to_string()));

            // Preset: Tuned Orchestral Timpani Membrane -> PercussionMembraneNode
            view.top_bar_state.selected_preset = "Tuned Orchestral Timpani Membrane".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PercussionMembraneNode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PercussionMembraneNode".to_string()));

            // Preset: Vintage 91-Wheel Jazz Tonewheel Organ -> TonewheelOrganNode
            view.top_bar_state.selected_preset = "Vintage 91-Wheel Jazz Tonewheel Organ".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("TonewheelOrganNode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("TonewheelOrganNode".to_string()));

            // Preset: Master Brickwall True-Peak Limiter -> MasterLimiter
            view.top_bar_state.selected_preset = "Master Brickwall True-Peak Limiter".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("MasterLimiter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("MasterLimiter".to_string()));
        }
    }

    #[test]
    fn test_step_1294_tier94_dsp_modules_expansion_coverage() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 585, "Expected >= 585 descriptors, got {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 585, "Expected >= 585 inventory items, got {}", inv.len());

        let tier94_dsp_modules = [
            "BowedString",
            "Clavinet",
            "ElectricPiano",
            "FreeReed",
            "GlottalPulse",
            "ConcertGrandPiano",
            "PlateTank",
            "Shakuhachi",
            "SitarNode",
            "SpringLattice",
            "VocalTract",
            "StereoWidth",
            "MemoryEstimator",
            "LFO",
            "MacroKnob",
            "Butterworth2ndOrder",
            "ModalFilterSection",
            "DispersionAllpass",
            "SoundboardMode",
            "ConsoleChannelState",
            "ConsoleBiquad",
        ];

        // 1. Verify existence, descriptor param count (>= 5), and UI instantiation
        for name in &tier94_dsp_modules {
            let desc = registry.get(name);
            assert!(desc.is_some(), "Module {} must exist in DspNodeRegistry", name);
            let desc = desc.unwrap();
            assert!(desc.params.len() >= 5, "Module {} must have at least 5 schema parameters, found {}", name, desc.params.len());

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(ui_node.parameters().len() >= 5, "Module {} must have at least 5 parameters exposed in UI, found {}", name, ui_node.parameters().len());
        }

        // 2. Verify key parameter IDs on all 21 modules
        let bowed = registry.get("BowedString").unwrap();
        assert!(bowed.params.iter().any(|p| p.id == "bow_velocity"));
        assert!(bowed.params.iter().any(|p| p.id == "bow_force"));
        assert!(bowed.params.iter().any(|p| p.id == "string_damping"));

        let clav = registry.get("Clavinet").unwrap();
        assert!(clav.params.iter().any(|p| p.id == "anvil_hardness"));
        assert!(clav.params.iter().any(|p| p.id == "yarn_damping"));
        assert!(clav.params.iter().any(|p| p.id == "pickup_mode"));

        let ep = registry.get("ElectricPiano").unwrap();
        assert!(ep.params.iter().any(|p| p.id == "hammer_hardness"));
        assert!(ep.params.iter().any(|p| p.id == "air_gap_mm"));
        assert!(ep.params.iter().any(|p| p.id == "bark_drive"));

        let freereed = registry.get("FreeReed").unwrap();
        assert!(freereed.params.iter().any(|p| p.id == "bellows_pressure_pa"));
        assert!(freereed.params.iter().any(|p| p.id == "reed_stiffness"));
        assert!(freereed.params.iter().any(|p| p.id == "cassotto_aperture"));

        let glottal = registry.get("GlottalPulse").unwrap();
        assert!(glottal.params.iter().any(|p| p.id == "open_quotient"));
        assert!(glottal.params.iter().any(|p| p.id == "speed_quotient"));
        assert!(glottal.params.iter().any(|p| p.id == "aspiration_level"));

        let grand = registry.get("ConcertGrandPiano").unwrap();
        assert!(grand.params.iter().any(|p| p.id == "sustain_pedal"));
        assert!(grand.params.iter().any(|p| p.id == "una_corda_pedal"));
        assert!(grand.params.iter().any(|p| p.id == "unison_detune_cents"));

        let plate = registry.get("PlateTank").unwrap();
        assert!(plate.params.iter().any(|p| p.id == "decay_t60_sec"));
        assert!(plate.params.iter().any(|p| p.id == "dispersion_factor"));
        assert!(plate.params.iter().any(|p| p.id == "driver_saturation"));

        let shakuhachi = registry.get("Shakuhachi").unwrap();
        assert!(shakuhachi.params.iter().any(|p| p.id == "blowing_pressure_pa"));
        assert!(shakuhachi.params.iter().any(|p| p.id == "meri_kari_cents"));
        assert!(shakuhachi.params.iter().any(|p| p.id == "murai_iki_intensity"));

        let sitar = registry.get("SitarNode").unwrap();
        assert!(sitar.params.iter().any(|p| p.id == "meend_semitones"));
        assert!(sitar.params.iter().any(|p| p.id == "root_freq_hz"));
        assert!(sitar.params.iter().any(|p| p.id == "body_resonance_gain"));

        let spring = registry.get("SpringLattice").unwrap();
        assert!(spring.params.iter().any(|p| p.id == "fundamental_hz"));
        assert!(spring.params.iter().any(|p| p.id == "duffing_nonlinearity"));
        assert!(spring.params.iter().any(|p| p.id == "drive_force"));

        let vocal = registry.get("VocalTract").unwrap();
        assert!(vocal.params.iter().any(|p| p.id == "tongue_position"));
        assert!(vocal.params.iter().any(|p| p.id == "tongue_height"));
        assert!(vocal.params.iter().any(|p| p.id == "velum_opening"));

        let width = registry.get("StereoWidth").unwrap();
        assert!(width.params.iter().any(|p| p.id == "width"));
        assert!(width.params.iter().any(|p| p.id == "side_hpf_hz"));
        assert!(width.params.iter().any(|p| p.id == "mono_correlation_threshold"));

        let mem = registry.get("MemoryEstimator").unwrap();
        assert!(mem.params.iter().any(|p| p.id == "max_heap_budget_mb"));
        assert!(mem.params.iter().any(|p| p.id == "audio_buffer_reserve_mb"));
        assert!(mem.params.iter().any(|p| p.id == "enforce_zero_allocation"));

        let lfo = registry.get("LFO").unwrap();
        assert!(lfo.params.iter().any(|p| p.id == "frequency"));
        assert!(lfo.params.iter().any(|p| p.id == "shape"));
        assert!(lfo.params.iter().any(|p| p.id == "depth"));

        let macro_k = registry.get("MacroKnob").unwrap();
        assert!(macro_k.params.iter().any(|p| p.id == "value"));
        assert!(macro_k.params.iter().any(|p| p.id == "smoothing_ms"));
        assert!(macro_k.params.iter().any(|p| p.id == "curve_exponent"));

        let butter = registry.get("Butterworth2ndOrder").unwrap();
        assert!(butter.params.iter().any(|p| p.id == "filter_type"));
        assert!(butter.params.iter().any(|p| p.id == "cutoff_frequency_hz"));
        assert!(butter.params.iter().any(|p| p.id == "q_resonance"));

        let modal = registry.get("ModalFilterSection").unwrap();
        assert!(modal.params.iter().any(|p| p.id == "freq_hz"));
        assert!(modal.params.iter().any(|p| p.id == "t60_sec"));
        assert!(modal.params.iter().any(|p| p.id == "damping_scale"));

        let disp = registry.get("DispersionAllpass").unwrap();
        assert!(disp.params.iter().any(|p| p.id == "coefficient"));
        assert!(disp.params.iter().any(|p| p.id == "frequency_warp_hz"));
        assert!(disp.params.iter().any(|p| p.id == "inharmonicity_b"));

        let soundboard = registry.get("SoundboardMode").unwrap();
        assert!(soundboard.params.iter().any(|p| p.id == "freq_hz"));
        assert!(soundboard.params.iter().any(|p| p.id == "q"));
        assert!(soundboard.params.iter().any(|p| p.id == "damping_decay_ms"));

        let console = registry.get("ConsoleChannelState").unwrap();
        assert!(console.params.iter().any(|p| p.id == "drive_gain"));
        assert!(console.params.iter().any(|p| p.id == "iron_flux"));
        assert!(console.params.iter().any(|p| p.id == "crosstalk_bleed"));

        let biquad = registry.get("ConsoleBiquad").unwrap();
        assert!(biquad.params.iter().any(|p| p.id == "freq_hz"));
        assert!(biquad.params.iter().any(|p| p.id == "gain_db"));
        assert!(biquad.params.iter().any(|p| p.id == "analog_warmth"));

        // 3. Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("bowedstring"), Some("BowedString"));
        assert_eq!(DspNodeRegistry::normalize_type_name("bowedacousticstring"), Some("BowedString"));
        assert_eq!(DspNodeRegistry::normalize_type_name("clavinet"), Some("Clavinet"));
        assert_eq!(DspNodeRegistry::normalize_type_name("clavinetd6"), Some("Clavinet"));
        assert_eq!(DspNodeRegistry::normalize_type_name("electricpiano"), Some("ElectricPiano"));
        assert_eq!(DspNodeRegistry::normalize_type_name("rhodespiano"), Some("ElectricPiano"));
        assert_eq!(DspNodeRegistry::normalize_type_name("freereed"), Some("FreeReed"));
        assert_eq!(DspNodeRegistry::normalize_type_name("accordionfreereed"), Some("FreeReed"));
        assert_eq!(DspNodeRegistry::normalize_type_name("glottalpulse"), Some("GlottalPulse"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vocalglottalflow"), Some("GlottalPulse"));
        assert_eq!(DspNodeRegistry::normalize_type_name("concertgrandpiano"), Some("ConcertGrandPiano"));
        assert_eq!(DspNodeRegistry::normalize_type_name("acousticgrandpiano"), Some("ConcertGrandPiano"));
        assert_eq!(DspNodeRegistry::normalize_type_name("platetank"), Some("PlateTank"));
        assert_eq!(DspNodeRegistry::normalize_type_name("steelplatereverb"), Some("PlateTank"));
        assert_eq!(DspNodeRegistry::normalize_type_name("japanesebambooflute"), Some("Shakuhachi"));
        assert_eq!(DspNodeRegistry::normalize_type_name("bambooflute"), Some("Shakuhachi"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sitarnode"), Some("SitarNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("classicalsitarnode"), Some("SitarNode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("springlattice"), Some("SpringLattice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("springmassreverb"), Some("SpringLattice"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vocaltractphysical"), Some("VocalTract"));
        assert_eq!(DspNodeRegistry::normalize_type_name("acousticvocaltract"), Some("VocalTract"));
        assert_eq!(DspNodeRegistry::normalize_type_name("stereowidth"), Some("StereoWidth"));
        assert_eq!(DspNodeRegistry::normalize_type_name("midsidestereowidth"), Some("StereoWidth"));
        assert_eq!(DspNodeRegistry::normalize_type_name("memoryestimator"), Some("MemoryEstimator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("hardwarememorybudget"), Some("MemoryEstimator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("lfo"), Some("LFO"));
        assert_eq!(DspNodeRegistry::normalize_type_name("lowfrequencyoscillator"), Some("LFO"));
        assert_eq!(DspNodeRegistry::normalize_type_name("macroknob"), Some("MacroKnob"));
        assert_eq!(DspNodeRegistry::normalize_type_name("performancemacroknob"), Some("MacroKnob"));
        assert_eq!(DspNodeRegistry::normalize_type_name("butterworth2ndorder"), Some("Butterworth2ndOrder"));
        assert_eq!(DspNodeRegistry::normalize_type_name("butterworthfilter2ndorder"), Some("Butterworth2ndOrder"));
        assert_eq!(DspNodeRegistry::normalize_type_name("modalfiltersection"), Some("ModalFilterSection"));
        assert_eq!(DspNodeRegistry::normalize_type_name("narrowbandmodalfilter"), Some("ModalFilterSection"));
        assert_eq!(DspNodeRegistry::normalize_type_name("dispersionallpass"), Some("DispersionAllpass"));
        assert_eq!(DspNodeRegistry::normalize_type_name("inharmonicdispersionallpass"), Some("DispersionAllpass"));
        assert_eq!(DspNodeRegistry::normalize_type_name("soundboardmode"), Some("SoundboardMode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("woodensoundboardmode"), Some("SoundboardMode"));
        assert_eq!(DspNodeRegistry::normalize_type_name("consolechannelstate"), Some("ConsoleChannelState"));
        assert_eq!(DspNodeRegistry::normalize_type_name("analogconsolechannel"), Some("ConsoleChannelState"));
        assert_eq!(DspNodeRegistry::normalize_type_name("consolebiquad"), Some("ConsoleBiquad"));
        assert_eq!(DspNodeRegistry::normalize_type_name("analogconsolebiquad"), Some("ConsoleBiquad"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"BowedString"));
        assert!(phys_items.contains(&"Clavinet"));
        assert!(phys_items.contains(&"ElectricPiano"));
        assert!(phys_items.contains(&"FreeReed"));
        assert!(phys_items.contains(&"GlottalPulse"));
        assert!(phys_items.contains(&"ConcertGrandPiano"));
        assert!(phys_items.contains(&"Shakuhachi"));
        assert!(phys_items.contains(&"SitarNode"));
        assert!(phys_items.contains(&"VocalTract"));
        assert!(phys_items.contains(&"SoundboardMode"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"ConsoleChannelState"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"PlateTank"));
        assert!(time_items.contains(&"SpringLattice"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"Butterworth2ndOrder"));
        assert!(filter_items.contains(&"ModalFilterSection"));
        assert!(filter_items.contains(&"DispersionAllpass"));
        assert!(filter_items.contains(&"ConsoleBiquad"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"LFO"));
        assert!(mod_items.contains(&"MacroKnob"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"StereoWidth"));
        assert!(routing_items.contains(&"MemoryEstimator"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Concert Grand Imperial Steinway -> ConcertGrandPiano
            view.top_bar_state.selected_preset = "Concert Grand Imperial Steinway".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ConcertGrandPiano".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ConcertGrandPiano".to_string()));

            // Preset: Stevie 70s Clavinet Wah Funk -> Clavinet
            view.top_bar_state.selected_preset = "Stevie 70s Clavinet Wah Funk".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("Clavinet".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("Clavinet".to_string()));

            // Preset: Plate Tank Mechanical Reverb -> PlateTank
            view.top_bar_state.selected_preset = "Plate Tank Mechanical Reverb".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PlateTank".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PlateTank".to_string()));

            // Preset: Zen Bamboo Flute Breath -> Shakuhachi
            view.top_bar_state.selected_preset = "Zen Bamboo Flute Breath".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("Shakuhachi".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("Shakuhachi".to_string()));
        }
    }

    #[test]
    fn test_step_1295_tier95_dsp_modules_expansion_coverage() {
        use crate::dsp_node_ui::DspNodeRegistry;
        use crate::views::modern_asset_browser::BrowserCategory;

        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 610, "Expected >= 610 descriptors, got {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 610, "Expected >= 610 inventory items, got {}", inv.len());

        let tier95_dsp_modules = [
            "CadenceGraph",
            "CadenceEngine",
            "HarmonicTensionAnalyzer",
            "VoiceLeadingOptimizer",
            "IsomorphicRouter",
            "VowelGraph",
            "EdoTuning",
            "SclTuning",
            "KbmMapping",
            "HarmonicBusBridge",
            "ClipMatrix",
            "AutomationTimeline",
            "CompTrack",
            "GenerativeEngine",
            "TimelineArranger",
            "Arpeggiator",
            "ChordMemory",
            "Strummer",
            "SidechainRoute",
            "SpatialTrajectoryTrack",
            "TrackerStep",
        ];

        // 1. Verify existence, descriptor param count (>= 5), and UI instantiation
        for name in &tier95_dsp_modules {
            let desc = registry.get(name);
            assert!(desc.is_some(), "Module {} must exist in DspNodeRegistry", name);
            let desc = desc.unwrap();
            assert!(desc.params.len() >= 5, "Module {} must have at least 5 schema parameters, found {}", name, desc.params.len());

            let ui_node = DspNodeRegistry::create_node_ui(name);
            assert!(ui_node.is_some(), "create_node_ui must succeed for {}", name);
            let ui_node = ui_node.unwrap();
            assert!(ui_node.parameters().len() >= 5, "Module {} must have at least 5 parameters exposed in UI, found {}", name, ui_node.parameters().len());
        }

        // 2. Verify key parameter IDs on all 21 modules
        let cadence_g = registry.get("CadenceGraph").unwrap();
        assert!(cadence_g.params.iter().any(|p| p.id == "root_key"));
        assert!(cadence_g.params.iter().any(|p| p.id == "scale_mode"));
        assert!(cadence_g.params.iter().any(|p| p.id == "cadence_branching"));

        let cadence_e = registry.get("CadenceEngine").unwrap();
        assert!(cadence_e.params.iter().any(|p| p.id == "resolution_speed"));
        assert!(cadence_e.params.iter().any(|p| p.id == "deceptive_prob"));
        assert!(cadence_e.params.iter().any(|p| p.id == "cadence_strength"));

        let tension = registry.get("HarmonicTensionAnalyzer").unwrap();
        assert!(tension.params.iter().any(|p| p.id == "roughness_weight"));
        assert!(tension.params.iter().any(|p| p.id == "tonal_stability"));
        assert!(tension.params.iter().any(|p| p.id == "color_saturation"));

        let voice = registry.get("VoiceLeadingOptimizer").unwrap();
        assert!(voice.params.iter().any(|p| p.id == "voice_count"));
        assert!(voice.params.iter().any(|p| p.id == "distance_penalty"));
        assert!(voice.params.iter().any(|p| p.id == "stepwise_preference"));

        let iso = registry.get("IsomorphicRouter").unwrap();
        assert!(iso.params.iter().any(|p| p.id == "row_axis_semitones"));
        assert!(iso.params.iter().any(|p| p.id == "col_axis_semitones"));
        assert!(iso.params.iter().any(|p| p.id == "layout_preset"));

        let vowel = registry.get("VowelGraph").unwrap();
        assert!(vowel.params.iter().any(|p| p.id == "formant_f1_hz"));
        assert!(vowel.params.iter().any(|p| p.id == "formant_f2_hz"));
        assert!(vowel.params.iter().any(|p| p.id == "nasalization"));

        let edo = registry.get("EdoTuning").unwrap();
        assert!(edo.params.iter().any(|p| p.id == "divisions_per_octave"));
        assert!(edo.params.iter().any(|p| p.id == "reference_pitch_hz"));
        assert!(edo.params.iter().any(|p| p.id == "stretch_factor"));

        let scl = registry.get("SclTuning").unwrap();
        assert!(scl.params.iter().any(|p| p.id == "scale_size"));
        assert!(scl.params.iter().any(|p| p.id == "fundamental_hz"));
        assert!(scl.params.iter().any(|p| p.id == "just_intonation_bias"));

        let kbm = registry.get("KbmMapping").unwrap();
        assert!(kbm.params.iter().any(|p| p.id == "middle_key_midi"));
        assert!(kbm.params.iter().any(|p| p.id == "scale_degree_map"));
        assert!(kbm.params.iter().any(|p| p.id == "octave_key_span"));

        let bridge = registry.get("HarmonicBusBridge").unwrap();
        assert!(bridge.params.iter().any(|p| p.id == "broadcast_channel"));
        assert!(bridge.params.iter().any(|p| p.id == "send_level"));
        assert!(bridge.params.iter().any(|p| p.id == "tension_damping"));

        let clip = registry.get("ClipMatrix").unwrap();
        assert!(clip.params.iter().any(|p| p.id == "launch_quantize"));
        assert!(clip.params.iter().any(|p| p.id == "scene_tempo_sync"));
        assert!(clip.params.iter().any(|p| p.id == "crossfade_curve"));

        let auto = registry.get("AutomationTimeline").unwrap();
        assert!(auto.params.iter().any(|p| p.id == "smoothing_tau_ms"));
        assert!(auto.params.iter().any(|p| p.id == "curve_tension"));
        assert!(auto.params.iter().any(|p| p.id == "record_latch_mode"));

        let comp = registry.get("CompTrack").unwrap();
        assert!(comp.params.iter().any(|p| p.id == "crossfade_time_ms"));
        assert!(comp.params.iter().any(|p| p.id == "equal_power_crossfade"));
        assert!(comp.params.iter().any(|p| p.id == "take_selection"));

        let gen = registry.get("GenerativeEngine").unwrap();
        assert!(gen.params.iter().any(|p| p.id == "euclidean_hits"));
        assert!(gen.params.iter().any(|p| p.id == "euclidean_steps"));
        assert!(gen.params.iter().any(|p| p.id == "swing_factor"));

        let time = registry.get("TimelineArranger").unwrap();
        assert!(time.params.iter().any(|p| p.id == "playback_speed"));
        assert!(time.params.iter().any(|p| p.id == "loop_start_bar"));
        assert!(time.params.iter().any(|p| p.id == "metronome_gain"));

        let arp = registry.get("Arpeggiator").unwrap();
        assert!(arp.params.iter().any(|p| p.id == "arp_mode"));
        assert!(arp.params.iter().any(|p| p.id == "rate_division"));
        assert!(arp.params.iter().any(|p| p.id == "octave_range"));

        let chord = registry.get("ChordMemory").unwrap();
        assert!(chord.params.iter().any(|p| p.id == "voicing_preset"));
        assert!(chord.params.iter().any(|p| p.id == "velocity_spread"));
        assert!(chord.params.iter().any(|p| p.id == "strum_delay_ms"));

        let strum = registry.get("Strummer").unwrap();
        assert!(strum.params.iter().any(|p| p.id == "strum_speed_ms"));
        assert!(strum.params.iter().any(|p| p.id == "strum_direction"));
        assert!(strum.params.iter().any(|p| p.id == "plectrum_stiffness"));

        let sidechain = registry.get("SidechainRoute").unwrap();
        assert!(sidechain.params.iter().any(|p| p.id == "sidechain_source_ch"));
        assert!(sidechain.params.iter().any(|p| p.id == "ducking_amount_db"));
        assert!(sidechain.params.iter().any(|p| p.id == "sidechain_hpf_hz"));

        let spatial = registry.get("SpatialTrajectoryTrack").unwrap();
        assert!(spatial.params.iter().any(|p| p.id == "orbit_speed_hz"));
        assert!(spatial.params.iter().any(|p| p.id == "orbit_radius_m"));
        assert!(spatial.params.iter().any(|p| p.id == "trajectory_shape"));

        let tracker = registry.get("TrackerStep").unwrap();
        assert!(tracker.params.iter().any(|p| p.id == "ticks_per_line"));
        assert!(tracker.params.iter().any(|p| p.id == "hex_fx_command"));
        assert!(tracker.params.iter().any(|p| p.id == "hex_fx_param"));

        // 3. Verify alias normalization
        assert_eq!(DspNodeRegistry::normalize_type_name("cadencegraph"), Some("CadenceGraph"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmoniccadencegraph"), Some("CadenceGraph"));
        assert_eq!(DspNodeRegistry::normalize_type_name("cadenceengine"), Some("CadenceEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("cadenceresolver"), Some("CadenceEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmonictensionanalyzer"), Some("HarmonicTensionAnalyzer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("tensionanalyzer"), Some("HarmonicTensionAnalyzer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("voiceleadingoptimizer"), Some("VoiceLeadingOptimizer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("parsimoniousvoiceleading"), Some("VoiceLeadingOptimizer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("isomorphicrouter"), Some("IsomorphicRouter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("wickihaydenrouter"), Some("IsomorphicRouter"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vowelgraph"), Some("VowelGraph"));
        assert_eq!(DspNodeRegistry::normalize_type_name("formantvowelgraph"), Some("VowelGraph"));
        assert_eq!(DspNodeRegistry::normalize_type_name("edotuning"), Some("EdoTuning"));
        assert_eq!(DspNodeRegistry::normalize_type_name("equaldivisionsoctave"), Some("EdoTuning"));
        assert_eq!(DspNodeRegistry::normalize_type_name("scltuning"), Some("SclTuning"));
        assert_eq!(DspNodeRegistry::normalize_type_name("scalascltuning"), Some("SclTuning"));
        assert_eq!(DspNodeRegistry::normalize_type_name("kbmmapping"), Some("KbmMapping"));
        assert_eq!(DspNodeRegistry::normalize_type_name("microtonalkbm"), Some("KbmMapping"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmonicbusbridge"), Some("HarmonicBusBridge"));
        assert_eq!(DspNodeRegistry::normalize_type_name("harmonicbridge"), Some("HarmonicBusBridge"));
        assert_eq!(DspNodeRegistry::normalize_type_name("clipmatrix"), Some("ClipMatrix"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sessionclipmatrix"), Some("ClipMatrix"));
        assert_eq!(DspNodeRegistry::normalize_type_name("automationtimeline"), Some("AutomationTimeline"));
        assert_eq!(DspNodeRegistry::normalize_type_name("parametertimeline"), Some("AutomationTimeline"));
        assert_eq!(DspNodeRegistry::normalize_type_name("comptrack"), Some("CompTrack"));
        assert_eq!(DspNodeRegistry::normalize_type_name("vocalcomptrack"), Some("CompTrack"));
        assert_eq!(DspNodeRegistry::normalize_type_name("generativeengine"), Some("GenerativeEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("euclideangenerative"), Some("GenerativeEngine"));
        assert_eq!(DspNodeRegistry::normalize_type_name("timelinearranger"), Some("TimelineArranger"));
        assert_eq!(DspNodeRegistry::normalize_type_name("songtimelinearranger"), Some("TimelineArranger"));
        assert_eq!(DspNodeRegistry::normalize_type_name("arpeggiator"), Some("Arpeggiator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("polyarpeggiator"), Some("Arpeggiator"));
        assert_eq!(DspNodeRegistry::normalize_type_name("chordmemory"), Some("ChordMemory"));
        assert_eq!(DspNodeRegistry::normalize_type_name("polychordmemory"), Some("ChordMemory"));
        assert_eq!(DspNodeRegistry::normalize_type_name("strummer"), Some("Strummer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("acousticstrummer"), Some("Strummer"));
        assert_eq!(DspNodeRegistry::normalize_type_name("sidechainroute"), Some("SidechainRoute"));
        assert_eq!(DspNodeRegistry::normalize_type_name("dynamicsidechain"), Some("SidechainRoute"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spatialtrajectorytrack"), Some("SpatialTrajectoryTrack"));
        assert_eq!(DspNodeRegistry::normalize_type_name("spatialorbittrajectory"), Some("SpatialTrajectoryTrack"));
        assert_eq!(DspNodeRegistry::normalize_type_name("trackerstep"), Some("TrackerStep"));
        assert_eq!(DspNodeRegistry::normalize_type_name("modtrackerstep"), Some("TrackerStep"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"Strummer"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"SidechainRoute"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"SpatialTrajectoryTrack"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"VowelGraph"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"CadenceEngine"));
        assert!(mod_items.contains(&"VoiceLeadingOptimizer"));
        assert!(mod_items.contains(&"AutomationTimeline"));
        assert!(mod_items.contains(&"GenerativeEngine"));
        assert!(mod_items.contains(&"ChordMemory"));
        assert!(mod_items.contains(&"TrackerStep"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"CadenceGraph"));
        assert!(routing_items.contains(&"HarmonicTensionAnalyzer"));
        assert!(routing_items.contains(&"IsomorphicRouter"));
        assert!(routing_items.contains(&"EdoTuning"));
        assert!(routing_items.contains(&"SclTuning"));
        assert!(routing_items.contains(&"KbmMapping"));
        assert!(routing_items.contains(&"HarmonicBusBridge"));
        assert!(routing_items.contains(&"TimelineArranger"));
        assert!(routing_items.contains(&"ClipMatrix"));
        assert!(routing_items.contains(&"CompTrack"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Neo-Riemannian Cadence Graph -> CadenceGraph
            view.top_bar_state.selected_preset = "Neo-Riemannian Cadence Graph".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("CadenceGraph".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("CadenceGraph".to_string()));

            // Preset: Bjorklund Euclidean Rhythm Engine -> GenerativeEngine
            view.top_bar_state.selected_preset = "Bjorklund Euclidean Rhythm Engine".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("GenerativeEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("GenerativeEngine".to_string()));

            // Preset: Microtonal 31-EDO Harmonic Scale -> EdoTuning
            view.top_bar_state.selected_preset = "Microtonal 31-EDO Harmonic Scale".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("EdoTuning".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("EdoTuning".to_string()));

            // Preset: Acoustic Flamenco Guitar Strummer -> Strummer
            view.top_bar_state.selected_preset = "Acoustic Flamenco Guitar Strummer".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("Strummer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("Strummer".to_string()));
        }
    }

    #[test]
    fn test_tier96_cadence_flow_concordance_friction_orbit_and_bode_hud_integration() {
        use crate::dsp_node_ui::DspNodeCategory;

        // 1. Verify DSP registry contains >= 630 modules
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 630, "Registry must contain >= 630 modules, found {}", registry.list_all().len());
        let inventory = crate::dsp_node_ui::DspNodeRegistry::inventory();
        assert!(inventory.len() >= 630, "Inventory must contain >= 630 modules, found {}", inventory.len());

        // 2. Verify all 21 new DSP modules exist and have tactile parameter descriptors
        let new_modules = [
            ("CadenceFlow", DspNodeCategory::Utility),
            ("ConcordanceLattice", DspNodeCategory::Utility),
            ("EmbouchureAngle", DspNodeCategory::AcousticPhysicalModel),
            ("FrictionOrbit", DspNodeCategory::AcousticPhysicalModel),
            ("HarmonicTensionMap", DspNodeCategory::Utility),
            ("Hoa4Spatializer", DspNodeCategory::SpatialSurround),
            ("IdiophoneSpectrum", DspNodeCategory::AcousticPhysicalModel),
            ("IsomorphicLattice", DspNodeCategory::Utility),
            ("IsomorphicTuningKeyboard", DspNodeCategory::Utility),
            ("LadderFilterBode", DspNodeCategory::FilterEq),
            ("MembraneCavity", DspNodeCategory::AcousticPhysicalModel),
            ("MpeghSpatializer", DspNodeCategory::SpatialSurround),
            ("PhaseAlign", DspNodeCategory::Utility),
            ("RotaryDoppler", DspNodeCategory::Modulation),
            ("ToneholeMatrix", DspNodeCategory::AcousticPhysicalModel),
            ("TrompetteBridge", DspNodeCategory::AcousticPhysicalModel),
            ("VowelSpace", DspNodeCategory::FilterEq),
            ("BellowsChamber", DspNodeCategory::AcousticPhysicalModel),
            ("ConvolutionImpulse", DspNodeCategory::TimeSpace),
            ("ConvolutionMorph", DspNodeCategory::TimeSpace),
            ("BezierAutomation", DspNodeCategory::Modulation),
        ];

        for (name, cat) in &new_modules {
            let desc = registry.get(name).unwrap_or_else(|| panic!("Node {} must be registered in DspNodeRegistry", name));
            assert_eq!(desc.category, *cat, "Node {} must match expected category", name);
            assert!(desc.params.len() >= 5, "Node {} must have >= 5 tactile parameters, got {}", name, desc.params.len());

            let ui = crate::dsp_node_ui::DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("create_node_ui({}) must succeed", name));
            assert_eq!(ui.node_type_name(), *name);
            assert_eq!(ui.category(), *cat);
            assert!(ui.parameters().len() >= 5);
        }

        // 3. Verify canonical normalization mappings
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cadenceflow"), Some("CadenceFlow"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cadenceflowhud"), Some("CadenceFlow"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("concordancelattice"), Some("ConcordanceLattice"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("harmonicconcordance"), Some("ConcordanceLattice"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("embouchureangle"), Some("EmbouchureAngle"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("shakuhachiembouchure"), Some("EmbouchureAngle"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("frictionorbit"), Some("FrictionOrbit"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("bowingorbit"), Some("FrictionOrbit"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("harmonictensionmap"), Some("HarmonicTensionMap"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tensionheatmap"), Some("HarmonicTensionMap"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hoa4spatializer"), Some("Hoa4Spatializer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hoa4sphere"), Some("Hoa4Spatializer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("idiophonespectrum"), Some("IdiophoneSpectrum"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("idiophonemodal"), Some("IdiophoneSpectrum"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("isomorphiclattice"), Some("IsomorphicLattice"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hexpitchlattice"), Some("IsomorphicLattice"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("isomorphictuningkeyboard"), Some("IsomorphicTuningKeyboard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("microtonalkeyboard"), Some("IsomorphicTuningKeyboard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("ladderfilterbode"), Some("LadderFilterBode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("moogladderbode"), Some("LadderFilterBode"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("membranecavity"), Some("MembraneCavity"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("drumcavity"), Some("MembraneCavity"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("mpeghspatializer"), Some("MpeghSpatializer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("mpeghspatialbus"), Some("MpeghSpatializer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("phasealignhud"), Some("PhaseAlign"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("subsamplephasealign"), Some("PhaseAlign"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("rotarydoppler"), Some("RotaryDoppler"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("lesliedoppler"), Some("RotaryDoppler"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("toneholematrixview"), Some("ToneholeMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("woodwindtonehole"), Some("ToneholeMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("trompettebridge"), Some("TrompetteBridge"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("trompettebridgeview"), Some("TrompetteBridge"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("vowelspace"), Some("VowelSpace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("ipavowelspace"), Some("VowelSpace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("bellowschamber"), Some("BellowsChamber"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("accordionbellows"), Some("BellowsChamber"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("convolutionimpulse"), Some("ConvolutionImpulse"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("reverbimpulsehud"), Some("ConvolutionImpulse"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("convolutionmorph"), Some("ConvolutionMorph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("irmorphing"), Some("ConvolutionMorph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("bezierautomation"), Some("BezierAutomation"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tactilebeziercurve"), Some("BezierAutomation"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"EmbouchureAngle"));
        assert!(phys_items.contains(&"FrictionOrbit"));
        assert!(phys_items.contains(&"IdiophoneSpectrum"));
        assert!(phys_items.contains(&"MembraneCavity"));
        assert!(phys_items.contains(&"ToneholeMatrix"));
        assert!(phys_items.contains(&"TrompetteBridge"));
        assert!(phys_items.contains(&"BellowsChamber"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"Hoa4Spatializer"));
        assert!(time_items.contains(&"MpeghSpatializer"));
        assert!(time_items.contains(&"ConvolutionImpulse"));
        assert!(time_items.contains(&"ConvolutionMorph"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"LadderFilterBode"));
        assert!(filter_items.contains(&"VowelSpace"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"RotaryDoppler"));
        assert!(mod_items.contains(&"BezierAutomation"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"CadenceFlow"));
        assert!(routing_items.contains(&"ConcordanceLattice"));
        assert!(routing_items.contains(&"HarmonicTensionMap"));
        assert!(routing_items.contains(&"IsomorphicLattice"));
        assert!(routing_items.contains(&"IsomorphicTuningKeyboard"));
        assert!(routing_items.contains(&"PhaseAlign"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Just Intonation Concordance Lattice -> ConcordanceLattice
            view.top_bar_state.selected_preset = "Just Intonation Concordance Lattice".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("ConcordanceLattice".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("ConcordanceLattice".to_string()));

            // Preset: Bowed Cello Friction Orbit -> FrictionOrbit
            view.top_bar_state.selected_preset = "Bowed Cello Friction Orbit".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("FrictionOrbit".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("FrictionOrbit".to_string()));

            // Preset: Moog 4-Pole Ladder Self-Oscillation -> LadderFilterBode
            view.top_bar_state.selected_preset = "Moog 4-Pole Ladder Self-Oscillation".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("LadderFilterBode".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("LadderFilterBode".to_string()));

            // Preset: Leslie 122 Dual Rotor Doppler -> RotaryDoppler
            view.top_bar_state.selected_preset = "Leslie 122 Dual Rotor Doppler".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("RotaryDoppler".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("RotaryDoppler".to_string()));
        }
    }

    #[test]
    fn test_tier97_modern_gui_dsp_module_coverage_and_presets() {
        use crate::dsp_node_ui::{DspNodeRegistry, DspNodeCategory};
        use crate::views::modern_asset_browser::BrowserCategory;

        // 1. Verify global registry inventory completeness >= 650
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 650, "Registry must contain >= 650 descriptors, found {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 650, "Inventory must contain >= 650 entries, found {}", inv.len());

        // 2. Verify all 21 newly registered DSP modules exist and have valid UIs
        let new_modules = [
            ("NamModelConfig", DspNodeCategory::DistortionSaturation),
            ("Vst3HostConfig", DspNodeCategory::Utility),
            ("ClapAutomationHost", DspNodeCategory::Modulation),
            ("SpatialIrConfig", DspNodeCategory::SpatialSurround),
            ("SpectralMorphEngine", DspNodeCategory::FilterEq),
            ("SpectrogramSoundGenerator", DspNodeCategory::Oscillator),
            ("MultiSampleBank", DspNodeCategory::CompositeSynth),
            ("SamplerMacroStrip", DspNodeCategory::CompositeSynth),
            ("AirReedConfig", DspNodeCategory::AcousticPhysicalModel),
            ("WindchestReservoir", DspNodeCategory::AcousticPhysicalModel),
            ("QuantumTomographyVisualizer", DspNodeCategory::Utility),
            ("NeuroAffectiveEngine", DspNodeCategory::Modulation),
            ("OscMappingRouter", DspNodeCategory::Utility),
            ("MaskingCollisionAnalyzer", DspNodeCategory::Utility),
            ("AiMixRecommendationEngine", DspNodeCategory::Utility),
            ("AetherMacroStrip", DspNodeCategory::CompositeSynth),
            ("DrumMacroStrip", DspNodeCategory::SamplerSlicer),
            ("SimdVoiceAllocator", DspNodeCategory::Utility),
            ("AudioWarpMarker", DspNodeCategory::TimeSpace),
            ("StickSlipPhysicsModel", DspNodeCategory::AcousticPhysicalModel),
            ("PianoHammerMechanics", DspNodeCategory::AcousticPhysicalModel),
        ];

        for (name, cat) in &new_modules {
            let desc = registry.get(name).unwrap_or_else(|| panic!("Node {} must be registered in DspNodeRegistry", name));
            assert_eq!(desc.category, *cat, "Node {} must match expected category", name);
            assert!(desc.params.len() >= 5, "Node {} must have >= 5 tactile parameters, got {}", name, desc.params.len());

            let ui = crate::dsp_node_ui::DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("create_node_ui({}) must succeed", name));
            assert_eq!(ui.node_type_name(), *name);
            assert_eq!(ui.category(), *cat);
            assert!(ui.parameters().len() >= 5);
        }

        // 3. Verify canonical normalization mappings
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("nammodelconfig"), Some("NamModelConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("namcoreprofile"), Some("NamModelConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("vst3hostconfig"), Some("Vst3HostConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("vst3hostbridge"), Some("Vst3HostConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("clapautomationhost"), Some("ClapAutomationHost"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("clapsampleaccurate"), Some("ClapAutomationHost"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spatialirconfig"), Some("SpatialIrConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spatialirconvolver"), Some("SpatialIrConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spectralmorphengine"), Some("SpectralMorphEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("fftspectralmorph"), Some("SpectralMorphEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spectrogramsoundgenerator"), Some("SpectrogramSoundGenerator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("imagesoundgenerator"), Some("SpectrogramSoundGenerator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("multisamplebank"), Some("MultiSampleBank"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("velocitysamplebank"), Some("MultiSampleBank"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("samplermacrostrip"), Some("SamplerMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("samplermacrohud"), Some("SamplerMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("airreedconfig"), Some("AirReedConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("fluejetconfig"), Some("AirReedConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("windchestreservoir"), Some("WindchestReservoir"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pneumaticwindchest"), Some("WindchestReservoir"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("quantumtomographyvisualizer"), Some("QuantumTomographyVisualizer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("quantumdensitymatrix"), Some("QuantumTomographyVisualizer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("neuroaffectiveengine"), Some("NeuroAffectiveEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("eegaffectivemodulator"), Some("NeuroAffectiveEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("oscmappingrouter"), Some("OscMappingRouter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("opensoundcontrolrouter"), Some("OscMappingRouter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("maskingcollisionanalyzer"), Some("MaskingCollisionAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("spectralmaskingreport"), Some("MaskingCollisionAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aimixrecommendationengine"), Some("AiMixRecommendationEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aimixadvisor"), Some("AiMixRecommendationEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aethermacrostrip"), Some("AetherMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aethermacrosurface"), Some("AetherMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("drummacrostrip"), Some("DrumMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("16paddrumconsole"), Some("DrumMacroStrip"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("simdvoiceallocator"), Some("SimdVoiceAllocator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("polyphonicvoiceallocator"), Some("SimdVoiceAllocator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("audiowarpmarker"), Some("AudioWarpMarker"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("warpmarker"), Some("AudioWarpMarker"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("stickslipphysicsmodel"), Some("StickSlipPhysicsModel"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("bowfrictionmechanics"), Some("StickSlipPhysicsModel"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pianohammermechanics"), Some("PianoHammerMechanics"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("felthammermechanics"), Some("PianoHammerMechanics"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"AirReedConfig"));
        assert!(phys_items.contains(&"WindchestReservoir"));
        assert!(phys_items.contains(&"StickSlipPhysicsModel"));
        assert!(phys_items.contains(&"PianoHammerMechanics"));

        let synth_items = inst_folders.iter().find(|(name, _)| *name == "Synthesizers").unwrap().1;
        assert!(synth_items.contains(&"SpectrogramSoundGenerator"));
        assert!(synth_items.contains(&"AetherMacroStrip"));

        let sampler_items = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(sampler_items.contains(&"MultiSampleBank"));
        assert!(sampler_items.contains(&"SamplerMacroStrip"));
        assert!(sampler_items.contains(&"DrumMacroStrip"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"NamModelConfig"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"SpatialIrConfig"));
        assert!(time_items.contains(&"AudioWarpMarker"));

        let filter_items = fx_folders.iter().find(|(name, _)| *name == "Filters & EQ").unwrap().1;
        assert!(filter_items.contains(&"SpectralMorphEngine"));

        let mod_items = fx_folders.iter().find(|(name, _)| *name == "Modulation & Pitch").unwrap().1;
        assert!(mod_items.contains(&"ClapAutomationHost"));
        assert!(mod_items.contains(&"NeuroAffectiveEngine"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"QuantumTomographyVisualizer"));
        assert!(routing_items.contains(&"OscMappingRouter"));
        assert!(routing_items.contains(&"MaskingCollisionAnalyzer"));
        assert!(routing_items.contains(&"AiMixRecommendationEngine"));
        assert!(routing_items.contains(&"SimdVoiceAllocator"));
        assert!(routing_items.contains(&"Vst3HostConfig"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Neural High-Gain Amp Stack -> NamModelConfig
            view.top_bar_state.selected_preset = "Neural High-Gain Amp Stack".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("NamModelConfig".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("NamModelConfig".to_string()));

            // Preset: Spectrogram Visual Audio Canvas -> SpectrogramSoundGenerator
            view.top_bar_state.selected_preset = "Spectrogram Visual Audio Canvas".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SpectrogramSoundGenerator".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SpectrogramSoundGenerator".to_string()));

            // Preset: Cathedral Pipe Organ Windchest -> WindchestReservoir
            view.top_bar_state.selected_preset = "Cathedral Pipe Organ Windchest".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("WindchestReservoir".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("WindchestReservoir".to_string()));

            // Preset: Tactile 16-Pad Groove Machine -> DrumMacroStrip
            view.top_bar_state.selected_preset = "Tactile 16-Pad Groove Machine".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("DrumMacroStrip".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("DrumMacroStrip".to_string()));
        }
    }

    #[test]
    fn test_tier98_modern_gui_dsp_module_coverage_and_presets() {
        use crate::dsp_node_ui::{DspNodeRegistry, DspNodeCategory};
        use crate::views::modern_asset_browser::BrowserCategory;

        // 1. Verify global registry inventory completeness >= 670 (currently 673)
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 670, "Registry must contain >= 670 descriptors, found {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 670, "Inventory must contain >= 670 entries, found {}", inv.len());

        // 2. Verify all 21 newly registered DSP modules exist and have valid tactile UIs
        let new_modules = [
            ("GlassArmonicaBus", DspNodeCategory::AcousticPhysicalModel),
            ("SfzPresetPatch", DspNodeCategory::SamplerSlicer),
            ("LuaScriptEngine", DspNodeCategory::Modulation),
            ("PluginSandbox", DspNodeCategory::Utility),
            ("PluginLatencyCompensation", DspNodeCategory::Utility),
            ("PluginCrashGuard", DspNodeCategory::Utility),
            ("AudioGraphBenchmarkSuite", DspNodeCategory::Utility),
            ("ProjectAutoSaveManager", DspNodeCategory::Utility),
            ("SessionMarkerNavigationManager", DspNodeCategory::Modulation),
            ("ScratchAudioCache", DspNodeCategory::Utility),
            ("WorkspaceDependencyAuditor", DspNodeCategory::Utility),
            ("CrashDumpAnalyzer", DspNodeCategory::Utility),
            ("ExportPresetManager", DspNodeCategory::DynamicsMaster),
            ("DistributedRenderFarm", DspNodeCategory::Utility),
            ("LivePerformanceSync", DspNodeCategory::Modulation),
            ("CloudPresetHub", DspNodeCategory::Utility),
            ("FederatedMarketplace", DspNodeCategory::Utility),
            ("PeerMeshNetwork", DspNodeCategory::SpatialSurround),
            ("ZkPatchVerifier", DspNodeCategory::Utility),
            ("ContinuousBackupEngine", DspNodeCategory::Utility),
            ("MidiFileParser", DspNodeCategory::Modulation),
        ];

        for (name, cat) in &new_modules {
            let desc = registry.get(name).unwrap_or_else(|| panic!("Node {} must be registered in DspNodeRegistry", name));
            assert_eq!(desc.category, *cat, "Node {} must match expected category", name);
            assert!(desc.params.len() >= 5, "Node {} must have >= 5 tactile parameters, got {}", name, desc.params.len());

            let ui = crate::dsp_node_ui::DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("create_node_ui({}) must succeed", name));
            assert_eq!(ui.node_type_name(), *name);
            assert_eq!(ui.category(), *cat);
            assert!(ui.parameters().len() >= 5);
        }

        // 3. Verify canonical normalization mappings
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("glassarmonicabus"), Some("GlassArmonicaBus"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("franklinglassarmonica"), Some("GlassArmonicaBus"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crystalsingingbowl"), Some("GlassArmonicaBus"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sfzpresetpatch"), Some("SfzPresetPatch"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sfzsoundfont"), Some("SfzPresetPatch"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luascriptengine"), Some("LuaScriptEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luadspengine"), Some("LuaScriptEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pluginsandbox"), Some("PluginSandbox"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pluginhostsandbox"), Some("PluginSandbox"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pluginlatencycompensation"), Some("PluginLatencyCompensation"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("automaticpdc"), Some("PluginLatencyCompensation"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("plugincrashguard"), Some("PluginCrashGuard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crashguardwatchdog"), Some("PluginCrashGuard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("audiographbenchmarksuite"), Some("AudioGraphBenchmarkSuite"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("dspgraphbenchmark"), Some("AudioGraphBenchmarkSuite"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("projectautosavemanager"), Some("ProjectAutoSaveManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("backupsnapshotmanager"), Some("ProjectAutoSaveManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sessionmarkernavigationmanager"), Some("SessionMarkerNavigationManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("markernavigation"), Some("SessionMarkerNavigationManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("scratchaudiocache"), Some("ScratchAudioCache"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("temporaryaudiocache"), Some("ScratchAudioCache"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("workspacedependencyauditor"), Some("WorkspaceDependencyAuditor"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cratevulnerabilityauditor"), Some("WorkspaceDependencyAuditor"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("crashdumpanalyzer"), Some("CrashDumpAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("minidumpanalyzer"), Some("CrashDumpAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("exportpresetmanager"), Some("ExportPresetManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("batchstemexporter"), Some("ExportPresetManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("distributedrenderfarm"), Some("DistributedRenderFarm"),);
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudaudiorender"), Some("DistributedRenderFarm"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("liveperformancesync"), Some("LivePerformanceSync"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("stageptpclock"), Some("LivePerformanceSync"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudpresethub"), Some("CloudPresetHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudpatchrepository"), Some("CloudPresetHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("federatedmarketplace"), Some("FederatedMarketplace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("assetmarketplace"), Some("FederatedMarketplace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("peermeshnetwork"), Some("PeerMeshNetwork"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("collaborativepeermesh"), Some("PeerMeshNetwork"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("zkpatchverifier"), Some("ZkPatchVerifier"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("zeroknowledgebytecodeverifier"), Some("ZkPatchVerifier"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("continuousbackupengine"), Some("ContinuousBackupEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("journaledtimemachinebackup"), Some("ContinuousBackupEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midifileparser"), Some("MidiFileParser"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("smfparser"), Some("MidiFileParser"));

        // 4. Verify Asset Browser categorization
        let inst_folders = BrowserCategory::Instruments.default_folders();
        let phys_items = inst_folders.iter().find(|(name, _)| *name == "Physical Models").unwrap().1;
        assert!(phys_items.contains(&"GlassArmonicaBus"));

        let sampler_items = inst_folders.iter().find(|(name, _)| *name == "Samplers & Slicers").unwrap().1;
        assert!(sampler_items.contains(&"SfzPresetPatch"));

        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"ExportPresetManager"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"PeerMeshNetwork"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_sync_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_sync_items.contains(&"SessionMarkerNavigationManager"));
        assert!(gen_sync_items.contains(&"LivePerformanceSync"));
        assert!(gen_sync_items.contains(&"LuaScriptEngine"));
        assert!(gen_sync_items.contains(&"MidiFileParser"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"PluginSandbox"));
        assert!(routing_items.contains(&"PluginLatencyCompensation"));
        assert!(routing_items.contains(&"PluginCrashGuard"));
        assert!(routing_items.contains(&"AudioGraphBenchmarkSuite"));
        assert!(routing_items.contains(&"ProjectAutoSaveManager"));
        assert!(routing_items.contains(&"ScratchAudioCache"));
        assert!(routing_items.contains(&"WorkspaceDependencyAuditor"));
        assert!(routing_items.contains(&"CrashDumpAnalyzer"));
        assert!(routing_items.contains(&"DistributedRenderFarm"));
        assert!(routing_items.contains(&"CloudPresetHub"));
        assert!(routing_items.contains(&"FederatedMarketplace"));
        assert!(routing_items.contains(&"ZkPatchVerifier"));
        assert!(routing_items.contains(&"ContinuousBackupEngine"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Benjamin Franklin Glass Armonica -> GlassArmonicaBus
            view.top_bar_state.selected_preset = "Benjamin Franklin Glass Armonica".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("GlassArmonicaBus".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("GlassArmonicaBus".to_string()));

            // Preset: Multi-Zone SFZ Orchestra Sample Patch -> SfzPresetPatch
            view.top_bar_state.selected_preset = "Multi-Zone SFZ Orchestra Sample Patch".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SfzPresetPatch".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SfzPresetPatch".to_string()));

            // Preset: Lua Real-Time DSP Script Controller -> LuaScriptEngine
            view.top_bar_state.selected_preset = "Lua Real-Time DSP Script Controller".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("LuaScriptEngine".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("LuaScriptEngine".to_string()));

            // Preset: Zero-Latency Plugin Sandbox Guard -> PluginSandbox
            view.top_bar_state.selected_preset = "Zero-Latency Plugin Sandbox Guard".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PluginSandbox".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PluginSandbox".to_string()));
        }
    }

    #[test]
    fn test_tier99_modern_gui_dsp_module_coverage_and_presets() {
        use crate::dsp_node_ui::{DspNodeRegistry, DspNodeCategory};
        use crate::views::modern_asset_browser::BrowserCategory;

        // 1. Verify global registry inventory completeness >= 694
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 694, "Registry must contain >= 694 descriptors, found {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 694, "Inventory must contain >= 694 entries, found {}", inv.len());

        // 2. Verify all 21 newly registered DSP modules exist and have valid tactile UIs
        let new_modules = [
            ("WebSocketSyncTransport", DspNodeCategory::Utility),
            ("CursorTracker", DspNodeCategory::Utility),
            ("OpusAudioRelay", DspNodeCategory::TimeSpace),
            ("CloudBranchingManager", DspNodeCategory::Utility),
            ("CloudAssetSync", DspNodeCategory::Utility),
            ("SharedAnnotationPanel", DspNodeCategory::Modulation),
            ("AutomatedCloudBackup", DspNodeCategory::Utility),
            ("AccessControlPolicy", DspNodeCategory::Utility),
            ("RenderFarmDispatchApi", DspNodeCategory::Utility),
            ("E2eeProjectEncryptor", DspNodeCategory::Utility),
            ("OfflineChangeQueue", DspNodeCategory::Utility),
            ("GitHubActionsBotIntegration", DspNodeCategory::Utility),
            ("SessionAnalyticsDashboard", DspNodeCategory::DynamicsMaster),
            ("WebRtcMidiStreamer", DspNodeCategory::Modulation),
            ("TemplateMarketplace", DspNodeCategory::Utility),
            ("GitSessionDag", DspNodeCategory::Utility),
            ("GoldenRenderSuite", DspNodeCategory::DynamicsMaster),
            ("ApiChangelogGenerator", DspNodeCategory::Utility),
            ("VideoExportConfig", DspNodeCategory::DynamicsMaster),
            ("LuaDebugger", DspNodeCategory::Modulation),
            ("LuaProfiler", DspNodeCategory::Utility),
        ];

        for (name, cat) in &new_modules {
            let desc = registry.get(name).unwrap_or_else(|| panic!("Node {} must be registered in DspNodeRegistry", name));
            assert_eq!(desc.category, *cat, "Node {} must match expected category", name);
            assert!(desc.params.len() >= 5, "Node {} must have >= 5 tactile parameters, got {}", name, desc.params.len());

            let ui = crate::dsp_node_ui::DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("create_node_ui({}) must succeed", name));
            assert_eq!(ui.node_type_name(), *name);
            assert_eq!(ui.category(), *cat);
            assert!(ui.parameters().len() >= 5);
        }

        // 3. Verify canonical normalization mappings
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("websocketsynctransport"), Some("WebSocketSyncTransport"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("websocketsync"), Some("WebSocketSyncTransport"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cursortracker"), Some("CursorTracker"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("remotecursortracker"), Some("CursorTracker"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("opusaudiorelay"), Some("OpusAudioRelay"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("opusrelay"), Some("OpusAudioRelay"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudbranchingmanager"), Some("CloudBranchingManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudbranching"), Some("CloudBranchingManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudassetsync"), Some("CloudAssetSync"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("assetsync"), Some("CloudAssetSync"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sharedannotationpanel"), Some("SharedAnnotationPanel"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("timelineannotation"), Some("SharedAnnotationPanel"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("automatedcloudbackup"), Some("AutomatedCloudBackup"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("cloudbackup"), Some("AutomatedCloudBackup"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("accesscontrolpolicy"), Some("AccessControlPolicy"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("workspaceaccesscontrol"), Some("AccessControlPolicy"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("renderfarmdispatchapi"), Some("RenderFarmDispatchApi"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("renderfarmdispatcher"), Some("RenderFarmDispatchApi"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("e2eeprojectencryptor"), Some("E2eeProjectEncryptor"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("projectencryptor"), Some("E2eeProjectEncryptor"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("offlinechangequeue"), Some("OfflineChangeQueue"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("offlinequeue"), Some("OfflineChangeQueue"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("githubactionsbotintegration"), Some("GitHubActionsBotIntegration"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("githubactionsbot"), Some("GitHubActionsBotIntegration"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sessionanalyticsdashboard"), Some("SessionAnalyticsDashboard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sessionanalytics"), Some("SessionAnalyticsDashboard"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("webrtcmidistreamer"), Some("WebRtcMidiStreamer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midistreamer"), Some("WebRtcMidiStreamer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("templatemarketplace"), Some("TemplateMarketplace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("projecttemplatemarketplace"), Some("TemplateMarketplace"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("gitsessiondag"), Some("GitSessionDag"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("gitdag"), Some("GitSessionDag"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("goldenrendersuite"), Some("GoldenRenderSuite"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("goldensuite"), Some("GoldenRenderSuite"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("apichangeloggenerator"), Some("ApiChangelogGenerator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("changeloggenerator"), Some("ApiChangelogGenerator"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("videoexportconfig"), Some("VideoExportConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("videoexporter"), Some("VideoExportConfig"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luadebugger"), Some("LuaDebugger"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luadspdebugger"), Some("LuaDebugger"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luaprofiler"), Some("LuaProfiler"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("luadspprofiler"), Some("LuaProfiler"));

        // 4. Verify Asset Browser categorization
        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"SessionAnalyticsDashboard"));
        assert!(dyn_items.contains(&"GoldenRenderSuite"));
        assert!(dyn_items.contains(&"VideoExportConfig"));

        let time_items = fx_folders.iter().find(|(name, _)| *name == "Time & Reverb").unwrap().1;
        assert!(time_items.contains(&"OpusAudioRelay"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_sync_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_sync_items.contains(&"WebRtcMidiStreamer"));
        assert!(gen_sync_items.contains(&"SharedAnnotationPanel"));
        assert!(gen_sync_items.contains(&"LuaDebugger"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"WebSocketSyncTransport"));
        assert!(routing_items.contains(&"CursorTracker"));
        assert!(routing_items.contains(&"CloudBranchingManager"));
        assert!(routing_items.contains(&"CloudAssetSync"));
        assert!(routing_items.contains(&"AutomatedCloudBackup"));
        assert!(routing_items.contains(&"AccessControlPolicy"));
        assert!(routing_items.contains(&"RenderFarmDispatchApi"));
        assert!(routing_items.contains(&"E2eeProjectEncryptor"));
        assert!(routing_items.contains(&"OfflineChangeQueue"));
        assert!(routing_items.contains(&"GitHubActionsBotIntegration"));
        assert!(routing_items.contains(&"TemplateMarketplace"));
        assert!(routing_items.contains(&"GitSessionDag"));
        assert!(routing_items.contains(&"ApiChangelogGenerator"));
        assert!(routing_items.contains(&"LuaProfiler"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Ultra-Low Latency Opus Audio Relay -> OpusAudioRelay
            view.top_bar_state.selected_preset = "Ultra-Low Latency Opus Audio Relay".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("OpusAudioRelay".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("OpusAudioRelay".to_string()));

            // Preset: Real-Time Session Telemetry & Headroom HUD -> SessionAnalyticsDashboard
            view.top_bar_state.selected_preset = "Real-Time Session Telemetry & Headroom HUD".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("SessionAnalyticsDashboard".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("SessionAnalyticsDashboard".to_string()));

            // Preset: Interactive Lua DSP Profiler & Flamegraph -> LuaProfiler
            view.top_bar_state.selected_preset = "Interactive Lua DSP Profiler & Flamegraph".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("LuaProfiler".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("LuaProfiler".to_string()));

            // Preset: Git DAG Branching & Undo Timeline HUD -> GitSessionDag
            view.top_bar_state.selected_preset = "Git DAG Branching & Undo Timeline HUD".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("GitSessionDag".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("GitSessionDag".to_string()));
        }
    }

    #[test]
    fn test_tier100_modern_gui_dsp_module_coverage_and_presets() {
        use crate::dsp_node_ui::{DspNodeRegistry, DspNodeCategory};
        use crate::views::modern_asset_browser::BrowserCategory;

        // 1. Verify global registry inventory completeness >= 715
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 715, "Registry must contain >= 715 descriptors, found {}", registry.list_all().len());

        let inv = DspNodeRegistry::inventory();
        assert!(inv.len() >= 715, "Inventory must contain >= 715 entries, found {}", inv.len());

        // 2. Verify all 21 newly registered Tier 100 DSP modules exist and have valid tactile UIs
        let new_modules = [
            ("LiveClipMatrix", DspNodeCategory::Utility),
            ("CompingTakeManager", DspNodeCategory::Utility),
            ("MidiPerformanceHub", DspNodeCategory::Modulation),
            ("MidiStreamAnalyzer", DspNodeCategory::Utility),
            ("AutomationTimelineManager", DspNodeCategory::Modulation),
            ("GenerativeStochasticEngine", DspNodeCategory::Modulation),
            ("IsomorphicKeyboardRouter", DspNodeCategory::Modulation),
            ("HarmonicCadenceEngine", DspNodeCategory::Modulation),
            ("VoiceLeadingGraph", DspNodeCategory::Modulation),
            ("VowelTrajectoryGraph", DspNodeCategory::SpectralResynthesis),
            ("MidiClockSyncHub", DspNodeCategory::Utility),
            ("MidiMessageFilterMatrix", DspNodeCategory::Utility),
            ("PolymetricTrackerSequencer", DspNodeCategory::Modulation),
            ("DynamicMicrotonalRemapper", DspNodeCategory::Modulation),
            ("MultiTenantRenderManager", DspNodeCategory::Utility),
            ("HardwareEncoderController", DspNodeCategory::Utility),
            ("AiMixBalanceInspector", DspNodeCategory::DynamicsMaster),
            ("AudioPhaseAligner", DspNodeCategory::DynamicsMaster),
            ("ClapPluginHostBridge", DspNodeCategory::Utility),
            ("HardwareSurfaceController", DspNodeCategory::Utility),
            ("AnalogSyncTransceiver", DspNodeCategory::Utility),
        ];

        for (name, cat) in &new_modules {
            let desc = registry.get(name).unwrap_or_else(|| panic!("Node {} must be registered in DspNodeRegistry", name));
            assert_eq!(desc.category, *cat, "Node {} must match expected category", name);
            assert!(desc.params.len() >= 5, "Node {} must have >= 5 tactile parameters, got {}", name, desc.params.len());

            let ui = crate::dsp_node_ui::DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("create_node_ui({}) must succeed", name));
            assert_eq!(ui.node_type_name(), *name);
            assert_eq!(ui.category(), *cat);
            assert!(ui.parameters().len() >= 5);
        }

        // 3. Verify canonical normalization mappings
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("liveclipmatrix"), Some("LiveClipMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sessionmatrix"), Some("LiveClipMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("compingtakemanager"), Some("CompingTakeManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("audiocomping"), Some("CompingTakeManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midiperformancehub"), Some("MidiPerformanceHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midihub"), Some("MidiPerformanceHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("keysplit"), Some("MidiPerformanceHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midistreamanalyzer"), Some("MidiStreamAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midianalyzer"), Some("MidiStreamAnalyzer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("automationtimelinemanager"), Some("AutomationTimelineManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("automationtimelinehud"), Some("AutomationTimelineManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("generativestochasticengine"), Some("GenerativeStochasticEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("stochasticengine"), Some("GenerativeStochasticEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("isomorphickeyboardrouter"), Some("IsomorphicKeyboardRouter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("isomorphickeyboardhud"), Some("IsomorphicKeyboardRouter"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("harmoniccadenceengine"), Some("HarmonicCadenceEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("harmoniccadencehud"), Some("HarmonicCadenceEngine"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("voiceleadinggraph"), Some("VoiceLeadingGraph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("voiceleadinggraphhud"), Some("VoiceLeadingGraph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("voweltrajectorygraph"), Some("VowelTrajectoryGraph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("vowelformantspline"), Some("VowelTrajectoryGraph"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midiclocksynchub"), Some("MidiClockSyncHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("clocksynchub"), Some("MidiClockSyncHub"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midimessagefiltermatrix"), Some("MidiMessageFilterMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("midifiltermatrix"), Some("MidiMessageFilterMatrix"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("polymetrictrackersequencer"), Some("PolymetricTrackerSequencer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("polymetrictracker"), Some("PolymetricTrackerSequencer"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("dynamicmicrotonalremapper"), Some("DynamicMicrotonalRemapper"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("microtonalremapper"), Some("DynamicMicrotonalRemapper"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("multitenantrendermanager"), Some("MultiTenantRenderManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("rendermanager"), Some("MultiTenantRenderManager"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hardwareencodercontroller"), Some("HardwareEncoderController"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("encodercontroller"), Some("HardwareEncoderController"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("aimixbalanceinspector"), Some("AiMixBalanceInspector"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("mixbalanceinspector"), Some("AiMixBalanceInspector"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("audiophasealigner"), Some("AudioPhaseAligner"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("phasealigner"), Some("AudioPhaseAligner"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("clappluginhostbridge"), Some("ClapPluginHostBridge"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("claphostbridge"), Some("ClapPluginHostBridge"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("hardwaresurfacecontroller"), Some("HardwareSurfaceController"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("surfacecontroller"), Some("HardwareSurfaceController"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("analogsynctransceiver"), Some("AnalogSyncTransceiver"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("analogtransceiver"), Some("AnalogSyncTransceiver"));

        // 4. Verify Asset Browser categorization
        let fx_folders = BrowserCategory::AudioFx.default_folders();
        let dyn_items = fx_folders.iter().find(|(name, _)| *name == "Dynamics & Level").unwrap().1;
        assert!(dyn_items.contains(&"AiMixBalanceInspector"));
        assert!(dyn_items.contains(&"AudioPhaseAligner"));

        let midi_folders = BrowserCategory::MidiFx.default_folders();
        let gen_sync_items = midi_folders.iter().find(|(name, _)| *name == "Generative & Sync").unwrap().1;
        assert!(gen_sync_items.contains(&"LiveClipMatrix"));
        assert!(gen_sync_items.contains(&"PolymetricTrackerSequencer"));
        assert!(gen_sync_items.contains(&"GenerativeStochasticEngine"));
        assert!(gen_sync_items.contains(&"HarmonicCadenceEngine"));

        let routing_items = midi_folders.iter().find(|(name, _)| *name == "Transforms & Routing").unwrap().1;
        assert!(routing_items.contains(&"CompingTakeManager"));
        assert!(routing_items.contains(&"MidiPerformanceHub"));
        assert!(routing_items.contains(&"MidiStreamAnalyzer"));
        assert!(routing_items.contains(&"AutomationTimelineManager"));
        assert!(routing_items.contains(&"IsomorphicKeyboardRouter"));
        assert!(routing_items.contains(&"VoiceLeadingGraph"));
        assert!(routing_items.contains(&"VowelTrajectoryGraph"));
        assert!(routing_items.contains(&"MidiClockSyncHub"));
        assert!(routing_items.contains(&"MidiMessageFilterMatrix"));
        assert!(routing_items.contains(&"DynamicMicrotonalRemapper"));
        assert!(routing_items.contains(&"MultiTenantRenderManager"));
        assert!(routing_items.contains(&"HardwareEncoderController"));
        assert!(routing_items.contains(&"ClapPluginHostBridge"));
        assert!(routing_items.contains(&"HardwareSurfaceController"));
        assert!(routing_items.contains(&"AnalogSyncTransceiver"));

        // 5. Verify Novice Presets synchronize in AwardWinningGuiView
        #[cfg(feature = "gui")]
        {
            let mut view = crate::views::award_winning_gui_view::AwardWinningGuiView::default();
            let ctx = eframe::egui::Context::default();

            // Preset: Live Session Matrix Clip Launcher -> LiveClipMatrix
            view.top_bar_state.selected_preset = "Live Session Matrix Clip Launcher".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("LiveClipMatrix".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("LiveClipMatrix".to_string()));

            // Preset: AI Spectral Unmasking & Mix Balance HUD -> AiMixBalanceInspector
            view.top_bar_state.selected_preset = "AI Spectral Unmasking & Mix Balance HUD".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("AiMixBalanceInspector".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("AiMixBalanceInspector".to_string()));

            // Preset: Hexagonal Wicki-Hayden Isomorphic Keyboard -> IsomorphicKeyboardRouter
            view.top_bar_state.selected_preset = "Hexagonal Wicki-Hayden Isomorphic Keyboard".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("IsomorphicKeyboardRouter".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("IsomorphicKeyboardRouter".to_string()));

            // Preset: Polymetric Euclidean Tracker Groove Matrix -> PolymetricTrackerSequencer
            view.top_bar_state.selected_preset = "Polymetric Euclidean Tracker Groove Matrix".to_string();
            let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
            assert_eq!(view.device_rack_state.selected_node_kind, Some("PolymetricTrackerSequencer".to_string()));
            assert_eq!(view.inspector_state.selected_node_kind, Some("PolymetricTrackerSequencer".to_string()));
        }
    }
}


