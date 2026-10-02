#![allow(clippy::all)]

//! Test suite for Turn #69 / Turn #32 deliverable — Shakuhachi Embouchure Angle & Bowed String Stick-Slip Friction HUDs Convergence (Milestones 16, 25 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::embouchure_angle_view::{
        EmbouchureAngleView, EmbouchurePreset, EMBOUCHURE_PUCK_HIT_RADIUS,
        MIN_ANGLE_DEG, MAX_ANGLE_DEG, MIN_PRESSURE_PA, MAX_PRESSURE_PA,
    };
    use crate::views::friction_orbit_view::{
        FrictionOrbitView, RosinProfile, FRICTION_ORBIT_PUCK_HIT_RADIUS,
        MIN_ORBIT_VELOCITY_MPS, MAX_ORBIT_VELOCITY_MPS, MIN_ORBIT_FORCE_N, MAX_ORBIT_FORCE_N,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn69_pure_embouchure_angle_view_initialization_and_presets() {
        let mut view = EmbouchureAngleView::new();
        assert_eq!(view.preset, EmbouchurePreset::HonkyokuD4);
        assert!((view.embouchure_angle_deg - 38.0).abs() < 1e-3);
        assert!((view.blowing_pressure_pa - 850.0).abs() < 1e-3);
        assert_eq!(EMBOUCHURE_PUCK_HIT_RADIUS, 22.0);

        // Test presets
        view.set_preset(EmbouchurePreset::JinashiZenA3);
        assert_eq!(view.preset, EmbouchurePreset::JinashiZenA3);

        view.set_preset(EmbouchurePreset::MinyoE4);
        assert_eq!(view.preset, EmbouchurePreset::MinyoE4);

        view.set_preset(EmbouchurePreset::SankyokuB3);
        assert_eq!(view.preset, EmbouchurePreset::SankyokuB3);

        view.set_preset(EmbouchurePreset::MuraiIkiExplosive);
        assert_eq!(view.preset, EmbouchurePreset::MuraiIkiExplosive);

        view.set_preset(EmbouchurePreset::KyotakuD3);
        assert_eq!(view.preset, EmbouchurePreset::KyotakuD3);

        // Physics updates from puck
        view.update_physics_from_puck(0.5, 0.5);
        let exp_angle = MIN_ANGLE_DEG + 0.5 * (MAX_ANGLE_DEG - MIN_ANGLE_DEG);
        let exp_press = MIN_PRESSURE_PA + 0.5 * (MAX_PRESSURE_PA - MIN_PRESSURE_PA);
        assert!((view.embouchure_angle_deg - exp_angle).abs() < 1e-3);
        assert!((view.blowing_pressure_pa - exp_press).abs() < 1e-3);

        // Acoustics
        assert!(view.jet_velocity_mps >= 10.0 && view.jet_velocity_mps <= 80.0);
        assert!(view.vortex_frequency_hz >= 200.0 && view.vortex_frequency_hz <= 5000.0);
        assert_eq!(view.harmonic_weights.len(), 8);
        assert!(view.harmonic_weights[0] > 0.0);

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("EMBOUCHURE"));
    }

    #[test]
    fn test_turn69_pure_friction_orbit_view_initialization_and_rosin_profiles() {
        let mut view = FrictionOrbitView::new();
        assert_eq!(view.rosin, RosinProfile::LightViolin);
        assert!((view.bow_velocity_mps - 0.45).abs() < 1e-3);
        assert!((view.bow_force_n - 1.25).abs() < 1e-3);
        assert!((view.bridge_proximity_beta - 0.12).abs() < 1e-3);
        assert_eq!(FRICTION_ORBIT_PUCK_HIT_RADIUS, 22.0);

        // Test Rosin Profiles
        view.set_rosin(RosinProfile::MediumCello);
        assert_eq!(view.rosin, RosinProfile::MediumCello);

        view.set_rosin(RosinProfile::DarkDoubleBass);
        assert_eq!(view.rosin, RosinProfile::DarkDoubleBass);

        view.set_rosin(RosinProfile::SyntheticClean);
        assert_eq!(view.rosin, RosinProfile::SyntheticClean);

        // Normalized conversions
        let norm_v = FrictionOrbitView::velocity_to_normalized(view.bow_velocity_mps);
        let back_v = FrictionOrbitView::normalized_to_velocity(norm_v);
        assert!((back_v - view.bow_velocity_mps).abs() < 1e-3);

        let norm_f = FrictionOrbitView::force_to_normalized(view.bow_force_n);
        let back_f = FrictionOrbitView::normalized_to_force(norm_f);
        assert!((back_f - view.bow_force_n).abs() < 1e-3);
        assert!(view.bow_velocity_mps >= MIN_ORBIT_VELOCITY_MPS && view.bow_velocity_mps <= MAX_ORBIT_VELOCITY_MPS);
        assert!(view.bow_force_n >= MIN_ORBIT_FORCE_N && view.bow_force_n <= MAX_ORBIT_FORCE_N);

        // Friction characteristic evaluation
        let mu_zero = view.evaluate_friction_curve(0.0);
        assert!((mu_zero - 0.85).abs() < 1e-3);
        let mu_pos = view.evaluate_friction_curve(0.5);
        assert!(mu_pos > 0.0);
        let mu_neg = view.evaluate_friction_curve(-0.5);
        assert!(mu_neg < 0.0);

        // Phase orbit point evaluation
        let (v0, f0) = view.evaluate_orbit_point(0.0);
        assert!(v0.is_finite());
        assert!(f0.is_finite());

        // Hit testing
        let canvas = Rect { x: 30.0, y: 50.0, width: 220.0, height: 160.0 };
        let puck_x = canvas.x + view.orbit_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.orbit_puck_pos.1) * canvas.height;
        assert!(view.hit_test_orbit_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_orbit_puck((puck_x + 8.0, puck_y - 8.0), canvas));
        assert!(!view.hit_test_orbit_puck((puck_x + 45.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("*"));
        assert!(snap.contains("O"));
    }

    #[test]
    fn test_turn69_pure_parambus_slot_arithmetic_channel_isolation() {
        let track_id = 3u32;
        let slot_idx = 2u32;
        let base_id = track_id * 1000 + slot_idx * 20;

        let pid_ang  = ParamId(base_id);
        let pid_prs  = ParamId(base_id + 1);
        let pid_vel  = ParamId(base_id + 2);
        let pid_mer  = ParamId(base_id + 3);

        assert_eq!(pid_ang.0, 3040);
        assert_eq!(pid_prs.0, 3041);
        assert_eq!(pid_vel.0, 3042);
        assert_eq!(pid_mer.0, 3043);

        // Verify next slot isolation
        let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
        assert_eq!(next_slot_base, 3060);
        assert!(next_slot_base > pid_mer.0);
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
    fn test_turn69_gui_award_winning_embouchure_and_friction_orbit_hud_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_embouchure_angle_hud_open());
        assert!(!view.is_friction_orbit_hud_open());

        view.open_embouchure_angle_hud();
        assert!(view.is_embouchure_angle_hud_open());
        view.close_embouchure_angle_hud();
        assert!(!view.is_embouchure_angle_hud_open());

        view.open_friction_orbit_hud();
        assert!(view.is_friction_orbit_hud_open());
        view.close_friction_orbit_hud();
        assert!(!view.is_friction_orbit_hud_open());
    }

    #[test]
    fn test_turn69_gui_headless_rendering_no_panics() {
        let mut view = AwardWinningGuiView::new();
        view.open_embouchure_angle_hud();
        view.open_friction_orbit_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_embouchure_angle_hud_open());
        assert!(view.is_friction_orbit_hud_open());
    }

    #[test]
    fn test_turn69_gui_live_parambus_atomic_sync_embouchure_and_friction() {
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

        // 1. Sync Shakuhachi Embouchure Angle HUD
        view.embouchure_angle_view.embouchure_angle_deg = 42.5;
        view.embouchure_angle_view.blowing_pressure_pa = 1200.0;
        view.embouchure_angle_view.jet_velocity_mps = 44.6;
        view.embouchure_angle_view.meri_kari_cents = -50.0;
        view.sync_embouchure_angle_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(42.5));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(1200.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(44.6));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(-50.0));

        assert_eq!(view.device_rack_state.node_param_values.get("embouchure_angle_deg"), Some(&42.5));
        assert_eq!(view.device_rack_state.node_param_values.get("blowing_pressure_pa"), Some(&1200.0));
        assert_eq!(view.inspector_state.node_param_values.get("embouchure_angle_deg"), Some(&42.5));

        // 2. Sync Bowed String Friction Orbit HUD
        view.friction_orbit_view.bow_velocity_mps = 0.85;
        view.friction_orbit_view.bow_force_n = 2.40;
        view.friction_orbit_view.rosin_adhesion_pct = 76.5;
        view.friction_orbit_view.helmholtz_coherence_score = 0.91;
        view.sync_friction_orbit_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(2.40));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(76.5));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.91));

        assert_eq!(view.device_rack_state.node_param_values.get("bow_velocity_mps"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("bow_force_n"), Some(&2.40));
        assert_eq!(view.inspector_state.node_param_values.get("bow_velocity_mps"), Some(&0.85));
    }

    #[test]
    fn test_turn69_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_embouchure_angle_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_embouchure_angle_hud_open());
        assert!(!view.device_rack_state.requested_open_embouchure_angle_hud);

        view.close_embouchure_angle_hud();
        assert!(!view.is_embouchure_angle_hud_open());

        view.inspector_state.requested_open_friction_orbit_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_friction_orbit_hud_open());
        assert!(!view.inspector_state.requested_open_friction_orbit_hud);
    }

    #[test]
    fn test_turn69_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let emb_action = palette.actions.iter().find(|a| a.action_id == "open_embouchure_angle_hud");
        assert!(emb_action.is_some());
        let emb = emb_action.unwrap();
        assert_eq!(emb.category, "Physical Modeling");
        assert!(emb.label.contains("Embouchure"));

        let fric_action = palette.actions.iter().find(|a| a.action_id == "open_friction_orbit_hud");
        assert!(fric_action.is_some());
        let fric = fric_action.unwrap();
        assert_eq!(fric.category, "Physical Modeling");
        assert!(fric.label.contains("Friction"));
    }
}
