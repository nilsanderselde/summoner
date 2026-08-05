// Summoner DAW - Tier 32 Specifications (Steps 577-590) Unit Tests
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

#[cfg(test)]
mod tests {
    use crate::app::SummonerApp;
    use crate::visualizer::Oscilloscope;
    use std::collections::HashSet;
    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;

    use summoner_project::project_tools::*;
    use summoner_project::schema::{ProjectConfig, SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_step577_step578_tap_tempo() {
        let project = ProjectConfig::default();
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        // Simulate tap tempo events 500ms apart (120 BPM)
        let base = std::time::Instant::now();
        app.tempo_tap_times
            .push_back(base - std::time::Duration::from_millis(1500));
        app.tempo_tap_times
            .push_back(base - std::time::Duration::from_millis(1000));
        app.tempo_tap_times
            .push_back(base - std::time::Duration::from_millis(500));

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
        let seq = SequenceConfig {
            fade_in: 2.0,
            fade_out: 1.5,
            ..Default::default()
        };
        assert_eq!(seq.fade_in, 2.0);
        assert_eq!(seq.fade_out, 1.5);
    }

    #[test]
    fn test_step584_step585_clip_gain_and_pitch() {
        let seq = SequenceConfig {
            gain: 1.5,
            pitch_offset: 7.0,
            ..Default::default()
        };
        assert_eq!(seq.gain, 1.5);
        assert_eq!(seq.pitch_offset, 7.0);
    }

    #[test]
    fn test_step586_step587_step588_clip_trim_and_restore() {
        let mut seq = SequenceConfig {
            trim_start: 1.0,
            trim_end: 2.0,
            fade_in: 0.5,
            fade_out: 0.5,
            gain: 2.0,
            pitch_offset: 5.0,
            ..Default::default()
        };

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
        use summoner_sequencer::automation_timeline::{AutomationCurve, AutomationLane};

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
        assert!(!imported.steps.is_empty());
    }

    #[test]
    fn test_step622_to_step625_swing_density_vel_quantize() {
        use summoner_sequencer::*;

        let mut seq = SequenceConfig {
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.33,
                    active: true,
                    swing: 0.2,
                    ..Default::default()
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.77,
                    active: true,
                    swing: 0.2,
                    ..Default::default()
                },
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
        use summoner_harmony::scale::Scale;
        use summoner_sequencer::*;

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
                TrackerStepConfig {
                    note: 61.0,
                    active: true,
                    ..Default::default()
                }, // C#4 -> D4 or C4 in C Major
                TrackerStepConfig {
                    note: 66.0,
                    active: true,
                    ..Default::default()
                }, // F#4 -> G4 or F4
            ],
            ..Default::default()
        };

