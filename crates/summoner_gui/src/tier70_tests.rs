// Summoner DAW - Tier 70 GUI Milestones Unit Test Suite (Steps 1571-1580)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::glass_armonica_view::{
        ArmonicaType, GlassArmonicaView, ARMONICA_PUCK_HIT_RADIUS,
    };
    use crate::views::hoa4_spatializer_view::{
        Hoa4Profile, Hoa4SpatializerView, HOA4_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_speech_to_singing_view::{
        NeuralSpeechToSingingView, VocalModel, SINGING_PUCK_HIT_RADIUS,
    };
    use crate::views::parallel_transient_saturator_view::{
        ParallelTransientSaturatorView, SaturatorMode, SATURATOR_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_tilt_view::{SpectralTiltView, TiltMode, TILT_PUCK_HIT_RADIUS};

    #[test]
    fn test_step_1571_1576_glass_armonica_modal_resonances_and_hit_targets() {
        let mut armonica = GlassArmonicaView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(ARMONICA_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ARMONICA_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Rotation Speed Conversion Roundtrip
        for speed in [0.1, 1.2, 2.5, 5.0, 10.0] {
            let norm = GlassArmonicaView::speed_to_normalized(speed);
            assert!((0.0..=1.0).contains(&norm));
            let back = GlassArmonicaView::normalized_to_speed(norm);
            assert!((back - speed).abs() < 1e-4, "Speed mismatch at {}", speed);
        }

        // 3. Normal Force Conversion Roundtrip
        for force in [0.05, 0.25, 0.45, 0.70, 1.00] {
            let norm = GlassArmonicaView::force_to_normalized(force);
            assert!((0.0..=1.0).contains(&norm));
            let back = GlassArmonicaView::normalized_to_force(norm);
            assert!(
                (back - force).abs() < 1e-4,
                "Normal force mismatch at {}",
                force
            );
        }

        // 4. Armonica Instrument Types and Nominal Values
        for itype in [
            ArmonicaType::FranklinArmonicaC4C7,
            ArmonicaType::CrystalSingingBowl432,
            ArmonicaType::WetFingerChalice,
            ArmonicaType::BorosilicateBell,
            ArmonicaType::MetallophoneResonator,
        ] {
            armonica.set_instrument_type(itype);
            let spd = itype.nominal_speed_rad_s();
            let force = itype.nominal_normal_force_n();
            let f0 = itype.nominal_fundamental_hz();
            let q = itype.nominal_q_factor();
            assert!((0.1..=10.0).contains(&spd));
            assert!((0.05..=1.00).contains(&force));
            assert!((100.0..=2000.0).contains(&f0));
            assert!((1000.0..=5000.0).contains(&q));
            assert!(!itype.instrument_name().is_empty());
        }

        // 5. Stick-Slip Friction & Modal Resonances Simulation
        armonica.set_instrument_type(ArmonicaType::FranklinArmonicaC4C7);
        armonica.rotation_speed_rad_s = 2.5;
        armonica.normal_force_n = 0.45;
        armonica.water_level_pct = 0.40;
        armonica.update_friction_simulation();
        assert!((0.2..=1.5).contains(&armonica.stick_slip_velocity_mps));
        assert!((1000.0..=4500.0).contains(&armonica.q_factor));
        for &amp in armonica.modal_amplitudes.iter() {
            assert!((0.0..=2.0).contains(&amp));
        }

        // 6. Hit Testing on Armonica Puck
        armonica.armonica_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(armonica.hit_test_armonica_puck((center_x, center_y), canvas));
        assert!(!armonica.hit_test_armonica_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = armonica.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1572_1577_spectral_tilt_phase_linearity_and_hit_targets() {
        let mut tilt = SpectralTiltView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(TILT_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(TILT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Pivot Frequency Conversion Roundtrip (Logarithmic)
        for freq in [200.0, 500.0, 1000.0, 2500.0, 5000.0] {
            let norm = SpectralTiltView::pivot_to_normalized(freq);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralTiltView::normalized_to_pivot(norm);
            assert!(
                (back - freq).abs() / freq < 1e-3,
                "Frequency mismatch at {}",
                freq
            );
        }

        // 3. Tilt Slope Conversion Roundtrip
        for slope in [-6.0, -3.0, 0.0, 1.5, 6.0] {
            let norm = SpectralTiltView::slope_to_normalized(slope);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralTiltView::normalized_to_slope(norm);
            assert!(
                (back - slope).abs() < 1e-4,
                "Tilt slope mismatch at {}",
                slope
            );
        }

        // 4. Tilt Modes & Phase Linearity
        for mode in [
            TiltMode::LinearSlopeTilt6dB,
            TiltMode::BaxandallDualShelf,
            TiltMode::PsychoacousticBark,
            TiltMode::PhaseLinearFIR,
            TiltMode::AdaptiveDynamicTilt,
        ] {
            tilt.set_tilt_mode(mode);
            let piv = mode.nominal_pivot_hz();
            let slp = mode.nominal_slope_db_oct();
            assert!((200.0..=5000.0).contains(&piv));
            assert!((-6.0..=6.0).contains(&slp));
            assert!(!mode.mode_name().is_empty());
        }

        // Phase-Linear FIR zero phase deviation check
        tilt.set_tilt_mode(TiltMode::PhaseLinearFIR);
        assert_eq!(tilt.phase_deviation_deg, 0.0);

        // 5. 8-Band Multiband Spectral Energy Calculation
        tilt.pivot_frequency_hz = 1000.0;
        tilt.tilt_slope_db_oct = 1.5;
        tilt.update_tilt_curve();
        for &band in tilt.spectral_bands.iter() {
            assert!((0.1..=3.0).contains(&band));
        }
        // Higher frequencies have higher energy when tilt slope is positive
        assert!(tilt.spectral_bands[7] > tilt.spectral_bands[0]);

        // 6. Hit Testing on Tilt Puck
        tilt.tilt_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(tilt.hit_test_tilt_puck((center_x, center_y), canvas));
        assert!(!tilt.hit_test_tilt_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = tilt.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1573_parallel_transient_saturator_dynamics_and_hit_targets() {
        let mut sat = ParallelTransientSaturatorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(SATURATOR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SATURATOR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Blend Ratio Conversion Roundtrip
        for blend in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = ParallelTransientSaturatorView::blend_to_normalized(blend);
            assert!((0.0..=1.0).contains(&norm));
            let back = ParallelTransientSaturatorView::normalized_to_blend(norm);
            assert!((back - blend).abs() < 1e-4, "Blend mismatch at {}", blend);
        }

        // 3. Saturation Drive Conversion Roundtrip
        for drive in [0.0, 6.0, 12.0, 18.0, 24.0] {
            let norm = ParallelTransientSaturatorView::drive_to_normalized(drive);
            assert!((0.0..=1.0).contains(&norm));
            let back = ParallelTransientSaturatorView::normalized_to_drive(norm);
            assert!((back - drive).abs() < 1e-4, "Drive mismatch at {}", drive);
        }

        // 4. Saturator Circuit Modes
        for mode in [
            SaturatorMode::TriodeTubeEvenOdd,
            SaturatorMode::TapeHysteresisFlux,
            SaturatorMode::FetTransientPunch,
            SaturatorMode::GermaniumDiodeGrit,
            SaturatorMode::CleanLinearDynamics,
        ] {
            sat.set_saturator_mode(mode);
            let d_nom = mode.nominal_drive_db();
            let b_nom = mode.nominal_blend();
            assert!((0.0..=24.0).contains(&d_nom));
            assert!((0.0..=1.0).contains(&b_nom));
            assert!(!mode.mode_name().is_empty());
        }

        // 5. THD & Harmonic Profile Simulation
        sat.set_saturator_mode(SaturatorMode::TriodeTubeEvenOdd);
        sat.transient_drive_db = 6.0;
        sat.update_saturation_simulation();
        assert!((0.05..=15.0).contains(&sat.thd_percent));
        assert!((3.0..=20.0).contains(&sat.crest_factor_db));
        for &harm in sat.harmonic_profile.iter() {
            assert!((0.0..=1.5).contains(&harm));
        }

        // 6. Hit Testing on Saturator Puck
        sat.saturation_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(sat.hit_test_saturator_puck((center_x, center_y), canvas));
        assert!(!sat.hit_test_saturator_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = sat.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1574_neural_speech_to_singing_retuning_and_hit_targets() {
        let mut singing = NeuralSpeechToSingingView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(SINGING_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SINGING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Target Pitch F0 Conversion Roundtrip (Logarithmic)
        for pitch in [55.0, 110.0, 220.0, 440.0, 880.0] {
            let norm = NeuralSpeechToSingingView::pitch_to_normalized(pitch);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralSpeechToSingingView::normalized_to_pitch(norm);
            assert!(
                (back - pitch).abs() / pitch < 1e-3,
                "Pitch mismatch at {}",
                pitch
            );
        }

        // 3. Vibrato Depth Conversion Roundtrip
        for vibrato in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let norm = NeuralSpeechToSingingView::vibrato_to_normalized(vibrato);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralSpeechToSingingView::normalized_to_vibrato(norm);
            assert!(
                (back - vibrato).abs() < 1e-4,
                "Vibrato mismatch at {}",
                vibrato
            );
        }

        // 4. Vocal Models
        for model in [
            VocalModel::BelCantoOperaTenor,
            VocalModel::PopModernVocalist,
            VocalModel::ChoralPolyphonicChoir,
            VocalModel::MicrotonalMaqamRetune,
            VocalModel::ExperimentalFormantMorph,
        ] {
            singing.set_vocal_model(model);
            let f0 = model.nominal_pitch_hz();
            let vib = model.nominal_vibrato_cents();
            let rate = model.nominal_vibrato_rate_hz();
            assert!((55.0..=880.0).contains(&f0));
            assert!((0.0..=100.0).contains(&vib));
            assert!((2.0..=9.0).contains(&rate));
            assert!(!model.model_name().is_empty());
        }

        // 5. Vocal Tract Formant Envelope Simulation
        singing.formant_shift_semitones = 0.0;
        singing.update_vocal_synthesis();
        assert!(singing.f0_confidence >= 0.90);
        for &env in singing.formant_envelope.iter() {
            assert!((0.1..=1.5).contains(&env));
        }

        // 6. Hit Testing on Singing Puck
        singing.singing_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(singing.hit_test_singing_puck((center_x, center_y), canvas));
        assert!(!singing.hit_test_singing_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = singing.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1575_hoa4_spatializer_energy_density_and_hit_targets() {
        let mut hoa4 = Hoa4SpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(HOA4_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(HOA4_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, 0.0, 45.0, 180.0] {
            let norm = Hoa4SpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa4SpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-3, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Conversion Roundtrip
        for el in [-90.0, -45.0, 0.0, 25.0, 90.0] {
            let norm = Hoa4SpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa4SpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-3, "Elevation mismatch at {}", el);
        }

        // 4. HOA4 Profiles
        for prof in [
            Hoa4Profile::Hoa4_25ChannelSphere,
            Hoa4Profile::BinauralHoa4Hrir,
            Hoa4Profile::Surround22_2Hoa4,
            Hoa4Profile::Dome7_1_4Hoa4,
            Hoa4Profile::EnergyMaxReDecoder,
        ] {
            hoa4.set_profile(prof);
            let ch = prof.channel_count();
            let ord = prof.ambisonic_order();
            assert!(ch >= 2);
            assert_eq!(ord, 4);
            assert!(!prof.profile_name().is_empty());
        }

        // 5. 3D Cartesian Position Evaluation
        hoa4.azimuth_deg = 45.0;
        hoa4.elevation_deg = 25.0;
        hoa4.distance_m = 3.0;
        let (x, y, z) = hoa4.evaluate_cartesian_position();
        assert!(x.is_finite() && y.is_finite() && z.is_finite());
        assert!((x * x + y * y + z * z).sqrt() > 2.8);

        // 6. 8-Octant 3D Energy Density Calculation
        hoa4.update_ambisonic_simulation();
        for &energy in hoa4.octant_energy.iter() {
            assert!((0.05..=1.0).contains(&energy));
        }

        // 7. Hit Testing on HOA4 Puck
        hoa4.hoa4_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(hoa4.hit_test_hoa4_puck((center_x, center_y), canvas));
        assert!(!hoa4.hit_test_hoa4_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = hoa4.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
