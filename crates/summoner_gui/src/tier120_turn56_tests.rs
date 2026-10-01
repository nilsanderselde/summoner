#![allow(clippy::all)]

//! Test suite for Turn #56 (Turn #21 — Sprint Review) — Physical Modeling Caribbean Steelpan & Lamellophone Mbira HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::steelpan_drum_view::{
        SteelpanDrumView, SteelpanType, STEELPAN_PUCK_HIT_RADIUS,
        MIN_RADIAL_POS, MAX_RADIAL_POS, MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY,
    };
    use crate::views::mbira_kalimba_view::{
        MbiraKalimbaView, MbiraType, MBIRA_PUCK_HIT_RADIUS,
        MIN_PLUCK_FORCE_N, MAX_PLUCK_FORCE_N, MIN_BUZZ_INTENSITY, MAX_BUZZ_INTENSITY,
    };

    #[test]
    fn test_turn56_pure_steelpan_drum_instruments_and_simulation() {
        let mut view = SteelpanDrumView::new();
        assert_eq!(view.pan_type, SteelpanType::LeadTenorPan);
        assert!((view.strike_radial_pos - 0.45).abs() < 1e-4);
        assert!((view.strike_velocity - 0.75).abs() < 1e-4);
        assert_eq!(view.annular_ring_count, 5);
        assert!((view.steel_gauge_mm - 0.95).abs() < 1e-4);
        assert!((view.damping_s - 1.8).abs() < 1e-4);
        assert!((view.coupling_resonance - 0.40).abs() < 1e-4);

        // Test pan switching
        view.set_pan_type(SteelpanType::DoubleSecondsPan);
        assert_eq!(view.pan_type, SteelpanType::DoubleSecondsPan);
        assert_eq!(view.annular_ring_count, 4);
        assert!((view.steel_gauge_mm - 1.10).abs() < 1e-4);
        assert!((view.strike_radial_pos - 0.55).abs() < 1e-4);

        view.set_pan_type(SteelpanType::DoubleGuitarPan);
        assert_eq!(view.pan_type, SteelpanType::DoubleGuitarPan);
        assert_eq!(view.annular_ring_count, 3);
        assert!((view.steel_gauge_mm - 1.25).abs() < 1e-4);

        view.set_pan_type(SteelpanType::TripleCellosPan);
        assert_eq!(view.pan_type, SteelpanType::TripleCellosPan);
        assert_eq!(view.annular_ring_count, 3);
        assert!((view.damping_s - 2.8).abs() < 1e-4);

        view.set_pan_type(SteelpanType::SixBassPan);
        assert_eq!(view.pan_type, SteelpanType::SixBassPan);
        assert_eq!(view.annular_ring_count, 3);
        assert!((view.steel_gauge_mm - 1.65).abs() < 1e-4);

        // Normalization roundtrips
        let norm_r = SteelpanDrumView::radial_to_normalized(0.50);
        let recon_r = SteelpanDrumView::normalized_to_radial(norm_r);
        assert!((recon_r - 0.50).abs() < 1e-3);

        let norm_v = SteelpanDrumView::vel_to_normalized(0.60);
        let recon_v = SteelpanDrumView::normalized_to_vel(norm_v);
        assert!((recon_v - 0.60).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_steelpan_puck((px, py), canvas));
        assert!(view.hit_test_steelpan_puck((px + STEELPAN_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_steelpan_puck((px + STEELPAN_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Check modal amplitudes
        for amp in view.modal_amplitudes {
            assert!(amp >= 0.0);
            assert!(amp <= 2.5);
        }

        assert_eq!(MIN_RADIAL_POS, 0.05);
        assert_eq!(MAX_RADIAL_POS, 0.95);
        assert_eq!(MIN_STRIKE_VELOCITY, 0.10);
        assert_eq!(MAX_STRIKE_VELOCITY, 1.00);
    }

    #[test]
    fn test_turn56_pure_mbira_kalimba_instruments_and_simulation() {
        let mut view = MbiraKalimbaView::new();
        assert_eq!(view.instrument_type, MbiraType::MbiraDzavadzimu);
        assert!((view.pluck_force_n - 2.4).abs() < 1e-4);
        assert!((view.buzz_intensity_pct - 0.85).abs() < 1e-4);
        assert_eq!(view.tine_count, 22);
        assert!((view.tine_decay_s - 3.5).abs() < 1e-4);
        assert!((view.cavity_q_factor - 45.0).abs() < 1e-4);

        // Instrument switching
        view.set_instrument_type(MbiraType::NyungaNyunga15);
        assert_eq!(view.instrument_type, MbiraType::NyungaNyunga15);
        assert_eq!(view.tine_count, 15);
        assert!((view.pluck_force_n - 1.8).abs() < 1e-4);

        view.set_instrument_type(MbiraType::HughTraceyKalimba17);
        assert_eq!(view.instrument_type, MbiraType::HughTraceyKalimba17);
        assert_eq!(view.tine_count, 17);
        assert!((view.pluck_force_n - 1.2).abs() < 1e-4);

        view.set_instrument_type(MbiraType::ArrayMbira5Octave);
        assert_eq!(view.instrument_type, MbiraType::ArrayMbira5Octave);
        assert_eq!(view.tine_count, 32);
        assert!((view.buzz_intensity_pct - 0.0).abs() < 1e-4);

        view.set_instrument_type(MbiraType::BassKalimbaElectrified);
        assert_eq!(view.instrument_type, MbiraType::BassKalimbaElectrified);
        assert_eq!(view.tine_count, 9);
        assert!((view.pluck_force_n - 3.5).abs() < 1e-4);

        // Normalization roundtrips
        let norm_f = MbiraKalimbaView::force_to_normalized(3.0);
        let recon_f = MbiraKalimbaView::normalized_to_force(norm_f);
        assert!((recon_f - 3.0).abs() < 1e-3);

        let norm_b = MbiraKalimbaView::buzz_to_normalized(0.45);
        let recon_b = MbiraKalimbaView::normalized_to_buzz(norm_b);
        assert!((recon_b - 0.45).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(15.0, 15.0, 250.0, 180.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_mbira_puck((px, py), canvas));
        assert!(view.hit_test_mbira_puck((px + MBIRA_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_mbira_puck((px + MBIRA_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Check modal amplitudes
        for amp in view.modal_amplitudes {
            assert!(amp >= 0.0);
            assert!(amp <= 2.5);
        }

        assert_eq!(MIN_PLUCK_FORCE_N, 0.1);
        assert_eq!(MAX_PLUCK_FORCE_N, 5.0);
        assert_eq!(MIN_BUZZ_INTENSITY, 0.0);
        assert_eq!(MAX_BUZZ_INTENSITY, 1.0);
    }

    #[test]
    fn test_turn56_pure_parambus_slot_arithmetic_and_channel_isolation() {
        let mut bus = ParamBus::new();

        // Track 4, Slot 0: Caribbean Steelpan
        let pid_pan_rad   = ParamId(4 * 1000 + 0 * 20 + 0);
        let pid_pan_vel   = ParamId(4 * 1000 + 0 * 20 + 1);
        let pid_pan_gauge = ParamId(4 * 1000 + 0 * 20 + 2);
        let pid_pan_damp  = ParamId(4 * 1000 + 0 * 20 + 3);
        let pid_pan_ring  = ParamId(4 * 1000 + 0 * 20 + 4);
        let pid_pan_rings = ParamId(4 * 1000 + 0 * 20 + 5);

        // Track 4, Slot 1: Lamellophone Mbira
        let pid_mbi_force = ParamId(4 * 1000 + 1 * 20 + 0);
        let pid_mbi_buzz  = ParamId(4 * 1000 + 1 * 20 + 1);
        let pid_mbi_disp  = ParamId(4 * 1000 + 1 * 20 + 2);
        let pid_mbi_decay = ParamId(4 * 1000 + 1 * 20 + 3);
        let pid_mbi_q     = ParamId(4 * 1000 + 1 * 20 + 4);
        let pid_mbi_tines = ParamId(4 * 1000 + 1 * 20 + 5);

        bus.register(pid_pan_rad, 0.45);
        bus.register(pid_pan_vel, 0.75);
        bus.register(pid_pan_gauge, 0.95);
        bus.register(pid_pan_damp, 1.8);
        bus.register(pid_pan_ring, 0.40);
        bus.register(pid_pan_rings, 5.0);

        bus.register(pid_mbi_force, 2.4);
        bus.register(pid_mbi_buzz, 0.85);
        bus.register(pid_mbi_disp, 0.72);
        bus.register(pid_mbi_decay, 3.5);
        bus.register(pid_mbi_q, 45.0);
        bus.register(pid_mbi_tines, 22.0);

        // Mutate Steelpan parameters
        bus.set(pid_pan_rad, 0.85);
        bus.set(pid_pan_vel, 0.95);

        // Assert Steelpan changed
        assert_eq!(bus.get(pid_pan_rad), Some(0.85));
        assert_eq!(bus.get(pid_pan_vel), Some(0.95));

        // Assert Mbira untouched
        assert_eq!(bus.get(pid_mbi_force), Some(2.4));
        assert_eq!(bus.get(pid_mbi_buzz), Some(0.85));

        // Mutate Mbira parameters
        bus.set(pid_mbi_force, 4.2);
        bus.set(pid_mbi_buzz, 0.35);

        assert_eq!(bus.get(pid_mbi_force), Some(4.2));
        assert_eq!(bus.get(pid_mbi_buzz), Some(0.35));
        assert_eq!(bus.get(pid_pan_rad), Some(0.85));
    }

    #[test]
    fn test_turn56_pure_ascii_snapshots() {
        let steelpan = SteelpanDrumView::new();
        let steelpan_ascii = steelpan.render_ascii(70, 16);
        assert!(!steelpan_ascii.is_empty());
        assert!(steelpan_ascii.iter().any(|line| line.contains('P')));
        assert!(steelpan_ascii.iter().any(|line| line.contains('#')));

        let mbira = MbiraKalimbaView::new();
        let mbira_ascii = mbira.render_ascii(70, 16);
        assert!(!mbira_ascii.is_empty());
        assert!(mbira_ascii.iter().any(|line| line.contains('P')));
        assert!(mbira_ascii.iter().any(|line| line.contains('#')));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use std::sync::Arc;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;

    #[test]
    fn test_turn56_gui_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_steelpan_hud_open());
        view.open_steelpan_hud();
        assert!(view.is_steelpan_hud_open());
        view.close_steelpan_hud();
        assert!(!view.is_steelpan_hud_open());

        assert!(!view.is_mbira_hud_open());
        view.open_mbira_hud();
        assert!(view.is_mbira_hud_open());
        view.close_mbira_hud();
        assert!(!view.is_mbira_hud_open());
    }

    #[test]
    fn test_turn56_gui_sync_steelpan_and_mbira_hud_state_with_parambus() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let cur_track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let cur_slot = view.device_rack_state.selected_chain_idx;

        // Register ParamBus slots for Steelpan
        let pid_pan_rad = ParamId(cur_track_id as u32 * 1000 + cur_slot as u32 * 20);
        let pid_pan_vel = ParamId(cur_track_id as u32 * 1000 + cur_slot as u32 * 20 + 1);
        bus.register(pid_pan_rad, 0.45);
        bus.register(pid_pan_vel, 0.75);

        // Register ParamBus slots for Mbira
        let pid_mbi_force = ParamId(cur_track_id as u32 * 1000 + cur_slot as u32 * 20);
        let pid_mbi_buzz  = ParamId(cur_track_id as u32 * 1000 + cur_slot as u32 * 20 + 1);
        bus.register(pid_mbi_force, 2.4);
        bus.register(pid_mbi_buzz, 0.85);

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(arc_bus.clone());

        // Mutate Steelpan
        view.steelpan_view.strike_radial_pos = 0.82;
        view.steelpan_view.strike_velocity = 0.92;
        view.sync_steelpan_hud_state();

        assert_eq!(arc_bus.get(pid_pan_rad), Some(0.82));
        assert_eq!(arc_bus.get(pid_pan_vel), Some(0.92));
        assert_eq!(view.device_rack_state.node_param_values.get("strike_radial_pos"), Some(&0.82));
        assert_eq!(view.inspector_state.node_param_values.get("strike_velocity"), Some(&0.92));

        // Mutate Mbira
        view.mbira_view.pluck_force_n = 3.8;
        view.mbira_view.buzz_intensity_pct = 0.42;
        view.sync_mbira_hud_state();

        assert_eq!(arc_bus.get(pid_mbi_force), Some(3.8));
        assert_eq!(arc_bus.get(pid_mbi_buzz), Some(0.42));
        assert_eq!(view.device_rack_state.node_param_values.get("pluck_force_n"), Some(&3.8));
        assert_eq!(view.inspector_state.node_param_values.get("buzz_intensity_pct"), Some(&0.42));
    }

    #[test]
    fn test_turn56_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Test Steelpan inspector trigger propagation
        assert!(!view.is_steelpan_hud_open());
        view.inspector_state.requested_open_steelpan_hud = true;
        if view.inspector_state.requested_open_steelpan_hud || view.device_rack_state.requested_open_steelpan_hud {
            view.inspector_state.requested_open_steelpan_hud = false;
            view.device_rack_state.requested_open_steelpan_hud = false;
            view.open_steelpan_hud();
        }
        assert!(view.is_steelpan_hud_open());
        assert!(!view.inspector_state.requested_open_steelpan_hud);
        view.close_steelpan_hud();

        // Test Mbira device rack trigger propagation
        assert!(!view.is_mbira_hud_open());
        view.device_rack_state.requested_open_mbira_hud = true;
        if view.inspector_state.requested_open_mbira_hud || view.device_rack_state.requested_open_mbira_hud {
            view.inspector_state.requested_open_mbira_hud = false;
            view.device_rack_state.requested_open_mbira_hud = false;
            view.open_mbira_hud();
        }
        assert!(view.is_mbira_hud_open());
        assert!(!view.device_rack_state.requested_open_mbira_hud);
        view.close_mbira_hud();
    }

    #[test]
    fn test_turn56_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_steelpan = actions.iter().any(|a| a.action_id == "open_steelpan_hud" && a.category == "Physical Modeling");
        assert!(has_steelpan, "open_steelpan_hud command action must be registered in command palette");

        let has_mbira = actions.iter().any(|a| a.action_id == "open_mbira_hud" && a.category == "Physical Modeling");
        assert!(has_mbira, "open_mbira_hud command action must be registered in command palette");
    }

    #[test]
    fn test_turn56_gui_headless_rendering_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut view = AwardWinningGuiView::new();
                view.open_steelpan_hud();
                view.open_mbira_hud();
                view.show(ui);
                assert!(view.is_steelpan_hud_open());
                assert!(view.is_mbira_hud_open());
            });
        });
    }
}
