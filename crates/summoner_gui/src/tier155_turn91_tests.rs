#![allow(clippy::all)]

//! Test suite for Turn #91 (Sprint Turn #17) — Tactile Bitcrusher & Morphology HUD
//! and Multi-Voice Analog Bucket-Brigade (BBD) Chorus Matrix HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::bitcrusher_morph_view::{
        BitcrusherMorphView, MorphQuantizeMode, BITCRUSHER_PUCK_HIT_RADIUS,
    };
    use crate::views::bbd_chorus_view::{
        BbdChorusView, BbdClockMode, BBD_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn91_pure_bitcrusher_morph_initialization_and_kinematics() {
        let mut view = BitcrusherMorphView::new();
        assert_eq!(view.mode, MorphQuantizeMode::Linear);
        assert!((view.bit_depth - 6.5).abs() < 1e-3);
        assert!((view.downsample_ratio - 8.0).abs() < 1e-3);
        assert!((view.jitter_pct - 12.0).abs() < 1e-3);
        assert!((view.anti_alias_cutoff_hz - 8000.0).abs() < 1e-3);
        assert!((view.pre_drive_db - 4.5).abs() < 1e-3);
        assert!((view.mix_pct - 100.0).abs() < 1e-3);
        assert_eq!(BITCRUSHER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Mode switching & names
        let modes = [
            (MorphQuantizeMode::Linear, "LINEAR"),
            (MorphQuantizeMode::CompandedMuLaw, "μ-LAW LOG"),
            (MorphQuantizeMode::CompandedALaw, "A-LAW LOG"),
            (MorphQuantizeMode::ChaoticFoldback, "CHAOTIC FOLDBACK"),
        ];
        for (m, name) in modes {
            view.set_mode(m);
            assert_eq!(view.mode, m);
            assert_eq!(m.mode_name(), name);
        }

        // Coordinate normalizations & round-trips for Bit Depth
        let bit_test = 14.5;
        let bit_norm = BitcrusherMorphView::bit_to_normalized(bit_test);
        let bit_back = BitcrusherMorphView::normalized_to_bit(bit_norm);
        assert!((bit_back - bit_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Downsample Ratio
        let down_test = 28.0;
        let down_norm = BitcrusherMorphView::downsample_to_normalized(down_test);
        let down_back = BitcrusherMorphView::normalized_to_downsample(down_norm);
        assert!((down_back - down_test).abs() < 1e-3);

        // Quantization transfer curve calculation (64 points)
        let curve = view.calculate_transfer_curve(64);
        assert_eq!(curve.len(), 64);
        for &(x, y) in &curve {
            assert!(x >= -1.0 && x <= 1.0);
            assert!(y >= -1.0 && y <= 1.0);
        }

        // Puck hit testing
        let canvas = Rect::new(40.0, 40.0, 160.0, 160.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_morph_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_morph_puck((puck_x + 50.0, puck_y + 50.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("TACTILE BITCRUSHER & MORPHOLOGY HUD"));
        assert!(snap.contains("[PASS]"));
    }

    #[test]
    fn test_turn91_pure_bbd_chorus_initialization_and_kinematics() {
        let mut view = BbdChorusView::new();
        assert_eq!(view.mode, BbdClockMode::VintageBbdCompanded);
        assert_eq!(view.voices.len(), 6);
        assert!((view.stereo_spread_percent - 85.0).abs() < 1e-3);
        assert!((view.feedback_percent - 30.0).abs() < 1e-3);
        assert!((view.mix_percent - 50.0).abs() < 1e-3);
        assert!((view.bbd_clock_rate_khz - 44.1).abs() < 1e-3);
        assert_eq!(BBD_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Clock modes switching & names
        let modes = [
            (BbdClockMode::VintageBbdCompanded, "VINTAGE BBD (MN3007)"),
            (BbdClockMode::CleanModernAnalog, "CLEAN ANALOG MATRIX"),
            (BbdClockMode::DimensionDSpatial, "DIMENSION D SPATIAL"),
            (BbdClockMode::LoFiClockBleed, "LO-FI CLOCK BLEED"),
        ];
        for (m, name) in modes {
            view.set_mode(m);
            assert_eq!(view.mode, m);
            assert_eq!(m.mode_name(), name);
        }

        // Coordinate normalizations & round-trips for Spread
        let spread_test = 65.0;
        let spread_norm = BbdChorusView::spread_to_normalized(spread_test);
        let spread_back = BbdChorusView::normalized_to_spread(spread_norm);
        assert!((spread_back - spread_test).abs() < 1e-3);

        // Coordinate normalizations & round-trips for Feedback
        let fb_test = -45.0;
        let fb_norm = BbdChorusView::feedback_to_normalized(fb_test);
        let fb_back = BbdChorusView::normalized_to_feedback(fb_norm);
        assert!((fb_back - fb_test).abs() < 1e-3);

        // Lissajous trajectory evaluation
        for step in 0..16 {
            let t = (step as f32 / 16.0) * std::f32::consts::TAU;
            let (l, r) = view.evaluate_lissajous_point(t);
            assert!(l >= -1.0 && l <= 1.0);
            assert!(r >= -1.0 && r <= 1.0);
        }

        // Spatial puck hit testing
        let canvas = Rect::new(30.0, 30.0, 180.0, 180.0);
        let puck_x = canvas.x + view.spatial_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.spatial_puck_pos.1) * canvas.height;
        assert!(view.hit_test_spatial_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_spatial_puck((puck_x + 60.0, puck_y + 60.0), canvas));

        // Deterministic ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MULTI-VOICE BBD CHORUS & LISSAJOUS MATRIX HUD"));
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
    fn test_turn91_gui_modal_window_lifecycles_bitcrusher_and_bbd_chorus() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Bitcrusher Morphology HUD
        assert!(!view.is_bitcrusher_morph_hud_open());
        view.open_bitcrusher_morph_hud();
        assert!(view.is_bitcrusher_morph_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_bitcrusher_morph_hud();
        assert!(!view.is_bitcrusher_morph_hud_open());

        // 2. BBD Chorus HUD
        assert!(!view.is_bbd_chorus_hud_open());
        view.open_bbd_chorus_hud();
        assert!(view.is_bbd_chorus_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_bbd_chorus_hud();
        assert!(!view.is_bbd_chorus_hud_open());
    }

    #[test]
    fn test_turn91_gui_lock_free_param_bus_dispatch_bitcrusher_and_bbd_chorus() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id as u32).unwrap_or(1);
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels for Bitcrusher (offsets 0..=5) and BBD Chorus (offsets 0..=3)
        for offset in 0..=5 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Bitcrusher sync
        view.bitcrusher_morph_view.bit_depth = 8.5;
        view.bitcrusher_morph_view.downsample_ratio = 16.0;
        view.bitcrusher_morph_view.jitter_pct = 25.0;
        view.bitcrusher_morph_view.anti_alias_cutoff_hz = 6500.0;
        view.bitcrusher_morph_view.pre_drive_db = 6.0;
        view.bitcrusher_morph_view.mix_pct = 85.0;

        view.sync_bitcrusher_morph_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("bit_depth"), Some(&8.5));
        assert_eq!(view.device_rack_state.node_param_values.get("downsample_ratio"), Some(&16.0));
        assert_eq!(view.device_rack_state.node_param_values.get("jitter_pct"), Some(&25.0));
        assert_eq!(view.device_rack_state.node_param_values.get("anti_alias_cutoff_hz"), Some(&6500.0));
        assert_eq!(view.device_rack_state.node_param_values.get("pre_drive_db"), Some(&6.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mix_pct"), Some(&85.0));

        let pid_bits = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_down = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_bits), Some(8.5));
        assert_eq!(arc_bus.get(pid_down), Some(16.0));

        // Test BBD Chorus sync
        view.bbd_chorus_view.stereo_spread_percent = 90.0;
        view.bbd_chorus_view.feedback_percent = 45.0;
        view.bbd_chorus_view.mix_percent = 60.0;
        view.bbd_chorus_view.bbd_clock_rate_khz = 32.0;

        view.sync_bbd_chorus_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("stereo_spread_percent"), Some(&90.0));
        assert_eq!(view.device_rack_state.node_param_values.get("feedback_percent"), Some(&45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("mix_percent"), Some(&60.0));
        assert_eq!(view.device_rack_state.node_param_values.get("bbd_clock_rate_khz"), Some(&32.0));

        let pid_spread = ParamId(track_id * 1000 + slot_idx as u32 * 20);
        let pid_fb = ParamId(track_id * 1000 + slot_idx as u32 * 20 + 1);
        assert_eq!(arc_bus.get(pid_spread), Some(90.0));
        assert_eq!(arc_bus.get(pid_fb), Some(45.0));
    }

    #[test]
    fn test_turn91_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Bitcrusher trigger propagation from rack
        view.device_rack_state.requested_open_bitcrusher_morph_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_bitcrusher_morph_hud);
        assert!(view.is_bitcrusher_morph_hud_open());
        view.close_bitcrusher_morph_hud();

        // 2. BBD Chorus trigger propagation from inspector
        view.inspector_state.requested_open_bbd_chorus_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_bbd_chorus_hud);
        assert!(view.is_bbd_chorus_hud_open());
        view.close_bbd_chorus_hud();
    }

    #[test]
    fn test_turn91_gui_command_palette_actions() {
        let palette = CommandPalette::new();
        let bc_action = palette.actions.iter().find(|a| a.action_id == "open_bitcrusher_morph_hud");
        assert!(bc_action.is_some(), "open_bitcrusher_morph_hud must be registered in command palette");
        let bc = bc_action.unwrap();
        assert_eq!(bc.category, "Distortion / Lo-Fi");

        let bbd_action = palette.actions.iter().find(|a| a.action_id == "open_bbd_chorus_hud");
        assert!(bbd_action.is_some(), "open_bbd_chorus_hud must be registered in command palette");
        let bbd = bbd_action.unwrap();
        assert_eq!(bbd.category, "Modulation / Time");
    }
}
