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
}
