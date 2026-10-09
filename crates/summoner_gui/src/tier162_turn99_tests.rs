#![allow(clippy::all)]

//! Test suite for Turn #99 (Sprint Turn #25) — Dynamic Harmonic Exciter & Brilliance HUD
//! and Psychoacoustic Stereo Widener & Vector Scope HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::harmonic_exciter_view::{
        ExciterMode, HarmonicExciterView, EXCITER_PUCK_HIT_RADIUS,
    };
    use crate::views::stereo_widener_view::{
        StereoWidenerMode, StereoWidenerView, STEREO_CROSSOVER_HANDLE_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn99_pure_harmonic_exciter_initialization_and_kinematics() {
        let mut view = HarmonicExciterView::new();
        assert_eq!(view.mode, ExciterMode::TapeHarmonics);
        assert!((view.crossover_freq_hz - 5000.0).abs() < 1e-3);
        assert!((view.drive_percent - 45.0).abs() < 1e-3);
        assert!((view.brilliance_db - 6.5).abs() < 1e-3);
        assert!((view.warmth_blend - 0.65).abs() < 1e-3);
        assert!((view.transient_weight - 50.0).abs() < 1e-3);
        assert!(!view.audition_harmonics);
        assert_eq!(EXCITER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (ExciterMode::TapeHarmonics, "Tape (3rd Harmonic)"),
            (ExciterMode::TubeEvenOrder, "Tube (2nd Harmonic)"),
            (ExciterMode::TransistorOddOrder, "Transistor Edge"),
            (ExciterMode::PsychoacousticAir, "Psychoacoustic Air"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Frequency [1000.0..20000.0 Hz]
        let f_test = 6500.0;
        let f_norm = HarmonicExciterView::freq_to_normalized(f_test);
        let f_back = HarmonicExciterView::normalized_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        // Drive [0.0..100.0 %]
        let dr_test = 72.5;
        let dr_norm = HarmonicExciterView::drive_to_normalized(dr_test);
        let dr_back = HarmonicExciterView::normalized_to_drive(dr_norm);
        assert!((dr_back - dr_test).abs() < 1e-3);

        // Brilliance [0.0..18.0 dB]
        let br_test = 11.2;
        let br_norm = HarmonicExciterView::brilliance_to_normalized(br_test);
        let br_back = HarmonicExciterView::normalized_to_brilliance(br_norm);
        assert!((br_back - br_test).abs() < 1e-3);

        // Warmth [0.0..1.0]
        let w_test = 0.82;
        let w_norm = HarmonicExciterView::warmth_to_normalized(w_test);
        let w_back = HarmonicExciterView::normalized_to_warmth(w_norm);
        assert!((w_back - w_test).abs() < 1e-3);

        // Transient weight [0.0..100.0 %]
        let tw_test = 38.0;
        let tw_norm = HarmonicExciterView::transient_to_normalized(tw_test);
        let tw_back = HarmonicExciterView::normalized_to_transient(tw_norm);
        assert!((tw_back - tw_test).abs() < 1e-3);

        // Harmonic response curve calculation
        let low_resp = view.evaluate_harmonic_response(500.0);
        let high_resp = view.evaluate_harmonic_response(10000.0);
        assert!(high_resp > low_resp);

        // Exciter puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.harmonic_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.harmonic_puck_pos.1) * canvas.height;
        assert!(view.hit_test_harmonic_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_harmonic_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DYNAMIC HARMONIC EXCITER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn99_pure_stereo_widener_initialization_and_kinematics() {
        let mut view = StereoWidenerView::new();
        assert_eq!(view.mode, StereoWidenerMode::FrequencySplitMultiband);
        assert!((view.low_band_width_pct - 0.0).abs() < 1e-3);
        assert!((view.mid_band_width_pct - 100.0).abs() < 1e-3);
        assert!((view.high_band_width_pct - 140.0).abs() < 1e-3);
        assert!((view.crossover_low_hz - 180.0).abs() < 1e-3);
        assert!((view.crossover_high_hz - 4000.0).abs() < 1e-3);
        assert!((view.haas_delay_ms - 8.5).abs() < 1e-3);
        assert!((view.phase_correlation - 0.82).abs() < 1e-3);
        assert!(view.mono_bass_enabled);
        assert!(!view.mono_compatibility_check);
        assert_eq!(STEREO_CROSSOVER_HANDLE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (StereoWidenerMode::MidSideBandWidth, "Mid/Side Band Width"),
            (StereoWidenerMode::HaasMicroDelay, "Haas Micro-Delay"),
            (StereoWidenerMode::FrequencySplitMultiband, "Frequency Split Multiband"),
            (StereoWidenerMode::BinauralHRTFSpatializer, "Binaural HRTF Spatializer"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Width [0.0..200.0 %]
        let w_test = 125.0;
        let w_norm = StereoWidenerView::width_to_normalized(w_test);
        let w_back = StereoWidenerView::normalized_to_width(w_norm);
        assert!((w_back - w_test).abs() < 1e-3);

        // Haas delay [0.0..30.0 ms]
        let hd_test = 14.5;
        let hd_norm = StereoWidenerView::haas_delay_to_normalized(hd_test);
        let hd_back = StereoWidenerView::normalized_to_haas_delay(hd_norm);
        assert!((hd_back - hd_test).abs() < 1e-3);

        // Stereo balance [-1.0..+1.0]
        let b_test = -0.35;
        let b_norm = StereoWidenerView::balance_to_normalized(b_test);
        let b_back = StereoWidenerView::normalized_to_balance(b_norm);
        assert!((b_back - b_test).abs() < 1e-3);

        // Frequency [20.0..20000.0 Hz]
        let f_test = 1200.0;
        let f_norm = StereoWidenerView::freq_to_norm_x(f_test);
        let f_back = StereoWidenerView::norm_x_to_freq(f_norm);
        assert!((f_back - f_test).abs() < 1.0);

        // Vector scope Lissajous projection
        let center = (130.0, 130.0);
        let radius = 90.0;
        let (x_mono, y_mono) = StereoWidenerView::project_vector_scope(1.0, 1.0, center, radius);
        assert!((x_mono - center.0).abs() < 1e-3); // Pure mono has 0 side component
        assert!(y_mono < center.1); // Pure mono is oriented upward

        // Crossover handle hit testing
        let canvas = Rect::new(0.0, 0.0, 500.0, 200.0);
        let low_x = canvas.x + StereoWidenerView::freq_to_norm_x(view.crossover_low_hz) * canvas.width;
        let high_x = canvas.x + StereoWidenerView::freq_to_norm_x(view.crossover_high_hz) * canvas.width;
        assert_eq!(view.hit_test_crossover_handle((low_x, 100.0), canvas), Some(0));
        assert_eq!(view.hit_test_crossover_handle((high_x, 100.0), canvas), Some(1));
        assert_eq!(view.hit_test_crossover_handle((low_x + 60.0, 100.0), canvas), None);

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("PSYCHOACOUSTIC STEREO WIDENER & VECTOR SCOPE HUD"));
        assert!(snap.contains("[PASS]"));
    }
}

#[cfg(test)]
#[cfg(feature = "gui")]
pub mod gui_tests {
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_inspector::show_modern_inspector_with_context;

    #[test]
    fn test_turn99_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Dynamic Harmonic Exciter HUD
        assert!(!view.is_harmonic_exciter_hud_open());
        view.open_harmonic_exciter_hud();
        assert!(view.is_harmonic_exciter_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_harmonic_exciter_hud();
        assert!(!view.is_harmonic_exciter_hud_open());

        // 2. Psychoacoustic Stereo Widener HUD
        assert!(!view.is_stereo_widener_hud_open());
        view.open_stereo_widener_hud();
        assert!(view.is_stereo_widener_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_stereo_widener_hud();
        assert!(!view.is_stereo_widener_hud_open());
    }

    #[test]
    fn test_turn99_gui_lock_free_param_bus_dispatch_harmonic_exciter_and_stereo_widener() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Harmonic Exciter (offsets 0..=3) and Stereo Widener (offsets 0..=4)
        for offset in 0..=4 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Harmonic Exciter sync
        view.harmonic_exciter_view.crossover_freq_hz = 7200.0;
        view.harmonic_exciter_view.drive_percent = 65.0;
        view.harmonic_exciter_view.brilliance_db = 9.5;
        view.harmonic_exciter_view.warmth_blend = 0.40;

        view.sync_harmonic_exciter_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("crossover_freq_hz"), Some(&7200.0));
        assert_eq!(view.device_rack_state.node_param_values.get("drive_percent"), Some(&65.0));
        assert_eq!(view.device_rack_state.node_param_values.get("brilliance_db"), Some(&9.5));
        assert_eq!(view.device_rack_state.node_param_values.get("warmth_blend"), Some(&0.40));

        let pid_drive = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_freq = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_brill = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_warmth = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);

        assert!((arc_bus.get(pid_drive).unwrap() - 65.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_freq).unwrap() - 7200.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_brill).unwrap() - 9.5).abs() < 1e-3);
        assert!((arc_bus.get(pid_warmth).unwrap() - 0.40).abs() < 1e-3);

        // Test Stereo Widener sync
        view.stereo_widener_view.haas_delay_ms = 12.0;
        view.stereo_widener_view.mid_band_width_pct = 150.0;
        view.stereo_widener_view.crossover_low_hz = 220.0;
        view.stereo_widener_view.crossover_high_hz = 5500.0;
        view.stereo_widener_view.balance_lr = 0.15;

        view.sync_stereo_widener_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("haas_delay_ms"), Some(&12.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mid_band_width_pct"), Some(&150.0));
        assert_eq!(view.device_rack_state.node_param_values.get("crossover_low_hz"), Some(&220.0));
        assert_eq!(view.device_rack_state.node_param_values.get("crossover_high_hz"), Some(&5500.0));
        assert_eq!(view.device_rack_state.node_param_values.get("balance_lr"), Some(&0.15));

        let pid_haas = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_width = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_xlow = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_xhigh = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        let pid_bal = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 4);

        assert!((arc_bus.get(pid_haas).unwrap() - 12.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_width).unwrap() - 150.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_xlow).unwrap() - 220.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_xhigh).unwrap() - 5500.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_bal).unwrap() - 0.15).abs() < 1e-3);
    }

    #[test]
    fn test_turn99_gui_inspector_and_rack_trigger_buttons() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Modern Inspector Trigger Propagation for Harmonic Exciter
        view.inspector_state.selected_node_kind = Some("HarmonicExciterNode".to_string());
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

        view.inspector_state.requested_open_harmonic_exciter_hud = true;
        assert!(!view.is_harmonic_exciter_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_harmonic_exciter_hud_open());
        view.close_harmonic_exciter_hud();

        // 2. Modern Inspector Trigger Propagation for Stereo Widener
        view.inspector_state.selected_node_kind = Some("StereoWidenerNode".to_string());
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

        view.inspector_state.requested_open_stereo_widener_hud = true;
        assert!(!view.is_stereo_widener_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_stereo_widener_hud_open());
        view.close_stereo_widener_hud();

        // 3. Modern Device Rack Trigger Propagation for Harmonic Exciter
        view.device_rack_state.selected_node_kind = Some("HarmonicExciter".to_string());
        view.device_rack_state.requested_open_harmonic_exciter_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_harmonic_exciter_hud_open());
        view.close_harmonic_exciter_hud();

        // 4. Modern Device Rack Trigger Propagation for Stereo Widener
        view.device_rack_state.selected_node_kind = Some("StereoWidener".to_string());
        view.device_rack_state.requested_open_stereo_widener_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_stereo_widener_hud_open());
        view.close_stereo_widener_hud();
    }
}
