#![allow(clippy::all)]

//! Test suite for Turn #115 (Sprint Turn #41) — Physical Modeling Non-Linear Felt Hammer
//! Strike & Sympathetic Resonance Matrix HUD and Analog Console Desk Emulation &
//! Transformer Character Modeler HUD Convergence (Milestones 11, 18, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::hammer_strike_view::{
        ExcitationPreset, HammerStrikeView, HAMMER_PUCK_HIT_RADIUS, NUM_SYMPATHETIC_STRINGS,
    };
    use crate::views::console_emulation_view::{
        ConsoleDeskPreset, ConsoleEmulationView, CONSOLE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn115_pure_hammer_strike_initialization_and_physics() {
        let mut view = HammerStrikeView::new();
        assert_eq!(view.preset, ExcitationPreset::PianoFeltHammer);
        assert_eq!(HAMMER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (ExcitationPreset::PianoFeltHammer, "Concert Grand Piano Felt Hammer", 0.003, 1.2e8, 2.4, 0.35, 0.12, 2.8),
            (ExcitationPreset::HarpsichordQuill, "Harpsichord Quill / Delrin Pick Snap", 0.0005, 5.0e7, 1.8, 0.15, 0.15, 3.5),
            (ExcitationPreset::NylonFingertip, "Classical Nylon Guitar Fleshy Pluck", 0.0015, 2.0e7, 1.5, 0.50, 0.20, 2.0),
            (ExcitationPreset::FretboardSlap, "Fretboard Slap Bass Metallic Impact", 0.008, 3.0e8, 3.2, 0.10, 0.08, 5.0),
        ];

        for (p, exp_name, exp_m, exp_k, exp_p, exp_mu, exp_beta, exp_vel) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (m, k, p_exp, mu, beta, vel) = p.nominal_parameters();
            assert_eq!(m, exp_m);
            assert_eq!(k, exp_k);
            assert_eq!(p_exp, exp_p);
            assert_eq!(mu, exp_mu);
            assert_eq!(beta, exp_beta);
            assert_eq!(vel, exp_vel);
            assert_eq!(view.mass_kg, exp_m);
            assert_eq!(view.stiffness_k, exp_k);
            assert_eq!(view.exponent_p, exp_p);
            assert_eq!(view.hysteresis_loss, exp_mu);
            assert_eq!(view.strike_position, exp_beta);
            assert_eq!(view.strike_velocity, exp_vel);
            assert!(view.peak_force_n > 0.0);
            assert!(view.contact_duration_ms > 0.0);
        }

        // Normalization round-trips for Strike Position [0.05 ..= 0.95]
        let pos_test = 0.25;
        let pos_norm = HammerStrikeView::strike_position_to_normalized(pos_test);
        let pos_back = HammerStrikeView::normalized_to_strike_position(pos_norm);
        assert!((pos_back - pos_test).abs() < 1e-4);

        // Normalization round-trips for Strike Velocity [0.1 ..= 10.0]
        let vel_test = 4.2;
        let vel_norm = HammerStrikeView::strike_velocity_to_normalized(vel_test);
        let vel_back = HammerStrikeView::normalized_to_strike_velocity(vel_norm);
        assert!((vel_back - vel_test).abs() < 1e-3);

        // Force waveform evaluation over contact window
        let f_mid = view.evaluate_contact_force(0.5);
        let f_start = view.evaluate_contact_force(0.0);
        let f_outside = view.evaluate_contact_force(1.2);
        assert!(f_mid > 0.0);
        assert_eq!(f_start, 0.0);
        assert_eq!(f_outside, 0.0);

        // Sympathetic resonance energy profile across 6 strings
        let energies = view.evaluate_sympathetic_energies();
        assert_eq!(energies.len(), NUM_SYMPATHETIC_STRINGS);
        for &e in &energies {
            assert!(e >= 0.0 && e <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_hammer_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_hammer_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_hammer_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_hammer_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PHYSICAL MODELING FELT HAMMER & SYMPATHETIC RESONANCE MATRIX"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn115_pure_console_emulation_initialization_and_coloration() {
        let mut view = ConsoleEmulationView::new();
        assert_eq!(view.preset, ConsoleDeskPreset::NeveVintage8048);
        assert_eq!(CONSOLE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Preset profiles and expected values
        let presets = [
            (ConsoleDeskPreset::NeveVintage8048, "Neve 8048 / 1073 (Vintage Class-A Desk)", 52.0, 1.8, 3.5, 0.65, 0.55),
            (ConsoleDeskPreset::SslGSeries4000, "SSL 4000 E/G (British VCA Punch & Sheen)", 90.0, 1.2, 2.0, 0.40, 0.35),
            (ConsoleDeskPreset::ApiDiscrete1608, "API 1608 / 2520 (Discrete Midrange Punch)", 2500.0, 2.5, 4.5, 0.75, 0.70),
        ];

        for (p, exp_name, exp_fc, exp_bump, exp_drv, exp_sat, exp_thd) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (fc, bump, drv, sat, thd) = p.nominal_parameters();
            assert_eq!(fc, exp_fc);
            assert_eq!(bump, exp_bump);
            assert_eq!(drv, exp_drv);
            assert_eq!(sat, exp_sat);
            assert_eq!(thd, exp_thd);
            assert_eq!(view.transformer_center_hz, exp_fc);
            assert_eq!(view.transformer_bump_db, exp_bump);
            assert_eq!(view.drive_db, exp_drv);
            assert_eq!(view.transformer_saturation, exp_sat);
            assert_eq!(view.thd_coloration, exp_thd);
            assert!(view.thd_percentage > 0.0);
        }

        // Normalization round-trips for Drive [0.0 ..= 12.0 dB]
        let drv_test = 6.4;
        let drv_norm = ConsoleEmulationView::drive_to_normalized(drv_test);
        let drv_back = ConsoleEmulationView::normalized_to_drive(drv_norm);
        assert!((drv_back - drv_test).abs() < 1e-4);

        // Normalization round-trips for Saturation [0.0 ..= 1.0]
        let sat_test = 0.55;
        let sat_norm = ConsoleEmulationView::saturation_to_normalized(sat_test);
        let sat_back = ConsoleEmulationView::normalized_to_saturation(sat_norm);
        assert!((sat_back - sat_test).abs() < 1e-4);

        // Frequency response evaluation across bands
        view.set_preset(ConsoleDeskPreset::NeveVintage8048);
        let bump_mag = view.evaluate_frequency_response_db(52.0);
        assert!(bump_mag > 1.0); // Bump at resonance center

        // Harmonic distortion evaluation
        let harmonics = view.evaluate_harmonic_profile();
        assert_eq!(harmonics[0], 0.0); // Fundamental is 0 dB
        assert!(harmonics[1] < 0.0);   // 2nd harmonic is below fundamental
        assert!(harmonics[2] < 0.0);   // 3rd harmonic is below fundamental

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 400.0, 220.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_console_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_console_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_console_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_console_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("ANALOG MIXING CONSOLE DESK EMULATION & TRANSFORMER CHARACTER"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn115_gui_hammer_and_console_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Hammer Strike Modal Lifecycle
        assert!(!view.is_hammer_strike_hud_open());
        view.open_hammer_strike_hud();
        assert!(view.is_hammer_strike_hud_open());
        view.close_hammer_strike_hud();
        assert!(!view.is_hammer_strike_hud_open());

        // 2. Console Emulation Modal Lifecycle
        assert!(!view.is_console_emulation_hud_open());
        view.open_console_emulation_hud();
        assert!(view.is_console_emulation_hud_open());
        view.close_console_emulation_hud();
        assert!(!view.is_console_emulation_hud_open());

        // 3. Hammer Strike Parameter Sync Bridge
        view.hammer_strike_view.strike_position = 0.18;
        view.hammer_strike_view.strike_velocity = 3.6;
        view.hammer_strike_view.sympathetic_coupling = 0.45;
        view.sync_hammer_strike_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("strike_position").copied(),
            Some(0.18)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("strike_velocity").copied(),
            Some(3.6)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("strike_position").copied(),
            Some(0.18)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("strike_velocity").copied(),
            Some(3.6)
        );

        // 4. Console Emulation Parameter Sync Bridge
        view.console_emulation_view.drive_db = 5.2;
        view.console_emulation_view.transformer_saturation = 0.70;
        view.console_emulation_view.output_trim_db = -1.5;
        view.sync_console_emulation_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("drive_db").copied(),
            Some(5.2)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("transformer_saturation").copied(),
            Some(0.70)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("drive_db").copied(),
            Some(5.2)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("transformer_saturation").copied(),
            Some(0.70)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_hammer_strike_modal_window(ui);
                view.show_console_emulation_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let hammer_action = palette.actions.iter().find(|a| a.action_id == "open_hammer_strike_hud");
        assert!(hammer_action.is_some(), "open_hammer_strike_hud should be registered");
        assert_eq!(hammer_action.unwrap().category, "Physical Modeling");

        let console_action = palette.actions.iter().find(|a| a.action_id == "open_console_emulation_hud");
        assert!(console_action.is_some(), "open_console_emulation_hud should be registered");
        assert_eq!(console_action.unwrap().category, "Dynamics & Saturation");
    }
}
