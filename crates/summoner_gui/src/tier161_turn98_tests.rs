#![allow(clippy::all)]

//! Test suite for Turn #98 (Sprint Turn #24 — Sprint Review) — Frequency Shifter & SSB Quadrature Modulator HUD
//! and Vintage Optical & VCA Compressor HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::frequency_shifter_view::{
        SidebandMode, FrequencyShifterView, FREQ_SHIFTER_PUCK_HIT_RADIUS,
    };
    use crate::views::optical_compressor_view::{
        CompressorTopology, OpticalCompressorView, COMPRESSOR_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn98_pure_frequency_shifter_initialization_and_kinematics() {
        let mut view = FrequencyShifterView::new();
        assert_eq!(view.mode, SidebandMode::UpperSSB);
        assert!((view.shift_hz - 0.0).abs() < 1e-3);
        assert!((view.fine_hz - 0.0).abs() < 1e-3);
        assert!((view.quadrature_phase_deg - 90.0).abs() < 1e-3);
        assert!((view.feedback_pct - 30.0).abs() < 1e-3);
        assert!((view.dry_wet_pct - 100.0).abs() < 1e-3);
        assert_eq!(FREQ_SHIFTER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (SidebandMode::UpperSSB, "UPPER SSB (USB)"),
            (SidebandMode::LowerSSB, "LOWER SSB (LSB)"),
            (SidebandMode::DualBode, "DUAL BODE STEREO"),
            (SidebandMode::RingMod, "RING MODULATION"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Shift [-5000.0..+5000.0 Hz]
        let shift_test = -1450.0;
        let shift_norm = FrequencyShifterView::shift_to_normalized(shift_test);
        let shift_back = FrequencyShifterView::normalized_to_shift(shift_norm);
        assert!((shift_back - shift_test).abs() < 1e-3);

        // Phase [0.0..360.0 deg]
        let phase_test = 270.0;
        let phase_norm = FrequencyShifterView::phase_to_normalized(phase_test);
        let phase_back = FrequencyShifterView::normalized_to_phase(phase_norm);
        assert!((phase_back - phase_test).abs() < 1e-3);

        // Feedback [0.0..95.0 %]
        let fb_test = 65.0;
        let fb_norm = FrequencyShifterView::feedback_to_normalized(fb_test);
        let fb_back = FrequencyShifterView::normalized_to_feedback(fb_norm);
        assert!((fb_back - fb_test).abs() < 1e-3);

        // Dry/Wet [0.0..100.0 %]
        let dw_test = 80.0;
        let dw_norm = FrequencyShifterView::dry_wet_to_normalized(dw_test);
        let dw_back = FrequencyShifterView::normalized_to_dry_wet(dw_norm);
        assert!((dw_back - dw_test).abs() < 1e-3);

        // Quadrature trajectory points generation
        let traj = view.generate_quadrature_trajectory(24);
        assert_eq!(traj.len(), 24);
        for (i_val, q_val) in traj {
            assert!(i_val.is_finite());
            assert!(q_val.is_finite());
        }

        // Orbital puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.orbital_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.orbital_puck_pos.1) * canvas.height;
        assert!(view.hit_test_orbital_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_orbital_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("FREQUENCY SHIFTER & SSB MODULATOR HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn98_pure_optical_compressor_initialization_and_kinematics() {
        let mut view = OpticalCompressorView::new();
        assert_eq!(view.topology, CompressorTopology::OptoT4BTeletronix);
        assert!((view.threshold_db - (-18.0)).abs() < 1e-3);
        assert!((view.ratio - 4.0).abs() < 1e-3);
        assert!((view.knee_width_db - 8.0).abs() < 1e-3);
        assert!((view.makeup_gain_db - 0.0).abs() < 1e-3);
        assert!((view.attack_ms - 10.0).abs() < 1e-3);
        assert!((view.release_ms - 150.0).abs() < 1e-3);
        assert_eq!(COMPRESSOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Topology switching & names
        let topologies = [
            (CompressorTopology::OptoT4BTeletronix, "OPTO T4B TELETRONIX"),
            (CompressorTopology::VcaFastFeedForward, "VCA FAST FEED-FORWARD"),
            (CompressorTopology::VariMuTubeVariableGain, "VARI-MU TUBE VARIABLE-GAIN"),
            (CompressorTopology::FetPeakLimiter1176, "FET PEAK LIMITER 1176"),
        ];

        for (topo, expected_name) in topologies {
            view.set_topology(topo);
            assert_eq!(view.topology, topo);
            assert_eq!(topo.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for dB [-60.0..0.0 dB]
        let db_test = -24.5;
        let db_norm = OpticalCompressorView::db_to_normalized(db_test);
        let db_back = OpticalCompressorView::normalized_to_db(db_norm);
        assert!((db_back - db_test).abs() < 1e-3);

        // Ratio [1.0..20.0]
        let ratio_test = 6.5;
        let ratio_norm = OpticalCompressorView::ratio_to_normalized(ratio_test);
        let ratio_back = OpticalCompressorView::normalized_to_ratio(ratio_norm);
        assert!((ratio_back - ratio_test).abs() < 1e-3);

        // Knee width [0.0..24.0 dB]
        let knee_test = 12.0;
        let knee_norm = OpticalCompressorView::knee_to_normalized(knee_test);
        let knee_back = OpticalCompressorView::normalized_to_knee(knee_norm);
        assert!((knee_back - knee_test).abs() < 1e-3);

        // Makeup gain [-12.0..+24.0 dB]
        let makeup_test = 4.5;
        let makeup_norm = OpticalCompressorView::makeup_to_normalized(makeup_test);
        let makeup_back = OpticalCompressorView::normalized_to_makeup(makeup_norm);
        assert!((makeup_back - makeup_test).abs() < 1e-3);

        // Attack [0.02..100.0 ms]
        let att_test = 25.0;
        let att_norm = OpticalCompressorView::attack_to_normalized(att_test);
        let att_back = OpticalCompressorView::normalized_to_attack(att_norm);
        assert!((att_back - att_test).abs() < 1e-3);

        // Release [10.0..2500.0 ms]
        let rel_test = 450.0;
        let rel_norm = OpticalCompressorView::release_to_normalized(rel_test);
        let rel_back = OpticalCompressorView::normalized_to_release(rel_norm);
        assert!((rel_back - rel_test).abs() < 1e-3);

        // Transfer curve evaluation below and above threshold
        let below = view.evaluate_transfer_curve(-30.0);
        assert!((below - (-30.0)).abs() < 1e-3);

        let above = view.evaluate_transfer_curve(0.0);
        assert!(above < 0.0); // Compressed

        // Knee puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.knee_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.knee_puck_pos.1) * canvas.height;
        assert!(view.hit_test_knee_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_knee_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("VINTAGE OPTICAL & VCA COMPRESSOR HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::show_modern_inspector_with_context;

    #[test]
    fn test_turn98_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Frequency Shifter & SSB Modulator HUD
        assert!(!view.is_frequency_shifter_hud_open());
        view.open_frequency_shifter_hud();
        assert!(view.is_frequency_shifter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_frequency_shifter_hud();
        assert!(!view.is_frequency_shifter_hud_open());

        // 2. Vintage Optical & VCA Compressor HUD
        assert!(!view.is_optical_compressor_hud_open());
        view.open_optical_compressor_hud();
        assert!(view.is_optical_compressor_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_optical_compressor_hud();
        assert!(!view.is_optical_compressor_hud_open());
    }

    #[test]
    fn test_turn98_gui_lock_free_param_bus_dispatch_frequency_shifter_and_optical_compressor() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Frequency Shifter (offsets 0..=4) and Optical Compressor (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Frequency Shifter sync
        view.frequency_shifter_view.shift_hz = 250.0;
        view.frequency_shifter_view.fine_hz = 2.5;
        view.frequency_shifter_view.quadrature_phase_deg = 180.0;
        view.frequency_shifter_view.feedback_pct = 40.0;
        view.frequency_shifter_view.dry_wet_pct = 85.0;

        view.sync_frequency_shifter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("shift_hz"), Some(&250.0));
        assert_eq!(view.device_rack_state.node_param_values.get("fine_shift_hz"), Some(&2.5));
        assert_eq!(view.device_rack_state.node_param_values.get("quadrature_phase_deg"), Some(&180.0));
        assert_eq!(view.device_rack_state.node_param_values.get("feedback_pct"), Some(&40.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet_pct"), Some(&85.0));

        let pid_shift = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_fine = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_phase = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_fb = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        let pid_wet = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 4);

        assert!((arc_bus.get(pid_shift).unwrap() - 250.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_fine).unwrap() - 2.5).abs() < 1e-3);
        assert!((arc_bus.get(pid_phase).unwrap() - 180.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_fb).unwrap() - 40.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_wet).unwrap() - 85.0).abs() < 1e-3);

        // Test Optical Compressor sync
        view.optical_compressor_view.threshold_db = -24.0;
        view.optical_compressor_view.ratio = 8.0;
        view.optical_compressor_view.knee_width_db = 10.0;
        view.optical_compressor_view.makeup_gain_db = 6.0;
        view.optical_compressor_view.attack_ms = 15.0;
        view.optical_compressor_view.release_ms = 250.0;

        view.sync_optical_compressor_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("threshold_db"), Some(&-24.0));
        assert_eq!(view.device_rack_state.node_param_values.get("ratio"), Some(&8.0));
        assert_eq!(view.device_rack_state.node_param_values.get("knee_width_db"), Some(&10.0));
        assert_eq!(view.device_rack_state.node_param_values.get("makeup_gain_db"), Some(&6.0));
        assert_eq!(view.device_rack_state.node_param_values.get("attack_ms"), Some(&15.0));
        assert_eq!(view.device_rack_state.node_param_values.get("release_ms"), Some(&250.0));

        let pid_thresh = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_ratio = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_knee = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_makeup = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        let pid_att = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 4);
        let pid_rel = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 5);

        assert!((arc_bus.get(pid_thresh).unwrap() - (-24.0)).abs() < 1e-3);
        assert!((arc_bus.get(pid_ratio).unwrap() - 8.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_knee).unwrap() - 10.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_makeup).unwrap() - 6.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_att).unwrap() - 15.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_rel).unwrap() - 250.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn98_gui_inspector_and_rack_trigger_buttons() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Modern Inspector Trigger Propagation for Frequency Shifter
        view.inspector_state.selected_node_kind = Some("FrequencyShifterNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(
                    ui,
                    &mut view.inspector_state,
                    view.live_param_bus.as_deref(),
                    1,
                );
            });
        });

        view.inspector_state.requested_open_frequency_shifter_hud = true;
        assert!(!view.is_frequency_shifter_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_frequency_shifter_hud_open());
        view.close_frequency_shifter_hud();

        // 2. Modern Inspector Trigger Propagation for Optical Compressor
        view.inspector_state.selected_node_kind = Some("OpticalCompressorNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(
                    ui,
                    &mut view.inspector_state,
                    view.live_param_bus.as_deref(),
                    1,
                );
            });
        });

        view.inspector_state.requested_open_optical_compressor_hud = true;
        assert!(!view.is_optical_compressor_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_optical_compressor_hud_open());
        view.close_optical_compressor_hud();

        // 3. Modern Device Rack Trigger Propagation for Frequency Shifter
        view.device_rack_state.selected_node_kind = Some("FrequencyShifter".to_string());
        view.device_rack_state.requested_open_frequency_shifter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_frequency_shifter_hud_open());
        view.close_frequency_shifter_hud();

        // 4. Modern Device Rack Trigger Propagation for Optical Compressor
        view.device_rack_state.selected_node_kind = Some("OpticalCompressor".to_string());
        view.device_rack_state.requested_open_optical_compressor_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_optical_compressor_hud_open());
        view.close_optical_compressor_hud();
    }
}
