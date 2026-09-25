// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #31 Integration & Regression Tests:
//! 1. Console Mixer track selection (`select_track_for_mixer`) state reflection across Inspector and Device Rack.
//! 2. Console Mixer `reset_all_channel_faders` to unity gain (1.0 = 0.0 dB) and center pan (0.0).
//! 3. Console Mixer `toggle_all_tracks_mute` global muting/unmuting killswitch.
//! 4. Master bus mono summing audition mode toggle (`toggle_master_mono`).
//! 5. Milestone 33 multi-track live Bézier automation evaluation across all project channels.
//! 6. Milestone 33 live ParamBus lock-free dispatch for multi-track gain, pan, mute, solo, master mono (9998), and master gain (9999).
//! 7. Headless egui Context rendering of the upgraded Console Mixer canvas without panics.

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::schema::ProjectConfig;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::{
        AutomationCurve, AutomationLane, AutomationPoint, AutomationTimeline, Interpolation,
    };

    #[test]
    fn test_turn31_mixer_track_selection_and_device_name() {
        let mut view = AwardWinningGuiView::new();

        // 1. Select Track 1 (Snare)
        assert!(view.select_track_for_mixer(1));
        assert_eq!(view.selected_track_idx, 1);
        assert_eq!(view.device_rack_state.device_name, "Snare");
        assert_eq!(view.inspector_state.target_name, "Snare");

        // 2. Select Track 0 (Kick)
        assert!(view.select_track_for_mixer(0));
        assert_eq!(view.selected_track_idx, 0);
        assert_eq!(view.device_rack_state.device_name, "Kick");
        assert_eq!(view.inspector_state.target_name, "Kick");

        // 3. Out of bounds safety
        assert!(!view.select_track_for_mixer(999));
        assert_eq!(view.selected_track_idx, 0);
    }

    #[test]
    fn test_turn31_mixer_reset_all_channel_faders() {
        let mut view = AwardWinningGuiView::new();
        assert!(view.tracks.len() >= 2);

        // Alter gains and pans
        view.tracks[0].gain = 0.42;
        view.tracks[0].pan = -0.75;
        view.tracks[1].gain = 1.38;
        view.tracks[1].pan = 0.60;

        view.reset_all_channel_faders();

        for tr in &view.tracks {
            assert_eq!(tr.gain, 1.0, "All track gains must reset to 1.0 (unity)");
            assert_eq!(tr.pan, 0.0, "All track pans must reset to 0.0 (center)");
        }
        assert_eq!(view.inspector_state.gain_db, 0.0);
        assert_eq!(view.inspector_state.pan_val, 0.0);
    }

    #[test]
    fn test_turn31_mixer_toggle_all_tracks_mute() {
        let mut view = AwardWinningGuiView::new();
        assert!(view.tracks.len() >= 2);

        // Ensure all unmuted initially
        for tr in &mut view.tracks {
            tr.is_muted = false;
        }

        // 1. Toggle -> all become muted
        let muted = view.toggle_all_tracks_mute();
        assert!(muted);
        for tr in &view.tracks {
            assert!(tr.is_muted);
        }
        assert!(view.inspector_state.is_muted);

        // 2. Toggle again -> all become unmuted
        let muted_again = view.toggle_all_tracks_mute();
        assert!(!muted_again);
        for tr in &view.tracks {
            assert!(!tr.is_muted);
        }
        assert!(!view.inspector_state.is_muted);
    }

    #[test]
    fn test_turn31_mixer_master_mono_audition_toggle() {
        let mut view = AwardWinningGuiView::new();

        assert!(!view.is_master_mono, "Default master mono audition should be false");

        // Toggle mono on
        let is_mono = view.toggle_master_mono();
        assert!(is_mono);
        assert!(view.is_master_mono);

        // Toggle mono off
        let is_mono_again = view.toggle_master_mono();
        assert!(!is_mono_again);
        assert!(!view.is_master_mono);
    }

    #[test]
    fn test_turn31_milestone33_multi_track_live_automation_evaluation() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        assert!(view.tracks.len() >= 2);
        let t1_id = view.tracks[0].id;
        let t2_id = view.tracks[1].id;

        // Create automation curve on Track 1: gain ramp 0.0 -> 1.0 across 16 beats
        let t1_gain_lane = format!("track_{}_gain", t1_id);
        timeline.lanes.insert(
            t1_gain_lane.clone(),
            AutomationLane {
                param_id: t1_gain_lane,
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: 0.2, interp: Interpolation::Linear },
                        AutomationPoint { beat: 16.0, value: 1.0, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        // Create automation curve on Track 2: pan sweep -1.0 -> +1.0 across 16 beats
        let t2_pan_lane = format!("track_{}_pan", t2_id);
        timeline.lanes.insert(
            t2_pan_lane.clone(),
            AutomationLane {
                param_id: t2_pan_lane,
                curve: AutomationCurve {
                    points: vec![
                        AutomationPoint { beat: 0.0, value: -1.0, interp: Interpolation::Linear },
                        AutomationPoint { beat: 16.0, value: 1.0, interp: Interpolation::Linear },
                    ],
                },
            },
        );

        // Selected track is Track 0
        view.selected_track_idx = 0;
        view.top_bar_state.is_playing = true;

        // Sync at beat 8.0 (halfway)
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 8.0, false);

        // Verify Track 1 gain evaluated (~0.6)
        let t1_gain = view.tracks[0].gain;
        assert!((t1_gain - 0.6).abs() < 1e-3, "Expected Track 1 gain ~0.6, got {}", t1_gain);

        // Verify Track 2 pan evaluated even though Track 1 is selected! (~0.0)
        let t2_pan = view.tracks[1].pan;
        assert!(t2_pan.abs() < 1e-3, "Expected Track 2 pan ~0.0, got {}", t2_pan);
    }

    #[test]
    fn test_turn31_milestone33_multi_track_parambus_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let project = ProjectConfig::default();
        let mut bus = ParamBus::new();
        let mut reg = AutomationRegistry::new();
        let mut timeline = AutomationTimeline::new();

        assert!(view.tracks.len() >= 2);
        let t1_id = view.tracks[0].id as u32;
        let t2_id = view.tracks[1].id as u32;

        // Register ParamBus slots for Track 1 and Track 2
        let t1_g_pid = ParamId(t1_id * 1000 + 200);
        let t1_p_pid = ParamId(t1_id * 1000 + 201);
        let t1_m_pid = ParamId(t1_id * 1000 + 202);
        let t1_s_pid = ParamId(t1_id * 1000 + 203);

        let t2_g_pid = ParamId(t2_id * 1000 + 200);
        let t2_p_pid = ParamId(t2_id * 1000 + 201);
        let t2_m_pid = ParamId(t2_id * 1000 + 202);
        let t2_s_pid = ParamId(t2_id * 1000 + 203);

        let master_pid = ParamId(9999);
        let mono_pid = ParamId(9998);

        for pid in [
            t1_g_pid, t1_p_pid, t1_m_pid, t1_s_pid,
            t2_g_pid, t2_p_pid, t2_m_pid, t2_s_pid,
            master_pid, mono_pid,
        ] {
            bus.register(pid, 0.0);
        }

        // Configure values
        view.tracks[0].gain = 0.85;
        view.tracks[0].pan = -0.45;
        view.tracks[0].is_muted = true;
        view.tracks[0].is_soloed = false;

        view.tracks[1].gain = 1.15;
        view.tracks[1].pan = 0.35;
        view.tracks[1].is_muted = false;
        view.tracks[1].is_soloed = true;

        view.top_bar_state.master_gain = 1.40;
        view.is_master_mono = true;

        // Dispatch
        view.sync_with_param_bus(&project, &bus, &mut reg, &mut timeline, 0.0, false);

        // Verify Track 1 ParamBus values
        assert_eq!(bus.get(t1_g_pid), Some(0.85));
        assert_eq!(bus.get(t1_p_pid), Some(-0.45));
        assert_eq!(bus.get(t1_m_pid), Some(1.0));
        assert_eq!(bus.get(t1_s_pid), Some(0.0));

        // Verify Track 2 ParamBus values
        assert_eq!(bus.get(t2_g_pid), Some(1.15));
        assert_eq!(bus.get(t2_p_pid), Some(0.35));
        assert_eq!(bus.get(t2_m_pid), Some(0.0));
        assert_eq!(bus.get(t2_s_pid), Some(1.0));

        // Verify Master Bus ParamBus values
        assert_eq!(bus.get(master_pid), Some(1.40));
        assert_eq!(bus.get(mono_pid), Some(1.0));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_top_bar::ModernViewTab;

    #[test]
    fn test_turn31_mixer_canvas_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        view.top_bar_state.active_tab = ModernViewTab::Mixer;

        // Render in Novice mode
        view.top_bar_state.is_pro_mode = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Render in Pro mode
        view.top_bar_state.is_pro_mode = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }
}
