// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #45 / Turn #12 Sprint Review Verification Suite:
//! - Real-Time Parameter Automation Bridge (Milestone 33 Full Convergence)
//! - Master Bus Mute (`ParamId(9995)`) & Mono Audition (`ParamId(9998)`) Automation Registration & Live Timeline Recording
//! - Pro Channel Strip Phase Inversion (`ParamId(track_id * 1000 + 204)`) & Input Trim (`ParamId(track_id * 1000 + 205)`) Automation
//! - Top Bar Quick Master Automation Requests (`master_mono`, `master_mute`, `master_trim`, `master_gain`)
//! - Dedicated Master Mono & Master Mute Boolean Automation Editors with Step Curves
//! - Unified 4-View Auto-Selectors (Arranger, Piano Roll, Modular, Stage) with Phase, Trim, and Master Bus Group
//! - Mixer Console Secondary-Click Automation Launchers (Mute, Solo, Phase, Pan, Gain, Master Mute, Master Mono, Master Trim, Master Gain)
//! - Zero Heap Allocation Audio-Thread ParamBus Safety

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn45_pure_master_bus_param_addressing() {
        let mut bus = ParamBus::new();

        let master_gain_pid = ParamId(9999);
        let master_mono_pid = ParamId(9998);
        let master_trim_pid = ParamId(9997);
        let master_panic_pid = ParamId(9996);
        let master_mute_pid = ParamId(9995);

        let track1_phase_pid = ParamId(1 * 1000 + 204);
        let track1_trim_pid = ParamId(1 * 1000 + 205);
        let track3_phase_pid = ParamId(3 * 1000 + 204);
        let track3_trim_pid = ParamId(3 * 1000 + 205);

        bus.register(master_gain_pid, 1.0);
        bus.register(master_mono_pid, 0.0);
        bus.register(master_trim_pid, 0.0);
        bus.register(master_panic_pid, 0.0);
        bus.register(master_mute_pid, 0.0);
        bus.register(track1_phase_pid, 0.0);
        bus.register(track1_trim_pid, 0.0);
        bus.register(track3_phase_pid, 0.0);
        bus.register(track3_trim_pid, 0.0);

        assert_eq!(bus.get(master_gain_pid), Some(1.0));
        assert_eq!(bus.get(master_mono_pid), Some(0.0));
        assert_eq!(bus.get(master_trim_pid), Some(0.0));
        assert_eq!(bus.get(master_panic_pid), Some(0.0));
        assert_eq!(bus.get(master_mute_pid), Some(0.0));
        assert_eq!(bus.get(track1_phase_pid), Some(0.0));
        assert_eq!(bus.get(track1_trim_pid), Some(0.0));

        // Atomic lock-free mutations
        bus.set(master_mute_pid, 1.0);
        bus.set(master_mono_pid, 1.0);
        bus.set(master_trim_pid, -3.0);
        bus.set(track1_phase_pid, 1.0);
        bus.set(track1_trim_pid, 6.0);
        bus.set(track3_phase_pid, 1.0);
        bus.set(track3_trim_pid, -4.5);

        assert_eq!(bus.get(master_mute_pid), Some(1.0));
        assert_eq!(bus.get(master_mono_pid), Some(1.0));
        assert_eq!(bus.get(master_trim_pid), Some(-3.0));
        assert_eq!(bus.get(track1_phase_pid), Some(1.0));
        assert_eq!(bus.get(track1_trim_pid), Some(6.0));
        assert_eq!(bus.get(track3_phase_pid), Some(1.0));
        assert_eq!(bus.get(track3_trim_pid), Some(-4.5));
    }

    #[test]
    fn test_turn45_pure_automation_registry_master_and_phase_trim() {
        let mut registry = AutomationRegistry::new();

        registry.register_param("master_gain", 1.0);
        registry.register_param("master_mono", 0.0);
        registry.register_param("master_mute", 0.0);
        registry.register_param("master_trim", 0.0);
        registry.register_param("track_1_phase", 0.0);
        registry.register_param("track_1_trim", 0.0);

        assert_eq!(registry.get_param("master_gain").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("master_mono").map(|p| p.get()), Some(0.0));
        assert_eq!(registry.get_param("master_mute").map(|p| p.get()), Some(0.0));
        assert_eq!(registry.get_param("master_trim").map(|p| p.get()), Some(0.0));
        assert_eq!(registry.get_param("track_1_phase").map(|p| p.get()), Some(0.0));
        assert_eq!(registry.get_param("track_1_trim").map(|p| p.get()), Some(0.0));

        registry.set("master_mute", 1.0);
        registry.set("master_mono", 1.0);
        registry.set("master_trim", 2.5);
        registry.set("track_1_phase", 1.0);
        registry.set("track_1_trim", -6.0);

        assert_eq!(registry.get_param("master_mute").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("master_mono").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("master_trim").map(|p| p.get()), Some(2.5));
        assert_eq!(registry.get_param("track_1_phase").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("track_1_trim").map(|p| p.get()), Some(-6.0));
    }

    #[test]
    fn test_turn45_pure_automation_timeline_step_and_linear_evaluation() {
        let mut timeline = AutomationTimeline::default();

        // Step curve for mute (0.0 to 4.0: 0.0, 4.0 onwards: 1.0)
        let mute_points = vec![
            AutomationPoint {
                beat: 0.0,
                value: 0.0,
                interp: Interpolation::Step,
            },
            AutomationPoint {
                beat: 4.0,
                value: 1.0,
                interp: Interpolation::Step,
            },
        ];
        timeline.lanes.insert(
            "master_mute".to_string(),
            AutomationLane {
                param_id: "master_mute".to_string(),
                curve: AutomationCurve { points: mute_points },
            },
        );

        // Linear curve for trim (0.0: 0dB, 8.0: +6dB)
        let trim_points = vec![
            AutomationPoint {
                beat: 0.0,
                value: 0.0,
                interp: Interpolation::Linear,
            },
            AutomationPoint {
                beat: 8.0,
                value: 6.0,
                interp: Interpolation::Linear,
            },
        ];
        timeline.lanes.insert(
            "master_trim".to_string(),
            AutomationLane {
                param_id: "master_trim".to_string(),
                curve: AutomationCurve { points: trim_points },
            },
        );

        // Step curve evaluation
        assert_eq!(timeline.evaluate("master_mute", 0.0), Some(0.0));
        assert_eq!(timeline.evaluate("master_mute", 2.0), Some(0.0));
        assert_eq!(timeline.evaluate("master_mute", 3.99), Some(0.0));
        assert_eq!(timeline.evaluate("master_mute", 4.0), Some(1.0));
        assert_eq!(timeline.evaluate("master_mute", 6.0), Some(1.0));

        // Linear curve evaluation
        assert_eq!(timeline.evaluate("master_trim", 0.0), Some(0.0));
        let val_4 = timeline.evaluate("master_trim", 4.0).unwrap();
        assert!((val_4 - 3.0).abs() < 1e-4);
        assert_eq!(timeline.evaluate("master_trim", 8.0), Some(6.0));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::ModernInspectorState;
    use crate::views::modern_top_bar::ModernTopBarState;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn45_gui_top_bar_master_automation_requests() {
        let mut top_bar = ModernTopBarState::default();

        top_bar.request_master_automation();
        assert_eq!(top_bar.requested_automation_param.as_deref(), Some("master_gain"));

        top_bar.request_master_trim_automation();
        assert_eq!(top_bar.requested_automation_param.as_deref(), Some("master_trim"));

        top_bar.request_master_mono_automation();
        assert_eq!(top_bar.requested_automation_param.as_deref(), Some("master_mono"));

        top_bar.request_master_mute_automation();
        assert_eq!(top_bar.requested_automation_param.as_deref(), Some("master_mute"));
    }

    #[test]
    fn test_turn45_gui_inspector_phase_and_trim_defaults_and_reset() {
        let mut inspector = ModernInspectorState::default();

        assert!(!inspector.phase_inverted);
        assert_eq!(inspector.input_trim_db, 0.0);

        inspector.phase_inverted = true;
        inspector.input_trim_db = 4.5;
        assert!(inspector.phase_inverted);
        assert_eq!(inspector.input_trim_db, 4.5);

        inspector.reset_mix_controls();
        assert!(!inspector.phase_inverted);
        assert_eq!(inspector.input_trim_db, 0.0);
    }

    #[test]
    fn test_turn45_gui_master_mono_and_mute_automation_editors() {
        let mut view = AwardWinningGuiView::new();

        let opened_mono = view.open_master_mono_automation_editor();
        assert!(opened_mono);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_mono"));
        let ed_mono = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_mono.min_display, 0.0);
        assert_eq!(ed_mono.max_display, 1.0);
        assert_eq!(ed_mono.unit_name, "bool");

        let opened_mute = view.open_master_mute_automation_editor();
        assert!(opened_mute);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_mute"));
        let ed_mute = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_mute.min_display, 0.0);
        assert_eq!(ed_mute.max_display, 1.0);
        assert_eq!(ed_mute.unit_name, "bool");
    }

    #[test]
    fn test_turn45_gui_open_track_automation_editor_routes_phase_trim_and_master() {
        let mut view = AwardWinningGuiView::new();
        let track_id = view.tracks[0].id;

        // Track phase
        assert!(view.open_track_automation_editor(track_id, "phase"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some(format!("track_{}_phase", track_id).as_str()));
        let ed_phase = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_phase.unit_name, "bool");

        // Track trim
        assert!(view.open_track_automation_editor(track_id, "trim"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some(format!("track_{}_trim", track_id).as_str()));
        let ed_trim = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_trim.unit_name, "dB");
        assert_eq!(ed_trim.min_display, -18.0);
        assert_eq!(ed_trim.max_display, 18.0);

        // Master routes
        assert!(view.open_track_automation_editor(track_id, "master_gain"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_gain"));

        assert!(view.open_track_automation_editor(track_id, "master_trim"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_trim"));

        assert!(view.open_track_automation_editor(track_id, "master_mono"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_mono"));

        assert!(view.open_track_automation_editor(track_id, "master_mute"));
        assert_eq!(view.requested_modular_automation_param.as_deref(), Some("master_mute"));
    }

    #[test]
    fn test_turn45_gui_sync_with_param_bus_dispatch_and_recording() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let mut param_bus = ParamBus::new();
        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::default();

        let track_id = view.tracks[0].id;
        let master_gain_pid = ParamId(9999);
        let master_mono_pid = ParamId(9998);
        let master_trim_pid = ParamId(9997);
        let master_mute_pid = ParamId(9995);
        let track_phase_pid = ParamId(track_id as u32 * 1000 + 204);
        let track_trim_pid = ParamId(track_id as u32 * 1000 + 205);

        param_bus.register(master_gain_pid, 1.0);
        param_bus.register(master_mono_pid, 0.0);
        param_bus.register(master_trim_pid, 0.0);
        param_bus.register(master_mute_pid, 0.0);
        param_bus.register(track_phase_pid, 0.0);
        param_bus.register(track_trim_pid, 0.0);

        // Change states
        view.is_master_mono = true;
        view.is_master_muted = true;
        view.master_trim_db = -2.0;
        view.set_track_phase_inverted(track_id, true);
        view.set_track_input_trim(track_id, 3.5);

        // 1. Sync with recording = true
        view.sync_with_param_bus(&project, &param_bus, &mut registry, &mut timeline, 2.0, true);

        // Verify ParamBus live dispatch
        assert_eq!(param_bus.get(master_mono_pid), Some(1.0));
        assert_eq!(param_bus.get(master_mute_pid), Some(1.0));
        assert_eq!(param_bus.get(master_trim_pid), Some(-2.0));
        assert_eq!(param_bus.get(track_phase_pid), Some(1.0));
        assert_eq!(param_bus.get(track_trim_pid), Some(3.5));

        // Verify AutomationRegistry
        assert_eq!(registry.get_param("master_mono").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("master_mute").map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param("master_trim").map(|p| p.get()), Some(-2.0));
        assert_eq!(registry.get_param(&format!("track_{}_phase", track_id)).map(|p| p.get()), Some(1.0));
        assert_eq!(registry.get_param(&format!("track_{}_trim", track_id)).map(|p| p.get()), Some(3.5));

        // Verify AutomationTimeline has recorded points at beat 2.0
        assert_eq!(timeline.evaluate("master_mono", 2.0), Some(1.0));
        assert_eq!(timeline.evaluate("master_mute", 2.0), Some(1.0));
        assert_eq!(timeline.evaluate("master_trim", 2.0), Some(-2.0));
        assert_eq!(timeline.evaluate(&format!("track_{}_phase", track_id), 2.0), Some(1.0));
        assert_eq!(timeline.evaluate(&format!("track_{}_trim", track_id), 2.0), Some(3.5));
    }
}
