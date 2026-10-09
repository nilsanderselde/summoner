#![allow(clippy::all)]

//! Test suite for Turn #106 (Sprint Turn #32) — Mastering Multi-Band Dynamic Crest Shaper & Punch Leveler
//! and Mastering Dynamic Linear-Phase Crossover & Multiband Limiter HUDs Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::dynamic_crest_shaper_view::{
        DynamicCrestShaperView, CrestTopology, CREST_PUCK_HIT_RADIUS,
    };
    use crate::views::linear_phase_crossover_view::{
        LinearPhaseCrossoverView, CrossoverSlope, CROSSOVER_PUCK_HIT_RADIUS,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn106_pure_dynamic_crest_shaper_initialization_and_kinematics() {
        let mut view = DynamicCrestShaperView::new();
        assert_eq!(view.topology, CrestTopology::PunchMaximizer);
        assert_eq!(CREST_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Topologies and their nominal values
        let topos = [
            (CrestTopology::PunchMaximizer, "PUNCH MAXIMIZER"),
            (CrestTopology::DensityCompactor, "DENSITY COMPACTOR"),
            (CrestTopology::MultibandTransientLeveler, "MULTIBAND LEVELER"),
            (CrestTopology::AcousticDynamicPreserver, "ACOUSTIC PRESERVER"),
            (CrestTopology::BroadcastLoudnessSculptor, "BROADCAST SCULPTOR"),
        ];

        for (t, expected_name) in topos {
            view.set_topology(t);
            assert_eq!(view.topology, t);
            assert_eq!(t.topology_name(), expected_name);
            assert!(t.nominal_crest_target_db() >= 3.0);
            assert!(t.nominal_expansion_ratio() >= 1.0);
            assert!(t.nominal_downward_ratio() >= 1.0);
            assert!(t.nominal_attack_ms() > 0.0);
        }

        // Normalization round-trips for Target Crest [3.0 ..= 24.0 dB]
        let crest_test = 14.5;
        let crest_norm = DynamicCrestShaperView::crest_to_normalized(crest_test);
        let crest_back = DynamicCrestShaperView::normalized_to_crest(crest_norm);
        assert!((crest_back - crest_test).abs() < 1e-3);

        // Normalization round-trips for Upward Expansion Ratio [1.0 ..= 4.0]
        let exp_test = 2.85;
        let exp_norm = DynamicCrestShaperView::ratio_to_normalized(exp_test);
        let exp_back = DynamicCrestShaperView::normalized_to_ratio(exp_norm);
        assert!((exp_back - exp_test).abs() < 1e-3);

        // Normalization round-trips for Downward Comp Ratio [1.0 ..= 12.0]
        let comp_test = 4.2;
        let comp_norm = DynamicCrestShaperView::downward_to_normalized(comp_test);
        let comp_back = DynamicCrestShaperView::normalized_to_downward(comp_norm);
        assert!((comp_back - comp_test).abs() < 1e-3);

        // Normalization round-trips for Attack [0.1 ..= 100.0 ms]
        let att_test = 5.5;
        let att_norm = DynamicCrestShaperView::attack_to_normalized(att_test);
        let att_back = DynamicCrestShaperView::normalized_to_attack(att_norm);
        assert!((att_back - att_test).abs() < 1e-3);

        // Dynamics simulation update and band factors
        view.set_topology(CrestTopology::PunchMaximizer);
        view.update_dynamics_simulation();
        for &cf in &view.band_crest_factors_db {
            assert!(cf >= 3.0 && cf <= 24.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(40.0, 40.0, 400.0, 300.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_crest_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_crest_puck((puck_x + 15.0, puck_y), canvas));
        assert!(view.hit_test_crest_puck((puck_x, puck_y + 15.0), canvas));
        assert!(!view.hit_test_crest_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING DYNAMIC CREST SHAPER & PUNCH LEVELER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn106_pure_linear_phase_crossover_initialization_and_kinematics() {
        let mut view = LinearPhaseCrossoverView::new();
        assert_eq!(view.slope_mode, CrossoverSlope::LinPhase48dBSymmetric);
        assert_eq!(CROSSOVER_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt bounding target

        // Slopes and nominal values
        let slopes = [
            (CrossoverSlope::LR24dBMinimumLatency, "24 dB/OCT FIR", 24.0, 128),
            (CrossoverSlope::LinPhase48dBSymmetric, "48 dB/OCT LIN-PHASE", 48.0, 512),
            (CrossoverSlope::LinPhase96dBUltraSharp, "96 dB/OCT BRICKWALL", 96.0, 2048),
            (CrossoverSlope::DynamicAdaptiveFFT, "DYNAMIC ADAPTIVE FFT", 60.0, 1024),
            (CrossoverSlope::MultiRateTransientFIR, "MULTI-RATE TRANSIENT", 36.0, 256),
        ];

        for (s, expected_name, db_oct, lat) in slopes {
            view.set_slope_mode(s);
            assert_eq!(view.slope_mode, s);
            assert_eq!(s.slope_name(), expected_name);
            assert_eq!(s.slope_db_per_oct(), db_oct);
            assert_eq!(s.latency_samples(), lat);
        }

        // Linear phase modes produce zero group delay
        view.set_slope_mode(CrossoverSlope::LinPhase48dBSymmetric);
        assert_eq!(view.group_delay_ms, 0.0);
        view.set_slope_mode(CrossoverSlope::LinPhase96dBUltraSharp);
        assert_eq!(view.group_delay_ms, 0.0);

        // Normalization round-trips for Crossover Frequency [40.0 ..= 16000.0 Hz]
        let freq_test = 2200.0;
        let freq_norm = LinearPhaseCrossoverView::freq_to_normalized(freq_test);
        let freq_back = LinearPhaseCrossoverView::normalized_to_freq(freq_norm);
        assert!((freq_back - freq_test).abs() < 1e-1);

        // Normalization round-trips for Ceiling [-18.0 ..= 0.0 dBFS]
        let ceil_test = -1.5;
        let ceil_norm = LinearPhaseCrossoverView::ceiling_to_normalized(ceil_test);
        let ceil_back = LinearPhaseCrossoverView::normalized_to_ceiling(ceil_norm);
        assert!((ceil_back - ceil_test).abs() < 1e-3);

        // Band gain reductions
        for &gr in &view.band_gr_db {
            assert!(gr <= 0.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(50.0, 50.0, 400.0, 300.0);
        let puck_x = canvas.x + view.xover_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.xover_puck_pos.1) * canvas.height;

        assert!(view.hit_test_xover_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_xover_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_xover_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_xover_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("MASTERING LINEAR-PHASE CROSSOVER & MULTIBAND LIMITER HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn106_gui_dynamic_crest_shaper_and_crossover_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Dynamic Crest Shaper Modal Lifecycle
        assert!(!view.is_dynamic_crest_shaper_hud_open());
        view.open_dynamic_crest_shaper_hud();
        assert!(view.is_dynamic_crest_shaper_hud_open());
        view.close_dynamic_crest_shaper_hud();
        assert!(!view.is_dynamic_crest_shaper_hud_open());

        // 2. Linear-Phase Crossover Modal Lifecycle
        assert!(!view.is_linear_phase_crossover_hud_open());
        view.open_linear_phase_crossover_hud();
        assert!(view.is_linear_phase_crossover_hud_open());
        view.close_linear_phase_crossover_hud();
        assert!(!view.is_linear_phase_crossover_hud_open());

        // 3. Dynamic Crest Shaper Parameter Sync Bridge
        view.dynamic_crest_shaper_view.target_crest_db = 17.5;
        view.dynamic_crest_shaper_view.expansion_ratio = 2.9;
        view.dynamic_crest_shaper_view.downward_comp_ratio = 1.4;
        view.dynamic_crest_shaper_view.attack_ms = 1.2;
        view.dynamic_crest_shaper_view.release_ms = 70.0;
        view.sync_dynamic_crest_shaper_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("target_crest_db").copied(),
            Some(17.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("expansion_ratio").copied(),
            Some(2.9)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("downward_comp_ratio").copied(),
            Some(1.4)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("crest").copied(),
            Some(17.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("ratio").copied(),
            Some(2.9)
        );

        // 4. Linear-Phase Crossover Parameter Sync Bridge
        view.linear_phase_crossover_view.split_low_hz = 140.0;
        view.linear_phase_crossover_view.split_mid_hz = 2500.0;
        view.linear_phase_crossover_view.split_high_hz = 8000.0;
        view.linear_phase_crossover_view.ceiling_db = -0.8;
        view.sync_linear_phase_crossover_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("split_mid_hz").copied(),
            Some(2500.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("ceiling_db").copied(),
            Some(-0.8)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("split_low_hz").copied(),
            Some(140.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("ceiling").copied(),
            Some(-0.8)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_dynamic_crest_shaper_modal_window(ui);
                view.show_linear_phase_crossover_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let crest_action = palette.actions.iter().find(|a| a.action_id == "open_dynamic_crest_shaper_hud");
        assert!(crest_action.is_some(), "open_dynamic_crest_shaper_hud should be registered");
        assert_eq!(crest_action.unwrap().category, "Mastering & Dynamics");

        let xover_action = palette.actions.iter().find(|a| a.action_id == "open_linear_phase_crossover_hud");
        assert!(xover_action.is_some(), "open_linear_phase_crossover_hud should be registered");
        assert_eq!(xover_action.unwrap().category, "Mastering & Dynamics");
    }
}
