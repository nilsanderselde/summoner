#![allow(clippy::all)]

//! Test suite for Turn #62 (Turn #27 Deliverable — Sprint Review) — Physical Modeling Vintage Rotary Speaker Horn/Drum Doppler Acceleration & Aeroelastic Free-Reed Phase Portrait HUDs Convergence (Milestones 28, 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::rotary_speaker_view::{
        RotarySpeakerView, RotarySpeedState, ROTARY_HANDLE_HIT_RADIUS,
    };
    use crate::views::free_reed_view::{
        FreeReedView, FreeReedHudPreset, FREE_REED_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_turn62_pure_rotary_speaker_physics_and_doppler() {
        let mut view = RotarySpeakerView::new();
        assert_eq!(view.speed_state, RotarySpeedState::Tremolo);
        assert!((view.horn_rpm - 395.0).abs() < 1e-2);
        assert!((view.drum_rpm - 338.0).abs() < 1e-2);
        assert!((view.horn_drum_balance_pct - 60.0).abs() < 1e-2);
        assert!((view.mic_distance_m - 0.65).abs() < 1e-2);
        assert!((view.mic_spread_deg - 120.0).abs() < 1e-2);
        assert!((view.drive_saturation_db - 6.0).abs() < 1e-2);

        // Test target RPMs for all motor speed states
        view.set_speed(RotarySpeedState::Stop);
        assert_eq!(view.target_horn_rpm(), 0.0);
        assert_eq!(view.target_drum_rpm(), 0.0);

        view.set_speed(RotarySpeedState::Brake);
        assert_eq!(view.target_horn_rpm(), 0.0);
        assert_eq!(view.target_drum_rpm(), 0.0);

        view.set_speed(RotarySpeedState::Chorale);
        assert!((view.target_horn_rpm() - 40.0).abs() < 1e-2);
        assert!((view.target_drum_rpm() - 36.0).abs() < 1e-2);

        view.set_speed(RotarySpeedState::Tremolo);
        assert!((view.target_horn_rpm() - 400.0).abs() < 1e-2);
        assert!((view.target_drum_rpm() - 342.0).abs() < 1e-2);

        // Test Doppler shift calculations
        let shift_0 = view.calculate_horn_doppler_shift(view.horn_angle_rad);
        assert!(shift_0.abs() < 1e-4);

        let shift_pi2 = view.calculate_horn_doppler_shift(view.horn_angle_rad - std::f32::consts::FRAC_PI_2);
        assert!(shift_pi2 > 0.0);

        // Physics step acceleration
        let prev_horn_angle = view.horn_angle_rad;
        view.update_physics(0.05);
        assert_ne!(view.horn_angle_rad, prev_horn_angle);

        // Microphone handle hit testing
        let mic_screen_pos = (150.0, 150.0);
        assert!(view.hit_test_mic_handle((150.0, 150.0), mic_screen_pos));
        assert!(view.hit_test_mic_handle((150.0 + ROTARY_HANDLE_HIT_RADIUS * 0.8, 150.0), mic_screen_pos));
        assert!(!view.hit_test_mic_handle((150.0 + ROTARY_HANDLE_HIT_RADIUS * 1.5, 150.0), mic_screen_pos));

        // Deterministic ASCII snapshot
        let snap_lines = view.render_ascii_snapshot(64, 16);
        assert_eq!(snap_lines.len(), 16);
        assert!(snap_lines[0].contains("ROTARY SPEAKER"));

        let snap_str = view.render_ascii_snapshot_str();
        assert!(!snap_str.is_empty());
        assert!(snap_str.contains("Horn:"));
    }

    #[test]
    fn test_turn62_pure_free_reed_aeroelastic_phase_portrait_and_musette() {
        let mut view = FreeReedView::new();
        assert_eq!(view.preset, FreeReedHudPreset::TangoBandoneonZinc);
        assert!((view.reed_stiffness - 1.25).abs() < 1e-2);
        assert!((view.cassotto_aperture - 0.70).abs() < 1e-2);
        assert!((view.musette_detune_cents - 2.0).abs() < 1e-2);
        assert_eq!(view.rank_energies.len(), 5);

        // Test preset choices
        view.set_preset(FreeReedHudPreset::FrenchMusetteMaple);
        assert_eq!(view.preset, FreeReedHudPreset::FrenchMusetteMaple);
        assert!((view.reed_stiffness - 1.00).abs() < 1e-2);
        assert!((view.cassotto_aperture - 0.85).abs() < 1e-2);
        assert!((view.musette_detune_cents - 18.0).abs() < 1e-2);

        view.set_preset(FreeReedHudPreset::RussianBayanDuralumin);
        assert_eq!(view.preset, FreeReedHudPreset::RussianBayanDuralumin);
        assert!((view.reed_stiffness - 1.40).abs() < 1e-2);
        assert!((view.cassotto_aperture - 1.00).abs() < 1e-2);

        view.set_preset(FreeReedHudPreset::VintageHarmoniumBrass);
        assert_eq!(view.preset, FreeReedHudPreset::VintageHarmoniumBrass);
        assert!((view.reed_stiffness - 0.85).abs() < 1e-2);
        assert!((view.cassotto_aperture - 0.50).abs() < 1e-2);

        view.set_preset(FreeReedHudPreset::EnglishConcertinaSteel);
        assert_eq!(view.preset, FreeReedHudPreset::EnglishConcertinaSteel);
        assert!((view.reed_stiffness - 1.15).abs() < 1e-2);
        assert!((view.cassotto_aperture - 0.90).abs() < 1e-2);

        // Normalization roundtrips
        let norm_stiff = FreeReedView::stiffness_to_normalized(1.5);
        let recon_stiff = FreeReedView::normalized_to_stiffness(norm_stiff);
        assert!((recon_stiff - 1.5).abs() < 1e-3);

        let norm_apert = FreeReedView::aperture_to_normalized(0.65);
        let recon_apert = FreeReedView::normalized_to_aperture(norm_apert);
        assert!((recon_apert - 0.65).abs() < 1e-3);

        // Puck updates
        view.update_physics_from_puck(0.4, 0.6);
        assert!((view.cassotto_aperture - 0.4).abs() < 1e-2);
        assert!((view.puck_pos.0 - 0.4).abs() < 1e-2);

        // Hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_free_reed_puck((px, py), canvas));
        assert!(view.hit_test_free_reed_puck((px + FREE_REED_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_free_reed_puck((px + FREE_REED_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Evaluate limit cycle phase portrait
        let trajectory = view.evaluate_phase_portrait();
        assert_eq!(trajectory.len(), 32);
        for &(x, y) in &trajectory {
            assert!(x.is_finite() && y.is_finite());
        }

        // Deterministic ASCII snapshot
        let snap_lines = view.render_ascii_snapshot(64, 16);
        assert_eq!(snap_lines.len(), 16);
        assert!(snap_lines[0].contains('+'));

        let snap_str = view.render_ascii_snapshot_str();
        assert!(!snap_str.is_empty());
        assert!(snap_str.contains("FREE-REED"));
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
    use crate::views::rotary_speaker_view::RotarySpeedState;

    #[test]
    fn test_turn62_gui_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Rotary Speaker HUD Lifecycle
        assert!(!view.is_rotary_speaker_hud_open());
        view.open_rotary_speaker_hud();
        assert!(view.is_rotary_speaker_hud_open());
        view.close_rotary_speaker_hud();
        assert!(!view.is_rotary_speaker_hud_open());

        // Free Reed HUD Lifecycle
        assert!(!view.is_free_reed_hud_open());
        view.open_free_reed_hud();
        assert!(view.is_free_reed_hud_open());
        view.close_free_reed_hud();
        assert!(!view.is_free_reed_hud_open());
    }

    #[test]
    fn test_turn62_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_rotary_speaker_hud();
        view.open_free_reed_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_rotary_speaker_hud_open());
        assert!(view.is_free_reed_hud_open());
    }

    #[test]
    fn test_turn62_gui_live_param_bus_atomic_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..6 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Rotary Speaker sync
        view.rotary_speaker_view.speed_state = RotarySpeedState::Tremolo;
        view.rotary_speaker_view.horn_rpm = 405.0;
        view.rotary_speaker_view.drum_rpm = 345.0;
        view.rotary_speaker_view.mic_spread_deg = 135.0;
        view.rotary_speaker_view.mic_distance_m = 0.85;
        view.rotary_speaker_view.drive_saturation_db = 8.5;
        view.sync_rotary_speaker_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(2.0)); // Tremolo = 2.0
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(405.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(345.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(135.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(8.5));

        // Test Free Reed sync
        view.free_reed_view.reed_stiffness = 1.65;
        view.free_reed_view.cassotto_aperture = 0.80;
        view.free_reed_view.musette_detune_cents = 15.0;
        view.free_reed_view.rank_energies[0] = 0.95;
        view.free_reed_view.rank_energies[1] = 0.90;
        view.free_reed_view.rank_energies[2] = 0.85;
        view.sync_free_reed_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(1.65));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.80));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(15.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.95));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.90));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.85));
    }

    #[test]
    fn test_turn62_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger from Inspector for Rotary Speaker
        view.inspector_state.requested_open_rotary_speaker_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_rotary_speaker_hud);
        assert!(view.is_rotary_speaker_hud_open());

        // Trigger from Device Rack for Free Reed
        view.device_rack_state.requested_open_free_reed_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_free_reed_hud);
        assert!(view.is_free_reed_hud_open());
    }

    #[test]
    fn test_turn62_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let has_rotary = palette.actions.iter().any(|a| a.action_id == "open_rotary_speaker_hud");
        let has_free_reed = palette.actions.iter().any(|a| a.action_id == "open_free_reed_hud");

        assert!(has_rotary, "Expected open_rotary_speaker_hud in command palette");
        assert!(has_free_reed, "Expected open_free_reed_hud in command palette");
    }
}
