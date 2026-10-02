#![allow(clippy::all)]

//! Test suite for Turn #68 / Turn #31 deliverable — Mechanical Plate Dispersion & Membrane Cavity Drum Displacement HUDs Convergence (Milestones 19, 20 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::plate_dispersion_view::{
        PlateDispersionView, PlateReverbViewProfile, PLATE_PUCK_HIT_RADIUS,
        NUM_DISPERSION_CURVE_POINTS,
    };
    use crate::views::membrane_cavity_view::{
        MembraneCavityView, MembraneInstrumentViewProfile, MEMBRANE_PUCK_HIT_RADIUS,
        MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn68_pure_plate_dispersion_view_initialization_and_profiles() {
        let mut view = PlateDispersionView::new();
        assert_eq!(view.profile, PlateReverbViewProfile::VintageEmt140Steel);
        assert!((view.dispersion_factor - 0.65).abs() < 1e-3);
        assert!((view.damper_position - 0.30).abs() < 1e-3);
        assert!((view.decay_t60_sec - 2.60).abs() < 0.2);
        assert_eq!(PLATE_PUCK_HIT_RADIUS, 22.0);

        // Test profiles
        view.set_profile(PlateReverbViewProfile::StudioPlateSuspension);
        assert_eq!(view.profile, PlateReverbViewProfile::StudioPlateSuspension);
        assert!((view.high_damping - 0.25).abs() < 1e-3);

        view.set_profile(PlateReverbViewProfile::GoldFoilPlate);
        assert_eq!(view.profile, PlateReverbViewProfile::GoldFoilPlate);
        assert!((view.high_damping - 0.60).abs() < 1e-3);

        view.set_profile(PlateReverbViewProfile::CompactMechanicalTank);
        assert_eq!(view.profile, PlateReverbViewProfile::CompactMechanicalTank);
        assert!((view.high_damping - 0.35).abs() < 1e-3);

        view.set_profile(PlateReverbViewProfile::HighTensionResonator);
        assert_eq!(view.profile, PlateReverbViewProfile::HighTensionResonator);
        assert!((view.high_damping - 0.18).abs() < 1e-3);

        // Normalized conversions
        let norm_d = PlateDispersionView::dispersion_to_normalized(view.dispersion_factor);
        let back_d = PlateDispersionView::normalized_to_dispersion(norm_d);
        assert!((back_d - view.dispersion_factor).abs() < 1e-3);

        let norm_p = PlateDispersionView::damper_to_normalized(view.damper_position);
        let back_p = PlateDispersionView::normalized_to_damper(norm_p);
        assert!((back_p - view.damper_position).abs() < 1e-3);

        // Dispersion curve evaluation
        let curve = view.evaluate_dispersion_curve();
        assert_eq!(curve.len(), NUM_DISPERSION_CURVE_POINTS);
        assert!(curve[0] > 0.0);
        assert!(curve[NUM_DISPERSION_CURVE_POINTS - 1] <= 1.0);
        assert!(curve[NUM_DISPERSION_CURVE_POINTS - 1] >= curve[0]);

        // Hit testing
        let canvas = Rect { x: 30.0, y: 60.0, width: 240.0, height: 180.0 };
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_plate_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_plate_puck((puck_x + 10.0, puck_y - 10.0), canvas));
        assert!(!view.hit_test_plate_puck((puck_x + 50.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("PLATE DISPERSION"));
    }

    #[test]
    fn test_turn68_pure_membrane_cavity_view_initialization_and_instruments() {
        let mut view = MembraneCavityView::new();
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::TimpaniKettle);
        assert!((view.radial_strike_pos - 0.70).abs() < 1e-3);
        assert!((view.strike_velocity - 0.80).abs() < 1e-3);
        assert!((view.air_cavity_depth - 0.85).abs() < 1e-3);
        assert!((view.fundamental_hz - 146.83).abs() < 1e-2);
        assert_eq!(MEMBRANE_PUCK_HIT_RADIUS, 22.0);

        // Test instruments
        view.set_instrument(MembraneInstrumentViewProfile::ConcertBassDrum);
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::ConcertBassDrum);
        assert!((view.air_cavity_depth - 0.95).abs() < 1e-3);

        view.set_instrument(MembraneInstrumentViewProfile::SnareDrum);
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::SnareDrum);
        assert!((view.rimshot_damping - 0.15).abs() < 1e-3);

        view.set_instrument(MembraneInstrumentViewProfile::TomTom);
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::TomTom);
        assert!((view.air_cavity_depth - 0.60).abs() < 1e-3);

        view.set_instrument(MembraneInstrumentViewProfile::BongosCongas);
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::BongosCongas);
        assert!((view.rimshot_damping - 0.35).abs() < 1e-3);

        view.set_instrument(MembraneInstrumentViewProfile::DjembeFramedrum);
        assert_eq!(view.instrument, MembraneInstrumentViewProfile::DjembeFramedrum);
        assert!((view.air_cavity_depth - 0.75).abs() < 1e-3);

        // Normalized conversions
        let norm_r = MembraneCavityView::radius_to_normalized(view.radial_strike_pos);
        let back_r = MembraneCavityView::normalized_to_radius(norm_r);
        assert!((back_r - view.radial_strike_pos).abs() < 1e-3);

        let norm_v = MembraneCavityView::velocity_to_normalized(view.strike_velocity);
        let back_v = MembraneCavityView::normalized_to_velocity(norm_v);
        assert!((back_v - view.strike_velocity).abs() < 1e-3);
        assert!(view.strike_velocity >= MIN_STRIKE_VELOCITY && view.strike_velocity <= MAX_STRIKE_VELOCITY);

        // Bessel membrane displacement evaluation
        let z_center = view.evaluate_membrane_displacement(0.0, 0.0);
        let z_edge = view.evaluate_membrane_displacement(1.0, 0.0);
        assert!(z_center.abs() > 0.0);
        assert!(z_edge.is_finite());

        // Hit testing
        let canvas = Rect { x: 40.0, y: 70.0, width: 200.0, height: 160.0 };
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_membrane_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_membrane_puck((puck_x + 8.0, puck_y - 8.0), canvas));
        assert!(!view.hit_test_membrane_puck((puck_x + 45.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("MEMBRANE HUD"));
    }

    #[test]
    fn test_turn68_pure_parambus_slot_arithmetic_channel_isolation() {
        let track_id = 2u32;
        let slot_idx = 1u32;
        let base_id = track_id * 1000 + slot_idx * 20;

        let pid_disp = ParamId(base_id);
        let pid_damp = ParamId(base_id + 1);
        let pid_t60  = ParamId(base_id + 2);
        let pid_high = ParamId(base_id + 3);

        assert_eq!(pid_disp.0, 2020);
        assert_eq!(pid_damp.0, 2021);
        assert_eq!(pid_t60.0, 2022);
        assert_eq!(pid_high.0, 2023);

        // Verify next slot isolation
        let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
        assert_eq!(next_slot_base, 2040);
        assert!(next_slot_base > pid_high.0);
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
    fn test_turn68_gui_award_winning_plate_dispersion_and_membrane_cavity_hud_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_plate_dispersion_hud_open());
        assert!(!view.is_membrane_cavity_hud_open());

        view.open_plate_dispersion_hud();
        assert!(view.is_plate_dispersion_hud_open());
        view.close_plate_dispersion_hud();
        assert!(!view.is_plate_dispersion_hud_open());

        view.open_membrane_cavity_hud();
        assert!(view.is_membrane_cavity_hud_open());
        view.close_membrane_cavity_hud();
        assert!(!view.is_membrane_cavity_hud_open());
    }

    #[test]
    fn test_turn68_gui_headless_rendering_no_panics() {
        let mut view = AwardWinningGuiView::new();
        view.open_plate_dispersion_hud();
        view.open_membrane_cavity_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_plate_dispersion_hud_open());
        assert!(view.is_membrane_cavity_hud_open());
    }

    #[test]
    fn test_turn68_gui_live_parambus_atomic_sync_plate_and_membrane() {
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

        // 1. Sync Plate Dispersion HUD
        view.plate_dispersion_view.dispersion_factor = 0.85;
        view.plate_dispersion_view.damper_position = 0.45;
        view.plate_dispersion_view.decay_t60_sec = 2.80;
        view.plate_dispersion_view.high_damping = 0.65;
        view.sync_plate_dispersion_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.45));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(2.80));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.65));

        assert_eq!(view.device_rack_state.node_param_values.get("dispersion_factor"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("damper_position"), Some(&0.45));
        assert_eq!(view.inspector_state.node_param_values.get("dispersion_factor"), Some(&0.85));

        // 2. Sync Membrane Cavity HUD
        view.membrane_cavity_view.radial_strike_pos = 0.55;
        view.membrane_cavity_view.strike_velocity = 0.90;
        view.membrane_cavity_view.air_cavity_depth = 0.80;
        view.membrane_cavity_view.rimshot_damping = 0.12;
        view.sync_membrane_cavity_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.55));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.90));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.80));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.12));

        assert_eq!(view.device_rack_state.node_param_values.get("radial_strike_pos"), Some(&0.55));
        assert_eq!(view.device_rack_state.node_param_values.get("strike_velocity"), Some(&0.90));
        assert_eq!(view.inspector_state.node_param_values.get("radial_strike_pos"), Some(&0.55));
    }

    #[test]
    fn test_turn68_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_plate_dispersion_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_plate_dispersion_hud_open());
        assert!(!view.device_rack_state.requested_open_plate_dispersion_hud);

        view.close_plate_dispersion_hud();
        assert!(!view.is_plate_dispersion_hud_open());

        view.inspector_state.requested_open_membrane_cavity_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_membrane_cavity_hud_open());
        assert!(!view.inspector_state.requested_open_membrane_cavity_hud);
    }

    #[test]
    fn test_turn68_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let plate_action = palette.actions.iter().find(|a| a.action_id == "open_plate_dispersion_hud");
        assert!(plate_action.is_some());
        let plate = plate_action.unwrap();
        assert_eq!(plate.category, "Physical Modeling");
        assert!(plate.label.contains("Plate Dispersion"));

        let membrane_action = palette.actions.iter().find(|a| a.action_id == "open_membrane_cavity_hud");
        assert!(membrane_action.is_some());
        let membrane = membrane_action.unwrap();
        assert_eq!(membrane.category, "Physical Modeling");
        assert!(membrane.label.contains("Membrane Cavity"));
    }
}
