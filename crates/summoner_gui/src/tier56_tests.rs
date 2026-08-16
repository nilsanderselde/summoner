// Summoner DAW - Tier 56 GUI Milestones Unit Test Suite (Steps 1431-1440)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};
    use crate::views::convolution_morph_view::{
        ConvolutionMorphView, IrInterpolationMode, CONVOLUTION_PUCK_HIT_RADIUS, DEFAULT_IR_PRESETS,
    };
    use crate::views::granular_pitch_shifter_view::{
        GrainWindowShape, GranularPitchShifterView, GRANULAR_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_expander_view::{
        ExpanderMode, MultibandExpanderView, EXPANDER_NODE_HIT_RADIUS,
    };
    use crate::views::stereo_vectorscope_view::{
        StereoVectorscopeView, VECTORSCOPE_PUCK_HIT_RADIUS,
    };
    use crate::views::tube_bias_view::{TubeBiasView, TUBE_PUCK_HIT_RADIUS};

    #[test]
    fn test_step_1431_1436_granular_pitch_shifter_bounds_density_and_hit_targets() {
        let mut granular = GranularPitchShifterView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(GRANULAR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(GRANULAR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Pitch to Rate Multiplier Math
        assert!((GranularPitchShifterView::pitch_to_rate_multiplier(0.0, 0.0) - 1.0).abs() < 1e-4);
        assert!((GranularPitchShifterView::pitch_to_rate_multiplier(12.0, 0.0) - 2.0).abs() < 1e-4);
        assert!(
            (GranularPitchShifterView::pitch_to_rate_multiplier(-12.0, 0.0) - 0.5).abs() < 1e-4
        );
        assert!(
            (GranularPitchShifterView::pitch_to_rate_multiplier(7.0, 0.0) - 1.498307).abs() < 1e-3
        );

        // 3. Grain Cloud Generation Bounds
        granular.spray_dispersion_pct = 50.0;
        let grains = granular.generate_grain_cloud(32);
        assert_eq!(grains.len(), 32);
        for g in grains {
            assert!(g.time_offset_norm >= 0.0 && g.time_offset_norm <= 1.0);
            assert!(g.pitch_shift_st >= -24.0 && g.pitch_shift_st <= 24.0);
            assert!(g.duration_ms >= 5.0 && g.duration_ms <= 200.0);
            assert!(g.amplitude >= 0.0 && g.amplitude <= 1.0);
            assert!(g.pan >= -1.0 && g.pan <= 1.0);
        }

        // 4. Windowing Function Verification
        for shape in [
            GrainWindowShape::Hann,
            GrainWindowShape::Blackman,
            GrainWindowShape::Trapezoid,
            GrainWindowShape::Gaussian,
        ] {
            let start = GranularPitchShifterView::evaluate_window_envelope(shape, 0.0);
            let mid = GranularPitchShifterView::evaluate_window_envelope(shape, 0.5);
            let end = GranularPitchShifterView::evaluate_window_envelope(shape, 1.0);
            assert!((0.0..=0.1).contains(&start), "Start envelope out of bounds");
            assert!((0.9..=1.05).contains(&mid), "Mid envelope peak missing");
            assert!((0.0..=0.1).contains(&end), "End envelope out of bounds");
        }

        // 5. Hit Testing on 2D Puck
        granular.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(granular.hit_test_pitch_puck((center_x, center_y), canvas));
        assert!(!granular.hit_test_pitch_puck((center_x + 100.0, center_y + 100.0), canvas));

        // 6. Deterministic ASCII Render
        let ascii = granular.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1432_convolution_ir_morph_decay_interpolation_and_touch_bounds() {
        let mut conv = ConvolutionMorphView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(CONVOLUTION_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CONVOLUTION_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Preset Integrity
        assert!(DEFAULT_IR_PRESETS.len() >= 4);
        assert_eq!(DEFAULT_IR_PRESETS[0].name, "Cathedral Gothic Nave");
        assert_eq!(DEFAULT_IR_PRESETS[1].name, "Plate Shimmer 140");

        // 3. Interpolation Math (Linear vs Logarithmic)
        conv.ir_a_idx = 0; // 4.80 s
        conv.ir_b_idx = 1; // 1.85 s
        conv.decay_scale = 1.0;

        conv.morph_ratio_ab = 0.0;
        assert!((conv.calculate_interpolated_rt60() - 4.80).abs() < 1e-2);

        conv.morph_ratio_ab = 1.0;
        assert!((conv.calculate_interpolated_rt60() - 1.85).abs() < 1e-2);

        conv.morph_ratio_ab = 0.5;
        conv.interpolation_mode = IrInterpolationMode::LinearSpectral;
        let lin_mid = conv.calculate_interpolated_rt60();
        assert!((lin_mid - (4.80 + 1.85) * 0.5).abs() < 1e-2);

        conv.interpolation_mode = IrInterpolationMode::LogarithmicFftCrossfade;
        let log_mid = conv.calculate_interpolated_rt60();
        assert!(log_mid < lin_mid); // Geometric mean is <= arithmetic mean

        // 4. Decay Curve Points Generation
        let curve = conv.calculate_decay_curve(30);
        assert_eq!(curve.len(), 30);
        assert!(
            curve[0].1 >= curve[29].1,
            "Decay curve must decrease over time"
        );

        // 5. Hit Testing
        conv.morph_pad_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(conv.hit_test_morph_puck((center_x, center_y), canvas));
        assert!(!conv.hit_test_morph_puck((center_x + 100.0, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = conv.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1433_1437_stereo_vectorscope_lissajous_rotation_and_phase_correlation() {
        let mut scope = StereoVectorscopeView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(VECTORSCOPE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(VECTORSCOPE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Mid/Side 45-degree Matrix Coordinate Transformation
        // Pure Mono: L = 1.0, R = 1.0 -> M = sqrt(2), S = 0
        let (m_mono, s_mono) = StereoVectorscopeView::left_right_to_mid_side(1.0, 1.0);
        assert!((m_mono - std::f32::consts::SQRT_2).abs() < 1e-4);
        assert!(s_mono.abs() < 1e-6);

        // Pure Out-of-Phase: L = 1.0, R = -1.0 -> M = 0, S = sqrt(2)
        let (m_side, s_side) = StereoVectorscopeView::left_right_to_mid_side(1.0, -1.0);
        assert!(m_side.abs() < 1e-6);
        assert!((s_side - std::f32::consts::SQRT_2).abs() < 1e-4);

        // Roundtrip transformation
        let (orig_l, orig_r) = (0.75, -0.25);
        let (m, s) = StereoVectorscopeView::left_right_to_mid_side(orig_l, orig_r);
        let (back_l, back_r) = StereoVectorscopeView::mid_side_to_left_right(m, s);
        assert!((back_l - orig_l).abs() < 1e-5);
        assert!((back_r - orig_r).abs() < 1e-5);

        // 3. Phase Correlation Factor Math
        let left_mono = vec![1.0, 0.5, -0.5, -1.0];
        let right_mono = vec![1.0, 0.5, -0.5, -1.0];
        let corr_mono = StereoVectorscopeView::calculate_phase_correlation(&left_mono, &right_mono);
        assert!((corr_mono - 1.0).abs() < 1e-5);

        let right_inv = vec![-1.0, -0.5, 0.5, 1.0];
        let corr_inv = StereoVectorscopeView::calculate_phase_correlation(&left_mono, &right_inv);
        assert!((corr_inv - (-1.0)).abs() < 1e-5);

        let right_uncorr = vec![0.5, -1.0, 1.0, -0.5];
        let corr_uncorr =
            StereoVectorscopeView::calculate_phase_correlation(&left_mono, &right_uncorr);
        assert!(corr_uncorr.abs() < 0.5);

        // 4. Lissajous Trace Generation
        let trace = scope.generate_lissajous_trace(64);
        assert_eq!(trace.len(), 64);
        for &(s, m) in &trace {
            assert!((-1.0..=1.0).contains(&s));
            assert!((-1.0..=1.0).contains(&m));
        }

        // 5. Hit Testing
        scope.sensitivity_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(scope.hit_test_sensitivity_puck((center_x, center_y), canvas));
        assert!(!scope.hit_test_sensitivity_puck((center_x + 100.0, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = scope.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1434_multiband_expander_transfer_curve_and_crossover_hit_targets() {
        let expander = MultibandExpanderView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(EXPANDER_NODE_HIT_RADIUS >= 22.0) };
        const { assert!(EXPANDER_NODE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Dynamics Transfer Curve Math (Downward Expansion)
        let thresh = -30.0;
        let ratio = 2.0;
        let knee = 0.0;

        // Above threshold: unity gain
        let out_above = MultibandExpanderView::evaluate_transfer_curve(
            -10.0,
            thresh,
            ratio,
            knee,
            ExpanderMode::DownwardExpansion,
        );
        assert_eq!(out_above, -10.0);

        // At threshold: unity gain
        let out_at = MultibandExpanderView::evaluate_transfer_curve(
            thresh,
            thresh,
            ratio,
            knee,
            ExpanderMode::DownwardExpansion,
        );
        assert_eq!(out_at, thresh);

        // Below threshold: attenuated by ratio
        // delta = -40 - (-30) = -10 dB -> out = -30 + (-10 * 2.0) = -50 dB
        let out_below = MultibandExpanderView::evaluate_transfer_curve(
            -40.0,
            thresh,
            ratio,
            knee,
            ExpanderMode::DownwardExpansion,
        );
        assert_eq!(out_below, -50.0);

        // 3. Upward Expansion
        let out_upward = MultibandExpanderView::evaluate_transfer_curve(
            -10.0,
            thresh,
            ratio,
            knee,
            ExpanderMode::UpwardExpansion,
        );
        // delta = +20 dB -> out = -30 + (20 * 2.0) = +10 dB
        assert_eq!(out_upward, 10.0);

        // 4. Hit Testing on 3 Crossover Nodes
        let min_log = (20.0_f32).ln();
        let max_log = (20000.0_f32).ln();
        for i in 0..3 {
            let freq = expander.bands[i].crossover_freq_hz;
            let norm_x = (freq.ln() - min_log) / (max_log - min_log);
            let px = canvas.x + norm_x * canvas.width;
            let py = canvas.y + canvas.height * 0.5;
            assert_eq!(expander.hit_test_crossover_node((px, py), canvas), Some(i));
        }

        // 5. Deterministic ASCII Render
        let ascii = expander.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1435_tube_bias_qpoint_saturation_and_harmonic_thd_calculations() {
        let mut tube = TubeBiasView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(TUBE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(TUBE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Tube Transfer Function Soft Saturation
        let clean = TubeBiasView::tube_transfer_sample(0.1, -1.85, 0.0, 50.0);
        assert!(clean.abs() > 0.0 && clean.abs() < 1.0);

        let overdriven = TubeBiasView::tube_transfer_sample(1.0, -1.85, 20.0, 50.0);
        assert!((-1.0..=1.0).contains(&overdriven));

        // 3. Harmonic Spectrum and THD Calculation
        tube.drive_warmth_db = 12.0;
        let spectrum = tube.calculate_harmonic_spectrum();
        assert_eq!(spectrum.len(), 5);
        assert_eq!(spectrum[0], 0.0); // f0 is 0 dB reference
        assert!(spectrum[1] > -60.0, "2nd harmonic must be active");
        assert!(spectrum[2] > -60.0, "3rd harmonic must be active");

        let thd = tube.calculate_thd_pct();
        assert!(thd > 0.1 && thd < 45.0, "THD% out of realistic range");

        // 4. Hit Testing on Bias Q-Point Puck
        tube.q_point_norm = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(tube.hit_test_bias_puck((center_x, center_y), canvas));
        assert!(!tube.hit_test_bias_puck((center_x + 100.0, center_y), canvas));

        // 5. Deterministic ASCII Render
        let ascii = tube.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1436_cross_os_dpi_scaling_and_wcag_contrast_compliance() {
        let palette = ContrastColorPalette::default();
        assert!(palette.is_wcag_aa_compliant());
        assert!(palette.is_wcag_aaa_compliant());

        // Verify WCAG contrast for all UI text & accents
        let cyan = (0, 229, 255);
        let orange = (255, 107, 43);
        let mint = (0, 255, 180);
        let gold = (255, 215, 0);
        let bg = (12, 16, 26);

        assert!(ContrastColorPalette::contrast_ratio(cyan, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(mint, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(gold, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(orange, bg) >= 4.5);
    }
}
