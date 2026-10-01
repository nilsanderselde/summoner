//! Comprehensive verification test suite for Turn #35:
//! Modern Device Rack & Node Inspector Two-Tier Tactile Ergonomics, Double-Click Unity/Default Resets,
//! Right-Click Live Bézier Automation Lane Requests, Universal Multi-Tiered DSP Parameter Resolution,
//! Live ParamBus Inspector Bindings, and Secondary Device Rack Dials Playback Automation.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::ModernDeviceRackState;
    use crate::views::modern_inspector::ModernInspectorState;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn35_device_rack_knobs_reset_and_automation_request() {
        let mut state = ModernDeviceRackState::default();
        assert_eq!(state.requested_automation_param, None);

        // Mutate knobs away from defaults
        state.cutoff = 0.10;
        state.resonance = 0.90;
        state.decay = 0.05;
        state.env_decay = 0.85;
        state.mod_amt = 0.15;
        state.drive = 0.95;
        state.osc_mix = 0.20;
        state.shape = 0.80;
        state.volume = 0.30;
        state.lfo_speed = 0.90;
        state.lfo_depth = 0.10;

        // Reset knobs restores all dials to factory defaults
        state.reset_knob_defaults();
        assert!((state.cutoff - 0.65).abs() < 1e-5);
        assert!((state.resonance - 0.45).abs() < 1e-5);
        assert!((state.decay - 0.50).abs() < 1e-5);
        assert!((state.env_decay - 0.35).abs() < 1e-5);
        assert!((state.mod_amt - 0.60).abs() < 1e-5);
        assert!((state.drive - 0.40).abs() < 1e-5);
        assert!((state.osc_mix - 0.75).abs() < 1e-5);
        assert!((state.shape - 0.50).abs() < 1e-5);
        assert!((state.volume - 0.85).abs() < 1e-5);
        assert!((state.lfo_speed - 0.40).abs() < 1e-5);
        assert!((state.lfo_depth - 0.60).abs() < 1e-5);

        // Automation request helper
        state.request_automation("cutoff");
        assert_eq!(state.requested_automation_param, Some("cutoff".to_string()));

        state.request_automation("lfo_speed");
        assert_eq!(state.requested_automation_param, Some("lfo_speed".to_string()));
    }

    #[test]
    fn test_turn35_inspector_gain_pan_mute_solo_reset_and_automation() {
        let mut state = ModernInspectorState::default();
        assert_eq!(state.requested_automation_param, None);

        // Mutate mix controls
        state.gain_db = -18.5;
        state.pan_val = 0.65;
        state.is_muted = true;
        state.is_soloed = true;

        // Reset mix controls restores unity gain, center pan, unmuted/unsoloed
        state.reset_mix_controls();
        assert_eq!(state.gain_db, 0.0);
        assert_eq!(state.pan_val, 0.0);
        assert!(!state.is_muted);
        assert!(!state.is_soloed);

        // Automation requests
        state.request_automation("gain");
        assert_eq!(state.requested_automation_param, Some("gain".to_string()));

        state.request_automation("pan");
        assert_eq!(state.requested_automation_param, Some("pan".to_string()));

        state.request_automation("mute");
        assert_eq!(state.requested_automation_param, Some("mute".to_string()));

        state.request_automation("solo");
        assert_eq!(state.requested_automation_param, Some("solo".to_string()));
    }

    #[test]
    fn test_turn35_open_track_automation_editor_multi_tiered_dsp_fallback() {
        let mut view = AwardWinningGuiView::new();
        let track_id = view.tracks[0].id;

        // 1. Direct standard parameter lookup
        assert!(view.open_track_automation_editor(track_id, "cutoff"));
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_cutoff", track_id)));

        // 2. Fallback to device rack's active DSP node (e.g. AetherSynth params)
        view.device_rack_state.selected_node_kind = Some("AetherSynth".to_string());
        view.device_rack_state.node_param_values.insert("osc_wave".to_string(), 0.75);
        assert!(view.open_track_automation_editor(track_id, "osc_wave"));
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_osc_wave", track_id)));

        // 3. Fallback to inspector's active DSP node (e.g. DemucsV4Separator or FilterSVF)
        view.device_rack_state.selected_node_kind = Some("AetherSynth".to_string());
        view.inspector_state.selected_node_kind = Some("FilterSVF".to_string());
        view.inspector_state.node_param_values.insert("q_factor".to_string(), 0.82);
        assert!(view.open_track_automation_editor(track_id, "q_factor"));
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_q_factor", track_id)));

        // 4. Fallback to universal DSP node registry across any registered DSP module
        // (e.g. NavierStokesFluidNode param "viscosity" or PluckedStringNode param "damping")
        assert!(view.open_track_automation_editor(track_id, "viscosity"));
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_viscosity", track_id)));

        assert!(view.open_track_automation_editor(track_id, "damping"));
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_damping", track_id)));

        // 5. Non-existent parameter returns false cleanly
        assert!(!view.open_track_automation_editor(track_id, "completely_invalid_param_xyz"));
    }

    #[test]
    fn test_turn35_sync_with_param_bus_lfo_and_secondary_dials_playback_and_recording() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0;
        let track_id = view.tracks[0].id;
        let project = ProjectConfig::default();
        let mut param_bus = ParamBus::new();

        // Register ParamBus offsets for secondary dials:
        // 100: cutoff, 101: reso, 102: decay, 103: env_decay, 104: mod_amt, 105: drive,
        // 106: volume, 107: osc_mix, 108: shape, 109: lfo_speed, 110: lfo_depth
        for offset in 100..=110 {
            param_bus.register(ParamId(track_id as u32 * 1000 + offset), 0.0);
        }

        let mut registry = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        // 1. Live recording pass
        view.device_rack_state.osc_mix = 0.42;
        view.device_rack_state.shape = 0.68;
        view.device_rack_state.lfo_speed = 0.77;
        view.device_rack_state.lfo_depth = 0.88;

        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut registry,
            &mut timeline,
            8.0,  // playhead_beat
            true, // is_recording_automation: true
        );

        // Verify ParamBus received values at correct offsets
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 107)), Some(0.42)); // osc_mix
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 108)), Some(0.68)); // shape
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 109)), Some(0.77)); // lfo_speed
        assert_eq!(param_bus.get(ParamId(track_id as u32 * 1000 + 110)), Some(0.88)); // lfo_depth

        // Verify AutomationTimeline has recorded the automated curves
        assert_eq!(timeline.evaluate(&format!("track_{}_osc_mix", track_id), 8.0), Some(0.42));
        assert_eq!(timeline.evaluate(&format!("track_{}_shape", track_id), 8.0), Some(0.68));
        assert_eq!(timeline.evaluate(&format!("track_{}_lfo_speed", track_id), 8.0), Some(0.77));
        assert_eq!(timeline.evaluate(&format!("track_{}_lfo_depth", track_id), 8.0), Some(0.88));

        // 2. Playback pass
        view.top_bar_state.is_playing = true;
        view.device_rack_state.osc_mix = 0.0;
        view.device_rack_state.shape = 0.0;
        view.device_rack_state.lfo_speed = 0.0;
        view.device_rack_state.lfo_depth = 0.0;

        view.sync_with_param_bus(
            &project,
            &param_bus,
            &mut registry,
            &mut timeline,
            8.0,   // playhead_beat
            false, // is_recording_automation: false -> playback
        );

        // Automated curves drive GUI dials back during playback
        assert!((view.device_rack_state.osc_mix - 0.42).abs() < 1e-4);
        assert!((view.device_rack_state.shape - 0.68).abs() < 1e-4);
        assert!((view.device_rack_state.lfo_speed - 0.77).abs() < 1e-4);
        assert!((view.device_rack_state.lfo_depth - 0.88).abs() < 1e-4);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack, ModernDeviceRackState};
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use summoner_core::param_bus::ParamBus;

    #[test]
    fn test_turn35_device_rack_and_inspector_canvas_rendering_without_panic() {
        let ctx = egui::Context::default();
        let mut bus = ParamBus::new();
        bus.register(summoner_core::param_bus::ParamId(1000), 0.5);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Device rack rendering
                let mut rack_state = ModernDeviceRackState::default();
                show_modern_device_rack(ui, &mut rack_state, None);

                // Inspector rendering with live param bus context
                let mut inspector_state = ModernInspectorState::default();
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });
    }

    #[test]
    fn test_turn35_award_winning_gui_view_with_device_rack_and_inspector_automation_dispatch() {
        let ctx = egui::Context::default();
        let mut view = AwardWinningGuiView::new();

        // Queue automation request from device rack
        view.device_rack_state.requested_automation_param = Some("cutoff".to_string());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Automation request consumed and editor opened
        assert_eq!(view.device_rack_state.requested_automation_param, None);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some(format!("track_{}_cutoff", view.tracks[view.selected_track_idx].id)));
    }
}
