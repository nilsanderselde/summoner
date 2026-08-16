// Summoner DAW - Tier 55 GUI Milestones Unit Test Suite (Steps 1421-1430)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::bitcrusher_morph_view::{
        BitcrusherMorphView, MorphQuantizeMode, BITCRUSHER_PUCK_HIT_RADIUS,
    };
    use crate::views::formant_filter_view::{
        FormantFilterView, FORMANT_PUCK_HIT_RADIUS, STANDARD_VOWELS,
    };
    use crate::views::rotary_speaker_view::{
        RotarySpeakerView, RotarySpeedState, ROTARY_HANDLE_HIT_RADIUS, SPEED_OF_SOUND_MPS,
    };
    use crate::views::sidechain_matrix_view::{
        SidechainMatrixView, BUS_NAMES, SIDECHAIN_NODE_HIT_RADIUS, SIDECHAIN_NUM_BUSES,
    };
    use crate::views::spectral_brush_editor::{
        HarmonicSeriesMode, SpectralBrushEditorView, BRUSH_HANDLE_HIT_RADIUS, SPECTRAL_MAX_FREQ_HZ,
        SPECTRAL_MIN_FREQ_HZ,
    };

    #[test]
    fn test_step_1421_1426_spectral_frequency_lasso_polygon_containment_and_touch_bounds() {
        let mut brush = SpectralBrushEditorView::new();
        let canvas = Rect::new(0.0, 50.0, 800.0, 240.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(BRUSH_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(BRUSH_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency Range and Logarithmic Coordinate Transformation
        assert_eq!(
            SpectralBrushEditorView::freq_to_norm_y(SPECTRAL_MIN_FREQ_HZ),
            0.0
        );
        assert_eq!(
            SpectralBrushEditorView::freq_to_norm_y(SPECTRAL_MAX_FREQ_HZ),
            1.0
        );
        let mid_freq = SpectralBrushEditorView::norm_y_to_freq(0.5);
        assert!((mid_freq - (SPECTRAL_MIN_FREQ_HZ * SPECTRAL_MAX_FREQ_HZ).sqrt()).abs() < 1.0);

        // Roundtrip frequency conversion
        for f in [50.0, 220.0, 1000.0, 4400.0, 15000.0] {
            let ny = SpectralBrushEditorView::freq_to_norm_y(f);
            let roundtrip = SpectralBrushEditorView::norm_y_to_freq(ny);
            assert!((roundtrip - f).abs() < 1e-2);
        }

        // 3. Point-in-Polygon Lasso Ray-Casting Containment
        let polygon = vec![(0.1, 0.1), (0.9, 0.1), (0.9, 0.9), (0.1, 0.9)];
        assert!(SpectralBrushEditorView::is_point_in_lasso(
            0.5, 0.5, &polygon
        ));
        assert!(SpectralBrushEditorView::is_point_in_lasso(
            0.2, 0.8, &polygon
        ));
        assert!(!SpectralBrushEditorView::is_point_in_lasso(
            0.05, 0.5, &polygon
        ));
        assert!(!SpectralBrushEditorView::is_point_in_lasso(
            0.95, 0.95, &polygon
        ));

        // 4. Harmonic Series Frequency Multipliers
        brush.fundamental_freq_hz = 100.0;
        brush.harmonic_count = 5;
        brush.harmonic_mode = HarmonicSeriesMode::All;
        let all_harmonics = brush.calculate_harmonic_frequencies();
        assert_eq!(all_harmonics, vec![100.0, 200.0, 300.0, 400.0, 500.0]);

        brush.harmonic_mode = HarmonicSeriesMode::Odd;
        let odd_harmonics = brush.calculate_harmonic_frequencies();
        assert_eq!(odd_harmonics, vec![100.0, 300.0, 500.0]);

        brush.harmonic_mode = HarmonicSeriesMode::Even;
        let even_harmonics = brush.calculate_harmonic_frequencies();
        assert_eq!(even_harmonics, vec![200.0, 400.0]);

        // 5. Hit Testing on Brush Cursor
        brush.cursor_pos_norm = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(brush.hit_test_brush((center_x, center_y), canvas));
        assert!(!brush.hit_test_brush((center_x + 150.0, center_y + 150.0), canvas));

        // 6. Deterministic ASCII Render
        let ascii = brush.render_ascii(32, 16);
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_step_1422_bitcrusher_morphology_transfer_curve_and_hit_targets() {
        let mut bc = BitcrusherMorphView::new();
        let canvas = Rect::new(410.0, 56.0, 370.0, 260.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(BITCRUSHER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(BITCRUSHER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Quantization math across modes
        // 1-bit linear quantization
        let q_pos = BitcrusherMorphView::quantize_sample(0.8, 1.0, MorphQuantizeMode::Linear, 0.0);
        let q_neg = BitcrusherMorphView::quantize_sample(-0.8, 1.0, MorphQuantizeMode::Linear, 0.0);
        assert!(q_pos >= 0.5);
        assert!(q_neg <= -0.5);

        // 8-bit linear quantization precision
        let q8 = BitcrusherMorphView::quantize_sample(0.5123, 8.0, MorphQuantizeMode::Linear, 0.0);
        assert!((q8 - 0.5123).abs() < (1.0 / 256.0));

        // Companded mu-law and A-law output bounded [-1.0 ..= 1.0]
        let mu_q =
            BitcrusherMorphView::quantize_sample(0.42, 4.0, MorphQuantizeMode::CompandedMuLaw, 0.0);
        assert!((-1.0..=1.0).contains(&mu_q));
        let a_q =
            BitcrusherMorphView::quantize_sample(-0.73, 4.0, MorphQuantizeMode::CompandedALaw, 0.0);
        assert!((-1.0..=1.0).contains(&a_q));

        // 3. Transfer Curve Generation
        let curve = bc.calculate_transfer_curve(32);
        assert_eq!(curve.len(), 32);
        assert!((curve[0].0 - (-1.0)).abs() < 1e-4);
        assert!((curve[31].0 - 1.0).abs() < 1e-4);

        // 4. Hit Testing on Morph Puck
        bc.puck_pos = (0.5, 0.5);
        let px = canvas.x + 0.5 * canvas.width;
        let py = canvas.y + 0.5 * canvas.height;
        assert!(bc.hit_test_morph_puck((px, py), canvas));
        assert!(!bc.hit_test_morph_puck((px + 60.0, py + 60.0), canvas));

        // 5. Deterministic ASCII Render
        let ascii = bc.render_ascii(30, 15);
        assert_eq!(ascii.len(), 15);
    }

    #[test]
    fn test_step_1423_formant_filter_vowel_interpolation_and_resonator_spectrum() {
        let mut formant = FormantFilterView::new();
        let canvas = Rect::new(20.0, 56.0, 370.0, 260.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(FORMANT_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(FORMANT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Vowel Definitions Consistency
        assert_eq!(STANDARD_VOWELS.len(), 5);
        for v in &STANDARD_VOWELS {
            assert!(v.f1_hz < v.f2_hz);
            assert!(v.f2_hz < v.f3_hz);
            assert!(v.f3_hz < v.f4_hz);
            assert!(v.f4_hz <= v.f5_hz);
        }

        // 3. Vowel /a/ Formant Interpolation Snapping
        formant.morph_pos = STANDARD_VOWELS[0].pad_norm_pos;
        let f_a = formant.calculate_interpolated_formants();
        assert!((f_a[0] - STANDARD_VOWELS[0].f1_hz).abs() < 15.0);
        assert!((f_a[1] - STANDARD_VOWELS[0].f2_hz).abs() < 25.0);

        // 4. Vocal Tract Length Scaling
        formant.vocal_tract_scale = 1.25;
        let f_scaled = formant.calculate_interpolated_formants();
        assert!((f_scaled[0] - (f_a[0] / 1.25)).abs() < 1.0);

        // 5. Frequency Response Peak Calculation
        formant.vocal_tract_scale = 1.0;
        let resp_at_f1 = formant.evaluate_frequency_response(f_a[0]);
        let resp_at_null = formant.evaluate_frequency_response(50.0);
        assert!(
            resp_at_f1 > resp_at_null,
            "Resonator response at F1 peak must be higher than floor"
        );

        // 6. Hit Testing on Vowel Puck
        let px = canvas.x + formant.morph_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - formant.morph_pos.1) * canvas.height;
        assert!(formant.hit_test_vowel_puck((px, py), canvas));

        // 7. Deterministic ASCII Render
        let ascii = formant.render_ascii(30, 15);
        assert_eq!(ascii.len(), 15);
    }

    #[test]
    fn test_step_1424_1427_rotary_speaker_doppler_angle_equations_and_hit_targets() {
        let mut rotary = RotarySpeakerView::new();

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(ROTARY_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(ROTARY_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Speed States and Target RPMs
        rotary.speed_state = RotarySpeedState::Stop;
        assert_eq!(rotary.target_horn_rpm(), 0.0);
        assert_eq!(rotary.target_drum_rpm(), 0.0);

        rotary.speed_state = RotarySpeedState::Chorale;
        assert_eq!(rotary.target_horn_rpm(), 40.0);
        assert_eq!(rotary.target_drum_rpm(), 36.0);

        rotary.speed_state = RotarySpeedState::Tremolo;
        assert_eq!(rotary.target_horn_rpm(), 400.0);
        assert_eq!(rotary.target_drum_rpm(), 342.0);

        // 3. Doppler Equation Verification
        // At 400 RPM, horn tip velocity = (400 * 2pi / 60) * 0.18 m = ~7.54 m/s
        // Doppler ratio = 7.54 / 343.0 = ~0.022 (+-2.2% pitch shift)
        rotary.horn_rpm = 400.0;
        rotary.horn_angle_rad = std::f32::consts::FRAC_PI_2;
        let doppler_max = rotary.calculate_horn_doppler_shift(0.0);
        assert!((doppler_max - (7.5398 / SPEED_OF_SOUND_MPS)).abs() < 1e-3);

        rotary.horn_angle_rad = 0.0;
        let doppler_zero = rotary.calculate_horn_doppler_shift(0.0);
        assert!(doppler_zero.abs() < 1e-4);

        // 4. Physics Acceleration Integration
        rotary.speed_state = RotarySpeedState::Tremolo;
        rotary.horn_rpm = 0.0;
        rotary.drum_rpm = 0.0;
        rotary.update_physics(0.5); // 0.5s of acceleration
        assert!(rotary.horn_rpm > 0.0);
        assert!(rotary.drum_rpm > 0.0);
        assert!(
            rotary.horn_rpm > rotary.drum_rpm,
            "Horn rotor must accelerate faster than heavy bass drum rotor"
        );

        // 5. Hit Testing on Microphone Handles
        let mic_pos = (150.0, 200.0);
        assert!(rotary.hit_test_mic_handle((150.0, 200.0), mic_pos));
        assert!(rotary.hit_test_mic_handle((165.0, 200.0), mic_pos)); // Within 22pt
        assert!(!rotary.hit_test_mic_handle((200.0, 200.0), mic_pos)); // Outside 22pt

        // 6. Deterministic ASCII Render
        let ascii = rotary.render_ascii(30, 15);
        assert_eq!(ascii.len(), 15);
    }

    #[test]
    fn test_step_1425_sidechain_matrix_routing_and_duck_curve_calculations() {
        let matrix = SidechainMatrixView::new();

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(SIDECHAIN_NODE_HIT_RADIUS >= 22.0) };
        const { assert!(SIDECHAIN_NODE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. 8x8 Routing Matrix Completeness
        assert_eq!(BUS_NAMES.len(), SIDECHAIN_NUM_BUSES);
        assert_eq!(matrix.routes.len(), 64);

        // Verify default route: Kick (0) -> Bass (4)
        let kick_bass = matrix
            .get_route(0, 4)
            .expect("Kick -> Bass route must exist");
        assert!(kick_bass.enabled);

        // 3. Dynamic Duck Curve Gain Reduction
        // Threshold: -18 dB, Ratio: 4:1
        // Input: -18 dB -> GR: 0 dB
        let gr_at_thresh = matrix.calculate_duck_curve_gr(-18.0, -18.0, 4.0);
        assert_eq!(gr_at_thresh, 0.0);

        // Input: -6 dB (12 dB overshoot) -> Compressed to -18 + 12/4 = -15 dB -> GR = -9 dB
        let gr_overshoot = matrix.calculate_duck_curve_gr(-6.0, -18.0, 4.0);
        assert!((gr_overshoot - (-9.0)).abs() < 1e-4);

        // Input: -30 dB (below threshold) -> GR: 0 dB
        let gr_below = matrix.calculate_duck_curve_gr(-30.0, -18.0, 4.0);
        assert_eq!(gr_below, 0.0);

        // 4. Hit Testing on Routing Nodes
        let node_center = (250.0, 180.0);
        assert!(matrix.hit_test_matrix_node((250.0, 180.0), node_center));
        assert!(matrix.hit_test_matrix_node((265.0, 180.0), node_center));
        assert!(!matrix.hit_test_matrix_node((300.0, 180.0), node_center));

        // 5. Deterministic ASCII Render
        let ascii = matrix.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }
}
