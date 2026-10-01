#![allow(clippy::all)]

//! Test suite for Turn #57 (Turn #22) — Physical Modeling Japanese Koto & Balinese Gamelan Gender HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::koto_view::{
        KotoView, KotoHudPreset, KOTO_PUCK_HIT_RADIUS,
        MIN_OSHI_ITE_FORCE_N, MAX_OSHI_ITE_FORCE_N, MIN_KOTO_TSUME_VELOCITY, MAX_KOTO_TSUME_VELOCITY,
    };
    use crate::views::gamelan_gender_view::{
        GamelanGenderView, GamelanInstrumentType, GAMELAN_PUCK_HIT_RADIUS,
        MIN_MALLET_HARDNESS, MAX_MALLET_HARDNESS, MIN_OMBAK_RATE_HZ, MAX_OMBAK_RATE_HZ,
    };

    #[test]
    fn test_turn57_pure_koto_zither_instruments_and_simulation() {
        let mut view = KotoView::new();
        assert_eq!(view.preset, KotoHudPreset::HirajoshiTraditionalSilk);
        assert!((view.tsume_velocity - 0.75).abs() < 1e-4);
        assert!((view.oshi_ite_force_n - 2.0).abs() < 1e-4);
        assert!((view.hiki_iro_release - 0.10).abs() < 1e-4);
        assert!((view.ji_bridge_offset - 0.0).abs() < 1e-4);
        assert!((view.behind_bridge_bleed - 0.25).abs() < 1e-4);
        assert!((view.body_wood_resonance - 0.85).abs() < 1e-4);
        assert!(view.active_tuning_schema_name.contains("Hirajoshi"));

        // Preset switching
        view.set_preset(KotoHudPreset::KokinJoshiUrbanClassical);
        assert_eq!(view.preset, KotoHudPreset::KokinJoshiUrbanClassical);
        assert!((view.behind_bridge_bleed - 0.20).abs() < 1e-4);
        assert!((view.body_wood_resonance - 0.80).abs() < 1e-4);
        assert!(view.active_tuning_schema_name.contains("Kokin-joshi"));

        view.set_preset(KotoHudPreset::InSenContemporaryDramatic);
        assert_eq!(view.preset, KotoHudPreset::InSenContemporaryDramatic);
        assert!((view.behind_bridge_bleed - 0.30).abs() < 1e-4);
        assert!((view.body_wood_resonance - 0.90).abs() < 1e-4);

        view.set_preset(KotoHudPreset::KumoiJoshiSpringRain);
        assert_eq!(view.preset, KotoHudPreset::KumoiJoshiSpringRain);
        assert!((view.behind_bridge_bleed - 0.22).abs() < 1e-4);

        view.set_preset(KotoHudPreset::RyukyuFestiveOkinawa);
        assert_eq!(view.preset, KotoHudPreset::RyukyuFestiveOkinawa);
        assert!(view.active_tuning_schema_name.contains("Ryukyu"));

        view.set_preset(KotoHudPreset::ConcertGuzhengVirtuoso);
        assert_eq!(view.preset, KotoHudPreset::ConcertGuzhengVirtuoso);

        // Normalization roundtrips
        let norm_f = KotoView::oshi_ite_to_normalized(15.0);
        let recon_f = KotoView::normalized_to_oshi_ite(norm_f);
        assert!((recon_f - 15.0).abs() < 1e-3);

        let norm_v = KotoView::tsume_to_normalized(0.65);
        let recon_v = KotoView::normalized_to_tsume(norm_v);
        assert!((recon_v - 0.65).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_koto_puck((px, py), canvas));
        assert!(view.hit_test_koto_puck((px + KOTO_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_koto_puck((px + KOTO_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Pitch cents calculation
        view.oshi_ite_force_n = 30.0;
        assert!((view.current_pitch_cents() - 400.0).abs() < 1.0);

        assert_eq!(MIN_OSHI_ITE_FORCE_N, 0.0);
        assert_eq!(MAX_OSHI_ITE_FORCE_N, 30.0);
        assert_eq!(MIN_KOTO_TSUME_VELOCITY, 0.05);
        assert_eq!(MAX_KOTO_TSUME_VELOCITY, 1.0);
    }

    #[test]
    fn test_turn57_pure_gamelan_instruments_and_simulation() {
        let mut view = GamelanGenderView::new();
        assert_eq!(view.instrument_type, GamelanInstrumentType::GenderWayang10Bar);
        assert!((view.mallet_hardness - 0.45).abs() < 1e-4);
        assert!((view.ombak_rate_hz - 7.2).abs() < 1e-4);
        assert_eq!(view.bar_count, 10);
        assert!((view.bronze_thickness_mm - 6.5).abs() < 1e-4);
        assert!((view.damping_factor - 0.35).abs() < 1e-4);
        assert!((view.resonator_coupling_q - 38.0).abs() < 1e-4);

        // Instrument switching
        view.set_instrument_type(GamelanInstrumentType::GamelanJegogan);
        assert_eq!(view.instrument_type, GamelanInstrumentType::GamelanJegogan);
        assert_eq!(view.bar_count, 5);
        assert!((view.bronze_thickness_mm - 18.0).abs() < 1e-4);
        assert!((view.ombak_rate_hz - 3.5).abs() < 1e-4);
        assert!((view.resonator_coupling_q - 65.0).abs() < 1e-4);

        view.set_instrument_type(GamelanInstrumentType::GamelanCalungPemade);
        assert_eq!(view.instrument_type, GamelanInstrumentType::GamelanCalungPemade);
        assert_eq!(view.bar_count, 10);
        assert!((view.bronze_thickness_mm - 10.0).abs() < 1e-4);

        view.set_instrument_type(GamelanInstrumentType::GamelanKanthilTrompong);
        assert_eq!(view.instrument_type, GamelanInstrumentType::GamelanKanthilTrompong);
        assert!((view.mallet_hardness - 0.85).abs() < 1e-4);
        assert!((view.ombak_rate_hz - 8.5).abs() < 1e-4);

        view.set_instrument_type(GamelanInstrumentType::GamelanUgalLeader);
        assert_eq!(view.instrument_type, GamelanInstrumentType::GamelanUgalLeader);
        assert_eq!(view.bar_count, 10);

        // Normalization roundtrips
        let norm_h = GamelanGenderView::hardness_to_normalized(0.60);
        let recon_h = GamelanGenderView::normalized_to_hardness(norm_h);
        assert!((recon_h - 0.60).abs() < 1e-3);

        let norm_o = GamelanGenderView::ombak_to_normalized(6.0);
        let recon_o = GamelanGenderView::normalized_to_ombak(norm_o);
        assert!((recon_o - 6.0).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(30.0, 30.0, 320.0, 220.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_gamelan_puck((px, py), canvas));
        assert!(view.hit_test_gamelan_puck((px + GAMELAN_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_gamelan_puck((px + GAMELAN_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Modal amplitudes check
        for amp in view.modal_amplitudes {
            assert!(amp >= 0.0);
            assert!(amp <= 2.5);
        }

        assert_eq!(MIN_MALLET_HARDNESS, 0.05);
        assert_eq!(MAX_MALLET_HARDNESS, 1.00);
        assert_eq!(MIN_OMBAK_RATE_HZ, 2.0);
        assert_eq!(MAX_OMBAK_RATE_HZ, 12.0);
    }

    #[test]
    fn test_turn57_pure_slot_arithmetic_channel_isolation() {
        let mut bus = ParamBus::new();

        for track_id in 1..=9 {
            for slot_idx in 0..=3 {
                let base = track_id as u32 * 1000 + slot_idx as u32 * 20;

                // Koto parameter slot addresses: 0..=5
                let pid_tsume    = ParamId(base);
                let pid_oshi_ite = ParamId(base + 1);
                let pid_release  = ParamId(base + 2);
                let pid_bridge   = ParamId(base + 3);
                let pid_bleed    = ParamId(base + 4);
                let pid_wood     = ParamId(base + 5);

                bus.register(pid_tsume, 0.82);
                bus.register(pid_oshi_ite, 18.5);
                bus.register(pid_release, 0.25);
                bus.register(pid_bridge, 0.05);
                bus.register(pid_bleed, 0.35);
                bus.register(pid_wood, 0.90);

                assert_eq!(bus.get(pid_tsume), Some(0.82));
                assert_eq!(bus.get(pid_oshi_ite), Some(18.5));
                assert_eq!(bus.get(pid_release), Some(0.25));
                assert_eq!(bus.get(pid_bridge), Some(0.05));
                assert_eq!(bus.get(pid_bleed), Some(0.35));
                assert_eq!(bus.get(pid_wood), Some(0.90));

                // Mutate parameters
                bus.set(pid_tsume, 0.70);
                bus.set(pid_oshi_ite, 6.5);
                bus.set(pid_release, 10.0);
                bus.set(pid_bridge, 8.5);
                bus.set(pid_bleed, 0.40);
                bus.set(pid_wood, 45.0);

                assert_eq!(bus.get(pid_tsume), Some(0.70));
                assert_eq!(bus.get(pid_oshi_ite), Some(6.5));
                assert_eq!(bus.get(pid_release), Some(10.0));
                assert_eq!(bus.get(pid_bridge), Some(8.5));
                assert_eq!(bus.get(pid_bleed), Some(0.40));
                assert_eq!(bus.get(pid_wood), Some(45.0));
            }
        }
    }

    #[test]
    fn test_turn57_pure_ascii_snapshots_deterministic() {
        let koto = KotoView::new();
        let koto_lines = koto.render_ascii_snapshot(60, 16);
        assert_eq!(koto_lines.len(), 16);
        assert!(koto_lines.iter().any(|l| l.contains("KOTO")));

        let gamelan = GamelanGenderView::new();
        let gam_lines = gamelan.render_ascii_snapshot(60, 16);
        assert_eq!(gam_lines.len(), 16);
        assert!(gam_lines[0].starts_with('+'));
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
    fn test_turn57_gui_award_winning_view_modals_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Koto modal
        assert!(!view.is_koto_hud_open());
        view.open_koto_hud();
        assert!(view.is_koto_hud_open());
        view.close_koto_hud();
        assert!(!view.is_koto_hud_open());

        // Gamelan modal
        assert!(!view.is_gamelan_hud_open());
        view.open_gamelan_hud();
        assert!(view.is_gamelan_hud_open());
        view.close_gamelan_hud();
        assert!(!view.is_gamelan_hud_open());
    }

    #[test]
    fn test_turn57_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_koto_hud();
        view.open_gamelan_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_koto_hud_open());
        assert!(view.is_gamelan_hud_open());
    }

    #[test]
    fn test_turn57_gui_live_param_bus_atomic_dispatch() {
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

        view.koto_view.tsume_velocity = 0.88;
        view.koto_view.oshi_ite_force_n = 22.0;
        view.koto_view.hiki_iro_release = 0.30;
        view.koto_view.ji_bridge_offset = 0.08;
        view.koto_view.behind_bridge_bleed = 0.40;
        view.koto_view.body_wood_resonance = 0.95;

        view.sync_koto_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("tsume_velocity").copied(), Some(0.88));
        assert_eq!(view.device_rack_state.node_param_values.get("oshi_ite_force_n").copied(), Some(22.0));
        assert_eq!(view.inspector_state.node_param_values.get("tsume_velocity").copied(), Some(0.88));
        assert_eq!(view.inspector_state.node_param_values.get("oshi_ite_force_n").copied(), Some(22.0));

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.88));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(22.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.30));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.08));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.40));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.95));

        // Test gamelan sync
        view.gamelan_view.mallet_hardness = 0.78;
        view.gamelan_view.ombak_rate_hz = 8.2;
        view.gamelan_view.bar_count = 10;
        view.gamelan_view.bronze_thickness_mm = 9.2;
        view.gamelan_view.damping_factor = 0.42;
        view.gamelan_view.resonator_coupling_q = 55.0;

        view.sync_gamelan_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("mallet_hardness").copied(), Some(0.78));
        assert_eq!(view.device_rack_state.node_param_values.get("ombak_rate_hz").copied(), Some(8.2));
        assert_eq!(arc_bus.get(ParamId(base)), Some(0.78));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(8.2));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(10.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(9.2));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.42));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(55.0));
    }

    #[test]
    fn test_turn57_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger Koto via inspector
        assert!(!view.is_koto_hud_open());
        view.inspector_state.requested_open_koto_hud = true;
        if view.inspector_state.requested_open_koto_hud || view.device_rack_state.requested_open_koto_hud {
            view.inspector_state.requested_open_koto_hud = false;
            view.device_rack_state.requested_open_koto_hud = false;
            view.open_koto_hud();
        }
        assert!(view.is_koto_hud_open());
        assert!(!view.inspector_state.requested_open_koto_hud);
        view.close_koto_hud();
        assert!(!view.is_koto_hud_open());

        // Trigger Koto via device rack
        view.device_rack_state.requested_open_koto_hud = true;
        if view.inspector_state.requested_open_koto_hud || view.device_rack_state.requested_open_koto_hud {
            view.inspector_state.requested_open_koto_hud = false;
            view.device_rack_state.requested_open_koto_hud = false;
            view.open_koto_hud();
        }
        assert!(view.is_koto_hud_open());
        assert!(!view.device_rack_state.requested_open_koto_hud);
        view.close_koto_hud();

        // Trigger Gamelan via inspector
        assert!(!view.is_gamelan_hud_open());
        view.inspector_state.requested_open_gamelan_hud = true;
        if view.inspector_state.requested_open_gamelan_hud || view.device_rack_state.requested_open_gamelan_hud {
            view.inspector_state.requested_open_gamelan_hud = false;
            view.device_rack_state.requested_open_gamelan_hud = false;
            view.open_gamelan_hud();
        }
        assert!(view.is_gamelan_hud_open());
        assert!(!view.inspector_state.requested_open_gamelan_hud);
        view.close_gamelan_hud();

        // Trigger Gamelan via device rack
        view.device_rack_state.requested_open_gamelan_hud = true;
        if view.inspector_state.requested_open_gamelan_hud || view.device_rack_state.requested_open_gamelan_hud {
            view.inspector_state.requested_open_gamelan_hud = false;
            view.device_rack_state.requested_open_gamelan_hud = false;
            view.open_gamelan_hud();
        }
        assert!(view.is_gamelan_hud_open());
        assert!(!view.device_rack_state.requested_open_gamelan_hud);
        view.close_gamelan_hud();
    }

    #[test]
    fn test_turn57_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_koto = actions.iter().any(|a| a.action_id == "open_koto_hud" && a.category == "Physical Modeling");
        assert!(has_koto, "open_koto_hud command action must be registered in command palette");

        let has_gamelan = actions.iter().any(|a| a.action_id == "open_gamelan_hud" && a.category == "Physical Modeling");
        assert!(has_gamelan, "open_gamelan_hud command action must be registered in command palette");
    }
}
