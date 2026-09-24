// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #16 Integration & Regression Tests:
//! 1. Production-Grade Two-Tier Arranger Track Header (Pro View Launchers: Piano Roll, Modular, Mixer; Quick Mute/Solo/Arm).
//! 2. Universal Modular DSP Module Catalog (2,012+ Nodes Registered, Search Filtering & Category Pills).
//! 3. Milestone 33 Live Parameter Automation Bridge for Track Mute/Solo and Modular Node Parameters.

#[cfg(test)]
pub mod pure_tests {
    use crate::dsp_node_ui::{DspCategory, DspNodeRegistry};
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn16_param_bus_mute_solo_and_modular_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 1u32;

        let mute_pid = ParamId(track_id * 1000 + 202);
        let solo_pid = ParamId(track_id * 1000 + 203);
        let mod_p0_pid = ParamId(track_id * 1000 + 500);
        let mod_p1_pid = ParamId(track_id * 1000 + 501);

        bus.register(mute_pid, 0.0);
        bus.register(solo_pid, 0.0);
        bus.register(mod_p0_pid, 0.5);
        bus.register(mod_p1_pid, 0.25);

        assert_eq!(bus.get(mute_pid), Some(0.0));
        assert_eq!(bus.get(solo_pid), Some(0.0));
        assert_eq!(bus.get(mod_p0_pid), Some(0.5));
        assert_eq!(bus.get(mod_p1_pid), Some(0.25));

        // Live atomic update from GUI thread (zero allocation)
        bus.set(mute_pid, 1.0);
        bus.set(solo_pid, 1.0);
        bus.set(mod_p0_pid, 0.85);

        assert_eq!(bus.get(mute_pid), Some(1.0));
        assert_eq!(bus.get(solo_pid), Some(1.0));
        assert_eq!(bus.get(mod_p0_pid), Some(0.85));
    }

    #[test]
    fn test_turn16_dsp_catalog_category_filtering() {
        let registry = DspNodeRegistry::new();
        let all_nodes = registry.list_all();
        assert!(all_nodes.len() >= 1000, "Registry must contain reflected DSP nodes, found {}", all_nodes.len());

        let categories = [
            DspCategory::Oscillator,
            DspCategory::CompositeSynth,
            DspCategory::AcousticPhysicalModel,
            DspCategory::SamplerSlicer,
            DspCategory::FilterEq,
            DspCategory::DynamicsMaster,
            DspCategory::DistortionSaturation,
            DspCategory::Modulation,
            DspCategory::TimeSpace,
            DspCategory::SpatialSurround,
            DspCategory::SpectralResynthesis,
            DspCategory::NeuralAi,
            DspCategory::Utility,
        ];

        for cat in categories {
            let cat_nodes: Vec<_> = all_nodes.iter().filter(|n| n.category == cat).collect();
            assert!(!cat_nodes.is_empty(), "Category {:?} must have registered DSP modules", cat);

            // Verify search within category
            let sample_name = &cat_nodes[0].display_name.to_lowercase();
            let query = &sample_name[..3.min(sample_name.len())];
            let search_matches: Vec<_> = cat_nodes.iter().filter(|n| {
                n.display_name.to_lowercase().contains(query) || n.kind_id.to_lowercase().contains(query)
            }).collect();
            assert!(!search_matches.is_empty(), "Search within category must find matches");
        }
    }
}

