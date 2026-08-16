// Summoner DAW - Tier 61 GUI Milestones Unit Test Suite (Steps 1481-1490)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::atmos_surround_view::{
        AtmosDownmixMode, AtmosSurroundView, ATMOS_PUCK_HIT_RADIUS,
    };
    use crate::views::convolution_impulse_view::{
        ConvolutionImpulseView, ImpulseResponseType, CONVOLUTION_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_spatial_view::{
        MultibandSpatialView, SpatialProcessMode, SPATIAL_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_resynthesis_view::{
        AdditiveSpectrumMode, SpectralResynthesisView, RESYNTH_PUCK_HIT_RADIUS,
    };
    use crate::views::tape_flutter_view::{
        TapeFlutterView, TapeFormula, TapeSpeed, TAPE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1481_1486_convolution_impulse_decay_curves_and_hit_targets() {
        let mut conv = ConvolutionImpulseView::new();
        let canvas = Rect::new(20.0, 106.0, 760.0, 234.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(CONVOLUTION_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CONVOLUTION_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. RT60 Conversion Roundtrip
        for test_rt60 in [0.1, 0.5, 1.5, 3.2, 8.0, 20.0] {
            let norm = ConvolutionImpulseView::rt60_to_normalized(test_rt60);
            assert!((0.0..=1.0).contains(&norm));
            let back = ConvolutionImpulseView::normalized_to_rt60(norm);
            assert!(
                (back - test_rt60).abs() / test_rt60 < 1e-4,
                "RT60 mismatch at {}",
                test_rt60
            );
        }

        // 3. HF Damping Conversion Roundtrip
        for test_damp in [500.0, 1000.0, 6500.0, 12000.0, 20000.0] {
            let norm = ConvolutionImpulseView::damping_to_normalized(test_damp);
            assert!((0.0..=1.0).contains(&norm));
            let back = ConvolutionImpulseView::normalized_to_damping(norm);
            assert!(
                (back - test_damp).abs() / test_damp < 1e-4,
                "Damping mismatch at {}",
                test_damp
            );
        }

        // 4. Pre-Delay Conversion Roundtrip
        for pre in [0.0, 25.0, 75.0, 150.0, 250.0] {
            let norm = ConvolutionImpulseView::predelay_to_normalized(pre);
            assert!((0.0..=1.0).contains(&norm));
            let back = ConvolutionImpulseView::normalized_to_predelay(norm);
            assert!((back - pre).abs() < 1e-4);
        }

        // 5. Exponential Decay Envelope Evaluation
        conv.rt60_decay_s = 2.0;
        let env_0 = conv.evaluate_decay_envelope(0.0);
        let env_half = conv.evaluate_decay_envelope(1.0);
        let env_rt60 = conv.evaluate_decay_envelope(2.0);

        assert!((env_0 - 1.0).abs() < 1e-3);
        assert!(env_0 > env_half);
        assert!(env_half > env_rt60);
        // At RT60 (2.0s), energy drops 60dB -> amplitude drops 10^-3 = 0.001
        assert!((env_rt60 - 0.001).abs() < 1e-3);

        // 6. Impulse Response Acoustic Profile Modes
        for mode in [
            ImpulseResponseType::CathedralStone,
            ImpulseResponseType::VintagePlate140,
            ImpulseResponseType::StudioLiveRoom,
            ImpulseResponseType::SpringTankTriple,
            ImpulseResponseType::GatedNonLinear,
            ImpulseResponseType::CustomWavIR,
        ] {
            conv.ir_type = mode;
            assert_eq!(conv.ir_type, mode);
        }

        // 7. Hit Testing on Decay Puck
        conv.decay_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(conv.hit_test_decay_puck((center_x, center_y), canvas));
        assert!(!conv.hit_test_decay_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = conv.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1482_1487_spectral_additive_resynthesis_harmonic_bounds_and_hit_targets() {
        let mut synth = SpectralResynthesisView::new();
        let canvas = Rect::new(20.0, 106.0, 760.0, 234.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(RESYNTH_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RESYNTH_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Fundamental Frequency Roundtrip
        for test_f0 in [20.0, 110.0, 440.0, 1000.0, 2000.0] {
            let norm = SpectralResynthesisView::fundamental_to_normalized(test_f0);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralResynthesisView::normalized_to_fundamental(norm);
            assert!(
                (back - test_f0).abs() / test_f0 < 1e-4,
                "f0 mismatch at {}",
                test_f0
            );
        }

        // 3. Spectral Tilt Conversion Roundtrip
        for test_tilt in [-24.0, -12.0, -6.0, 0.0, 6.0] {
            let norm = SpectralResynthesisView::tilt_to_normalized(test_tilt);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralResynthesisView::normalized_to_tilt(norm);
            assert!((back - test_tilt).abs() < 1e-4);
        }

        // 4. Inharmonicity Stretch Roundtrip
        for test_b in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let norm = SpectralResynthesisView::stretch_to_normalized(test_b);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralResynthesisView::normalized_to_stretch(norm);
            assert!((back - test_b).abs() < 1e-4);
        }

        // 5. Partial Frequency Stretch Calculation
        synth.fundamental_f0_hz = 100.0;
        synth.inharmonicity_stretch = 0.0;
        let p1_freq = synth.compute_partial_frequency(1);
        let p4_freq = synth.compute_partial_frequency(4);
        assert!((p1_freq - 100.0).abs() < 1e-3);
        assert!((p4_freq - 400.0).abs() < 1e-3);

        synth.inharmonicity_stretch = 0.5;
        let p4_stretched = synth.compute_partial_frequency(4);
        assert!(
            p4_stretched > 400.0,
            "Inharmonicity must stretch higher partials"
        );

        // 6. Additive Modes
        for mode in [
            AdditiveSpectrumMode::SawtoothCascade,
            AdditiveSpectrumMode::SquareHollow,
            AdditiveSpectrumMode::BellInharmonic,
            AdditiveSpectrumMode::VocalFormantAA,
            AdditiveSpectrumMode::MetallicPlate,
        ] {
            synth.mode = mode;
            assert_eq!(synth.mode, mode);
        }

        // 7. Hit Testing on Spectral Puck
        synth.spectral_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(synth.hit_test_spectral_puck((center_x, center_y), canvas));
        assert!(!synth.hit_test_spectral_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = synth.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1483_multiband_spatial_imager_and_phase_correlation_ellipse() {
        let mut imager = MultibandSpatialView::new();
        let canvas = Rect::new(20.0, 106.0, 760.0, 234.0);

        // 1. Hit Target Enforcement
        const { assert!(SPATIAL_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SPATIAL_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Width Conversion Roundtrip
        for w in [0.0, 50.0, 100.0, 150.0, 200.0] {
            let norm = MultibandSpatialView::width_to_normalized(w);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandSpatialView::normalized_to_width(norm);
            assert!((back - w).abs() < 1e-4);
        }

        // 3. M/S Balance Conversion Roundtrip
        for ms in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let norm = MultibandSpatialView::ms_balance_to_normalized(ms);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandSpatialView::normalized_to_ms_balance(norm);
            assert!((back - ms).abs() < 1e-4);
        }

        // 4. Phase Correlation Calculation
        let pure_mono_r = MultibandSpatialView::calculate_phase_correlation(1.0, 0.0);
        let pure_side_r = MultibandSpatialView::calculate_phase_correlation(0.0, 1.0);
        let balanced_r = MultibandSpatialView::calculate_phase_correlation(1.0, 1.0);

        assert!((pure_mono_r - 1.0).abs() < 1e-4);
        assert!((pure_side_r - (-1.0)).abs() < 1e-4);
        assert!((balanced_r - 0.0).abs() < 1e-4);

        // 5. Band Selection & Modes
        for mode in [
            SpatialProcessMode::MidSideMatrix,
            SpatialProcessMode::HaasInterauralDelay,
            SpatialProcessMode::PolarEllipticSpread,
            SpatialProcessMode::BinauralBccpSpatial,
        ] {
            imager.mode = mode;
            assert_eq!(imager.mode, mode);
        }

        // 6. Hit Testing
        imager.spatial_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(imager.hit_test_spatial_puck((center_x, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = imager.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1484_analog_tape_flutter_wow_and_hysteresis() {
        let mut tape = TapeFlutterView::new();
        let canvas = Rect::new(20.0, 106.0, 760.0, 234.0);

        // 1. Hit Target Enforcement
        const { assert!(TAPE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(TAPE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Drive Conversion Roundtrip
        for drive in [-12.0, -6.0, 0.0, 6.0, 12.0, 24.0] {
            let norm = TapeFlutterView::drive_to_normalized(drive);
            assert!((0.0..=1.0).contains(&norm));
            let back = TapeFlutterView::normalized_to_drive(norm);
            assert!((back - drive).abs() < 1e-4);
        }

        // 3. Modulation Depth Roundtrip
        for mod_pct in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let norm = TapeFlutterView::modulation_to_normalized(mod_pct);
            assert!((0.0..=1.0).contains(&norm));
            let back = TapeFlutterView::normalized_to_modulation(norm);
            assert!((back - mod_pct).abs() < 1e-4);
        }

        // 4. Hysteresis Saturation Transfer Function
        tape.saturation_drive_db = 0.0;
        let out_0 = tape.evaluate_hysteresis_curve(0.0);
        let out_pos = tape.evaluate_hysteresis_curve(1.0);
        let out_neg = tape.evaluate_hysteresis_curve(-1.0);
        assert!((out_0 - 0.0).abs() < 1e-4);
        assert!((0.0..=1.0).contains(&out_pos));
        assert!((-1.0..0.0).contains(&out_neg));

        // 5. Tape Speeds & Formulas
        for speed in [
            TapeSpeed::Ips3_75,
            TapeSpeed::Ips7_5,
            TapeSpeed::Ips15,
            TapeSpeed::Ips30,
        ] {
            tape.tape_speed = speed;
            assert_eq!(tape.tape_speed, speed);
        }

        for formula in [
            TapeFormula::Type1Ferric,
            TapeFormula::Type2Chrome,
            TapeFormula::Type3Ferrochrome,
            TapeFormula::Type4Master911,
        ] {
            tape.tape_formula = formula;
            assert_eq!(tape.tape_formula, formula);
        }

        // 6. Hit Testing
        tape.tape_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(tape.hit_test_tape_puck((center_x, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = tape.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1485_dolby_atmos_surround_radar_and_vbap_gains() {
        let mut atmos = AtmosSurroundView::new();
        let canvas = Rect::new(20.0, 106.0, 760.0, 234.0);

        // 1. Hit Target Enforcement
        const { assert!(ATMOS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ATMOS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Coordinate Conversion Roundtrip
        for coord in [-1.0, -0.75, 0.0, 0.5, 1.0] {
            let norm = AtmosSurroundView::coord_to_normalized(coord);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosSurroundView::normalized_to_coord(norm);
            assert!((back - coord).abs() < 1e-4);
        }

        // 3. VBAP 12-Channel Speaker Gains
        atmos.object_x = -1.0; // Hard Left
        atmos.object_y = 1.0; // Front
        atmos.object_z_height = 0.0; // Ear level
        atmos.update_vbap_gains();

        // Channel 0 is Left front bed speaker -> should have highest energy
        let l_gain = atmos.speaker_energy_gains[0];
        let r_gain = atmos.speaker_energy_gains[2];
        assert!(
            l_gain > r_gain,
            "Left speaker gain must exceed Right for hard-left object"
        );

        // Top height overhead test
        atmos.object_z_height = 1.0; // Ceiling
        atmos.update_vbap_gains();
        let ltf_gain = atmos.speaker_energy_gains[8]; // Left Top Front
        assert!(ltf_gain > 0.0);

        // 4. Downmix Modes
        for mode in [
            AtmosDownmixMode::Full714Immersive,
            AtmosDownmixMode::Surround51Legacy,
            AtmosDownmixMode::Stereo20Binaural,
        ] {
            atmos.downmix_mode = mode;
            assert_eq!(atmos.downmix_mode, mode);
        }

        // 5. Hit Testing
        atmos.atmos_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(atmos.hit_test_atmos_puck((center_x, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = atmos.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }
}
