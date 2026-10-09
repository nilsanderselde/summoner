#![allow(clippy::all)]

//! Test suite for Turn #100 (Sprint Turn #26) — Psychoacoustic Subharmonic Synth & Phase Alignment HUD
//! and Analog Tape Wow/Flutter & Hysteresis Saturation HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::subharmonic_synth_view::{
        SubharmonicProfile, SubharmonicSynthView, SUBHARMONIC_PUCK_HIT_RADIUS,
    };
    use crate::views::tape_flutter_view::{
        TapeFormula, TapeSpeed, TapeFlutterView, TAPE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn100_pure_subharmonic_synth_initialization_and_kinematics() {
        let mut view = SubharmonicSynthView::new();
        assert_eq!(view.profile, SubharmonicProfile::DualOctaveAligned);
        assert!((view.crossover_freq_hz - 80.0).abs() < 1e-3);
        assert!((view.sub_drive_db - 4.5).abs() < 1e-3);
        assert!((view.phase_alignment_deg - 0.0).abs() < 1e-3);
        assert!((view.dry_wet_mix_pct - 70.0).abs() < 1e-3);
        assert_eq!(SUBHARMONIC_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profile switching & names
        let profiles = [
            (SubharmonicProfile::SubOctave12st, "SUB -1 OCT (-12st)"),
            (SubharmonicProfile::SubOctave24st, "SUB -2 OCT (-24st)"),
            (SubharmonicProfile::SubFifth19st, "SUB 5TH (-19st)"),
            (SubharmonicProfile::DualOctaveAligned, "DUAL SUB ALIGNED"),
            (SubharmonicProfile::SaturatedTransient, "SATURATED TRANSIENT"),
        ];

        for (profile, expected_name) in profiles {
            view.set_profile(profile);
            assert_eq!(view.profile, profile);
            assert_eq!(profile.name(), expected_name);
            assert_eq!(profile.profile_name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Frequency [20.0..160.0 Hz]
        let f_test = 85.0;
        let f_norm = SubharmonicSynthView::freq_to_normalized(f_test);
        let f_back = SubharmonicSynthView::normalized_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        // Drive [-24.0..+18.0 dB]
        let d_test = 6.5;
        let d_norm = SubharmonicSynthView::drive_to_normalized(d_test);
        let d_back = SubharmonicSynthView::normalized_to_drive(d_norm);
        assert!((d_back - d_test).abs() < 1e-3);

        // Phase [-180.0..+180.0 deg]
        let p_test = 45.0;
        let p_norm = SubharmonicSynthView::phase_to_normalized(p_test);
        let p_back = SubharmonicSynthView::normalized_to_phase(p_norm);
        assert!((p_back - p_test).abs() < 1e-3);

        // Mix [0.0..100.0 %]
        let m_test = 62.5;
        let m_norm = SubharmonicSynthView::mix_to_normalized(m_test);
        let m_back = SubharmonicSynthView::normalized_to_mix(m_norm);
        assert!((m_back - m_test).abs() < 1e-3);

        // Synthesis model updates
        view.phase_alignment_deg = 0.0;
        view.update_synthesis_model();
        assert!((view.phase_correlation - 1.0).abs() < 1e-3);

        view.phase_alignment_deg = 180.0;
        view.update_synthesis_model();
        assert!((view.phase_correlation - (-1.0)).abs() < 1e-3);

        // Sub puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.sub_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.sub_puck_pos.1) * canvas.height;
        assert!(view.hit_test_sub_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_sub_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PSYCHOACOUSTIC SUBHARMONIC SYNTH HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn100_pure_tape_flutter_initialization_and_kinematics() {
        let mut view = TapeFlutterView::new();
        assert_eq!(view.tape_speed, TapeSpeed::Ips15);
        assert_eq!(view.tape_formula, TapeFormula::Type4Master911);
        assert!((view.saturation_drive_db - 6.5).abs() < 1e-3);
        assert!((view.wow_depth_percent - 35.0).abs() < 1e-3);
        assert!((view.wow_rate_hz - 0.85).abs() < 1e-3);
        assert!((view.flutter_depth_percent - 25.0).abs() < 1e-3);
        assert!((view.flutter_rate_hz - 28.0).abs() < 1e-3);
        assert_eq!(TAPE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Speed switching & names
        let speeds = [
            (TapeSpeed::Ips3_75, "3.75 ips (Lo-Fi Vintage)"),
            (TapeSpeed::Ips7_5, "7.5 ips (Classic Reel)"),
            (TapeSpeed::Ips15, "15 ips (Studio Punch)"),
            (TapeSpeed::Ips30, "30 ips (Mastering Grade)"),
        ];

        for (speed, expected_name) in speeds {
            view.set_speed(speed);
            assert_eq!(view.tape_speed, speed);
            assert_eq!(speed.name(), expected_name);
        }

        // Formula switching & names
        let formulas = [
            (TapeFormula::Type1Ferric, "Type I (Ferric Normal)"),
            (TapeFormula::Type2Chrome, "Type II (Chrome High)"),
            (TapeFormula::Type3Ferrochrome, "Type III (Ferrochrome Mid)"),
            (TapeFormula::Type4Master911, "Type IV (Master 911)"),
        ];

        for (formula, expected_name) in formulas {
            view.set_formula(formula);
            assert_eq!(view.tape_formula, formula);
            assert_eq!(formula.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Saturation Drive [-12.0..+24.0 dB]
        let dr_test = 8.5;
        let dr_norm = TapeFlutterView::drive_to_normalized(dr_test);
        let dr_back = TapeFlutterView::normalized_to_drive(dr_norm);
        assert!((dr_back - dr_test).abs() < 1e-3);

        // Modulation [0.0..100.0 %]
        let m_test = 42.0;
        let m_norm = TapeFlutterView::modulation_to_normalized(m_test);
        let m_back = TapeFlutterView::normalized_to_modulation(m_norm);
        assert!((m_back - m_test).abs() < 1e-3);

        // Wow Rate [0.2..3.0 Hz]
        let wr_test = 1.25;
        let wr_norm = TapeFlutterView::wow_rate_to_normalized(wr_test);
        let wr_back = TapeFlutterView::normalized_to_wow_rate(wr_norm);
        assert!((wr_back - wr_test).abs() < 1e-3);

        // Flutter Depth [0.0..100.0 %]
        let fd_test = 33.0;
        let fd_norm = TapeFlutterView::flutter_depth_to_normalized(fd_test);
        let fd_back = TapeFlutterView::normalized_to_flutter_depth(fd_norm);
        assert!((fd_back - fd_test).abs() < 1e-3);

        // Flutter Rate [10.0..50.0 Hz]
        let fr_test = 35.0;
        let fr_norm = TapeFlutterView::flutter_rate_to_normalized(fr_test);
        let fr_back = TapeFlutterView::normalized_to_flutter_rate(fr_norm);
        assert!((fr_back - fr_test).abs() < 1e-3);

        // Bias Calibration [-6.0..+6.0 dB]
        let b_test = 2.5;
        let b_norm = TapeFlutterView::bias_to_normalized(b_test);
        let b_back = TapeFlutterView::normalized_to_bias(b_norm);
        assert!((b_back - b_test).abs() < 1e-3);

        // Azimuth Skew [-5.0..+5.0 deg]
        let az_test = -1.2;
        let az_norm = TapeFlutterView::azimuth_to_normalized(az_test);
        let az_back = TapeFlutterView::normalized_to_azimuth(az_norm);
        assert!((az_back - az_test).abs() < 1e-3);

        // Curve evaluations
        assert!(view.evaluate_hysteresis_curve(0.0).abs() < 1e-3);
        assert!(view.evaluate_hysteresis_curve(5.0) <= 1.0);
        let speed_mod = view.evaluate_speed_modulation(0.2);
        assert!(speed_mod > 0.9 && speed_mod < 1.1);

        // Tape puck hit testing
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.tape_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.tape_puck_pos.1) * canvas.height;
        assert!(view.hit_test_tape_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_tape_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ANALOG TAPE WOW/FLUTTER HUD"));
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
    fn test_turn100_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Psychoacoustic Subharmonic Synth HUD
        assert!(!view.is_subharmonic_synth_hud_open());
        view.open_subharmonic_synth_hud();
        assert!(view.is_subharmonic_synth_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_subharmonic_synth_hud();
        assert!(!view.is_subharmonic_synth_hud_open());

        // 2. Analog Tape Wow/Flutter HUD
        assert!(!view.is_tape_flutter_hud_open());
        view.open_tape_flutter_hud();
        assert!(view.is_tape_flutter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_tape_flutter_hud();
        assert!(!view.is_tape_flutter_hud_open());
    }

    #[test]
    fn test_turn100_gui_lock_free_param_bus_dispatch_subharmonic_and_tape_flutter() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Subharmonic Synth (offsets 0..=3) and Tape Flutter (offsets 0..=3)
        for offset in 0..=4 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Subharmonic Synth sync
        view.subharmonic_synth_view.crossover_freq_hz = 95.0;
        view.subharmonic_synth_view.sub_drive_db = 8.0;
        view.subharmonic_synth_view.phase_alignment_deg = 30.0;
        view.subharmonic_synth_view.dry_wet_mix_pct = 85.0;

        view.sync_subharmonic_synth_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("crossover_freq_hz"), Some(&95.0));
        assert_eq!(view.device_rack_state.node_param_values.get("sub_drive_db"), Some(&8.0));
        assert_eq!(view.device_rack_state.node_param_values.get("phase_alignment_deg"), Some(&30.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet_mix_pct"), Some(&85.0));

        let pid_drive = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_freq = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_phase = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_mix = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_drive).unwrap() - 8.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_freq).unwrap() - 95.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_phase).unwrap() - 30.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_mix).unwrap() - 85.0).abs() < 1e-3);

        // Test Tape Flutter sync
        view.tape_flutter_view.saturation_drive_db = 10.5;
        view.tape_flutter_view.wow_depth_percent = 45.0;
        view.tape_flutter_view.flutter_depth_percent = 30.0;
        view.tape_flutter_view.bias_calibration_db = 2.0;

        view.sync_tape_flutter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("saturation_drive_db"), Some(&10.5));
        assert_eq!(view.device_rack_state.node_param_values.get("wow_depth_percent"), Some(&45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("flutter_depth_percent"), Some(&30.0));
        assert_eq!(view.device_rack_state.node_param_values.get("bias_calibration_db"), Some(&2.0));

        let pid_dr = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_wow = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_flut = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_bias = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_dr).unwrap() - 10.5).abs() < 1e-3);
        assert!((arc_bus.get(pid_wow).unwrap() - 45.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_flut).unwrap() - 30.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_bias).unwrap() - 2.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn100_gui_inspector_and_rack_trigger_buttons() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Modern Inspector Trigger Propagation for Subharmonic Synth
        view.inspector_state.selected_node_kind = Some("SubharmonicSynthNode".to_string());
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

        view.inspector_state.requested_open_subharmonic_synth_hud = true;
        assert!(!view.is_subharmonic_synth_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_subharmonic_synth_hud_open());
        view.close_subharmonic_synth_hud();

        // 2. Modern Inspector Trigger Propagation for Tape Flutter
        view.inspector_state.selected_node_kind = Some("TapeFlutterNode".to_string());
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

        view.inspector_state.requested_open_tape_flutter_hud = true;
        assert!(!view.is_tape_flutter_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_tape_flutter_hud_open());
        view.close_tape_flutter_hud();

        // 3. Modern Device Rack Trigger Propagation for Subharmonic Synth
        view.device_rack_state.selected_node_kind = Some("SubharmonicSynth".to_string());
        view.device_rack_state.requested_open_subharmonic_synth_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_subharmonic_synth_hud_open());
        view.close_subharmonic_synth_hud();

        // 4. Modern Device Rack Trigger Propagation for Tape Flutter
        view.device_rack_state.selected_node_kind = Some("TapeFlutter".to_string());
        view.device_rack_state.requested_open_tape_flutter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_tape_flutter_hud_open());
        view.close_tape_flutter_hud();
    }
}
