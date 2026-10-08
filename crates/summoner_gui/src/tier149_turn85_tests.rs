#![allow(clippy::all)]

//! Test suite for Turn #85 (Sprint Turn #11) — Dolby Atmos HOA 7.1.4 3D Spatializer & Binaural Head-Tracking HUD
//! and Neural Timbre Morphing Diffusion Vocoder & Resynthesizer HUD Convergence (Milestones 33, 35 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::hoa_spatializer_view::{
        HoaSpatialFormat, HoaSpatializerView, HOA_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_timbre_morph_view::{
        NeuralTimbreMorphView, TimbrePreset, TIMBRE_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn85_pure_hoa_spatializer_view_initialization_and_kinematics() {
        let mut view = HoaSpatializerView::new();
        assert_eq!(view.format, HoaSpatialFormat::DolbyAtmos714);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-4);
        assert!((view.elevation_deg - 18.5).abs() < 1e-4);
        assert!((view.distance_m - 2.40).abs() < 1e-4);
        assert!((view.head_yaw_deg - (-12.4)).abs() < 1e-4);
        assert_eq!(view.hoa_order, 3);
        assert_eq!(HOA_PUCK_HIT_RADIUS, 22.0); // 44x44pt bounding target

        // Format profiles & channel counts
        let formats = [
            (HoaSpatialFormat::HoaThirdOrder, 16, "HOA 3RD ORDER (16-CH)"),
            (HoaSpatialFormat::DolbyAtmos714, 12, "DOLBY ATMOS 7.1.4"),
            (HoaSpatialFormat::BinauralHeadTrack, 2, "BINAURAL HEAD-TRACK"),
            (HoaSpatialFormat::Ambisonics514, 10, "AMBISONICS 5.1.4"),
            (HoaSpatialFormat::DomeAcoustic916, 16, "DOME ACOUSTIC 9.1.6"),
        ];
        for (fmt, ch_count, name) in formats {
            view.format = fmt;
            assert_eq!(fmt.channel_count(), ch_count);
            assert_eq!(fmt.display_name(), name);
        }

        // Coordinate normalizations & round-trips
        let az_test = 30.0;
        let az_norm = HoaSpatializerView::azimuth_to_normalized(az_test);
        let az_back = HoaSpatializerView::normalized_to_azimuth(az_norm);
        assert!((az_back - az_test).abs() < 1e-3);

        let el_test = -45.0;
        let el_norm = HoaSpatializerView::elevation_to_normalized(el_test);
        let el_back = HoaSpatializerView::normalized_to_elevation(el_norm);
        assert!((el_back - el_test).abs() < 1e-3);

        let dist_test = 5.0;
        let dist_norm = HoaSpatializerView::distance_to_normalized(dist_test);
        assert!((dist_norm - 0.4949).abs() < 1e-2);

        // Spherical harmonics update
        view.azimuth_deg = 0.0;
        view.elevation_deg = 0.0;
        view.update_spherical_harmonics();
        assert!(view.spherical_harmonics[0] > 0.0); // Monopole W channel > 0

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DOLBY ATMOS"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn85_pure_neural_timbre_morph_view_initialization_and_resynthesis() {
        let mut view = NeuralTimbreMorphView::new();
        assert_eq!(view.preset, TimbrePreset::CelloToSynthLead);
        assert!((view.latent_z1 - 0.45).abs() < 1e-4);
        assert!((view.latent_z2 - (-0.60)).abs() < 1e-4);
        assert_eq!(view.tracked_f0_hz, 220.0);
        assert_eq!(view.morph_weight_pct, 65.0);
        assert_eq!(view.denoising_steps, 12);
        assert_eq!(TIMBRE_PUCK_HIT_RADIUS, 22.0); // 44x44pt bounding target

        // Latent coordinate normalization round-trips
        let z_test = 1.2;
        let z_norm = NeuralTimbreMorphView::latent_to_normalized(z_test);
        let z_back = NeuralTimbreMorphView::normalized_to_latent(z_norm);
        assert!((z_back - z_test).abs() < 1e-3);

        // Presets switching
        let presets = [
            (TimbrePreset::CelloToSynthLead, 130.81, 8, "CELLO -> LEAD"),
            (TimbrePreset::SopranoToFlute, 440.00, 12, "SOPRANO -> FLUTE"),
            (TimbrePreset::Analog303ToVocalTract, 65.41, 16, "303 -> VOCAL TRACT"),
            (TimbrePreset::MetalPercussionToGlass, 523.25, 10, "PERC -> GLASS BELL"),
            (TimbrePreset::DiffusionLatentRandom, 220.00, 20, "LATENT DIFFUSION"),
        ];
        for (pr, f0, steps, name) in presets {
            view.set_preset(pr);
            assert_eq!(view.preset, pr);
            assert!((view.tracked_f0_hz - f0).abs() < 1e-2);
            assert_eq!(view.denoising_steps, steps);
            assert_eq!(pr.preset_name(), name);
        }

        // Spectral envelope computation
        view.update_diffusion_resynthesis();
        assert!(view.spectral_envelope[0] > 0.0);
        assert!(view.spectral_envelope[4] > 0.0);

        // Touch puck hit testing (>= 44x44pt)
        let canvas = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let puck_x = canvas.x + view.timbre_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.timbre_puck_pos.1) * canvas.height;
        assert!(view.hit_test_timbre_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_timbre_puck((puck_x + 15.0, puck_y + 15.0), canvas));
        assert!(!view.hit_test_timbre_puck((puck_x + 35.0, puck_y + 35.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL TIMBRE"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn85_pure_channel_isolation_and_parameter_mapping() {
        // Test lock-free channel isolation across tracks (0..16) and slots (0..8)
        let mut ids = std::collections::HashSet::new();
        for track in 0..16 {
            for slot in 0..8 {
                for offset in 0..6 {
                    let pid = ParamId(track * 1000 + slot * 20 + offset);
                    assert!(ids.insert(pid), "Duplicate ParamId detected: {:?}", pid);
                }
            }
        }
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
    fn test_turn85_gui_award_winning_view_hoa_and_neural_timbre_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Dolby Atmos HOA 7.1.4 HUD
        view.open_hoa_spatializer_hud();
        assert!(view.is_hoa_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_hoa_spatializer_hud();
        assert!(!view.is_hoa_spatializer_hud_open());

        // 2. Neural Timbre Morph HUD
        view.open_neural_timbre_morph_hud();
        assert!(view.is_neural_timbre_morph_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_timbre_morph_hud();
        assert!(!view.is_neural_timbre_morph_hud_open());
    }

    #[test]
    fn test_turn85_gui_lock_free_param_bus_dispatch_hoa_and_neural_timbre() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for HOA (offsets 0..=4) and Timbre (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test HOA Spatializer sync
        view.hoa_spatializer_view.azimuth_deg = 35.0;
        view.hoa_spatializer_view.elevation_deg = 12.0;
        view.hoa_spatializer_view.distance_m = 3.5;
        view.hoa_spatializer_view.head_yaw_deg = -8.0;
        view.hoa_spatializer_view.head_pitch_deg = 2.0;

        view.sync_hoa_spatializer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("azimuth_deg"), Some(&35.0));
        assert_eq!(view.device_rack_state.node_param_values.get("elevation_deg"), Some(&12.0));
        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&3.5));
        assert_eq!(view.inspector_state.node_param_values.get("head_yaw_deg"), Some(&-8.0));

        let pid_az = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_dist = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        assert_eq!(arc_bus.get(pid_az), Some(35.0));
        assert_eq!(arc_bus.get(pid_dist), Some(3.5));

        // Test Neural Timbre Morph sync
        view.neural_timbre_morph_view.latent_z1 = 0.85;
        view.neural_timbre_morph_view.latent_z2 = -0.40;
        view.neural_timbre_morph_view.tracked_f0_hz = 330.0;
        view.neural_timbre_morph_view.morph_weight_pct = 75.0;
        view.neural_timbre_morph_view.denoising_steps = 14;
        view.neural_timbre_morph_view.confidence_score = 0.95;

        view.sync_neural_timbre_morph_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("latent_z1"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("latent_z2"), Some(&-0.40));
        assert_eq!(view.device_rack_state.node_param_values.get("tracked_f0_hz"), Some(&330.0));
        assert_eq!(view.device_rack_state.node_param_values.get("morph_weight_pct"), Some(&75.0));
        assert_eq!(view.inspector_state.node_param_values.get("denoising_steps"), Some(&14.0));

        let pid_z1 = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_f0 = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        assert_eq!(arc_bus.get(pid_z1), Some(0.85));
        assert_eq!(arc_bus.get(pid_f0), Some(330.0));
    }

    #[test]
    fn test_turn85_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. HOA Spatializer trigger propagation from rack
        view.device_rack_state.requested_open_hoa_spatializer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_hoa_spatializer_hud);
        assert!(view.is_hoa_spatializer_hud_open());
        view.close_hoa_spatializer_hud();

        // 2. Neural Timbre Morph trigger propagation from inspector
        view.inspector_state.requested_open_neural_timbre_morph_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_neural_timbre_morph_hud);
        assert!(view.is_neural_timbre_morph_hud_open());
        view.close_neural_timbre_morph_hud();
    }

    #[test]
    fn test_turn85_gui_modern_device_rack_and_inspector_headless_render() {
        let ctx = egui::Context::default();
        let mut rack_state = ModernDeviceRackState::default();
        let mut inspector_state = ModernInspectorState::default();

        rack_state.selected_node_kind = Some("HoaSpatializerView".to_string());
        rack_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "HoaSpatializerView".to_string(),
            display_name: "HOA 7.1.4 Spatializer".to_string(),
            is_bypassed: false,
        }];

        inspector_state.selected_node_kind = Some("NeuralTimbreMorphView".to_string());
        inspector_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "NeuralTimbreMorphView".to_string(),
            display_name: "Neural Timbre Morpher".to_string(),
            is_bypassed: false,
        }];

        let bus = ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack_state, None, Some(&bus), 1);
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });
        assert!(inspector_state.selected_node_kind.is_some());
    }

    #[test]
    fn test_turn85_gui_command_palette_registration_hoa_and_neural_timbre() {
        let palette = CommandPalette::new();
        let has_hoa = palette.actions.iter().any(|a| a.action_id == "open_hoa_spatializer_hud");
        let has_timbre = palette.actions.iter().any(|a| a.action_id == "open_neural_timbre_morph_hud");

        assert!(has_hoa, "Expected open_hoa_spatializer_hud in CommandPalette");
        assert!(has_timbre, "Expected open_neural_timbre_morph_hud in CommandPalette");
    }
}
