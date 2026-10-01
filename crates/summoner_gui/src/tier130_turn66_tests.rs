#![allow(clippy::all)]

//! Test suite for Turn #66 / Turn #29 deliverable — Spruce Soundboard & Bridge Wave Scattering & Sympathetic Resonance Coupling HUDs Convergence (Milestones 18, 26 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::soundboard_bridge_view::{
        SoundboardBridgeView, SoundboardHudPreset, SOUNDBOARD_PUCK_HIT_RADIUS,
        MIN_BRIDGE_BLEED, MAX_BRIDGE_BLEED, MIN_DECAY_SCALE, MAX_DECAY_SCALE,
    };
    use crate::views::sympathetic_coupling_view::{
        SympatheticCouplingView, SympatheticPreset, SYMPATHETIC_PUCK_HIT_RADIUS,
        NUM_COUPLED_STRINGS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn66_pure_soundboard_bridge_view_initialization_and_presets() {
        let mut view = SoundboardBridgeView::new();
        assert_eq!(view.preset, SoundboardHudPreset::SteinwayD9Foot);
        assert!((view.bridge_bleed - 0.35).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 1.0).abs() < 1e-3);
        assert!((view.bridge_impedance - 420.0).abs() < 1e-3);
        assert!((view.inharmonicity_b - 0.00018).abs() < 1e-6);
        assert_eq!(view.modal_frequencies.len(), 8);
        assert_eq!(view.modal_qs.len(), 8);
        assert_eq!(SOUNDBOARD_PUCK_HIT_RADIUS, 22.0);

        // Test presets
        view.set_preset(SoundboardHudPreset::BösendorferImperial);
        assert_eq!(view.preset, SoundboardHudPreset::BösendorferImperial);
        assert!((view.bridge_bleed - 0.45).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 1.35).abs() < 1e-3);
        assert!((view.bridge_impedance - 380.0).abs() < 1e-3);
        assert!((view.inharmonicity_b - 0.00012).abs() < 1e-6);

        view.set_preset(SoundboardHudPreset::YamahaCFX);
        assert_eq!(view.preset, SoundboardHudPreset::YamahaCFX);
        assert!((view.bridge_bleed - 0.28).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 0.85).abs() < 1e-3);
        assert!((view.bridge_impedance - 480.0).abs() < 1e-3);
        assert!((view.inharmonicity_b - 0.00025).abs() < 1e-6);

        view.set_preset(SoundboardHudPreset::IntimateStudio);
        assert_eq!(view.preset, SoundboardHudPreset::IntimateStudio);
        assert!((view.bridge_bleed - 0.20).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 0.75).abs() < 1e-3);
        assert!((view.bridge_impedance - 520.0).abs() < 1e-3);
        assert!((view.inharmonicity_b - 0.00010).abs() < 1e-6);

        view.set_preset(SoundboardHudPreset::ImpressionistUnaCorda);
        assert_eq!(view.preset, SoundboardHudPreset::ImpressionistUnaCorda);
        assert!((view.bridge_bleed - 0.50).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 1.20).abs() < 1e-3);
        assert!((view.bridge_impedance - 360.0).abs() < 1e-3);

        view.set_preset(SoundboardHudPreset::PreparedAvantGarde);
        assert_eq!(view.preset, SoundboardHudPreset::PreparedAvantGarde);
        assert!((view.bridge_bleed - 0.15).abs() < 1e-3);
        assert!((view.soundboard_decay_scale - 0.40).abs() < 1e-3);
        assert!((view.bridge_impedance - 600.0).abs() < 1e-3);

        // Puck updates
        view.update_physics_from_puck(0.50, 0.50);
        assert!((view.puck_pos.0 - 0.50).abs() < 1e-3);
        assert!((view.puck_pos.1 - 0.50).abs() < 1e-3);
        assert!(view.bridge_bleed >= MIN_BRIDGE_BLEED && view.bridge_bleed <= MAX_BRIDGE_BLEED);
        assert!(view.soundboard_decay_scale >= MIN_DECAY_SCALE && view.soundboard_decay_scale <= MAX_DECAY_SCALE);

        // Hit testing
        let puck_x = 50.0 + view.puck_pos.0 * 200.0;
        let puck_y = 100.0 + (1.0 - view.puck_pos.1) * 150.0;
        assert!(view.hit_test_soundboard_puck((puck_x, puck_y), 50.0, 100.0, 200.0, 150.0));
        assert!(view.hit_test_soundboard_puck((puck_x + 10.0, puck_y + 10.0), 50.0, 100.0, 200.0, 150.0));
        assert!(!view.hit_test_soundboard_puck((puck_x + 35.0, puck_y), 50.0, 100.0, 200.0, 150.0));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SOUNDBOARD & BRIDGE HUD"));
        assert!(snap.contains("Bridge Bleed:"));
        assert!(snap.contains("Modes:"));
    }

    #[test]
    fn test_turn66_pure_sympathetic_coupling_view_initialization_and_presets() {
        let mut view = SympatheticCouplingView::new();
        assert_eq!(view.preset, SympatheticPreset::AcousticGuitar);
        assert!((view.coupling_strength - 0.15).abs() < 1e-3);
        assert!((view.bridge_loss - 0.995).abs() < 1e-4);
        assert!((view.bridge_impedance_z - 2.5).abs() < 1e-3);
        assert_eq!(view.string_energy_levels.len(), NUM_COUPLED_STRINGS);
        assert_eq!(view.string_frequencies_hz.len(), NUM_COUPLED_STRINGS);
        assert_eq!(SYMPATHETIC_PUCK_HIT_RADIUS, 22.0);

        // Presets
        view.preset = SympatheticPreset::GrandPiano;
        view.update_physics();
        assert!((view.coupling_strength - 0.35).abs() < 1e-3);
        assert!((view.bridge_loss - 0.998).abs() < 1e-4);

        view.preset = SympatheticPreset::ClassicalNylon;
        view.update_physics();
        assert!((view.coupling_strength - 0.10).abs() < 1e-3);
        assert!((view.bridge_loss - 0.992).abs() < 1e-4);

        view.preset = SympatheticPreset::ConcertHarp;
        view.update_physics();
        assert!((view.coupling_strength - 0.40).abs() < 1e-3);
        assert!((view.bridge_loss - 0.999).abs() < 1e-4);

        view.preset = SympatheticPreset::SitarTarab;
        view.update_physics();
        assert!((view.coupling_strength - 0.30).abs() < 1e-3);
        assert!((view.bridge_loss - 0.996).abs() < 1e-4);

        // Matrix cell evaluations
        let diag = view.evaluate_coupling_matrix_cell(0, 0);
        let off_diag = view.evaluate_coupling_matrix_cell(0, 1);
        let far_off = view.evaluate_coupling_matrix_cell(0, 5);
        assert!(diag > off_diag);
        assert!(off_diag > far_off);

        // Hit testing
        let canvas = Rect { x: 60.0, y: 120.0, width: 240.0, height: 180.0 };
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_coupling_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_coupling_puck((puck_x + 8.0, puck_y - 8.0), canvas));
        assert!(!view.hit_test_coupling_puck((puck_x + 40.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("="));
        assert!(snap.contains("+"));
    }

    #[test]
    fn test_turn66_pure_parambus_slot_arithmetic_channel_isolation() {
        let track_id = 3u32;
        let slot_idx = 2u32;
        let base_id = track_id * 1000 + slot_idx * 20;

        let pid_bleed  = ParamId(base_id);
        let pid_decay  = ParamId(base_id + 1);
        let pid_imp    = ParamId(base_id + 2);
        let pid_inharm = ParamId(base_id + 3);

        assert_eq!(pid_bleed.0, 3040);
        assert_eq!(pid_decay.0, 3041);
        assert_eq!(pid_imp.0, 3042);
        assert_eq!(pid_inharm.0, 3043);

        // Verify next slot isolation
        let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
        assert_eq!(next_slot_base, 3060);
        assert!(next_slot_base > pid_inharm.0);
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
    fn test_turn66_gui_award_winning_soundboard_and_sympathetic_hud_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_soundboard_hud_open());
        assert!(!view.is_sympathetic_hud_open());

        view.open_soundboard_hud();
        assert!(view.is_soundboard_hud_open());
        view.close_soundboard_hud();
        assert!(!view.is_soundboard_hud_open());

        view.open_sympathetic_hud();
        assert!(view.is_sympathetic_hud_open());
        view.close_sympathetic_hud();
        assert!(!view.is_sympathetic_hud_open());
    }

    #[test]
    fn test_turn66_gui_headless_rendering_no_panics() {
        let mut view = AwardWinningGuiView::new();
        view.open_soundboard_hud();
        view.open_sympathetic_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_soundboard_hud_open());
        assert!(view.is_sympathetic_hud_open());
    }

    #[test]
    fn test_turn66_gui_live_parambus_atomic_sync_soundboard_and_sympathetic() {
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

        // 1. Sync Soundboard HUD
        view.soundboard_view.bridge_bleed = 0.42;
        view.soundboard_view.soundboard_decay_scale = 1.15;
        view.soundboard_view.bridge_impedance = 460.0;
        view.soundboard_view.inharmonicity_b = 0.00022;
        view.sync_soundboard_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.42));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(1.15));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(460.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.00022));

        assert_eq!(view.device_rack_state.node_param_values.get("bridge_bleed"), Some(&0.42));
        assert_eq!(view.device_rack_state.node_param_values.get("soundboard_decay_scale"), Some(&1.15));
        assert_eq!(view.inspector_state.node_param_values.get("bridge_bleed"), Some(&0.42));

        // 2. Sync Sympathetic HUD
        view.sympathetic_view.coupling_strength = 0.28;
        view.sympathetic_view.bridge_loss = 0.996;
        view.sympathetic_view.bridge_impedance_z = 3.2;
        view.sympathetic_view.total_radiated_energy = 0.65;
        view.sync_sympathetic_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.28));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.996));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(3.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.65));

        assert_eq!(view.device_rack_state.node_param_values.get("coupling_strength"), Some(&0.28));
        assert_eq!(view.device_rack_state.node_param_values.get("bridge_loss"), Some(&0.996));
        assert_eq!(view.inspector_state.node_param_values.get("coupling_strength"), Some(&0.28));
    }

    #[test]
    fn test_turn66_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // 1. Trigger Soundboard from Inspector
        view.inspector_state.requested_open_soundboard_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_soundboard_hud);
        assert!(view.is_soundboard_hud_open());
        view.close_soundboard_hud();

        // 2. Trigger Sympathetic from Inspector
        view.inspector_state.requested_open_sympathetic_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_sympathetic_hud);
        assert!(view.is_sympathetic_hud_open());
        view.close_sympathetic_hud();

        // 3. Trigger Soundboard from Device Rack
        view.device_rack_state.requested_open_soundboard_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_soundboard_hud);
        assert!(view.is_soundboard_hud_open());
        view.close_soundboard_hud();

        // 4. Trigger Sympathetic from Device Rack
        view.device_rack_state.requested_open_sympathetic_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_sympathetic_hud);
        assert!(view.is_sympathetic_hud_open());
    }

    #[test]
    fn test_turn66_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let has_soundboard = palette.actions.iter().any(|a| a.action_id == "open_soundboard_hud");
        let has_sympathetic = palette.actions.iter().any(|a| a.action_id == "open_sympathetic_hud");

        assert!(has_soundboard, "Expected open_soundboard_hud in command palette");
        assert!(has_sympathetic, "Expected open_sympathetic_hud in command palette");
    }
}
