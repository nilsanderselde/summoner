#![allow(clippy::all)]

//! Test suite for Turn #90 (Sprint Turn #16) — Neural Audio Super-Resolution HUD
//! and Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier HUD Convergence (Milestones 33 & 36).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::neural_super_resolution_view::{
        NeuralSuperResolutionView, SuperResolutionProfile, SUPER_RES_PUCK_HIT_RADIUS,
    };
    use crate::views::nam_amp_view::{
        NamAmpProfile, NamAmpView, NAM_AMP_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn90_pure_neural_super_resolution_initialization_and_kinematics() {
        let mut view = NeuralSuperResolutionView::new();
        assert_eq!(view.profile, SuperResolutionProfile::WaveletTransientAir);
        assert!((view.hf_boost_db - 4.5).abs() < 1e-3);
        assert!((view.spectral_continuity - 0.75).abs() < 1e-3);
        assert!((view.harmonic_density - 0.60).abs() < 1e-3);
        assert!((view.cutoff_frequency_hz - 12000.0).abs() < 1e-3);
        assert!((view.ultrasonic_air_db - 3.5).abs() < 1e-3);
        assert_eq!(SUPER_RES_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & names
        let profiles = [
            (SuperResolutionProfile::WaveletTransientAir, "TRANSIENT WAVELET AIR"),
            (SuperResolutionProfile::VocalSparkleLPC, "VOCAL SPARKLE LPC"),
            (SuperResolutionProfile::SubHarmonicOctave, "SUBHARMONIC OCTAVE"),
            (SuperResolutionProfile::DiffusionSpectrogram, "DIFFUSION SPECTROGRAM"),
            (SuperResolutionProfile::ArchiveRestoration, "ARCHIVE RESTORATION"),
        ];
        for (p, name) in profiles {
            view.set_profile(p);
            assert_eq!(view.profile, p);
            assert_eq!(p.profile_name(), name);
        }

        // Coordinate normalizations & round-trips for HF Boost
        let boost_test = 7.5;
        let boost_norm = NeuralSuperResolutionView::boost_to_normalized(boost_test);
        let boost_back = NeuralSuperResolutionView::normalized_to_boost(boost_norm);
        assert!((boost_back - boost_test).abs() < 1e-2);

        // Coordinate normalizations & round-trips for Continuity
        let cont_test = 0.88;
        let cont_norm = NeuralSuperResolutionView::continuity_to_normalized(cont_test);
        let cont_back = NeuralSuperResolutionView::normalized_to_continuity(cont_norm);
        assert!((cont_back - cont_test).abs() < 1e-3);

        // Spectrum simulation evaluation (16 bands)
        assert_eq!(view.spectrum_bands.len(), 16);
        for &val in view.spectrum_bands.iter() {
            assert!(val > 0.0);
        }

        // Puck hit testing
        let canvas = Rect::new(50.0, 50.0, 150.0, 150.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_super_res_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_super_res_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL AUDIO SUPER-RESOLUTION & BANDWIDTH EXTENSION HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn90_pure_nam_amp_initialization_and_simulation() {
        let mut view = NamAmpView::new();
        assert_eq!(view.profile, NamAmpProfile::BritishCrunchPlexi);
        assert!((view.drive - 2.5).abs() < 1e-3);
        assert!((view.output_gain - 0.95).abs() < 1e-3);
        assert!((view.gate_threshold_db - -60.0).abs() < 1e-3);
        assert!((view.cabinet_air - 0.35).abs() < 1e-3);
        assert!((view.tube_sag - 0.35).abs() < 1e-3);
        assert!((view.presence_boost_db - 1.5).abs() < 1e-3);
        assert_eq!(NAM_AMP_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Architecture profiles & nominal parameters
        let profiles = [
            (NamAmpProfile::CleanTube1965, 1.2, -72.0, "VINTAGE CLEAN 6V6"),
            (NamAmpProfile::BritishCrunchPlexi, 2.5, -60.0, "BRITISH PLEXI EL34"),
            (NamAmpProfile::ModernHighGainLead, 4.2, -48.0, "MODERN RECTO 12AX7"),
            (NamAmpProfile::BassSlapAmpegSVT, 2.0, -65.0, "ALL-TUBE BASS SVT"),
            (NamAmpProfile::BoutiqueClassAOverdrive, 3.0, -55.0, "BOUTIQUE CLASS-A EL84"),
        ];
        for (p, exp_drive, exp_gate, name) in profiles {
            view.set_profile(p);
            assert_eq!(view.profile, p);
            assert!((view.drive - exp_drive).abs() < 1e-3);
            assert!((view.gate_threshold_db - exp_gate).abs() < 1e-3);
            assert_eq!(p.profile_name(), name);
        }

        // Coordinate normalizations & round-trips
        let drive_test = 3.6;
        let drive_norm = NamAmpView::drive_to_normalized(drive_test);
        let drive_back = NamAmpView::normalized_to_drive(drive_norm);
        assert!((drive_back - drive_test).abs() < 1e-3);

        let gate_test = -45.0;
        let gate_norm = NamAmpView::gate_to_normalized(gate_test);
        let gate_back = NamAmpView::normalized_to_gate(gate_norm);
        assert!((gate_back - gate_test).abs() < 1e-3);

        // Saturation curve and dilation activity
        assert_eq!(view.saturation_curve.len(), 16);
        assert_eq!(view.wavenet_dilation_activity.len(), 4);

        // Puck hit testing
        let canvas = Rect::new(40.0, 40.0, 160.0, 160.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_amp_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_amp_puck((puck_x + 55.0, puck_y + 55.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("NEURAL AMP MODELER (NAM) WAVENET GUITAR & BASS AMPLIFIER HUD"));
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
    fn test_turn90_gui_modal_window_lifecycles_super_res_and_nam_amp() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Neural Super-Resolution HUD
        assert!(!view.is_neural_super_resolution_hud_open());
        view.open_neural_super_resolution_hud();
        assert!(view.is_neural_super_resolution_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_neural_super_resolution_hud();
        assert!(!view.is_neural_super_resolution_hud_open());

        // 2. NAM Amp HUD
        assert!(!view.is_nam_amp_hud_open());
        view.open_nam_amp_hud();
        assert!(view.is_nam_amp_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_nam_amp_hud();
        assert!(!view.is_nam_amp_hud_open());
    }

    #[test]
    fn test_turn90_gui_lock_free_param_bus_dispatch_super_res_and_nam_amp() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Super-Res (offsets 0..=5) and NAM Amp (offsets 0..=5)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Super-Resolution sync
        view.neural_super_resolution_view.hf_boost_db = 6.2;
        view.neural_super_resolution_view.spectral_continuity = 0.92;
        view.neural_super_resolution_view.harmonic_density = 0.85;
        view.neural_super_resolution_view.cutoff_frequency_hz = 14500.0;
        view.neural_super_resolution_view.ultrasonic_air_db = 4.2;
        view.neural_super_resolution_view.reconstruction_quality_pct = 99.2;

        view.sync_neural_super_resolution_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("hf_boost_db"), Some(&6.2));
        assert_eq!(view.device_rack_state.node_param_values.get("spectral_continuity"), Some(&0.92));
        assert_eq!(view.device_rack_state.node_param_values.get("harmonic_density"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("cutoff_frequency_hz"), Some(&14500.0));
        assert_eq!(view.device_rack_state.node_param_values.get("ultrasonic_air_db"), Some(&4.2));
        assert_eq!(view.device_rack_state.node_param_values.get("reconstruction_quality_pct"), Some(&99.2));

        let pid_boost = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_cont = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_boost), Some(6.2));
        assert_eq!(arc_bus.get(pid_cont), Some(0.92));

        // Test NAM Amp sync
        view.nam_amp_view.drive = 3.8;
        view.nam_amp_view.output_gain = 1.15;
        view.nam_amp_view.gate_threshold_db = -52.0;
        view.nam_amp_view.cabinet_air = 0.45;
        view.nam_amp_view.tube_sag = 0.40;
        view.nam_amp_view.presence_boost_db = 2.5;

        view.sync_nam_amp_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("drive"), Some(&3.8));
        assert_eq!(view.device_rack_state.node_param_values.get("output_gain"), Some(&1.15));
        assert_eq!(view.device_rack_state.node_param_values.get("gate_threshold_db"), Some(&-52.0));
        assert_eq!(view.inspector_state.node_param_values.get("cabinet_air"), Some(&0.45));
        assert_eq!(view.inspector_state.node_param_values.get("tube_sag"), Some(&0.40));
        assert_eq!(view.inspector_state.node_param_values.get("presence_boost_db"), Some(&2.5));

        let pid_drive = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_gain = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_drive), Some(3.8));
        assert_eq!(arc_bus.get(pid_gain), Some(1.15));
    }

    #[test]
    fn test_turn90_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Super-Resolution trigger propagation from rack
        view.device_rack_state.requested_open_neural_super_resolution_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_neural_super_resolution_hud);
        assert!(view.is_neural_super_resolution_hud_open());
        view.close_neural_super_resolution_hud();

        // 2. NAM Amp trigger propagation from inspector
        view.inspector_state.requested_open_nam_amp_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_nam_amp_hud);
        assert!(view.is_nam_amp_hud_open());
        view.close_nam_amp_hud();
    }

    #[test]
    fn test_turn90_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let sr_action = palette.actions.iter().find(|a| a.action_id == "open_neural_super_resolution_hud");
        assert!(sr_action.is_some(), "open_neural_super_resolution_hud must be registered in command palette");
        let sr = sr_action.unwrap();
        assert_eq!(sr.category, "Restoration");

        let nam_action = palette.actions.iter().find(|a| a.action_id == "open_nam_amp_hud");
        assert!(nam_action.is_some(), "open_nam_amp_hud must be registered in command palette");
        let nam = nam_action.unwrap();
        assert_eq!(nam.category, "Amplification");
    }
}
