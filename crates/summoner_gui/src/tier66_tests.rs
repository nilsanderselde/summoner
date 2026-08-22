// Summoner DAW - Tier 66 GUI Milestones Unit Test Suite (Steps 1531-1540)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::mpegh_spatializer_view::{
        HrtfProfile, MpeghFormat, MpeghSpatializerView, MPEGH_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_timbre_view::{
        NeuralTimbreView, TimbreModel, NEURAL_TIMBRE_PUCK_HIT_RADIUS,
    };
    use crate::views::oversampled_limiter_view::{
        DitherShapingCurve, LimiterProfile, OversampledLimiterView, LIMITER_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_reshaper_view::{
        ReshaperPreset, SpectralReshaperView, RESHAPER_PUCK_HIT_RADIUS,
    };
    use crate::views::woodwind_jet_view::{
        WoodwindInstrument, WoodwindJetView, WOODWIND_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1531_1536_woodwind_jet_impedance_and_hit_targets() {
        let mut woodwind = WoodwindJetView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(WOODWIND_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(WOODWIND_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Jet Pressure Conversion Roundtrip
        for pressure in [0.10, 0.50, 1.25, 2.50, 4.00] {
            let norm = WoodwindJetView::pressure_to_normalized(pressure);
            assert!((0.0..=1.0).contains(&norm));
            let back = WoodwindJetView::normalized_to_pressure(norm);
            assert!(
                (back - pressure).abs() < 1e-4,
                "Pressure mismatch at {}",
                pressure
            );
        }

        // 3. Jet Offset Conversion Roundtrip
        for offset in [2.0, 4.5, 7.0, 11.5, 15.0] {
            let norm = WoodwindJetView::offset_to_normalized(offset);
            assert!((0.0..=1.0).contains(&norm));
            let back = WoodwindJetView::normalized_to_offset(norm);
            assert!(
                (back - offset).abs() < 1e-4,
                "Offset mismatch at {}",
                offset
            );
        }

        // 4. Bore Length Conversion Roundtrip
        for length in [0.20, 0.32, 0.60, 0.95, 1.20] {
            let norm = WoodwindJetView::length_to_normalized(length);
            assert!((0.0..=1.0).contains(&norm));
            let back = WoodwindJetView::normalized_to_length(norm);
            assert!(
                (back - length).abs() < 1e-4,
                "Bore length mismatch at {}",
                length
            );
        }

        // 5. Instrument Presets & Properties
        for inst in [
            WoodwindInstrument::FluteC,
            WoodwindInstrument::PiccoloC,
            WoodwindInstrument::RecorderAlto,
            WoodwindInstrument::Shakuhachi,
            WoodwindInstrument::PanFlute,
        ] {
            woodwind.instrument = inst;
            let nom_len = inst.nominal_tube_length_m();
            let jet_dist = inst.nominal_jet_distance_mm();
            let cutoff = inst.nominal_cutoff_hz();
            assert!(nom_len > 0.0);
            assert!(jet_dist > 0.0);
            assert!((1000.0..=6000.0).contains(&cutoff));
        }

        // 6. Effective Tube Length from Tonehole Fingerings
        woodwind.tonehole_state = [true, true, true, true, true, true]; // all closed
        let len_all_closed = woodwind.calculate_effective_tube_length();
        woodwind.tonehole_state = [false, false, false, false, false, false]; // all open
        let len_all_open = woodwind.calculate_effective_tube_length();
        assert!(
            len_all_open < len_all_closed,
            "Opening toneholes must shorten effective acoustic bore length"
        );

        // 7. Hit Testing on Jet Puck
        woodwind.jet_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(woodwind.hit_test_jet_puck((center_x, center_y), canvas));
        assert!(!woodwind.hit_test_jet_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = woodwind.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1532_1537_spectral_reshaper_dynamic_envelopes_and_hit_targets() {
        let mut reshaper = SpectralReshaperView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(RESHAPER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RESHAPER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Attack Gain Conversion Roundtrip
        for attack in [-12.0, -6.0, 0.0, 4.5, 12.0] {
            let norm = SpectralReshaperView::attack_to_normalized(attack);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralReshaperView::normalized_to_attack(norm);
            assert!(
                (back - attack).abs() < 1e-4,
                "Attack mismatch at {}",
                attack
            );
        }

        // 3. Sustain Gain Conversion Roundtrip
        for sustain in [-12.0, -4.5, 0.0, 3.0, 12.0] {
            let norm = SpectralReshaperView::sustain_to_normalized(sustain);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralReshaperView::normalized_to_sustain(norm);
            assert!(
                (back - sustain).abs() < 1e-4,
                "Sustain mismatch at {}",
                sustain
            );
        }

        // 4. De-Bleed Threshold Conversion Roundtrip
        for thresh in [-60.0, -45.0, -30.0, -15.0, 0.0] {
            let norm = SpectralReshaperView::thresh_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralReshaperView::normalized_to_thresh(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 5. Preset Profiles
        for preset in [
            ReshaperPreset::DrumKitOverhead,
            ReshaperPreset::SnareCloseMic,
            ReshaperPreset::AcousticGuitarSnap,
            ReshaperPreset::VocalPlosiveTamer,
            ReshaperPreset::MasterPunch,
        ] {
            reshaper.set_preset(preset);
            let xo = preset.default_crossovers_hz();
            assert!(xo[0] < xo[1] && xo[1] < xo[2]);
            let att = preset.default_attack_db();
            assert_eq!(att.len(), 4);
        }

        // 6. Multi-Band Frequency Response Evaluation
        let resp_low = reshaper.evaluate_frequency_response(100.0);
        let resp_high = reshaper.evaluate_frequency_response(10000.0);
        assert!(resp_low.is_finite());
        assert!(resp_high.is_finite());

        // 7. Hit Testing on Band Puck
        reshaper.band_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(reshaper.hit_test_band_puck((center_x, center_y), canvas));
        assert!(!reshaper.hit_test_band_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = reshaper.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1533_oversampled_limiter_inter_sample_and_hit_targets() {
        let mut limiter = OversampledLimiterView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(LIMITER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(LIMITER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-18.0, -12.0, -6.0, -2.5, 0.0] {
            let norm = OversampledLimiterView::thresh_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = OversampledLimiterView::normalized_to_thresh(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 3. Ceiling Conversion Roundtrip
        for ceiling in [-6.0, -3.0, -1.0, -0.3, 0.0] {
            let norm = OversampledLimiterView::ceiling_to_normalized(ceiling);
            assert!((0.0..=1.0).contains(&norm));
            let back = OversampledLimiterView::normalized_to_ceiling(norm);
            assert!(
                (back - ceiling).abs() < 1e-4,
                "Ceiling mismatch at {}",
                ceiling
            );
        }

        // 4. Release Time Conversion Roundtrip
        for rel in [1.0, 15.0, 85.0, 250.0, 1000.0] {
            let norm = OversampledLimiterView::release_to_normalized(rel);
            assert!((0.0..=1.0).contains(&norm));
            let back = OversampledLimiterView::normalized_to_release(norm);
            assert!(
                (back - rel).abs() < 1e-2,
                "Release mismatch at {}",
                rel
            );
        }

        // 5. Limiter Profiles
        for profile in [
            LimiterProfile::TransparentClean,
            LimiterProfile::WarmAnalogTape,
            LimiterProfile::PunchyTransient,
            LimiterProfile::BroadcastEbuR128,
            LimiterProfile::AggressiveClubLoudness,
        ] {
            limiter.set_profile(profile);
            let ceil = profile.default_ceiling_dbtp();
            let thr = profile.default_threshold_db();
            assert!(ceil <= 0.0 && ceil >= -6.0);
            assert!(thr <= 0.0 && thr >= -18.0);
        }

        // 6. Dither Noise Shaping Curves
        for curve in [
            DitherShapingCurve::FlatTpdf,
            DitherShapingCurve::Lipshitz,
            DitherShapingCurve::EWeighted,
            DitherShapingCurve::FWeighted,
            DitherShapingCurve::ModifiedShibata,
        ] {
            limiter.dither_curve = curve;
            let snr = curve.snr_improvement_db();
            assert!(snr >= 0.0);
            let mag_1k = limiter.evaluate_noise_shaping_curve(1000.0);
            let mag_18k = limiter.evaluate_noise_shaping_curve(18000.0);
            assert!(mag_1k < 0.0 && mag_18k < 0.0);
        }

        // 7. Sinc ISP Reconstruction
        let sinc_peak = limiter.evaluate_sinc_isp(0.0);
        let sinc_sub = limiter.evaluate_sinc_isp(0.5);
        assert!((sinc_peak - 1.0).abs() < 1e-4);
        assert!(sinc_sub < sinc_peak);

        // 8. Hit Testing on Limiter Puck
        limiter.limiter_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(limiter.hit_test_limiter_puck((center_x, center_y), canvas));
        assert!(!limiter.hit_test_limiter_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = limiter.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1534_neural_timbre_transfer_and_hit_targets() {
        let mut neural = NeuralTimbreView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(NEURAL_TIMBRE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NEURAL_TIMBRE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Latent Coordinate Conversion Roundtrip
        for coord in [-2.0, -1.0, 0.0, 0.85, 2.0] {
            let norm = NeuralTimbreView::coord_to_normalized(coord);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralTimbreView::normalized_to_coord(norm);
            assert!(
                (back - coord).abs() < 1e-4,
                "Coord mismatch at {}",
                coord
            );
        }

        // 3. Flow Rate Conversion Roundtrip
        for flow in [0.05, 0.40, 1.20, 5.00, 10.00] {
            let norm = NeuralTimbreView::flow_to_normalized(flow);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralTimbreView::normalized_to_flow(norm);
            assert!(
                (back - flow).abs() < 1e-4,
                "Flow mismatch at {}",
                flow
            );
        }

        // 4. Timbre Models
        for model in [
            TimbreModel::VocalFormantMorph,
            TimbreModel::CelloResonanceFlow,
            TimbreModel::AnalogMoogLead,
            TimbreModel::GlassMalletBell,
            TimbreModel::AlienBiomorphic,
        ] {
            neural.set_model(model);
            let flow = model.default_flow_rate_hz();
            let mse = model.convergence_mse();
            assert!(flow > 0.0);
            assert!(mse > 0.0 && mse < 0.1);
        }

        // 5. Flow Velocity ODE Vector Field
        let (vx, vy) = neural.evaluate_flow_velocity(0.5, -0.3);
        assert!(vx.is_finite() && vy.is_finite());

        // 6. Spectral Envelope Formant Transfer
        let (src, out) = neural.evaluate_spectral_envelope(0.25);
        assert!(src >= 0.0 && out >= 0.0);

        // 7. Hit Testing on Neural Timbre Puck
        neural.timbre_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(neural.hit_test_timbre_puck((center_x, center_y), canvas));
        assert!(!neural.hit_test_timbre_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = neural.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1535_mpegh_spatializer_and_hit_targets() {
        let mut mpegh = MpeghSpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(MPEGH_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MPEGH_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, 0.0, 45.0, 180.0] {
            let norm = MpeghSpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghSpatializerView::normalized_to_azimuth(norm);
            assert!(
                (back - az).abs() < 1e-3,
                "Azimuth mismatch at {}",
                az
            );
        }

        // 3. Elevation Conversion Roundtrip
        for el in [-90.0, -45.0, 0.0, 15.0, 90.0] {
            let norm = MpeghSpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghSpatializerView::normalized_to_elevation(norm);
            assert!(
                (back - el).abs() < 1e-3,
                "Elevation mismatch at {}",
                el
            );
        }

        // 4. MPEG-H Formats
        for fmt in [
            MpeghFormat::Mpegh714,
            MpeghFormat::Mpegh51,
            MpeghFormat::Mpegh222Dome,
            MpeghFormat::MpeghBinaural,
            MpeghFormat::MpeghDynamicObj,
        ] {
            mpegh.set_format(fmt);
            let ch = fmt.channel_count();
            assert!(ch > 0);
        }

        // 5. Personalized HRTF Profiles
        for prof in [
            HrtfProfile::KemarStandard,
            HrtfProfile::GenelecAural,
            HrtfProfile::SphericalHead,
            HrtfProfile::Photogrammetry3D,
        ] {
            mpegh.hrtf_profile = prof;
            let notch = prof.notch_freq_khz();
            assert!((5.0..=12.0).contains(&notch));
        }

        // 6. ITD & ILD Binaural Calculations (Woodworth Formula)
        mpegh.azimuth_deg = 90.0;
        mpegh.elevation_deg = 0.0;
        mpegh.update_spatial_calculations();
        assert!(mpegh.itd_microseconds > 600.0, "ITD at 90 deg must be large (> 600 μs)");
        assert!(mpegh.ild_db > 10.0, "ILD at 90 deg must be large (> 10 dB)");

        // 7. HRTF Magnitude Response
        let (mag_l, mag_r) = mpegh.evaluate_hrtf_magnitude(4000.0);
        assert!(mag_l.is_finite() && mag_r.is_finite());

        // 8. Hit Testing on MPEG-H Puck
        mpegh.object_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(mpegh.hit_test_object_puck((center_x, center_y), canvas));
        assert!(!mpegh.hit_test_object_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = mpegh.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
