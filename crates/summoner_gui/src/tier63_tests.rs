// Summoner DAW - Tier 63 GUI Milestones Unit Test Suite (Steps 1501-1510)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::ebu_loudness_radar_view::{
        EbuLoudnessRadarView, LoudnessStandard, EBU_RADAR_PUCK_HIT_RADIUS,
    };
    use crate::views::membrane_resonator_view::{
        MembraneMaterial, MembraneResonatorView, MEMBRANE_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_vocoder_morph_view::{
        NeuralVocoderMorphView, VocoderMorphMode, NEURAL_VOCODER_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_aligner_view::{
        AlignmentAlgorithm, SpectralAlignerView, ALIGNER_PUCK_HIT_RADIUS,
    };
    use crate::views::upward_compressor_view::{
        UpwardCompressionProfile, UpwardCompressorView, UPWARD_COMPRESSOR_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1501_1506_neural_vocoder_formant_tracking_and_hit_targets() {
        let mut vocoder = NeuralVocoderMorphView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(NEURAL_VOCODER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NEURAL_VOCODER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. F1 Formant Frequency Conversion Roundtrip
        for f1 in [200.0, 350.0, 500.0, 750.0, 1000.0, 1200.0] {
            let norm = NeuralVocoderMorphView::f1_freq_to_normalized(f1);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralVocoderMorphView::normalized_to_f1_freq(norm);
            assert!((back - f1).abs() < 1e-4, "F1 mismatch at {}", f1);
        }

        // 3. F2 Formant Frequency Conversion Roundtrip
        for f2 in [600.0, 1000.0, 1500.0, 2200.0, 2800.0, 3200.0] {
            let norm = NeuralVocoderMorphView::f2_freq_to_normalized(f2);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralVocoderMorphView::normalized_to_f2_freq(norm);
            assert!((back - f2).abs() < 1e-4, "F2 mismatch at {}", f2);
        }

        // 4. Formant Shift Semitones Roundtrip
        for shift in [-24.0, -12.0, -5.0, 0.0, 7.0, 18.0, 24.0] {
            let norm = NeuralVocoderMorphView::formant_shift_to_normalized(shift);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralVocoderMorphView::normalized_to_formant_shift(norm);
            assert!((back - shift).abs() < 1e-4, "Shift mismatch at {}", shift);
        }

        // 5. Spectral Formant Envelope Evaluation
        let env_center = vocoder.evaluate_spectral_envelope(500.0);
        let env_valley = vocoder.evaluate_spectral_envelope(100.0);
        assert!(
            env_center > env_valley,
            "Formant resonance at 500Hz must be greater than 100Hz valley"
        );

        // 6. Modes Selection
        for mode in [
            VocoderMorphMode::NeuralLpc16,
            VocoderMorphMode::PhoneticVowel,
            VocoderMorphMode::RoboticCarrier,
            VocoderMorphMode::CepstralMorph,
            VocoderMorphMode::SpectralResynth,
        ] {
            vocoder.mode = mode;
            assert_eq!(vocoder.mode, mode);
        }

        // 7. Hit Testing on Vowel Space Puck
        vocoder.formant_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(vocoder.hit_test_formant_puck((center_x, center_y), canvas));
        assert!(!vocoder.hit_test_formant_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = vocoder.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1502_spectral_transient_auto_aligner() {
        let mut aligner = SpectralAlignerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(ALIGNER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ALIGNER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Delay Offset Conversion Roundtrip
        for delay in [-50.0, -25.0, 0.0, 2.35, 18.2, 50.0] {
            let norm = SpectralAlignerView::delay_to_normalized(delay);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralAlignerView::normalized_to_delay(norm);
            assert!((back - delay).abs() < 1e-4, "Delay mismatch at {}", delay);
        }

        // 3. Phase Angle Conversion Roundtrip
        for phase in [-180.0, -90.0, 0.0, 45.0, 135.0, 180.0] {
            let norm = SpectralAlignerView::phase_to_normalized(phase);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralAlignerView::normalized_to_phase(norm);
            assert!((back - phase).abs() < 1e-4, "Phase mismatch at {}", phase);
        }

        // 4. Cross-Correlation Peak Evaluation
        aligner.selected_channel_idx = 1;
        aligner.channels[1].delay_offset_ms = 5.0;
        let peak_val = aligner.evaluate_correlation_curve(5.0);
        let off_val = aligner.evaluate_correlation_curve(15.0);
        assert!(peak_val > off_val);

        // 5. Alignment Algorithms
        for algo in [
            AlignmentAlgorithm::CrossCorrelation,
            AlignmentAlgorithm::SpectralPhaseFft,
            AlignmentAlgorithm::TransientOnset,
            AlignmentAlgorithm::SubBandDelay,
            AlignmentAlgorithm::InfrasonicLock,
        ] {
            aligner.algorithm = algo;
            assert_eq!(aligner.algorithm, algo);
        }

        // 6. Hit Testing
        aligner.delay_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(aligner.hit_test_delay_puck((center_x, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = aligner.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1503_1507_upward_compressor_transfer_thresholds_and_hit_targets() {
        let mut upward = UpwardCompressorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(UPWARD_COMPRESSOR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(UPWARD_COMPRESSOR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-60.0, -48.0, -36.0, -24.0, -10.0] {
            let norm = UpwardCompressorView::threshold_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = UpwardCompressorView::normalized_to_threshold(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 3. Boost Conversion Roundtrip
        for boost in [0.0, 3.5, 8.0, 12.5, 18.0] {
            let norm = UpwardCompressorView::boost_to_normalized(boost);
            assert!((0.0..=1.0).contains(&norm));
            let back = UpwardCompressorView::normalized_to_boost(norm);
            assert!((back - boost).abs() < 1e-4, "Boost mismatch at {}", boost);
        }

        // 4. Upward Compression Transfer Curve Behavior
        upward.bands[0].threshold_dbfs = -30.0;
        upward.bands[0].max_boost_db = 10.0;
        upward.bands[0].ratio = 2.0;

        // Above threshold -> unity gain (transients uncompressed)
        let out_above = upward.evaluate_transfer_curve(-10.0, 0);
        assert!((out_above - (-10.0)).abs() < 1e-4);

        // Below threshold -> boosted upward
        let out_below = upward.evaluate_transfer_curve(-40.0, 0);
        assert!(out_below > -40.0);

        // 5. Profiles
        for prof in [
            UpwardCompressionProfile::LowLevelDetail,
            UpwardCompressionProfile::OttAggressive,
            UpwardCompressionProfile::BroadcastDensity,
            UpwardCompressionProfile::VocalAirExtract,
            UpwardCompressionProfile::LinearPhaseMaster,
        ] {
            upward.profile = prof;
            assert_eq!(upward.profile, prof);
        }

        // 6. Hit Testing
        upward.upward_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(upward.hit_test_upward_puck((center_x, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = upward.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1504_membrane_resonator_physics_and_materials() {
        let mut membrane = MembraneResonatorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(MEMBRANE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MEMBRANE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Tension Conversion Roundtrip
        for tension in [500.0, 1500.0, 3500.0, 5500.0, 8000.0] {
            let norm = MembraneResonatorView::tension_to_normalized(tension);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembraneResonatorView::normalized_to_tension(norm);
            assert!(
                (back - tension).abs() < 1e-4,
                "Tension mismatch at {}",
                tension
            );
        }

        // 3. Materials and Physical Simulation
        for mat in [
            MembraneMaterial::MylarDrumhead,
            MembraneMaterial::CalfskinVintage,
            MembraneMaterial::TitaniumFoil,
            MembraneMaterial::SiliconeElastic,
            MembraneMaterial::CarbonComposite,
        ] {
            membrane.material = mat;
            membrane.update_physics_simulation();
            assert!(membrane.fundamental_freq_hz >= 20.0);
            assert!(membrane.material.surface_density_kg_m2() > 0.0);
            assert!(membrane.material.internal_damping_coeff() > 0.0);
        }

        // 4. Higher tension produces higher fundamental pitch
        membrane.membrane_tension_nm = 1000.0;
        membrane.update_physics_simulation();
        let low_f0 = membrane.fundamental_freq_hz;

        membrane.membrane_tension_nm = 6000.0;
        membrane.update_physics_simulation();
        let high_f0 = membrane.fundamental_freq_hz;

        assert!(
            high_f0 > low_f0,
            "Higher membrane tension must yield higher frequency"
        );

        // 5. Hit Testing
        membrane.strike_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(membrane.hit_test_strike_puck((center_x, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = membrane.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1505_ebu_loudness_radar_metering() {
        let mut radar = EbuLoudnessRadarView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(EBU_RADAR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(EBU_RADAR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. LUFS Conversion Roundtrip
        for lufs in [-36.0, -28.0, -23.0, -14.0, -6.0] {
            let norm = EbuLoudnessRadarView::lufs_to_normalized(lufs);
            assert!((0.0..=1.0).contains(&norm));
            let back = EbuLoudnessRadarView::normalized_to_lufs(norm);
            assert!((back - lufs).abs() < 1e-4, "LUFS mismatch at {}", lufs);
        }

        // 3. True Peak dBTP Conversion Roundtrip
        for tp in [-6.0, -3.0, -1.0, 0.0, 1.5, 3.0] {
            let norm = EbuLoudnessRadarView::dbtp_to_normalized(tp);
            assert!((0.0..=1.0).contains(&norm));
            let back = EbuLoudnessRadarView::normalized_to_dbtp(norm);
            assert!((back - tp).abs() < 1e-4, "dBTP mismatch at {}", tp);
        }

        // 4. Loudness Standards Target Mappings
        assert_eq!(
            LoudnessStandard::EbuR128Broadcast.target_integrated_lufs(),
            -23.0
        );
        assert_eq!(
            LoudnessStandard::ItuBs1770Tv.target_integrated_lufs(),
            -24.0
        );
        assert_eq!(
            LoudnessStandard::AesTd1004Club.target_integrated_lufs(),
            -16.0
        );
        assert_eq!(
            LoudnessStandard::StreamingMusic.target_integrated_lufs(),
            -14.0
        );
        assert_eq!(
            LoudnessStandard::PodcastSpoken.target_integrated_lufs(),
            -19.0
        );

        // 5. Hit Testing
        radar.target_trim_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(radar.hit_test_trim_puck((center_x, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = radar.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }
}
