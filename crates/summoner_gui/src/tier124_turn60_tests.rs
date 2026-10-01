#![allow(clippy::all)]

//! Test suite for Turn #60 (Turn #25 Deliverable) — Physical Modeling Electromechanical Tine & Reed Electric Piano & 9-Drawbar Tonewheel Organ HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::layout_math::Rect;
    use crate::views::electric_piano_view::{
        ElectricPianoView, GuiEpProfile, GuiEpModel, EP_PUCK_HIT_RADIUS,
    };
    use crate::views::tonewheel_organ_view::{
        TonewheelOrganView, GuiVibratoMode, DrawbarColorGroup, DRAWBAR_INFO, DRAWBAR_HANDLE_HIT_RADIUS,
    };

    #[test]
    fn test_turn60_pure_electric_piano_models_and_simulation() {
        let mut view = ElectricPianoView::new();
        assert_eq!(view.profile, GuiEpProfile::ClassicRhodesSuitcase);
        assert_eq!(view.model, GuiEpModel::RhodesTine);
        assert!((view.air_gap_mm - 1.8).abs() < 1e-4);
        assert!((view.alignment_offset_mm - 0.45).abs() < 1e-4);
        assert!((view.hammer_hardness - 0.50).abs() < 1e-4);
        assert!((view.bark_drive - 0.55).abs() < 1e-4);
        assert!((view.tonebar_coupling - 0.70).abs() < 1e-4);
        assert!((view.damper_clunk_volume - 0.35).abs() < 1e-4);
        assert!(view.tremolo_enabled);
        assert!(view.tremolo_stereo);

        // Profile switching
        view.set_profile(GuiEpProfile::BarkingDynoRhodes);
        assert_eq!(view.profile, GuiEpProfile::BarkingDynoRhodes);
        assert_eq!(view.model, GuiEpModel::RhodesTine);
        assert!((view.air_gap_mm - 1.0).abs() < 1e-4);
        assert!((view.alignment_offset_mm - 0.60).abs() < 1e-4);
        assert!((view.bark_drive - 0.90).abs() < 1e-4);
        assert!(!view.tremolo_enabled);

        view.set_profile(GuiEpProfile::MellowRhodesStage);
        assert_eq!(view.profile, GuiEpProfile::MellowRhodesStage);
        assert_eq!(view.model, GuiEpModel::RhodesTine);
        assert!((view.air_gap_mm - 2.4).abs() < 1e-4);
        assert!((view.alignment_offset_mm - 0.20).abs() < 1e-4);

        view.set_profile(GuiEpProfile::ClassicWurlitzer200A);
        assert_eq!(view.profile, GuiEpProfile::ClassicWurlitzer200A);
        assert_eq!(view.model, GuiEpModel::WurlitzerReed);
        assert!((view.air_gap_mm - 1.2).abs() < 1e-4);
        assert!((view.alignment_offset_mm - 0.15).abs() < 1e-4);
        assert!(view.tremolo_enabled);
        assert!(!view.tremolo_stereo);

        view.set_profile(GuiEpProfile::SoulOverdrivenWurli);
        assert_eq!(view.profile, GuiEpProfile::SoulOverdrivenWurli);
        assert_eq!(view.model, GuiEpModel::WurlitzerReed);
        assert!((view.air_gap_mm - 0.9).abs() < 1e-4);
        assert!((view.tube_drive_db - 14.0).abs() < 1e-4);

        view.set_profile(GuiEpProfile::BelledAmbientRhodes);
        assert_eq!(view.profile, GuiEpProfile::BelledAmbientRhodes);
        assert_eq!(view.model, GuiEpModel::RhodesTine);
        assert!((view.tonebar_coupling - 0.95).abs() < 1e-4);

        // Normalization roundtrips
        let norm_gap = ElectricPianoView::air_gap_to_normalized(2.5);
        let recon_gap = ElectricPianoView::normalized_to_air_gap(norm_gap);
        assert!((recon_gap - 2.5).abs() < 1e-3);

        let norm_off = ElectricPianoView::alignment_to_normalized(0.5);
        let recon_off = ElectricPianoView::normalized_to_alignment(norm_off);
        assert!((recon_off - 0.5).abs() < 1e-3);

        // Hit testing
        let canvas = Rect::new(10.0, 10.0, 200.0, 150.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_ep_puck((px, py), canvas));
        assert!(view.hit_test_ep_puck((px + EP_PUCK_HIT_RADIUS * 0.8, py), canvas));
        assert!(!view.hit_test_ep_puck((px + EP_PUCK_HIT_RADIUS * 1.5, py), canvas));

        // Wave evaluation
        let wave = view.evaluate_pickup_wave();
        assert_eq!(wave.len(), 32);
        for val in wave {
            assert!(val >= -1.0 && val <= 1.0);
        }
    }

    #[test]
    fn test_turn60_pure_tonewheel_organ_drawbars_and_percussion() {
        let mut view = TonewheelOrganView::new();
        assert_eq!(view.drawbars, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0]);
        assert_eq!(view.vibrato_mode, GuiVibratoMode::C3);
        assert!(view.percussion.enabled);
        assert!(view.percussion.third_harmonic);
        assert!(view.percussion.fast_decay);

        // Preset registrations
        view.set_preset("Jimmy Smith Jazz");
        assert_eq!(view.drawbars, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(view.percussion.soft_volume);

        view.set_preset("Full Organ");
        assert_eq!(view.drawbars, [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]);
        assert!(!view.percussion.enabled);

        view.set_preset("Mellow Flutes");
        assert_eq!(view.drawbars, [0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(view.vibrato_mode, GuiVibratoMode::Off);

        view.set_preset("Brother Jack");
        assert_eq!(view.drawbars, [8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0, 8.0]);
        assert_eq!(view.vibrato_mode, GuiVibratoMode::V2);

        view.set_preset("Whiter Shade");
        assert_eq!(view.drawbars, [6.0, 8.0, 8.0, 6.0, 0.0, 0.0, 0.0, 0.0, 4.0]);
        assert_eq!(view.vibrato_mode, GuiVibratoMode::C2);

        // Drawbar set and normalization
        view.set_drawbar(0, 5.5);
        assert!((view.drawbars[0] - 5.5).abs() < 1e-4);
        let norm_d = TonewheelOrganView::drawbar_to_normalized(view.drawbars[0]);
        let recon_d = TonewheelOrganView::normalized_to_drawbar(norm_d);
        assert!((recon_d - 5.5).abs() < 1e-3);

        // Hit testing
        assert!(view.hit_test_drawbar_handle((100.0, 100.0), (100.0, 100.0)));
        assert!(view.hit_test_drawbar_handle((100.0 + DRAWBAR_HANDLE_HIT_RADIUS * 0.8, 100.0), (100.0, 100.0)));
        assert!(!view.hit_test_drawbar_handle((100.0 + DRAWBAR_HANDLE_HIT_RADIUS * 1.5, 100.0), (100.0, 100.0)));

        assert_eq!(DRAWBAR_INFO.len(), 9);
        assert_eq!(DRAWBAR_INFO[0].2, DrawbarColorGroup::BrownSubOctave);
        assert_eq!(DRAWBAR_INFO[2].2, DrawbarColorGroup::WhiteOctave);
        assert_eq!(DRAWBAR_INFO[4].2, DrawbarColorGroup::BlackHarmonic);
    }

    #[test]
    fn test_turn60_pure_channel_slot_arithmetic_isolation() {
        let mut bus = ParamBus::new();
        // Tracks 1..4, chain slots 0..3
        for track_id in 1..=4 {
            for slot_idx in 0..=3 {
                let base = track_id * 1000 + slot_idx * 20;
                for offset in 0..6 {
                    let pid = ParamId(base + offset);
                    let val = (track_id as f32) * 100.0 + (slot_idx as f32) * 10.0 + offset as f32;
                    bus.register(pid, val);
                }
            }
        }

        // Verify isolation
        for track_id in 1..=4 {
            for slot_idx in 0..=3 {
                let base = track_id * 1000 + slot_idx * 20;
                for offset in 0..6 {
                    let pid = ParamId(base + offset);
                    let expected = (track_id as f32) * 100.0 + (slot_idx as f32) * 10.0 + offset as f32;
                    assert_eq!(bus.get(pid), Some(expected));
                }
            }
        }
    }

    #[test]
    fn test_turn60_pure_ascii_snapshots_deterministic() {
        let ep = ElectricPianoView::new();
        let ep_lines = ep.render_ascii_snapshot(80, 16);
        assert_eq!(ep_lines.len(), 16);
        assert!(ep_lines[0].contains("EP HUD"));

        let organ = TonewheelOrganView::new();
        let organ_lines = organ.render_ascii_snapshot(80, 16);
        assert!(!organ_lines.is_empty());
        assert!(organ_lines[0].contains("TONEWHEEL ORGAN"));
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
    fn test_turn60_gui_award_winning_view_modals_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // Electric Piano modal
        assert!(!view.is_electric_piano_hud_open());
        view.open_electric_piano_hud();
        assert!(view.is_electric_piano_hud_open());
        view.close_electric_piano_hud();
        assert!(!view.is_electric_piano_hud_open());

        // Tonewheel Organ modal
        assert!(!view.is_tonewheel_organ_hud_open());
        view.open_tonewheel_organ_hud();
        assert!(view.is_tonewheel_organ_hud_open());
        view.close_tonewheel_organ_hud();
        assert!(!view.is_tonewheel_organ_hud_open());
    }

    #[test]
    fn test_turn60_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_electric_piano_hud();
        view.open_tonewheel_organ_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_electric_piano_hud_open());
        assert!(view.is_tonewheel_organ_hud_open());
    }

    #[test]
    fn test_turn60_gui_live_param_bus_atomic_dispatch() {
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

        // Test Electric Piano sync
        view.electric_piano_view.air_gap_mm = 2.2;
        view.electric_piano_view.alignment_offset_mm = 0.8;
        view.electric_piano_view.hammer_hardness = 0.65;
        view.electric_piano_view.bark_drive = 0.75;
        view.electric_piano_view.tonebar_coupling = 0.80;
        view.electric_piano_view.damper_clunk_volume = 0.40;

        view.sync_electric_piano_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("air_gap_mm").copied(), Some(2.2));
        assert_eq!(view.device_rack_state.node_param_values.get("alignment_offset_mm").copied(), Some(0.8));
        assert_eq!(view.inspector_state.node_param_values.get("air_gap_mm").copied(), Some(2.2));
        assert_eq!(view.inspector_state.node_param_values.get("alignment_offset_mm").copied(), Some(0.8));

        assert_eq!(arc_bus.get(ParamId(base)), Some(2.2));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.8));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.75));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.80));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.40));

        // Test Tonewheel Organ sync
        view.tonewheel_organ_view.drawbars[0] = 7.0;
        view.tonewheel_organ_view.drawbars[2] = 6.0;
        view.tonewheel_organ_view.drawbars[3] = 5.0;
        view.tonewheel_organ_view.drawbars[8] = 4.0;
        view.tonewheel_organ_view.key_click_amount = 0.55;
        view.tonewheel_organ_view.crosstalk_amount = 0.12;

        view.sync_tonewheel_organ_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("drawbar_16").copied(), Some(7.0));
        assert_eq!(view.device_rack_state.node_param_values.get("drawbar_8").copied(), Some(6.0));
        assert_eq!(view.inspector_state.node_param_values.get("drawbar_16").copied(), Some(7.0));
        assert_eq!(view.inspector_state.node_param_values.get("drawbar_8").copied(), Some(6.0));

        assert_eq!(arc_bus.get(ParamId(base)), Some(7.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(6.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(5.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(4.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.55));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.12));
    }

    #[test]
    fn test_turn60_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Trigger from inspector
        view.inspector_state.requested_open_electric_piano_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_electric_piano_hud_open());
        assert!(!view.inspector_state.requested_open_electric_piano_hud);
        view.close_electric_piano_hud();

        // Trigger from rack
        view.device_rack_state.requested_open_tonewheel_organ_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_tonewheel_organ_hud_open());
        assert!(!view.device_rack_state.requested_open_tonewheel_organ_hud);
    }

    #[test]
    fn test_turn60_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let ep_action = palette.actions.iter().find(|a| a.action_id == "open_electric_piano_hud");
        assert!(ep_action.is_some());
        let ep = ep_action.unwrap();
        assert_eq!(ep.category, "Physical Modeling");
        assert!(ep.label.contains("Electric Piano"));

        let organ_action = palette.actions.iter().find(|a| a.action_id == "open_tonewheel_organ_hud");
        assert!(organ_action.is_some());
        let organ = organ_action.unwrap();
        assert_eq!(organ.category, "Physical Modeling");
        assert!(organ.label.contains("Tonewheel Organ"));
    }
}
