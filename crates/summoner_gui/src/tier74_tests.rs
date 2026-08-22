// Summoner DAW - Tier 74 GUI Milestones Unit Test Suite (Steps 1611-1620)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::gamelan_gender_view::{
        GamelanGenderView, GamelanInstrumentType, GAMELAN_PUCK_HIT_RADIUS,
    };
    use crate::views::mpegh_trajectory_view::{
        MpeghTrajectoryProfile, MpeghTrajectoryView, MPEGH_TRAJ_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_radiance_view::{
        NeuralRadianceView, RadianceFieldModel, RADIANCE_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_masking_view::{
        MaskingModelType, SpectralMaskingView, MASKING_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_clipper_view::{
        ClipperProfileType, TransientClipperView, CLIPPER_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1611_1616_gamelan_gender_modal_resonances_and_hit_targets() {
        let mut gamelan = GamelanGenderView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(GAMELAN_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(GAMELAN_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Mallet Hardness Conversion Roundtrip
        for h in [0.05, 0.20, 0.45, 0.65, 0.85, 1.00] {
            let norm = GamelanGenderView::hardness_to_normalized(h);
            assert!((0.0..=1.0).contains(&norm));
            let back = GamelanGenderView::normalized_to_hardness(norm);
            assert!((back - h).abs() < 1e-4, "Hardness mismatch at {}", h);
        }

        // 3. Ombak Beating Rate Conversion Roundtrip
        for rate in [2.0, 3.5, 6.0, 7.2, 8.5, 12.0] {
            let norm = GamelanGenderView::ombak_to_normalized(rate);
            assert!((0.0..=1.0).contains(&norm));
            let back = GamelanGenderView::normalized_to_ombak(norm);
            assert!(
                (back - rate).abs() < 1e-4,
                "Ombak rate mismatch at {}",
                rate
            );
        }

        // 4. Instrument Types and Nominal Values
        for itype in [
            GamelanInstrumentType::GenderWayang10Bar,
            GamelanInstrumentType::GamelanJegogan,
            GamelanInstrumentType::GamelanCalungPemade,
            GamelanInstrumentType::GamelanKanthilTrompong,
            GamelanInstrumentType::GamelanUgalLeader,
        ] {
            gamelan.set_instrument_type(itype);
            let h = itype.nominal_mallet_hardness();
            let ombak = itype.nominal_ombak_rate_hz();
            let bars = itype.nominal_bar_count();
            let thick = itype.nominal_bronze_thickness_mm();
            let damp = itype.nominal_damping_factor();
            let q = itype.nominal_resonator_q();

            assert!((0.05..=1.00).contains(&h));
            assert!((2.0..=12.0).contains(&ombak));
            assert!((5..=14).contains(&bars));
            assert!((4.0..=20.0).contains(&thick));
            assert!((0.10..=0.80).contains(&damp));
            assert!((20.0..=80.0).contains(&q));
            assert!(!itype.instrument_name().is_empty());
        }

        // 5. Physics Simulation Verification
        gamelan.set_instrument_type(GamelanInstrumentType::GenderWayang10Bar);
        gamelan.mallet_hardness = 0.45;
        gamelan.ombak_rate_hz = 7.2;
        gamelan.update_gamelan_simulation();
        assert!(gamelan.modal_amplitudes[0] > 0.3); // Fundamental
        assert!(gamelan.modal_amplitudes[1] > 0.2); // Transverse mode
        assert!(gamelan.modal_amplitudes[4] > 0.1); // Ombak beating modulation

        // 6. Hit Testing on Puck
        gamelan.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(gamelan.hit_test_gamelan_puck((center_x, center_y), canvas));
        assert!(!gamelan.hit_test_gamelan_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = gamelan.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1612_1617_spectral_masking_thresholds_and_hit_targets() {
        let mut masking = SpectralMaskingView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(MASKING_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MASKING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Masker Center Frequency Conversion Roundtrip
        for f in [50.0, 250.0, 1000.0, 4000.0, 16000.0] {
            let norm = SpectralMaskingView::freq_to_normalized(f);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralMaskingView::normalized_to_freq(norm);
            assert!((back - f).abs() / f < 1e-3, "Freq mismatch at {}", f);
        }

        // 3. Masker Level dB Conversion Roundtrip
        for lvl in [20.0, 45.0, 70.0, 85.0, 110.0] {
            let norm = SpectralMaskingView::level_to_normalized(lvl);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralMaskingView::normalized_to_level(norm);
            assert!((back - lvl).abs() < 1e-4, "Level mismatch at {}", lvl);
        }

        // 4. Masking Models and Nominal Values
        for model in [
            MaskingModelType::Mpeg1Layer3Psychoacoustic,
            MaskingModelType::AacLdPsychoacoustic,
            MaskingModelType::OpusCeltPerceptual,
            MaskingModelType::SpatialAudioMasking3D,
            MaskingModelType::HiResMasteringDither,
        ] {
            masking.set_model(model);
            let f = model.nominal_masker_freq_hz();
            let l = model.nominal_masker_level_db();
            let tonality = model.nominal_tonality_index();
            let spread = model.nominal_bark_spread_db();

            assert!((50.0..=16000.0).contains(&f));
            assert!((20.0..=110.0).contains(&l));
            assert!((0.0..=1.0).contains(&tonality));
            assert!((15.0..=35.0).contains(&spread));
            assert!(!model.model_name().is_empty());
        }

        // 5. Critical Bands Masking Simulation
        masking.set_model(MaskingModelType::Mpeg1Layer3Psychoacoustic);
        masking.masker_freq_hz = 1000.0;
        masking.masker_level_db = 85.0;
        masking.update_masking_simulation();
        assert!(masking.global_masking_threshold_db > 50.0);
        assert!(masking.bit_reduction_pct > 20.0);
        assert_eq!(masking.critical_band_thresholds.len(), 8);

        // 6. Hit Testing
        masking.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(masking.hit_test_masking_puck((center_x, center_y), canvas));
        assert!(!masking.hit_test_masking_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = masking.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1613_transient_clipper_transfer_and_hit_targets() {
        let mut clipper = TransientClipperView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(CLIPPER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CLIPPER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Drive dB Conversion Roundtrip
        for d in [-6.0, 0.0, 6.0, 12.0, 18.0] {
            let norm = TransientClipperView::drive_to_normalized(d);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientClipperView::normalized_to_drive(norm);
            assert!((back - d).abs() < 1e-4, "Drive mismatch at {}", d);
        }

        // 3. Knee Softness dB Conversion Roundtrip
        for k in [0.0, 1.5, 3.5, 6.0, 12.0] {
            let norm = TransientClipperView::knee_to_normalized(k);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientClipperView::normalized_to_knee(norm);
            assert!((back - k).abs() < 1e-4, "Knee mismatch at {}", k);
        }

        // 4. Clipper Profiles
        for prof in [
            ClipperProfileType::LinearPhase4BandMaster,
            ClipperProfileType::Oversampled16xPcmCeiling,
            ClipperProfileType::TapeSoftSaturatingClipper,
            ClipperProfileType::HardPunchEDMClipper,
            ClipperProfileType::BroadcastMultiStageClipper,
        ] {
            clipper.set_profile(prof);
            let d = prof.nominal_drive_db();
            let k = prof.nominal_knee_db();
            let over = prof.nominal_oversampling_factor();
            let ceil = prof.nominal_ceiling_dbfs();

            assert!((-6.0..=18.0).contains(&d));
            assert!((0.0..=12.0).contains(&k));
            assert!([1, 2, 4, 8, 16].contains(&over));
            assert!((-2.0..=0.0).contains(&ceil));
            assert!(!prof.profile_name().is_empty());
        }

        // 5. Transfer Function Curve & Multi-Band Reduction Simulation
        clipper.set_profile(ClipperProfileType::LinearPhase4BandMaster);
        clipper.clip_drive_db = 6.0;
        clipper.knee_softness_db = 3.5;
        clipper.update_clipper_simulation();
        assert_eq!(clipper.transfer_curve_pts.len(), 16);
        assert_eq!(clipper.band_clipping_reduction_db.len(), 6);
        assert!(clipper.crest_factor_reduction_db > 1.0);
        assert!(clipper.thd_distortion_pct > 0.0);

        // 6. Hit Testing
        clipper.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(clipper.hit_test_clipper_puck((center_x, center_y), canvas));
        assert!(!clipper.hit_test_clipper_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = clipper.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1614_neural_radiance_room_transfer_and_hit_targets() {
        let mut radiance = NeuralRadianceView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(RADIANCE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RADIANCE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Heading Azimuth Deg Conversion Roundtrip
        for h in [-180.0, -90.0, 0.0, 30.0, 180.0] {
            let norm = NeuralRadianceView::heading_to_normalized(h);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralRadianceView::normalized_to_heading(norm);
            assert!((back - h).abs() < 1e-4, "Heading mismatch at {}", h);
        }

        // 3. Distance Conversion Roundtrip
        for dist in [0.5, 1.2, 3.5, 6.0, 12.0] {
            let norm = NeuralRadianceView::distance_to_normalized(dist);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralRadianceView::normalized_to_distance(norm);
            assert!((back - dist).abs() < 1e-4, "Distance mismatch at {}", dist);
        }

        // 4. Radiance Models
        for model in [
            RadianceFieldModel::InstantNeRFAcoustics,
            RadianceFieldModel::ContinuousFieldHRIR,
            RadianceFieldModel::BinauralRoomRadianceNeRF,
            RadianceFieldModel::WaveguideNeuralMesh,
            RadianceFieldModel::DiffusionRoomImpulseField,
        ] {
            radiance.set_model(model);
            let h = model.nominal_heading_deg();
            let d = model.nominal_distance_m();
            let lat = model.nominal_latent_dim();
            let alpha = model.nominal_absorption_alpha();

            assert!((-180.0..=180.0).contains(&h));
            assert!((0.5..=12.0).contains(&d));
            assert!((64..=512).contains(&lat));
            assert!((0.05..=0.85).contains(&alpha));
            assert!(!model.model_name().is_empty());
        }

        // 5. Acoustic Radiance Simulation
        radiance.set_model(RadianceFieldModel::InstantNeRFAcoustics);
        radiance.heading_deg = 30.0;
        radiance.distance_m = 3.5;
        radiance.update_radiance_simulation();
        assert_eq!(radiance.acoustic_radiance_rays.len(), 8);
        assert!(radiance.directivity_clarity_db > 5.0);
        assert!(radiance.rendering_latency_ms < 10.0);

        // 6. Hit Testing
        radiance.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(radiance.hit_test_radiance_puck((center_x, center_y), canvas));
        assert!(!radiance.hit_test_radiance_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = radiance.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1615_mpegh_dynamic_trajectory_and_hit_targets() {
        let mut mpegh = MpeghTrajectoryView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(MPEGH_TRAJ_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MPEGH_TRAJ_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Deg Conversion Roundtrip
        for az in [-180.0, -90.0, 0.0, 45.0, 180.0] {
            let norm = MpeghTrajectoryView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghTrajectoryView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Deg Conversion Roundtrip
        for el in [-90.0, -45.0, 0.0, 35.0, 90.0] {
            let norm = MpeghTrajectoryView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghTrajectoryView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4, "Elevation mismatch at {}", el);
        }

        // 4. Trajectory Profiles
        for prof in [
            MpeghTrajectoryProfile::Mpegh3DHelicalAscent,
            MpeghTrajectoryProfile::Mpegh3DOverheadFlyby,
            MpeghTrajectoryProfile::Mpegh3DEquatorialOrbit,
            MpeghTrajectoryProfile::Mpegh3DPendulumSwing,
            MpeghTrajectoryProfile::Mpegh3DMultiObjectMaster,
        ] {
            mpegh.set_trajectory(prof);
            let az = prof.nominal_azimuth_deg();
            let el = prof.nominal_elevation_deg();
            let dist = prof.nominal_distance_m();
            let speed = prof.nominal_speed_hz();
            let spread = prof.nominal_object_spread();

            assert!((-180.0..=180.0).contains(&az));
            assert!((-90.0..=90.0).contains(&el));
            assert!((0.5..=20.0).contains(&dist));
            assert!((0.1..=2.0).contains(&speed));
            assert!((0.0..=1.0).contains(&spread));
            assert!(!prof.profile_name().is_empty());
        }

        // 5. VBAP Energy Distribution Simulation
        mpegh.set_trajectory(MpeghTrajectoryProfile::Mpegh3DHelicalAscent);
        mpegh.azimuth_deg = 45.0;
        mpegh.elevation_deg = 35.0;
        mpegh.update_mpegh_simulation();
        assert_eq!(mpegh.speaker_group_gains.len(), 8);
        assert!(mpegh.vbap_energy_focus > 0.4);

        // 6. Hit Testing
        mpegh.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(mpegh.hit_test_mpegh_puck((center_x, center_y), canvas));
        assert!(!mpegh.hit_test_mpegh_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = mpegh.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
