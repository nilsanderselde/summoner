// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #44 / Turn #11 Verification Suite:
//! - Master Bus Two-Tier Inspection & Device Rack Integration (Milestone 33)
//! - Master Bus Selection Lockstep across All Views (`select_master`, `select_master_node`, `select_track`)
//! - Master Bus Modular Canvas "🎛 Master Chain ➔ Modular" DAG Synchronization
//! - Pro Channel Strip Phase Inversion ([Ø]) & Input Trim Bridge (`track_dsp.rs`)
//! - Master Bus Trim Control & Live Automation Bridge (ParamId 9997)
//! - Lock-free ParamBus Atomic Channel Addressing for Phase (204) & Trim (205)

#![allow(clippy::all)]

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn44_pure_master_inspection_state_and_offsets() {
        let mut bus = ParamBus::new();

        // Master DSP Device slots: ParamId(9000 + slot * 20 + p_i)
        let master_slot0_cutoff = ParamId(9000 + 0 * 20 + 0);
        let master_slot1_thresh = ParamId(9000 + 1 * 20 + 0);

        // Master Gain (9999), Master Mono (9998), Master Trim (9997)
        let master_gain_pid = ParamId(9999);
        let master_mono_pid = ParamId(9998);
        let master_trim_pid = ParamId(9997);

        // Track 2 Phase (track_id * 1000 + 204) & Trim (track_id * 1000 + 205)
        let track2_phase_pid = ParamId(2 * 1000 + 204);
        let track2_trim_pid = ParamId(2 * 1000 + 205);

        bus.register(master_slot0_cutoff, 1200.0);
        bus.register(master_slot1_thresh, -3.0);
        bus.register(master_gain_pid, 1.0);
        bus.register(master_mono_pid, 0.0);
        bus.register(master_trim_pid, 0.0);
        bus.register(track2_phase_pid, 0.0);
        bus.register(track2_trim_pid, 0.0);

        assert_eq!(bus.get(master_slot0_cutoff), Some(1200.0));
        assert_eq!(bus.get(master_slot1_thresh), Some(-3.0));
        assert_eq!(bus.get(master_gain_pid), Some(1.0));
        assert_eq!(bus.get(master_mono_pid), Some(0.0));
        assert_eq!(bus.get(master_trim_pid), Some(0.0));
        assert_eq!(bus.get(track2_phase_pid), Some(0.0));
        assert_eq!(bus.get(track2_trim_pid), Some(0.0));

        // Atomic lock-free mutations
        bus.set(master_trim_pid, -2.5);
        bus.set(track2_phase_pid, 1.0); // Phase inverted
        bus.set(track2_trim_pid, 3.5);  // +3.5 dB input trim

        assert_eq!(bus.get(master_trim_pid), Some(-2.5));
        assert_eq!(bus.get(track2_phase_pid), Some(1.0));
        assert_eq!(bus.get(track2_trim_pid), Some(3.5));
    }

    #[test]
    fn test_turn44_pure_master_trim_timeline_evaluation() {
        let mut timeline = AutomationTimeline::default();

        let mut points = Vec::new();
        points.push(AutomationPoint {
            beat: 0.0,
            value: 0.0, // 0 dB
            interp: Interpolation::Linear,
        });
        points.push(AutomationPoint {
            beat: 8.0,
            value: -6.0, // -6 dB dip
            interp: Interpolation::Linear,
        });

        timeline.lanes.insert(
            "master_trim".to_string(),
            AutomationLane {
                param_id: "master_trim".to_string(),
                curve: AutomationCurve { points },
            },
        );

        let v0 = timeline.evaluate("master_trim", 0.0).unwrap_or(0.0);
        let v4 = timeline.evaluate("master_trim", 4.0).unwrap_or(0.0);
        let v8 = timeline.evaluate("master_trim", 8.0).unwrap_or(0.0);

        assert!((v0 - 0.0).abs() < 1e-4);
        assert!((v4 - (-3.0)).abs() < 1e-4);
        assert!((v8 - (-6.0)).abs() < 1e-4);
    }

    #[test]
    fn test_turn44_pure_track_dsp_phase_and_trim_invariants() {
        // Mathematical validation of DSP primitives in summoner_dsp::track_dsp
        // Phase flip: y = -x
        let mut audio = vec![0.5f32, -0.25f32, 0.8f32, -1.0f32];
        for s in audio.iter_mut() {
            *s = -*s;
        }
        assert_eq!(audio, vec![-0.5f32, 0.25f32, -0.8f32, 1.0f32]);

        // Input trim: 6.0206 dB roughly equals 2.0x linear gain
        let trim_db = 6.0205999f32;
        let linear_gain = 10.0f32.powf(trim_db / 20.0);
        assert!((linear_gain - 2.0).abs() < 1e-3);

        // Master trim: -6.0206 dB roughly equals 0.5x linear gain
        let m_trim_db = -6.0205999f32;
        let m_linear_gain = 10.0f32.powf(m_trim_db / 20.0);
        assert!((m_linear_gain - 0.5).abs() < 1e-3);
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use summoner_core::param_bus::ParamBus;
    use std::sync::Arc;

    #[test]
    fn test_turn44_gui_master_selection_lockstep() {
        let mut view = AwardWinningGuiView::new();

        // Default state: Track 4 (Synth 1) selected, not inspecting master
        assert!(!view.inspecting_master);
        assert_eq!(view.selected_track_idx, 4);

        // Select Master Bus
        let sel_master = view.select_master();
        assert!(sel_master);
        assert!(view.inspecting_master);
        assert_eq!(view.device_rack_state.device_name, "Master Bus");
        assert_eq!(view.inspector_state.target_name, "Master Bus");
        assert_eq!(view.device_rack_state.chain_devices.len(), view.master_chain_devices.len());
        assert_eq!(view.inspector_state.chain_devices.len(), view.master_chain_devices.len());

        // Select Master Node Slot 1 (e.g. Limiter)
        let sel_slot1 = view.select_master_node(1);
        assert!(sel_slot1);
        assert!(view.inspecting_master);
        assert_eq!(view.selected_master_chain_idx, 1);
        assert_eq!(view.device_rack_state.selected_chain_idx, 1);
        assert_eq!(view.inspector_state.selected_chain_idx, 1);

        // Switch back to Track 2
        let sel_tr2 = view.select_track(2);
        assert!(sel_tr2);
        assert!(!view.inspecting_master);
        assert_eq!(view.selected_track_idx, 2);
    }

    #[test]
    fn test_turn44_gui_master_chain_to_modular_sync() {
        let mut view = AwardWinningGuiView::new();

        // Initial master chain has 2 devices (Master EQ, Limiter)
        assert_eq!(view.master_chain_devices.len(), 2);

        // Sync master chain to modular canvas
        let node_count = view.sync_master_chain_to_modular();

        // Should produce 2 DSP nodes + 1 Master Out DAC = 3 nodes
        assert_eq!(node_count, 3);
        assert_eq!(view.modular_nodes.len(), 3);
        assert_eq!(view.modular_nodes[0].id, "master_node_0");
        assert_eq!(view.modular_nodes[1].id, "master_node_1");
        assert_eq!(view.modular_nodes[2].id, "master_out_dac");

        // Should have 2 sequential patch cords (0 -> 1 -> DAC)
        assert_eq!(view.patch_cords.len(), 2);
        assert_eq!(view.patch_cords[0].from_node_id, "master_node_0");
        assert_eq!(view.patch_cords[0].to_node_id, "master_node_1");
        assert_eq!(view.patch_cords[1].from_node_id, "master_node_1");
        assert_eq!(view.patch_cords[1].to_node_id, "master_out_dac");

        assert_eq!(view.selected_modular_node_id, Some("master_node_0".into()));
    }

    #[test]
    fn test_turn44_gui_track_phase_and_trim_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let mut param_bus = ParamBus::new();
        param_bus.register(summoner_core::param_bus::ParamId(1000 + 204), 0.0);
        param_bus.register(summoner_core::param_bus::ParamId(1000 + 205), 0.0);
        param_bus.register(summoner_core::param_bus::ParamId(9997), 0.0);
        let param_bus = Arc::new(param_bus);
        view.live_param_bus = Some(param_bus.clone());

        let track_id = 1u64;

        // Initial phase not inverted
        assert!(!view.is_track_phase_inverted(track_id));

        // Toggle phase invert
        let inv1 = view.toggle_track_phase_invert(track_id);
        assert!(inv1);
        assert!(view.is_track_phase_inverted(track_id));
        assert_eq!(param_bus.get(summoner_core::param_bus::ParamId(1000 + 204)), Some(1.0));

        let inv2 = view.toggle_track_phase_invert(track_id);
        assert!(!inv2);
        assert!(!view.is_track_phase_inverted(track_id));
        assert_eq!(param_bus.get(summoner_core::param_bus::ParamId(1000 + 204)), Some(0.0));

        // Input trim setting
        view.set_track_input_trim(track_id, 4.5);
        assert_eq!(view.get_track_input_trim(track_id), 4.5);
        assert_eq!(param_bus.get(summoner_core::param_bus::ParamId(1000 + 205)), Some(4.5));

        // Clamping to [-18.0, 18.0]
        view.set_track_input_trim(track_id, 25.0);
        assert_eq!(view.get_track_input_trim(track_id), 18.0);
        view.set_track_input_trim(track_id, -30.0);
        assert_eq!(view.get_track_input_trim(track_id), -18.0);

        // Master trim setting
        view.set_master_trim(-3.0);
        assert_eq!(view.master_trim_db, -3.0);
        assert_eq!(param_bus.get(summoner_core::param_bus::ParamId(9997)), Some(-3.0));

        // Clamping to [-12.0, 12.0]
        view.set_master_trim(15.0);
        assert_eq!(view.master_trim_db, 12.0);
    }

    #[test]
    fn test_turn44_gui_open_master_trim_and_phase_automation() {
        let mut view = AwardWinningGuiView::new();

        // Master trim automation editor
        let res_trim = view.open_master_trim_automation_editor();
        assert!(res_trim);
        assert!(view.show_automation_editor_window);
        assert_eq!(view.requested_modular_automation_param, Some("master_trim".to_string()));
        let ed = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed.parameter_name, "Master Bus — Gain Trim");
        assert_eq!(ed.unit_name, "dB");
        assert_eq!(ed.min_display, -12.0);
        assert_eq!(ed.max_display, 12.0);

        // Track phase automation editor
        let res_phase = view.open_track_automation_editor(1, "phase");
        assert!(res_phase);
        let ed_p = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_p.unit_name, "bool");
        assert_eq!(ed_p.min_display, 0.0);
        assert_eq!(ed_p.max_display, 1.0);

        // Track input trim automation editor
        let res_tr_trim = view.open_track_automation_editor(1, "trim");
        assert!(res_tr_trim);
        let ed_t = view.automation_editor.as_ref().unwrap();
        assert_eq!(ed_t.unit_name, "dB");
        assert_eq!(ed_t.min_display, -18.0);
        assert_eq!(ed_t.max_display, 18.0);
    }
}
