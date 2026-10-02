#![allow(clippy::all)]

//! Test suite for Turn #71 (Turn #34 deliverable) — Physical Spring Reverb Tank & Spectral Comb Resonator HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::comb_resonator_view::{
        CombPolarity, CombResonatorProfile, CombResonatorView, COMB_PUCK_HIT_RADIUS,
    };
    use crate::views::spring_reverb_view::{
        SpringReverbProfile, SpringReverbView, SPRING_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn71_pure_spring_reverb_view_initialization_and_profiles() {
        let mut view = SpringReverbView::new();
        assert_eq!(view.profile, SpringReverbProfile::AccutronicsType4);
        assert_eq!(view.num_springs, 3);
        assert!((view.tension_pct - 60.0).abs() < 1e-3);
        assert!((view.dispersion_chirp_pct - 65.0).abs() < 1e-3);
        assert_eq!(SPRING_PUCK_HIT_RADIUS, 22.0);

        // Test Profiles
        view.set_profile(SpringReverbProfile::AccutronicsType4);
        assert_eq!(view.profile, SpringReverbProfile::AccutronicsType4);
        assert_eq!(view.num_springs, 2);
        assert!((view.tension_pct - 55.0).abs() < 1e-3);
        assert!((view.dispersion_chirp_pct - 60.0).abs() < 1e-3);

        view.set_profile(SpringReverbProfile::StudioMasterType9);
        assert_eq!(view.profile, SpringReverbProfile::StudioMasterType9);
        assert_eq!(view.num_springs, 3);
        assert!((view.tension_pct - 70.0).abs() < 1e-3);
        assert!((view.decay_seconds - 4.8).abs() < 1e-3);

        view.set_profile(SpringReverbProfile::DubSpaceDrip);
        assert_eq!(view.profile, SpringReverbProfile::DubSpaceDrip);
        assert!((view.dispersion_chirp_pct - 88.0).abs() < 1e-3);
        assert!((view.drive_saturation_db - 12.0).abs() < 1e-3);

        view.set_profile(SpringReverbProfile::SurfSpringKick);
        assert_eq!(view.profile, SpringReverbProfile::SurfSpringKick);
        assert_eq!(view.num_springs, 2);
        assert!((view.tension_pct - 85.0).abs() < 1e-3);

        view.set_profile(SpringReverbProfile::LoFiTrashChamber);
        assert_eq!(view.profile, SpringReverbProfile::LoFiTrashChamber);
        assert!((view.tension_pct - 20.0).abs() < 1e-3);
        assert!((view.decay_seconds - 1.6).abs() < 1e-3);

        // Dispersion delay ms check
        let delay_low = view.calculate_dispersion_delay_ms(100.0);
        let delay_high = view.calculate_dispersion_delay_ms(10000.0);
        assert!(delay_low > delay_high);

        // Spring coil vertices
        let pts = view.generate_spring_coil_vertices(0, 400.0, 200.0);
        assert_eq!(pts.len(), 40);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        view.pluck_puck_pos = (0.5, 0.5);
        let px = canvas.x + 0.5 * canvas.width;
        let py = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_pluck_puck((px, py), canvas));
        assert!(!view.hit_test_pluck_puck((px + 100.0, py), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SPRING REVERB"));
    }

    #[test]
    fn test_turn71_pure_comb_resonator_view_initialization_and_profiles() {
        let mut view = CombResonatorView::new();
        assert_eq!(view.profile, CombResonatorProfile::TunedChimeA4);
        assert!((view.base_frequency_hz - 440.0).abs() < 1e-3);
        assert!((view.feedback_pct - 85.0).abs() < 1e-3);
        assert_eq!(COMB_PUCK_HIT_RADIUS, 22.0);

        // Test Profiles
        view.set_profile(CombResonatorProfile::SubHollowNotch);
        assert_eq!(view.profile, CombResonatorProfile::SubHollowNotch);
        assert!((view.base_frequency_hz - 55.0).abs() < 1e-3);
        assert_eq!(view.polarity, CombPolarity::Negative);

        view.set_profile(CombResonatorProfile::FormantVocalFlange);
        assert_eq!(view.profile, CombResonatorProfile::FormantVocalFlange);
        assert!((view.base_frequency_hz - 1200.0).abs() < 1e-3);
        assert_eq!(view.polarity, CombPolarity::ComplexRing);

        view.set_profile(CombResonatorProfile::StereoQuadrature);
        assert_eq!(view.profile, CombResonatorProfile::StereoQuadrature);
        assert!((view.base_frequency_hz - 880.0).abs() < 1e-3);
        assert_eq!(view.polarity, CombPolarity::Positive);

        view.set_profile(CombResonatorProfile::MetallicHighQ);
        assert_eq!(view.profile, CombResonatorProfile::MetallicHighQ);
        assert!((view.feedback_pct - 96.0).abs() < 1e-3);

        // Frequency logarithmic conversions roundtrip
        for test_f in [20.0, 100.0, 440.0, 1000.0, 5000.0, 20000.0] {
            let norm = CombResonatorView::freq_to_normalized(test_f);
            let back = CombResonatorView::normalized_to_freq(norm);
            assert!((back - test_f).abs() < test_f * 0.02);
        }

        // Magnitude response evaluation
        let mag = view.evaluate_magnitude_response(3200.0);
        assert!(mag >= 0.0 && mag <= 1.0);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        view.puck_pos = (0.5, 0.5);
        let px = canvas.x + 0.5 * canvas.width;
        let py = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_puck((px, py), canvas));
        assert!(!view.hit_test_puck((px + 100.0, py), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("COMB RESONATOR"));
    }

    #[test]
    fn test_turn71_pure_slot_arithmetic_channel_isolation() {
        let track_id = 3u32;
        let slot_id = 2u32;

        let pid_0 = ParamId(track_id * 1000 + slot_id * 20);
        let pid_1 = ParamId(track_id * 1000 + slot_id * 20 + 1);
        let pid_2 = ParamId(track_id * 1000 + slot_id * 20 + 2);
        let pid_3 = ParamId(track_id * 1000 + slot_id * 20 + 3);

        assert_eq!(pid_0.0, 3040);
        assert_eq!(pid_1.0, 3041);
        assert_eq!(pid_2.0, 3042);
        assert_eq!(pid_3.0, 3043);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 3060);
        assert!(pid_3.0 < next_slot_pid.0);
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
    fn test_turn71_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Spring Reverb modal
        view.open_spring_reverb_hud();
        assert!(view.is_spring_reverb_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_spring_reverb_hud();
        assert!(!view.is_spring_reverb_hud_open());

        // Open Comb Resonator modal
        view.open_comb_resonator_hud();
        assert!(view.is_comb_resonator_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_comb_resonator_hud();
        assert!(!view.is_comb_resonator_hud_open());
    }

    #[test]
    fn test_turn71_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync Spring Reverb Tank HUD
        view.spring_reverb_view.tension_pct = 75.0;
        view.spring_reverb_view.dispersion_chirp_pct = 82.0;
        view.spring_reverb_view.decay_seconds = 4.2;
        view.spring_reverb_view.drive_saturation_db = 9.5;
        view.sync_spring_reverb_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(75.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(82.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(4.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(9.5));

        assert_eq!(view.device_rack_state.node_param_values.get("tension_pct"), Some(&75.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dispersion_chirp_pct"), Some(&82.0));
        assert_eq!(view.inspector_state.node_param_values.get("tension_pct"), Some(&75.0));

        // 2. Sync Comb Resonator HUD
        view.comb_resonator_view.base_frequency_hz = 520.0;
        view.comb_resonator_view.feedback_pct = 88.0;
        view.comb_resonator_view.dampening_hz = 7200.0;
        view.comb_resonator_view.stereo_spread_pct = 45.0;
        view.sync_comb_resonator_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(520.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(88.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(7200.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(45.0));

        assert_eq!(view.device_rack_state.node_param_values.get("base_frequency_hz"), Some(&520.0));
        assert_eq!(view.device_rack_state.node_param_values.get("feedback_pct"), Some(&88.0));
        assert_eq!(view.inspector_state.node_param_values.get("base_frequency_hz"), Some(&520.0));
    }

    #[test]
    fn test_turn71_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_spring_reverb_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spring_reverb_hud_open());
        assert!(!view.device_rack_state.requested_open_spring_reverb_hud);

        view.close_spring_reverb_hud();
        assert!(!view.is_spring_reverb_hud_open());

        view.inspector_state.requested_open_comb_resonator_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_comb_resonator_hud_open());
        assert!(!view.inspector_state.requested_open_comb_resonator_hud);
    }

    #[test]
    fn test_turn71_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let spring_action = palette.actions.iter().find(|a| a.action_id == "open_spring_reverb_hud");
        assert!(spring_action.is_some());
        let spring = spring_action.unwrap();
        assert_eq!(spring.category, "Physical Modeling");
        assert!(spring.label.contains("Spring Reverb"));

        let comb_action = palette.actions.iter().find(|a| a.action_id == "open_comb_resonator_hud");
        assert!(comb_action.is_some());
        let comb = comb_action.unwrap();
        assert_eq!(comb.category, "Physical Modeling");
        assert!(comb.label.contains("Comb"));
    }
}
