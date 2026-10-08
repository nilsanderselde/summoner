#![allow(clippy::all)]

//! Test suite for Turn #93 (Sprint Turn #19) — Analog Ladder 24dB/oct Filter HUD
//! and Granular Spectral Cloud Freeze & Trajectory HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::ladder_filter_view::{
        LadderFilterView, LadderTopology, LADDER_PUCK_HIT_RADIUS,
    };
    use crate::views::granular_freeze_view::{
        GranularFreezeView, GrainEnvelopeWindow, GRANULAR_FREEZE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn93_pure_ladder_filter_initialization_and_kinematics() {
        let mut view = LadderFilterView::new();
        assert_eq!(view.topology, LadderTopology::MoogTransistor4Pole);
        assert!((view.cutoff_freq_hz - 1450.0).abs() < 1e-3);
        assert!((view.resonance_q - 6.5).abs() < 1e-3);
        assert!((view.drive_saturation - 35.0).abs() < 1e-3);
        assert!((view.key_tracking_percent - 50.0).abs() < 1e-3);
        assert!((view.envelope_depth - 40.0).abs() < 1e-3);
        assert_eq!(LADDER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Topology switching & labels
        let topologies = [
            (LadderTopology::MoogTransistor4Pole, "MOOG 24dB TRANSISTOR"),
            (LadderTopology::Tb303DiodeLadder, "TB-303 DIODE LADDER"),
            (LadderTopology::OberheimSem2Pole, "SEM 12dB 2-POLE"),
            (LadderTopology::Ms20SallenKeyKorg, "MS-20 SALLEN-KEY"),
        ];
        for (topo, label) in topologies {
            view.set_topology(topo);
            assert_eq!(view.topology, topo);
            assert_eq!(topo.label(), label);
        }

        // Coordinate normalizations & round-trips for Cutoff [20..20000 Hz]
        let cutoff_test = 2500.0;
        let cutoff_norm = LadderFilterView::freq_to_normalized(cutoff_test);
        let cutoff_back = LadderFilterView::normalized_to_freq(cutoff_norm);
        assert!((cutoff_back - cutoff_test).abs() < 1.0);

        // Coordinate normalizations & round-trips for Resonance Q [0.0..10.0]
        let q_test = 7.5;
        let q_norm = LadderFilterView::resonance_to_normalized(q_test);
        let q_back = LadderFilterView::normalized_to_resonance(q_norm);
        assert!((q_back - q_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Drive [0.0..100.0 %]
        let drv_test = 65.0;
        let drv_norm = LadderFilterView::drive_to_normalized(drv_test);
        let drv_back = LadderFilterView::normalized_to_drive(drv_norm);
        assert!((drv_back - drv_test).abs() < 1e-3);

        // Filter transfer response evaluation
        let mag_low = view.evaluate_filter_response(100.0);
        let mag_cutoff = view.evaluate_filter_response(view.cutoff_freq_hz);
        let mag_high = view.evaluate_filter_response(18000.0);
        assert!(mag_low > 0.5);
        assert!(mag_cutoff > 0.0);
        assert!(mag_high < mag_low);

        // Saturation curve evaluation
        let sat_pos = view.evaluate_saturation_curve(0.7);
        let sat_neg = view.evaluate_saturation_curve(-0.7);
        assert!(sat_pos > 0.0 && sat_pos <= 1.0);
        assert!(sat_neg < 0.0 && sat_neg >= -1.0);

        // Self-oscillation threshold check
        view.resonance_q = 8.5;
        assert!(view.check_self_oscillation());
        view.resonance_q = 6.0;
        assert!(!view.check_self_oscillation());

        // Filter puck hit testing
        let canvas = Rect::new(20.0, 100.0, 480.0, 230.0);
        let puck_x = canvas.x + view.filter_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.filter_puck_pos.1) * canvas.height;
        assert!(view.hit_test_filter_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_filter_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ANALOG LADDER FILTER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn93_pure_granular_freeze_initialization_and_kinematics() {
        let mut view = GranularFreezeView::new();
        assert_eq!(view.window_type, GrainEnvelopeWindow::HannSmooth);
        assert!((view.grain_size_ms - 120.0).abs() < 1e-3);
        assert!((view.grain_density_hz - 35.0).abs() < 1e-3);
        assert!((view.pitch_spray_st - 7.0).abs() < 1e-3);
        assert!((view.position_jitter_pct - 25.0).abs() < 1e-3);
        assert!(view.is_frozen);
        assert_eq!(view.active_grain_count, 42);
        assert!((view.stereo_spread_pct - 85.0).abs() < 1e-3);
        assert_eq!(GRANULAR_FREEZE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Window envelope types & metadata
        let windows = [
            (GrainEnvelopeWindow::HannSmooth, "HANN BELL (SMOOTH)"),
            (GrainEnvelopeWindow::BlackmanHarris, "BLACKMAN-HARRIS (4-TERM)"),
            (GrainEnvelopeWindow::GaussianBell, "GAUSSIAN BELL (DIFFUSE)"),
            (GrainEnvelopeWindow::TukeyTapered, "TUKEY FLAT (PUNCH)"),
            (GrainEnvelopeWindow::TrapezoidSharp, "TRAPEZOID (CHOP)"),
        ];
        for (win, label) in windows {
            view.set_window_type(win);
            assert_eq!(view.window_type, win);
            assert_eq!(win.label(), label);
        }

        // Coordinate normalizations & round-trips for Grain Size [10..500 ms]
        let size_test = 240.0;
        let size_norm = GranularFreezeView::size_to_normalized(size_test);
        let size_back = GranularFreezeView::normalized_to_size(size_norm);
        assert!((size_back - size_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Density [2..100 grains/s]
        let dens_test = 50.0;
        let dens_norm = GranularFreezeView::density_to_normalized(dens_test);
        let dens_back = GranularFreezeView::normalized_to_density(dens_norm);
        assert!((dens_back - dens_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Pitch Spray [-24..+24 st]
        let spray_test = 12.0;
        let spray_norm = GranularFreezeView::pitch_to_normalized(spray_test);
        let spray_back = GranularFreezeView::normalized_to_pitch(spray_norm);
        assert!((spray_back - spray_test).abs() < 1e-3);

        // Window envelope evaluation across phases
        for &(win, _) in &windows {
            view.set_window_type(win);
            let w_mid = view.evaluate_window_envelope(0.5);
            let w_edge = view.evaluate_window_envelope(0.0);
            assert!(w_mid > 0.0 && w_mid <= 1.0);
            assert!(w_edge >= 0.0 && w_edge <= 0.2);
        }

        // Simulated grain particle evaluation
        let particle = view.evaluate_grain_particle(0);
        assert!(particle.pos_norm >= 0.0 && particle.pos_norm <= 1.0);
        assert!(particle.amplitude >= 0.0 && particle.amplitude <= 1.0);

        // Freeze puck hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let puck_x = canvas.x + view.cloud_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.cloud_puck_pos.1) * canvas.height;
        assert!(view.hit_test_freeze_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_freeze_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("GRANULAR CLOUD FREEZE HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn93_gui_modal_window_lifecycles_ladder_filter_and_granular_freeze() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Analog Ladder Filter HUD
        assert!(!view.is_ladder_filter_hud_open());
        view.open_ladder_filter_hud();
        assert!(view.is_ladder_filter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_ladder_filter_hud();
        assert!(!view.is_ladder_filter_hud_open());

        // 2. Granular Spectral Cloud Freeze HUD
        assert!(!view.is_granular_freeze_hud_open());
        view.open_granular_freeze_hud();
        assert!(view.is_granular_freeze_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_granular_freeze_hud();
        assert!(!view.is_granular_freeze_hud_open());
    }

    #[test]
    fn test_turn93_gui_lock_free_param_bus_dispatch_ladder_filter_and_granular_freeze() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Ladder Filter (offsets 0..=4) and Granular Freeze (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Ladder Filter sync
        view.ladder_filter_view.cutoff_freq_hz = 2400.0;
        view.ladder_filter_view.resonance_q = 8.5;
        view.ladder_filter_view.drive_saturation = 55.0;
        view.ladder_filter_view.key_tracking_percent = 75.0;
        view.ladder_filter_view.envelope_depth = 60.0;

        view.sync_ladder_filter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("cutoff_freq_hz"), Some(&2400.0));
        assert_eq!(view.device_rack_state.node_param_values.get("resonance_q"), Some(&8.5));
        assert_eq!(view.device_rack_state.node_param_values.get("drive_saturation"), Some(&55.0));
        assert_eq!(view.device_rack_state.node_param_values.get("key_tracking_percent"), Some(&75.0));
        assert_eq!(view.device_rack_state.node_param_values.get("envelope_depth"), Some(&60.0));

        let pid_cut = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_res = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_cut), Some(2400.0));
        assert_eq!(arc_bus.get(pid_res), Some(8.5));

        // Test Granular Freeze sync
        view.granular_freeze_view.grain_size_ms = 180.0;
        view.granular_freeze_view.grain_density_hz = 65.0;
        view.granular_freeze_view.pitch_spray_st = 14.0;
        view.granular_freeze_view.position_jitter_pct = 40.0;
        view.granular_freeze_view.stereo_spread_pct = 95.0;
        view.granular_freeze_view.is_frozen = true;

        view.sync_granular_freeze_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("grain_size_ms"), Some(&180.0));
        assert_eq!(view.device_rack_state.node_param_values.get("grain_density_hz"), Some(&65.0));
        assert_eq!(view.device_rack_state.node_param_values.get("pitch_spray_st"), Some(&14.0));
        assert_eq!(view.device_rack_state.node_param_values.get("position_jitter_pct"), Some(&40.0));
        assert_eq!(view.device_rack_state.node_param_values.get("stereo_spread_pct"), Some(&95.0));
        assert_eq!(view.device_rack_state.node_param_values.get("is_frozen"), Some(&1.0));

        let pid_size = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_dens = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_size), Some(180.0));
        assert_eq!(arc_bus.get(pid_dens), Some(65.0));
    }

    #[test]
    fn test_turn93_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Ladder Filter trigger propagation from rack
        view.device_rack_state.requested_open_ladder_filter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_ladder_filter_hud);
        assert!(view.is_ladder_filter_hud_open());
        view.close_ladder_filter_hud();

        // 2. Granular Freeze trigger propagation from inspector
        view.inspector_state.requested_open_granular_freeze_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_granular_freeze_hud);
        assert!(view.is_granular_freeze_hud_open());
        view.close_granular_freeze_hud();
    }

    #[test]
    fn test_turn93_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let ladder_action = palette.actions.iter().find(|a| a.action_id == "open_ladder_filter_hud");
        assert!(ladder_action.is_some(), "open_ladder_filter_hud must be registered in command palette");
        let ladder = ladder_action.unwrap();
        assert_eq!(ladder.category, "Filter / EQ");

        let granular_action = palette.actions.iter().find(|a| a.action_id == "open_granular_freeze_hud");
        assert!(granular_action.is_some(), "open_granular_freeze_hud must be registered in command palette");
        let granular = granular_action.unwrap();
        assert_eq!(granular.category, "Granular / Time");
    }
}
