// Summoner DAW - Tier 62 GUI Milestones Unit Test Suite (Steps 1491-1500)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::fm_matrix_view::{FmAlgorithm, FmMatrixView, FmWaveform, FM_PUCK_HIT_RADIUS};
    use crate::views::k_system_meter_view::{
        KSystemMeterView, KSystemScale, K_SYSTEM_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_saturator_view::{
        MultibandSaturatorView, SaturationModel, SATURATOR_PUCK_HIT_RADIUS,
    };
    use crate::views::raytraced_reverb_view::{
        RaytracedReverbView, WallMaterial, RAYTRACER_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_grain_cloud_view::{
        GrainWindowShape, SpectralGrainCloudView, GRAIN_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1491_1496_fm_operator_matrix_modulation_indices_and_hit_targets() {
        let mut fm = FmMatrixView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(FM_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(FM_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Modulation Index Conversion Roundtrip
        for test_idx in [0.0, 1.5, 3.5, 7.2, 10.0] {
            let norm = FmMatrixView::mod_index_to_normalized(test_idx);
            assert!((0.0..=1.0).contains(&norm));
            let back = FmMatrixView::normalized_to_mod_index(norm);
            assert!(
                (back - test_idx).abs() < 1e-4,
                "Mod index mismatch at {}",
                test_idx
            );
        }

        // 3. Ratio Conversion Roundtrip
        for test_ratio in [0.5, 1.0, 2.0, 3.5, 8.0, 16.0, 32.0] {
            let norm = FmMatrixView::ratio_to_normalized(test_ratio);
            assert!((0.0..=1.0).contains(&norm));
            let back = FmMatrixView::normalized_to_ratio(norm);
            assert!(
                (back - test_ratio).abs() / test_ratio < 1e-4,
                "Ratio mismatch at {}",
                test_ratio
            );
        }

        // 4. Bessel Sideband Energy Distribution Calculation
        let sb_zero = fm.compute_bessel_sideband_energy(0.0);
        assert!((sb_zero[0] - 1.0).abs() < 1e-3);
        assert!(sb_zero[1] < 1e-3);

        let sb_mod = fm.compute_bessel_sideband_energy(3.5);
        assert!(sb_mod[1] > 0.0);
        assert!(sb_mod[2] > 0.0);

        // 5. Algorithm Presets
        for algo in [
            FmAlgorithm::Algo1LinearCascade,
            FmAlgorithm::Algo5DualCascade,
            FmAlgorithm::Algo16BranchModulator,
            FmAlgorithm::Algo22ParallelCarrier,
            FmAlgorithm::Algo32PureAdditive,
        ] {
            fm.algorithm = algo;
            assert_eq!(fm.algorithm, algo);
        }

        // 6. Waveforms
        for wave in [
            FmWaveform::Sine,
            FmWaveform::Triangle,
            FmWaveform::Sawtooth,
            FmWaveform::SquarePulse,
            FmWaveform::FormantTX,
        ] {
            fm.operators[0].waveform = wave;
            assert_eq!(fm.operators[0].waveform, wave);
        }

        // 7. Hit Testing on Modulation Puck
        fm.matrix_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(fm.hit_test_matrix_puck((center_x, center_y), canvas));
        assert!(!fm.hit_test_matrix_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = fm.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1492_spectral_grain_cloud_and_window_envelopes() {
        let mut cloud = SpectralGrainCloudView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(GRAIN_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(GRAIN_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Grain Rate Conversion Roundtrip
        for rate in [1.0, 10.0, 45.0, 100.0, 200.0] {
            let norm = SpectralGrainCloudView::rate_to_normalized(rate);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralGrainCloudView::normalized_to_rate(norm);
            assert!((back - rate).abs() / rate < 1e-4);
        }

        // 3. Duration Conversion Roundtrip
        for dur in [5.0, 20.0, 65.0, 200.0, 500.0] {
            let norm = SpectralGrainCloudView::duration_to_normalized(dur);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralGrainCloudView::normalized_to_duration(norm);
            assert!((back - dur).abs() / dur < 1e-4);
        }

        // 4. Pitch Transposition Conversion Roundtrip
        for pitch in [-24.0, -12.0, 0.0, 7.0, 24.0] {
            let norm = SpectralGrainCloudView::pitch_to_normalized(pitch);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralGrainCloudView::normalized_to_pitch(norm);
            assert!((back - pitch).abs() < 1e-4);
        }

        // 5. Window Envelope Shapes
        for shape in [
            GrainWindowShape::HannCosine,
            GrainWindowShape::GaussianBell,
            GrainWindowShape::BlackmanHarris,
            GrainWindowShape::TrapezoidLinear,
            GrainWindowShape::ExponentialDecay,
        ] {
            cloud.window_shape = shape;
            let val_mid = cloud.evaluate_window_envelope(0.5);
            assert!(val_mid > 0.0);
            let val_edge = cloud.evaluate_window_envelope(0.0);
            assert!(val_edge >= 0.0);
        }

        // 6. Hit Testing
        cloud.emitter_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(cloud.hit_test_emitter_puck((center_x, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = cloud.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1493_1497_multiband_saturator_transfer_curves_and_hit_targets() {
        let mut sat = MultibandSaturatorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(SATURATOR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SATURATOR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Drive Conversion Roundtrip
        for drive in [0.0, 3.5, 6.8, 12.0, 18.0, 24.0] {
            let norm = MultibandSaturatorView::drive_to_normalized(drive);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandSaturatorView::normalized_to_drive(norm);
            assert!((back - drive).abs() < 1e-4);
        }

        // 3. Bias Conversion Roundtrip
        for bias in [-1.0, -0.5, 0.0, 0.35, 1.0] {
            let norm = MultibandSaturatorView::bias_to_normalized(bias);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandSaturatorView::normalized_to_bias(norm);
            assert!((back - bias).abs() < 1e-4);
        }

        // 4. Saturation Curve Transfer Function Evaluation
        for model in [
            SaturationModel::TapeHysteresis,
            SaturationModel::TriodeTubeWarmth,
            SaturationModel::GermaniumDiodeClip,
            SaturationModel::AsymmetricOverdrive,
            SaturationModel::SoftKneeLimiter,
        ] {
            sat.bands[sat.selected_band_idx].model = model;
            let out_zero = sat.evaluate_transfer_curve(0.0, sat.selected_band_idx);
            let out_pos = sat.evaluate_transfer_curve(1.0, sat.selected_band_idx);
            let out_neg = sat.evaluate_transfer_curve(-1.0, sat.selected_band_idx);
            assert!(out_pos.abs() <= 2.0);
            assert!(out_neg.abs() <= 2.0);
            assert!(!out_zero.is_nan());
        }

        // 5. Hit Testing
        sat.saturator_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(sat.hit_test_saturator_puck((center_x, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = sat.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1494_acoustic_raytraced_reverb_and_materials() {
        let mut reverb = RaytracedReverbView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(RAYTRACER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RAYTRACER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Dimension Conversion Roundtrip
        for dim in [2.0, 8.0, 12.0, 25.0, 50.0] {
            let norm = RaytracedReverbView::dimension_to_normalized(dim);
            assert!((0.0..=1.0).contains(&norm));
            let back = RaytracedReverbView::normalized_to_dimension(norm);
            assert!((back - dim).abs() < 1e-4);
        }

        // 3. Wall Absorption Materials
        for mat in [
            WallMaterial::PolishedConcrete,
            WallMaterial::HardwoodPlank,
            WallMaterial::StudioAcousticFoam,
            WallMaterial::DoubleGlazedGlass,
            WallMaterial::HeavyVelvetDrape,
        ] {
            reverb.room_material = mat;
            let alpha = reverb.get_material_absorption_alpha();
            assert!((0.0..=1.0).contains(&alpha));
        }

        // 4. Sabine RT60 Estimation
        reverb.room_dimensions_m = (10.0, 10.0, 3.0);
        reverb.room_material = WallMaterial::StudioAcousticFoam;
        reverb.update_raytrace_simulation();
        let foam_rt60 = reverb.calculated_rt60_estimate_s;

        reverb.room_material = WallMaterial::PolishedConcrete;
        reverb.update_raytrace_simulation();
        let concrete_rt60 = reverb.calculated_rt60_estimate_s;

        assert!(
            concrete_rt60 > foam_rt60,
            "Polished concrete must have longer RT60 than studio acoustic foam"
        );

        // 5. Hit Testing on Source and Listener
        reverb.source_pos = (0.3, 0.7);
        reverb.listener_pos = (0.6, 0.4);
        let s_x = canvas.x + 0.3 * canvas.width;
        let s_y = canvas.y + (1.0 - 0.7) * canvas.height;
        let l_x = canvas.x + 0.6 * canvas.width;
        let l_y = canvas.y + (1.0 - 0.4) * canvas.height;

        assert!(reverb.hit_test_source_puck((s_x, s_y), canvas));
        assert!(reverb.hit_test_listener_puck((l_x, l_y), canvas));
        assert!(!reverb.hit_test_source_puck((l_x, l_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = reverb.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }

    #[test]
    fn test_step_1495_broadcast_k_system_mastering_metering() {
        let mut k_meter = KSystemMeterView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(K_SYSTEM_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(K_SYSTEM_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Monitor Calibration Trim Roundtrip
        for spl in [70.0, 75.0, 83.0, 85.0, 90.0] {
            let norm = KSystemMeterView::trim_to_normalized(spl);
            assert!((0.0..=1.0).contains(&norm));
            let back = KSystemMeterView::normalized_to_trim(norm);
            assert!((back - spl).abs() < 1e-4);
        }

        // 3. K-System Scale Headroom and Zero VU mappings
        assert_eq!(KSystemScale::K20CinemaClassical.zero_vu_dbfs(), -20.0);
        assert_eq!(KSystemScale::K20CinemaClassical.headroom_db(), 20.0);

        assert_eq!(KSystemScale::K14PopRockBroadcast.zero_vu_dbfs(), -14.0);
        assert_eq!(KSystemScale::K14PopRockBroadcast.headroom_db(), 14.0);

        assert_eq!(KSystemScale::K12RadioCommercial.zero_vu_dbfs(), -12.0);
        assert_eq!(KSystemScale::K12RadioCommercial.headroom_db(), 12.0);

        // 4. K-Scale Delta Conversion
        k_meter.scale = KSystemScale::K14PopRockBroadcast;
        let k_reading = k_meter.dbfs_to_k_scale(-14.0);
        assert!((k_reading - 0.0).abs() < 1e-4); // -14 dBFS on K-14 is 0 VU

        let k_over = k_meter.dbfs_to_k_scale(-10.0);
        assert!((k_over - 4.0).abs() < 1e-4); // -10 dBFS on K-14 is +4 dB

        // 5. Hit Testing
        k_meter.target_trim_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(k_meter.hit_test_trim_puck((center_x, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = k_meter.render_ascii(41, 11);
        assert_eq!(ascii.len(), 11);
    }
}
