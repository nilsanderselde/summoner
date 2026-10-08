#![allow(clippy::all)]

//! Test suite for Turn #86 (Sprint Turn #12 — Sprint Review) — Continuous Spatial Neural Radiance Field & HRTF/BRIR HUD
//! and Neural Polyphonic Vocal Stylizer & Microtonal Ornament Resynthesizer HUD Convergence (Milestones 33, 35 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_radiance_view::{
        NeuralRadianceView, RadianceFieldModel, RADIANCE_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_vocal_stylizer_view::{
        NeuralVocalStylizerView, VocalStyleModel, STYLIZE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn86_pure_neural_radiance_view_initialization_and_kinematics() {
        let mut view = NeuralRadianceView::new();
        assert_eq!(view.model, RadianceFieldModel::InstantNeRFAcoustics);
        assert!((view.heading_deg - 30.0).abs() < 1e-4);
        assert!((view.distance_m - 3.5).abs() < 1e-4);
        assert_eq!(view.latent_dim, 256);
        assert!((view.absorption_alpha - 0.25).abs() < 1e-4);
        assert_eq!(RADIANCE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Model profiles & nominal parameters
        let models = [
            (RadianceFieldModel::InstantNeRFAcoustics, 30.0, 3.5, 256, 0.25, "INSTANT ACOUSTIC NeRF (HASH GRID)"),
            (RadianceFieldModel::ContinuousFieldHRIR, -45.0, 1.2, 128, 0.65, "CONTINUOUS FIELD HRIR (IMPLICIT SH)"),
            (RadianceFieldModel::BinauralRoomRadianceNeRF, 60.0, 5.0, 512, 0.35, "6-DoF BINAURAL BRIR RADIANCE"),
            (RadianceFieldModel::WaveguideNeuralMesh, 0.0, 2.8, 128, 0.15, "WAVEGUIDE NEURAL FDTD MESH"),
            (RadianceFieldModel::DiffusionRoomImpulseField, -90.0, 8.0, 512, 0.45, "DIFFUSION SPATIAL IMPULSE FIELD"),
        ];
        for (m, heading, dist, dim, alpha, name) in models {
            view.set_model(m);
            assert_eq!(view.model, m);
            assert!((view.heading_deg - heading).abs() < 1e-4);
            assert!((view.distance_m - dist).abs() < 1e-4);
            assert_eq!(view.latent_dim, dim);
            assert!((view.absorption_alpha - alpha).abs() < 1e-4);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips
        let head_test = 45.0;
        let head_norm = NeuralRadianceView::heading_to_normalized(head_test);
        let head_back = NeuralRadianceView::normalized_to_heading(head_norm);
        assert!((head_back - head_test).abs() < 1e-3);

        let dist_test = 6.0;
        let dist_norm = NeuralRadianceView::distance_to_normalized(dist_test);
        let dist_back = NeuralRadianceView::normalized_to_distance(dist_norm);
        assert!((dist_back - dist_test).abs() < 1e-3);

        // Simulation update & ray energy check
        view.heading_deg = 0.0;
        view.distance_m = 2.0;
        view.update_radiance_simulation();
        assert!(view.acoustic_radiance_rays[0] > 0.0);
        assert!(view.directivity_clarity_db > 0.0);
        assert!(view.rendering_latency_ms > 0.0);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL CONTINUOUS SPATIAL RADIANCE FIELD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn86_pure_neural_vocal_stylizer_view_initialization_and_resynthesis() {
        let mut view = NeuralVocalStylizerView::new();
        assert_eq!(view.vocal_model, VocalStyleModel::BelCantoOpera);
        assert!((view.style_blend_pct - 0.85).abs() < 1e-4);
        assert!((view.ornament_depth_pct - 0.45).abs() < 1e-4);
        assert_eq!(view.melisma_rate_hz, 4.5);
        assert_eq!(view.vibrato_rate_hz, 5.8);
        assert_eq!(view.vibrato_depth_cents, 95.0);
        assert_eq!(view.microtonal_snap_cents, 25.0);
        assert_eq!(STYLIZE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Model profiles & nominal parameters
        let models = [
            (VocalStyleModel::BelCantoOpera, 0.85, 0.45, 5.8, 95.0, "BEL CANTO OPERA"),
            (VocalStyleModel::ContemporaryPopBelt, 0.70, 0.25, 6.5, 35.0, "CONTEMPORARY BELT"),
            (VocalStyleModel::BulgarianChoirOpenThroat, 0.90, 0.80, 3.2, 10.0, "BULGARIAN CHOIR"),
            (VocalStyleModel::TuvanThroatKargyraa, 0.95, 0.65, 4.0, 20.0, "TUVAN THROAT"),
            (VocalStyleModel::GospelMelismaExpressive, 0.80, 0.90, 6.2, 80.0, "GOSPEL MELISMA"),
        ];
        for (m, blend, ornament, v_rate, v_depth, name) in models {
            view.set_vocal_model(m);
            assert_eq!(view.vocal_model, m);
            assert!((view.style_blend_pct - blend).abs() < 1e-4);
            assert!((view.ornament_depth_pct - ornament).abs() < 1e-4);
            assert!((view.vibrato_rate_hz - v_rate).abs() < 1e-4);
            assert!((view.vibrato_depth_cents - v_depth).abs() < 1e-4);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips
        let blend_test = 0.65;
        let blend_norm = NeuralVocalStylizerView::blend_to_normalized(blend_test);
        let blend_back = NeuralVocalStylizerView::normalized_to_blend(blend_norm);
        assert!((blend_back - blend_test).abs() < 1e-3);

        let ornm_test = 0.75;
        let ornm_norm = NeuralVocalStylizerView::ornament_to_normalized(ornm_test);
        let ornm_back = NeuralVocalStylizerView::normalized_to_ornament(ornm_norm);
        assert!((ornm_back - ornm_test).abs() < 1e-3);

        // Simulation update & radar axes
        view.style_blend_pct = 0.80;
        view.ornament_depth_pct = 0.50;
        view.update_neural_simulation();
        for &axis in &view.expression_radar_axes {
            assert!(axis >= 0.0 && axis <= 1.0);
        }

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL VOCAL EXPRESSION STYLIZER"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn86_pure_hit_test_pucks() {
        let rad_view = NeuralRadianceView::new();
        let rect = Rect {
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 200.0,
        };

        let puck_x = rect.x + rad_view.puck_pos.0 * rect.width;
        let puck_y = rect.y + (1.0 - rad_view.puck_pos.1) * rect.height;
        assert!(rad_view.hit_test_radiance_puck((puck_x, puck_y), rect));
        assert!(!rad_view.hit_test_radiance_puck((puck_x + 60.0, puck_y + 60.0), rect));

        let voc_view = NeuralVocalStylizerView::new();
        let voc_puck_x = rect.x + voc_view.puck_pos.0 * rect.width;
        let voc_puck_y = rect.y + (1.0 - voc_view.puck_pos.1) * rect.height;
        assert!(voc_view.hit_test_stylizer_puck((voc_puck_x, voc_puck_y), rect));
        assert!(!voc_view.hit_test_stylizer_puck((voc_puck_x + 60.0, voc_puck_y + 60.0), rect));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{
        show_modern_device_rack_with_context, ModernDeviceRackState,
    };
    use crate::views::modern_inspector::{
        show_modern_inspector_with_context, ModernInspectorState,
    };
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn86_gui_award_winning_view_radiance_and_vocal_stylizer_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Continuous Spatial Radiance Field HUD
        view.open_neural_radiance_hud();
        assert!(view.is_neural_radiance_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_radiance_hud();
        assert!(!view.is_neural_radiance_hud_open());

        // 2. Neural Vocal Stylizer HUD
        view.open_neural_vocal_stylizer_hud();
        assert!(view.is_neural_vocal_stylizer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_vocal_stylizer_hud();
        assert!(!view.is_neural_vocal_stylizer_hud_open());
    }

    #[test]
    fn test_turn86_gui_lock_free_param_bus_dispatch_radiance_and_vocal_stylizer() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Radiance (offsets 0..=5) and Vocal Stylizer (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Neural Radiance sync
        view.neural_radiance_view.heading_deg = 45.0;
        view.neural_radiance_view.distance_m = 4.2;
        view.neural_radiance_view.latent_dim = 256;
        view.neural_radiance_view.absorption_alpha = 0.35;
        view.neural_radiance_view.directivity_clarity_db = 18.0;
        view.neural_radiance_view.rendering_latency_ms = 1.9;

        view.sync_neural_radiance_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("heading_deg"), Some(&45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&4.2));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_dim"), Some(&256.0));
        assert_eq!(view.inspector_state.node_param_values.get("absorption_alpha"), Some(&0.35));

        let pid_head = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_dist = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_head), Some(45.0));
        assert_eq!(arc_bus.get(pid_dist), Some(4.2));

        // Test Neural Vocal Stylizer sync
        view.neural_vocal_stylizer_view.style_blend_pct = 0.88;
        view.neural_vocal_stylizer_view.ornament_depth_pct = 0.62;
        view.neural_vocal_stylizer_view.vibrato_rate_hz = 6.4;
        view.neural_vocal_stylizer_view.vibrato_depth_cents = 85.0;
        view.neural_vocal_stylizer_view.microtonal_snap_cents = 15.0;
        view.neural_vocal_stylizer_view.melisma_rate_hz = 5.2;

        view.sync_neural_vocal_stylizer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("style_blend_pct"), Some(&0.88));
        assert_eq!(view.device_rack_state.node_param_values.get("ornament_depth_pct"), Some(&0.62));
        assert_eq!(view.device_rack_state.node_param_values.get("vibrato_rate_hz"), Some(&6.4));
        assert_eq!(view.inspector_state.node_param_values.get("vibrato_depth_cents"), Some(&85.0));

        let pid_blend = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_ornm = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_blend), Some(0.88));
        assert_eq!(arc_bus.get(pid_ornm), Some(0.62));
    }

    #[test]
    fn test_turn86_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Neural Radiance trigger propagation from rack
        view.device_rack_state.requested_open_neural_radiance_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_radiance_hud);
        assert!(view.is_neural_radiance_hud_open());
        view.close_neural_radiance_hud();

        // 2. Neural Vocal Stylizer trigger propagation from inspector
        view.inspector_state.requested_open_neural_vocal_stylizer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_vocal_stylizer_hud);
        assert!(view.is_neural_vocal_stylizer_hud_open());
        view.close_neural_vocal_stylizer_hud();
    }

    #[test]
    fn test_turn86_gui_modern_device_rack_and_inspector_headless_render() {
        let ctx = egui::Context::default();
        let mut rack_state = ModernDeviceRackState::default();
        let mut inspector_state = ModernInspectorState::default();

        rack_state.selected_node_kind = Some("NeuralRadianceView".to_string());
        rack_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "NeuralRadianceView".to_string(),
            display_name: "Continuous Radiance Field".to_string(),
            is_bypassed: false,
        }];

        inspector_state.selected_node_kind = Some("NeuralVocalStylizerView".to_string());
        inspector_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "NeuralVocalStylizerView".to_string(),
            display_name: "Neural Vocal Stylizer".to_string(),
            is_bypassed: false,
        }];

        let bus = ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(
                    ui,
                    &mut rack_state,
                    None,
                    Some(&bus),
                    1,
                );
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });

        assert_eq!(rack_state.chain_devices.len(), 1);
        assert_eq!(inspector_state.chain_devices.len(), 1);
    }

    #[test]
    fn test_turn86_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let rad_act = palette.actions.iter().find(|a| a.action_id == "open_neural_radiance_hud");
        assert!(rad_act.is_some());
        assert_eq!(rad_act.unwrap().category, "Spatial");

        let voc_act = palette.actions.iter().find(|a| a.action_id == "open_neural_vocal_stylizer_hud");
        assert!(voc_act.is_some());
        assert_eq!(voc_act.unwrap().category, "Synthesis");
    }
}
