// Summoner DAW - Tier 53 GUI Milestones Unit Test Suite (Steps 1381-1390)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::ambisonic_radar_view::{
        AmbisonicFormat, AmbisonicRadarView, TrajectoryShape, AMBISONIC_PUCK_HIT_RADIUS,
    };
    use crate::views::granular_cloud_view::{
        GrainWindowShape, GranularCloudView, EMITTER_PUCK_HIT_RADIUS,
    };
    use crate::views::loop_slicer_view::{GlitchPadMode, LoopSlicerView, SLICE_MARKER_HIT_RADIUS};
    use crate::views::spectral_morph_view::{
        SpectralMorphMode, SpectralMorphView, VowelFormant, MORPH_CROSSFADER_HANDLE_RADIUS,
    };
    use crate::views::transient_shaper_view::{
        BandMuteSolo, TransientShaperView, CROSSOVER_HANDLE_HIT_RADIUS,
    };

    #[test]
    fn test_step_1381_1386_transient_shaper_crossover_bounds_and_hit_targets() {
        let mut shaper = TransientShaperView::new();
        assert_eq!(shaper.bands.len(), 3);
        assert_eq!(shaper.crossover_low_mid_hz, 250.0);
        assert_eq!(shaper.crossover_mid_high_hz, 3500.0);

        let canvas = Rect::new(0.0, 50.0, 800.0, 200.0);

        // 1. Log-frequency coordinate transformations
        let freq_1k = 1000.0_f32;
        let norm_x = TransientShaperView::freq_to_norm_x(freq_1k);
        let roundtrip_f = TransientShaperView::norm_x_to_freq(norm_x);
        assert!(
            (roundtrip_f - freq_1k).abs() < 1.0,
            "Frequency roundtrip error too large: {roundtrip_f} vs {freq_1k}"
        );

        let screen_x = shaper.freq_to_screen_x(freq_1k, canvas);
        assert!((canvas.x..=canvas.x + canvas.width).contains(&screen_x));
        let roundtrip_screen_f = shaper.screen_x_to_freq(screen_x, canvas);
        assert!((roundtrip_screen_f - freq_1k).abs() < 1.0);

        // 2. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(CROSSOVER_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(CROSSOVER_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // Test Hit Detection on low-mid crossover handle
        let lm_x = shaper.freq_to_screen_x(shaper.crossover_low_mid_hz, canvas);
        let hit_lm = shaper.hit_test_crossover_handle((lm_x, canvas.y + 50.0), canvas);
        assert_eq!(hit_lm, Some(0));

        // Test Hit Detection on mid-high crossover handle
        let mh_x = shaper.freq_to_screen_x(shaper.crossover_mid_high_hz, canvas);
        let hit_mh = shaper.hit_test_crossover_handle((mh_x, canvas.y + 50.0), canvas);
        assert_eq!(hit_mh, Some(1));

        // Test Miss Detection far away
        let miss = shaper.hit_test_crossover_handle((lm_x + 60.0, canvas.y + 50.0), canvas);
        assert_eq!(miss, None);

        // 3. Band Region Hit Testing
        assert_eq!(shaper.hit_test_band_region(canvas.x + 10.0, canvas), 0); // Low
        assert_eq!(shaper.hit_test_band_region((lm_x + mh_x) * 0.5, canvas), 1); // Mid
        assert_eq!(
            shaper.hit_test_band_region(canvas.x + canvas.width - 10.0, canvas),
            2
        ); // High

        // 4. Crossover Separation & Clamping Constraints
        shaper.set_crossover_low_mid(10000.0); // Try pushing past mid-high
        assert!(shaper.crossover_low_mid_hz < shaper.crossover_mid_high_hz);
        assert_eq!(shaper.bands[0].max_freq_hz, shaper.crossover_low_mid_hz);
        assert_eq!(shaper.bands[1].min_freq_hz, shaper.crossover_low_mid_hz);

        shaper.set_crossover_mid_high(50.0); // Try pushing below low-mid
        assert!(shaper.crossover_mid_high_hz > shaper.crossover_low_mid_hz);

        // 5. Band Parameters & Modes
        shaper.bands[0].attack_gain_db = 6.0;
        shaper.bands[0].sustain_gain_db = -4.0;
        shaper.bands[0].mode = BandMuteSolo::Solo;
        assert_eq!(shaper.bands[0].mode, BandMuteSolo::Solo);

        // 6. Real-time meter updates
        shaper.update_band_meters(0, 0.85, 0.40);
        assert_eq!(shaper.bands[0].detected_transient_peak, 0.85);
        assert_eq!(shaper.bands[0].detected_sustain_level, 0.40);

        // 7. Deterministic ASCII Render
        shaper.set_crossover_low_mid(250.0);
        shaper.set_crossover_mid_high(3500.0);
        let ascii = shaper.render_ascii(30);
        assert_eq!(ascii.len(), 30);
        assert!(ascii.contains('L'));
        assert!(ascii.contains('|'));
        assert!(ascii.contains('M'));
        assert!(ascii.contains('H'));
    }

    #[test]
    fn test_step_1382_1387_ambisonic_radar_polar_projections_and_touch_radii() {
        let mut radar = AmbisonicRadarView::new();
        assert_eq!(radar.format, AmbisonicFormat::Atmos714);
        assert_eq!(radar.sources.len(), 3);

        let center = (200.0, 200.0);
        let radius = 150.0;

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(AMBISONIC_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(AMBISONIC_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Polar coordinate <-> Screen position projection
        let az = 0.0_f32; // Front
        let dist = 2.5_f32; // Half max distance (5.0m max)
        let (sx, sy) = radar.polar_to_screen_pos(az, dist, center, radius);
        assert_eq!(sx, center.0);
        assert!(sy < center.1); // Front is up (negative Y)

        let (roundtrip_az, roundtrip_dist) = radar.screen_pos_to_polar((sx, sy), center, radius);
        assert!((roundtrip_az - az).abs() < 1.0);
        assert!((roundtrip_dist - dist).abs() < 0.1);

        // Test East (+90 deg)
        let (sx_e, sy_e) = radar.polar_to_screen_pos(90.0, 5.0, center, radius);
        assert!(sx_e > center.0);
        assert!((sy_e - center.1).abs() < 1.0);

        // 3. Source Hit Testing
        let src0 = &radar.sources[0];
        let (s0_x, s0_y) =
            radar.polar_to_screen_pos(src0.azimuth_deg, src0.distance_m, center, radius);
        let hit = radar.hit_test_source((s0_x, s0_y), center, radius);
        assert_eq!(hit, Some(0));

        let miss = radar.hit_test_source((s0_x + 50.0, s0_y + 50.0), center, radius);
        assert_ne!(miss, Some(0));

        // 4. Source Trajectory Stepping
        let mut moving_src = radar.sources[0].clone();
        moving_src.trajectory = TrajectoryShape::OrbitCircle;
        moving_src.trajectory_speed_hz = 1.0;
        let initial_az = moving_src.azimuth_deg;
        moving_src.step_trajectory(0.25); // Advance 1/4 second
        assert_ne!(moving_src.azimuth_deg, initial_az);

        // 5. Add Source
        let added_idx = radar.add_source("Ambient Cloud", 120.0, 30.0, 4.0);
        assert_eq!(added_idx, 3);
        assert_eq!(radar.sources.len(), 4);

        // 6. Format Channel Count
        assert_eq!(AmbisonicFormat::FirstOrderBFormat.channel_count(), 4);
        assert_eq!(AmbisonicFormat::Atmos714.channel_count(), 12);
        assert_eq!(AmbisonicFormat::Immersive916.channel_count(), 16);

        // 7. Deterministic ASCII Render
        let ascii = radar.render_ascii(9);
        assert!(ascii.contains('L')); // Listener in center
        assert!(ascii.contains('1')); // Source 1
    }

    #[test]
    fn test_step_1383_granular_cloud_dispersion_and_window_functions() {
        let mut cloud = GranularCloudView::new();
        assert_eq!(cloud.density, 16);
        assert!(!cloud.active_grains.is_empty());

        let canvas = Rect::new(0.0, 0.0, 600.0, 300.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(EMITTER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(EMITTER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Cloud Coordinate Transformations
        let (ex, ey) = cloud.cloud_coords_to_screen(
            cloud.emitter_pos_norm,
            cloud.emitter_pitch_semitones,
            canvas,
        );
        assert!((0.0..=600.0).contains(&ex));
        assert!((0.0..=300.0).contains(&ey));

        let (roundtrip_pos, roundtrip_pitch) = cloud.screen_to_cloud_coords((ex, ey), canvas);
        assert!((roundtrip_pos - cloud.emitter_pos_norm).abs() < 0.01);
        assert!((roundtrip_pitch - cloud.emitter_pitch_semitones).abs() < 0.1);

        // 3. Emitter Hit Detection
        assert!(cloud.hit_test_emitter((ex, ey), canvas));
        assert!(!cloud.hit_test_emitter((ex + 40.0, ey + 40.0), canvas));

        // 4. Window Function Evaluations
        for shape in [
            GrainWindowShape::Hanning,
            GrainWindowShape::Blackman,
            GrainWindowShape::Gaussian,
            GrainWindowShape::Trapezoid,
        ] {
            let center_amp = shape.evaluate(0.5);
            assert!(
                center_amp > 0.5,
                "Center amplitude should be high for {shape:?}"
            );
        }
        assert!(GrainWindowShape::ExponentialDecay.evaluate(0.0) >= 0.9);
        assert!(GrainWindowShape::ExponentialDecay.evaluate(1.0) < 0.1);

        // 5. Grain Particle Simulation Step
        let initial_age = cloud.active_grains[0].age_ms;
        cloud.step_grains(20.0);
        assert_ne!(cloud.active_grains[0].age_ms, initial_age);

        // 6. Deterministic ASCII Render
        let ascii = cloud.render_ascii(20, 10);
        assert!(ascii.contains('E')); // Emitter center
        assert!(ascii.contains('*')); // Grains
    }

    #[test]
    fn test_step_1384_spectral_morphing_crossfader_and_formants() {
        let mut morph = SpectralMorphView::new();
        assert_eq!(morph.source_a_bins.len(), 64);
        assert_eq!(morph.source_b_bins.len(), 64);
        assert_eq!(morph.morphed_bins.len(), 64);

        let track_rect = Rect::new(50.0, 300.0, 700.0, MIN_HIT_TARGET_PT);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch box)
        const { assert!(MORPH_CROSSFADER_HANDLE_RADIUS >= 22.0) };
        const { assert!(MORPH_CROSSFADER_HANDLE_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Crossfader Hit Testing
        let handle_x = track_rect.x + morph.morph_crossfade * track_rect.width;
        let handle_y = track_rect.y + track_rect.height * 0.5;
        assert!(morph.hit_test_crossfader((handle_x, handle_y), track_rect));
        assert!(!morph.hit_test_crossfader((handle_x + 50.0, handle_y), track_rect));

        // 3. Morph Crossfade Spectrum Interpolation
        morph.set_crossfade(0.0); // 100% Source A
        assert_eq!(morph.morph_crossfade, 0.0);
        let a_centroid = morph.harmonic_centroid_hz;

        morph.set_crossfade(1.0); // 100% Source B
        assert_eq!(morph.morph_crossfade, 1.0);
        let b_centroid = morph.harmonic_centroid_hz;
        assert!(
            b_centroid > a_centroid,
            "Source B should have higher centroid than Source A"
        );

        // 4. Formant Vowel Filter Application
        morph.active_formant = VowelFormant::VowelA;
        morph.recalculate_morphed_spectrum();
        assert!(morph.active_formant.formant_frequencies_hz().is_some());

        // 5. Morph Modes
        for mode in [
            SpectralMorphMode::LinearInterpolation,
            SpectralMorphMode::EqualPowerSpectral,
            SpectralMorphMode::FormantPreservingWarp,
            SpectralMorphMode::PhaseVocoderSmear,
            SpectralMorphMode::ConvolutionalBlend,
        ] {
            morph.mode = mode;
            morph.recalculate_morphed_spectrum();
            assert!(morph.harmonic_centroid_hz > 0.0);
        }

        // 6. Deterministic ASCII Render
        let ascii = morph.render_ascii(32);
        assert_eq!(ascii.len(), 32);
        assert!(!ascii.is_empty());
    }

    #[test]
    fn test_step_1385_loop_slicer_and_glitch_pad_matrix() {
        let mut slicer = LoopSlicerView::new(44100 * 2, 44100, 120.0, 16);
        assert_eq!(slicer.slices.len(), 16);
        assert_eq!(slicer.total_samples, 44100 * 2);

        let canvas = Rect::new(0.0, 50.0, 800.0, 120.0);
        let pad_origin = (100.0, 200.0);
        let pad_size = (120.0, MIN_HIT_TARGET_PT + 12.0); // 120x56pt

        // 1. Pad Dimensions Compliance (>=44x44pt)
        assert!(pad_size.0 >= MIN_HIT_TARGET_PT);
        assert!(pad_size.1 >= MIN_HIT_TARGET_PT);

        // 2. Pad Matrix Bounding Boxes & Hit Testing
        for idx in 0..16 {
            let rect = slicer.calculate_pad_rect(idx, pad_origin, pad_size);
            assert_eq!(rect.width, pad_size.0);
            assert_eq!(rect.height, pad_size.1);

            let hit = slicer.hit_test_pad((rect.x + 10.0, rect.y + 10.0), pad_origin, pad_size);
            assert_eq!(hit, Some(idx));
        }

        // 3. Slice Marker Minimum Hit Target (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(SLICE_MARKER_HIT_RADIUS >= 22.0) };
        const { assert!(SLICE_MARKER_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // Test Hit Detection on slice 2 marker
        let s2_x = slicer.sample_to_screen_x(slicer.slices[2].start_sample, canvas);
        let hit_m = slicer.hit_test_slice_marker((s2_x, canvas.y + 10.0), canvas);
        assert_eq!(hit_m, Some(2));

        // 4. Sample <-> Screen X Coordinate Transformations
        let sample_pos = 44100;
        let screen_x = slicer.sample_to_screen_x(sample_pos, canvas);
        let roundtrip_sample = slicer.screen_x_to_sample(screen_x, canvas);
        let diff = (roundtrip_sample as i64 - sample_pos as i64).abs();
        assert!(diff <= 50);

        // 5. Trigger Pad & Glitch Modes
        slicer.trigger_pad(3);
        assert_eq!(slicer.selected_slice_idx, 3);
        assert_eq!(slicer.active_playing_slice_idx, Some(3));
        assert!(slicer.slices[3].is_playing);
        assert_eq!(slicer.slices[3].mode, GlitchPadMode::Reverse);

        // 6. Deterministic ASCII Render
        let ascii = slicer.render_ascii(32);
        assert_eq!(ascii.len(), 32);
        assert!(ascii.contains('*')); // Active playing pad marker
    }
}
