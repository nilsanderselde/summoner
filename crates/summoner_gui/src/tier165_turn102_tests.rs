#![allow(clippy::all)]

//! Test suite for Turn #102 (Sprint Turn #28) — Dynamic Spectral De-Esser & 64-Band Spectral Matching Linear-Phase EQ HUDs,
//! Multi-Band Dynamic Resonance Suppressor, and 6-Operator FM Matrix HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::spectral_deesser_view::{
        DeEsserMode, SpectralDeEsserView, DEESSER_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_matching_eq_view::{
        MatchingProfile, SpectralMatchingEqView, MATCH_EQ_NUM_BANDS, MATCH_EQ_PUCK_HIT_RADIUS,
    };
    use crate::views::resonance_suppressor_view::{
        ResonanceSuppressorView, SuppressorMode, RESONANCE_NODE_HIT_RADIUS,
    };
    use crate::views::fm_matrix_view::{
        FmAlgorithm, FmMatrixView, FM_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn102_pure_spectral_deesser_initialization_and_kinematics() {
        let mut view = SpectralDeEsserView::new();
        assert_eq!(view.mode, DeEsserMode::SplitBand);
        assert!((view.frequency_hz - 6500.0).abs() < 1e-3);
        assert!((view.threshold_db - (-24.0)).abs() < 1e-3);
        assert!((view.reduction_range_db - 12.0).abs() < 1e-3);
        assert!((view.bandwidth_q - 1.8).abs() < 1e-3);
        assert!(!view.audition_sibilance);
        assert_eq!(DEESSER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (DeEsserMode::SplitBand, "Split Band"),
            (DeEsserMode::WideBand, "Wide Band"),
            (DeEsserMode::DynamicNotch, "Dynamic Notch"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Frequency [2000.0..16000.0 Hz]
        let f_test = 7200.0;
        let f_norm = SpectralDeEsserView::freq_to_normalized(f_test);
        let f_back = SpectralDeEsserView::normalized_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        // Threshold [-60.0..0.0 dB]
        let th_test = -18.5;
        let th_norm = SpectralDeEsserView::db_to_normalized(th_test);
        let th_back = SpectralDeEsserView::normalized_to_db(th_norm);
        assert!((th_back - th_test).abs() < 1e-3);

        // Bandwidth Q [0.5..5.0]
        let q_test = 3.4;
        let q_norm = SpectralDeEsserView::bandwidth_to_normalized(q_test);
        let q_back = SpectralDeEsserView::normalized_to_bandwidth(q_norm);
        assert!((q_back - q_test).abs() < 1e-3);

        // Reduction Range [0.0..30.0 dB]
        let red_test = 16.5;
        let red_norm = SpectralDeEsserView::reduction_to_normalized(red_test);
        let red_back = SpectralDeEsserView::normalized_to_reduction(red_norm);
        assert!((red_back - red_test).abs() < 1e-3);

        // Attack [1.0..50.0 ms]
        let atk_test = 12.5;
        let atk_norm = SpectralDeEsserView::attack_to_normalized(atk_test);
        let atk_back = SpectralDeEsserView::normalized_to_attack(atk_norm);
        assert!((atk_back - atk_test).abs() < 1e-3);

        // Release [10.0..500.0 ms]
        let rel_test = 120.0;
        let rel_norm = SpectralDeEsserView::release_to_normalized(rel_test);
        let rel_back = SpectralDeEsserView::normalized_to_release(rel_norm);
        assert!((rel_back - rel_test).abs() < 1e-3);

        // Attenuation curve evaluations
        let att_center = view.evaluate_attenuation_response(view.frequency_hz);
        let att_far = view.evaluate_attenuation_response(200.0);
        assert!(att_center > 0.0);
        assert!(att_center > att_far);

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.sibilance_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.sibilance_puck_pos.1) * canvas.height;
        assert!(view.hit_test_sibilance_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_sibilance_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SPECTRAL DE-ESSER"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn102_pure_spectral_matching_eq_initialization_and_kinematics() {
        let mut view = SpectralMatchingEqView::new();
        assert_eq!(view.profile, MatchingProfile::ReferenceTrackMatch);
        assert_eq!(MATCH_EQ_NUM_BANDS, 64);
        assert_eq!(view.source_spectrum_db.len(), 64);
        assert_eq!(view.target_spectrum_db.len(), 64);
        assert_eq!(view.matched_gain_db.len(), 64);
        assert!((view.match_amount_percent - 75.0).abs() < 1e-3);
        assert!((view.smoothing_semitones - 4.5).abs() < 1e-3);
        assert!((view.gain_limit_db - 12.0).abs() < 1e-3);
        assert!(view.linear_phase);
        assert_eq!(MATCH_EQ_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Profile switching & names
        let profiles = [
            (MatchingProfile::ReferenceTrackMatch, "Reference Track Match"),
            (MatchingProfile::PinkNoiseMasterTarget, "Pink Noise Master Target"),
            (MatchingProfile::LoudnessBalancedTarget, "Loudness Balanced Target"),
            (MatchingProfile::WarmAnalogMasterTilt, "Warm Analog Master Tilt"),
        ];

        for (profile, expected_name) in profiles {
            view.set_profile(profile);
            assert_eq!(view.profile, profile);
            assert_eq!(profile.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Amount [0.0..100.0 %]
        let amt_test = 68.0;
        let amt_norm = SpectralMatchingEqView::amount_to_normalized(amt_test);
        let amt_back = SpectralMatchingEqView::normalized_to_amount(amt_norm);
        assert!((amt_back - amt_test).abs() < 1e-3);

        // Smoothing [1.0..24.0 semitones]
        let sm_test = 8.5;
        let sm_norm = SpectralMatchingEqView::smoothing_to_normalized(sm_test);
        let sm_back = SpectralMatchingEqView::normalized_to_smoothing(sm_norm);
        assert!((sm_back - sm_test).abs() < 1e-3);

        // Limit [3.0..24.0 dB]
        let lim_test = 15.0;
        let lim_norm = SpectralMatchingEqView::limit_to_normalized(lim_test);
        let lim_back = SpectralMatchingEqView::normalized_to_limit(lim_norm);
        assert!((lim_back - lim_test).abs() < 1e-3);

        // Frequency [20.0..20000.0 Hz]
        let f_test = 1000.0;
        let f_norm = SpectralMatchingEqView::freq_to_normalized(f_test);
        let f_back = SpectralMatchingEqView::normalized_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        // Band center frequencies
        let f_min = SpectralMatchingEqView::band_center_freq(0);
        let f_max = SpectralMatchingEqView::band_center_freq(63);
        assert!((f_min - 20.0).abs() < 1.0);
        assert!((f_max - 20000.0).abs() < 1.0);

        // Recompute match curve
        view.match_amount_percent = 50.0;
        view.gain_limit_db = 6.0;
        view.recompute_match_curve();
        for &val in &view.matched_gain_db {
            assert!(val >= -6.0 && val <= 6.0);
        }

        // Touch puck hit testing
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.eq_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.eq_puck_pos.1) * canvas.height;
        assert!(view.hit_test_eq_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_eq_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SPECTRAL MATCH EQ"));
        assert!(snap.contains("Bands:64"));
    }

    #[test]
    fn test_turn102_pure_resonance_suppressor_initialization_and_kinematics() {
        let mut view = ResonanceSuppressorView::new();
        assert_eq!(view.mode, SuppressorMode::FastSurgical);
        assert_eq!(view.nodes.len(), 4);
        assert_eq!(view.selected_node_idx, 1);
        assert_eq!(RESONANCE_NODE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (SuppressorMode::FastSurgical, "Surgical"),
            (SuppressorMode::MusicalSmooth, "Smooth"),
            (SuppressorMode::DeepHarmonicTame, "Harmonic"),
        ];

        for (mode, expected_name) in modes {
            view.mode = mode;
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips
        let f_test = 2500.0;
        let f_norm = ResonanceSuppressorView::freq_to_normalized(f_test);
        let f_back = ResonanceSuppressorView::normalized_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        let d_test = 14.5;
        let d_norm = ResonanceSuppressorView::depth_to_normalized(d_test);
        let d_back = ResonanceSuppressorView::normalized_to_depth(d_norm);
        assert!((d_back - d_test).abs() < 1e-3);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("RESONANCE SUPPRESSOR"));
    }

    #[test]
    fn test_turn102_pure_fm_matrix_initialization_and_kinematics() {
        let mut view = FmMatrixView::new();
        assert_eq!(view.algorithm, FmAlgorithm::Algo1LinearCascade);
        assert_eq!(view.operators.len(), 6);
        assert_eq!(FM_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Algorithm switching & names
        let algos = [
            (FmAlgorithm::Algo1LinearCascade, "Linear Cascade"),
            (FmAlgorithm::Algo5DualCascade, "Dual Cascade"),
            (FmAlgorithm::Algo16BranchModulator, "Branch Modulator"),
            (FmAlgorithm::Algo22ParallelCarrier, "Parallel Carrier"),
            (FmAlgorithm::Algo32PureAdditive, "Pure Additive"),
        ];

        for (algo, expected_name) in algos {
            view.set_algorithm(algo);
            assert_eq!(view.algorithm, algo);
            assert_eq!(algo.name(), expected_name);
        }

        // Coordinate normalizations & round-trips
        let mod_test = 4.2;
        let mod_norm = FmMatrixView::mod_index_to_normalized(mod_test);
        let mod_back = FmMatrixView::normalized_to_mod_index(mod_norm);
        assert!((mod_back - mod_test).abs() < 1e-3);

        let r_test = 2.0;
        let r_norm = FmMatrixView::ratio_to_normalized(r_test);
        let r_back = FmMatrixView::normalized_to_ratio(r_norm);
        assert!((r_back - r_test).abs() < 1e-3);

        // Bessel sidebands
        let sidebands = view.compute_bessel_sideband_energy(3.5);
        assert_eq!(sidebands.len(), 8);
        assert!(sidebands[0] > 0.0);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("+"));
        assert!(snap.contains("|"));
        assert!(snap.contains("C"));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::show_modern_inspector_with_context;

    #[test]
    fn test_turn102_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Multi-Band Dynamic Resonance Suppressor HUD
        assert!(!view.is_resonance_suppressor_hud_open());
        view.open_resonance_suppressor_hud();
        assert!(view.is_resonance_suppressor_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_resonance_suppressor_hud();
        assert!(!view.is_resonance_suppressor_hud_open());

        // 2. 6-Operator FM Matrix Synthesizer HUD
        assert!(!view.is_fm_matrix_hud_open());
        view.open_fm_matrix_hud();
        assert!(view.is_fm_matrix_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_fm_matrix_hud();
        assert!(!view.is_fm_matrix_hud_open());

        // 3. Dynamic Spectral De-Esser HUD
        assert!(!view.is_spectral_deesser_hud_open());
        view.open_spectral_deesser_hud();
        assert!(view.is_spectral_deesser_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_spectral_deesser_hud();
        assert!(!view.is_spectral_deesser_hud_open());

        // 4. 64-Band Dynamic Linear-Phase Spectral Matching EQ HUD
        assert!(!view.is_spectral_matching_eq_hud_open());
        view.open_spectral_matching_eq_hud();
        assert!(view.is_spectral_matching_eq_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_spectral_matching_eq_hud();
        assert!(!view.is_spectral_matching_eq_hud_open());
    }

    #[test]
    fn test_turn102_gui_lock_free_param_bus_dispatch_all_four_huds() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for slot parameters (offsets 0..=3)
        for offset in 0..=3 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // 1. Resonance Suppressor Sync
        view.resonance_suppressor_view.global_sensitivity = 80.0;
        view.resonance_suppressor_view.attack_ms = 4.5;
        if let Some(node) = view.resonance_suppressor_view.nodes.get_mut(1) {
            node.depth_db = 18.0;
            node.q = 12.0;
        }
        view.sync_resonance_suppressor_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("global_sensitivity"), Some(&80.0));
        assert_eq!(view.device_rack_state.node_param_values.get("depth_db"), Some(&18.0));
        assert_eq!(view.device_rack_state.node_param_values.get("bandwidth_q"), Some(&12.0));
        assert_eq!(view.device_rack_state.node_param_values.get("attack_ms"), Some(&4.5));

        let pid_sens = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_depth = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_q = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_atk = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_sens).unwrap() - 80.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_depth).unwrap() - 18.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_q).unwrap() - 12.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_atk).unwrap() - 4.5).abs() < 1e-3);

        // 2. FM Matrix Sync
        view.fm_matrix_view.algorithm = crate::views::fm_matrix_view::FmAlgorithm::Algo5DualCascade;
        view.fm_matrix_view.operators[5].output_level = 0.55;
        view.fm_matrix_view.matrix_puck_pos.0 = 0.72;
        view.fm_matrix_view.operators[0].ratio = 2.0;
        view.sync_fm_matrix_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("algorithm_idx"), Some(&1.0));
        assert_eq!(view.device_rack_state.node_param_values.get("op1_ratio"), Some(&2.0));

        let pid_algo = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_fb = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_rich = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_ratio = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_algo).unwrap() - 1.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_fb).unwrap() - 5.5).abs() < 1e-3);
        assert!((arc_bus.get(pid_rich).unwrap() - 72.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_ratio).unwrap() - 2.0).abs() < 1e-3);

        // 3. Spectral De-Esser Sync
        view.spectral_deesser_view.frequency_hz = 7500.0;
        view.spectral_deesser_view.threshold_db = -19.0;
        view.spectral_deesser_view.reduction_range_db = 15.0;
        view.spectral_deesser_view.bandwidth_q = 3.0;
        view.sync_spectral_deesser_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("frequency_hz"), Some(&7500.0));
        assert_eq!(view.device_rack_state.node_param_values.get("threshold_db"), Some(&-19.0));
        assert_eq!(view.device_rack_state.node_param_values.get("reduction_range_db"), Some(&15.0));
        assert_eq!(view.device_rack_state.node_param_values.get("bandwidth_q"), Some(&3.0));

        let pid_freq = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_thresh = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_red = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_de_q = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_freq).unwrap() - 7500.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_thresh).unwrap() - (-19.0)).abs() < 1e-3);
        assert!((arc_bus.get(pid_red).unwrap() - 15.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_de_q).unwrap() - 3.0).abs() < 1e-3);

        // 4. Spectral Matching EQ Sync
        view.spectral_matching_eq_view.match_amount_percent = 85.0;
        view.spectral_matching_eq_view.smoothing_semitones = 6.0;
        view.spectral_matching_eq_view.gain_limit_db = 9.0;
        view.spectral_matching_eq_view.linear_phase = true;
        view.sync_spectral_matching_eq_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("match_amount_percent"), Some(&85.0));
        assert_eq!(view.device_rack_state.node_param_values.get("smoothing_semitones"), Some(&6.0));
        assert_eq!(view.device_rack_state.node_param_values.get("gain_limit_db"), Some(&9.0));
        assert_eq!(view.device_rack_state.node_param_values.get("linear_phase"), Some(&1.0));

        let pid_amt = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_smooth = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_limit = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_phase = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_amt).unwrap() - 85.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_smooth).unwrap() - 6.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_limit).unwrap() - 9.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_phase).unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn102_gui_inspector_and_rack_trigger_buttons_all_four() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Inspector Trigger: Resonance Suppressor
        view.inspector_state.selected_node_kind = Some("ResonanceSuppressorNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(
                    ui,
                    &mut view.inspector_state,
                    view.live_param_bus.as_deref(),
                    1,
                );
            });
        });

        view.inspector_state.requested_open_resonance_suppressor_hud = true;
        assert!(!view.is_resonance_suppressor_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_resonance_suppressor_hud_open());
        assert!(!view.inspector_state.requested_open_resonance_suppressor_hud);
        view.close_resonance_suppressor_hud();

        // 2. Inspector Trigger: FM Matrix
        view.inspector_state.selected_node_kind = Some("FmMatrixNode".to_string());
        view.inspector_state.requested_open_fm_matrix_hud = true;
        assert!(!view.is_fm_matrix_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_fm_matrix_hud_open());
        assert!(!view.inspector_state.requested_open_fm_matrix_hud);
        view.close_fm_matrix_hud();

        // 3. Inspector Trigger: Spectral De-Esser
        view.inspector_state.selected_node_kind = Some("SpectralDeEsserNode".to_string());
        view.inspector_state.requested_open_spectral_deesser_hud = true;
        assert!(!view.is_spectral_deesser_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spectral_deesser_hud_open());
        assert!(!view.inspector_state.requested_open_spectral_deesser_hud);
        view.close_spectral_deesser_hud();

        // 4. Inspector Trigger: Spectral Matching EQ
        view.inspector_state.selected_node_kind = Some("SpectralMatchingEqNode".to_string());
        view.inspector_state.requested_open_spectral_matching_eq_hud = true;
        assert!(!view.is_spectral_matching_eq_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spectral_matching_eq_hud_open());
        assert!(!view.inspector_state.requested_open_spectral_matching_eq_hud);
        view.close_spectral_matching_eq_hud();

        // 5. Device Rack Trigger: Resonance Suppressor
        view.device_rack_state.selected_node_kind = Some("ResonanceSuppressor".to_string());
        view.device_rack_state.requested_open_resonance_suppressor_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_resonance_suppressor_hud_open());
        assert!(!view.device_rack_state.requested_open_resonance_suppressor_hud);
        view.close_resonance_suppressor_hud();

        // 6. Device Rack Trigger: FM Matrix
        view.device_rack_state.selected_node_kind = Some("FmMatrix".to_string());
        view.device_rack_state.requested_open_fm_matrix_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_fm_matrix_hud_open());
        assert!(!view.device_rack_state.requested_open_fm_matrix_hud);
        view.close_fm_matrix_hud();

        // 7. Device Rack Trigger: Spectral De-Esser
        view.device_rack_state.selected_node_kind = Some("SpectralDeEsser".to_string());
        view.device_rack_state.requested_open_spectral_deesser_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spectral_deesser_hud_open());
        assert!(!view.device_rack_state.requested_open_spectral_deesser_hud);
        view.close_spectral_deesser_hud();

        // 8. Device Rack Trigger: Spectral Matching EQ
        view.device_rack_state.selected_node_kind = Some("SpectralMatchingEq".to_string());
        view.device_rack_state.requested_open_spectral_matching_eq_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spectral_matching_eq_hud_open());
        assert!(!view.device_rack_state.requested_open_spectral_matching_eq_hud);
        view.close_spectral_matching_eq_hud();
    }

    #[test]
    fn test_turn102_gui_command_palette_actions_all_four() {
        let palette = CommandPalette::new();
        let target_ids = [
            "open_resonance_suppressor_hud",
            "open_fm_matrix_hud",
            "open_spectral_deesser_hud",
            "open_spectral_matching_eq_hud",
        ];

        for action_id in target_ids {
            let found = palette.actions.iter().any(|a| a.action_id == action_id);
            assert!(found, "Command palette missing action: {}", action_id);
        }
    }
}
