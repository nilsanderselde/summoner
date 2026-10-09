#![allow(clippy::all)]

//! Test suite for Turn #97 (Sprint Turn #23) — Analog Tape Flanger Through-Zero HUD
//! and Multi-Tap Delay Matrix & Spatial Bounce HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::through_zero_flanger_view::{
        TapeFlangerMode, ThroughZeroFlangerView, FLANGER_PUCK_HIT_RADIUS,
    };
    use crate::views::multitap_delay_view::{
        MultitapDelayView, MULTITAP_HANDLE_HIT_RADIUS, MAX_DELAY_TAPS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn97_pure_through_zero_flanger_initialization_and_kinematics() {
        let mut view = ThroughZeroFlangerView::new();
        assert_eq!(view.mode, TapeFlangerMode::ThroughZeroLinear);
        assert!((view.manual_delay_ms - 0.0).abs() < 1e-3);
        assert!((view.lfo_rate_hz - 0.25).abs() < 1e-3);
        assert!((view.lfo_depth_ms - 2.5).abs() < 1e-3);
        assert!((view.feedback_pct - 65.0).abs() < 1e-3);
        assert!((view.tape_saturation_pct - 35.0).abs() < 1e-3);
        assert!((view.wow_flutter_pct - 15.0).abs() < 1e-3);
        assert!((view.dry_wet_pct - 50.0).abs() < 1e-3);
        assert_eq!(FLANGER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (TapeFlangerMode::ThroughZeroLinear, "LINEAR TZF"),
            (TapeFlangerMode::ThroughZeroExponential, "EXPONENTIAL TZF"),
            (TapeFlangerMode::BarberPoleFlanger, "BARBER-POLE SPIRAL"),
        ];

        for (mode, expected_name) in modes {
            view.set_mode(mode);
            assert_eq!(view.mode, mode);
            assert_eq!(mode.name(), expected_name);
        }

        // Coordinate normalizations & round-trips for Delay [-5.0..+5.0 ms]
        let delay_test = -2.75;
        let delay_norm = ThroughZeroFlangerView::delay_to_normalized(delay_test);
        let delay_back = ThroughZeroFlangerView::normalized_to_delay(delay_norm);
        assert!((delay_back - delay_test).abs() < 1e-3);

        // Feedback [-99.0..+99.0 %]
        let fb_test = 84.5;
        let fb_norm = ThroughZeroFlangerView::feedback_to_normalized(fb_test);
        let fb_back = ThroughZeroFlangerView::normalized_to_feedback(fb_norm);
        assert!((fb_back - fb_test).abs() < 1e-3);

        // Comb notch magnitude evaluation at zero crossover vs offset
        view.manual_delay_ms = 0.0;
        let null_mag = view.evaluate_notch_magnitude(1000.0);
        assert!((null_mag - 0.05).abs() < 1e-3); // Perfect phase null

        view.manual_delay_ms = 2.0;
        let offset_mag = view.evaluate_notch_magnitude(1000.0);
        assert!(offset_mag.is_finite());
        assert!(offset_mag >= 0.0 && offset_mag <= 1.0);

        // Flanger puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let puck_x = canvas.x + view.zero_cross_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.zero_cross_puck_pos.1) * canvas.height;
        assert!(view.hit_test_zero_cross_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_zero_cross_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("THROUGH-ZERO FLANGER HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn97_pure_multitap_delay_initialization_and_kinematics() {
        let mut view = MultitapDelayView::new();
        assert_eq!(view.taps.len(), 4);
        assert!((view.tempo_bpm - 120.0).abs() < 1e-3);
        assert!(view.sync_to_bpm);
        assert!((view.ping_pong_width_pct - 75.0).abs() < 1e-3);
        assert!((view.diffusion_pct - 40.0).abs() < 1e-3);
        assert!((view.master_dry_wet_pct - 50.0).abs() < 1e-3);
        assert_eq!(MULTITAP_HANDLE_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Tap management
        assert!(view.add_tap(625.0, 30.0, -0.5));
        assert_eq!(view.taps.len(), 5);
        assert!(view.remove_tap(4));
        assert_eq!(view.taps.len(), 4);

        // Max taps check
        for i in 0..4 {
            view.add_tap(700.0 + i as f32 * 50.0, 20.0, 0.0);
        }
        assert_eq!(view.taps.len(), MAX_DELAY_TAPS);
        assert!(!view.add_tap(999.0, 10.0, 0.0)); // Over capacity

        // Coordinate normalizations & round-trips for Time [10.0..2000.0 ms]
        let time_test = 425.0;
        let time_norm = MultitapDelayView::time_to_normalized(time_test);
        let time_back = MultitapDelayView::normalized_to_time(time_norm);
        assert!((time_back - time_test).abs() < 1e-3);

        // Pan [-1.0..+1.0]
        let pan_test = -0.65;
        let pan_norm = MultitapDelayView::pan_to_normalized(pan_test);
        let pan_back = MultitapDelayView::normalized_to_pan(pan_norm);
        assert!((pan_back - pan_test).abs() < 1e-3);

        // Stereo width [0.0..100.0 %]
        let width_test = 88.0;
        let width_norm = MultitapDelayView::width_to_normalized(width_test);
        let width_back = MultitapDelayView::normalized_to_width(width_norm);
        assert!((width_back - width_test).abs() < 1e-3);

        // Diffusion [0.0..100.0 %]
        let diff_test = 62.5;
        let diff_norm = MultitapDelayView::diffusion_to_normalized(diff_test);
        let diff_back = MultitapDelayView::normalized_to_diffusion(diff_norm);
        assert!((diff_back - diff_test).abs() < 1e-3);

        // Tap handle hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let tap = &view.taps[0];
        let tap_x = canvas.x + MultitapDelayView::time_to_normalized(tap.time_ms) * canvas.width;
        let tap_y = canvas.y + (1.0 - MultitapDelayView::pan_to_normalized(tap.pan)) * canvas.height;
        assert!(view.hit_test_tap((tap_x, tap_y), canvas, 0));
        assert!(!view.hit_test_tap((tap_x + 50.0, tap_y + 50.0), canvas, 0));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTI-TAP DELAY MATRIX HUD"));
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
    fn test_turn97_gui_modal_window_lifecycles() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Analog Tape Flanger Through-Zero HUD
        assert!(!view.is_through_zero_flanger_hud_open());
        view.open_through_zero_flanger_hud();
        assert!(view.is_through_zero_flanger_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_through_zero_flanger_hud();
        assert!(!view.is_through_zero_flanger_hud_open());

        // 2. Multi-Tap Delay Matrix & Spatial Bounce HUD
        assert!(!view.is_multitap_delay_hud_open());
        view.open_multitap_delay_hud();
        assert!(view.is_multitap_delay_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_multitap_delay_hud();
        assert!(!view.is_multitap_delay_hud_open());
    }

    #[test]
    fn test_turn97_gui_lock_free_param_bus_dispatch_flanger_and_multitap() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Flanger (offsets 0..=6) and Multi-Tap (offsets 0..=4)
        for offset in 0..=6 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Flanger sync
        view.through_zero_flanger_view.manual_delay_ms = -1.25;
        view.through_zero_flanger_view.lfo_rate_hz = 0.85;
        view.through_zero_flanger_view.lfo_depth_ms = 3.2;
        view.through_zero_flanger_view.feedback_pct = -45.0;
        view.through_zero_flanger_view.tape_saturation_pct = 60.0;
        view.through_zero_flanger_view.wow_flutter_pct = 25.0;
        view.through_zero_flanger_view.dry_wet_pct = 70.0;

        view.sync_through_zero_flanger_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("manual_delay_ms"), Some(&-1.25));
        assert_eq!(view.device_rack_state.node_param_values.get("lfo_rate_hz"), Some(&0.85));
        assert_eq!(view.device_rack_state.node_param_values.get("lfo_depth_ms"), Some(&3.2));
        assert_eq!(view.device_rack_state.node_param_values.get("feedback_pct"), Some(&-45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("tape_saturation_pct"), Some(&60.0));
        assert_eq!(view.device_rack_state.node_param_values.get("wow_flutter_pct"), Some(&25.0));
        assert_eq!(view.device_rack_state.node_param_values.get("dry_wet_pct"), Some(&70.0));

        let pid_delay = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_rate = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_fb = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        assert!((arc_bus.get(pid_delay).unwrap() - (-1.25)).abs() < 1e-3);
        assert!((arc_bus.get(pid_rate).unwrap() - 0.85).abs() < 1e-3);
        assert!((arc_bus.get(pid_fb).unwrap() - (-45.0)).abs() < 1e-3);

        // Test Multi-Tap sync
        view.multitap_delay_view.tempo_bpm = 135.0;
        view.multitap_delay_view.sync_to_bpm = true;
        view.multitap_delay_view.ping_pong_width_pct = 85.0;
        view.multitap_delay_view.diffusion_pct = 55.0;
        view.multitap_delay_view.master_dry_wet_pct = 65.0;

        view.sync_multitap_delay_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("tempo_bpm"), Some(&135.0));
        assert_eq!(view.device_rack_state.node_param_values.get("sync_to_bpm"), Some(&1.0));
        assert_eq!(view.device_rack_state.node_param_values.get("ping_pong_width_pct"), Some(&85.0));
        assert_eq!(view.device_rack_state.node_param_values.get("diffusion_pct"), Some(&55.0));
        assert_eq!(view.device_rack_state.node_param_values.get("master_dry_wet_pct"), Some(&65.0));

        let pid_wet = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_width = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        let pid_diff = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 2);
        let pid_bpm = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 3);
        let pid_sync = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 4);
        assert!((arc_bus.get(pid_wet).unwrap() - 65.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_width).unwrap() - 85.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_diff).unwrap() - 55.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_bpm).unwrap() - 135.0).abs() < 1e-3);
        assert!((arc_bus.get(pid_sync).unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn97_gui_inspector_and_rack_trigger_buttons() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Modern Inspector Trigger Propagation for Flanger
        view.inspector_state.selected_node_kind = Some("ThroughZeroFlangerNode".to_string());
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

        view.inspector_state.requested_open_through_zero_flanger_hud = true;
        assert!(!view.is_through_zero_flanger_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_through_zero_flanger_hud_open());
        view.close_through_zero_flanger_hud();

        // 2. Modern Inspector Trigger Propagation for Multitap
        view.inspector_state.selected_node_kind = Some("MultitapDelayNode".to_string());
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

        view.inspector_state.requested_open_multitap_delay_hud = true;
        assert!(!view.is_multitap_delay_hud_open());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multitap_delay_hud_open());
        view.close_multitap_delay_hud();

        // 3. Modern Device Rack Trigger Propagation for Flanger
        view.device_rack_state.selected_node_kind = Some("EffectFlanger".to_string());
        view.device_rack_state.requested_open_through_zero_flanger_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_through_zero_flanger_hud_open());
        view.close_through_zero_flanger_hud();

        // 4. Modern Device Rack Trigger Propagation for Multitap
        view.device_rack_state.selected_node_kind = Some("MultiTapModulatedDiffuser".to_string());
        view.device_rack_state.requested_open_multitap_delay_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_multitap_delay_hud_open());
        view.close_multitap_delay_hud();
    }
}
