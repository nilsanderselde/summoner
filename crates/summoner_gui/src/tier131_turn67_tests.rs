#![allow(clippy::all)]

//! Test suite for Turn #67 / Turn #30 deliverable — Physical Modeling Woodwind Air-Jet Embouchure & 6-Tonehole Acoustic Radiation Matrix HUDs Convergence (Milestones 17, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::woodwind_jet_view::{
        WoodwindJetView, WoodwindInstrument, WOODWIND_PUCK_HIT_RADIUS,
        MIN_JET_PRESSURE_KPA, MAX_JET_PRESSURE_KPA, MIN_JET_OFFSET_MM, MAX_JET_OFFSET_MM,
        MIN_TUBE_LENGTH_M, MAX_TUBE_LENGTH_M,
    };
    use crate::views::tonehole_matrix_view::{
        ToneholeMatrixView, ToneholeDisplayMode, TONEHOLE_PUCK_HIT_RADIUS,
        MIN_CUTOFF_HZ, MAX_CUTOFF_HZ, MIN_BORE_LENGTH_M, MAX_BORE_LENGTH_M,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn67_pure_woodwind_jet_view_initialization_and_instruments() {
        let mut view = WoodwindJetView::new();
        assert_eq!(view.instrument, WoodwindInstrument::FluteC);
        assert!((view.jet_pressure_kpa - 1.25).abs() < 1e-3);
        assert!((view.jet_offset_mm - 7.0).abs() < 1e-3);
        assert_eq!(view.tonehole_state, [true; 6]);
        assert!((view.jet_velocity_ms - 45.6).abs() < 1.0);
        assert!((view.effective_bore_length_m - 0.60).abs() < 1e-3);
        assert!(view.jet_pressure_kpa >= MIN_JET_PRESSURE_KPA && view.jet_pressure_kpa <= MAX_JET_PRESSURE_KPA);
        assert!(view.jet_offset_mm >= MIN_JET_OFFSET_MM && view.jet_offset_mm <= MAX_JET_OFFSET_MM);
        assert!(view.effective_bore_length_m >= MIN_TUBE_LENGTH_M && view.effective_bore_length_m <= MAX_TUBE_LENGTH_M);
        assert_eq!(WOODWIND_PUCK_HIT_RADIUS, 22.0);

        // Test instruments
        view.set_instrument(WoodwindInstrument::PiccoloC);
        assert_eq!(view.instrument, WoodwindInstrument::PiccoloC);
        assert!((view.jet_offset_mm - 4.5).abs() < 1e-3);
        assert!((view.tonehole_cutoff_hz - 4400.0).abs() < 1.0);

        view.set_instrument(WoodwindInstrument::RecorderAlto);
        assert_eq!(view.instrument, WoodwindInstrument::RecorderAlto);
        assert!((view.jet_offset_mm - 3.5).abs() < 1e-3);
        assert!((view.tonehole_cutoff_hz - 1800.0).abs() < 1.0);

        view.set_instrument(WoodwindInstrument::Shakuhachi);
        assert_eq!(view.instrument, WoodwindInstrument::Shakuhachi);
        assert!((view.jet_offset_mm - 10.0).abs() < 1e-3);
        assert!((view.tonehole_cutoff_hz - 1600.0).abs() < 1.0);

        view.set_instrument(WoodwindInstrument::PanFlute);
        assert_eq!(view.instrument, WoodwindInstrument::PanFlute);
        assert!((view.jet_offset_mm - 5.0).abs() < 1e-3);
        assert!(view.instrument.is_stopped_pipe());

        // Normalized conversions
        let norm_p = WoodwindJetView::pressure_to_normalized(view.jet_pressure_kpa);
        let back_p = WoodwindJetView::normalized_to_pressure(norm_p);
        assert!((back_p - view.jet_pressure_kpa).abs() < 1e-3);

        let norm_o = WoodwindJetView::offset_to_normalized(view.jet_offset_mm);
        let back_o = WoodwindJetView::normalized_to_offset(norm_o);
        assert!((back_o - view.jet_offset_mm).abs() < 1e-3);

        let norm_l = WoodwindJetView::length_to_normalized(view.effective_bore_length_m);
        let back_l = WoodwindJetView::normalized_to_length(norm_l);
        assert!((back_l - view.effective_bore_length_m).abs() < 1e-3);

        // Effective tube length with open holes
        view.tonehole_state[0] = false;
        view.tonehole_state[1] = false;
        let shorter_len = view.calculate_effective_tube_length();
        assert!(shorter_len < view.instrument.nominal_tube_length_m());

        // Physics update
        view.jet_pressure_kpa = 2.50;
        view.jet_offset_mm = 8.0;
        view.update_physics_simulation();
        assert!(view.jet_velocity_ms > 50.0);
        assert!(view.jet_delay_ms > 0.0);
        assert!(view.acoustic_coupling_score > 0.0 && view.acoustic_coupling_score <= 1.0);

        // Hit testing
        let canvas = Rect { x: 40.0, y: 80.0, width: 200.0, height: 160.0 };
        let puck_x = canvas.x + view.jet_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.jet_puck_pos.1) * canvas.height;
        assert!(view.hit_test_jet_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_jet_puck((puck_x + 10.0, puck_y - 10.0), canvas));
        assert!(!view.hit_test_jet_puck((puck_x + 40.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("="));
    }

    #[test]
    fn test_turn67_pure_tonehole_matrix_view_initialization_and_modes() {
        let mut view = ToneholeMatrixView::new();
        assert_eq!(view.display_mode, ToneholeDisplayMode::StandingWaveProfile);
        assert!((view.bore_length_m - 0.60).abs() < 1e-3);
        assert!((view.fundamental_hz - 283.3).abs() < 2.0);
        assert!((view.lattice_cutoff_hz - 1675.0).abs() < 20.0);
        assert!((view.radiated_power_mw - 14.5).abs() < 4.0);
        assert!(view.bore_length_m >= MIN_BORE_LENGTH_M && view.bore_length_m <= MAX_BORE_LENGTH_M);
        assert!(view.lattice_cutoff_hz >= MIN_CUTOFF_HZ && view.lattice_cutoff_hz <= MAX_CUTOFF_HZ);
        assert_eq!(TONEHOLE_PUCK_HIT_RADIUS, 22.0);

        // Display modes
        view.set_display_mode(ToneholeDisplayMode::RadiationImpedanceSpectrum);
        assert_eq!(view.display_mode, ToneholeDisplayMode::RadiationImpedanceSpectrum);

        view.set_display_mode(ToneholeDisplayMode::ScatteringMatrix3Port);
        assert_eq!(view.display_mode, ToneholeDisplayMode::ScatteringMatrix3Port);

        // Fingering mask (open holes)
        view.set_fingering_mask(0b001111); // 4 closed, 2 open
        assert!(view.hole_open_fractions[0] < 0.05);
        assert!(view.hole_open_fractions[4] > 0.95);
        assert!(view.fundamental_hz > 260.0);

        // Standing wave evaluation
        let p_start = view.evaluate_standing_wave_pressure(0.0);
        let p_mid = view.evaluate_standing_wave_pressure(0.3);
        let p_end = view.evaluate_standing_wave_pressure(1.0);
        assert!(p_mid.abs() > p_start.abs());
        assert!(p_end.abs() < 0.2);

        // Reflection magnitude
        let r_low = view.evaluate_reflection_magnitude(500.0);
        let r_high = view.evaluate_reflection_magnitude(5000.0);
        assert!(r_low > r_high);

        // Normalized conversions
        let norm_l = ToneholeMatrixView::length_to_normalized(view.bore_length_m);
        let back_l = ToneholeMatrixView::normalized_to_length(norm_l);
        assert!((back_l - view.bore_length_m).abs() < 1e-3);

        let norm_c = ToneholeMatrixView::cutoff_to_normalized(view.lattice_cutoff_hz);
        let back_c = ToneholeMatrixView::normalized_to_cutoff(norm_c);
        assert!((back_c - view.lattice_cutoff_hz).abs() < 1e-1);

        // Hit testing
        let canvas = Rect { x: 50.0, y: 100.0, width: 220.0, height: 150.0 };
        let puck_x = canvas.x + view.radiation_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.radiation_puck_pos.1) * canvas.height;
        assert!(view.hit_test_radiation_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_radiation_puck((puck_x + 8.0, puck_y + 8.0), canvas));
        assert!(!view.hit_test_radiation_puck((puck_x + 35.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("*"));
    }

    #[test]
    fn test_turn67_pure_parambus_slot_arithmetic_channel_isolation() {
        let track_id = 4u32;
        let slot_idx = 3u32;
        let base_id = track_id * 1000 + slot_idx * 20;

        let pid_press = ParamId(base_id);
        let pid_off   = ParamId(base_id + 1);
        let pid_vel   = ParamId(base_id + 2);
        let pid_coup  = ParamId(base_id + 3);

        assert_eq!(pid_press.0, 4060);
        assert_eq!(pid_off.0, 4061);
        assert_eq!(pid_vel.0, 4062);
        assert_eq!(pid_coup.0, 4063);

        // Verify next slot isolation
        let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
        assert_eq!(next_slot_base, 4080);
        assert!(next_slot_base > pid_coup.0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use std::sync::Arc;

    #[test]
    fn test_turn67_gui_award_winning_woodwind_jet_and_tonehole_matrix_hud_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_woodwind_jet_hud_open());
        assert!(!view.is_tonehole_matrix_hud_open());

        view.open_woodwind_jet_hud();
        assert!(view.is_woodwind_jet_hud_open());
        view.close_woodwind_jet_hud();
        assert!(!view.is_woodwind_jet_hud_open());

        view.open_tonehole_matrix_hud();
        assert!(view.is_tonehole_matrix_hud_open());
        view.close_tonehole_matrix_hud();
        assert!(!view.is_tonehole_matrix_hud_open());
    }

    #[test]
    fn test_turn67_gui_headless_rendering_no_panics() {
        let mut view = AwardWinningGuiView::new();
        view.open_woodwind_jet_hud();
        view.open_tonehole_matrix_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_woodwind_jet_hud_open());
        assert!(view.is_tonehole_matrix_hud_open());
    }

    #[test]
    fn test_turn67_gui_live_parambus_atomic_sync_woodwind_jet_and_tonehole_matrix() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..10 {
            bus.register(ParamId(base + i), 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // 1. Sync Woodwind Jet HUD
        view.woodwind_jet_view.jet_pressure_kpa = 2.10;
        view.woodwind_jet_view.jet_offset_mm = 8.5;
        view.woodwind_jet_view.jet_velocity_ms = 59.2;
        view.woodwind_jet_view.acoustic_coupling_score = 0.88;
        view.sync_woodwind_jet_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(2.10));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(8.5));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(59.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.88));

        assert_eq!(view.device_rack_state.node_param_values.get("jet_pressure_kpa"), Some(&2.10));
        assert_eq!(view.device_rack_state.node_param_values.get("jet_offset_mm"), Some(&8.5));
        assert_eq!(view.inspector_state.node_param_values.get("jet_pressure_kpa"), Some(&2.10));

        // 2. Sync Tonehole Matrix HUD
        view.tonehole_matrix_view.bore_length_m = 0.72;
        view.tonehole_matrix_view.lattice_cutoff_hz = 2850.0;
        view.tonehole_matrix_view.fundamental_hz = 238.3;
        view.tonehole_matrix_view.radiated_power_mw = 18.2;
        view.sync_tonehole_matrix_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.72));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(2850.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(238.3));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(18.2));

        assert_eq!(view.device_rack_state.node_param_values.get("bore_length_m"), Some(&0.72));
        assert_eq!(view.device_rack_state.node_param_values.get("lattice_cutoff_hz"), Some(&2850.0));
        assert_eq!(view.inspector_state.node_param_values.get("bore_length_m"), Some(&0.72));
    }

    #[test]
    fn test_turn67_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let has_jet = palette.actions.iter().any(|a| a.action_id == "open_woodwind_jet_hud");
        let has_th = palette.actions.iter().any(|a| a.action_id == "open_tonehole_matrix_hud");

        assert!(has_jet, "CommandPalette must register open_woodwind_jet_hud");
        assert!(has_th, "CommandPalette must register open_tonehole_matrix_hud");
    }

    #[test]
    fn test_turn67_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Inspector trigger for Woodwind Jet
        view.inspector_state.requested_open_woodwind_jet_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_woodwind_jet_hud_open());
        assert!(!view.inspector_state.requested_open_woodwind_jet_hud);

        view.close_woodwind_jet_hud();

        // Device Rack trigger for Tonehole Matrix
        view.device_rack_state.requested_open_tonehole_matrix_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_tonehole_matrix_hud_open());
        assert!(!view.device_rack_state.requested_open_tonehole_matrix_hud);
    }
}
