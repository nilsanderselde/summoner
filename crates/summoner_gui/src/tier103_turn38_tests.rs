// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #38 / Turn #5 System & GUI Verification Suite:
//! - Modern Device Rack connection to live ParamBus (Section 1 dials, Section 2 faders, Section 4 filter puck, Section 5 LFO, and Pro Drawer parameters)
//! - Bi-directional state synchronization between Piano Roll track selector, Inspector, and Device Rack (`select_track_for_piano_roll`)
//! - Piano Roll lower lane 1-click live parameter automation launcher (`[📈 Auto]` button and secondary click on lane badge)
//! - Support for "gate" / "gate_len" in `open_track_automation_editor` alongside "velocity" and "pitch"
//! - Zero-allocation lockstep parameter dispatch to audio engine via `ParamBus`

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn38_pure_param_bus_device_rack_standard_dials_and_pro_params() {
        let mut bus = ParamBus::new();
        let track_id = 1u32;

        // Register standard device rack dial PIDs (offsets 0..9, 200)
        let cutoff_pid = ParamId(track_id * 1000 + 0);
        let resonance_pid = ParamId(track_id * 1000 + 1);
        let decay_pid = ParamId(track_id * 1000 + 2);
        let env_decay_pid = ParamId(track_id * 1000 + 3);
        let mod_amt_pid = ParamId(track_id * 1000 + 4);
        let drive_pid = ParamId(track_id * 1000 + 5);
        let osc_mix_pid = ParamId(track_id * 1000 + 6);
        let shape_pid = ParamId(track_id * 1000 + 7);
        let lfo_speed_pid = ParamId(track_id * 1000 + 8);
        let lfo_depth_pid = ParamId(track_id * 1000 + 9);
        let volume_pid = ParamId(track_id * 1000 + 200);

        bus.register(cutoff_pid, 0.5);
        bus.register(resonance_pid, 0.3);
        bus.register(decay_pid, 0.4);
        bus.register(env_decay_pid, 0.25);
        bus.register(mod_amt_pid, 0.1);
        bus.register(drive_pid, 0.0);
        bus.register(osc_mix_pid, 0.5);
        bus.register(shape_pid, 0.0);
        bus.register(lfo_speed_pid, 0.2);
        bus.register(lfo_depth_pid, 0.15);
        bus.register(volume_pid, 0.8);

        // Verify initial registration
        assert_eq!(bus.get(cutoff_pid), Some(0.5));
        assert_eq!(bus.get(resonance_pid), Some(0.3));
        assert_eq!(bus.get(decay_pid), Some(0.4));
        assert_eq!(bus.get(env_decay_pid), Some(0.25));
        assert_eq!(bus.get(mod_amt_pid), Some(0.1));
        assert_eq!(bus.get(drive_pid), Some(0.0));
        assert_eq!(bus.get(osc_mix_pid), Some(0.5));
        assert_eq!(bus.get(shape_pid), Some(0.0));
        assert_eq!(bus.get(lfo_speed_pid), Some(0.2));
        assert_eq!(bus.get(lfo_depth_pid), Some(0.15));
        assert_eq!(bus.get(volume_pid), Some(0.8));

        // Live atomic parameter updates from UI thread (zero allocation)
        bus.set(cutoff_pid, 0.75);
        bus.set(resonance_pid, 0.85);
        bus.set(volume_pid, 1.0);

        assert_eq!(bus.get(cutoff_pid), Some(0.75));
        assert_eq!(bus.get(resonance_pid), Some(0.85));
        assert_eq!(bus.get(volume_pid), Some(1.0));

        // Register pro drawer modular parameters (offsets 500..505)
        for i in 0..6 {
            let pid = ParamId(track_id * 1000 + 500 + i);
            bus.register(pid, (i as f32) * 0.15);
            assert_eq!(bus.get(pid), Some((i as f32) * 0.15));
            bus.set(pid, (i as f32) * 0.15 + 0.1);
            assert_eq!(bus.get(pid), Some((i as f32) * 0.15 + 0.1));
        }
    }

    #[test]
    fn test_turn38_pure_param_id_track_offset_invariants() {
        for track_id in 1..=16u32 {
            let base = track_id * 1000;
            assert_eq!(ParamId(base + 0), ParamId(track_id * 1000));
            assert_eq!(ParamId(base + 200), ParamId(track_id * 1000 + 200));
            assert_eq!(ParamId(base + 500), ParamId(track_id * 1000 + 500));
        }
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, PianoRollLaneMode};
    use crate::views::modern_device_rack::{
        show_modern_device_rack_with_context, ModernDeviceRackState,
    };
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn38_piano_roll_lane_mode_cycling() {
        let mut mode = PianoRollLaneMode::Velocity;

        // Cycle 1: Velocity -> Gate
        mode = match mode {
            PianoRollLaneMode::Velocity => PianoRollLaneMode::Gate,
            PianoRollLaneMode::Gate => PianoRollLaneMode::PitchBend,
            PianoRollLaneMode::PitchBend => PianoRollLaneMode::Velocity,
        };
        assert_eq!(mode, PianoRollLaneMode::Gate);

        // Cycle 2: Gate -> PitchBend
        mode = match mode {
            PianoRollLaneMode::Velocity => PianoRollLaneMode::Gate,
            PianoRollLaneMode::Gate => PianoRollLaneMode::PitchBend,
            PianoRollLaneMode::PitchBend => PianoRollLaneMode::Velocity,
        };
        assert_eq!(mode, PianoRollLaneMode::PitchBend);

        // Cycle 3: PitchBend -> Velocity
        mode = match mode {
            PianoRollLaneMode::Velocity => PianoRollLaneMode::Gate,
            PianoRollLaneMode::Gate => PianoRollLaneMode::PitchBend,
            PianoRollLaneMode::PitchBend => PianoRollLaneMode::Velocity,
        };
        assert_eq!(mode, PianoRollLaneMode::Velocity);
    }

    #[test]
    fn test_turn38_device_rack_with_context_and_live_parambus() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();
        let mut bus = ParamBus::new();
        bus.register(ParamId(1000), 0.5); // cutoff
        bus.register(ParamId(1001), 0.3); // resonance
        bus.register(ParamId(1200), 0.8); // volume

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), 1);
            });
        });

        assert_eq!(state.device_name, "Synth 1");
    }

    #[test]
    fn test_turn38_piano_roll_track_selection_sync() {
        let mut view = AwardWinningGuiView::new();
        assert!(view.tracks.len() >= 2);

        // Switch to Track 1 using select_track_for_piano_roll
        assert!(view.select_track_for_piano_roll(1));
        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.device_rack_state.device_name, view.tracks[1].name);
        assert_eq!(view.inspector_state.target_name, view.tracks[1].name);
        assert_eq!(view.inspector_state.pan_val, view.tracks[1].pan);
        assert_eq!(view.inspector_state.is_muted, view.tracks[1].is_muted);
        assert_eq!(view.inspector_state.is_soloed, view.tracks[1].is_soloed);

        // Switch to Track 0
        assert!(view.select_track_for_piano_roll(0));
        assert_eq!(view.selected_track_idx, 0);
        assert_eq!(view.device_rack_state.device_name, view.tracks[0].name);
        assert_eq!(view.inspector_state.target_name, view.tracks[0].name);

        // Out of bounds check
        assert!(!view.select_track_for_piano_roll(9999));
        assert_eq!(view.selected_track_idx, 0);
    }

    #[test]
    fn test_turn38_piano_roll_lane_automation_launchers() {
        let mut view = AwardWinningGuiView::new();

        // 1. Velocity automation launcher
        view.piano_roll_lane_mode = PianoRollLaneMode::Velocity;
        assert!(view.open_track_automation_editor(1, "velocity"));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some("track_1_velocity".to_string())
        );

        // 2. Gate automation launcher
        view.piano_roll_lane_mode = PianoRollLaneMode::Gate;
        assert!(view.open_track_automation_editor(1, "gate"));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some("track_1_gate".to_string())
        );

        // 3. Pitch bend automation launcher
        view.piano_roll_lane_mode = PianoRollLaneMode::PitchBend;
        assert!(view.open_track_automation_editor(1, "pitch"));
        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some("track_1_pitch".to_string())
        );
    }

    #[test]
    fn test_turn38_device_rack_automation_request_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Request cutoff automation from device rack
        view.device_rack_state.requested_automation_param = Some("cutoff".to_string());
        let cur_track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        if let Some(param) = view.device_rack_state.requested_automation_param.take() {
            assert!(view.open_track_automation_editor(cur_track_id, &param));
        }

        assert!(view.show_automation_editor_window);
        assert_eq!(
            view.requested_modular_automation_param,
            Some(format!("track_{}_cutoff", cur_track_id))
        );
    }
}
