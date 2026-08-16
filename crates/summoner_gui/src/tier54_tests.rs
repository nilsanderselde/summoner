// Summoner DAW - Tier 54 GUI Milestones Unit Test Suite (Steps 1401-1410)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::reverb_space_view::{
        ReverbAlgorithm, ReverbSpaceView, REVERB_OBJECT_HIT_RADIUS,
    };
    use crate::views::ribbon_controller_view::{
        RibbonControllerView, RibbonQuantizeMode, RIBBON_PUCK_HIT_RADIUS,
    };
    use crate::views::stereo_widener_view::{
        StereoWidenerMode, StereoWidenerView, STEREO_CROSSOVER_HANDLE_HIT_RADIUS,
    };
    use crate::views::tape_emulator_view::{
        TapeEmulatorView, TapeFormulation, TapeSpeedIps, TAPE_DRIVE_HANDLE_HIT_RADIUS,
    };
    use crate::views::vocoder_matrix_view::{
        VocoderBandState, VocoderMatrixView, VOCODER_BAND_HANDLE_HIT_RADIUS, VOCODER_MAX_FREQ_HZ,
        VOCODER_MIN_FREQ_HZ, VOCODER_NUM_BANDS,
    };

    #[test]
    fn test_step_1401_1406_vocoder_matrix_bandwidth_and_hit_targets() {
        let mut vocoder = VocoderMatrixView::new();
        assert_eq!(vocoder.bands.len(), VOCODER_NUM_BANDS);
        assert_eq!(vocoder.bands.len(), 64);

        let canvas = Rect::new(0.0, 50.0, 800.0, 220.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(VOCODER_BAND_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(VOCODER_BAND_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency Range and Bandwidth Monotonicity
        assert!((vocoder.bands[0].center_freq_hz - VOCODER_MIN_FREQ_HZ).abs() < 1.0);
        assert!((vocoder.bands[63].center_freq_hz - VOCODER_MAX_FREQ_HZ).abs() < 10.0);

        for i in 0..VOCODER_NUM_BANDS - 1 {
            assert!(
                vocoder.bands[i].center_freq_hz < vocoder.bands[i + 1].center_freq_hz,
                "Bands must be strictly monotonically increasing in center frequency"
            );
            assert!(
                vocoder.bands[i].bandwidth_hz > 0.0,
                "Bandwidth must be strictly positive"
            );
        }

        // 3. Screen coordinate <-> Band index mapping
        for idx in [0, 16, 32, 48, 63] {
            let bx = vocoder.band_idx_to_screen_x(idx, canvas);
            let roundtrip_idx = vocoder.screen_x_to_band_idx(bx, canvas);
            assert_eq!(roundtrip_idx, idx);
        }

        // 4. Hit Testing on Band Handles
        let b16_x = vocoder.band_idx_to_screen_x(16, canvas);
        let hit = vocoder.hit_test_band_handle((b16_x, canvas.y + 50.0), canvas);
        assert_eq!(hit, Some(16));

        let miss = vocoder.hit_test_band_handle((b16_x, canvas.y - 40.0), canvas);
        assert_eq!(miss, None);

        // 5. Formant Tilt Gain Calculation
        vocoder.formant_tilt_db_oct = 3.0; // +3 dB / octave
        let gain_1k = vocoder.calculate_formant_tilt_gain_db(32); // pivot ~1 kHz
        assert!(gain_1k.abs() < 3.0);
        let gain_high = vocoder.calculate_formant_tilt_gain_db(63);
        let gain_low = vocoder.calculate_formant_tilt_gain_db(0);
        assert!(
            gain_high > gain_low,
            "High bands must have higher gain with positive formant tilt"
        );

        // 6. Freeze Buffer State & Deterministic Spectrum Capture
        vocoder.bands[10].modulator_level = 0.85;
        vocoder.toggle_freeze_buffer();
        assert!(vocoder.freeze_buffer_enabled);
        assert_eq!(vocoder.frozen_modulator_spectrum[10], 0.85);

        // Live updates should be ignored while frozen
        vocoder.update_band_levels(10, 0.20, 0.50);
        assert_eq!(vocoder.frozen_modulator_spectrum[10], 0.85);

        vocoder.toggle_freeze_buffer();
        assert!(!vocoder.freeze_buffer_enabled);

        // 7. Band States & Parameter Modifiers
        vocoder.bands[5].state = VocoderBandState::Solo;
        assert_eq!(vocoder.bands[5].state, VocoderBandState::Solo);

        // 8. Deterministic ASCII Render
        let ascii = vocoder.render_ascii(32);
        assert_eq!(ascii.len(), 32);
    }

    #[test]
    fn test_step_1402_1407_ribbon_controller_touch_coordinates_and_pitch_quantization() {
        let mut ribbon = RibbonControllerView::new();
        assert_eq!(ribbon.base_note_midi, 36); // C2
        assert_eq!(ribbon.num_octaves, 4); // C2 to C6 = 48 semitones

        let canvas = Rect::new(0.0, 50.0, 800.0, 200.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(RIBBON_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RIBBON_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Continuous Pitch <-> Screen Coordinate Mapping
        let test_pitch = 60.0; // Middle C (C4)
        let sx = ribbon.pitch_to_screen_x(test_pitch, canvas);
        let roundtrip_pitch = ribbon.screen_x_to_pitch(sx, canvas);
        assert!((roundtrip_pitch - test_pitch).abs() < 0.05);

        // Test pitch bounds
        let left_pitch = ribbon.screen_x_to_pitch(canvas.x, canvas);
        assert_eq!(left_pitch, 36.0); // C2
        let right_pitch = ribbon.screen_x_to_pitch(canvas.x + canvas.width, canvas);
        assert_eq!(right_pitch, 84.0); // C6

        // 3. Timbre / Y-Axis <-> Screen Y Coordinate Mapping
        let test_timbre = 0.75;
        let sy = ribbon.timbre_to_screen_y(test_timbre, canvas);
        let roundtrip_timbre = ribbon.screen_y_to_timbre(sy, canvas);
        assert!((roundtrip_timbre - test_timbre).abs() < 0.01);

        // 4. Quantization Modes
        // Semitone quantization
        ribbon.quantize_mode = RibbonQuantizeMode::SemitoneStep;
        assert_eq!(ribbon.apply_quantization(60.4), 60.0);
        assert_eq!(ribbon.apply_quantization(60.6), 61.0);

        // Major scale quantization (Root C: C, D, E, F, G, A, B -> 60, 62, 64, 65, 67, 69, 71)
        ribbon.quantize_mode = RibbonQuantizeMode::MajorScale;
        assert_eq!(ribbon.apply_quantization(61.0), 60.0); // C# snaps to C or D (nearest)
        assert_eq!(ribbon.apply_quantization(63.0), 62.0); // D# snaps to D or E

        // Microtonal 19-EDO quantization
        ribbon.quantize_mode = RibbonQuantizeMode::MicrotonalEdo(19);
        let q_edo19 = ribbon.apply_quantization(60.0);
        assert!((q_edo19 - 60.0).abs() < 1e-4);

        // 5. Multi-Touch Hit Detection and Polyphonic Allocation
        let touch0 = &ribbon.touches[0];
        let t0_x = ribbon.pitch_to_screen_x(touch0.note_pitch, canvas);
        let t0_y = ribbon.timbre_to_screen_y(touch0.y_timbre, canvas);

        let hit = ribbon.hit_test_touch((t0_x, t0_y), canvas);
        assert_eq!(hit, Some(0));

        let miss = ribbon.hit_test_touch((t0_x + 60.0, t0_y + 60.0), canvas);
        assert_ne!(miss, Some(0));

        // Add 4th touch far from existing touches
        let new_idx = ribbon.trigger_touch((canvas.x + 650.0, canvas.y + 100.0), canvas, 0.90);
        assert_eq!(new_idx, 3);
        assert_eq!(ribbon.touches.len(), 4);

        // 6. Note String Formatting
        assert_eq!(RibbonControllerView::pitch_to_note_string(60.0), "C4");
        assert_eq!(RibbonControllerView::pitch_to_note_string(69.0), "A4");

        // 7. Deterministic ASCII Render
        let ascii = ribbon.render_ascii(32);
        assert_eq!(ascii.len(), 32);
        assert!(ascii.contains('*'));
    }

    #[test]
    fn test_step_1403_stereo_widener_and_vector_scope() {
        let mut widener = StereoWidenerView::new();
        assert_eq!(widener.mode, StereoWidenerMode::FrequencySplitMultiband);
        assert_eq!(widener.low_band_width_pct, 0.0); // Mono Bass
        assert_eq!(widener.crossover_low_hz, 180.0);
        assert_eq!(widener.crossover_high_hz, 4000.0);

        let canvas = Rect::new(0.0, 50.0, 800.0, 200.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(STEREO_CROSSOVER_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(STEREO_CROSSOVER_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency <-> Coordinate Mapping
        let freq_1k = 1000.0;
        let norm_x = StereoWidenerView::freq_to_norm_x(freq_1k);
        let roundtrip_f = StereoWidenerView::norm_x_to_freq(norm_x);
        assert!((roundtrip_f - freq_1k).abs() < 1.0);

        // 3. Crossover Handle Hit Testing
        let low_x =
            canvas.x + StereoWidenerView::freq_to_norm_x(widener.crossover_low_hz) * canvas.width;
        let hit_low = widener.hit_test_crossover_handle((low_x, canvas.y + 50.0), canvas);
        assert_eq!(hit_low, Some(0));

        let high_x =
            canvas.x + StereoWidenerView::freq_to_norm_x(widener.crossover_high_hz) * canvas.width;
        let hit_high = widener.hit_test_crossover_handle((high_x, canvas.y + 50.0), canvas);
        assert_eq!(hit_high, Some(1));

        // 4. Vector Scope Lissajous Coordinate Projection
        let center = (200.0, 200.0);
        let radius = 100.0;

        // Pure Mono (L = 1.0, R = 1.0) -> Side = 0 (x = center), Mid = sqrt(2) (y is top)
        let (mono_x, mono_y) = StereoWidenerView::project_vector_scope(1.0, 1.0, center, radius);
        assert!((mono_x - center.0).abs() < 1.0);
        assert!(mono_y < center.1); // Y is up

        // Pure Out-of-Phase (L = 1.0, R = -1.0) -> Mid = 0 (y = center), Side = sqrt(2) (x is right)
        let (oop_x, oop_y) = StereoWidenerView::project_vector_scope(1.0, -1.0, center, radius);
        assert!((oop_y - center.1).abs() < 1.0);
        assert!(oop_x > center.0); // X is right

        // 5. Phase Correlation Updates
        widener.update_correlation(0.95);
        assert_eq!(widener.phase_correlation, 0.95);
        widener.update_correlation(-0.40);
        assert_eq!(widener.phase_correlation, -0.40);

        // 6. Deterministic ASCII Render
        let ascii = widener.render_ascii(30);
        assert_eq!(ascii.len(), 30);
        assert!(ascii.contains('M')); // Mono bass
        assert!(ascii.contains('|')); // Crossover
        assert!(ascii.contains('W')); // Width
        assert!(ascii.contains('H')); // High width
    }

    #[test]
    fn test_step_1404_reverb_space_ray_tracing_and_damping() {
        let mut reverb = ReverbSpaceView::new();
        assert_eq!(reverb.algorithm, ReverbAlgorithm::HallConcert);
        assert_eq!(reverb.room_size_m, 35.0);
        assert_eq!(reverb.decay_time_rt60_s, 2.8);

        let canvas = Rect::new(0.0, 50.0, 400.0, 300.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(REVERB_OBJECT_HIT_RADIUS >= 22.0) };
        const { assert!(REVERB_OBJECT_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Room Coordinate <-> Screen Transformations
        let norm_pos = (0.35, 0.65);
        let screen_pos = reverb.room_to_screen_pos(norm_pos, canvas);
        let roundtrip_norm = reverb.screen_to_room_pos(screen_pos, canvas);
        assert!((roundtrip_norm.0 - norm_pos.0).abs() < 0.01);
        assert!((roundtrip_norm.1 - norm_pos.1).abs() < 0.01);

        // 3. Source and Listener Hit Detection
        let src_screen = reverb.room_to_screen_pos(reverb.source_pos_norm, canvas);
        let hit_src = reverb.hit_test_source_or_listener(src_screen, canvas);
        assert_eq!(hit_src, Some(0)); // Source

        let lis_screen = reverb.room_to_screen_pos(reverb.listener_pos_norm, canvas);
        let hit_lis = reverb.hit_test_source_or_listener(lis_screen, canvas);
        assert_eq!(hit_lis, Some(1)); // Listener

        let miss =
            reverb.hit_test_source_or_listener((src_screen.0 + 50.0, src_screen.1 + 50.0), canvas);
        assert_eq!(miss, None);

        // 4. Acoustic Ray Tracing Simulation
        assert_eq!(reverb.ray_traces.len(), 12);
        for ray in &reverb.ray_traces {
            assert!(ray.points.len() >= 2);
            for pt in &ray.points {
                assert!((0.0..=1.0).contains(&pt.0));
                assert!((0.0..=1.0).contains(&pt.1));
            }
        }

        // 5. Algorithm Presets
        for algo in [
            ReverbAlgorithm::PlateReverb,
            ReverbAlgorithm::HallConcert,
            ReverbAlgorithm::CathedralSpace,
            ReverbAlgorithm::RoomChamber,
            ReverbAlgorithm::ShimmerEthereal,
            ReverbAlgorithm::NonLinearGate,
        ] {
            reverb.algorithm = algo;
            assert!(!algo.name().is_empty());
        }

        // 6. Deterministic ASCII Render
        let ascii = reverb.render_ascii(20, 10);
        assert!(ascii.contains('S')); // Source
        assert!(ascii.contains('L')); // Listener
    }

    #[test]
    fn test_step_1405_tape_emulator_hysteresis_and_wow_flutter() {
        let mut tape = TapeEmulatorView::new();
        assert_eq!(tape.speed_ips, TapeSpeedIps::Ips15);
        assert_eq!(tape.formulation, TapeFormulation::Master900HighOutput);
        assert_eq!(tape.input_drive_db, 6.0);

        let canvas = Rect::new(0.0, 50.0, 400.0, 200.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(TAPE_DRIVE_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(TAPE_DRIVE_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Magnetic Saturation Non-linear Transfer Function (B-H Saturation)
        let linear_out = tape.evaluate_saturation(0.1);
        let hard_out = tape.evaluate_saturation(1.0);
        assert!(hard_out.abs() > linear_out.abs());
        assert!(hard_out.abs() <= 2.0);

        // Symmetry test for un-biased input
        let pos_out = tape.evaluate_saturation(0.5);
        let neg_out = tape.evaluate_saturation(-0.5);
        assert!((pos_out + neg_out).abs() < 1e-4);

        // 3. Spool Physics Animation Stepping
        let init_rot = tape.spool_rotation_rad;
        tape.step_physics(0.1);
        assert_ne!(tape.spool_rotation_rad, init_rot);

        // 4. Drive Handle Hit Testing
        let norm_drive = ((tape.input_drive_db + 12.0) / 36.0).clamp(0.0, 1.0);
        let hx = canvas.x + norm_drive * canvas.width;
        let hy = canvas.y + canvas.height * 0.5;
        assert!(tape.hit_test_drive_handle((hx, hy), canvas));
        assert!(!tape.hit_test_drive_handle((hx + 50.0, hy + 50.0), canvas));

        // 5. Tape Speeds & Frequency Response
        assert_eq!(TapeSpeedIps::Ips3_75.frequency_cutoff_hz(), 12000.0);
        assert_eq!(TapeSpeedIps::Ips30.frequency_cutoff_hz(), 24000.0);

        // 6. Formulations Headroom
        assert!(
            TapeFormulation::Master900HighOutput.max_output_level_db()
                > TapeFormulation::TypeINormal.max_output_level_db()
        );

        // 7. Deterministic ASCII Render
        let ascii = tape.render_ascii(20);
        assert_eq!(ascii.len(), 20);
        assert!(ascii.contains('O')); // Spool
        assert!(ascii.contains('H')); // Head
    }
}
