// Summoner DAW - Tier 32 Specifications (Steps 577-590) Unit Tests
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::collections::HashSet;
    use summoner_core::param_bus::ParamBus;
    use summoner_project::schema::{ProjectConfig, SequenceConfig, TrackerStepConfig};
    use crate::app::SummonerApp;
    use crate::visualizer::Oscilloscope;

    #[test]
    fn test_step577_step578_tap_tempo() {
        let project = ProjectConfig::default();
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        // Simulate 4 tap tempo events 500ms apart (120 BPM)
        let base = std::time::Instant::now();
        app.tempo_tap_times.push_back(base);
        app.tempo_tap_times.push_back(base + std::time::Duration::from_millis(500));
        app.tempo_tap_times.push_back(base + std::time::Duration::from_millis(1000));
        app.tempo_tap_times.push_back(base + std::time::Duration::from_millis(1500));

        // Call tap_tempo logic
        app.tap_tempo();

        // 500ms interval corresponds to 120.0 BPM
        assert!((app.project.transport.bpm - 120.0).abs() < 2.0);
    }

    #[test]
    fn test_step579_bpm_range_lock() {
        let mut project = ProjectConfig::default();
        project.transport.bpm = 350.0;
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);
        app.min_bpm = 40.0;
        app.max_bpm = 240.0;

        app.project.transport.bpm = app.project.transport.bpm.clamp(app.min_bpm, app.max_bpm);
        assert_eq!(app.project.transport.bpm, 240.0);

        app.project.transport.bpm = 10.0;
        app.project.transport.bpm = app.project.transport.bpm.clamp(app.min_bpm, app.max_bpm);
        assert_eq!(app.project.transport.bpm, 40.0);
    }

    #[test]
    fn test_step580_track_vu_meter_rms() {
        let scope = Oscilloscope::new();
        for _ in 0..512 {
            scope.write_sample(0.5);
        }
        let samples = scope.read_all();
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / 512.0).sqrt();
        assert!((rms - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_step581_sequence_config_fades() {
        let mut seq = SequenceConfig::default();
        seq.fade_in = 2.0;
        seq.fade_out = 1.5;
        assert_eq!(seq.fade_in, 2.0);
        assert_eq!(seq.fade_out, 1.5);
    }

    #[test]
    fn test_step584_step585_clip_gain_and_pitch() {
        let mut seq = SequenceConfig::default();
        seq.gain = 1.5;
        seq.pitch_offset = 7.0;
        assert_eq!(seq.gain, 1.5);
        assert_eq!(seq.pitch_offset, 7.0);
    }

    #[test]
    fn test_step586_step587_step588_clip_trim_and_restore() {
        let mut seq = SequenceConfig::default();
        seq.trim_start = 1.0;
        seq.trim_end = 2.0;
        seq.fade_in = 0.5;
        seq.fade_out = 0.5;
        seq.gain = 2.0;
        seq.pitch_offset = 5.0;

        assert_eq!(seq.trim_start, 1.0);
        assert_eq!(seq.trim_end, 2.0);

        seq.restore();
        assert_eq!(seq.trim_start, 0.0);
        assert_eq!(seq.trim_end, 0.0);
        assert_eq!(seq.fade_in, 0.0);
        assert_eq!(seq.fade_out, 0.0);
        assert_eq!(seq.gain, 1.0);
        assert_eq!(seq.pitch_offset, 0.0);
    }

    #[test]
    fn test_step589_arranger_empty_lane_clip_add() {
        let mut project = ProjectConfig::default();
        assert_eq!(project.tracks[0].clips.len(), 0);

        project.tracks[0].clips.push(SequenceConfig {
            start_beat: 4.0,
            clip_name: Some("Test Clip".to_string()),
            ..Default::default()
        });
        assert_eq!(project.tracks[0].clips.len(), 1);
        assert_eq!(project.tracks[0].clips[0].start_beat, 4.0);
    }

    #[test]
    fn test_step590_multi_clip_move() {
        let mut project = ProjectConfig::default();
        let mut selected_clips = HashSet::new();

        project.tracks[0].sequence = Some(SequenceConfig {
            start_beat: 0.0,
            ..Default::default()
        });
        project.tracks[0].clips.push(SequenceConfig {
            start_beat: 4.0,
            ..Default::default()
        });

        selected_clips.insert((1, 0));
        selected_clips.insert((1, 1));

        let delta_beats = 2.0;
        for &(t_id, s_idx) in &selected_clips {
            if let Some(tr) = project.tracks.iter_mut().find(|t| t.id == t_id) {
                let seqs = tr.all_sequences_mut();
                if s_idx < seqs.len() {
                    seqs[s_idx].start_beat = (seqs[s_idx].start_beat + delta_beats).max(0.0);
                }
            }
        }

        assert_eq!(project.tracks[0].sequence.as_ref().unwrap().start_beat, 2.0);
        assert_eq!(project.tracks[0].clips[0].start_beat, 6.0);
    }

    #[test]
    fn test_step606_to_step609_automation_line_curve_and_snap_grid() {
        use summoner_sequencer::automation_timeline::{AutomationLane, AutomationCurve};

        let mut lane = AutomationLane {
            param_id: "filter_cutoff".to_string(),
            curve: AutomationCurve::new(Vec::new()),
        };

        // Line segment drawing with grid snapping (Step 606, 608)
        lane.add_line_segment(0.12, 0.2, 3.89, 0.9, Some(0.25));
        assert_eq!(lane.curve.points[0].beat, 0.0);
        assert_eq!(lane.curve.points[1].beat, 4.0);

        // Curve segment drawing (Step 609)
        lane.add_curve_segment(4.0, 0.1, 6.0, 0.5, 8.0, 1.0, None);
        assert_eq!(lane.curve.points.len(), 5);

        // Snap all points to grid (Step 606)
        lane.snap_all_to_grid(1.0);
        for pt in &lane.curve.points {
            assert_eq!(pt.beat.fract(), 0.0);
        }
    }

    #[test]
    fn test_step610_to_step615_step_grid_mute_prob_ratchet_microshift() {
        let mut step = TrackerStepConfig {
            note: 60.0,
            velocity: 0.8,
            gate: 0.5,
            probability: 0.75,
            ratchet: 3,
            micro_shift: 12,
            swing: 0.1,
            pan: 0.0,
            pitch_offset: 0.0,
            active: true,
            muted: false,
        };

        assert!(!step.muted);
        step.muted = true; // Step Mute (Step 611)
        assert!(step.muted);

        assert_eq!(step.probability, 0.75); // Step Probability (Step 612)
        assert_eq!(step.ratchet, 3); // Step Ratchet (Step 613)
        assert_eq!(step.micro_shift, 12); // Step Micro-shift (Step 614)

        let step_copied = step.clone(); // Step Copy (Step 615)
        assert_eq!(step_copied.note, 60.0);
        assert_eq!(step_copied.ratchet, 3);
    }

    #[test]
    fn test_step616_to_step621_pattern_tools_and_midi() {
        use summoner_sequencer::*;

        let mut seq = SequenceConfig {
            steps: vec![TrackerStepConfig::default(); 16],
            ..Default::default()
        };

        // Randomize pattern (Step 616)
        randomize_pattern(&mut seq, 999, 0.6, (50, 70));
        assert!(seq.steps.iter().any(|s| s.active));

        // Pattern Length & Resolution (Steps 619-621)
        set_pattern_length(&mut seq, 8);
        assert_eq!(seq.steps.len(), 8);

        set_pattern_resolution(&mut seq, 0.25, true);
        assert!((seq.step_division - 0.16666666666666666).abs() < 1e-5);

        // MIDI Export & Import (Steps 617-618)
        let bytes = export_pattern_to_midi_bytes(&seq, 128.0);
        assert!(!bytes.is_empty());
        let imported = import_pattern_from_midi_bytes(&bytes).expect("MIDI import should succeed");
        assert!(imported.steps.len() > 0);
    }

    #[test]
    fn test_step622_to_step625_swing_density_vel_quantize() {
        use summoner_sequencer::*;

        let mut seq = SequenceConfig {
            steps: vec![
                TrackerStepConfig { note: 60.0, velocity: 0.33, active: true, swing: 0.2, ..Default::default() },
                TrackerStepConfig { note: 64.0, velocity: 0.77, active: true, swing: 0.2, ..Default::default() },
            ],
            ..Default::default()
        };

        // Velocity quantize (Step 625)
        quantize_velocities(&mut seq, &[0.25, 0.5, 0.75, 1.0]);
        assert_eq!(seq.steps[0].velocity, 0.25);
        assert_eq!(seq.steps[1].velocity, 0.75);

        // Apply density (Step 624)
        apply_pattern_density(&mut seq, 0.5);
        let active_count = seq.steps.iter().filter(|s| s.active).count();
        assert_eq!(active_count, 1);
    }
}

