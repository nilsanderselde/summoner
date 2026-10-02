#![allow(clippy::all)]

//! Test suite for Turn #70 / Turn #33 deliverable — Pipe Organ Rank Voicing & Electric Piano Tine Resonator HUDs Convergence (Milestones 22, 24 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::rank_voicing_view::{
        RankVoicingView, VoicingRankType, RANK_VOICING_PUCK_HIT_RADIUS,
        MIN_CUTUP, MAX_CUTUP, MIN_TOE_HOLE, MAX_TOE_HOLE,
    };
    use crate::views::tine_resonator_view::{
        TineResonatorView, TineResonatorProfile, TINE_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn70_pure_rank_voicing_view_initialization_and_ranks() {
        let mut view = RankVoicingView::new();
        assert_eq!(view.rank_type, VoicingRankType::Principal8);
        assert!((view.cutup_ratio - 0.25).abs() < 1e-3);
        assert!((view.toe_hole_aperture - 0.85).abs() < 1e-3);
        assert_eq!(RANK_VOICING_PUCK_HIT_RADIUS, 22.0);

        // Test pipe ranks
        view.set_rank(VoicingRankType::Bourdon16);
        assert_eq!(view.rank_type, VoicingRankType::Bourdon16);

        view.set_rank(VoicingRankType::Octave4);
        assert_eq!(view.rank_type, VoicingRankType::Octave4);

        view.set_rank(VoicingRankType::Flute4);
        assert_eq!(view.rank_type, VoicingRankType::Flute4);

        view.set_rank(VoicingRankType::SuperOctave2);
        assert_eq!(view.rank_type, VoicingRankType::SuperOctave2);

        view.set_rank(VoicingRankType::MixtureIV);
        assert_eq!(view.rank_type, VoicingRankType::MixtureIV);

        view.set_rank(VoicingRankType::Trompette8);
        assert_eq!(view.rank_type, VoicingRankType::Trompette8);

        view.set_rank(VoicingRankType::VoxHumana8);
        assert_eq!(view.rank_type, VoicingRankType::VoxHumana8);

        // Physics updates from puck
        view.update_physics_from_puck(0.5, 0.5);
        let exp_cutup = MIN_CUTUP + 0.5 * (MAX_CUTUP - MIN_CUTUP);
        let exp_toe = MIN_TOE_HOLE + 0.5 * (MAX_TOE_HOLE - MIN_TOE_HOLE);
        assert!((view.cutup_ratio - exp_cutup).abs() < 1e-3);
        assert!((view.toe_hole_aperture - exp_toe).abs() < 1e-3);

        // Acoustics
        assert!(view.air_velocity_mps > 0.0);
        assert!(view.chiff_duration_ms > 0.0);
        assert_eq!(view.harmonic_weights.len(), 8);
        assert!(view.harmonic_weights[0] > 0.0);

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
        assert!(snap.contains('V'));
    }

    #[test]
    fn test_turn70_pure_tine_resonator_view_initialization_and_profiles() {
        let mut view = TineResonatorView::new();
        assert_eq!(view.profile, TineResonatorProfile::RhodesStage73);
        assert!((view.tonebar_coupling - 0.70).abs() < 1e-3);
        assert!((view.beam_stiffness - 0.42).abs() < 1e-3);
        assert_eq!(TINE_PUCK_HIT_RADIUS, 22.0);

        // Test Profiles
        view.set_profile(TineResonatorProfile::RhodesSuitcase88);
        assert_eq!(view.profile, TineResonatorProfile::RhodesSuitcase88);
        assert!((view.tonebar_coupling - 0.85).abs() < 1e-3);
        assert!((view.beam_stiffness - 0.32).abs() < 1e-3);

        view.set_profile(TineResonatorProfile::Wurlitzer200A);
        assert_eq!(view.profile, TineResonatorProfile::Wurlitzer200A);
        assert!((view.tonebar_coupling - 0.45).abs() < 1e-3);
        assert!((view.beam_stiffness - 0.68).abs() < 1e-3);

        view.set_profile(TineResonatorProfile::YamahaCP70);
        assert_eq!(view.profile, TineResonatorProfile::YamahaCP70);
        assert!((view.tonebar_coupling - 0.30).abs() < 1e-3);
        assert!((view.beam_stiffness - 0.85).abs() < 1e-3);

        view.set_profile(TineResonatorProfile::CustomHybrid);
        assert_eq!(view.profile, TineResonatorProfile::CustomHybrid);

        // Physics updates from puck
        view.update_physics_from_puck(0.65, 0.48);
        assert!((view.tonebar_coupling - 0.65).abs() < 1e-3);
        assert!((view.beam_stiffness - 0.48).abs() < 1e-3);

        // Cantilever deflection evaluation
        let deflection = view.evaluate_beam_deflection();
        assert_eq!(deflection.len(), 32);
        // Tip deflection is non-zero
        assert!(deflection[31].abs() > 0.0);

        // Optical tremolo evaluation
        let lissajous = view.evaluate_tremolo_lissajous();
        assert_eq!(lissajous.len(), 32);
        for &(l, r) in &lissajous {
            assert!(l >= 0.0 && l <= 1.0);
            assert!(r >= 0.0 && r <= 1.0);
        }

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("TINE RESONATOR"));
    }

    #[test]
    fn test_turn70_pure_slot_arithmetic_channel_isolation() {
        let track_id = 2u32;
        let slot_id = 3u32;

        let pid_cut = ParamId(track_id * 1000 + slot_id * 20);
        let pid_toe = ParamId(track_id * 1000 + slot_id * 20 + 1);
        let pid_lan = ParamId(track_id * 1000 + slot_id * 20 + 2);
        let pid_prs = ParamId(track_id * 1000 + slot_id * 20 + 3);

        assert_eq!(pid_cut.0, 2060);
        assert_eq!(pid_toe.0, 2061);
        assert_eq!(pid_lan.0, 2062);
        assert_eq!(pid_prs.0, 2063);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 2080);
        assert!(pid_prs.0 < next_slot_pid.0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use std::sync::Arc;

    #[test]
    fn test_turn70_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Rank Voicing modal
        view.open_rank_voicing_hud();
        assert!(view.is_rank_voicing_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_rank_voicing_hud();
        assert!(!view.is_rank_voicing_hud_open());

        // Open Tine Resonator modal
        view.open_tine_resonator_hud();
        assert!(view.is_tine_resonator_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_tine_resonator_hud();
        assert!(!view.is_tine_resonator_hud_open());
    }

    #[test]
    fn test_turn70_gui_live_parambus_slot_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..10 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // 1. Sync Pipe Organ Rank Voicing HUD
        view.rank_voicing_view.cutup_ratio = 0.35;
        view.rank_voicing_view.toe_hole_aperture = 0.75;
        view.rank_voicing_view.languid_height_mm = 0.50;
        view.rank_voicing_view.wind_pressure_mmh2o = 110.0;
        view.sync_rank_voicing_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.35));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.75));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.50));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(110.0));

        assert_eq!(view.device_rack_state.node_param_values.get("cutup_ratio"), Some(&0.35));
        assert_eq!(view.device_rack_state.node_param_values.get("toe_hole_aperture"), Some(&0.75));
        assert_eq!(view.inspector_state.node_param_values.get("cutup_ratio"), Some(&0.35));

        // 2. Sync Electromechanical Tine Resonator HUD
        view.tine_resonator_view.tonebar_coupling = 0.82;
        view.tine_resonator_view.beam_stiffness = 0.44;
        view.tine_resonator_view.tremolo_rate_hz = 6.4;
        view.tine_resonator_view.tremolo_depth = 0.72;
        view.sync_tine_resonator_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.82));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.44));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(6.4));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.72));

        assert_eq!(view.device_rack_state.node_param_values.get("tonebar_coupling"), Some(&0.82));
        assert_eq!(view.device_rack_state.node_param_values.get("beam_stiffness"), Some(&0.44));
        assert_eq!(view.inspector_state.node_param_values.get("tonebar_coupling"), Some(&0.82));
    }

    #[test]
    fn test_turn70_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_rank_voicing_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_rank_voicing_hud_open());
        assert!(!view.device_rack_state.requested_open_rank_voicing_hud);

        view.close_rank_voicing_hud();
        assert!(!view.is_rank_voicing_hud_open());

        view.inspector_state.requested_open_tine_resonator_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_tine_resonator_hud_open());
        assert!(!view.inspector_state.requested_open_tine_resonator_hud);
    }

    #[test]
    fn test_turn70_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let rank_action = palette.actions.iter().find(|a| a.action_id == "open_rank_voicing_hud");
        assert!(rank_action.is_some());
        let rank = rank_action.unwrap();
        assert_eq!(rank.category, "Physical Modeling");
        assert!(rank.label.contains("Voicing"));

        let tine_action = palette.actions.iter().find(|a| a.action_id == "open_tine_resonator_hud");
        assert!(tine_action.is_some());
        let tine = tine_action.unwrap();
        assert_eq!(tine.category, "Physical Modeling");
        assert!(tine.label.contains("Tine"));
    }
}
