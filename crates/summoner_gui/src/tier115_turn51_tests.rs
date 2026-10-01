#![cfg(feature = "gui")]

// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 115 Test Suite — Turn #51 (Turn #17):
//! Interactive 3D Holographic Ambisonic Radar HUD & Polar Trajectory Canvas (Milestone 35).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::hoa5_radar_view::{
        Hoa5RadarPreset, Hoa5RadarView, TrajectoryMode, HOA5_RADAR_PUCK_HIT_RADIUS,
        HOA5_TOTAL_CHANNELS, MAX_RADAR_AZIMUTH_DEG, MAX_RADAR_DISTANCE_M,
        MAX_RADAR_ELEVATION_DEG, MIN_RADAR_AZIMUTH_DEG, MIN_RADAR_DISTANCE_M,
        MIN_RADAR_ELEVATION_DEG,
    };

    #[test]
    fn test_turn51_hoa5_radar_view_initialization() {
        let view = Hoa5RadarView::new();

        assert_eq!(view.preset, Hoa5RadarPreset::Hoa5_36ChannelSphere);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-4);
        assert!((view.elevation_deg - 20.0).abs() < 1e-4);
        assert!((view.distance_m - 3.0).abs() < 1e-4);
        assert_eq!(view.ambisonic_order, 5);
        assert!((view.energy_focus - 0.70).abs() < 1e-4);
        assert!(view.wavefront_anim_enabled);
        assert_eq!(view.trajectory_mode, TrajectoryMode::ManualStatic);
        assert_eq!(view.spherical_harmonic_levels.len(), HOA5_TOTAL_CHANNELS);
        assert_eq!(view.order_energies.len(), 6);
    }

    #[test]
    fn test_turn51_hoa5_radar_presets() {
        let mut view = Hoa5RadarView::new();

        let presets = [
            (Hoa5RadarPreset::Hoa5_36ChannelSphere, 45.0, 20.0, 3.0, 0.70),
            (Hoa5RadarPreset::MaxReAcousticFocus, 0.0, 15.0, 2.0, 0.95),
            (Hoa5RadarPreset::BinauralHoa5Hrir, -60.0, 0.0, 1.5, 0.85),
            (Hoa5RadarPreset::Surround22_2Broadcast, 90.0, 30.0, 4.5, 0.80),
            (Hoa5RadarPreset::Dome9_1_6Master, -135.0, 45.0, 3.5, 0.75),
            (Hoa5RadarPreset::DopplerOrbitalFlyby, 0.0, 10.0, 2.5, 0.65),
        ];

        for (p, exp_az, exp_el, exp_dist, exp_foc) in presets {
            view.set_preset(p);
            assert_eq!(view.preset, p);
            assert!((view.azimuth_deg - exp_az).abs() < 1e-3, "Preset {:?} Azimuth mismatch", p);
            assert!((view.elevation_deg - exp_el).abs() < 1e-3, "Preset {:?} Elevation mismatch", p);
            assert!((view.distance_m - exp_dist).abs() < 1e-3, "Preset {:?} Distance mismatch", p);
            assert!((view.energy_focus - exp_foc).abs() < 1e-3, "Preset {:?} Focus mismatch", p);
            assert_eq!(view.ambisonic_order, 5);
        }
    }

    #[test]
    fn test_turn51_spherical_to_cartesian_coordinates() {
        let mut view = Hoa5RadarView::new();

        // 1. North/Front: Azimuth 0°, Elevation 0°, Distance 5.0m -> X=0, Y=5, Z=0
        view.set_azimuth_elevation_distance(0.0, 0.0, 5.0);
        let (x1, y1, z1) = view.cartesian_coordinates();
        assert!(x1.abs() < 1e-3);
        assert!((y1 - 5.0).abs() < 1e-3);
        assert!(z1.abs() < 1e-3);

        // 2. East/Right: Azimuth 90°, Elevation 0°, Distance 5.0m -> X=5, Y=0, Z=0
        view.set_azimuth_elevation_distance(90.0, 0.0, 5.0);
        let (x2, y2, z2) = view.cartesian_coordinates();
        assert!((x2 - 5.0).abs() < 1e-3);
        assert!(y2.abs() < 1e-3);
        assert!(z2.abs() < 1e-3);

        // 3. Zenith Overhead: Azimuth 0°, Elevation 90°, Distance 5.0m -> X=0, Y=0, Z=5
        view.set_azimuth_elevation_distance(0.0, 90.0, 5.0);
        let (x3, y3, z3) = view.cartesian_coordinates();
        assert!(x3.abs() < 1e-3);
        assert!(y3.abs() < 1e-3);
        assert!((z3 - 5.0).abs() < 1e-3);

        // 4. Clamping bounds
        view.set_azimuth_elevation_distance(500.0, -200.0, 100.0);
        assert_eq!(view.azimuth_deg, MAX_RADAR_AZIMUTH_DEG);
        assert_eq!(view.elevation_deg, MIN_RADAR_ELEVATION_DEG);
        assert_eq!(view.distance_m, MAX_RADAR_DISTANCE_M);

        view.set_azimuth_elevation_distance(-500.0, 200.0, -10.0);
        assert_eq!(view.azimuth_deg, MIN_RADAR_AZIMUTH_DEG);
        assert_eq!(view.elevation_deg, MAX_RADAR_ELEVATION_DEG);
        assert_eq!(view.distance_m, MIN_RADAR_DISTANCE_M);
    }

    #[test]
    fn test_turn51_36_channel_spherical_harmonics_decomposition() {
        let mut view = Hoa5RadarView::new();
        view.set_azimuth_elevation_distance(45.0, 30.0, 2.0);

        // Check Monopole (Order 0: W)
        assert!(view.spherical_harmonic_levels[0] > 0.0, "W component must be non-zero");

        // Check Dipoles (Order 1: Y, Z, X)
        assert!(view.spherical_harmonic_levels[1] > 0.0);
        assert!(view.spherical_harmonic_levels[2] > 0.0);
        assert!(view.spherical_harmonic_levels[3] > 0.0);

        // Check Quadrupoles (Order 2: ACN 4..8)
        for i in 4..=8 {
            assert!(view.spherical_harmonic_levels[i] >= 0.0);
        }

        // Check Order energies
        let total_energy: f32 = view.order_energies.iter().sum();
        assert!(total_energy > 0.0, "Total soundfield energy must be positive");
        assert!(view.order_energies[0] > 0.0, "Order 0 energy must be positive");
        assert!(view.order_energies[1] > 0.0, "Order 1 energy must be positive");
    }

    #[test]
    fn test_turn51_doppler_trajectory_simulation() {
        let mut view = Hoa5RadarView::new();
        view.trajectory_mode = TrajectoryMode::DopplerFlybySweep;
        view.trajectory_speed_hz = 1.0;

        // Advance time across multiple steps
        let dt = 0.05;
        let mut observed_doppler = false;

        for _ in 0..20 {
            view.advance_time(dt);
            if (view.doppler_frequency_ratio - 1.0).abs() > 0.001 {
                observed_doppler = true;
                break;
            }
        }

        assert!(observed_doppler, "Doppler ratio should change during flyby trajectory sweep");
        assert!(view.doppler_frequency_ratio >= 0.5 && view.doppler_frequency_ratio <= 2.0);
    }

    #[test]
    fn test_turn51_touch_target_and_puck_hit_radius() {
        // Touch target compliance: hit bounding target must be >= 44x44pt
        let hit_box_diameter = HOA5_RADAR_PUCK_HIT_RADIUS * 2.0;
        assert!(hit_box_diameter >= 44.0, "Hit target diameter must be >= 44.0pt for WCAG AAA touch target compliance");
    }

    #[test]
    fn test_turn51_deterministic_ascii_snapshot() {
        let mut view = Hoa5RadarView::new();
        view.set_preset(Hoa5RadarPreset::Hoa5_36ChannelSphere);

        let snapshot = view.render_snapshot_ascii();
        assert!(snapshot.contains("=== HOA5 3D AMBISONIC RADAR HUD ==="));
        assert!(snapshot.contains("5th-Order HOA (36-Ch Spherical)"));
        assert!(snapshot.contains("Azimuth: +45.0°"));
        assert!(snapshot.contains("Elevation: +20.0°"));
        assert!(snapshot.contains("Order 0 (Monopole 1ch)"));
        assert!(snapshot.contains("Order 5 (Triacon 11ch)"));
        assert!(snapshot.contains("Total Soundfield Energy:"));
    }
}