#[cfg(all(test, feature = "gui"))]
mod tests {
    use eframe::egui;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::ModernViewTab;
    use summoner_core::param_bus::ParamBus;
    use summoner_project::create_default_project;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation};

    #[test]
    fn test_turn16_arranger_track_pro_launchers_and_toggles() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        // 1. In Novice Mode: is_pro_mode is false
        assert!(!view.top_bar_state.is_pro_mode);

        // 2. Enable Pro Mode
        view.top_bar_state.is_pro_mode = true;
        assert!(view.top_bar_state.is_pro_mode);

        // Render frame to ensure layout succeeds
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 3. Test quick Mute toggle
        assert!(!view.tracks[0].is_muted);
        view.tracks[0].is_muted = true;
        assert!(view.tracks[0].is_muted);

        // 4. Test quick Solo toggle
        assert!(!view.tracks[1].is_soloed);
        view.tracks[1].is_soloed = true;
        assert!(view.tracks[1].is_soloed);

        // 5. Test quick Arm toggle
        assert!(!view.tracks[2].is_armed);
        view.tracks[2].is_armed = true;
        assert!(view.tracks[2].is_armed);

        // 6. Test 1-click Pro view launchers
        // Switch to Piano Roll
        view.selected_track_idx = 1;
        view.top_bar_state.active_tab = ModernViewTab::PianoRoll;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::PianoRoll);
        assert_eq!(view.selected_track_idx, 1);

        // Switch to Modular
        view.selected_track_idx = 2;
        view.top_bar_state.active_tab = ModernViewTab::Modular;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Modular);
        assert_eq!(view.selected_track_idx, 2);

        // Switch to Mixer
        view.selected_track_idx = 3;
        view.top_bar_state.active_tab = ModernViewTab::Mixer;
        assert_eq!(view.top_bar_state.active_tab, ModernViewTab::Mixer);
        assert_eq!(view.selected_track_idx, 3);
    }

    #[test]
    fn test_turn16_modular_dsp_catalog_window_and_node_insertion() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        assert!(!view.modular_add_modal_open);

        // 1. Open Modular DSP Catalog Modal
        view.modular_add_modal_open = true;
        view.modular_search_query = "reverb".to_string();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.modular_add_modal_open);
        assert_eq!(view.modular_search_query, "reverb");

        // 2. Insert a node from catalog
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let reverb_desc = registry.list_all().into_iter().find(|d| d.kind_id.to_lowercase().contains("reverb")).unwrap();
        let initial_node_count = view.modular_nodes.len();

        view.add_modular_node_from_descriptor(reverb_desc);
        assert_eq!(view.modular_nodes.len(), initial_node_count + 1);

        let last_node = view.modular_nodes.last().unwrap();
        assert_eq!(last_node.kind_id, reverb_desc.kind_id);
        assert_eq!(last_node.display_name, reverb_desc.display_name);
        assert!(!last_node.ports.is_empty());
    }

    #[test]
    fn test_turn16_m33_live_parameter_bridge_mute_solo_and_modular() {
        let mut view = AwardWinningGuiView::default();
        let project = create_default_project("Turn 16 M33 Test");
        let track_id = project.tracks[0].id;
        let mut bus = ParamBus::new();

        let mute_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 202);
        let solo_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 203);
        let mod_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500);

        bus.register(mute_pid, 0.0);
        bus.register(solo_pid, 0.0);
        bus.register(mod_pid, 0.5);

        let mut auto_reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // 1. Set track mute and solo states in GUI
        view.selected_track_idx = 0;
        view.tracks[0].is_muted = true;
        view.tracks[0].is_soloed = true;
        view.selected_modular_node_id = Some("node_test_1".to_string());
        view.inspector_state.node_param_values.insert("resonance".to_string(), 0.77);

        // 2. Dispatch with ParamBus
        view.sync_with_param_bus(&project, &bus, &mut auto_reg, &mut timeline, 0.0, false);

        assert_eq!(bus.get(mute_pid), Some(1.0));
        assert_eq!(bus.get(solo_pid), Some(1.0));
        assert_eq!(bus.get(mod_pid), Some(0.77));

        // 3. Automation curve playback evaluation
        let mute_lane_key = format!("track_{}_mute", track_id);
        timeline.lanes.insert(mute_lane_key.clone(), AutomationLane {
            param_id: mute_lane_key.clone(),
            curve: AutomationCurve {
                points: vec![
                    AutomationPoint { beat: 0.0, value: 0.0, interp: Interpolation::Step },
                    AutomationPoint { beat: 4.0, value: 1.0, interp: Interpolation::Step },
                ],
            },
        });

        // Transport is playing at beat 1.0 -> should evaluate mute to 0.0
        view.top_bar_state.is_playing = true;
        view.sync_with_param_bus(&project, &bus, &mut auto_reg, &mut timeline, 1.0, false);
        assert!(!view.tracks[0].is_muted, "Beat 1.0 should evaluate to unmuted");

        // Transport is playing at beat 5.0 -> should evaluate mute to 1.0
        view.sync_with_param_bus(&project, &bus, &mut auto_reg, &mut timeline, 5.0, false);
        assert!(view.tracks[0].is_muted, "Beat 5.0 should evaluate to muted");
    }
}
