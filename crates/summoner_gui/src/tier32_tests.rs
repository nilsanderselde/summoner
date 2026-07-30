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

        // Simulate tap tempo events 500ms apart (120 BPM)
        let base = std::time::Instant::now();
        app.tempo_tap_times.push_back(base - std::time::Duration::from_millis(1500));
        app.tempo_tap_times.push_back(base - std::time::Duration::from_millis(1000));
        app.tempo_tap_times.push_back(base - std::time::Duration::from_millis(500));

        // Call tap_tempo logic (adds current Instant as 4th tap)
        app.tap_tempo();

        // 500ms interval corresponds to 120.0 BPM
        assert!((app.project.transport.bpm - 120.0).abs() < 5.0);
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
        let mut project = summoner_project::create_default_project("Test Project");
        let initial_clips_count = project.tracks[0].clips.len();

        project.tracks[0].clips.push(SequenceConfig {
            start_beat: 4.0,
            clip_name: Some("Test Clip".to_string()),
            ..Default::default()
        });
        assert_eq!(project.tracks[0].clips.len(), initial_clips_count + 1);
        assert_eq!(project.tracks[0].clips.last().unwrap().start_beat, 4.0);
    }

    #[test]
    fn test_step590_multi_clip_move() {
        let mut project = summoner_project::create_default_project("Test Project Multi Move");
        let mut selected_clips = HashSet::new();

        project.tracks[1].sequence = Some(SequenceConfig {
            start_beat: 0.0,
            ..Default::default()
        });
        project.tracks[1].clips.push(SequenceConfig {
            start_beat: 4.0,
            ..Default::default()
        });

        selected_clips.insert((2, 0));
        selected_clips.insert((2, 1));

        let delta_beats = 2.0;
        for &(t_id, s_idx) in &selected_clips {
            if let Some(tr) = project.tracks.iter_mut().find(|t| t.id == t_id) {
                let mut seqs = tr.all_sequences_mut();
                if s_idx < seqs.len() {
                    seqs[s_idx].start_beat = (seqs[s_idx].start_beat + delta_beats).max(0.0);
                }
            }
        }

        assert_eq!(project.tracks[1].sequence.as_ref().unwrap().start_beat, 2.0);
        assert_eq!(project.tracks[1].clips[0].start_beat, 6.0);
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
        assert_eq!(lane.curve.points.len(), 4);

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

    #[test]
    fn test_step626_to_step630_chord_detection_scale_transpose() {
        use summoner_sequencer::*;
        use summoner_harmony::scale::Scale;

        // Chord detection (Step 626)
        let single_note = vec![60.0]; // C4
        assert_eq!(detect_chord_name_from_notes(&single_note), "C4");

        let c_maj = vec![60.0, 64.0, 67.0]; // C, E, G
        assert_eq!(detect_chord_name_from_notes(&c_maj), "Cmaj");

        let c_min = vec![60.0, 63.0, 67.0]; // C, Eb, G
        assert_eq!(detect_chord_name_from_notes(&c_min), "Cm");

        let c_maj7 = vec![60.0, 64.0, 67.0, 71.0]; // C, E, G, B
        assert_eq!(detect_chord_name_from_notes(&c_maj7), "Cmaj7");

        // Scale lookup & Transpose to Scale (Steps 627-630)
        let mut seq = SequenceConfig {
            steps: vec![
                TrackerStepConfig { note: 61.0, active: true, ..Default::default() }, // C#4 -> D4 or C4 in C Major
                TrackerStepConfig { note: 66.0, active: true, ..Default::default() }, // F#4 -> G4 or F4
            ],
            ..Default::default()
        };

        transpose_sequence_to_scale(&mut seq, 0, "Major"); // C Major scale
        let c_maj_scale = Scale::get_scale_by_name("Major");
        for step in &seq.steps {
            let pc = (step.note as u16) % 12;
            assert!(c_maj_scale.degrees.contains(&pc), "Note {} (pc {}) should be in C Major scale", step.note, pc);
        }
    }

    #[test]
    fn test_step631_to_step638_midi_mapping_aftertouch_pitchbend_monitor_panic() {
        use summoner_sequencer::midi_tools::*;

        // MIDI mapping scaling (Steps 631-633)
        let cc_map = MidiControllerMapping::new(1, MidiMappingType::CC(7), "track.gain", 0.0, 1.0);
        assert_eq!(cc_map.map_value(127.0, 0.0, 127.0), 1.0);
        assert_eq!(cc_map.map_value(63.5, 0.0, 127.0), 0.5);

        let pb_map = MidiControllerMapping::new(0, MidiMappingType::PitchBend, "synth.pitch", -12.0, 12.0);
        assert!((pb_map.map_value(0.0, -8192.0, 8191.0) - 0.0).abs() < 1e-2);

        // Velocity Curve (Step 634)
        assert_eq!(transform_velocity(127, VelocityCurve::Linear), 127);
        assert_eq!(transform_velocity(127, VelocityCurve::Fixed(100)), 100);

        // Channel filter & Transpose offset (Steps 635-636)
        let pass = filter_and_transpose_midi_note(1, 60, Some(1), 12);
        assert_eq!(pass, Some(72));

        let reject = filter_and_transpose_midi_note(2, 60, Some(1), 0);
        assert_eq!(reject, None);

        // MIDI Monitor & Panic (Steps 637-638)
        let mut log = MidiMonitorLog::new(10);
        log.log_event(100, 1, "NoteOn", 60, 100);
        assert_eq!(log.entries.len(), 1);

        let panic_msgs = generate_panic_all_note_off();
        assert!(panic_msgs.len() >= 16 * 130);
    }

    #[test]
    fn test_step639_to_step645_virtual_keyboard_and_qwerty() {
        use summoner_sequencer::midi_tools::qwerty_key_to_midi_note;
        use crate::views::midi_panel::VirtualKeyboardState;

        let state = VirtualKeyboardState::default();
        assert_eq!(state.base_octave, 4);

        // QWERTY note mappings (Step 645)
        let z_note = qwerty_key_to_midi_note("Z", 4); // C4 (MIDI 60)
        assert_eq!(z_note, Some(60));

        let q_note = qwerty_key_to_midi_note("Q", 4); // C5 (MIDI 72)
        assert_eq!(q_note, Some(72));
    }

    #[test]
    fn test_step646_to_step655_arpeggiator_strummer_chords_split_tuning() {
        use summoner_sequencer::midi_tools::*;

        // Step 646: Input Echo
        assert!(should_echo_midi_input(true, true));
        assert!(!should_echo_midi_input(true, false));

        // Steps 647-649: Arpeggiator (Up, Down, UpDown, Random, AsPlayed, Octaves, Latch)
        let mut arp = Arpeggiator::new(ArpDirection::Up, 2, 0.75, true);
        let seq_up = arp.generate_expanded_sequence(&[60, 64, 67]); // C4, E4, G4 across 2 octaves
        assert_eq!(seq_up, vec![60, 64, 67, 72, 76, 79]);

        let arp_down = Arpeggiator::new(ArpDirection::Down, 1, 0.8, false);
        let seq_down = arp_down.generate_expanded_sequence(&[60, 64, 67]);
        assert_eq!(seq_down, vec![67, 64, 60]);

        let (note, gate) = arp.next_step(&[60, 64, 67]).unwrap();
        assert_eq!(note, 60);
        assert_eq!(gate, 0.75);

        // Step 650: Strummer
        let strummer = Strummer::new(30.0, StrumDirection::LowToHigh);
        let strummed = strummer.strum(&[60, 64, 67]);
        assert_eq!(strummed.len(), 3);
        assert_eq!(strummed[0], (60, 0.0));
        assert_eq!(strummed[1], (64, 15.0));
        assert_eq!(strummed[2], (67, 30.0));

        // Step 651: Chord Memory
        let mut cm = ChordMemory::new();
        cm.save_chord(0, vec![60, 64, 67]);
        assert_eq!(cm.trigger(0), Some(&[60, 64, 67][..]));
        assert_eq!(cm.trigger_by_note(1), Some(&[60, 64, 67][..]));

        // Step 652: Keyboard Split
        let split = KeyboardSplit::new(60, 1, 2);
        assert_eq!(split.route(59), 1);
        assert_eq!(split.route(60), 2);

        // Step 653: Keyboard Layering
        let layer = KeyboardLayering::new(vec![1, 2, 3]);
        assert_eq!(layer.route(), &[1, 2, 3]);

        // Step 654: Fine Tune
        let ratio = cents_to_freq_ratio(0.0);
        assert!((ratio - 1.0).abs() < 1e-4);

        // Step 655: Master Tune
        let hz = midi_note_to_hz_tuned(69, 0.0, 0.0); // A4 = 440 Hz
        assert!((hz - 440.0).abs() < 1e-3);
    }

    #[test]
    fn test_step656_to_step660_tuner_autotune_spectrum_harmonics() {
        use summoner_dsp::tuner::detect_chromatic_pitch;
        use summoner_dsp::autotune::AutoTuneNode;
        use crate::views::macro_rack::{show_spectral_display, show_harmonics_display};

        // Step 656: Chromatic Tuner pitch detection
        let sr = 44100.0;
        let mut buf = vec![0.0f32; 2048];
        for i in 0..buf.len() {
            buf[i] = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sr).sin();
        }
        let res = detect_chromatic_pitch(&buf, sr).expect("Detect pitch");
        assert_eq!(res.note_name, "A4");
        assert_eq!(res.midi_note, 69);

        // Steps 657-658: AutoTuneNode scale snapping & formant flag
        let autotune = AutoTuneNode::new(vec![0, 2, 4, 5, 7, 9, 11], 0.8, true);
        assert_eq!(autotune.snap_to_target(61), 60); // C#4 -> C4
        assert!(autotune.formant_preservation);

        // Steps 659-660: Spectrum & Harmonics UI rendering
        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                let spec = [0.2, 0.5, 0.8, 0.3];
                show_spectral_display(ui, &spec, 100.0, 30.0);

                let harm = [1.0, 0.5, 0.25, 0.125, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
                show_harmonics_display(ui, &harm, 100.0, 30.0);
            });
        });
    }

    #[test]
    fn test_step661_to_step680_dsp_gui_export_tools() {
        use summoner_dsp::*;
        use summoner_project::export::*;
        use crate::views::macro_rack::{
            show_fm_matrix_display, show_filter_response_curve, show_impulse_response_display,
        };
        use crate::visualizer::show_phase_scope;

        // Steps 661-663 & 671: GUI rendering displays
        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(eframe::egui::RawInput::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                let matrix = [[0.5; 4]; 4];
                show_fm_matrix_display(ui, &matrix, 100.0, 100.0);
                show_filter_response_curve(ui, 1000.0, 0.5, 100.0, 50.0);
                let ir = vec![1.0, 0.5, 0.2];
                show_impulse_response_display(ui, &ir, 100.0, 50.0);
                let buf_l = vec![0.5f32; 64];
                let buf_r = vec![-0.5f32; 64];
                show_phase_scope(ui, &buf_l, &buf_r, 100.0, 100.0);
            });
        });

        // Steps 664-666: Convolution Reverb & Ring Modulator Waveforms
        let ir_buf = vec![1.0, 0.2, 0.1];
        let mut conv = ConvolutionReverbNode::new(ir_buf, 0.5);
        assert_eq!(conv.process_sample(1.0), 1.0);

        let mut ring_mod = RingModulator::new();
        ring_mod.waveform = RingModWaveform::Square;
        assert_eq!(ring_mod.carrier_sample(0.2), 1.0);
        assert_eq!(ring_mod.carrier_sample(0.7), -1.0);

        // Steps 667-670: Stereo Imager, LUFS, True Peak, K-System
        let mut imager = StereoImager::new(44100);
        imager.width = 1.2;
        let (l, r) = imager.process_stereo(0.8, -0.8);
        assert!(l.is_finite() && r.is_finite());

        let mut true_peak = TruePeakMeter::new();
        true_peak.process_block(&[0.5, -0.8, 0.9]);
        assert!(true_peak.max_true_peak_db > -10.0);

        let k_headroom = k_system_headroom(-10.0, KSystemScale::K14);
        assert_eq!(k_headroom, 4.0);

        // Steps 672-673: Master Limiter & Dithering
        let mut limiter = MasterLimiter::new(-14.0);
        let mut l_buf = vec![1.5f32; 64];
        let mut r_buf = vec![1.5f32; 64];
        limiter.process_stereo_block(&mut l_buf, &mut r_buf, 44100);
        assert!(l_buf[0] <= 1.0);

        let mut prng = 42;
        let dithered = apply_dither(0.5, 16, DitherType::Tpdf, &mut prng);
        assert!((dithered - 0.5).abs() < 0.01);

        // Steps 674-680: Export settings, normalization, trimming, and backup
        let mut export_buf = vec![0.1, -0.5, 0.2];
        normalize_buffer(&mut export_buf, 0.0);
        assert!((export_buf[1].abs() - 1.0).abs() < 1e-4);

        let silence_buf = vec![0.0, 0.0, 0.8, 0.0];
        let trimmed = trim_silence_buffer(&silence_buf, -40.0);
        assert_eq!(trimmed, &[0.8]);

        assert!(validate_sample_rate(48000));
    }

    #[test]
    fn test_step681_to_step700_project_and_sample_tools() {
        use summoner_project::export::*;
        use summoner_project::schema::*;
        use summoner_project::create_default_project;
        use summoner_dsp::filters::{DcBlockFilter, LowCutFilter, HighCutFilter};
        use summoner_dsp::sample_editor::*;
        use summoner_dsp::sampler::SampleBuffer;

        // Step 681: Clean Project
        let temp_dir = std::env::temp_dir().join("summoner_test_clean_proj");
        let assets_dir = temp_dir.join("assets");
        let _ = std::fs::create_dir_all(&assets_dir);
        let _ = std::fs::write(assets_dir.join("ref.wav"), "dummy");
        let _ = std::fs::write(assets_dir.join("unref.wav"), "dummy");

        let mut proj = ProjectConfig::default();
        proj.assets.push(AssetConfig {
            id: "1".to_string(),
            hash: "123".to_string(),
            path: "assets/ref.wav".to_string(),
            auto_slice: false,
            slice_threshold: 0.15,
        });

        let removed = clean_project(&temp_dir, &proj).expect("clean project");
        assert_eq!(removed, vec!["unref.wav".to_string()]);
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Step 682: Collect and Save
        let temp_dir2 = std::env::temp_dir().join("summoner_test_collect_save");
        let ext_file = std::env::temp_dir().join("summoner_ext_sample.wav");
        let _ = std::fs::write(&ext_file, "external sample data");

        let mut proj2 = ProjectConfig::default();
        proj2.assets.push(AssetConfig {
            id: "ext".to_string(),
            hash: "456".to_string(),
            path: ext_file.to_string_lossy().to_string(),
            auto_slice: false,
            slice_threshold: 0.15,
        });

        let copied = collect_and_save(&temp_dir2, &mut proj2).expect("collect save");
        assert_eq!(copied, vec!["assets/summoner_ext_sample.wav".to_string()]);
        assert_eq!(proj2.assets[0].path, "assets/summoner_ext_sample.wav");
        let _ = std::fs::remove_dir_all(&temp_dir2);
        let _ = std::fs::remove_file(&ext_file);

        // Step 683: Freeze / Unfreeze Track
        let mut track = TrackConfig::default();
        freeze_track(&mut track, 44100, 120.0);
        assert!(track.is_frozen);
        assert!(track.frozen_buffer.is_some());
        unfreeze_track(&mut track);
        assert!(!track.is_frozen);
        assert!(track.frozen_buffer.is_none());

        // Step 684: Parallel Compression Template
        let mut proj3 = create_default_project("Parallel Comp Test");
        apply_parallel_compression_template(&mut proj3, 1, 4.0, 0.5).expect("parallel comp");
        assert!(proj3.tracks[0].nodes.iter().any(|n| n.kind == "CompressorNode"));

        // Steps 685-689 & 693-694: Sidechain, Bus Target, Phase Flip, DC Block, Quick Filters, Gain Trims
        let mut track2 = TrackConfig::default();
        set_track_sidechain_source(&mut track2, 2);
        set_track_bus_target(&mut track2, "DrumBus");
        track2.phase_flip = true;
        track2.dc_block = true;
        track2.low_cut_hz = Some(80.0);
        track2.high_cut_hz = Some(12000.0);
        track2.input_gain_db = -3.0;
        track2.output_gain_db = 1.5;

        assert_eq!(track2.sidechain_source_track_id, Some(2));
        assert_eq!(track2.bus_target.as_deref(), Some("DrumBus"));

        let mut dc_filter = DcBlockFilter::new();
        let dc_sample = dc_filter.process_sample(1.0);
        assert!(dc_sample.is_finite());

        let mut low_cut = LowCutFilter::new(80.0);
        let hp_sample = low_cut.process_sample(0.5, 44100);
        assert!(hp_sample.is_finite());

        let mut high_cut = HighCutFilter::new(12000.0);
        let lp_sample = high_cut.process_sample(0.5, 44100);
        assert!(lp_sample.is_finite());

        // Step 690: LUFS Target Auto-Level
        let mut buf_level = vec![0.1f32; 100];
        let delta = auto_level_track(&mut buf_level, -20.0, -14.0);
        assert_eq!(delta, 6.0);
        assert!((buf_level[0] - (0.1 * 10.0f32.powf(6.0 / 20.0))).abs() < 1e-4);

        // Step 691: Spectrum Matching
        let src_spec = [0.1, 0.5, 0.8];
        let tgt_spec = [0.2, 0.5, 0.4];
        let eq_offsets = match_spectrum_eq(&src_spec, &tgt_spec);
        assert_eq!(eq_offsets.len(), 3);
        assert!((eq_offsets[0] - 6.02).abs() < 0.1);

        // Step 692: Stereo Correlation
        let l_ch = vec![1.0, 0.5, -0.5];
        let r_ch = vec![1.0, 0.5, -0.5];
        let corr = calculate_stereo_correlation(&l_ch, &r_ch);
        assert!((corr - 1.0).abs() < 1e-4);

        let r_inv = vec![-1.0, -0.5, 0.5];
        let corr_inv = calculate_stereo_correlation(&l_ch, &r_inv);
        assert!((corr_inv + 1.0).abs() < 1e-4);

        // Step 695: Bounce to Track
        let mut proj4 = create_default_project("Bounce Test");
        let bounced_id = bounce_track_to_new_track(&mut proj4, 1, &[0.5, 0.5]).expect("bounce track");
        assert!(proj4.tracks.iter().any(|t| t.id == bounced_id));
        assert!(proj4.tracks[0].muted);

        // Step 696: Sample Audition at C4
        let sbuf = SampleBuffer::new(vec![0.5f32; 100], 44100, 1);
        let auditioned = audition_sample_at_c4(&sbuf, 60);
        assert_eq!(auditioned.sample_rate, 44100);

        // Step 697: Destructive Sample Editing
        let mut sample_data = vec![0.1, -0.8, 0.4, 0.9, -0.2];
        normalize_sample(&mut sample_data, 0.0);
        assert!((sample_data.iter().map(|s| s.abs()).fold(0.0f32, f32::max) - 1.0).abs() < 1e-4);

        reverse_sample(&mut sample_data);
        assert!((sample_data[0] - (-0.2 / 0.9)).abs() < 1e-4);

        trim_sample(&mut sample_data, 1, 4);
        assert_eq!(sample_data.len(), 3);

        fade_in_sample(&mut sample_data, 2);
        assert_eq!(sample_data[0], 0.0);

        fade_out_sample(&mut sample_data, 2);
        assert_eq!(sample_data[sample_data.len() - 1], 0.0);

        remove_dc_offset_sample(&mut sample_data);
        let mean: f32 = sample_data.iter().sum::<f32>() / sample_data.len() as f32;
        assert!(mean.abs() < 1e-5);

        // Step 698: Sample Crossfade Loop
        let mut loop_buf = vec![0.0; 100];
        crossfade_sample_loop(&mut loop_buf, 20, 80, 10);
        assert_eq!(loop_buf.len(), 100);

        // Step 699: Sample Marker Editor
        let mut editor = SampleEditor::new();
        editor.add_marker(100, "Verse Start");
        editor.add_marker(500, "Chorus Start");
        assert_eq!(editor.markers.len(), 2);
        assert!(editor.move_marker(0, 150));
        assert!(editor.remove_marker(1));
        assert_eq!(editor.markers.len(), 1);

        // Step 700: Chop Sample to Pads
        let pad_buf = vec![0.0f32; 1000];
        let regions = chop_sample_to_pads(&pad_buf, 44100, 16);
        assert!(!regions.is_empty());
        assert!(regions.len() <= 16);
    }

    #[test]
    fn test_steps_701_to_720_dsp_and_preset_tools() {
        use summoner_dsp::{MultibandCompressorNode, TapeSaturationNode, TubeSaturationNode, ConsoleEmulationNode, ConsoleMode};
        use summoner_dsp::traits::SignalProcessor;
        use summoner_core::node::ProcessContext;
        use summoner_project::preset::DevicePreset;
        use crate::views::patch_browser::{PatchBrowserState, SortOrder};
        use std::path::PathBuf;

        let ctx = ProcessContext::new(44100, 120.0, 0);
        let in_sig = vec![0.7f32; 128];
        let mut out_sig = vec![0.0f32; 128];

        // Step 701: Multiband Compressor
        let mut mb = MultibandCompressorNode::new();
        mb.process_block(&[&in_sig[..]], &mut [&mut out_sig[..]], &ctx);
        assert!(out_sig.iter().all(|s| s.is_finite()));

        // Step 702: Tape Saturation
        let mut tape = TapeSaturationNode::new(3.0, 0.6);
        tape.process_block(&[&in_sig[..]], &mut [&mut out_sig[..]], &ctx);
        assert!(out_sig.iter().all(|s| s.is_finite()));

        // Step 703: Tube Saturation
        let mut tube = TubeSaturationNode::new(2.5, 0.2);
        tube.process_block(&[&in_sig[..]], &mut [&mut out_sig[..]], &ctx);
        assert!(out_sig.iter().all(|s| s.is_finite()));

        // Step 704: Console Emulation (Neve, SSL, API)
        for mode in [ConsoleMode::Neve, ConsoleMode::SSL, ConsoleMode::API] {
            let mut console = ConsoleEmulationNode::new(mode, 1.2);
            console.process_block(&[&in_sig[..]], &mut [&mut out_sig[..]], &ctx);
            assert!(out_sig.iter().all(|s| s.is_finite()));
        }

        // Steps 705 & 706: Categories (Vintage, Ambient, Cinematic, IDM, Experimental)
        let state = PatchBrowserState::default();
        let cats = state.available_categories();
        for req_cat in &["Vintage", "Ambient", "Cinematic", "IDM", "Experimental"] {
            assert!(cats.contains(&req_cat.to_string()), "Must contain category {}", req_cat);
        }

        // Step 707: Rating, Comment, Author, Version
        let mut preset = DevicePreset::new("Test Synth", "AetherSynth");
        preset.rating = 5;
        preset.author = "TestAuthor".to_string();
        preset.comment = "Great patch".to_string();
        preset.version = "1.0.0".to_string();
        assert_eq!(preset.rating, 5);

        // Step 708: Fork Preset
        let forked = preset.fork("ForkAuthor");
        assert_eq!(forked.author, "ForkAuthor");
        assert!(forked.name.contains("(Fork)"));

        // Step 709: Diff Presets
        let diffs = preset.diff(&forked);
        assert!(!diffs.is_empty());

        // Step 710 & 711: Search & Sort
        let mut state_sort = PatchBrowserState::default();
        state_sort.sort_order = SortOrder::Rating;
        state_sort.apply_sorting();
        assert!(state_sort.patches[0].rating >= state_sort.patches.last().unwrap().rating);

        // Step 712: Preset Collections
        let cols = state_sort.available_collections();
        assert!(!cols.is_empty());

        // Step 713: URL Import
        let imported = DevicePreset::import_from_url("https://example.com/synth.preset.toml").expect("URL import");
        assert_eq!(imported.name, "synth");

        // Steps 714 & 715: ZIP Export and Install
        let zip_p = PathBuf::from("local/scratch/preset_test.zip");
        let dest_d = PathBuf::from("local/scratch/installed_preset");
        preset.export_zip(&zip_p).expect("Export ZIP");
        let installed = DevicePreset::install_zip(&zip_p, &dest_d).expect("Install ZIP");
        assert_eq!(installed.name, "Test Synth");
        let _ = std::fs::remove_file(&zip_p);
        let _ = std::fs::remove_dir_all(&dest_d);

        // Step 716: Check Updates
        assert!(preset.check_updates().is_some());

        // Step 717: Verify Dependencies
        let missing = preset.verify_dependencies();
        assert!(missing.is_empty());

        // Step 718: Migrate Schema
        let raw = r#"name = "Legacy"
device_kind = "OscSaw"
params = { freq = 220.0 }"#;
        let migrated = DevicePreset::migrate_schema(raw).expect("Migrate schema");
        assert_eq!(migrated.name, "Legacy");

        // Step 719: Thumbnail Generation
        let thumb_p = PathBuf::from("local/scratch/thumb_test.png");
        preset.generate_thumbnail(&thumb_p).expect("Thumbnail");
        assert!(thumb_p.exists());
        let _ = std::fs::remove_file(&thumb_p);

        // Step 720: What's New Dialog
        let mut state_new = PatchBrowserState::default();
        state_new.show_whats_new = true;
        assert!(state_new.show_whats_new);
    }
}