#[cfg(test)]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::hoa5_radar_view::Hoa5RadarView;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use std::sync::Arc;

    #[test]
    fn test_turn51_hoa5_radar_view_headless_egui_render() {
        let mut view = Hoa5RadarView::new();
        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Ensure state is valid after render
        assert_eq!(view.ambisonic_order, 5);
        assert!(view.distance_m >= 0.5);
    }

    #[test]
    fn test_turn51_award_winning_gui_view_hoa5_modal_lifecycle() {
        let mut daw = AwardWinningGuiView::new();

        assert!(!daw.is_hoa5_radar_hud_open());

        daw.open_hoa5_radar_hud();
        assert!(daw.is_hoa5_radar_hud_open());

        daw.close_hoa5_radar_hud();
        assert!(!daw.is_hoa5_radar_hud_open());
    }

    #[test]
    fn test_turn51_parambus_atomic_slot_dispatch() {
        let mut daw = AwardWinningGuiView::new();
        daw.selected_track_idx = 0; // Track 1 (id: 1, slot: 0)
        let mut bus = ParamBus::new();

        // Track 1, Slot 0 base PID: 1000 + 0 = 1000
        let pid_az = ParamId(1000);
        let pid_el = ParamId(1001);
        let pid_dist = ParamId(1002);
        let pid_ord = ParamId(1003);
        let pid_foc = ParamId(1004);

        bus.register(pid_az, 0.0);
        bus.register(pid_el, 0.0);
        bus.register(pid_dist, 1.0);
        bus.register(pid_ord, 1.0);
        bus.register(pid_foc, 0.5);

        let bus = Arc::new(bus);
        daw.live_param_bus = Some(bus.clone());

        daw.hoa5_radar_view.azimuth_deg = 65.0;
        daw.hoa5_radar_view.elevation_deg = 35.0;
        daw.hoa5_radar_view.distance_m = 4.2;
        daw.hoa5_radar_view.ambisonic_order = 5;
        daw.hoa5_radar_view.energy_focus = 0.90;

        daw.sync_hoa5_radar_hud_state();

        assert!((bus.get(pid_az).unwrap() - 65.0).abs() < 1e-4);
        assert!((bus.get(pid_el).unwrap() - 35.0).abs() < 1e-4);
        assert!((bus.get(pid_dist).unwrap() - 4.2).abs() < 1e-4);
        assert!((bus.get(pid_ord).unwrap() - 5.0).abs() < 1e-4);
        assert!((bus.get(pid_foc).unwrap() - 0.90).abs() < 1e-4);
    }

    #[test]
    fn test_turn51_device_rack_and_inspector_triggers() {
        let mut daw = AwardWinningGuiView::new();

        // 1. Device rack trigger
        assert!(!daw.is_hoa5_radar_hud_open());
        daw.device_rack_state.requested_open_hoa5_radar_hud = true;

        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                daw.show(ui);
            });
        });

        assert!(daw.is_hoa5_radar_hud_open());
        assert!(!daw.device_rack_state.requested_open_hoa5_radar_hud);

        daw.close_hoa5_radar_hud();
        assert!(!daw.is_hoa5_radar_hud_open());

        // 2. Inspector trigger
        daw.inspector_state.requested_open_hoa5_radar_hud = true;
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                daw.show(ui);
            });
        });

        assert!(daw.is_hoa5_radar_hud_open());
        assert!(!daw.inspector_state.requested_open_hoa5_radar_hud);
    }

    #[test]
    fn test_turn51_command_palette_hoa5_action() {
        let palette = crate::command_palette::CommandPalette::new();
        let action = palette.actions.iter().find(|a| a.action_id == "open_hoa5_radar_hud");

        assert!(action.is_some(), "open_hoa5_radar_hud must be registered in command palette");
        let act = action.unwrap();
        assert_eq!(act.category, "Spatial & Ambisonics");
        assert!(act.label.contains("5th-Order Ambisonics"));
    }
}
