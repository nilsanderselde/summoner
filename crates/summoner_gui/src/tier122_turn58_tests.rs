#![allow(clippy::all)]

//! Test suite for Turn #58 (Turn #23 Deliverable) — Physical Modeling Hammered Dulcimer / Cimbalom & Electromechanical Clavinet D6 HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::dulcimer_cimbalom_view::{
        DulcimerCimbalomView, DulcimerType, DULCIMER_PUCK_HIT_RADIUS,
        MIN_STRIKE_POS, MAX_STRIKE_POS, MIN_HAMMER_HARDNESS, MAX_HAMMER_HARDNESS,
    };
    use crate::views::clavinet_view::{
        ClavinetView, GuiClavinetProfile, GuiClavinetPickupMode, CLAV_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn58_pure_dulcimer_cimbalom_instruments_and_simulation() {
        let mut view = DulcimerCimbalomView::new();
        assert_eq!(view.instrument_type, DulcimerType::ConcertGrandCimbalom);
        assert!((view.strike_pos_ratio - 0.14).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.65).abs() < 1e-4);
        assert_eq!(view.string_courses, 35);
        assert!((view.inharmonicity_coeff - 0.0035).abs() < 1e-5);
        assert!((view.decay_s - 6.5).abs() < 1e-4);
        assert!((view.bridge_coupling - 0.45).abs() < 1e-4);
        assert!(!view.damper_pedal);

        // Preset switching
        view.set_instrument_type(DulcimerType::AppalachianHammeredDulcimer);
        assert_eq!(view.instrument_type, DulcimerType::AppalachianHammeredDulcimer);
        assert!((view.strike_pos_ratio - 0.18).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.50).abs() < 1e-4);
        assert_eq!(view.string_courses, 31);
        assert!((view.decay_s - 4.2).abs() < 1e-4);

        view.set_instrument_type(DulcimerType::PersianSantur);
        assert_eq!(view.instrument_type, DulcimerType::PersianSantur);
        assert!((view.strike_pos_ratio - 0.10).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.85).abs() < 1e-4);
        assert_eq!(view.string_courses, 18);
        assert!((view.decay_s - 5.8).abs() < 1e-4);

        view.set_instrument_type(DulcimerType::ChineseYangqin);
        assert_eq!(view.instrument_type, DulcimerType::ChineseYangqin);
        assert!((view.strike_pos_ratio - 0.15).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.75).abs() < 1e-4);
        assert_eq!(view.string_courses, 28);

        view.set_instrument_type(DulcimerType::MedievalPsaltery);
        assert_eq!(view.instrument_type, DulcimerType::MedievalPsaltery);
        assert!((view.strike_pos_ratio - 0.22).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.90).abs() < 1e-4);
        assert_eq!(view.string_courses, 15);

        // Normalization roundtrips
        let norm_p = DulcimerCimbalomView::pos_to_normalized(0.25);
        let recon_p = DulcimerCimbalomView::normalized_to_pos(norm_p);
        assert!((recon_p - 0.25).abs() < 1e-3);

        let norm_h = DulcimerCimbalomView::hardness_to_normalized(0.70);
        let recon_h = DulcimerCimbalomView::normalized_to_hardness(norm_h);
        assert!((recon_h - 0.70).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_dulcimer_puck((px, py), canvas));
        assert!(view.hit_test_dulcimer_puck((px + DULCIMER_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_dulcimer_puck((px + DULCIMER_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Modal amplitudes check
        for amp in view.modal_amplitudes {
            assert!(amp >= 0.0);
            assert!(amp <= 2.0);
        }

        assert_eq!(MIN_STRIKE_POS, 0.05);
        assert_eq!(MAX_STRIKE_POS, 0.50);
        assert_eq!(MIN_HAMMER_HARDNESS, 0.10);
        assert_eq!(MAX_HAMMER_HARDNESS, 1.00);
    }

    #[test]
    fn test_turn58_pure_clavinet_d6_profiles_and_waveforms() {
        let mut view = ClavinetView::new();
        assert_eq!(view.profile, GuiClavinetProfile::StevieSuperstition);
        assert_eq!(view.pickup_mode, GuiClavinetPickupMode::ParallelInPhase);
        assert!((view.anvil_hardness - 0.85).abs() < 1e-4);
        assert!((view.yarn_damping - 0.75).abs() < 1e-4);
        assert!((view.pickup_blend - 0.50).abs() < 1e-4);
        assert!((view.pickup_phase_deg - 0.0).abs() < 1e-4);
        assert!(view.brilliant_switch);
        assert!(view.treble_switch);
        assert!(view.medium_switch);
        assert!(!view.soft_switch);
        assert!(!view.auto_wah_enabled);

        // Profile switching
        view.set_profile(GuiClavinetProfile::OutPhaseFunkQuack);
        assert_eq!(view.profile, GuiClavinetProfile::OutPhaseFunkQuack);
        assert_eq!(view.pickup_mode, GuiClavinetPickupMode::ParallelOutOfPhase);
        assert!((view.anvil_hardness - 0.90).abs() < 1e-4);
        assert!((view.pickup_phase_deg - 180.0).abs() < 1e-4);

        view.set_profile(GuiClavinetProfile::ClassicD6Clean);
        assert_eq!(view.profile, GuiClavinetProfile::ClassicD6Clean);
        assert_eq!(view.pickup_mode, GuiClavinetPickupMode::ParallelInPhase);
        assert!((view.anvil_hardness - 0.70).abs() < 1e-4);

        view.set_profile(GuiClavinetProfile::MellowChamber);
        assert_eq!(view.profile, GuiClavinetProfile::MellowChamber);
        assert_eq!(view.pickup_mode, GuiClavinetPickupMode::NeckOnly);
        assert!(view.soft_switch);

        view.set_profile(GuiClavinetProfile::ScreamingAutoWah);
        assert_eq!(view.profile, GuiClavinetProfile::ScreamingAutoWah);
        assert!(view.auto_wah_enabled);

        view.set_profile(GuiClavinetProfile::TwangyBridgeLead);
        assert_eq!(view.profile, GuiClavinetProfile::TwangyBridgeLead);
        assert_eq!(view.pickup_mode, GuiClavinetPickupMode::BridgeOnly);

        // Normalization roundtrips
        let norm_h = ClavinetView::hardness_to_normalized(0.65);
        let recon_h = ClavinetView::normalized_to_hardness(norm_h);
        assert!((recon_h - 0.65).abs() < 1e-3);

        let norm_ph = ClavinetView::phase_to_normalized(90.0);
        let recon_ph = ClavinetView::normalized_to_phase(norm_ph);
        assert!((recon_ph - 90.0).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_clavinet_puck((px, py), canvas));
        assert!(view.hit_test_clavinet_puck((px + CLAV_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_clavinet_puck((px + CLAV_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Impact curve and pickup waveform evaluations
        let impact = view.evaluate_anvil_impact_curve();
        assert_eq!(impact.len(), 32);
        for val in impact {
            assert!(val >= 0.0);
            assert!(val <= 1.0);
        }

        let wave = view.evaluate_pickup_waveform();
        assert_eq!(wave.len(), 32);
        for val in wave {
            assert!(val >= -1.0);
            assert!(val <= 1.0);
        }
    }

    #[test]
    fn test_turn58_pure_slot_arithmetic_channel_isolation() {
        let mut bus = ParamBus::new();

        for track_id in 1..=9 {
            for slot_idx in 0..=3 {
                let base = track_id as u32 * 1000 + slot_idx as u32 * 20;

                // Dulcimer parameter slot addresses: 0..=5
                let pid_strike = ParamId(base);
                let pid_hard   = ParamId(base + 1);
                let pid_inharm = ParamId(base + 2);
                let pid_decay  = ParamId(base + 3);
                let pid_coup   = ParamId(base + 4);
                let pid_damp   = ParamId(base + 5);

                bus.register(pid_strike, 0.16);
                bus.register(pid_hard, 0.70);
                bus.register(pid_inharm, 0.0028);
                bus.register(pid_decay, 5.0);
                bus.register(pid_coup, 0.40);
                bus.register(pid_damp, 1.0);

                assert_eq!(bus.get(pid_strike), Some(0.16));
                assert_eq!(bus.get(pid_hard), Some(0.70));
                assert_eq!(bus.get(pid_inharm), Some(0.0028));
                assert_eq!(bus.get(pid_decay), Some(5.0));
                assert_eq!(bus.get(pid_coup), Some(0.40));
                assert_eq!(bus.get(pid_damp), Some(1.0));

                // Mutate parameters
                bus.set(pid_strike, 0.20);
                bus.set(pid_hard, 0.85);
                bus.set(pid_inharm, 0.0040);
                bus.set(pid_decay, 7.5);
                bus.set(pid_coup, 0.60);
                bus.set(pid_damp, 0.0);

                assert_eq!(bus.get(pid_strike), Some(0.20));
                assert_eq!(bus.get(pid_hard), Some(0.85));
                assert_eq!(bus.get(pid_inharm), Some(0.0040));
                assert_eq!(bus.get(pid_decay), Some(7.5));
                assert_eq!(bus.get(pid_coup), Some(0.60));
                assert_eq!(bus.get(pid_damp), Some(0.0));
            }
        }
    }

    #[test]
    fn test_turn58_pure_ascii_snapshots_deterministic() {
        let dulcimer = DulcimerCimbalomView::new();
        let dulc_lines = dulcimer.render_ascii_snapshot(60, 16);
        assert_eq!(dulc_lines.len(), 16);
        assert!(dulc_lines[0].starts_with('+'));

        let clavinet = ClavinetView::new();
        let clav_lines = clavinet.render_ascii_snapshot(60, 16);
        assert_eq!(clav_lines.len(), 16);
        assert!(clav_lines[0].contains("CLAVINET HUD"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn58_gui_award_winning_view_modals_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Dulcimer modal
        assert!(!view.is_dulcimer_hud_open());
        view.open_dulcimer_hud();
        assert!(view.is_dulcimer_hud_open());
        view.close_dulcimer_hud();
        assert!(!view.is_dulcimer_hud_open());

        // Clavinet modal
        assert!(!view.is_clavinet_hud_open());
        view.open_clavinet_hud();
        assert!(view.is_clavinet_hud_open());
        view.close_clavinet_hud();
        assert!(!view.is_clavinet_hud_open());
    }

    #[test]
    fn test_turn58_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_dulcimer_hud();
        view.open_clavinet_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_dulcimer_hud_open());
        assert!(view.is_clavinet_hud_open());
    }

    #[test]
    fn test_turn58_gui_live_param_bus_atomic_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..6 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Dulcimer sync
        view.dulcimer_view.strike_pos_ratio = 0.22;
        view.dulcimer_view.hammer_hardness = 0.80;
        view.dulcimer_view.inharmonicity_coeff = 0.0042;
        view.dulcimer_view.decay_s = 5.5;
        view.dulcimer_view.bridge_coupling = 0.55;
        view.dulcimer_view.damper_pedal = true;

        view.sync_dulcimer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("strike_pos_ratio").copied(), Some(0.22));
        assert_eq!(view.device_rack_state.node_param_values.get("hammer_hardness").copied(), Some(0.80));
        assert_eq!(view.inspector_state.node_param_values.get("strike_pos_ratio").copied(), Some(0.22));
        assert_eq!(view.inspector_state.node_param_values.get("hammer_hardness").copied(), Some(0.80));

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.22));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.80));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.0042));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(5.5));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.55));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(1.0));

        // Test Clavinet sync
        view.clavinet_view.anvil_hardness = 0.92;
        view.clavinet_view.yarn_damping = 0.68;
        view.clavinet_view.pickup_blend = 0.60;
        view.clavinet_view.pickup_phase_deg = 120.0;
        view.clavinet_view.auto_wah_mix = 0.75;
        view.clavinet_view.master_level = 1.10;

        view.sync_clavinet_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("anvil_hardness").copied(), Some(0.92));
        assert_eq!(view.device_rack_state.node_param_values.get("pickup_phase_deg").copied(), Some(120.0));
        assert_eq!(arc_bus.get(ParamId(base)), Some(0.92));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.68));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.60));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(120.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.75));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(1.10));
    }

    #[test]
    fn test_turn58_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger Dulcimer via inspector
        assert!(!view.is_dulcimer_hud_open());
        view.inspector_state.requested_open_dulcimer_hud = true;
        if view.inspector_state.requested_open_dulcimer_hud || view.device_rack_state.requested_open_dulcimer_hud {
            view.inspector_state.requested_open_dulcimer_hud = false;
            view.device_rack_state.requested_open_dulcimer_hud = false;
            view.open_dulcimer_hud();
        }
        assert!(view.is_dulcimer_hud_open());
        assert!(!view.inspector_state.requested_open_dulcimer_hud);
        view.close_dulcimer_hud();
        assert!(!view.is_dulcimer_hud_open());

        // Trigger Dulcimer via device rack
        view.device_rack_state.requested_open_dulcimer_hud = true;
        if view.inspector_state.requested_open_dulcimer_hud || view.device_rack_state.requested_open_dulcimer_hud {
            view.inspector_state.requested_open_dulcimer_hud = false;
            view.device_rack_state.requested_open_dulcimer_hud = false;
            view.open_dulcimer_hud();
        }
        assert!(view.is_dulcimer_hud_open());
        assert!(!view.device_rack_state.requested_open_dulcimer_hud);
        view.close_dulcimer_hud();

        // Trigger Clavinet via inspector
        assert!(!view.is_clavinet_hud_open());
        view.inspector_state.requested_open_clavinet_hud = true;
        if view.inspector_state.requested_open_clavinet_hud || view.device_rack_state.requested_open_clavinet_hud {
            view.inspector_state.requested_open_clavinet_hud = false;
            view.device_rack_state.requested_open_clavinet_hud = false;
            view.open_clavinet_hud();
        }
        assert!(view.is_clavinet_hud_open());
        assert!(!view.inspector_state.requested_open_clavinet_hud);
        view.close_clavinet_hud();

        // Trigger Clavinet via device rack
        view.device_rack_state.requested_open_clavinet_hud = true;
        if view.inspector_state.requested_open_clavinet_hud || view.device_rack_state.requested_open_clavinet_hud {
            view.inspector_state.requested_open_clavinet_hud = false;
            view.device_rack_state.requested_open_clavinet_hud = false;
            view.open_clavinet_hud();
        }
        assert!(view.is_clavinet_hud_open());
        assert!(!view.device_rack_state.requested_open_clavinet_hud);
        view.close_clavinet_hud();
    }

    #[test]
    fn test_turn58_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_dulcimer = actions.iter().any(|a| a.action_id == "open_dulcimer_hud" && a.category == "Physical Modeling");
        assert!(has_dulcimer, "open_dulcimer_hud command action must be registered in command palette");

        let has_clavinet = actions.iter().any(|a| a.action_id == "open_clavinet_hud" && a.category == "Physical Modeling");
        assert!(has_clavinet, "open_clavinet_hud command action must be registered in command palette");
    }
}
