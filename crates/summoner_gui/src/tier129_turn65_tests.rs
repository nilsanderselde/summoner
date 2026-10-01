#![allow(clippy::all)]

//! Test suite for Turn #65 / Turn #28 deliverable — Pneumatic Bellows Compression Dynamics & Sitar Curved Jawari Bridge HUDs Convergence (Milestones 27, 28 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::bellows_view::{
        BellowsView, BellowsHudPreset, BELLOWS_PUCK_HIT_RADIUS,
        MIN_BELLOWS_PRESSURE_PA, MAX_BELLOWS_PRESSURE_PA, MIN_VALVE_VELOCITY, MAX_VALVE_VELOCITY,
    };
    use crate::views::jawari_bridge_view::{
        JawariBridgeView, JawariBridgeHudPreset, JAWARI_PUCK_HIT_RADIUS,
        MIN_JAWARI_CLEARANCE_GAP, MAX_JAWARI_CLEARANCE_GAP,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn65_pure_bellows_view_initialization_and_presets() {
        let mut view = BellowsView::new();
        assert_eq!(view.preset, BellowsHudPreset::TangoBandoneonAccented);
        assert!((view.bellows_pressure_pa - 680.0).abs() < 1e-3);
        assert!((view.valve_velocity - 0.90).abs() < 1e-3);
        assert!((view.cassotto_aperture - 0.70).abs() < 1e-3);
        assert!((view.musette_detune_cents - 2.0).abs() < 1e-3);
        assert_eq!(view.register_mask, 0b00011);
        assert!((view.reed_stiffness - 1.25).abs() < 1e-3);
        assert_eq!(BELLOWS_PUCK_HIT_RADIUS, 22.0);

        // Test presets
        view.set_preset(BellowsHudPreset::FrenchMusetteAccordion);
        assert_eq!(view.preset, BellowsHudPreset::FrenchMusetteAccordion);
        assert!((view.bellows_pressure_pa - 420.0).abs() < 1e-3);
        assert!((view.musette_detune_cents - 18.0).abs() < 1e-3);
        assert_eq!(view.register_mask, 0b01110);

        view.set_preset(BellowsHudPreset::RussianBayanTutti);
        assert_eq!(view.preset, BellowsHudPreset::RussianBayanTutti);
        assert!((view.bellows_pressure_pa - 850.0).abs() < 1e-3);
        assert_eq!(view.register_mask, 0b11111);

        view.set_preset(BellowsHudPreset::VintageHarmoniumDrone);
        assert_eq!(view.preset, BellowsHudPreset::VintageHarmoniumDrone);
        assert!((view.bellows_pressure_pa - (-280.0)).abs() < 1e-3);
        assert_eq!(view.register_mask, 0b00111);

        view.set_preset(BellowsHudPreset::EnglishConcertinaFast);
        assert_eq!(view.preset, BellowsHudPreset::EnglishConcertinaFast);
        assert!((view.bellows_pressure_pa - 520.0).abs() < 1e-3);
        assert_eq!(view.register_mask, 0b00010);

        // Normalization checks
        assert_eq!(BellowsView::pressure_to_normalized(MIN_BELLOWS_PRESSURE_PA), 0.0);
        assert_eq!(BellowsView::pressure_to_normalized(MAX_BELLOWS_PRESSURE_PA), 1.0);
        assert!((BellowsView::normalized_to_pressure(0.5) - 0.0).abs() < 1e-3);

        assert_eq!(BellowsView::velocity_to_normalized(MIN_VALVE_VELOCITY), 0.0);
        assert_eq!(BellowsView::velocity_to_normalized(MAX_VALVE_VELOCITY), 1.0);

        // Puck updates
        view.update_physics_from_puck(0.75, 0.50);
        assert!((view.puck_pos.0 - 0.75).abs() < 1e-3);
        assert!((view.puck_pos.1 - 0.50).abs() < 1e-3);
        assert!(view.bellows_pressure_pa > 0.0);

        // Hit testing
        let canvas_rect = Rect { x: 50.0, y: 100.0, width: 200.0, height: 150.0 };
        let puck_x = canvas_rect.x + view.puck_pos.0 * canvas_rect.width;
        let puck_y = canvas_rect.y + (1.0 - view.puck_pos.1) * canvas_rect.height;
        assert!(view.hit_test_bellows_puck((puck_x, puck_y), canvas_rect));
        assert!(view.hit_test_bellows_puck((puck_x + 10.0, puck_y + 10.0), canvas_rect));
        assert!(!view.hit_test_bellows_puck((puck_x + 35.0, puck_y), canvas_rect));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("BELLOWS DYNAMICS HUD"));
        assert!(snap.contains("Aperture:"));
    }

    #[test]
    fn test_turn65_pure_jawari_bridge_view_initialization_and_presets() {
        let mut view = JawariBridgeView::new();
        assert_eq!(view.preset, JawariBridgeHudPreset::DeerHornRaviShankar);
        assert!((view.clearance_gap_mm - 0.18).abs() < 1e-3);
        assert!((view.jiva_thread_pos - 0.45).abs() < 1e-3);
        assert!((view.curvature_c - 120.0).abs() < 1e-3);
        assert!((view.tarab_bleed - 0.40).abs() < 1e-3);
        assert_eq!(view.tarab_energy.len(), 13);
        assert_eq!(JAWARI_PUCK_HIT_RADIUS, 22.0);

        // Preset transitions and acoustic profiles
        view.set_preset(JawariBridgeHudPreset::CamelBoneVilayatKhan);
        assert_eq!(view.preset, JawariBridgeHudPreset::CamelBoneVilayatKhan);
        assert!((view.clearance_gap_mm - 0.08).abs() < 1e-3);
        assert!((view.curvature_c - 240.0).abs() < 1e-3);
        assert!((view.tarab_bleed - 0.35).abs() < 1e-3);

        view.set_preset(JawariBridgeHudPreset::EbonySurbaharMellow);
        assert_eq!(view.preset, JawariBridgeHudPreset::EbonySurbaharMellow);
        assert!((view.clearance_gap_mm - 0.35).abs() < 1e-3);
        assert!((view.curvature_c - 60.0).abs() < 1e-3);

        view.set_preset(JawariBridgeHudPreset::SyntheticDelrinPrecision);
        assert_eq!(view.preset, JawariBridgeHudPreset::SyntheticDelrinPrecision);
        assert!((view.clearance_gap_mm - 0.15).abs() < 1e-3);
        assert!((view.curvature_c - 160.0).abs() < 1e-3);

        view.set_preset(JawariBridgeHudPreset::ElectricMetalSizzle);
        assert_eq!(view.preset, JawariBridgeHudPreset::ElectricMetalSizzle);
        assert!((view.clearance_gap_mm - 0.05).abs() < 1e-3);
        assert!((view.curvature_c - 320.0).abs() < 1e-3);

        view.set_preset(JawariBridgeHudPreset::OpenAcousticGourd);
        assert_eq!(view.preset, JawariBridgeHudPreset::OpenAcousticGourd);
        assert!((view.clearance_gap_mm - 0.22).abs() < 1e-3);
        assert!((view.curvature_c - 90.0).abs() < 1e-3);

        // Puck updates
        view.update_physics_from_puck(0.60, 0.40);
        assert!((view.puck_pos.0 - 0.60).abs() < 1e-3);
        assert!((view.puck_pos.1 - 0.40).abs() < 1e-3);
        assert!(view.clearance_gap_mm >= MIN_JAWARI_CLEARANCE_GAP);
        assert!(view.clearance_gap_mm <= MAX_JAWARI_CLEARANCE_GAP);

        // Hit testing
        let canvas_rect = Rect { x: 40.0, y: 80.0, width: 220.0, height: 160.0 };
        let puck_x = canvas_rect.x + view.puck_pos.0 * canvas_rect.width;
        let puck_y = canvas_rect.y + (1.0 - view.puck_pos.1) * canvas_rect.height;
        assert!(view.hit_test_jawari_puck((puck_x, puck_y), canvas_rect));
        assert!(view.hit_test_jawari_puck((puck_x + 8.0, puck_y - 8.0), canvas_rect));
        assert!(!view.hit_test_jawari_puck((puck_x + 40.0, puck_y), canvas_rect));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("JAWARI BRIDGE & TARAB RESONATOR HUD"));
        assert!(snap.contains("Clearance h0:"));
        assert!(snap.contains("Tarab E:"));
    }

    #[test]
    fn test_turn65_pure_parambus_slot_arithmetic_channel_isolation() {
        let track_id = 2u32;
        let slot_idx = 1u32;
        let base_id = track_id * 1000 + slot_idx * 20;

        let pid_pres = ParamId(base_id);
        let pid_vel  = ParamId(base_id + 1);
        let pid_cass = ParamId(base_id + 2);
        let pid_mus  = ParamId(base_id + 3);
        let pid_st   = ParamId(base_id + 4);

        assert_eq!(pid_pres.0, 2020);
        assert_eq!(pid_vel.0, 2021);
        assert_eq!(pid_cass.0, 2022);
        assert_eq!(pid_mus.0, 2023);
        assert_eq!(pid_st.0, 2024);

        // Verify next slot isolation
        let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
        assert_eq!(next_slot_base, 2040);
        assert!(next_slot_base > pid_st.0);
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
    fn test_turn65_gui_award_winning_bellows_and_jawari_hud_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_bellows_hud_open());
        assert!(!view.is_jawari_bridge_hud_open());

        view.open_bellows_hud();
        assert!(view.is_bellows_hud_open());
        view.close_bellows_hud();
        assert!(!view.is_bellows_hud_open());

        view.open_jawari_bridge_hud();
        assert!(view.is_jawari_bridge_hud_open());
        view.close_jawari_bridge_hud();
        assert!(!view.is_jawari_bridge_hud_open());
    }

    #[test]
    fn test_turn65_gui_headless_rendering_no_panics() {
        let mut view = AwardWinningGuiView::new();
        view.open_bellows_hud();
        view.open_jawari_bridge_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_bellows_hud_open());
        assert!(view.is_jawari_bridge_hud_open());
    }

    #[test]
    fn test_turn65_gui_live_parambus_atomic_sync_bellows_and_jawari() {
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

        // 1. Sync Bellows HUD
        view.bellows_view.bellows_pressure_pa = 720.0;
        view.bellows_view.valve_velocity = 0.85;
        view.bellows_view.cassotto_aperture = 0.65;
        view.bellows_view.musette_detune_cents = 6.0;
        view.bellows_view.reed_stiffness = 1.30;
        view.sync_bellows_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(720.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(6.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(1.30));

        assert_eq!(view.device_rack_state.node_param_values.get("bellows_pressure_pa"), Some(&720.0));
        assert_eq!(view.device_rack_state.node_param_values.get("valve_velocity"), Some(&0.85));
        assert_eq!(view.inspector_state.node_param_values.get("bellows_pressure_pa"), Some(&720.0));

        // 2. Sync Jawari Bridge HUD
        view.jawari_bridge_view.clearance_gap_mm = 0.28;
        view.jawari_bridge_view.jiva_thread_pos = 0.52;
        view.jawari_bridge_view.curvature_c = 140.0;
        view.jawari_bridge_view.tarab_bleed = 0.45;
        view.sync_jawari_bridge_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.28));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.52));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(140.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.45));

        assert_eq!(view.device_rack_state.node_param_values.get("clearance_gap_mm"), Some(&0.28));
        assert_eq!(view.device_rack_state.node_param_values.get("jiva_thread_pos"), Some(&0.52));
        assert_eq!(view.inspector_state.node_param_values.get("clearance_gap_mm"), Some(&0.28));
    }

    #[test]
    fn test_turn65_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // 1. Trigger Bellows from Inspector
        view.inspector_state.requested_open_bellows_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_bellows_hud);
        assert!(view.is_bellows_hud_open());
        view.close_bellows_hud();

        // 2. Trigger Jawari Bridge from Inspector
        view.inspector_state.requested_open_jawari_bridge_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_jawari_bridge_hud);
        assert!(view.is_jawari_bridge_hud_open());
        view.close_jawari_bridge_hud();

        // 3. Trigger Bellows from Device Rack
        view.device_rack_state.requested_open_bellows_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_bellows_hud);
        assert!(view.is_bellows_hud_open());
        view.close_bellows_hud();

        // 4. Trigger Jawari Bridge from Device Rack
        view.device_rack_state.requested_open_jawari_bridge_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_jawari_bridge_hud);
        assert!(view.is_jawari_bridge_hud_open());
    }

    #[test]
    fn test_turn65_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let has_bellows = palette.actions.iter().any(|a| a.action_id == "open_bellows_hud");
        let has_jawari = palette.actions.iter().any(|a| a.action_id == "open_jawari_bridge_hud");

        assert!(has_bellows, "Expected open_bellows_hud in command palette");
        assert!(has_jawari, "Expected open_jawari_bridge_hud in command palette");
    }
}
