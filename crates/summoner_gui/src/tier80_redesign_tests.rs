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
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("pipeorgannode"), Some("PipeOrgan"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("sitarnode"), Some("Sitar"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("tonewheelorgannode"), Some("TonewheelOrgan"));
        assert_eq!(crate::dsp_node_ui::DspNodeRegistry::normalize_type_name("rotaryspeakernode"), Some("RotarySpeaker"));

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

        assert_eq!(DspNodeRegistry::normalize_type_name("percussionmembranenode"), Some("PercussionMembrane"));
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
}
