#![allow(clippy::all)]

//! Test suite for Turn #63 (Turn #28 Deliverable) — Granular Synthesis Cloud Dispersion & Spring-Mass Lattice Deformation HUDs Convergence (Milestones 12, 20 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::granular_cloud_view::{
        GranularCloudView, GrainWindowShape, EMITTER_PUCK_HIT_RADIUS,
    };
    use crate::views::spring_lattice_view::{
        SpringLatticeView, SpringLatticeViewProfile, SPRING_PUCK_HIT_RADIUS,
        LATTICE_VIEW_GRID_DIM, MIN_DRIVE_FORCE, MAX_DRIVE_FORCE,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn63_pure_granular_cloud_view_initialization_and_puck_coordinates() {
        let mut view = GranularCloudView::new();
        assert!((view.emitter_pos_norm - 0.45).abs() < 1e-3);
        assert!((view.emitter_pitch_semitones - 0.0).abs() < 1e-3);
        assert!((view.spray_width_norm - 0.15).abs() < 1e-3);
        assert!((view.spray_height_semitones - 3.5).abs() < 1e-3);
        assert!((view.grain_rate_hz - 35.0).abs() < 1e-3);
        assert!((view.grain_size_ms - 80.0).abs() < 1e-3);
        assert_eq!(view.density, 16);
        assert_eq!(view.window_shape, GrainWindowShape::Hanning);
        assert_eq!(view.active_grains.len(), 16);

        // Coordinate transforms
        let canvas = Rect::new(50.0, 100.0, 400.0, 300.0);
        let (screen_x, screen_y) = view.cloud_coords_to_screen(0.5, 0.0, canvas);
        assert!((screen_x - 250.0).abs() < 1e-2);
        // Pitch 0.0 is center -> norm = 24/48 = 0.5 -> sy = 100 + (1 - 0.5)*300 = 250
        assert!((screen_y - 250.0).abs() < 1e-2);

        let (recon_pos, recon_pitch) = view.screen_to_cloud_coords((screen_x, screen_y), canvas);
        assert!((recon_pos - 0.5).abs() < 1e-3);
        assert!(recon_pitch.abs() < 1e-3);

        // Hit testing emitter puck
        let (puck_x, puck_y) = view.cloud_coords_to_screen(
            view.emitter_pos_norm,
            view.emitter_pitch_semitones,
            canvas,
        );
        assert!(view.hit_test_emitter((puck_x, puck_y), canvas));
        assert!(view.hit_test_emitter((puck_x + EMITTER_PUCK_HIT_RADIUS * 0.8, puck_y), canvas));
        assert!(!view.hit_test_emitter((puck_x + EMITTER_PUCK_HIT_RADIUS * 1.5, puck_y), canvas));

        // Stepping grains updates ages and amplitudes
        let prev_age = view.active_grains[0].age_ms;
        view.step_grains(20.0);
        assert_ne!(view.active_grains[0].age_ms, prev_age);
        assert!(view.active_grains[0].amplitude >= 0.0);
    }

    #[test]
    fn test_turn63_pure_granular_window_shapes_evaluation() {
        let windows = [
            (GrainWindowShape::Hanning, "Hanning (Smooth)"),
            (GrainWindowShape::Blackman, "Blackman (Clean)"),
            (GrainWindowShape::Gaussian, "Gaussian (Warm)"),
            (GrainWindowShape::Trapezoid, "Trapezoid (Punch)"),
            (GrainWindowShape::ExponentialDecay, "Exp Decay (Percussive)"),
        ];

        for (w, expected_name) in windows {
            assert_eq!(w.display_name(), expected_name);
            for p_step in 0..=10 {
                let phase = p_step as f32 / 10.0;
                let val = w.evaluate(phase);
                assert!(val.is_finite(), "Window {:?} produced non-finite value at phase {}", w, phase);
                assert!(val >= -0.05 && val <= 1.05, "Window {:?} out of bounds: {} at phase {}", w, val, phase);
            }
        }

        // Specific boundary tests
        assert!(GrainWindowShape::Hanning.evaluate(0.0) < 1e-4);
        assert!((GrainWindowShape::Hanning.evaluate(0.5) - 1.0).abs() < 1e-4);
        assert!(GrainWindowShape::Hanning.evaluate(1.0) < 1e-4);

        assert!((GrainWindowShape::ExponentialDecay.evaluate(0.0) - 1.0).abs() < 1e-4);
        assert!(GrainWindowShape::ExponentialDecay.evaluate(1.0) < 0.01);
    }

    #[test]
    fn test_turn63_pure_spring_lattice_view_profiles_and_duffing_physics() {
        let mut view = SpringLatticeView::new();
        assert_eq!(view.profile, SpringLatticeViewProfile::VintageAccutronicsSpring);
        assert!((view.drive_force - 0.80).abs() < 1e-2);
        assert!((view.spring_nonlinearity - 0.45).abs() < 1e-2);
        assert!((view.linear_stiffness - 2800.0).abs() < 1e-1);

        // Test profiles nominal physics
        let profiles = [
            (SpringLatticeViewProfile::VintageAccutronicsSpring, "ACCUTRONICS SPRING TANK", 2800.0),
            (SpringLatticeViewProfile::StudioPlateSuspension, "STUDIO SUSPENSION PLATE", 5500.0),
            (SpringLatticeViewProfile::DualHelicalSpringTank, "DUAL HELICAL SPRING TANK", 3200.0),
            (SpringLatticeViewProfile::ResonantHelicalCoil, "RESONANT HELICAL COIL", 4200.0),
            (SpringLatticeViewProfile::NonlinearShakerTable, "NONLINEAR SHAKER TABLE", 1800.0),
            (SpringLatticeViewProfile::MultiAxisSpringLattice, "MULTI-AXIS SPRING LATTICE", 3800.0),
        ];

        for (prof, name, expected_k) in profiles {
            assert_eq!(prof.name(), name);
            let (k, _beta, _damping, tension) = prof.nominal_physics();
            assert!((k - expected_k).abs() < 1e-1);
            assert!(tension > 0.0 && tension <= 1.0);

            view.profile = prof;
            view.update_physics();
            assert_eq!(view.linear_stiffness, k);
            assert_eq!(view.boundary_tension, tension);
            assert!(view.peak_displacement_mm > 0.0);
        }

        // Normalization round-trips
        let norm_drive = SpringLatticeView::drive_to_normalized(0.60);
        let recon_drive = SpringLatticeView::normalized_to_drive(norm_drive);
        assert!((recon_drive - 0.60).abs() < 1e-3);

        let norm_nonlin = SpringLatticeView::nonlinearity_to_normalized(0.72);
        let recon_nonlin = SpringLatticeView::normalized_to_nonlinearity(norm_nonlin);
        assert!((recon_nonlin - 0.72).abs() < 1e-3);

        // Clamping bounds
        assert_eq!(SpringLatticeView::normalized_to_drive(0.0), MIN_DRIVE_FORCE);
        assert_eq!(SpringLatticeView::normalized_to_drive(1.0), MAX_DRIVE_FORCE);
    }

    #[test]
    fn test_turn63_pure_spring_lattice_node_displacement_and_hit_test() {
        let view = SpringLatticeView::new();
        assert_eq!(LATTICE_VIEW_GRID_DIM, 6);

        for gy in 0..LATTICE_VIEW_GRID_DIM {
            for gx in 0..LATTICE_VIEW_GRID_DIM {
                let disp = view.evaluate_node_displacement(gx, gy);
                assert!(disp.is_finite(), "Displacement at ({}, {}) was non-finite", gx, gy);
                assert!(disp >= -1.5 && disp <= 1.5, "Displacement at ({}, {}) out of bounds: {}", gx, gy, disp);
            }
        }

        // Puck hit test
        let canvas = Rect::new(20.0, 30.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_spring_puck((px, py), canvas));
        assert!(view.hit_test_spring_puck((px + SPRING_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_spring_puck((px + SPRING_PUCK_HIT_RADIUS * 1.5, py), canvas));
    }

    #[test]
    fn test_turn63_pure_granular_and_spring_lattice_parambus_slot_arithmetic() {
        // Track 1, Slot 0 vs Slot 1
        let t1_s0_base = 1 * 1000 + 0 * 20; // 1000
        let t1_s1_base = 1 * 1000 + 1 * 20; // 1020
        assert_eq!(t1_s0_base, 1000);
        assert_eq!(t1_s1_base, 1020);

        // Granular offsets: 0..=6 -> 1000..=1006
        let granular_last = ParamId((t1_s0_base + 6) as u32);
        assert!(granular_last.0 < t1_s1_base as u32);

        // Spring lattice offsets: 0..=5 -> 1000..=1005
        let spring_last = ParamId((t1_s0_base + 5) as u32);
        assert!(spring_last.0 < t1_s1_base as u32);

        // Track 2 isolation
        let t2_s0_base = 2 * 1000 + 0 * 20; // 2000
        assert!(t2_s0_base > (t1_s0_base + 19 * 20));
    }

    #[test]
    fn test_turn63_pure_deterministic_ascii_snapshots() {
        // Granular Cloud View ASCII snapshot
        let g_view = GranularCloudView::new();
        let g_snap = g_view.render_ascii_snapshot(64, 16);
        assert_eq!(g_snap.len(), 16);
        let g_str = g_view.render_ascii_snapshot_str();
        assert!(!g_str.is_empty());
        assert!(g_str.contains('E'), "Expected emitter marker 'E' in granular ascii snapshot");
        assert!(g_str.contains('*'), "Expected grain markers '*' in granular ascii snapshot");

        // Spring Lattice View ASCII snapshot
        let s_view = SpringLatticeView::new();
        let s_snap = s_view.render_ascii_snapshot(64, 16);
        assert_eq!(s_snap.len(), 16);
        let s_str = s_view.render_ascii_snapshot_str();
        assert!(!s_str.is_empty());
        assert!(s_str.contains('+') || s_str.contains('#') || s_str.contains('.'), "Expected wireframe grid points in spring lattice snapshot");
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use std::sync::Arc;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn63_gui_granular_cloud_and_spring_lattice_modal_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        // Granular Cloud HUD Lifecycle
        assert!(!view.is_granular_cloud_hud_open());
        view.open_granular_cloud_hud();
        assert!(view.is_granular_cloud_hud_open());
        view.close_granular_cloud_hud();
        assert!(!view.is_granular_cloud_hud_open());

        // Spring-Mass Lattice HUD Lifecycle
        assert!(!view.is_spring_lattice_hud_open());
        view.open_spring_lattice_hud();
        assert!(view.is_spring_lattice_hud_open());
        view.close_spring_lattice_hud();
        assert!(!view.is_spring_lattice_hud_open());
    }

    #[test]
    fn test_turn63_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_granular_cloud_hud();
        view.open_spring_lattice_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_granular_cloud_hud_open());
        assert!(view.is_spring_lattice_hud_open());
    }

    #[test]
    fn test_turn63_gui_live_param_bus_atomic_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..7 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Granular Cloud HUD sync
        view.granular_cloud_view.emitter_pos_norm = 0.62;
        view.granular_cloud_view.emitter_pitch_semitones = 7.0;
        view.granular_cloud_view.spray_width_norm = 0.28;
        view.granular_cloud_view.spray_height_semitones = 5.5;
        view.granular_cloud_view.grain_rate_hz = 48.0;
        view.granular_cloud_view.grain_size_ms = 120.0;
        view.granular_cloud_view.density = 24;
        view.sync_granular_cloud_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.62));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(7.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.28));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(5.5));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(48.0));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(120.0));
        assert_eq!(arc_bus.get(ParamId(base + 6)), Some(24.0));

        assert_eq!(view.device_rack_state.node_param_values.get("emitter_pos"), Some(&0.62));
        assert_eq!(view.inspector_state.node_param_values.get("emitter_pos"), Some(&0.62));

        // Test Spring-Mass Lattice HUD sync
        view.spring_lattice_view.drive_force = 0.92;
        view.spring_lattice_view.spring_nonlinearity = 0.65;
        view.spring_lattice_view.boundary_tension = 0.88;
        view.spring_lattice_view.linear_stiffness = 3400.0;
        view.spring_lattice_view.fundamental_hz = 112.5;
        view.spring_lattice_view.peak_displacement_mm = 4.2;
        view.sync_spring_lattice_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.92));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.88));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(3400.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(112.5));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(4.2));

        assert_eq!(view.device_rack_state.node_param_values.get("drive_force"), Some(&0.92));
        assert_eq!(view.inspector_state.node_param_values.get("drive_force"), Some(&0.92));
    }

    #[test]
    fn test_turn63_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // 1. Trigger Granular Cloud from Inspector
        view.inspector_state.requested_open_granular_cloud_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_granular_cloud_hud);
        assert!(view.is_granular_cloud_hud_open());
        view.close_granular_cloud_hud();

        // 2. Trigger Spring Lattice from Inspector
        view.inspector_state.requested_open_spring_lattice_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_spring_lattice_hud);
        assert!(view.is_spring_lattice_hud_open());
        view.close_spring_lattice_hud();

        // 3. Trigger Granular Cloud from Device Rack
        view.device_rack_state.requested_open_granular_cloud_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_granular_cloud_hud);
        assert!(view.is_granular_cloud_hud_open());
        view.close_granular_cloud_hud();

        // 4. Trigger Spring Lattice from Device Rack
        view.device_rack_state.requested_open_spring_lattice_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_spring_lattice_hud);
        assert!(view.is_spring_lattice_hud_open());
    }

    #[test]
    fn test_turn63_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let has_granular = palette.actions.iter().any(|a| a.action_id == "open_granular_cloud_hud");
        let has_spring_lattice = palette.actions.iter().any(|a| a.action_id == "open_spring_lattice_hud");

        assert!(has_granular, "Expected open_granular_cloud_hud in command palette");
        assert!(has_spring_lattice, "Expected open_spring_lattice_hud in command palette");
    }
}
