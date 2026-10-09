#![allow(clippy::all)]

//! Test suite for Turn #113 (Sprint Turn #39 — Sprint Review) — Physical Modeling
//! Wooden Cassotto Tone Chamber & Resonant Soundbox HUD and Zero-Gravity Navier-Stokes
//! Acoustic Fluid Dynamics & Molecular Resonator HUD Convergence (Milestones 28, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::cassotto_chamber_view::{
        CassottoChamberView, CassottoChamberProfile, CASSOTTO_PUCK_HIT_RADIUS,
    };
    use crate::views::zero_gravity_fluid_view::{
        ZeroGravityFluidView, FluidAcousticRegime, FLUID_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn113_pure_cassotto_chamber_initialization_and_kinematics() {
        let mut view = CassottoChamberView::new();
        assert_eq!(view.profile, CassottoChamberProfile::DobleCassottoBandoneon);
        assert_eq!(CASSOTTO_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profiles and expected values
        let profiles = [
            (CassottoChamberProfile::DobleCassottoBandoneon, "Doble Cassotto Bandoneon (Premier Chamber)", 460.0, 1420.0, 2600.0, 3.8, 0.85),
            (CassottoChamberProfile::StandardAccordionChamber, "Standard Double Cassotto (Italian Maple)", 520.0, 1680.0, 3200.0, 3.2, 0.75),
            (CassottoChamberProfile::ConcertinaSpruceCavity, "Vintage Concertina Spruce Body", 680.0, 2100.0, 4800.0, 2.4, 0.50),
            (CassottoChamberProfile::HarmoniumResonantBox, "Harmonium Resonant Soundbox", 340.0, 1150.0, 2100.0, 4.5, 0.95),
            (CassottoChamberProfile::OpenAirBayan, "Open-Air Russian Bayan (Direct)", 850.0, 2800.0, 8000.0, 1.5, 0.25),
        ];

        for (p, expected_name, exp_f1, exp_f2, exp_fc, exp_q, exp_warmth) in profiles {
            view.set_profile(p);
            assert_eq!(p.name(), expected_name);
            let (f1, f2, fc, q, w) = p.nominal_acoustics();
            assert_eq!(f1, exp_f1);
            assert_eq!(f2, exp_f2);
            assert_eq!(fc, exp_fc);
            assert_eq!(q, exp_q);
            assert_eq!(w, exp_warmth);
            assert_eq!(view.mode1_freq_hz, exp_f1);
            assert_eq!(view.mode2_freq_hz, exp_f2);
            assert_eq!(view.lowpass_cutoff_hz, exp_fc);
            assert_eq!(view.cavity_q, exp_q);
            assert_eq!(view.wood_warmth, exp_warmth);
        }

        // Normalization round-trips for Aperture [0.0 ..= 1.0]
        let ap_test = 0.65;
        let ap_norm = CassottoChamberView::aperture_to_normalized(ap_test);
        let ap_back = CassottoChamberView::normalized_to_aperture(ap_norm);
        assert!((ap_back - ap_test).abs() < 1e-4);

        // Normalization round-trips for Wood Warmth [0.0 ..= 2.0]
        let warmth_test = 1.45;
        let warmth_norm = CassottoChamberView::warmth_to_normalized(warmth_test);
        let warmth_back = CassottoChamberView::normalized_to_warmth(warmth_norm);
        assert!((warmth_back - warmth_test).abs() < 1e-4);

        // Frequency response evaluation across acoustic spectrum
        let resp_sub = view.evaluate_frequency_response(60.0);
        let resp_mid = view.evaluate_frequency_response(600.0);
        let resp_high = view.evaluate_frequency_response(10000.0);
        assert!(resp_sub >= -36.0 && resp_sub <= 18.0);
        assert!(resp_mid >= -36.0 && resp_mid <= 18.0);
        assert!(resp_high >= -36.0 && resp_high <= 18.0);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_chamber_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_chamber_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_chamber_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_chamber_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PHYSICAL MODELING WOODEN CASSOTTO TONE CHAMBER"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn113_pure_zero_gravity_fluid_initialization_and_kinematics() {
        let mut view = ZeroGravityFluidView::new();
        assert_eq!(view.regime, FluidAcousticRegime::MicrogravityWaterSphere);
        assert_eq!(FLUID_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Regimes and expected values
        let regimes = [
            (FluidAcousticRegime::MicrogravityWaterSphere, "Microgravity Water Sphere (Capillary Wave)", 1.0, 1482.0, 72.8, 0.995),
            (FluidAcousticRegime::SupercriticalCo2Vortex, "Supercritical CO2 Vortex Breakdown", 0.05, 330.0, 18.5, 0.990),
            (FluidAcousticRegime::LiquidHeliumSuperfluid, "Liquid Helium-4 Quantum Superfluid", 0.001, 220.0, 0.35, 0.999),
            (FluidAcousticRegime::ViscousGlycerinDroplet, "Viscous Glycerin Acoustic Droplet", 50.0, 1920.0, 63.4, 0.950),
            (FluidAcousticRegime::FerrofluidMagneticSuspension, "Ferrofluid Magnetic Core Suspension", 25.0, 1250.0, 45.0, 0.980),
        ];

        for (r, expected_name, exp_visc, exp_c, exp_tension, exp_damp) in regimes {
            view.set_regime(r);
            assert_eq!(r.name(), expected_name);
            let (visc, c, tension, damp) = r.nominal_parameters();
            assert_eq!(visc, exp_visc);
            assert_eq!(c, exp_c);
            assert_eq!(tension, exp_tension);
            assert_eq!(damp, exp_damp);
            assert_eq!(view.viscosity_cst, exp_visc);
            assert_eq!(view.sound_speed_mps, exp_c);
            assert_eq!(view.surface_tension_mnm, exp_tension);
            assert_eq!(view.pressure_damping, exp_damp);
        }

        // Normalization round-trips for Viscosity [0.001 ..= 50.0 cSt] (log-scale)
        let visc_test = 2.5;
        let visc_norm = ZeroGravityFluidView::viscosity_to_normalized(visc_test);
        let visc_back = ZeroGravityFluidView::normalized_to_viscosity(visc_norm);
        assert!((visc_back - visc_test).abs() < 1e-2);

        // Normalization round-trips for Sound Speed [100.0 ..= 2000.0 m/s]
        let c_test = 1250.0;
        let c_norm = ZeroGravityFluidView::sound_speed_to_normalized(c_test);
        let c_back = ZeroGravityFluidView::normalized_to_sound_speed(c_norm);
        assert!((c_back - c_test).abs() < 1e-3);

        // Capillary frequency and 4x4 pressure field simulation
        view.droplet_radius_mm = 40.0;
        view.surface_tension_mnm = 70.0;
        view.sound_speed_mps = 1500.0;
        view.update_simulation();
        assert!(view.capillary_freq_hz >= 0.5 && view.capillary_freq_hz <= 2000.0);
        for &p in &view.pressure_grid {
            assert!(p >= -1.0 && p <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_fluid_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_fluid_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_fluid_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_fluid_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ZERO-GRAVITY ACOUSTIC FLUID DYNAMICS"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn113_gui_cassotto_and_fluid_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Cassotto Chamber Modal Lifecycle
        assert!(!view.is_cassotto_chamber_hud_open());
        view.open_cassotto_chamber_hud();
        assert!(view.is_cassotto_chamber_hud_open());
        view.close_cassotto_chamber_hud();
        assert!(!view.is_cassotto_chamber_hud_open());

        // 2. Zero-Gravity Fluid Modal Lifecycle
        assert!(!view.is_zero_gravity_fluid_hud_open());
        view.open_zero_gravity_fluid_hud();
        assert!(view.is_zero_gravity_fluid_hud_open());
        view.close_zero_gravity_fluid_hud();
        assert!(!view.is_zero_gravity_fluid_hud_open());

        // 3. Cassotto Chamber Parameter Sync Bridge
        view.cassotto_chamber_view.aperture = 0.72;
        view.cassotto_chamber_view.wood_warmth = 1.25;
        view.cassotto_chamber_view.lowpass_cutoff_hz = 3100.0;
        view.cassotto_chamber_view.cavity_q = 3.6;
        view.sync_cassotto_chamber_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("aperture").copied(),
            Some(0.72)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("wood_warmth").copied(),
            Some(1.25)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("aperture").copied(),
            Some(0.72)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("wood_warmth").copied(),
            Some(1.25)
        );

        // 4. Zero-Gravity Fluid Parameter Sync Bridge
        view.zero_gravity_fluid_view.viscosity_cst = 2.45;
        view.zero_gravity_fluid_view.sound_speed_mps = 1620.0;
        view.zero_gravity_fluid_view.surface_tension_mnm = 58.0;
        view.zero_gravity_fluid_view.pressure_damping = 0.992;
        view.sync_zero_gravity_fluid_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("viscosity_cst").copied(),
            Some(2.45)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("sound_speed_mps").copied(),
            Some(1620.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("viscosity_cst").copied(),
            Some(2.45)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("sound_speed_mps").copied(),
            Some(1620.0)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_cassotto_chamber_modal_window(ui);
                view.show_zero_gravity_fluid_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let cas_action = palette.actions.iter().find(|a| a.action_id == "open_cassotto_chamber_hud");
        assert!(cas_action.is_some(), "open_cassotto_chamber_hud should be registered");
        assert_eq!(cas_action.unwrap().category, "Physical Modeling");

        let fluid_action = palette.actions.iter().find(|a| a.action_id == "open_zero_gravity_fluid_hud");
        assert!(fluid_action.is_some(), "open_zero_gravity_fluid_hud should be registered");
        assert_eq!(fluid_action.unwrap().category, "Physical Modeling");
    }
}
