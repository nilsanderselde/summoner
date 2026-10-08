#![allow(clippy::all)]

//! Test suite for Turn #91 (Sprint Turn #17) — Neural Diffusion Audio Inpainter HUD
//! and Neural Timbre Transfer Morphing Resynthesizer HUD Convergence (Milestones 33 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_inpaint_view::{
        NeuralInpaintView, InpaintModel, INPAINT_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_timbre_view::{
        NeuralTimbreView, TimbreModel, NEURAL_TIMBRE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn91_pure_neural_inpaint_initialization_and_kinematics() {
        let mut view = NeuralInpaintView::new();
        assert_eq!(view.model, InpaintModel::DropoutRepair);
        assert!((view.mask_center_time_ms - 250.0).abs() < 1e-3);
        assert!((view.mask_center_freq_hz - 1000.0).abs() < 1e-3);
        assert!((view.mask_duration_ms - 45.0).abs() < 1e-3);
        assert!((view.mask_bandwidth_oct - 1.5).abs() < 1e-3);
        assert_eq!(view.diffusion_steps, 25);
        assert!((view.guidance_scale - 4.5).abs() < 1e-3);
        assert_eq!(INPAINT_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & names
        let models = [
            (InpaintModel::DropoutRepair, 25, 4.5, "DROPOUT REPAIR (GAP SYNTHESIS)"),
            (InpaintModel::SpectralDeClick, 10, 2.5, "SPECTRAL DE-CLICK (TRANSIENT)"),
            (InpaintModel::PlosiveThump, 18, 3.5, "PLOSIVE THUMP (LF RESTORE)"),
            (InpaintModel::MicClipRestore, 20, 5.0, "MIC CLIP RESTORE (DECLIP)"),
            (InpaintModel::StemBleedEraser, 32, 7.0, "STEM BLEED ERASER (SPILL)"),
        ];
        for (m, exp_steps, exp_guide, name) in models {
            view.set_model(m);
            assert_eq!(view.model, m);
            assert_eq!(view.diffusion_steps, exp_steps);
            assert!((view.guidance_scale - exp_guide).abs() < 1e-3);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips for Time
        let time_test = 320.0;
        let time_norm = NeuralInpaintView::time_to_normalized(time_test);
        let time_back = NeuralInpaintView::normalized_to_time(time_norm);
        assert!((time_back - time_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Frequency (log scale)
        let freq_test = 2500.0;
        let freq_norm = NeuralInpaintView::freq_to_normalized(freq_test);
        let freq_back = NeuralInpaintView::normalized_to_freq(freq_norm);
        assert!((freq_back - freq_test).abs() < 1.0);

        // Mask center evaluation
        view.mask_center_time_ms = 250.0;
        view.mask_center_freq_hz = 1000.0;
        let center_weight = view.evaluate_inpaint_mask(250.0, 1000.0);
        assert!((center_weight - 1.0).abs() < 1e-3);
        let far_weight = view.evaluate_inpaint_mask(450.0, 8000.0);
        assert_eq!(far_weight, 0.0);

        // Puck hit testing
        let canvas = Rect::new(50.0, 50.0, 150.0, 150.0);
        let puck_x = canvas.x + view.inpaint_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.inpaint_puck_pos.1) * canvas.height;
        assert!(view.hit_test_inpaint_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_inpaint_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL DIFFUSION AUDIO INPAINTER & GENERATIVE SPECTRAL REPAIR HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn91_pure_neural_timbre_initialization_and_flow_simulation() {
        let mut view = NeuralTimbreView::new();
        assert_eq!(view.model, TimbreModel::VocalFormantMorph);
        assert!((view.latent_coord.0 - 0.45).abs() < 1e-3);
        assert!((view.latent_coord.1 - -0.30).abs() < 1e-3);
        assert!((view.flow_rate_hz - 1.20).abs() < 1e-3);
        assert!((view.residual_blend - 0.20).abs() < 1e-3);
        assert!(view.is_full_neural_mode);
        assert_eq!(NEURAL_TIMBRE_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & nominal parameters
        let models = [
            (TimbreModel::VocalFormantMorph, 1.20, 0.20, "VOCAL FORMANT MORPH"),
            (TimbreModel::CelloResonanceFlow, 0.65, 0.35, "CELLO ACOUSTIC RESONANCE"),
            (TimbreModel::AnalogMoogLead, 2.50, 0.10, "ANALOG MOOG 24DB LADDER"),
            (TimbreModel::GlassMalletBell, 0.40, 0.50, "GLASS MALLET HARMONIC BELL"),
            (TimbreModel::AlienBiomorphic, 1.85, 0.05, "ALIEN BIOMORPHIC HYBRID"),
        ];
        for (m, exp_flow, exp_blend, name) in models {
            view.set_model(m);
            assert_eq!(view.model, m);
            assert!((view.flow_rate_hz - exp_flow).abs() < 1e-3);
            assert!((view.residual_blend - exp_blend).abs() < 1e-3);
            assert_eq!(m.model_name(), name);
        }

        // Coordinate normalizations & round-trips for Latent Coordinate
        let coord_test = 0.85;
        let coord_norm = NeuralTimbreView::coord_to_normalized(coord_test);
        let coord_back = NeuralTimbreView::normalized_to_coord(coord_norm);
        assert!((coord_back - coord_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Flow Rate
        let flow_test = 4.25;
        let flow_norm = NeuralTimbreView::flow_to_normalized(flow_test);
        let flow_back = NeuralTimbreView::normalized_to_flow(flow_norm);
        assert!((flow_back - flow_test).abs() < 1e-3);

        // Flow velocity and envelope evaluation
        let (vx, vy) = view.evaluate_flow_velocity(0.5, -0.5);
        assert!(vx.is_finite());
        assert!(vy.is_finite());
        let (src_mag, out_mag) = view.evaluate_spectral_envelope(0.5);
        assert!(src_mag >= 0.0);
        assert!(out_mag >= 0.0);

        // Puck hit testing
        let canvas = Rect::new(40.0, 40.0, 160.0, 160.0);
        let puck_x = canvas.x + view.timbre_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.timbre_puck_pos.1) * canvas.height;
        assert!(view.hit_test_timbre_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_timbre_puck((puck_x + 55.0, puck_y + 55.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL TIMBRE TRANSFER MORPHING RESYNTHESIZER HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn91_gui_modal_window_lifecycles_inpaint_and_timbre() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Neural Inpainter HUD
        assert!(!view.is_neural_inpaint_hud_open());
        view.open_neural_inpaint_hud();
        assert!(view.is_neural_inpaint_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_inpaint_hud();
        assert!(!view.is_neural_inpaint_hud_open());

        // 2. Neural Timbre HUD
        assert!(!view.is_neural_timbre_hud_open());
        view.open_neural_timbre_hud();
        assert!(view.is_neural_timbre_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_timbre_hud();
        assert!(!view.is_neural_timbre_hud_open());
    }

    #[test]
    fn test_turn91_gui_lock_free_param_bus_dispatch_inpaint_and_timbre() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Inpainter (offsets 0..=5) and Timbre (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Inpainter sync
        view.neural_inpaint_view.mask_center_time_ms = 310.0;
        view.neural_inpaint_view.mask_center_freq_hz = 2400.0;
        view.neural_inpaint_view.mask_duration_ms = 60.0;
        view.neural_inpaint_view.mask_bandwidth_oct = 2.0;
        view.neural_inpaint_view.diffusion_steps = 35;
        view.neural_inpaint_view.guidance_scale = 6.5;

        view.sync_neural_inpaint_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("mask_center_time_ms"), Some(&310.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mask_center_freq_hz"), Some(&2400.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mask_duration_ms"), Some(&60.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mask_bandwidth_oct"), Some(&2.0));
        assert_eq!(view.device_rack_state.node_param_values.get("diffusion_steps"), Some(&35.0));
        assert_eq!(view.device_rack_state.node_param_values.get("guidance_scale"), Some(&6.5));

        let pid_time = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_freq = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_time), Some(310.0));
        assert_eq!(arc_bus.get(pid_freq), Some(2400.0));

        // Test Timbre sync
        view.neural_timbre_view.latent_coord = (0.75, -0.60);
        view.neural_timbre_view.flow_rate_hz = 3.25;
        view.neural_timbre_view.residual_blend = 0.45;
        view.neural_timbre_view.is_full_neural_mode = false;
        view.neural_timbre_view.timbre_convergence_pct = 98.7;

        view.sync_neural_timbre_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("latent_z1"), Some(&0.75));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_z2"), Some(&-0.60));
        assert_eq!(view.device_rack_state.node_param_values.get("flow_rate_hz"), Some(&3.25));
        assert_eq!(view.device_rack_state.node_param_values.get("residual_blend"), Some(&0.45));
        assert_eq!(view.device_rack_state.node_param_values.get("is_full_neural_mode"), Some(&0.0));
        assert_eq!(view.device_rack_state.node_param_values.get("timbre_convergence_pct"), Some(&98.7));

        assert_eq!(view.inspector_state.node_param_values.get("latent_z1"), Some(&0.75));
        assert_eq!(view.inspector_state.node_param_values.get("latent_z2"), Some(&-0.60));

        let pid_z1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_z2 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_z1), Some(0.75));
        assert_eq!(arc_bus.get(pid_z2), Some(-0.60));
    }

    #[test]
    fn test_turn91_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Inpaint trigger propagation from rack
        view.device_rack_state.requested_open_neural_inpaint_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_inpaint_hud);
        assert!(view.is_neural_inpaint_hud_open());
        view.close_neural_inpaint_hud();

        // 2. Timbre trigger propagation from inspector
        view.inspector_state.requested_open_neural_timbre_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_timbre_hud);
        assert!(view.is_neural_timbre_hud_open());
        view.close_neural_timbre_hud();
    }

    #[test]
    fn test_turn91_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let inpaint_action = palette.actions.iter().find(|a| a.action_id == "open_neural_inpaint_hud");
        assert!(inpaint_action.is_some(), "open_neural_inpaint_hud must be registered in command palette");
        let inp = inpaint_action.unwrap();
        assert_eq!(inp.category, "Restoration");

        let timbre_action = palette.actions.iter().find(|a| a.action_id == "open_neural_timbre_hud");
        assert!(timbre_action.is_some(), "open_neural_timbre_hud must be registered in command palette");
        let tim = timbre_action.unwrap();
        assert_eq!(tim.category, "Synthesis");
    }
}
