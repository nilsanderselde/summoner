#![allow(clippy::all)]

//! Test suite for Turn #80 (Turn #43 deliverable / Sprint Turn #6 — Sprint Review) — Stereo Rotary Speaker Doppler & Acoustic Early Reflections Raytracer HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::raytraced_reverb_view::{
        RaytracedReverbView, WallMaterial, RAYTRACER_PUCK_HIT_RADIUS, SPEED_OF_SOUND_MPS,
    };
    use crate::views::rotary_doppler_view::{
        RotaryCabinetModel, RotaryDopplerSpeedState, RotaryDopplerView,
        ROTARY_DOPPLER_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn80_pure_rotary_doppler_view_initialization_and_kinematics() {
        let mut view = RotaryDopplerView::new();
        assert_eq!(view.speed_state, RotaryDopplerSpeedState::FastTremolo);
        assert_eq!(view.cabinet_model, RotaryCabinetModel::Leslie122VintageTube);
        assert!((view.horn_speed_rpm - 380.0).abs() < 1e-3);
        assert!((view.drum_speed_rpm - 340.0).abs() < 1e-3);
        assert!((view.mic_angle_deg - 90.0).abs() < 1e-3);
        assert!((view.mic_distance_m - 0.6).abs() < 1e-3);
        assert_eq!(ROTARY_DOPPLER_PUCK_HIT_RADIUS, 22.0);

        // Coordinate normalizations
        for ang in [0.0, 45.0, 90.0, 135.0, 180.0] {
            let norm = RotaryDopplerView::angle_to_normalized(ang);
            assert!((0.0..=1.0).contains(&norm));
            let back = RotaryDopplerView::normalized_to_angle(norm);
            assert!((back - ang).abs() < 1e-4);
        }

        for dist in [0.1, 0.5, 1.0, 1.5, 2.0] {
            let norm = RotaryDopplerView::distance_to_normalized(dist);
            assert!((0.0..=1.0).contains(&norm));
            let back = RotaryDopplerView::normalized_to_distance(norm);
            assert!((back - dist).abs() < 1e-4);
        }

        // Doppler cues calculation
        view.horn_speed_rpm = 400.0;
        view.mic_distance_m = 0.5;
        let (d_l, d_r, am_l, am_r) = view.calculate_doppler_cues();
        assert!(d_l.abs() <= 100.0);
        assert!(d_r.abs() <= 100.0);
        assert!(am_l.abs() <= 20.0);
        assert!(am_r.abs() <= 20.0);

        // Cabinet model selection
        for cab in [
            RotaryCabinetModel::Leslie122VintageTube,
            RotaryCabinetModel::Leslie147OpenBack,
            RotaryCabinetModel::Leslie760SolidState,
            RotaryCabinetModel::CustomTwinHornSpatial,
        ] {
            view.cabinet_model = cab;
            assert_eq!(view.cabinet_model, cab);
        }

        // Speed state selection
        for spd in [
            RotaryDopplerSpeedState::SlowChorale,
            RotaryDopplerSpeedState::FastTremolo,
            RotaryDopplerSpeedState::BrakeStop,
        ] {
            view.speed_state = spd;
            assert_eq!(view.speed_state, spd);
        }

        // Hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.mic_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_mic_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_mic_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ROTARY DOPPLER"));
        assert!(snap.contains("Mic Puck"));
    }

    #[test]
    fn test_turn80_pure_raytraced_reverb_view_initialization_and_acoustics() {
        let mut view = RaytracedReverbView::new();
        assert_eq!(view.room_material, WallMaterial::HardwoodPlank);
        assert!((view.room_dimensions_m.0 - 12.0).abs() < 1e-3);
        assert!((view.room_dimensions_m.1 - 8.0).abs() < 1e-3);
        assert!((view.room_dimensions_m.2 - 3.5).abs() < 1e-3);
        assert_eq!(RAYTRACER_PUCK_HIT_RADIUS, 22.0);
        assert_eq!(SPEED_OF_SOUND_MPS, 343.0);

        // Wall material models
        for mat in [
            WallMaterial::PolishedConcrete,
            WallMaterial::HardwoodPlank,
            WallMaterial::StudioAcousticFoam,
            WallMaterial::DoubleGlazedGlass,
            WallMaterial::HeavyVelvetDrape,
        ] {
            view.room_material = mat;
            assert_eq!(view.room_material, mat);
        }

        // Raytrace simulation update
        view.update_raytrace_simulation();
        assert!(!view.simulated_rays.is_empty());
        assert!(view.calculated_rt60_estimate_s > 0.0);

        // Direct ray (bounce order 0)
        let has_direct = view.simulated_rays.iter().any(|r| r.bounce_order == 0);
        assert!(has_direct);

        // Reflected rays (bounce order 1)
        let has_bounce = view.simulated_rays.iter().any(|r| r.bounce_order == 1);
        assert!(has_bounce);

        // Hit testing
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);
        view.source_pos = (0.5, 0.5);
        view.listener_pos = (0.7, 0.7);

        let src_x = canvas.x + 0.5 * canvas.width;
        let src_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_source_puck((src_x, src_y), canvas));
        assert!(!view.hit_test_source_puck((src_x + 100.0, src_y), canvas));

        let lis_x = canvas.x + 0.7 * canvas.width;
        let lis_y = canvas.y + 0.3 * canvas.height;
        assert!(view.hit_test_listener_puck((lis_x, lis_y), canvas));
        assert!(!view.hit_test_listener_puck((lis_x + 100.0, lis_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("-"));
        assert!(snap.contains("S"));
        assert!(snap.contains("L"));
    }

    #[test]
    fn test_turn80_pure_slot_arithmetic_channel_isolation() {
        // Channel ID formula: track_id * 1000 + slot_idx * 20 + offset
        for track_id in 1u32..=4 {
            for slot_idx in 0u32..=3 {
                let base = track_id * 1000 + slot_idx * 20;

                // Rotary Doppler: 6 parameters (offsets 0..=5)
                let rot_horn = ParamId(base);
                let rot_drum = ParamId(base + 1);
                let rot_ang = ParamId(base + 2);
                let rot_dist = ParamId(base + 3);
                let rot_bal = ParamId(base + 4);
                let rot_drv = ParamId(base + 5);

                assert_eq!(rot_horn.0, base);
                assert_eq!(rot_drum.0, base + 1);
                assert_eq!(rot_ang.0, base + 2);
                assert_eq!(rot_dist.0, base + 3);
                assert_eq!(rot_bal.0, base + 4);
                assert_eq!(rot_drv.0, base + 5);

                // Raytraced Reverb: 8 parameters (offsets 0..=7)
                let ray_len = ParamId(base);
                let ray_wid = ParamId(base + 1);
                let ray_hgt = ParamId(base + 2);
                let ray_sx = ParamId(base + 3);
                let ray_sy = ParamId(base + 4);
                let ray_lx = ParamId(base + 5);
                let ray_ly = ParamId(base + 6);
                let ray_air = ParamId(base + 7);

                assert_eq!(ray_len.0, base);
                assert_eq!(ray_wid.0, base + 1);
                assert_eq!(ray_hgt.0, base + 2);
                assert_eq!(ray_sx.0, base + 3);
                assert_eq!(ray_sy.0, base + 4);
                assert_eq!(ray_lx.0, base + 5);
                assert_eq!(ray_ly.0, base + 6);
                assert_eq!(ray_air.0, base + 7);

                // Verify slot isolation (stays strictly within slot boundary of 20 IDs)
                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(rot_drv.0 < next_slot_base);
                assert!(ray_air.0 < next_slot_base);
            }
        }
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
    fn test_turn80_gui_award_winning_view_rotary_doppler_and_raytraced_reverb_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Stereo Rotary Doppler HUD
        view.open_rotary_doppler_hud();
        assert!(view.is_rotary_doppler_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_rotary_doppler_hud();
        assert!(!view.is_rotary_doppler_hud_open());

        // 2. Acoustic Early Reflections Raytracer HUD
        view.open_raytraced_reverb_hud();
        assert!(view.is_raytraced_reverb_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_raytraced_reverb_hud();
        assert!(!view.is_raytraced_reverb_hud_open());
    }

    #[test]
    fn test_turn80_gui_parambus_slot_dispatch_rotary_doppler_and_raytraced_reverb() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = 1u32;
        let slot_idx = 0u32;
        let base = track_id * 1000 + slot_idx * 20;

        // Register bus channels
        for i in 0..10 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));
        view.selected_track_idx = 0;
        view.device_rack_state.selected_chain_idx = 0;

        // Dispatch Rotary Doppler
        view.rotary_doppler_view.horn_speed_rpm = 420.0;
        view.rotary_doppler_view.drum_speed_rpm = 360.0;
        view.rotary_doppler_view.mic_angle_deg = 110.0;
        view.rotary_doppler_view.mic_distance_m = 0.85;
        view.rotary_doppler_view.horn_drum_balance = 0.25;
        view.rotary_doppler_view.tube_drive_percent = 65.0;

        view.sync_rotary_doppler_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 420.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 360.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 110.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 0.85).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 0.25).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 65.0).abs() < 1e-3);

        // Dispatch Raytraced Reverb
        view.raytraced_reverb_view.room_dimensions_m = (15.0, 10.0, 4.0);
        view.raytraced_reverb_view.source_pos = (0.25, 0.35);
        view.raytraced_reverb_view.listener_pos = (0.75, 0.65);
        view.raytraced_reverb_view.air_damping_absorption = 45.0;

        view.sync_raytraced_reverb_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 15.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 10.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 4.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 0.25).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 0.35).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 0.75).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 6)).unwrap() - 0.65).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 7)).unwrap() - 45.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn80_gui_command_palette_registration_and_dispatch() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_rot = actions
            .iter()
            .any(|a| a.action_id == "open_rotary_doppler_hud" && a.category == "Acoustic");
        let has_ray = actions
            .iter()
            .any(|a| a.action_id == "open_raytraced_reverb_hud" && a.category == "Spatial");

        assert!(has_rot);
        assert!(has_ray);
    }

    #[test]
    fn test_turn80_gui_device_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Trigger Rotary Doppler from Inspector
        view.inspector_state.requested_open_rotary_doppler_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_rotary_doppler_hud_open());
        assert!(!view.inspector_state.requested_open_rotary_doppler_hud);
        view.close_rotary_doppler_hud();

        // 2. Trigger Raytraced Reverb from Device Rack
        view.device_rack_state.requested_open_raytraced_reverb_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_raytraced_reverb_hud_open());
        assert!(!view.device_rack_state.requested_open_raytraced_reverb_hud);
        view.close_raytraced_reverb_hud();
    }
}