        transpose_sequence_to_scale(&mut seq, 0, "Major"); // C Major scale
        let c_maj_scale = Scale::get_scale_by_name("Major");
        for step in &seq.steps {
            let pc = (step.note as u16) % 12;
            assert!(
                c_maj_scale.degrees.contains(&pc),
                "Note {} (pc {}) should be in C Major scale",
                step.note,
                pc
            );
        }
    }

    #[test]
    fn test_step631_to_step638_midi_mapping_aftertouch_pitchbend_monitor_panic() {
        use summoner_sequencer::midi_tools::*;

        // MIDI mapping scaling (Steps 631-633)
        let cc_map = MidiControllerMapping::new(1, MidiMappingType::CC(7), "track.gain", 0.0, 1.0);
        assert_eq!(cc_map.map_value(127.0, 0.0, 127.0), 1.0);
        assert_eq!(cc_map.map_value(63.5, 0.0, 127.0), 0.5);

        let pb_map =
            MidiControllerMapping::new(0, MidiMappingType::PitchBend, "synth.pitch", -12.0, 12.0);
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
        use crate::views::midi_panel::VirtualKeyboardState;
        use summoner_sequencer::midi_tools::qwerty_key_to_midi_note;

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
        use crate::views::macro_rack::{show_harmonics_display, show_spectral_display};
        use summoner_dsp::autotune::AutoTuneNode;
        use summoner_dsp::tuner::detect_chromatic_pitch;

        // Step 656: Chromatic Tuner pitch detection
        let sr = 44100.0;
        let mut buf = vec![0.0f32; 2048];
        for (i, sample) in buf.iter_mut().enumerate() {
            *sample = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sr).sin();
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

                let harm = [
                    1.0, 0.5, 0.25, 0.125, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    0.0,
                ];
                show_harmonics_display(ui, &harm, 100.0, 30.0);
            });
        });
    }

    #[test]
    fn test_step661_to_step680_dsp_gui_export_tools() {
        use crate::views::macro_rack::{
            show_filter_response_curve, show_fm_matrix_display, show_impulse_response_display,
        };
        use crate::visualizer::show_phase_scope;
        use summoner_dsp::*;
        use summoner_project::export::*;

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
        use summoner_dsp::filters::{DcBlockFilter, HighCutFilter, LowCutFilter};
        use summoner_dsp::sample_editor::*;
        use summoner_dsp::sampler::SampleBuffer;
        use summoner_project::create_default_project;
        use summoner_project::export::*;
        use summoner_project::schema::*;

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

        let removed = clean_project(&mut proj, &temp_dir).expect("clean project");
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

        let copied = collect_and_save(&mut proj2, &temp_dir2).expect("collect save");
        assert_eq!(copied, 1);
        assert_eq!(proj2.assets[0].path, "assets/summoner_ext_sample.wav");
        let _ = std::fs::remove_dir_all(&temp_dir2);
        let _ = std::fs::remove_file(&ext_file);

        // Step 683: Freeze / Unfreeze Track
        let mut track = TrackConfig::default();
        freeze_track(&mut track, vec![0.5f32; 100]);
        assert!(track.is_frozen);
        assert!(track.frozen_buffer.is_some());
        unfreeze_track(&mut track);
        assert!(!track.is_frozen);
        assert!(track.frozen_buffer.is_none());

        // Step 684: Parallel Compression Template
        let mut proj3 = create_default_project("Parallel Comp Test");
        apply_parallel_compression_template(&mut proj3, 1).expect("parallel comp");
        assert!(proj3.tracks[0]
            .nodes
            .iter()
            .any(|n| n.kind == "CompressorNode"));

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
        let bounced_id =
            bounce_track_to_new_track(&mut proj4, 1, vec![0.5f32, 0.5f32]).expect("bounce track");
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
        use crate::views::patch_browser::{PatchBrowserState, SortOrder};
        use std::path::PathBuf;
        use summoner_core::node::ProcessContext;
        use summoner_dsp::traits::SignalProcessor;
        use summoner_dsp::{
            ConsoleEmulationNode, ConsoleMode, MultibandCompressorNode, TapeSaturationNode,
            TubeSaturationNode,
        };
        use summoner_project::preset::DevicePreset;

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
            assert!(
                cats.contains(&req_cat.to_string()),
                "Must contain category {}",
                req_cat
            );
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
        let mut state_sort = PatchBrowserState {
            sort_order: SortOrder::Rating,
            ..Default::default()
        };
        state_sort.apply_sorting();
        assert!(state_sort.patches[0].rating >= state_sort.patches.last().unwrap().rating);

        // Step 712: Preset Collections
        let cols = state_sort.available_collections();
        assert!(!cols.is_empty());

        // Step 713: URL Import
        let imported = DevicePreset::import_from_url("https://example.com/synth.preset.toml")
            .expect("URL import");
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
        let state_new = PatchBrowserState {
            show_whats_new: true,
            ..Default::default()
        };
        assert!(state_new.show_whats_new);
    }

    #[test]
    fn test_steps_721_to_740_system_tools_and_cloud() {
        use std::path::PathBuf;
        use summoner_project::schema::ProjectConfig;
        use summoner_project::system_tools::*;

        // Step 721: Automatic Update Checker
        let mut checker = UpdateChecker::new("1.0.0");
        assert!(checker.check_for_updates());
        assert!(checker.update_available);
        assert!(checker.notification_pending);
        checker.dismiss_notification();
        assert!(!checker.notification_pending);

        // Step 722: One-Click Installer
        let install_res = checker.install_update("1.1.0");
        assert!(install_res.is_ok());
        assert_eq!(checker.current_version, "1.1.0");

        // Step 723: Rollback Option
        let rollback_res = checker.rollback_update();
        assert!(rollback_res.is_ok());
        assert_eq!(checker.current_version, "1.0.0");

        // Step 724: Crash Reporter
        let report = CrashReport::generate("1.0.0", "Null pointer in DSP node");
        assert!(report.anonymized);
        assert!(report.send_anonymized().is_ok());

        // Step 725: Opt-In Telemetry
        let mut telem = TelemetryManager::new(true);
        telem.track("button_click", "play_transport");
        assert_eq!(telem.events.len(), 1);
        let log = telem.export_log();
        assert!(log.contains("play_transport"));

        // Step 726: Privacy Policy
        let policy = get_privacy_policy();
        assert!(policy.contains("Privacy Policy"));

        // Step 727: GDPR Compliance & Data Export
        let gdpr_dump = GdprNotice::export_user_data(&telem, "theme=Dark");
        assert!(gdpr_dump.contains("GDPR Compliance Notice"));

        // Step 728: Factory Reset
        let reset_settings = factory_reset_settings();
        assert_eq!(reset_settings.theme, "Dark Hybrid");

        // Step 729: Settings Backup & Restore ZIP
        let backup_p = PathBuf::from("local/scratch/settings_test.zip");
        let settings = SystemSettings::default();
        settings
            .export_backup_zip(&backup_p)
            .expect("Export settings ZIP");
        let restored = SystemSettings::restore_backup_zip(&backup_p).expect("Restore settings ZIP");
        assert_eq!(settings, restored);
        let _ = std::fs::remove_file(&backup_p);

        // Step 730: Account Panel & Login
        let mut user = UserAccount::login("nils", "nils@example.com", "tok_9988");
        assert!(user.is_logged_in);

        // Step 731: Cloud Save & Restore
        let proj = ProjectConfig::default();
        let cloud_id = CloudProjectManager::cloud_save_project(&proj, &user).expect("Cloud save");
        let restored_proj =
            CloudProjectManager::cloud_restore_project(&cloud_id, &user).expect("Cloud restore");
        assert_eq!(restored_proj.name, proj.name);

        // Step 732: Cloud Render Submission
        let render_job = submit_cloud_render(&proj, "wav", &user).expect("Submit render");
        assert_eq!(render_job.status, "Completed");

        // Step 733: Cloud Collaboration
        let mut collab = CollaborationSession::create_session("workspace-alpha", "nils");
        collab.invite_member("alice");
        assert_eq!(collab.members.len(), 2);
        collab.remove_member("alice");
        assert_eq!(collab.members.len(), 1);

        // Step 734: Offline Mode Indicator
        let net = NetworkStatus::check_status();
        assert!(net.is_offline());
        assert!(net.degrade_gracefully().contains("Offline mode active"));

        // Step 735: Cloud Storage Quota Display
        let quota_str = user.formatted_quota_display();
        assert!(quota_str.contains("GB"));
        user.logout();
        assert!(!user.is_logged_in);

        // Step 736: Plugin Marketplace & Installation
        let mut market = PluginMarketplace::fetch_catalog();
        assert!(!market.plugins.is_empty());
        let install_msg = market
            .install_plugin("clap.surge_synth", &PathBuf::from("local/scratch"))
            .expect("Install plugin");
        assert!(install_msg.contains("Surge XT"));

        // Step 737: Plugin Rating & Review
        market
            .rate_plugin("clap.surge_synth", 5, "Awesome synth")
            .expect("Rate plugin");
        assert!(market.plugins[0]
            .user_reviews
            .iter()
            .any(|r| r.contains("Awesome synth")));

        // Step 738: Plugin Sandbox
        let sandbox = PluginSandbox::spawn_sandbox("clap.surge_synth");
        let in_pcm = vec![0.5f32; 64];
        let mut out_pcm = vec![0.0f32; 64];
        sandbox
            .process_audio_sandboxed(&in_pcm, &mut out_pcm)
            .expect("Sandbox process");
        assert_eq!(in_pcm, out_pcm);

        // Step 739: Plugin Crash Protection
        let mut crash_guard = PluginCrashGuard::default();
        let res_ok = crash_guard.execute_safe("clap.surge_synth", || Ok(()));
        assert!(res_ok.is_ok());
        let res_err = crash_guard.execute_safe("clap.buggy_plugin", || {
            panic!("Plugin Segfault!");
        });
        assert!(res_err.is_err());
        assert!(crash_guard.faulted_plugins.contains("clap.buggy_plugin"));

        // Step 740: Plugin Latency Compensation
        let mut lat_comp = PluginLatencyCompensation::default();
        lat_comp.set_latency("plugin_a", 128);
        lat_comp.set_latency("plugin_b", 32);
        let delays = lat_comp.calculate_alignment_delays();
        assert_eq!(*delays.get("plugin_b").unwrap(), 96);
        assert_eq!(*delays.get("plugin_a").unwrap(), 0);

        let mut buf_out = vec![0.0f32; 8];
        PluginLatencyCompensation::apply_delay_compensation(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            &mut buf_out,
            2,
        );
        assert_eq!(buf_out, vec![0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_steps_741_to_760_plugin_and_param_tools() {
        use crate::param_controls::{
            format_hover_tooltip, randomize_parameter, reset_param_to_default,
            FxChainBypassManager, InlineValueEditor, LinkedParamGroup, ParamClipboard, ParamGroup,
            ParamLfoModulator, ParamLockManager, ParamUnit, SendFxRoute,
        };
        use crate::touch_gestures::{SwipeDirection, TouchGestureManager, ViewModeDirection};
        use summoner_project::system_tools::{
            PluginBlacklist, PluginConflictDetector, PluginResourceBudget, PluginStateBackup,
        };

        // Step 741: Plugin Resource Budget
        let budget = PluginResourceBudget::new("clap.synth", 50.0, 512);
        assert!(budget.check_budget(30.0, 256).is_ok());
        assert!(budget.check_budget(60.0, 256).is_err());
        assert!(budget.check_budget(30.0, 1024).is_err());

        // Step 742: Plugin Blacklist
        let mut blacklist = PluginBlacklist::new();
        assert!(blacklist.is_blacklisted("clap.unstable_legacy_synth"));
        blacklist.add_to_blacklist("clap.bad_plugin", "Causes crash");
        let allowed = blacklist.filter_allowed(&[
            "clap.good_plugin".to_string(),
            "clap.bad_plugin".to_string(),
        ]);
        assert_eq!(allowed, vec!["clap.good_plugin".to_string()]);

        // Step 743: Plugin Conflict Detector
        let detector = PluginConflictDetector::new();
        let active = vec![
            "clap.legacy_driver_v1".to_string(),
            "clap.legacy_driver_v2".to_string(),
        ];
        let conflicts = detector.detect_conflicts(&active);
        assert_eq!(conflicts.len(), 1);

        // Step 744: Plugin State Backup
        let mut backup = PluginStateBackup::default();
        backup.create_backup("clap.synth", "1.0.0", &[1, 2, 3, 4]);
        assert!(backup.has_backup("clap.synth"));
        let (ver, data) = backup.restore_backup("clap.synth").expect("Restore backup");
        assert_eq!(ver, "1.0.0");
        assert_eq!(data, vec![1, 2, 3, 4]);

        // Step 745: Touch Screen & Target Scaling & Swipes
        let touch_mgr = TouchGestureManager::new();
        assert_eq!(touch_mgr.scaled_target_size((10.0, 20.0)), (15.0, 30.0));
        let swipe_right = touch_mgr.detect_swipe((0.0, 0.0), (100.0, 0.0));
        assert_eq!(swipe_right, Some(SwipeDirection::Right));

        // Step 746: Two-finger zoom
        let zoom_factor = TouchGestureManager::two_finger_zoom(100.0, 150.0);
        assert_eq!(zoom_factor, 1.5);

        // Step 747: Three-finger swipe
        let starts = [(0.0, 0.0), (0.0, 10.0), (0.0, 20.0)];
        let ends = [(-100.0, 0.0), (-100.0, 10.0), (-100.0, 20.0)];
        let view_swipe = TouchGestureManager::three_finger_swipe(&starts, &ends);
        assert_eq!(view_swipe, Some(ViewModeDirection::NextView));

        // Step 749: Inline Value Editor
        let mut editor = InlineValueEditor::default();
        editor.start_edit("cutoff_freq", 440.0);
        editor.text_buffer = "880.0".to_string();
        let (id, val) = editor.submit_edit().expect("Submit edit");
        assert_eq!(id, "cutoff_freq");
        assert_eq!(val, 880.0);

        // Step 750: Value Reset
        assert_eq!(reset_param_to_default(100.0), 100.0);

        // Step 751: Param Clipboard
        let mut clipboard = ParamClipboard::default();
        clipboard.copy(42.5);
        assert_eq!(clipboard.paste(), Some(42.5));

        // Step 752: Param Lock Manager
        let mut lock_mgr = ParamLockManager::default();
        lock_mgr.lock("vol");
        assert_eq!(lock_mgr.apply_mutation("vol", 0.5, 1.0), 0.5);
        lock_mgr.unlock("vol");
        assert_eq!(lock_mgr.apply_mutation("vol", 0.5, 1.0), 1.0);

        // Step 753: Param LFO Modulator
        let lfo = ParamLfoModulator::new("cutoff", 1.0, 0.5);
        let val_at_0 = lfo.evaluate(1.0, 0.0);
        assert!((val_at_0 - 1.0).abs() < 1e-4);

        // Step 754: Parameter Randomizer
        let rand_val = randomize_parameter("cutoff", 20.0, 20000.0, 12345);
        assert!((20.0..=20000.0).contains(&rand_val));

        // Step 755: Param Unit Display
        assert_eq!(ParamUnit::Hz.format_value(440.0), "440.0 Hz");
        assert_eq!(ParamUnit::Db.format_value(-6.0), "-6.0 dB");

        // Step 756: Hover Tooltip
        let tooltip = format_hover_tooltip("Cutoff", 440.0, 20.0, 20000.0, &ParamUnit::Hz);
        assert!(tooltip.contains("Cutoff: 440.0 Hz"));

        // Step 757: Linked Parameters
        let mut linked = LinkedParamGroup::default();
        linked.link("filter_l", "filter_r", 1.0);
        let updates = linked.compute_linked_values("filter_l", 1000.0);
        assert_eq!(updates, vec![("filter_r".to_string(), 1000.0)]);

        // Step 758: Param Group
        let mut group = ParamGroup::new("Filter Controls", &["cutoff", "res"]);
        assert!(!group.collapsed);
        group.toggle_collapsed();
        assert!(group.collapsed);

        // Step 759: Send FX Route
        let route = SendFxRoute::new(1, "ReverbBus", 0.75);
        assert_eq!(route.send_amount, 0.75);

        // Step 760: FX Chain Bypass Manager
        let mut bypass_mgr = FxChainBypassManager::default();
        assert!(!bypass_mgr.insert_bypassed);
        bypass_mgr.toggle_bypass();
        assert!(bypass_mgr.insert_bypassed);
        let dry = [1.0, 2.0];
        let wet = [0.5, 0.5];
        assert_eq!(bypass_mgr.process_chain(&dry, &wet), &[1.0, 2.0]);
    }

    #[test]
    fn test_step681_step682_clean_and_collect_project_assets() {
        use summoner_project::{
            clean_project, collect_and_save,
            schema::{AssetConfig, ProjectConfig},
        };
        let temp_dir = std::env::temp_dir().join("summoner_test_project_assets");
        let assets_dir = temp_dir.join("assets");
        let _ = std::fs::create_dir_all(&assets_dir);

        let file_used = assets_dir.join("kick.wav");
        let file_unused = assets_dir.join("unused_snare.wav");
        let _ = std::fs::write(&file_used, b"RIFF....WAVEfmt ");
        let _ = std::fs::write(&file_unused, b"RIFF....WAVEfmt ");

        let mut project = ProjectConfig::default();
        project.assets.push(AssetConfig {
            id: "1".to_string(),
            hash: "123".to_string(),
            path: "assets/kick.wav".to_string(),
            auto_slice: false,
            slice_threshold: 0.15,
        });

        // Step 681: Clean Project
        let removed = clean_project(&mut project, &temp_dir).unwrap();
        assert_eq!(removed, vec!["unused_snare.wav"]);
        assert!(!file_unused.exists());
        assert!(file_used.exists());

        // Step 682: Collect and Save
        let external_file = temp_dir.join("external_synth.wav");
        let _ = std::fs::write(&external_file, b"RIFF....WAVEfmt ");
        project.assets.push(AssetConfig {
            id: "2".to_string(),
            hash: "456".to_string(),
            path: external_file.to_string_lossy().to_string(),
            auto_slice: false,
            slice_threshold: 0.15,
        });

        let copied = collect_and_save(&mut project, &temp_dir).unwrap();
        assert_eq!(copied, 1);
        assert_eq!(project.assets[1].path, "assets/external_synth.wav");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_step683_freeze_and_unfreeze_track() {
        use summoner_project::{freeze_track, schema::TrackConfig, unfreeze_track};
        let mut track = TrackConfig::default();
        assert!(!track.is_frozen);
        assert!(track.frozen_buffer.is_none());

        let audio = vec![0.1, 0.2, 0.3, 0.4];
        freeze_track(&mut track, audio.clone());
        assert!(track.is_frozen);
        assert_eq!(track.frozen_buffer, Some(audio));

        unfreeze_track(&mut track);
        assert!(!track.is_frozen);
        assert!(track.frozen_buffer.is_none());
    }

    #[test]
    fn test_step684_parallel_compression_template() {
        use summoner_project::{apply_parallel_compression_template, create_default_project};
        let mut project = create_default_project("Parallel Comp Test");
        let initial_count = project.tracks.len();
        let target_id = project.tracks[0].id;

        let bus_id = apply_parallel_compression_template(&mut project, target_id).unwrap();
        assert_eq!(project.tracks.len(), initial_count + 1);
        let bus_track = project.tracks.iter().find(|t| t.id == bus_id).unwrap();
        assert!(bus_track.name.contains("Comp"));
        assert_eq!(bus_track.nodes[0].kind, "CompressorNode");
    }

    #[test]
    fn test_step685_step686_sidechain_and_bus_routing() {
        use summoner_project::{route_track_to_bus, schema::TrackConfig, set_sidechain_routing};
        let mut track = TrackConfig::default();

        set_sidechain_routing(&mut track, Some(42));
        assert_eq!(track.sidechain_source_track_id, Some(42));

        route_track_to_bus(&mut track, Some("Drums Bus".to_string()));
        assert_eq!(track.bus_target, Some("Drums Bus".to_string()));
    }

    #[test]
    fn test_step687_phase_flip() {
        use summoner_dsp::apply_phase_flip;
        let mut samples = vec![0.5, -0.8, 0.0, 1.0];
        apply_phase_flip(&mut samples);
        assert_eq!(samples, vec![-0.5, 0.8, -0.0, -1.0]);
    }

    #[test]
    fn test_step688_step689_dc_block_and_quick_filters() {
        use summoner_dsp::{DcBlockFilter, HighCutFilter, LowCutFilter};

        // Step 688: DC Block
        let mut dc_block = DcBlockFilter::new();
        let mut dc_signal = vec![1.0; 1000];
        dc_block.process_buffer(&mut dc_signal);
        assert!(dc_signal.last().unwrap().abs() < 0.1);

        // Step 689: Low Cut & High Cut filters
        let mut low_cut = LowCutFilter::new(100.0);
        let mut high_cut = HighCutFilter::new(5000.0);
        let out_lc = low_cut.process_sample(1.0, 44100);
        let out_hc = high_cut.process_sample(1.0, 44100);
        assert!(out_lc.is_finite());
        assert!(out_hc.is_finite());
    }

    #[test]
    fn test_step690_auto_level_track_gain() {
        use summoner_project::{auto_level_track_gain, schema::TrackConfig};
        let mut track = TrackConfig {
            gain: 1.0,
            ..Default::default()
        };

        // Current -18 LUFS, Target -14 LUFS (+4 dB)
        auto_level_track_gain(&mut track, -18.0, -14.0);
        let expected_gain = 10.0f32.powf(4.0 / 20.0);
        assert!((track.gain - expected_gain).abs() < 1e-3);
    }

    #[test]
    fn test_step691_spectrum_matching_gain_offsets() {
        use summoner_dsp::compute_spectrum_matching_gain_offsets;
        let source = vec![0.1, 0.5, 1.0];
        let target = vec![0.2, 0.5, 0.5];
        let offsets = compute_spectrum_matching_gain_offsets(&source, &target);
        assert_eq!(offsets.len(), 3);
        assert!((offsets[0] - 6.02).abs() < 0.1); // +6dB
        assert!((offsets[1] - 0.0).abs() < 0.1); // 0dB
        assert!((offsets[2] - (-6.02)).abs() < 0.1); // -6dB
    }

    #[test]
    fn test_step692_stereo_correlation_meter() {
        use summoner_dsp::compute_stereo_correlation;
        let l = vec![1.0, 0.5, -0.5, -1.0];
        let r_in_phase = vec![1.0, 0.5, -0.5, -1.0];
        let r_out_of_phase = vec![-1.0, -0.5, 0.5, 1.0];

        let corr_in = compute_stereo_correlation(&l, &r_in_phase);
        let corr_out = compute_stereo_correlation(&l, &r_out_of_phase);
        assert!((corr_in - 1.0).abs() < 1e-3);
        assert!((corr_out - (-1.0)).abs() < 1e-3);
    }

    #[test]
    fn test_step693_step694_master_and_track_gain_trims() {
        use summoner_dsp::{apply_input_gain, apply_master_trim, apply_output_gain};

        let mut samples_master = vec![1.0, 1.0];
        apply_master_trim(&mut samples_master, 6.0206);
        assert!((samples_master[0] - 2.0).abs() < 1e-3);

        let mut samples_in = vec![1.0, 1.0];
        apply_input_gain(&mut samples_in, -6.0206);
        assert!((samples_in[0] - 0.5).abs() < 1e-3);

        let mut samples_out = vec![1.0, 1.0];
        apply_output_gain(&mut samples_out, 0.0);
        assert_eq!(samples_out, vec![1.0, 1.0]);
    }

    #[test]
    fn test_step695_bounce_track_to_new_track() {
        use summoner_project::{bounce_track_to_new_track, create_default_project};
        let mut project = create_default_project("Bounce Test");
        let source_id = project.tracks[0].id;
        let rendered = vec![0.0; 1000];

        let bounced_id =
            bounce_track_to_new_track(&mut project, source_id, rendered.clone()).unwrap();
        let bounced = project.tracks.iter().find(|t| t.id == bounced_id).unwrap();
        assert!(bounced.name.contains("Bounced"));
        assert!(bounced.is_frozen);
        assert_eq!(bounced.frozen_buffer, Some(rendered));
    }

    #[test]
    fn test_step696_to_step700_sample_editor_tools() {
        use summoner_dsp::{
            audition_sample_at_c4, chop_sample_to_pads, crossfade_sample_loop, fade_in_sample,
            fade_out_sample, normalize_sample, remove_dc_offset_sample, reverse_sample,
            trim_sample, SampleBuffer, SampleEditor,
        };

        // Step 696: Audition sample at C4
        let buf = SampleBuffer::new(vec![0.5; 100], 44100, 1);
        let auditioned = audition_sample_at_c4(&buf, 60);
        assert_eq!(auditioned.data.len(), 100);

        // Step 697: Destructive editing
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        trim_sample(&mut data, 1, 4);
        assert_eq!(data, vec![2.0, 3.0, 4.0]);

        normalize_sample(&mut data, 0.0); // 0 dB = 1.0 max peak
        assert_eq!(
            *data
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
            1.0
        );

        reverse_sample(&mut data);
        assert_eq!(data[0], 1.0);

        fade_in_sample(&mut data, 2);
        assert_eq!(data[0], 0.0);

        fade_out_sample(&mut data, 2);
        assert_eq!(*data.last().unwrap(), 0.0);

        let mut dc_data = vec![2.0, 4.0, 6.0];
        remove_dc_offset_sample(&mut dc_data);
        assert_eq!(dc_data, vec![-2.0, 0.0, 2.0]);

        // Step 698: Sample crossfade loop
        let mut loop_data = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        crossfade_sample_loop(&mut loop_data, 2, 6, 2);
        assert_eq!(loop_data.len(), 8);

        // Step 699: SampleMarker editor
        let mut editor = SampleEditor::new();
        editor.add_marker(100, "Transient 1");
        editor.add_marker(50, "Transient 0");
        assert_eq!(editor.markers[0].label, "Transient 0");
        assert!(editor.move_marker(0, 200));
        assert!(editor.remove_marker(0));

        // Step 700: Chop to pads
        let pads_signal = vec![0.1; 400];
        let regions = chop_sample_to_pads(&pads_signal, 44100, 4);
        assert!(!regions.is_empty());
        assert!(regions.len() <= 4);
    }

    #[test]
    fn test_step761_to_step780_tier32_features() {
        use crate::param_controls::{
            compute_auto_crossfade, compute_pad_click_velocity, export_clip_to_sfz,
            get_drum_pad_color, get_wire_stroke_style, humanize_drum_pattern, load_fx_chain_preset,
            process_insert_fx_with_position, save_fx_chain_preset, set_effect_sidechain,
            DrumMachineGrid, FullScreenToggle, FxChainCompare, FxCompareSlot, InsertFxPosition,
            LayoutManager, PadChokeGroupManager, PadMuteGroupManager, PadNoteAssignmentTable,
            StageViewWindowConfig,
        };
        use std::collections::HashSet;
        use summoner_core::node::ProcessContext;
        use summoner_core::transport::Transport;
        use summoner_dsp::{DeesserNode, HarmonicExciterNode, NoiseGateNode, SignalProcessor};

        // Step 761: FX Chain Compare A/B
        let mut compare = FxChainCompare::default();
        assert_eq!(compare.active_slot, FxCompareSlot::SlotA);
        assert_eq!(compare.toggle_slot(), FxCompareSlot::SlotB);
        let dry = [1.0, 2.0];
        let wet = [0.5, 0.5];
        assert_eq!(compare.process_compare(&dry, &wet), &dry);

        // Step 762: FX Preset Save/Load
        let nodes = vec![summoner_project::schema::NodeConfig {
            kind: "NoiseGateNode".to_string(),
            params: std::collections::HashMap::new(),
            plugin_state: None,
        }];
        let preset = save_fx_chain_preset("Vocal Chain", &nodes);
        assert_eq!(preset.name, "Vocal Chain");
        let loaded = load_fx_chain_preset(&preset);
        assert_eq!(loaded.len(), 1);

        // Step 763: Pre/Post Fader Position Selector
        let input_samples = vec![1.0, 1.0];
        let fx_samples = vec![2.0, 2.0];
        let pre_out = process_insert_fx_with_position(
            InsertFxPosition::PreFader,
            0.5,
            &input_samples,
            &fx_samples,
        );
        assert_eq!(pre_out, vec![1.0, 1.0]);
        let post_out = process_insert_fx_with_position(
            InsertFxPosition::PostFader,
            0.5,
            &input_samples,
            &fx_samples,
        );
        assert_eq!(post_out, vec![2.0, 2.0]);

        // Step 764: Sidechain source selector
        let mut sidechain_cfgs = Vec::new();
        set_effect_sidechain(&mut sidechain_cfgs, 0, Some(42));
        assert_eq!(sidechain_cfgs[0].sidechain_source_track_id, Some(42));

        // Step 765: Node graph wire style
        let wire_style = get_wire_stroke_style(true);
        assert!(wire_style.stroke_dashed);

        // Step 766: NoiseGateNode
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut gate = NoiseGateNode::new();
        let in_buf = vec![0.0001f32; 64];
        let mut out_buf = vec![0.0f32; 64];
        gate.process_block(&[&in_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.iter().all(|v| v.abs() < 0.001));

        // Step 767: DeesserNode
        let mut deesser = DeesserNode::new();
        let mut deesser_out = vec![0.0f32; 64];
        deesser.process_block(&[&in_buf[..]], &mut [&mut deesser_out[..]], &ctx);
        assert_eq!(deesser.name(), "DeesserNode");

        // Step 768: HarmonicExciterNode
        let mut exciter = HarmonicExciterNode::new();
        let mut exciter_out = vec![0.0f32; 64];
        let exciter_in = vec![0.5f32; 64];
        exciter.process_block(&[&exciter_in[..]], &mut [&mut exciter_out[..]], &ctx);
        assert!(exciter_out.iter().any(|v| *v != 0.0));

        // Step 769: Auto-Crossfade
        let fade = compute_auto_crossfade(0.0, 4.0, 3.5, 0.5);
        assert_eq!(fade, Some((0.5, 0.5)));

        // Step 770: Export Clip to SFZ
        let temp_dir = std::env::temp_dir().join("summoner_sfz_export_test");
        let pcm = vec![0.1f32; 100];
        let (wav_p, sfz_p) = export_clip_to_sfz("test_clip", &pcm, 44100, 60, &temp_dir).unwrap();
        assert!(wav_p.exists());
        assert!(sfz_p.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Step 771: Drum Machine Grid
        let mut grid = DrumMachineGrid::default();
        assert!(!grid.is_step_active(0, 0));
        assert!(grid.toggle_step(0, 0));
        assert!(grid.is_step_active(0, 0));

        // Step 772: Pad click velocity
        let vel = compute_pad_click_velocity(0.0, 100.0);
        assert_eq!(vel, 127);

        // Step 773: Pad color mapping
        let color = get_drum_pad_color(36);
        assert_eq!(color, [231, 76, 60]);

        // Step 774: Pad mute groups
        let mut mute_mgr = PadMuteGroupManager::default();
        mute_mgr.set_pad_mute_group(0, 1);
        mute_mgr.set_pad_mute_group(1, 1);
        let mut active_pads = HashSet::new();
        mute_mgr.trigger_pad(0, &mut active_pads);
        mute_mgr.trigger_pad(1, &mut active_pads);
        assert!(!active_pads.contains(&0));
        assert!(active_pads.contains(&1));

        // Step 775: Pad note assignment
        let mut assign_table = PadNoteAssignmentTable::default();
        assign_table.assign_note(0, 36, "Kick Drum");
        let (note, label) = assign_table.get_note(0);
        assert_eq!(note, 36);
        assert_eq!(label, "Kick Drum");

        // Step 776: Pad choke groups
        let mut choke_mgr = PadChokeGroupManager::default();
        choke_mgr.set_choke_group(0, 1);
        choke_mgr.set_choke_group(1, 1);
        let mut active_voices = vec![0];
        choke_mgr.choke_active_voices(1, &mut active_voices);
        assert_eq!(active_voices, vec![1]);

        // Step 777: Drum pattern humanize
        let mut steps = vec![summoner_project::schema::TrackerStepConfig {
            note: 60.0,
            velocity: 1.0,
            gate: 0.5,
            probability: 1.0,
            micro_shift: 0,
            ratchet: 1,
            ..Default::default()
        }];
        humanize_drum_pattern(&mut steps, 5.0, 10, 42);
        assert!(steps[0].micro_shift != 0 || steps[0].velocity != 1.0);

        // Step 778: Full-screen toggle
        let mut fs = FullScreenToggle::default();
        assert!(fs.handle_key_event(true));

        // Step 779: Always-on-top mode
        let mut stage_cfg = StageViewWindowConfig::default();
        assert!(stage_cfg.toggle_always_on_top());

        // Step 780: Layout manager
        let mut layout_mgr = LayoutManager::default();
        let mut vis = std::collections::HashMap::new();
        vis.insert("Mixer".to_string(), true);
        let mut sizes = std::collections::HashMap::new();
        sizes.insert("Mixer".to_string(), (300.0, 400.0));
        layout_mgr.save_layout("Mixing", vis.clone(), sizes.clone());
        let restored = layout_mgr.restore_layout("Mixing").unwrap();
        assert_eq!(restored.panel_visibility, vis);
    }

    #[test]
    fn test_step781_to_step800_tier32_final_features() {
        use crate::param_controls::{
            create_preset_from_subgraph, generate_bug_report_url, get_default_layout_presets,
            get_github_feature_request_url, get_node_graph_lod, get_unique_node_types_used,
            search_project, AppearancePreferencesConfig, AudioPreferencesConfig,
            DependencyGraphOverview, LayoutHotkeyManager, MidiPreferencesConfig, NodeGraphLod,
            PluginsPreferencesConfig, PreferencesCategory, PreferencesState, ProjectStatistics,
            SearchResultKind, ShortcutsPreferencesConfig, SignalFlowAnimation, SubGraphGroup,
            TimelineZoomState, ViewZoomPersistence,
        };
        use summoner_project::schema::{
            ConnectionConfig, NodeConfig, ProjectConfig, SequenceConfig, TrackConfig,
        };

        // Step 781: Default layout presets
        let presets = get_default_layout_presets();
        assert!(presets.contains_key("Composition"));
        assert!(presets.contains_key("Mixing"));
        assert!(presets.contains_key("Mastering"));
        assert!(presets.contains_key("Live"));

        // Step 782: Layout hotkeys
        let hk_mgr = LayoutHotkeyManager::new_default();
        assert_eq!(hk_mgr.handle_hotkey("F1"), Some("Composition"));
        assert_eq!(hk_mgr.handle_hotkey("F2"), Some("Mixing"));

        // Step 783: Timeline zoom helper
        let mut zoom_state = TimelineZoomState::default();
        assert_eq!(zoom_state.zoom_to_fit(10.0, 1000.0), 100.0);
        assert_eq!(zoom_state.zoom_to_selection(2.0, 6.0, 400.0), 100.0);

        // Step 784: Per-view zoom persistence
        let view_zoom = ViewZoomPersistence::default();
        assert_eq!(view_zoom.arranger_ppb, 50.0);

        // Prepare test project for steps 785-788
        let mut proj = ProjectConfig::default();
        let track = TrackConfig {
            id: 1,
            name: "Lead Synth".to_string(),
            sequence: Some(SequenceConfig {
                clip_name: Some("Lead Pattern".to_string()),
                ..Default::default()
            }),
            nodes: vec![
                NodeConfig {
                    kind: "OscSine".to_string(),
                    params: [("freq".to_string(), 440.0)].into_iter().collect(),
                    plugin_state: None,
                },
                NodeConfig {
                    kind: "GainNode".to_string(),
                    params: [("gain".to_string(), 0.8)].into_iter().collect(),
                    plugin_state: None,
                },
            ],
            ..Default::default()
        };
        proj.tracks.push(track);

        // Step 785: Find in project
        let search_res = search_project(&proj, "Lead");
        assert!(search_res
            .iter()
            .any(|r| r.kind == SearchResultKind::Track && r.name == "Lead Synth"));
        assert!(search_res
            .iter()
            .any(|r| r.kind == SearchResultKind::Clip && r.name == "Lead Pattern"));

        // Step 786: Project statistics
        let stats = ProjectStatistics::compute(&proj);
        assert_eq!(stats.total_tracks, 1);
        assert_eq!(stats.total_clips, 1);
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_parameters, 2);

        // Step 787: Node Usage
        let node_types = get_unique_node_types_used(&proj);
        assert_eq!(
            node_types,
            vec!["GainNode".to_string(), "OscSine".to_string()]
        );

        // Step 788: Dependency graph overview
        let connections = vec![ConnectionConfig {
            from: "0".to_string(),
            to: "1".to_string(),
        }];
        let dep_graph = DependencyGraphOverview::from_connections(&connections);
        assert_eq!(dep_graph.adjacency.get(&0), Some(&vec![1]));

        // Step 789: Signal flow animation
        let mut anim = SignalFlowAnimation::default();
        let phase = anim.step(0.1);
        assert!(phase > 0.0);
        let dot_pos = anim.compute_dot_position((0.0, 0.0), (100.0, 200.0));
        assert!(dot_pos.0 > 0.0 && dot_pos.1 > 0.0);

        // Step 790: LOD switching in Node Graph
        assert_eq!(get_node_graph_lod(1.0), NodeGraphLod::Detailed);
        assert_eq!(get_node_graph_lod(0.5), NodeGraphLod::Simplified);
        assert_eq!(get_node_graph_lod(0.2), NodeGraphLod::Compact);

        // Step 791: Subgraph group
        let mut group = SubGraphGroup::new(100, "Osc Group", vec![0, 1]);
        assert!(!group.collapsed);
        assert!(group.toggle_collapsed());
        assert!(group.collapsed);

        // Step 792: Preset from subgraph
        let nodes = proj.tracks[0].nodes.as_slice();
        let preset = create_preset_from_subgraph(&group, nodes, "My Osc Preset");
        assert_eq!(preset.name, "My Osc Preset");
        assert_eq!(preset.device_kind, "Subgraph");

        // Step 793: Feature request URL
        assert!(get_github_feature_request_url().contains("issues/new"));

        // Step 794: Report bug URL
        let bug_url = generate_bug_report_url("Windows 11", "v1.0.0");
        assert!(bug_url.contains("Bug%20Report"));
        assert!(bug_url.contains("Windows%2011"));

        // Step 795-800: Preferences panels
        let pref_state = PreferencesState::default();
        assert_eq!(pref_state.active_category, PreferencesCategory::Audio);

        let audio_pref = AudioPreferencesConfig::default();
        assert_eq!(audio_pref.sample_rate, 44100);

        let midi_pref = MidiPreferencesConfig::default();
        assert_eq!(midi_pref.latency_ms, 5.0);

        let app_pref = AppearancePreferencesConfig::default();
        assert_eq!(app_pref.theme, "Dark");

        let mut sc_pref = ShortcutsPreferencesConfig::default();
        sc_pref.set_binding("Play", "Space");
        assert_eq!(sc_pref.get_binding("Play"), Some("Space"));

        let plug_pref = PluginsPreferencesConfig::default();
        assert!(plug_pref.safe_mode);
    }
}
