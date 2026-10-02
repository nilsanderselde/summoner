#![allow(clippy::all)]

//! Test suite for Turn #74 (Turn #37 deliverable) — Struck Idiophone Resonator Bank & Underwater Sonar Hydrophone Cavitation HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::idiophone_spectrum_view::{
        IdiophoneSpectrumView, IdiophoneViewProfile, IDIOPHONE_PUCK_HIT_RADIUS, NUM_VIEW_MODES,
    };
    use crate::views::sonar_hydrophone_view::{
        SonarHydrophoneView, SonarMode, SONAR_HYDROPHONE_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn74_pure_idiophone_spectrum_view_initialization_and_profiles() {
        let mut view = IdiophoneSpectrumView::new();
        assert_eq!(view.instrument, IdiophoneViewProfile::MarimbaWood);
        assert!((view.mallet_hardness - 0.35).abs() < 1e-3);
        assert!((view.strike_velocity - 0.80).abs() < 1e-3);
        assert!((view.strike_position - 0.20).abs() < 1e-3);
        assert_eq!(IDIOPHONE_PUCK_HIT_RADIUS, 22.0);

        // Profile tests
        view.set_profile(IdiophoneViewProfile::XylophoneRosewood);
        assert_eq!(view.profile(), IdiophoneViewProfile::XylophoneRosewood);
        assert_eq!(view.instrument.name(), "ORCHESTRAL XYLOPHONE");
        let xylo_ratios = view.instrument.modal_ratios();
        assert_eq!(xylo_ratios[0], 1.0);
        assert_eq!(xylo_ratios[1], 3.0);
        assert_eq!(xylo_ratios[2], 9.0);

        view.set_profile(IdiophoneViewProfile::VibraphoneAluminum);
        assert_eq!(view.profile(), IdiophoneViewProfile::VibraphoneAluminum);
        assert_eq!(view.instrument.name(), "CONCERT VIBRAPHONE");
        assert!(view.tremolo_rate_hz > 0.0);
        assert!(view.tremolo_depth > 0.0);

        view.set_profile(IdiophoneViewProfile::SteelpanTrinidad);
        assert_eq!(view.profile(), IdiophoneViewProfile::SteelpanTrinidad);

        view.set_profile(IdiophoneViewProfile::KalimbaMbira);
        assert_eq!(view.profile(), IdiophoneViewProfile::KalimbaMbira);

        view.set_profile(IdiophoneViewProfile::GlockenspielBell);
        assert_eq!(view.profile(), IdiophoneViewProfile::GlockenspielBell);

        // Coordinate normalization
        let norm_h = IdiophoneSpectrumView::hardness_to_normalized(0.65);
        assert!((norm_h - 0.65).abs() < 1e-3);
        let h_back = IdiophoneSpectrumView::normalized_to_hardness(norm_h);
        assert!((h_back - 0.65).abs() < 1e-3);

        let norm_v = IdiophoneSpectrumView::velocity_to_normalized(0.70);
        assert!(norm_v > 0.0 && norm_v < 1.0);
        let v_back = IdiophoneSpectrumView::normalized_to_velocity(norm_v);
        assert!((v_back - 0.70).abs() < 1e-3);

        // Spectrum evaluation
        let spectrum = view.evaluate_modal_spectrum();
        assert_eq!(spectrum.len(), NUM_VIEW_MODES);
        for &amp in &spectrum {
            assert!(amp >= 0.02 && amp <= 1.0);
        }

        // Hit testing
        let canvas = Rect::new(10.0, 10.0, 400.0, 300.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_idiophone_puck((px, py), canvas));
        assert!(!view.hit_test_idiophone_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn74_pure_sonar_hydrophone_view_initialization_and_modes() {
        let mut view = SonarHydrophoneView::new();
        assert_eq!(view.mode, SonarMode::ActiveSonarPing);
        assert!((view.depth_m - 250.0).abs() < 1e-3);
        assert!((view.water_temp_c - 12.5).abs() < 1e-3);
        assert!((view.salinity_ppt - 35.0).abs() < 1e-3);
        assert_eq!(SONAR_HYDROPHONE_PUCK_HIT_RADIUS, 22.0);

        // Modes
        view.set_mode(SonarMode::PassiveHydrophoneListening);
        assert_eq!(view.mode, SonarMode::PassiveHydrophoneListening);
        assert_eq!(view.mode.nominal_ping_freq_hz(), 500.0);
        assert!(!view.mode.is_active_emitter());

        view.set_mode(SonarMode::DeepOceanCavitation);
        assert_eq!(view.mode, SonarMode::DeepOceanCavitation);
        assert_eq!(view.mode.nominal_ping_freq_hz(), 12500.0);

        view.set_mode(SonarMode::ThermoclineWaveguide);
        assert_eq!(view.mode, SonarMode::ThermoclineWaveguide);
        assert_eq!(view.mode.nominal_ping_freq_hz(), 180.0);
        assert!(view.mode.is_active_emitter());

        view.set_mode(SonarMode::ArcticUnderIceRefraction);
        assert_eq!(view.mode, SonarMode::ArcticUnderIceRefraction);
        assert_eq!(view.mode.nominal_ping_freq_hz(), 1200.0);

        // Acoustic simulation
        assert!(view.sound_speed_mps > 1400.0 && view.sound_speed_mps < 1600.0);
        assert!(view.minnaert_resonance_hz > 500.0);

        // Transmission loss
        let tl_near = view.evaluate_transmission_loss_db(100.0);
        let tl_far = view.evaluate_transmission_loss_db(5000.0);
        assert!(tl_far > tl_near);

        // Coordinate normalization
        let norm_d = SonarHydrophoneView::depth_to_normalized(500.0);
        assert!(norm_d > 0.0 && norm_d < 1.0);
        let d_back = SonarHydrophoneView::normalized_to_depth(norm_d);
        assert!((d_back - 500.0).abs() < 1e-1);

        let norm_t = SonarHydrophoneView::temp_to_normalized(15.0);
        assert!((norm_t - 0.5).abs() < 1e-3);
        let t_back = SonarHydrophoneView::normalized_to_temp(norm_t);
        assert!((t_back - 15.0).abs() < 1e-3);

        let norm_c = SonarHydrophoneView::cavitation_to_normalized(1.5);
        assert!(norm_c > 0.0 && norm_c < 1.0);
        let c_back = SonarHydrophoneView::normalized_to_cavitation(norm_c);
        assert!((c_back - 1.5).abs() < 1e-3);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let puck_x = canvas.x + view.sonar_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.sonar_puck_pos.1) * canvas.height;
        assert!(view.hit_test_sonar_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_sonar_puck((puck_x + 90.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn74_pure_slot_arithmetic_isolation() {
        let track_id = 4u32;
        let slot_id = 3u32;
        let base_pid = ParamId(track_id * 1000 + slot_id * 20);

        let pid_hrd = ParamId(base_pid.0);
        let pid_vel = ParamId(base_pid.0 + 1);
        let pid_pos = ParamId(base_pid.0 + 2);
        let pid_tub = ParamId(base_pid.0 + 3);

        assert_eq!(pid_hrd.0, 4060);
        assert_eq!(pid_vel.0, 4061);
        assert_eq!(pid_pos.0, 4062);
        assert_eq!(pid_tub.0, 4063);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 4080);
        assert!(pid_tub.0 < next_slot_pid.0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn74_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Idiophone Spectrum modal
        view.open_idiophone_spectrum_hud();
        assert!(view.is_idiophone_spectrum_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_idiophone_spectrum_hud();
        assert!(!view.is_idiophone_spectrum_hud_open());

        // Open Sonar Hydrophone modal
        view.open_sonar_hydrophone_hud();
        assert!(view.is_sonar_hydrophone_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_sonar_hydrophone_hud();
        assert!(!view.is_sonar_hydrophone_hud_open());
    }

    #[test]
    fn test_turn74_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync Idiophone Spectrum HUD
        view.idiophone_spectrum_view.mallet_hardness = 0.65;
        view.idiophone_spectrum_view.strike_velocity = 0.85;
        view.idiophone_spectrum_view.strike_position = 0.30;
        view.idiophone_spectrum_view.resonator_tube_mix = 0.55;
        view.sync_idiophone_spectrum_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.30));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.55));

        assert_eq!(
            view.device_rack_state.node_param_values.get("mallet_hardness"),
            Some(&0.65)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("mallet_hardness"),
            Some(&0.65)
        );

        // 2. Sync Sonar Hydrophone HUD
        view.sonar_hydrophone_view.depth_m = 450.0;
        view.sonar_hydrophone_view.water_temp_c = 8.5;
        view.sonar_hydrophone_view.salinity_ppt = 32.0;
        view.sonar_hydrophone_view.cavitation_index = 1.25;
        view.sync_sonar_hydrophone_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(450.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(8.5));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(32.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(1.25));

        assert_eq!(
            view.device_rack_state.node_param_values.get("depth_m"),
            Some(&450.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("depth_m"),
            Some(&450.0)
        );
    }

    #[test]
    fn test_turn74_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_idiophone_spectrum_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_idiophone_spectrum_hud_open());
        assert!(!view.device_rack_state.requested_open_idiophone_spectrum_hud);

        view.close_idiophone_spectrum_hud();
        assert!(!view.is_idiophone_spectrum_hud_open());

        view.inspector_state.requested_open_sonar_hydrophone_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_sonar_hydrophone_hud_open());
        assert!(!view.inspector_state.requested_open_sonar_hydrophone_hud);
    }

    #[test]
    fn test_turn74_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let id_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_idiophone_spectrum_hud");
        assert!(id_action.is_some());
        let id = id_action.unwrap();
        assert_eq!(id.category, "Physical Modeling");
        assert!(id.label.contains("Idiophone Modal Resonator"));

        let so_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_sonar_hydrophone_hud");
        assert!(so_action.is_some());
        let so = so_action.unwrap();
        assert_eq!(so.category, "Physical Modeling");
        assert!(so.label.contains("Sonar & Hydrophone"));
    }
}
