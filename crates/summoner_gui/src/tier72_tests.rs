// Summoner DAW - Tier 72 GUI Milestones Unit Test Suite (Steps 1591-1600)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::atmos_916_spatializer_view::{
        Atmos916SpatializerView, AtmosRoomType, ATMOS_PUCK_HIT_RADIUS,
    };
    use crate::views::dulcimer_cimbalom_view::{
        DulcimerCimbalomView, DulcimerType, DULCIMER_PUCK_HIT_RADIUS,
    };
    use crate::views::dynamic_stereo_width_view::{
        DynamicStereoWidthView, StereoWidthProfile, STEREO_PUCK_HIT_RADIUS,
    };
    use crate::views::equal_loudness_contour_view::{
        EqualLoudnessContourView, LoudnessStandard, LOUDNESS_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_choir_formant_view::{
        ChoirEnsembleType, NeuralChoirFormantView, CHOIR_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1591_1596_dulcimer_cimbalom_modal_dispersion_and_hit_targets() {
        let mut dulcimer = DulcimerCimbalomView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(DULCIMER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DULCIMER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Strike Position Conversion Roundtrip
        for pos in [0.05, 0.14, 0.22, 0.35, 0.50] {
            let norm = DulcimerCimbalomView::pos_to_normalized(pos);
            assert!((0.0..=1.0).contains(&norm));
            let back = DulcimerCimbalomView::normalized_to_pos(norm);
            assert!((back - pos).abs() < 1e-4, "Strike pos mismatch at {}", pos);
        }

        // 3. Hammer Hardness Conversion Roundtrip
        for hard in [0.10, 0.50, 0.65, 0.85, 1.00] {
            let norm = DulcimerCimbalomView::hardness_to_normalized(hard);
            assert!((0.0..=1.0).contains(&norm));
            let back = DulcimerCimbalomView::normalized_to_hardness(norm);
            assert!(
                (back - hard).abs() < 1e-4,
                "Hammer hardness mismatch at {}",
                hard
            );
        }

        // 4. Instrument Types and Nominal Values
        for itype in [
            DulcimerType::ConcertGrandCimbalom,
            DulcimerType::AppalachianHammeredDulcimer,
            DulcimerType::PersianSantur,
            DulcimerType::ChineseYangqin,
            DulcimerType::MedievalPsaltery,
        ] {
            dulcimer.set_instrument_type(itype);
            let p = itype.nominal_strike_pos();
            let h = itype.nominal_hammer_hardness();
            let c = itype.nominal_courses();
            let d = itype.nominal_decay_s();
            assert!((0.05..=0.50).contains(&p));
            assert!((0.10..=1.00).contains(&h));
            assert!((12..=40).contains(&c));
            assert!((1.0..=10.0).contains(&d));
            assert!(!itype.instrument_name().is_empty());
        }

        // 5. Physics Simulation Verification
        dulcimer.set_instrument_type(DulcimerType::ConcertGrandCimbalom);
        dulcimer.strike_pos_ratio = 0.14;
        dulcimer.hammer_hardness = 0.65;
        dulcimer.update_physics_simulation();
        assert!(dulcimer.modal_amplitudes[0] > 0.3);
        assert!(dulcimer.modal_amplitudes[5] > 0.1);

        // 6. Hit Testing on Puck
        dulcimer.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(dulcimer.hit_test_dulcimer_puck((center_x, center_y), canvas));
        assert!(!dulcimer.hit_test_dulcimer_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = dulcimer.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1592_1597_equal_loudness_contour_phase_compensation_and_hit_targets() {
        let mut loudness = EqualLoudnessContourView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(LOUDNESS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(LOUDNESS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Monitoring SPL Conversion Roundtrip
        for spl in [40.0, 60.0, 75.0, 83.0, 100.0] {
            let norm = EqualLoudnessContourView::spl_to_normalized(spl);
            assert!((0.0..=1.0).contains(&norm));
            let back = EqualLoudnessContourView::normalized_to_spl(norm);
            assert!((back - spl).abs() < 1e-4, "SPL mismatch at {}", spl);
        }

        // 3. Compensation Amount Conversion Roundtrip
        for comp in [0.0, 0.25, 0.50, 0.85, 1.0] {
            let norm = EqualLoudnessContourView::comp_to_normalized(comp);
            assert!((0.0..=1.0).contains(&norm));
            let back = EqualLoudnessContourView::normalized_to_comp(norm);
            assert!(
                (back - comp).abs() < 1e-4,
                "Compensation mismatch at {}",
                comp
            );
        }

        // 4. Loudness Standards
        for std in [
            LoudnessStandard::Iso226_2003,
            LoudnessStandard::FletcherMunson1933,
            LoudnessStandard::RobinsonDadson1956,
            LoudnessStandard::EbuR128KWeighted,
            LoudnessStandard::CinemaXCurve,
        ] {
            loudness.set_standard(std);
            let spl = std.nominal_spl_db();
            let comp = std.nominal_compensation();
            assert!((40.0..=100.0).contains(&spl));
            assert!((0.0..=1.0).contains(&comp));
            assert!(!std.standard_name().is_empty());
        }

        // 5. Critical Bands Gain Simulation
        loudness.set_standard(LoudnessStandard::Iso226_2003);
        loudness.monitoring_spl_db = 60.0;
        loudness.compensation_amount = 0.85;
        loudness.update_contour_simulation();
        assert!(loudness.bass_boost_db > 0.0); // Bass should be boosted when monitoring quieter than 83dB SPL
        assert_eq!(loudness.band_gains_db.len(), 8);

        // 6. Hit Testing
        loudness.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(loudness.hit_test_loudness_puck((center_x, center_y), canvas));
        assert!(!loudness.hit_test_loudness_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = loudness.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1593_dynamic_stereo_width_elliptical_mono_and_hit_targets() {
        let mut stereo = DynamicStereoWidthView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(STEREO_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(STEREO_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Elliptical Bass Cutoff Roundtrip
        for hz in [40.0, 80.0, 120.0, 180.0, 300.0] {
            let norm = DynamicStereoWidthView::bass_to_normalized(hz);
            assert!((0.0..=1.0).contains(&norm));
            let back = DynamicStereoWidthView::normalized_to_bass(norm);
            assert!((back - hz).abs() < 1e-4, "Bass cutoff mismatch at {}", hz);
        }

        // 3. Side Width Ratio Roundtrip
        for width in [0.0, 0.95, 1.25, 1.60, 2.50] {
            let norm = DynamicStereoWidthView::width_to_normalized(width);
            assert!((0.0..=1.0).contains(&norm));
            let back = DynamicStereoWidthView::normalized_to_width(norm);
            assert!((back - width).abs() < 1e-4, "Width mismatch at {}", width);
        }

        // 4. Stereo Profiles
        for prof in [
            StereoWidthProfile::BroadcastMasteringClean,
            StereoWidthProfile::ClubVinylPressing,
            StereoWidthProfile::CinematicSuperWide,
            StereoWidthProfile::AcousticNaturalDepth,
            StereoWidthProfile::EDMPolyrhythmicHyperWidth,
        ] {
            stereo.set_profile(prof);
            let fc = prof.nominal_bass_cutoff_hz();
            let w = prof.nominal_width_ratio();
            assert!((40.0..=300.0).contains(&fc));
            assert!((0.0..=2.5).contains(&w));
            assert!(!prof.profile_name().is_empty());
        }

        // 5. 4-Band Width Simulation
        stereo.set_profile(StereoWidthProfile::BroadcastMasteringClean);
        stereo.elliptical_bass_hz = 120.0;
        stereo.side_width_ratio = 1.25;
        stereo.update_stereo_simulation();
        assert_eq!(stereo.band_widths[0], 0.0); // Sub bass should be mono below 120Hz
        assert!(stereo.band_widths[3] > 1.0); // Air band expanded
        assert!(stereo.phase_correlation > 0.5);

        // 6. Hit Testing
        stereo.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(stereo.hit_test_stereo_puck((center_x, center_y), canvas));
        assert!(!stereo.hit_test_stereo_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = stereo.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1594_neural_choir_formant_morpher_and_hit_targets() {
        let mut choir = NeuralChoirFormantView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(CHOIR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CHOIR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Formant F1 Roundtrip
        for f1 in [200.0, 450.0, 650.0, 780.0, 1000.0] {
            let norm = NeuralChoirFormantView::f1_to_normalized(f1);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralChoirFormantView::normalized_to_f1(norm);
            assert!((back - f1).abs() < 1e-4, "F1 mismatch at {}", f1);
        }

        // 3. Formant F2 Roundtrip
        for f2 in [500.0, 950.0, 1400.0, 1950.0, 3000.0] {
            let norm = NeuralChoirFormantView::f2_to_normalized(f2);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralChoirFormantView::normalized_to_f2(norm);
            assert!((back - f2).abs() < 1e-4, "F2 mismatch at {}", f2);
        }

        // 4. Choir Ensemble Types
        for ens in [
            ChoirEnsembleType::ClassicalSATB,
            ChoirEnsembleType::BulgarianWomensChoir,
            ChoirEnsembleType::GregorianMonasticChant,
            ChoirEnsembleType::ContemporaryVocalEnsemble,
            ChoirEnsembleType::GospelChoirWallOfSound,
        ] {
            choir.set_ensemble(ens);
            let f1 = ens.nominal_f1_hz();
            let f2 = ens.nominal_f2_hz();
            let v = ens.nominal_voice_count();
            assert!((200.0..=1000.0).contains(&f1));
            assert!((500.0..=3000.0).contains(&f2));
            assert!((4..=64).contains(&v));
            assert!(!ens.ensemble_name().is_empty());
        }

        // 5. Vocal Formant Simulation
        choir.set_ensemble(ChoirEnsembleType::ClassicalSATB);
        choir.formant_f1_hz = 650.0;
        choir.formant_f2_hz = 1400.0;
        choir.update_choir_simulation();
        assert!(choir.formant_f3_hz > choir.formant_f2_hz);
        assert_eq!(choir.voice_formant_peaks.len(), 5);

        // 6. Hit Testing
        choir.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(choir.hit_test_choir_puck((center_x, center_y), canvas));
        assert!(!choir.hit_test_choir_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = choir.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1595_atmos_916_acoustic_room_raytracing_and_hit_targets() {
        let mut atmos = Atmos916SpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(ATMOS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ATMOS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Coordinate Conversion Roundtrip
        for coord in [-8.0, -4.0, 0.0, 1.5, 8.0] {
            let norm = Atmos916SpatializerView::coord_to_normalized(coord);
            assert!((0.0..=1.0).contains(&norm));
            let back = Atmos916SpatializerView::normalized_to_coord(norm);
            assert!((back - coord).abs() < 1e-4, "Coord mismatch at {}", coord);
        }

        // 3. Atmos Room Types
        for room in [
            AtmosRoomType::MasteringStage916,
            AtmosRoomType::CinemaAuditoriumDolby,
            AtmosRoomType::NearfieldMixingStudio,
            AtmosRoomType::BinauralAtmosSpatializer,
            AtmosRoomType::CarAudio16ChannelArray,
        ] {
            atmos.set_room_type(room);
            let ch = room.nominal_channels();
            let rays = room.nominal_ray_count();
            let rt = room.nominal_reverb_rt60_s();
            assert!((2..=64).contains(&ch));
            assert!((32..=512).contains(&rays));
            assert!((0.1..=2.0).contains(&rt));
            assert!(!room.room_name().is_empty());
        }

        // 4. 16-Channel Raytrace Simulation
        atmos.set_room_type(AtmosRoomType::MasteringStage916);
        atmos.source_x_m = 1.5;
        atmos.source_y_m = 2.0;
        atmos.update_raytrace_simulation();
        assert_eq!(atmos.speaker_energy_levels.len(), 16);
        for &energy in atmos.speaker_energy_levels.iter() {
            assert!((0.0..=1.0).contains(&energy));
        }

        // 5. Hit Testing
        atmos.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(atmos.hit_test_atmos_puck((center_x, center_y), canvas));
        assert!(!atmos.hit_test_atmos_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Render
        let ascii = atmos.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
