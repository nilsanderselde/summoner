#![allow(clippy::all)]

//! Test suite for Turn #55 (Turn #20) — Physical Modeling Sitar & Turkish Ney Flute HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::sitar_view::{
        SitarView, SitarHudPreset, SITAR_PUCK_HIT_RADIUS, MIN_SITAR_STRIKE_VELOCITY, MAX_SITAR_STRIKE_VELOCITY,
        MIN_MEEND_SEMITONES, MAX_MEEND_SEMITONES,
    };
    use crate::views::turkish_ney_view::{
        TurkishNeyView, TurkishNeyType, BashpareHornType, NEY_PUCK_HIT_RADIUS,
        MIN_JET_VELOCITY_MPS, MAX_JET_VELOCITY_MPS, MIN_EMBOUCHURE_ANGLE_DEG, MAX_EMBOUCHURE_ANGLE_DEG,
    };

    #[test]
    fn test_turn55_pure_sitar_presets_and_acoustics() {
        let mut view = SitarView::new();
        assert_eq!(view.preset, SitarHudPreset::RagaYamanAlap);
        assert!((view.strike_velocity - 0.75).abs() < 1e-4);
        assert!((view.meend_pull_semitones - 1.5).abs() < 1e-4);
        assert!((view.jawari_gap_mm - 0.18).abs() < 1e-4);
        assert!((view.jiva_thread_pos - 0.45).abs() < 1e-4);
        assert!((view.tarab_bleed - 0.40).abs() < 1e-4);
        assert_eq!(view.active_raga_name, "Raga Yaman (Evening)");

        // Test preset switching
        view.set_preset(SitarHudPreset::VilayatKhanGayaki);
        assert_eq!(view.preset, SitarHudPreset::VilayatKhanGayaki);
        assert_eq!(view.active_raga_name, "Raga Bhairav (Dawn)");
        assert!((view.jawari_gap_mm - 0.08).abs() < 1e-4);

        view.set_preset(SitarHudPreset::SurbaharDeepBass);
        assert_eq!(view.preset, SitarHudPreset::SurbaharDeepBass);
        assert_eq!(view.active_raga_name, "Raga Darbari (Midnight)");
        assert!((view.jawari_gap_mm - 0.35).abs() < 1e-4);

        view.set_preset(SitarHudPreset::ElectricSitarJhajhar);
        assert_eq!(view.preset, SitarHudPreset::ElectricSitarJhajhar);
        assert_eq!(view.active_raga_name, "Raga Bilawal (Morning)");
        assert!((view.jawari_gap_mm - 0.05).abs() < 1e-4);

        // Test puck physics updates
        view.update_physics_from_puck(0.5, 0.5);
        let expected_meend = MIN_MEEND_SEMITONES + 0.5 * (MAX_MEEND_SEMITONES - MIN_MEEND_SEMITONES);
        let expected_vel = MIN_SITAR_STRIKE_VELOCITY + 0.5 * (MAX_SITAR_STRIKE_VELOCITY - MIN_SITAR_STRIKE_VELOCITY);
        assert!((view.meend_pull_semitones - expected_meend).abs() < 1e-3);
        assert!((view.strike_velocity - expected_vel).abs() < 1e-3);

        // Test puck hit test
        let canvas = Rect::new(50.0, 50.0, 300.0, 200.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_sitar_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_sitar_puck((puck_x + SITAR_PUCK_HIT_RADIUS * 0.9, puck_y), canvas));
        assert!(!view.hit_test_sitar_puck((puck_x + SITAR_PUCK_HIT_RADIUS * 1.5, puck_y), canvas));
    }

    #[test]
    fn test_turn55_pure_turkish_ney_instruments_and_simulation() {
        let mut view = TurkishNeyView::new();
        assert_eq!(view.ney_type, TurkishNeyType::MansurNey);
        assert_eq!(view.bashpare_type, BashpareHornType::WaterBuffaloHorn);
        assert!((view.jet_velocity_mps - 18.5).abs() < 1e-3);
        assert!((view.embouchure_angle_deg - 42.0).abs() < 1e-3);
        assert!((view.bore_length_cm - 78.0).abs() < 1e-3);

        // Instrument switching
        view.set_ney_type(TurkishNeyType::KizNey);
        assert_eq!(view.ney_type, TurkishNeyType::KizNey);
        assert!((view.bore_length_cm - 68.0).abs() < 1e-3);

        view.set_ney_type(TurkishNeyType::SahNey);
        assert_eq!(view.ney_type, TurkishNeyType::SahNey);
        assert!((view.bore_length_cm - 86.0).abs() < 1e-3);

        // Normalization roundtrips
        let norm_v = TurkishNeyView::velocity_to_normalized(25.0);
        let recon_v = TurkishNeyView::normalized_to_velocity(norm_v);
        assert!((recon_v - 25.0).abs() < 1e-3);

        let norm_a = TurkishNeyView::angle_to_normalized(35.0);
        let recon_a = TurkishNeyView::normalized_to_angle(norm_a);
        assert!((recon_a - 35.0).abs() < 1e-3);

        // Puck hit test
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_ney_puck((px, py), canvas));
        assert!(view.hit_test_ney_puck((px + NEY_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_ney_puck((px + NEY_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Check modal amplitudes
        for amp in view.modal_amplitudes {
            assert!(amp >= 0.0);
            assert!(amp <= 2.0);
        }

        // Test constant limits
        assert_eq!(MIN_JET_VELOCITY_MPS, 5.0);
        assert_eq!(MAX_JET_VELOCITY_MPS, 45.0);
        assert_eq!(MIN_EMBOUCHURE_ANGLE_DEG, 15.0);
        assert_eq!(MAX_EMBOUCHURE_ANGLE_DEG, 65.0);
    }

    #[test]
    fn test_turn55_pure_parambus_slot_arithmetic_and_channel_isolation() {
        let mut bus = ParamBus::new();

        // Track 3, Slot 0: Sitar
        let pid_sit_vel = ParamId(3 * 1000 + 0 * 20 + 0);
        let pid_sit_mee = ParamId(3 * 1000 + 0 * 20 + 1);
        let pid_sit_gap = ParamId(3 * 1000 + 0 * 20 + 2);
        let pid_sit_jiv = ParamId(3 * 1000 + 0 * 20 + 3);
        let pid_sit_tar = ParamId(3 * 1000 + 0 * 20 + 4);
        let pid_sit_chk = ParamId(3 * 1000 + 0 * 20 + 5);

        // Track 3, Slot 1: Turkish Ney
        let pid_ney_jet = ParamId(3 * 1000 + 1 * 20 + 0);
        let pid_ney_ang = ParamId(3 * 1000 + 1 * 20 + 1);
        let pid_ney_len = ParamId(3 * 1000 + 1 * 20 + 2);
        let pid_ney_tur = ParamId(3 * 1000 + 1 * 20 + 3);
        let pid_ney_los = ParamId(3 * 1000 + 1 * 20 + 4);
        let pid_ney_q   = ParamId(3 * 1000 + 1 * 20 + 5);

        bus.register(pid_sit_vel, 0.75);
        bus.register(pid_sit_mee, 1.5);
        bus.register(pid_sit_gap, 0.18);
        bus.register(pid_sit_jiv, 0.45);
        bus.register(pid_sit_tar, 0.40);
        bus.register(pid_sit_chk, 0.0);

        bus.register(pid_ney_jet, 18.5);
        bus.register(pid_ney_ang, 42.0);
        bus.register(pid_ney_len, 78.0);
        bus.register(pid_ney_tur, 0.30);
        bus.register(pid_ney_los, 0.05);
        bus.register(pid_ney_q, 48.0);

        // Mutate Sitar parameters
        bus.set(pid_sit_mee, 3.25);
        bus.set(pid_sit_vel, 0.95);

        // Assert Sitar changed
        assert_eq!(bus.get(pid_sit_mee), Some(3.25));
        assert_eq!(bus.get(pid_sit_vel), Some(0.95));

        // Assert Turkish Ney untouched
        assert_eq!(bus.get(pid_ney_jet), Some(18.5));
        assert_eq!(bus.get(pid_ney_ang), Some(42.0));

        // Mutate Ney parameters
        bus.set(pid_ney_jet, 32.0);
        bus.set(pid_ney_ang, 55.0);

        assert_eq!(bus.get(pid_ney_jet), Some(32.0));
        assert_eq!(bus.get(pid_ney_ang), Some(55.0));
        assert_eq!(bus.get(pid_sit_mee), Some(3.25));
    }

    #[test]
    fn test_turn55_pure_ascii_snapshots() {
        let sitar = SitarView::new();
        let sitar_ascii = sitar.render_ascii(70, 16);
        assert!(!sitar_ascii.is_empty());
        assert!(sitar_ascii.iter().any(|line| line.contains("SITAR PERFORMANCE HUD")));

        let ney = TurkishNeyView::new();
        let ney_ascii = ney.render_ascii(70, 16);
        assert!(!ney_ascii.is_empty());
        assert!(ney_ascii.iter().any(|line| line.contains('N')));
        assert!(ney_ascii.iter().any(|line| line.contains('=')));
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
    fn test_turn55_gui_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_sitar_hud_open());
        view.open_sitar_hud();
        assert!(view.is_sitar_hud_open());
        view.close_sitar_hud();
        assert!(!view.is_sitar_hud_open());

        assert!(!view.is_turkish_ney_hud_open());
        view.open_turkish_ney_hud();
        assert!(view.is_turkish_ney_hud_open());
        view.close_turkish_ney_hud();
        assert!(!view.is_turkish_ney_hud_open());
    }

    #[test]
    fn test_turn55_gui_inspector_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.inspector_state.requested_open_sitar_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_sitar_hud_open());
        assert!(!view.inspector_state.requested_open_sitar_hud);
        view.close_sitar_hud();

        view.inspector_state.requested_open_turkish_ney_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_turkish_ney_hud_open());
        assert!(!view.inspector_state.requested_open_turkish_ney_hud);
    }

    #[test]
    fn test_turn55_gui_device_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_sitar_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_sitar_hud_open());
        assert!(!view.device_rack_state.requested_open_sitar_hud);
        view.close_sitar_hud();

        view.device_rack_state.requested_open_turkish_ney_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_turkish_ney_hud_open());
        assert!(!view.device_rack_state.requested_open_turkish_ney_hud);
    }

    #[test]
    fn test_turn55_gui_live_parambus_sync() {
        let mut view = AwardWinningGuiView::new();
        view.selected_track_idx = 0; // Track 1
        let mut bus = ParamBus::new();

        let track_id = 1;
        let slot_idx = 0;
        let pid_vel = ParamId(track_id * 1000 + slot_idx * 20 + 0);
        let pid_mee = ParamId(track_id * 1000 + slot_idx * 20 + 1);

        bus.register(pid_vel, 0.1);
        bus.register(pid_mee, 0.1);

        let bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&bus));

        view.sitar_view.strike_velocity = 0.88;
        view.sitar_view.meend_pull_semitones = 3.5;
        view.sync_sitar_hud_state();

        assert_eq!(bus.get(pid_vel), Some(0.88));
        assert_eq!(bus.get(pid_mee), Some(3.5));

        let pid_jet = ParamId(track_id * 1000 + slot_idx * 20 + 0);
        let pid_ang = ParamId(track_id * 1000 + slot_idx * 20 + 1);
        view.turkish_ney_view.jet_velocity_mps = 35.0;
        view.turkish_ney_view.embouchure_angle_deg = 52.0;
        view.sync_turkish_ney_hud_state();

        assert_eq!(bus.get(pid_jet), Some(35.0));
        assert_eq!(bus.get(pid_ang), Some(52.0));
    }

    #[test]
    fn test_turn55_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let has_sitar = palette.actions.iter().any(|a| a.action_id == "open_sitar_hud");
        let has_ney = palette.actions.iter().any(|a| a.action_id == "open_turkish_ney_hud");

        assert!(has_sitar, "open_sitar_hud should be registered in CommandPalette");
        assert!(has_ney, "open_turkish_ney_hud should be registered in CommandPalette");
    }

    #[test]
    fn test_turn55_gui_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_sitar_hud();
        view.open_turkish_ney_hud();

        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(!output.shapes.is_empty(), "Headless render should produce visual output shapes");
    }
}
