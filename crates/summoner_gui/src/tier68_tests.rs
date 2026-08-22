// Summoner DAW - Tier 68 GUI Milestones Unit Test Suite (Steps 1551-1560)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::neural_phoneme_view::{
        NeuralPhonemeView, PhonemeModel, PHONEME_PUCK_HIT_RADIUS,
    };
    use crate::views::nhk222_spatializer_view::{
        Nhk222SpatializerView, NhkFormat, NHK222_PUCK_HIT_RADIUS,
    };
    use crate::views::sonar_hydrophone_view::{
        SonarHydrophoneView, SonarMode, SONAR_HYDROPHONE_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_unwrapper_view::{
        TransientUnwrapperView, UnwrapperMode, TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS,
    };
    use crate::views::vari_mu_master_view::{
        TubeProfile, VariMuMasterView, VARI_MU_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1551_1556_sonar_hydrophone_cavitation_impedance_and_hit_targets() {
        let mut sonar = SonarHydrophoneView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(SONAR_HYDROPHONE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SONAR_HYDROPHONE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Depth Conversion Roundtrip (Logarithmic)
        for depth in [1.0, 15.0, 250.0, 1200.0, 5000.0] {
            let norm = SonarHydrophoneView::depth_to_normalized(depth);
            assert!((0.0..=1.0).contains(&norm));
            let back = SonarHydrophoneView::normalized_to_depth(norm);
            assert!(
                (back - depth).abs() / depth < 1e-3,
                "Depth mismatch at {}",
                depth
            );
        }

        // 3. Water Temperature Conversion Roundtrip
        for temp in [0.0, 4.0, 12.5, 20.0, 30.0] {
            let norm = SonarHydrophoneView::temp_to_normalized(temp);
            assert!((0.0..=1.0).contains(&norm));
            let back = SonarHydrophoneView::normalized_to_temp(norm);
            assert!(
                (back - temp).abs() < 1e-4,
                "Temperature mismatch at {}",
                temp
            );
        }

        // 4. Cavitation Number Conversion Roundtrip
        for sigma in [0.05, 0.5, 1.2, 3.5, 5.0] {
            let norm = SonarHydrophoneView::cavitation_to_normalized(sigma);
            assert!((0.0..=1.0).contains(&norm));
            let back = SonarHydrophoneView::normalized_to_cavitation(norm);
            assert!(
                (back - sigma).abs() < 1e-4,
                "Cavitation index mismatch at {}",
                sigma
            );
        }

        // 5. Sonar Modes & Operating Frequencies
        for mode in [
            SonarMode::ActiveSonarPing,
            SonarMode::PassiveHydrophoneListening,
            SonarMode::DeepOceanCavitation,
            SonarMode::ThermoclineWaveguide,
            SonarMode::ArcticUnderIceRefraction,
        ] {
            sonar.set_mode(mode);
            let f0 = mode.nominal_ping_freq_hz();
            let dur = mode.default_pulse_duration_ms();
            assert!(f0 > 0.0);
            assert!(dur >= 0.0);
        }

        // 6. Mackenzie Sound Speed Formula & Minnaert Resonance
        sonar.water_temp_c = 15.0;
        sonar.salinity_ppt = 35.0;
        sonar.depth_m = 100.0;
        sonar.update_acoustic_simulation();
        assert!((1400.0..=1600.0).contains(&sonar.sound_speed_mps));
        assert!(sonar.minnaert_resonance_hz > 1000.0);
        assert!((30.0..=120.0).contains(&sonar.ambient_noise_db));

        // 7. Transmission Loss Evaluation
        let tl_100 = sonar.evaluate_transmission_loss_db(100.0);
        let tl_1000 = sonar.evaluate_transmission_loss_db(1000.0);
        assert!(tl_100 > 0.0);
        assert!(tl_1000 > tl_100);

        // 8. Hit Testing on Sonar Puck
        sonar.sonar_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(sonar.hit_test_sonar_puck((center_x, center_y), canvas));
        assert!(!sonar.hit_test_sonar_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = sonar.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1552_1557_transient_unwrapper_spatial_depth_decorrelation_and_hit_targets() {
        let mut unwrapper = TransientUnwrapperView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Spatial Width Conversion Roundtrip
        for width in [0.0, 50.0, 100.0, 140.0, 200.0] {
            let norm = TransientUnwrapperView::width_to_normalized(width);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientUnwrapperView::normalized_to_width(norm);
            assert!((back - width).abs() < 1e-4, "Width mismatch at {}", width);
        }

        // 3. Decorrelation Delay Conversion Roundtrip
        for delay in [0.0, 2.5, 8.2, 15.0, 25.0] {
            let norm = TransientUnwrapperView::delay_to_normalized(delay);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientUnwrapperView::normalized_to_delay(norm);
            assert!((back - delay).abs() < 1e-4, "Delay mismatch at {}", delay);
        }

        // 4. Unwrap Angle Conversion Roundtrip
        for angle in [-90.0, -45.0, 0.0, 25.0, 90.0] {
            let norm = TransientUnwrapperView::angle_to_normalized(angle);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientUnwrapperView::normalized_to_angle(norm);
            assert!((back - angle).abs() < 1e-4, "Angle mismatch at {}", angle);
        }

        // 5. Unwrapper Modes & Defaults
        for mode in [
            UnwrapperMode::WideStereoExpansion,
            UnwrapperMode::BinauralDepthExtraction,
            UnwrapperMode::DrumTransientDecomb,
            UnwrapperMode::MasteringSpatialUnwrap,
            UnwrapperMode::MicrotonalPhaseDecorrelate,
        ] {
            unwrapper.set_mode(mode);
            let w = mode.default_width_pct();
            let d = mode.default_decorrelation_delay_ms();
            let xover = mode.default_mono_crossover_hz();
            assert!((0.0..=200.0).contains(&w));
            assert!((0.0..=25.0).contains(&d));
            assert!((40.0..=1000.0).contains(&xover));
        }

        // 6. IACC & Lissajous Stereo Spread Evaluation
        let (l, r) = unwrapper.evaluate_stereo_spread(0.5);
        assert!(l.is_finite() && r.is_finite());
        assert!((-1.0..=1.0).contains(&unwrapper.iacc_correlation));

        // 7. Hit Testing on Unwrapper Puck
        unwrapper.unwrapper_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(unwrapper.hit_test_unwrapper_puck((center_x, center_y), canvas));
        assert!(!unwrapper.hit_test_unwrapper_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = unwrapper.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1553_vari_mu_vacuum_tube_optical_master_compressor_and_hit_targets() {
        let mut vari_mu = VariMuMasterView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(VARI_MU_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(VARI_MU_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-40.0, -25.0, -14.0, 0.0, 10.0] {
            let norm = VariMuMasterView::threshold_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = VariMuMasterView::normalized_to_threshold(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 3. Drive Conversion Roundtrip
        for drive in [-12.0, 0.0, 4.5, 12.0, 24.0] {
            let norm = VariMuMasterView::drive_to_normalized(drive);
            assert!((0.0..=1.0).contains(&norm));
            let back = VariMuMasterView::normalized_to_drive(norm);
            assert!((back - drive).abs() < 1e-4, "Drive mismatch at {}", drive);
        }

        // 4. Stereo Link Conversion Roundtrip
        for link in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let norm = VariMuMasterView::link_to_normalized(link);
            assert!((0.0..=1.0).contains(&norm));
            let back = VariMuMasterView::normalized_to_link(norm);
            assert!((back - link).abs() < 1e-4, "Link mismatch at {}", link);
        }

        // 5. Tube Profiles
        for prof in [
            TubeProfile::Fairchild670Vintage,
            TubeProfile::ManleyVariableMu,
            TubeProfile::TeletronixLa2aOpto,
            TubeProfile::Neve33609DiodeBridge,
            TubeProfile::PultecTubeFeedback,
        ] {
            vari_mu.set_profile(prof);
            let att = prof.default_attack_ms();
            let rel = prof.default_release_ms();
            let r = prof.nominal_base_ratio();
            assert!(att > 0.0);
            assert!(rel > 0.0);
            assert!((1.0..=10.0).contains(&r));
            assert!(!prof.harmonic_profile_name().is_empty());
        }

        // 6. Dynamic Transfer Curve Evaluation
        let out_sub = vari_mu.evaluate_transfer_curve(-30.0);
        let out_over = vari_mu.evaluate_transfer_curve(0.0);
        assert!(out_sub.is_finite());
        assert!(out_over.is_finite());
        assert!(out_over < 0.0 + vari_mu.input_drive_db + vari_mu.makeup_gain_db); // Compression effect

        // 7. Hit Testing on Vari-Mu Puck
        vari_mu.vari_mu_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(vari_mu.hit_test_vari_mu_puck((center_x, center_y), canvas));
        assert!(!vari_mu.hit_test_vari_mu_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = vari_mu.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1554_neural_phoneme_morphing_vocoder_and_hit_targets() {
        let mut phoneme = NeuralPhonemeView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(PHONEME_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(PHONEME_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Formant F1 Conversion Roundtrip (Logarithmic)
        for f1 in [200.0, 350.0, 500.0, 800.0, 1200.0] {
            let norm = NeuralPhonemeView::f1_to_normalized(f1);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralPhonemeView::normalized_to_f1(norm);
            assert!((back - f1).abs() / f1 < 1e-3, "F1 mismatch at {}", f1);
        }

        // 3. Formant F2 Conversion Roundtrip (Logarithmic)
        for f2 in [600.0, 1200.0, 1800.0, 2600.0, 3500.0] {
            let norm = NeuralPhonemeView::f2_to_normalized(f2);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralPhonemeView::normalized_to_f2(norm);
            assert!((back - f2).abs() / f2 < 1e-3, "F2 mismatch at {}", f2);
        }

        // 4. Vocal Tract Length Conversion Roundtrip
        for tract in [8.0, 12.5, 17.0, 21.0, 25.0] {
            let norm = NeuralPhonemeView::tract_to_normalized(tract);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralPhonemeView::normalized_to_tract(norm);
            assert!(
                (back - tract).abs() < 1e-4,
                "Tract length mismatch at {}",
                tract
            );
        }

        // 5. Phoneme Models & Formant Classification
        for model in [
            PhonemeModel::VowelFormantMorph,
            PhonemeModel::WhisperToVoicedTransfer,
            PhonemeModel::RoboticVocoderCarrier,
            PhonemeModel::AlienFormantShift,
            PhonemeModel::LatentDiffusionInterpolate,
        ] {
            phoneme.set_model(model);
            let f1 = model.default_f1_hz();
            let f2 = model.default_f2_hz();
            let t = model.default_tract_length_cm();
            assert!((200.0..=1200.0).contains(&f1));
            assert!((600.0..=3500.0).contains(&f2));
            assert!((8.0..=25.0).contains(&t));
            assert!(!phoneme.active_phoneme_symbol.is_empty());
        }

        // 6. Spectral Envelope Magnitude Evaluation
        let mag_f1 = phoneme.evaluate_spectral_envelope_db(phoneme.formant_f1_hz);
        let mag_f2 = phoneme.evaluate_spectral_envelope_db(phoneme.formant_f2_hz);
        let mag_low = phoneme.evaluate_spectral_envelope_db(50.0);
        assert!(mag_f1 > mag_low);
        assert!(mag_f2 > mag_low);

        // 7. Hit Testing on Phoneme Puck
        phoneme.phoneme_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(phoneme.hit_test_phoneme_puck((center_x, center_y), canvas));
        assert!(!phoneme.hit_test_phoneme_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = phoneme.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1555_nhk222_spatializer_hemispherical_dome_and_hit_targets() {
        let mut nhk = Nhk222SpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(NHK222_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NHK222_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, 0.0, 45.0, 180.0] {
            let norm = Nhk222SpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222SpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-3, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Conversion Roundtrip
        for el in [-30.0, -10.0, 0.0, 25.0, 60.0, 90.0] {
            let norm = Nhk222SpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222SpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-3, "Elevation mismatch at {}", el);
        }

        // 4. NHK Formats & Channels
        for fmt in [
            NhkFormat::NHK222FullDome,
            NhkFormat::NHK222Downmix91,
            NhkFormat::NHK222Downmix51,
            NhkFormat::NHK222BinauralDome,
            NhkFormat::NHK222ObjectMaster,
        ] {
            nhk.set_format(fmt);
            let ch = fmt.channel_count();
            assert!(ch > 0);
        }

        // 5. 3-Tier Layer Energy Panning
        nhk.elevation_deg = 0.0;
        nhk.update_spatial_distribution();
        assert_eq!(nhk.middle_layer_energy, 1.0);
        assert_eq!(nhk.top_layer_energy, 0.0);
        assert_eq!(nhk.bottom_layer_energy, 0.0);

        nhk.elevation_deg = 90.0;
        nhk.update_spatial_distribution();
        assert_eq!(nhk.top_layer_energy, 1.0);
        assert_eq!(nhk.middle_layer_energy, 0.0);

        // 6. Cartesian 3D Coordinates
        let (x, y, z) = nhk.evaluate_cartesian_position();
        assert!(x.is_finite() && y.is_finite() && z.is_finite());

        // 7. Hit Testing on NHK Puck
        nhk.nhk_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(nhk.hit_test_nhk_puck((center_x, center_y), canvas));
        assert!(!nhk.hit_test_nhk_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = nhk.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
