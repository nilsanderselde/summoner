// Summoner DAW - Tier 65 GUI Milestones Unit Test Suite (Steps 1521-1530)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::hoa_spatializer_view::{
        HoaSpatialFormat, HoaSpatializerView, HOA_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_wavetable_view::{
        LatentArchitecture, NeuralWavetableView, NEURAL_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_unmasker_view::{
        SpectralUnmaskerView, UnmaskerRouting, UNMASKER_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_declicker_view::{
        TransientDeclickerView, VinylRestorationMode, DECLICKER_PUCK_HIT_RADIUS,
    };
    use crate::views::waveguide_brass_view::{
        BrassInstrument, WaveguideBrassView, BRASS_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1521_1526_waveguide_brass_impedance_and_hit_targets() {
        let mut brass = WaveguideBrassView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(BRASS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(BRASS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Lip Tension Conversion Roundtrip
        for tension in [50.0, 120.0, 440.0, 880.0, 1200.0] {
            let norm = WaveguideBrassView::tension_to_normalized(tension);
            assert!((0.0..=1.0).contains(&norm));
            let back = WaveguideBrassView::normalized_to_tension(norm);
            assert!((back - tension).abs() < 1e-3, "Tension mismatch at {}", tension);
        }

        // 3. Blowing Pressure Conversion Roundtrip
        for pressure in [0.20, 1.00, 2.50, 5.00, 8.00] {
            let norm = WaveguideBrassView::pressure_to_normalized(pressure);
            assert!((0.0..=1.0).contains(&norm));
            let back = WaveguideBrassView::normalized_to_pressure(norm);
            assert!((back - pressure).abs() < 1e-4, "Pressure mismatch at {}", pressure);
        }

        // 4. Bore Length Conversion Roundtrip
        for length in [0.50, 1.48, 2.75, 3.75, 5.50] {
            let norm = WaveguideBrassView::length_to_normalized(length);
            assert!((0.0..=1.0).contains(&norm));
            let back = WaveguideBrassView::normalized_to_length(norm);
            assert!((back - length).abs() < 1e-4, "Bore length mismatch at {}", length);
        }

        // 5. Instrument Presets and Acoustic Properties
        for inst in [
            BrassInstrument::TrumpetBb,
            BrassInstrument::FrenchHornF,
            BrassInstrument::TromboneBb,
            BrassInstrument::TubaEb,
            BrassInstrument::FlugelhornBb,
        ] {
            brass.instrument = inst;
            let nom_len = inst.nominal_tube_length_m();
            let flare = inst.bell_flare_exponent();
            let cutoff = inst.nominal_cutoff_hz();
            assert!(nom_len > 0.0);
            assert!(flare > 0.0 && flare < 1.0);
            assert!(cutoff >= 300.0 && cutoff <= 2000.0);
        }

        // 6. Radiation Reflection Evaluation
        let refl_low = brass.evaluate_radiation_reflection(100.0);
        let refl_high = brass.evaluate_radiation_reflection(2000.0);
        assert!(refl_low >= refl_high, "Low frequency reflection must be higher than high frequency");

        // 7. Hit Testing on Lip-Reed Puck
        brass.embouchure_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(brass.hit_test_embouchure_puck((center_x, center_y), canvas));
        assert!(!brass.hit_test_embouchure_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = brass.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1522_1527_spectral_unmasker_collision_and_hit_targets() {
        let mut unmasker = SpectralUnmaskerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(UNMASKER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(UNMASKER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency Conversion Roundtrip
        for freq in [20.0, 100.0, 1000.0, 5000.0, 20000.0] {
            let norm = SpectralUnmaskerView::freq_to_normalized(freq);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralUnmaskerView::normalized_to_freq(norm);
            assert!((back - freq).abs() / freq < 1e-3, "Freq mismatch at {}", freq);
        }

        // 3. Reduction Depth Conversion Roundtrip
        for depth in [0.0, 3.5, 6.0, 12.0, 18.0] {
            let norm = SpectralUnmaskerView::depth_to_normalized(depth);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralUnmaskerView::normalized_to_depth(norm);
            assert!((back - depth).abs() < 1e-4, "Depth mismatch at {}", depth);
        }

        // 4. Sensitivity Conversion Roundtrip
        for sens in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let norm = SpectralUnmaskerView::sensitivity_to_normalized(sens);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralUnmaskerView::normalized_to_sensitivity(norm);
            assert!((back - sens).abs() < 1e-4, "Sensitivity mismatch at {}", sens);
        }

        // 5. Routing Preset Configurations
        for routing in [
            UnmaskerRouting::KickVsBass,
            UnmaskerRouting::VocalVsSynth,
            UnmaskerRouting::SnareVsGuitar,
            UnmaskerRouting::DialogVsBgm,
            UnmaskerRouting::CustomBus,
        ] {
            unmasker.routing = routing;
            let target_f = routing.target_center_freq_hz();
            let target_q = routing.target_q_factor();
            let nominal_db = routing.nominal_reduction_db();
            assert!(target_f >= 20.0 && target_f <= 20000.0);
            assert!(target_q >= 0.5 && target_q <= 10.0);
            assert!(nominal_db >= 0.0 && nominal_db <= 18.0);
        }

        // 6. Hit Testing on Collision Puck
        unmasker.unmasker_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(unmasker.hit_test_unmasker_puck((center_x, center_y), canvas));
        assert!(!unmasker.hit_test_unmasker_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = unmasker.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1523_transient_declicker_vinyl_restoration() {
        let mut declicker = TransientDeclickerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(DECLICKER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DECLICKER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-48.0, -36.0, -24.0, -12.0, 0.0] {
            let norm = TransientDeclickerView::threshold_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientDeclickerView::normalized_to_threshold(norm);
            assert!((back - thresh).abs() < 1e-4, "Threshold mismatch at {}", thresh);
        }

        // 3. Click Width Conversion Roundtrip
        for width in [0.05, 0.50, 1.20, 3.00, 5.00] {
            let norm = TransientDeclickerView::width_to_normalized(width);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientDeclickerView::normalized_to_width(norm);
            assert!((back - width).abs() < 1e-4, "Width mismatch at {}", width);
        }

        // 4. Vinyl Restoration Preset Configurations
        for mode in [
            VinylRestorationMode::VinylMicrogroove,
            VinylRestorationMode::Shellac78Rpm,
            VinylRestorationMode::DigitalClicks,
            VinylRestorationMode::ThumpAndPlop,
            VinylRestorationMode::TapeDropout,
        ] {
            declicker.mode = mode;
            let def_thresh = mode.default_threshold_db();
            let def_w = mode.default_click_width_ms();
            let crossover = mode.linear_phase_crossover_hz();
            assert!(def_thresh <= 0.0 && def_thresh >= -48.0);
            assert!(def_w >= 0.05 && def_w <= 5.00);
            assert!(crossover >= 200.0 && crossover <= 10000.0);
        }

        // 5. Hit Testing on De-Clicker Puck
        declicker.declicker_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(declicker.hit_test_declicker_puck((center_x, center_y), canvas));
        assert!(!declicker.hit_test_declicker_puck((center_x + 100.0, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = declicker.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1524_neural_wavetable_morphing_synth() {
        let mut neural = NeuralWavetableView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(NEURAL_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NEURAL_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Latent Coordinate Conversion Roundtrip
        for coord in [-2.50, -1.25, 0.0, 0.62, 1.85, 2.50] {
            let norm = NeuralWavetableView::coord_to_normalized(coord);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralWavetableView::normalized_to_coord(norm);
            assert!((back - coord).abs() < 1e-4, "Coord mismatch at {}", coord);
        }

        // 3. Morph Speed Conversion Roundtrip
        for speed in [0.01, 0.85, 2.40, 10.0, 20.0] {
            let norm = NeuralWavetableView::speed_to_normalized(speed);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralWavetableView::normalized_to_speed(norm);
            assert!((back - speed).abs() < 1e-4, "Speed mismatch at {}", speed);
        }

        // 4. Orbit Radius Conversion Roundtrip
        for radius in [0.00, 0.45, 0.90, 1.50, 2.00] {
            let norm = NeuralWavetableView::radius_to_normalized(radius);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralWavetableView::normalized_to_radius(norm);
            assert!((back - radius).abs() < 1e-4, "Radius mismatch at {}", radius);
        }

        // 5. Latent Architecture Profiles and FID Score
        for arch in [
            LatentArchitecture::VaeContinuous,
            LatentArchitecture::TransformerDyn,
            LatentArchitecture::DiffusionRes,
            LatentArchitecture::Hypersphere4D,
            LatentArchitecture::SpectralFlow,
        ] {
            neural.architecture = arch;
            let fid = arch.reconstruction_fid_score();
            assert!(fid >= 95.0 && fid <= 100.0);
            let def_speed = arch.default_morph_speed_hz();
            assert!(def_speed >= 0.01 && def_speed <= 20.0);
        }

        // 6. Wavetable and Harmonics Synthesis Evaluation
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let sample = neural.evaluate_wavetable_sample(t);
            assert!((-1.0..=1.0).contains(&sample));
        }

        for h in 1..=16 {
            let energy = neural.evaluate_harmonic_energy(h);
            assert!(energy > 0.0 && energy <= 1.0);
        }

        // 7. Hit Testing on Latent Puck
        neural.latent_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(neural.hit_test_latent_puck((center_x, center_y), canvas));
        assert!(!neural.hit_test_latent_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = neural.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1525_hoa_spatializer_and_spherical_harmonics() {
        let mut hoa = HoaSpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(HOA_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(HOA_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, -30.0, 0.0, 45.0, 120.0, 180.0] {
            let norm = HoaSpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = HoaSpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Conversion Roundtrip
        for el in [-90.0, -45.0, 0.0, 18.5, 45.0, 90.0] {
            let norm = HoaSpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = HoaSpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4, "Elevation mismatch at {}", el);
        }

        // 4. Distance Conversion Roundtrip
        for dist in [0.10, 1.00, 2.40, 5.00, 8.50, 10.00] {
            let norm = HoaSpatializerView::distance_to_normalized(dist);
            assert!((0.0..=1.0).contains(&norm));
            let back = HoaSpatializerView::normalized_to_distance(norm);
            assert!((back - dist).abs() < 1e-4, "Distance mismatch at {}", dist);
        }

        // 5. Relative Azimuth and Head-Tracking Yaw
        hoa.azimuth_deg = 45.0;
        hoa.head_yaw_deg = 15.0;
        assert_eq!(hoa.effective_relative_azimuth_deg(), 30.0);

        hoa.head_yaw_deg = -170.0;
        hoa.azimuth_deg = 170.0;
        assert_eq!(hoa.effective_relative_azimuth_deg(), -20.0);

        // 6. 16-Channel HOA 3rd Order Spherical Harmonic Decomposition
        hoa.azimuth_deg = 0.0;
        hoa.elevation_deg = 0.0;
        hoa.distance_m = 1.0;
        hoa.head_yaw_deg = 0.0;
        hoa.update_spherical_harmonics();

        // At (0 deg az, 0 deg el), X = front, Y = side (0), Z = up (0)
        // Order 0 (ACN 0) = W = 1.0 * dist_att
        assert!(hoa.spherical_harmonics[0] > 0.0);
        // Order 1: Y (ACN 1) should be 0.0, Z (ACN 2) should be 0.0, X (ACN 3) should be positive
        assert!(hoa.spherical_harmonics[1].abs() < 1e-4);
        assert!(hoa.spherical_harmonics[2].abs() < 1e-4);
        assert!(hoa.spherical_harmonics[3] > 0.0);

        // 7. Spatial Formats Channel Count
        for fmt in [
            HoaSpatialFormat::HoaThirdOrder,
            HoaSpatialFormat::DolbyAtmos714,
            HoaSpatialFormat::BinauralHeadTrack,
            HoaSpatialFormat::Ambisonics514,
            HoaSpatialFormat::DomeAcoustic916,
        ] {
            hoa.format = fmt;
            assert!(fmt.channel_count() > 0);
        }

        // 8. Hit Testing on Source Puck
        hoa.source_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(hoa.hit_test_source_puck((center_x, center_y), canvas));
        assert!(!hoa.hit_test_source_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = hoa.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }
}
