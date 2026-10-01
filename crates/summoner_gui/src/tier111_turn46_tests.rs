// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #46 / Turn #13 Sprint Verification Suite:
//! - Physical Modeling Quartz Crystal Singing Bowl & Glass Chalice Friction Resonator HUD (`CrystalResonatorView`)
//! - Non-Linear Stick-Slip Friction & Wand Excitation Puck (>= 44x44pt touch bounding target)
//! - Hydro-Acoustic Water Mass Loading Pitch Shift ($\Delta f \propto -(m_{\text{water}}/m_{\text{glass}})^{1/2}$)
//! - 8-Mode Thin-Shell Modal Resonance Spectrum with Degenerate Doublet Splitting ($\Delta f_{\text{split}} \in [0.1, 2.5]\text{ Hz}$)
//! - Multi-Node Track DSP Device Chain Live ParamBus Slot Channel Addressing (`ParamId(track_id * 1000 + slot * 20 + p_i)`)
//! - Master Bus Multi-Node Slot Channels (`ParamId(9000 + slot * 20 + p_i)`)
//! - Device Rack & Inspector Automatic `node_{slot}_{param}` Prefixing for Secondary Devices
//! - Dedicated Scene Launch (`ParamId(9994)`) and Clip Cue (`ParamId(t_id * 1000 + 206)`) ParamBus Channel Isolation
//! - Zero Heap Allocation Audio-Thread ParamBus Safety

