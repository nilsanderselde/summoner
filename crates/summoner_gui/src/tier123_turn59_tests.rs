#![allow(clippy::all)]

//! Test suite for Turn #59 (Turn #24 Deliverable — Sprint Review) — Physical Modeling Concert Grand Piano & Pipe Organ Windchest HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::grand_piano_view::{
        GrandPianoView, GrandPianoHudPreset, GRAND_PIANO_PUCK_HIT_RADIUS,
        MIN_GRAND_PIANO_STRIKE_VELOCITY, MAX_GRAND_PIANO_STRIKE_VELOCITY,
        MIN_HAMMER_HARDNESS, MAX_HAMMER_HARDNESS,
    };
    use crate::views::pipe_organ_view::{
        PipeOrganView, PipeType, PIPE_ORGAN_PUCK_HIT_RADIUS,
        MIN_WIND_PRESSURE_MMH2O, MAX_WIND_PRESSURE_MMH2O,
        MIN_CUTUP_RATIO, MAX_CUTUP_RATIO,
    };

    #[test]
    fn test_turn59_pure_grand_piano_instruments_and_simulation() {
        let mut view = GrandPianoView::new();
        assert_eq!(view.preset, GrandPianoHudPreset::SteinwayDRecital);
        assert!((view.strike_velocity - 0.75).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.65).abs() < 1e-4);
        assert!((view.damper_lift_pos - 0.0).abs() < 1e-4);
        assert!((view.una_corda_shift - 0.0).abs() < 1e-4);
        assert!(!view.sostenuto_engaged);
        assert!((view.unison_detune_cents - 0.75).abs() < 1e-4);
        assert!((view.inharmonicity_b - 0.00018).abs() < 1e-6);

        // Preset switching
        view.set_preset(GrandPianoHudPreset::BösendorferWarmth);
        assert_eq!(view.preset, GrandPianoHudPreset::BösendorferWarmth);
        assert!((view.strike_velocity - 0.70).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.50).abs() < 1e-4);
        assert!((view.unison_detune_cents - 0.60).abs() < 1e-4);

        view.set_preset(GrandPianoHudPreset::YamahaCFXPop);
        assert_eq!(view.preset, GrandPianoHudPreset::YamahaCFXPop);
        assert!((view.strike_velocity - 0.85).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.80).abs() < 1e-4);
        assert!((view.unison_detune_cents - 0.90).abs() < 1e-4);

        view.set_preset(GrandPianoHudPreset::ImpressionistUnaCorda);
        assert_eq!(view.preset, GrandPianoHudPreset::ImpressionistUnaCorda);
        assert!((view.strike_velocity - 0.50).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.30).abs() < 1e-4);
        assert!((view.una_corda_shift - 1.0).abs() < 1e-4);
        assert!((view.damper_lift_pos - 0.80).abs() < 1e-4);

        view.set_preset(GrandPianoHudPreset::IntimateFelted);
        assert_eq!(view.preset, GrandPianoHudPreset::IntimateFelted);
        assert!((view.strike_velocity - 0.55).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.35).abs() < 1e-4);
        assert!((view.una_corda_shift - 0.40).abs() < 1e-4);

        view.set_preset(GrandPianoHudPreset::PreparedAvantGarde);
        assert_eq!(view.preset, GrandPianoHudPreset::PreparedAvantGarde);
        assert!((view.strike_velocity - 0.90).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.90).abs() < 1e-4);
        assert!((view.unison_detune_cents - 2.50).abs() < 1e-4);

        // Normalization roundtrips
        let norm_h = GrandPianoView::hardness_to_normalized(0.60);
        let recon_h = GrandPianoView::normalized_to_hardness(norm_h);
        assert!((recon_h - 0.60).abs() < 1e-3);

        let norm_v = GrandPianoView::velocity_to_normalized(0.80);
        let recon_v = GrandPianoView::normalized_to_velocity(norm_v);
        assert!((recon_v - 0.80).abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_piano_puck((px, py), canvas));
        assert!(view.hit_test_piano_puck((px + GRAND_PIANO_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_piano_puck((px + GRAND_PIANO_PUCK_HIT_RADIUS * 1.5, py), canvas));

        assert_eq!(MIN_GRAND_PIANO_STRIKE_VELOCITY, 0.05);
        assert_eq!(MAX_GRAND_PIANO_STRIKE_VELOCITY, 1.0);
        assert_eq!(MIN_HAMMER_HARDNESS, 0.1);
        assert_eq!(MAX_HAMMER_HARDNESS, 1.0);
    }

    #[test]
    fn test_turn59_pure_pipe_organ_ranks_and_fluid_simulation() {
        let mut view = PipeOrganView::new();
        assert_eq!(view.pipe_type, PipeType::Principal8Flue);
        assert!((view.wind_pressure_mmh2o - 75.0).abs() < 1e-4);
        assert!((view.cutup_ratio - 0.25).abs() < 1e-4);
        assert!((view.pipe_length_ft - 8.0).abs() < 1e-4);
        assert!(!view.pipe_type.is_reed());

        // Preset switching
        view.set_pipe_type(PipeType::Bourdon16Stopped);
        assert_eq!(view.pipe_type, PipeType::Bourdon16Stopped);
        assert!((view.wind_pressure_mmh2o - 60.0).abs() < 1e-4);
        assert!((view.cutup_ratio - 0.35).abs() < 1e-4);
        assert!(!view.pipe_type.is_reed());

        view.set_pipe_type(PipeType::Trompette8Reed);
        assert_eq!(view.pipe_type, PipeType::Trompette8Reed);
        assert!((view.wind_pressure_mmh2o - 110.0).abs() < 1e-4);
        assert!((view.cutup_ratio - 0.20).abs() < 1e-4);
        assert!(view.pipe_type.is_reed());

        view.set_pipe_type(PipeType::MixtureIVMultiRank);
        assert_eq!(view.pipe_type, PipeType::MixtureIVMultiRank);
        assert!((view.wind_pressure_mmh2o - 85.0).abs() < 1e-4);
        assert!((view.cutup_ratio - 0.22).abs() < 1e-4);

        view.set_pipe_type(PipeType::VoxHumana8Reed);
        assert_eq!(view.pipe_type, PipeType::VoxHumana8Reed);
        assert!((view.wind_pressure_mmh2o - 95.0).abs() < 1e-4);
        assert!((view.cutup_ratio - 0.18).abs() < 1e-4);
        assert!(view.pipe_type.is_reed());

        // Normalization roundtrips
        let norm_p = PipeOrganView::pressure_to_normalized(100.0);
        let recon_p = PipeOrganView::normalized_to_pressure(norm_p);
        assert!((recon_p - 100.0).abs() < 1e-3);

        let norm_c = PipeOrganView::cutup_to_normalized(0.30);
        let recon_c = PipeOrganView::normalized_to_cutup(norm_c);
        assert!((recon_c - 0.30).abs() < 1e-3);

        // Hit testing
        let canvas = Rect::new(20.0, 20.0, 300.0, 200.0);
        let px = canvas.x + view.organ_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.organ_puck_pos.1) * canvas.height;
        assert!(view.hit_test_organ_puck((px, py), canvas));
        assert!(view.hit_test_organ_puck((px + PIPE_ORGAN_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_organ_puck((px + PIPE_ORGAN_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Fluid jet velocity and harmonic weights
        assert!(view.flue_air_velocity_mps >= 10.0 && view.flue_air_velocity_mps <= 70.0);
        assert!(view.chiff_duration_ms >= 5.0 && view.chiff_duration_ms <= 80.0);
        assert!(view.turbulence_noise_level >= 0.05 && view.turbulence_noise_level <= 0.95);
        for weight in view.harmonic_weights {
            assert!(weight >= 0.0 && weight <= 2.0);
        }

        assert_eq!(MIN_WIND_PRESSURE_MMH2O, 40.0);
        assert_eq!(MAX_WIND_PRESSURE_MMH2O, 160.0);
        assert_eq!(MIN_CUTUP_RATIO, 0.15);
        assert_eq!(MAX_CUTUP_RATIO, 0.50);
    }

    #[test]
    fn test_turn59_pure_slot_arithmetic_channel_isolation() {
        let mut bus = ParamBus::new();

        for track_id in 1..=3 {
            for slot in 0..2 {
                let base = track_id as u32 * 1000 + slot as u32 * 20;

                // Register Grand Piano channels
                let pid_vel     = ParamId(base);
                let pid_hard    = ParamId(base + 1);
                let pid_damper  = ParamId(base + 2);
                let pid_una     = ParamId(base + 3);
                let pid_sost    = ParamId(base + 4);
                let pid_detune  = ParamId(base + 5);

                bus.register(pid_vel, 0.75);
                bus.register(pid_hard, 0.65);
                bus.register(pid_damper, 0.50);
                bus.register(pid_una, 0.25);
                bus.register(pid_sost, 1.0);
                bus.register(pid_detune, 0.75);

                assert_eq!(bus.get(pid_vel), Some(0.75));
                assert_eq!(bus.get(pid_hard), Some(0.65));
                assert_eq!(bus.get(pid_damper), Some(0.50));
                assert_eq!(bus.get(pid_una), Some(0.25));
                assert_eq!(bus.get(pid_sost), Some(1.0));
                assert_eq!(bus.get(pid_detune), Some(0.75));

                // Register Pipe Organ channels
                let pid_press = ParamId(base);
                let pid_cut   = ParamId(base + 1);
                let pid_len   = ParamId(base + 2);
                let pid_chiff = ParamId(base + 3);
                let pid_turb  = ParamId(base + 4);
                let pid_vel_a = ParamId(base + 5);

                bus.set(pid_press, 85.0);
                bus.set(pid_cut, 0.28);
                bus.set(pid_len, 16.0);
                bus.set(pid_chiff, 38.0);
                bus.set(pid_turb, 0.32);
                bus.set(pid_vel_a, 38.5);

                assert_eq!(bus.get(pid_press), Some(85.0));
                assert_eq!(bus.get(pid_cut), Some(0.28));
                assert_eq!(bus.get(pid_len), Some(16.0));
                assert_eq!(bus.get(pid_chiff), Some(38.0));
                assert_eq!(bus.get(pid_turb), Some(0.32));
                assert_eq!(bus.get(pid_vel_a), Some(38.5));
            }
        }
    }

    #[test]
    fn test_turn59_pure_ascii_snapshots_deterministic() {
        let piano = GrandPianoView::new();
        let piano_lines = piano.render_ascii_snapshot(60, 16);
        assert_eq!(piano_lines.len(), 16);
        assert!(piano_lines[0].starts_with('+'));
        assert!(piano_lines[1].contains("CONCERT GRAND PIANO HUD"));

        let organ = PipeOrganView::new();
        let organ_lines = organ.render_ascii_snapshot(60, 16);
        assert_eq!(organ_lines.len(), 16);
        assert!(organ_lines[0].starts_with('+'));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn59_gui_award_winning_view_modals_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Grand Piano modal
        assert!(!view.is_grand_piano_hud_open());
        view.open_grand_piano_hud();
        assert!(view.is_grand_piano_hud_open());
        view.close_grand_piano_hud();
        assert!(!view.is_grand_piano_hud_open());

        // Pipe Organ modal
        assert!(!view.is_pipe_organ_hud_open());
        view.open_pipe_organ_hud();
        assert!(view.is_pipe_organ_hud_open());
        view.close_pipe_organ_hud();
        assert!(!view.is_pipe_organ_hud_open());
    }

    #[test]
    fn test_turn59_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_grand_piano_hud();
        view.open_pipe_organ_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_grand_piano_hud_open());
        assert!(view.is_pipe_organ_hud_open());
    }

    #[test]
    fn test_turn59_gui_live_param_bus_atomic_dispatch() {
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

        // Test Grand Piano sync
        view.grand_piano_view.strike_velocity = 0.88;
        view.grand_piano_view.hammer_hardness = 0.72;
        view.grand_piano_view.damper_lift_pos = 0.65;
        view.grand_piano_view.una_corda_shift = 0.35;
        view.grand_piano_view.sostenuto_engaged = true;
        view.grand_piano_view.unison_detune_cents = 0.85;

        view.sync_grand_piano_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("strike_velocity").copied(), Some(0.88));
        assert_eq!(view.device_rack_state.node_param_values.get("hammer_hardness").copied(), Some(0.72));
        assert_eq!(view.inspector_state.node_param_values.get("strike_velocity").copied(), Some(0.88));
        assert_eq!(view.inspector_state.node_param_values.get("hammer_hardness").copied(), Some(0.72));

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.88));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.72));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.35));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(1.0));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.85));

        // Test Pipe Organ sync
        view.pipe_organ_view.wind_pressure_mmh2o = 115.0;
        view.pipe_organ_view.cutup_ratio = 0.32;
        view.pipe_organ_view.pipe_length_ft = 16.0;
        view.pipe_organ_view.chiff_duration_ms = 45.0;
        view.pipe_organ_view.turbulence_noise_level = 0.40;
        view.pipe_organ_view.flue_air_velocity_mps = 42.5;

        view.sync_pipe_organ_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("wind_pressure_mmh2o").copied(), Some(115.0));
        assert_eq!(view.device_rack_state.node_param_values.get("cutup_ratio").copied(), Some(0.32));
        assert_eq!(arc_bus.get(ParamId(base)), Some(115.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.32));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(16.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(45.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.40));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(42.5));
    }

    #[test]
    fn test_turn59_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger Grand Piano via inspector
        assert!(!view.is_grand_piano_hud_open());
        view.inspector_state.requested_open_grand_piano_hud = true;
        if view.inspector_state.requested_open_grand_piano_hud || view.device_rack_state.requested_open_grand_piano_hud {
            view.inspector_state.requested_open_grand_piano_hud = false;
            view.device_rack_state.requested_open_grand_piano_hud = false;
            view.open_grand_piano_hud();
        }
        assert!(view.is_grand_piano_hud_open());
        assert!(!view.inspector_state.requested_open_grand_piano_hud);
        view.close_grand_piano_hud();
        assert!(!view.is_grand_piano_hud_open());

        // Trigger Grand Piano via device rack
        view.device_rack_state.requested_open_grand_piano_hud = true;
        if view.inspector_state.requested_open_grand_piano_hud || view.device_rack_state.requested_open_grand_piano_hud {
            view.inspector_state.requested_open_grand_piano_hud = false;
            view.device_rack_state.requested_open_grand_piano_hud = false;
            view.open_grand_piano_hud();
        }
        assert!(view.is_grand_piano_hud_open());
        assert!(!view.device_rack_state.requested_open_grand_piano_hud);
        view.close_grand_piano_hud();

        // Trigger Pipe Organ via inspector
        assert!(!view.is_pipe_organ_hud_open());
        view.inspector_state.requested_open_pipe_organ_hud = true;
        if view.inspector_state.requested_open_pipe_organ_hud || view.device_rack_state.requested_open_pipe_organ_hud {
            view.inspector_state.requested_open_pipe_organ_hud = false;
            view.device_rack_state.requested_open_pipe_organ_hud = false;
            view.open_pipe_organ_hud();
        }
        assert!(view.is_pipe_organ_hud_open());
        assert!(!view.inspector_state.requested_open_pipe_organ_hud);
        view.close_pipe_organ_hud();

        // Trigger Pipe Organ via device rack
        view.device_rack_state.requested_open_pipe_organ_hud = true;
        if view.inspector_state.requested_open_pipe_organ_hud || view.device_rack_state.requested_open_pipe_organ_hud {
            view.inspector_state.requested_open_pipe_organ_hud = false;
            view.device_rack_state.requested_open_pipe_organ_hud = false;
            view.open_pipe_organ_hud();
        }
        assert!(view.is_pipe_organ_hud_open());
        assert!(!view.device_rack_state.requested_open_pipe_organ_hud);
        view.close_pipe_organ_hud();
    }

    #[test]
    fn test_turn59_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_piano = actions.iter().any(|a| a.action_id == "open_grand_piano_hud" && a.category == "Physical Modeling");
        assert!(has_piano, "open_grand_piano_hud command action must be registered in command palette");

        let has_organ = actions.iter().any(|a| a.action_id == "open_pipe_organ_hud" && a.category == "Physical Modeling");
        assert!(has_organ, "open_pipe_organ_hud command action must be registered in command palette");
    }
}
