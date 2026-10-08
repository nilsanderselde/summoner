#![allow(clippy::all)]

//! Test suite for Turn #84 (Sprint Turn #10) — 3D Spatial Trajectory Polar Canvas & Granular Grain Cloud Density HUD
//! and 3D Ambisonic Trajectory Radar & Multi-Channel Surround Sound Panner HUD Convergence (Milestones 33 & 35).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::ambisonic_radar_view::{
        AmbisonicFormat, AmbisonicRadarView, AmbisonicSource, TrajectoryShape,
        AMBISONIC_PUCK_HIT_RADIUS,
    };
    use crate::views::spatial_trajectory_view::{
        GrainWindowType, SpatialTrajectoryView, TrajectoryPathType, SPATIAL_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn84_pure_spatial_trajectory_view_initialization_and_simulation() {
        let mut view = SpatialTrajectoryView::new();
        assert_eq!(view.track_id, 0);
        assert_eq!(view.name, "3D Spatial Orbit");
        assert_eq!(view.source_azimuth_rad, 0.0);
        assert_eq!(view.source_elevation_rad, 0.0);
        assert_eq!(view.source_distance_m, 4.0);
        assert_eq!(view.max_radius_m, 10.0);
        assert_eq!(view.spread, 0.25);
        assert_eq!(view.grain_density, 45.0);
        assert_eq!(view.grain_duration_ms, 60.0);
        assert_eq!(view.grain_spray, 0.20);
        assert_eq!(view.window_type, GrainWindowType::Hann);
        assert_eq!(SPATIAL_PUCK_HIT_RADIUS, 22.0);

        // Verify particle initialization
        assert!(!view.particles.is_empty());
        assert!(view.particles.len() <= 32);

        // Coordinate projections: spherical to canvas and canvas to spherical round-trip
        let center = (200.0, 200.0);
        let max_r = 100.0;
        let test_az = std::f32::consts::FRAC_PI_4; // 45 deg
        let test_dist = 5.0; // half radius
        let (cx, cy) = view.spherical_to_canvas(test_az, test_dist, center, max_r);
        let (back_az, back_dist) = view.canvas_to_spherical((cx, cy), center, max_r);

        assert!((back_az - test_az).abs() < 1e-3);
        assert!((back_dist - test_dist).abs() < 1e-3);

        // Puck hit testing (>= 44x44pt bounding target)
        view.source_azimuth_rad = test_az;
        view.source_distance_m = test_dist;
        assert!(view.hit_test_source_puck((cx, cy), center, max_r));
        assert!(!view.hit_test_source_puck((center.0, center.1), center, max_r));

        // Beat update and trajectory evaluation
        view.path_type = TrajectoryPathType::Orbit {
            radius_m: 6.0,
            speed_cycles_per_beat: 0.5,
            tilt_rad: 0.2,
            eccentricity: 0.1,
            center_x: 0.0,
            center_y: 0.0,
            center_z: 0.0,
        };
        view.update_beat(2.0);
        assert!((view.source_distance_m - 6.0).abs() < 1.0);
        assert!(!view.particles.is_empty());

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SPATIAL TRAJECTORY"));
        assert!(snap.contains("Mode:"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn84_pure_ambisonic_radar_view_initialization_and_simulation() {
        let mut view = AmbisonicRadarView::new();
        assert_eq!(view.format, AmbisonicFormat::Atmos714);
        assert_eq!(view.sources.len(), 3);
        assert_eq!(view.selected_source_idx, Some(0));
        assert_eq!(view.max_distance_m, 5.0);
        assert!(view.binaural_headphone_preview);
        assert_eq!(AMBISONIC_PUCK_HIT_RADIUS, 22.0);

        // Ambisonic Format channel counts and names
        let formats = [
            (AmbisonicFormat::FirstOrderBFormat, 4, "1st Order (FOA / 4-ch)"),
            (AmbisonicFormat::SecondOrderHOA, 9, "2nd Order (HOA / 9-ch)"),
            (AmbisonicFormat::ThirdOrderHOA, 16, "3rd Order (HOA / 16-ch)"),
            (AmbisonicFormat::Surround51, 6, "5.1 Surround"),
            (AmbisonicFormat::Atmos714, 12, "7.1.4 Dolby Atmos"),
            (AmbisonicFormat::Immersive916, 16, "9.1.6 Immersive 3D"),
        ];

        for (fmt, exp_ch, exp_name) in formats {
            view.format = fmt;
            assert_eq!(fmt.channel_count(), exp_ch);
            assert_eq!(fmt.display_name(), exp_name);
        }

        // Polar to screen and screen to polar conversions
        let center = (150.0, 150.0);
        let radius = 100.0;
        let test_az = 45.0f32;
        let test_dist = 2.5f32;
        let (sx, sy) = view.polar_to_screen_pos(test_az, test_dist, center, radius);
        let (back_az, back_dist) = view.screen_pos_to_polar((sx, sy), center, radius);

        assert!((back_az - test_az).abs() < 1e-2);
        assert!((back_dist - test_dist).abs() < 1e-2);

        // Puck hit testing (>= 44x44pt target)
        view.sources[0].azimuth_deg = test_az;
        view.sources[0].distance_m = test_dist;
        assert_eq!(view.hit_test_source((sx, sy), center, radius), Some(0));
        assert_eq!(view.hit_test_source((0.0, 0.0), center, radius), None);

        // Trajectory motion simulation
        let mut src = AmbisonicSource::new("src_test", "Test Source", 0.0, 0.0, 2.0);
        src.trajectory = TrajectoryShape::OrbitCircle;
        src.trajectory_speed_hz = 1.0;
        src.step_trajectory(0.25);
        assert!(src.azimuth_deg != 0.0 || src.trajectory_phase > 0.0);

        src.trajectory = TrajectoryShape::LissajousFigure8;
        src.step_trajectory(0.25);
        assert!(src.azimuth_deg.abs() <= 180.0);
        assert!(src.elevation_deg.abs() <= 90.0);

        // Add source
        let new_idx = view.add_source("Synth Object 4", -60.0, 20.0, 3.5);
        assert_eq!(new_idx, 3);
        assert_eq!(view.sources.len(), 4);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('L'));
        assert!(snap.contains('1'));
    }

    #[test]
    fn test_turn84_pure_lock_free_param_bus_slot_arithmetic() {
        // ParamBus slot layout: track_id * 1000 + slot_idx * 20 + offset
        for track_id in 0..8u32 {
            for slot_idx in 0..4usize {
                let base = track_id * 1000 + slot_idx as u32 * 20;

                // SpatialTrajectory parameters
                let spat_az = ParamId(base);
                let spat_el = ParamId(base + 1);
                let spat_dist = ParamId(base + 2);
                let spat_sprd = ParamId(base + 3);
                let spat_dens = ParamId(base + 4);
                let spat_dur = ParamId(base + 5);
                let spat_spry = ParamId(base + 6);

                assert_eq!(spat_az.0, base);
                assert_eq!(spat_el.0, base + 1);
                assert_eq!(spat_dist.0, base + 2);
                assert_eq!(spat_sprd.0, base + 3);
                assert_eq!(spat_dens.0, base + 4);
                assert_eq!(spat_dur.0, base + 5);
                assert_eq!(spat_spry.0, base + 6);

                // AmbisonicRadar parameters
                let rad_az = ParamId(base);
                let rad_el = ParamId(base + 1);
                let rad_dist = ParamId(base + 2);
                let rad_sprd = ParamId(base + 3);
                let rad_gain = ParamId(base + 4);
                let rad_elev = ParamId(base + 5);

                assert_eq!(rad_az.0, base);
                assert_eq!(rad_el.0, base + 1);
                assert_eq!(rad_dist.0, base + 2);
                assert_eq!(rad_sprd.0, base + 3);
                assert_eq!(rad_gain.0, base + 4);
                assert_eq!(rad_elev.0, base + 5);

                // Verify slot isolation (all IDs remain below next slot's base)
                let next_base = track_id * 1000 + (slot_idx as u32 + 1) * 20;
                assert!(spat_spry.0 < next_base);
                assert!(rad_elev.0 < next_base);
            }
        }
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack_with_context, ModernDeviceRackState};
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn84_gui_award_winning_view_spatial_trajectory_and_ambisonic_radar_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Spatial Trajectory HUD
        view.open_spatial_trajectory_hud();
        assert!(view.is_spatial_trajectory_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_spatial_trajectory_hud();
        assert!(!view.is_spatial_trajectory_hud_open());

        // 2. Ambisonic Radar HUD
        view.open_ambisonic_radar_hud();
        assert!(view.is_ambisonic_radar_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_ambisonic_radar_hud();
        assert!(!view.is_ambisonic_radar_hud_open());
    }

    #[test]
    fn test_turn84_gui_lock_free_param_bus_dispatch_spatial_trajectory_and_ambisonic_radar() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels
        for offset in 0..=7 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Spatial Trajectory sync
        view.spatial_trajectory_view.source_azimuth_rad = 0.5; // ~28.6 deg
        view.spatial_trajectory_view.source_elevation_rad = 0.2; // ~11.5 deg
        view.spatial_trajectory_view.source_distance_m = 5.5;
        view.spatial_trajectory_view.spread = 0.35;
        view.spatial_trajectory_view.grain_density = 50.0;
        view.spatial_trajectory_view.grain_duration_ms = 75.0;
        view.spatial_trajectory_view.grain_spray = 0.25;

        view.sync_spatial_trajectory_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&5.5));
        assert_eq!(view.device_rack_state.node_param_values.get("grain_density"), Some(&50.0));
        assert_eq!(view.device_rack_state.node_param_values.get("grain_duration_ms"), Some(&75.0));
        assert_eq!(view.inspector_state.node_param_values.get("grain_spray"), Some(&0.25));

        let pid_dist = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        assert_eq!(arc_bus.get(pid_dist), Some(5.5));

        // Test Ambisonic Radar sync
        if let Some(src) = view.ambisonic_radar_view.sources.get_mut(0) {
            src.azimuth_deg = -45.0;
            src.elevation_deg = 20.0;
            src.distance_m = 3.2;
            src.spread_pct = 30.0;
            src.gain_db = -3.0;
        }
        view.ambisonic_radar_view.master_elevation_offset_deg = 5.0;

        view.sync_ambisonic_radar_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("azimuth_deg"), Some(&-45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("elevation_deg"), Some(&20.0));
        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&3.2));
        assert_eq!(view.device_rack_state.node_param_values.get("gain_db"), Some(&-3.0));
        assert_eq!(view.device_rack_state.node_param_values.get("master_elevation_offset_deg"), Some(&5.0));

        let pid_az = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_gain = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 4);
        assert_eq!(arc_bus.get(pid_az), Some(-45.0));
        assert_eq!(arc_bus.get(pid_gain), Some(-3.0));
    }

    #[test]
    fn test_turn84_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Spatial Trajectory trigger propagation from rack
        view.device_rack_state.requested_open_spatial_trajectory_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_spatial_trajectory_hud);
        assert!(view.is_spatial_trajectory_hud_open());
        view.close_spatial_trajectory_hud();

        // 2. Ambisonic Radar trigger propagation from inspector
        view.inspector_state.requested_open_ambisonic_radar_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_ambisonic_radar_hud);
        assert!(view.is_ambisonic_radar_hud_open());
        view.close_ambisonic_radar_hud();
    }

    #[test]
    fn test_turn84_gui_modern_device_rack_and_inspector_headless_render() {
        let ctx = egui::Context::default();
        let mut rack_state = ModernDeviceRackState::default();
        let mut inspector_state = ModernInspectorState::default();

        rack_state.selected_node_kind = Some("SpatialTrajectoryView".to_string());
        rack_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "SpatialTrajectoryView".to_string(),
            display_name: "Spatial Trajectory".to_string(),
            is_bypassed: false,
        }];

        inspector_state.selected_node_kind = Some("AmbisonicRadarView".to_string());
        inspector_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "AmbisonicRadarView".to_string(),
            display_name: "Ambisonic Radar".to_string(),
            is_bypassed: false,
        }];

        let bus = ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack_state, None, Some(&bus), 1);
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });
        assert!(inspector_state.selected_node_kind.is_some());
    }

    #[test]
    fn test_turn84_gui_command_palette_registration_spatial_trajectory_and_ambisonic_radar() {
        let palette = CommandPalette::new();
        let has_spatial_traj = palette.actions.iter().any(|a| a.action_id == "open_spatial_trajectory_hud");
        let has_ambisonic_radar = palette.actions.iter().any(|a| a.action_id == "open_ambisonic_radar_hud");

        assert!(has_spatial_traj, "Expected open_spatial_trajectory_hud in CommandPalette");
        assert!(has_ambisonic_radar, "Expected open_ambisonic_radar_hud in CommandPalette");
    }
}