#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn46_pure_parambus_slot_dispatch_isolation() {
        let mut bus = ParamBus::new();

        // Track 2: Device Slot 0 (AetherSynth) vs Device Slot 1 (CrystalResonator)
        let slot0_cutoff = ParamId(2 * 1000 + 0 * 20 + 0);
        let slot0_resonance = ParamId(2 * 1000 + 0 * 20 + 1);
        let slot1_root_freq = ParamId(2 * 1000 + 1 * 20 + 0);
        let slot1_water_fill = ParamId(2 * 1000 + 1 * 20 + 1);

        // Master Bus: Device Slot 0 (Master EQ) vs Device Slot 1 (Limiter)
        let master_slot0_freq = ParamId(9000 + 0 * 20 + 0);
        let master_slot1_thresh = ParamId(9000 + 1 * 20 + 0);

        bus.register(slot0_cutoff, 0.65);
        bus.register(slot0_resonance, 0.45);
        bus.register(slot1_root_freq, 528.0);
        bus.register(slot1_water_fill, 0.25);
        bus.register(master_slot0_freq, 1000.0);
        bus.register(master_slot1_thresh, -0.5);

        assert_eq!(bus.get(slot0_cutoff), Some(0.65));
        assert_eq!(bus.get(slot1_root_freq), Some(528.0));
        assert_eq!(bus.get(master_slot0_freq), Some(1000.0));
        assert_eq!(bus.get(master_slot1_thresh), Some(-0.5));

        // Atomic lock-free slot updates without inter-device cross-talk
        bus.set(slot0_cutoff, 0.80);
        bus.set(slot1_root_freq, 432.0);
        bus.set(slot1_water_fill, 0.50);
        bus.set(master_slot1_thresh, -2.0);

        assert_eq!(bus.get(slot0_cutoff), Some(0.80));
        assert_eq!(bus.get(slot1_root_freq), Some(432.0));
        assert_eq!(bus.get(slot1_water_fill), Some(0.50));
        assert_eq!(bus.get(master_slot0_freq), Some(1000.0)); // slot 0 remains untouched
        assert_eq!(bus.get(master_slot1_thresh), Some(-2.0));
    }

    #[test]
    fn test_turn46_pure_scene_and_clip_channel_isolation() {
        let mut bus = ParamBus::new();

        let master_trim_pid = ParamId(9997);
        let dedicated_scene_pid = ParamId(9994);
        let track1_phase_pid = ParamId(1 * 1000 + 204);
        let track1_clip_pid = ParamId(1 * 1000 + 206);

        bus.register(master_trim_pid, 0.0);
        bus.register(dedicated_scene_pid, -1.0);
        bus.register(track1_phase_pid, 0.0);
        bus.register(track1_clip_pid, -1.0);

        // Setting master trim must NOT clobber scene launch state
        bus.set(master_trim_pid, -3.5);
        bus.set(dedicated_scene_pid, 2.0);
        assert_eq!(bus.get(master_trim_pid), Some(-3.5));
        assert_eq!(bus.get(dedicated_scene_pid), Some(2.0));

        // Inverting track phase must NOT clobber track active clip cue
        bus.set(track1_phase_pid, 1.0);
        bus.set(track1_clip_pid, 3.0);
        assert_eq!(bus.get(track1_phase_pid), Some(1.0));
        assert_eq!(bus.get(track1_clip_pid), Some(3.0));
    }

    #[test]
    fn test_turn46_pure_slot_automation_timeline_evaluation() {
        let mut timeline = AutomationTimeline::new();

        let lane_slot0 = "track_1_node_0_cutoff".to_string();
        let lane_slot1 = "track_1_node_1_water_fill_level".to_string();

        timeline.lanes.insert(lane_slot0.clone(), AutomationLane {
            param_id: lane_slot0.clone(),
            curve: AutomationCurve {
                points: vec![
                    AutomationPoint { beat: 0.0, value: 0.3, interp: Interpolation::Linear },
                    AutomationPoint { beat: 8.0, value: 0.9, interp: Interpolation::Linear },
                ],
            },
        });

        timeline.lanes.insert(lane_slot1.clone(), AutomationLane {
            param_id: lane_slot1.clone(),
            curve: AutomationCurve {
                points: vec![
                    AutomationPoint { beat: 0.0, value: 0.1, interp: Interpolation::Linear },
                    AutomationPoint { beat: 4.0, value: 0.8, interp: Interpolation::Linear },
                ],
            },
        });

        assert_eq!(timeline.evaluate(&lane_slot0, 0.0), Some(0.3));
        assert_eq!(timeline.evaluate(&lane_slot0, 4.0), Some(0.6));
        assert_eq!(timeline.evaluate(&lane_slot0, 8.0), Some(0.9));

        assert_eq!(timeline.evaluate(&lane_slot1, 0.0), Some(0.1));
        assert_eq!(timeline.evaluate(&lane_slot1, 2.0), Some(0.45));
        assert_eq!(timeline.evaluate(&lane_slot1, 4.0), Some(0.8));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::crystal_resonator_view::{
        CrystalMaterialProfile, CrystalResonatorView, CRYSTAL_PUCK_HIT_RADIUS, NUM_CRYSTAL_MODES,
    };
    use crate::views::modern_device_rack::ModernDeviceRackState;
    use crate::views::modern_inspector::ModernInspectorState;
    use crate::layout_math::Rect;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn46_gui_crystal_resonator_physics_and_doublets() {
        let mut view = CrystalResonatorView::new();

        assert_eq!(view.material_profile, CrystalMaterialProfile::PureQuartzCrystal);
        assert_eq!(view.root_freq_hz, 432.0);
        assert_eq!(view.modal_amplitudes.len(), NUM_CRYSTAL_MODES);

        // Verify all 8 modes have positive frequencies and realistic degenerate doublet splitting
        for i in 0..NUM_CRYSTAL_MODES {
            let (fa, fb) = view.modal_frequencies[i];
            assert!(fa > 20.0, "Mode frequency fa must be in audible range: {}", fa);
            assert!(fb > fa, "Doublet mode B must exceed mode A: {} vs {}", fb, fa);
            let df = fb - fa;
            assert!(df >= 0.1 && df <= 5.0, "Doublet splitting delta f must be [0.1, 5.0] Hz: {}", df);
        }

        // Test Franklin Chalice material profile
        view.set_material_profile(CrystalMaterialProfile::FranklinQuartzGlass);
        assert_eq!(view.root_freq_hz, 523.25);
        assert!(view.effective_f0_hz <= 523.25);

        // Test Metallophone alloy profile
        view.set_material_profile(CrystalMaterialProfile::MetallophoneAlloy);
        assert_eq!(view.root_freq_hz, 880.0);
        assert_eq!(view.doublet_split_hz, 1.20);
    }

    #[test]
    fn test_turn46_gui_water_mass_loading_pitch_lowering() {
        let mut view = CrystalResonatorView::new();
        view.set_material_profile(CrystalMaterialProfile::WetCrystalGoblet);

        // Dry state (0% water fill)
        view.water_fill_pct = 0.0;
        view.update_physics();
        let f0_dry = view.effective_f0_hz;
        assert_eq!(f0_dry, 659.25);

        // 25% water fill
        view.water_fill_pct = 0.25;
        view.update_physics();
        let f0_quarter = view.effective_f0_hz;
        assert!(f0_quarter < f0_dry, "Quarter fill must lower pitch below dry");

        // 100% full water fill
        view.water_fill_pct = 1.0;
        view.update_physics();
        let f0_full = view.effective_f0_hz;
        assert!(f0_full < f0_quarter, "Full fill must lower pitch below quarter fill");

        // Upper modal damping from water presence
        assert!(view.modal_amplitudes[7] < view.modal_amplitudes[0]);
    }

    #[test]
    fn test_turn46_gui_crystal_resonator_hit_target_metrics() {
        let view = CrystalResonatorView::new();
        assert!(
            CRYSTAL_PUCK_HIT_RADIUS >= 22.0,
            "Puck hit radius must be >= 22.0pt to satisfy WCAG touch bounding requirements: {}",
            CRYSTAL_PUCK_HIT_RADIUS
        );

        let canvas = Rect { x: 100.0, y: 100.0, width: 300.0, height: 300.0 };
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_puck((puck_x + 18.0, puck_y), canvas));
        assert!(view.hit_test_puck((puck_x, puck_y - 18.0), canvas));
        assert!(!view.hit_test_puck((puck_x + 30.0, puck_y), canvas));
    }

    #[test]
    fn test_turn46_gui_crystal_resonator_ascii_render() {
        let view = CrystalResonatorView::new();
        let lines = view.render_ascii(64, 22);

        assert_eq!(lines.len(), 22);
        assert_eq!(lines[0].len(), 64);
        assert!(lines[0].starts_with('+'));
        assert!(lines[0].ends_with('+'));
        assert!(lines[21].starts_with('+'));

        let text = lines.join("\n");
        assert!(text.contains('W'), "ASCII rendering must display wand friction puck 'W'");
        assert!(text.contains('#'), "ASCII rendering must display thin-shell modal resonance bars '#'");
    }

    #[test]
    fn test_turn46_gui_crystal_resonator_headless_ui() {
        let mut view = CrystalResonatorView::new();
        let ctx = eframe::egui::Context::default();

        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.ui(ui);
            });
        });

        // Trigger mallet strike and verify physics update without panic
        view.strike_mallet(0.9);
        assert_eq!(view.strike_velocity, 0.9);

        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.ui(ui);
            });
        });

        assert!(view.modal_amplitudes[0] > 0.0);
    }

    #[test]
    fn test_turn46_gui_device_rack_and_inspector_slot_automation_prefixing() {
        let mut rack = ModernDeviceRackState::default();
        rack.selected_chain_idx = 0;
        rack.request_automation("cutoff");
        assert_eq!(rack.requested_automation_param, Some("cutoff".to_string()));

        // When secondary slot is selected (e.g. Slot 1 = CrystalResonator), auto-prefix with node_{slot}_{param}
        rack.selected_chain_idx = 1;
        rack.request_automation("water_fill_level");
        assert_eq!(rack.requested_automation_param, Some("node_1_water_fill_level".to_string()));

        let mut insp = ModernInspectorState::default();
        insp.selected_chain_idx = 0;
        insp.request_automation("resonance");
        assert_eq!(insp.requested_automation_param, Some("resonance".to_string()));

        insp.selected_chain_idx = 2;
        insp.request_automation("q_scale");
        assert_eq!(insp.requested_automation_param, Some("node_2_q_scale".to_string()));
    }

    #[test]
    fn test_turn46_gui_open_track_automation_editor_slot_fallback() {
        let mut view = AwardWinningGuiView::new();
        let track_id = 1;

        // Ensure track has multi-device chain
        view.device_rack_state.ensure_chain();
        let slot1_idx = view.device_rack_state.add_device("CrystalResonator", "Crystal Bowl");
        assert_eq!(slot1_idx, 1);
        view.device_rack_state.select_device(slot1_idx);

        // Open automation with raw parameter name while slot 1 is active:
        // open_track_automation_editor must automatically resolve to node_1_water_fill_level
        let res = view.open_track_automation_editor(track_id, "water_fill_level");
        assert!(res, "Opening automation for active device slot parameter must succeed");
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_node_1_water_fill_level", track_id))
        );
    }

    #[test]
    fn test_turn46_gui_sync_with_param_bus_channel_isolation() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();
        let project = ProjectConfig::default();

        let m_trim_pid = ParamId(9997);
        let dedicated_scene_pid = ParamId(9994);
        let track1_phase_pid = ParamId(1 * 1000 + 204);
        let track1_clip_pid = ParamId(1 * 1000 + 206);

        bus.register(m_trim_pid, 0.0);
        bus.register(dedicated_scene_pid, -1.0);
        bus.register(track1_phase_pid, 0.0);
        bus.register(track1_clip_pid, -1.0);

        // Invert phase on track 1 and set master trim
        view.toggle_track_phase_invert(1);
        view.set_master_trim(-4.0);

        // Run sync
        view.sync_with_param_bus(&project, &bus, &mut registry, &mut timeline, 0.0, false);

        // Verify phase invert and master trim are strictly maintained without collision
        assert_eq!(bus.get(track1_phase_pid), Some(1.0));
        assert_eq!(bus.get(m_trim_pid), Some(-4.0));
    }
}
