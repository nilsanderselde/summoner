// Summoner DAW - Tier 59 GUI Milestones Unit Test Suite (Steps 1461-1470)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};
    use crate::views::binaural_panner_view::{
        BinauralPannerView, HrtfModel, BINAURAL_PUCK_HIT_RADIUS,
    };
    use crate::views::harmonic_exciter_view::{
        ExciterMode, HarmonicExciterView, EXCITER_PUCK_HIT_RADIUS,
    };
    use crate::views::optical_compressor_view::{
        CompressorTopology, OpticalCompressorView, COMPRESSOR_PUCK_HIT_RADIUS,
    };
    use crate::views::polar_phase_correlator_view::{
        PhaseBallisticsMode, PolarPhaseCorrelatorView, CORRELATOR_HANDLE_HIT_RADIUS,
    };
    use crate::views::resonance_suppressor_view::{
        ResonanceSuppressorView, SuppressorMode, MAX_RESONANCE_NODES, RESONANCE_NODE_HIT_RADIUS,
    };

    #[test]
    fn test_step_1461_1466_dynamic_harmonic_exciter_saturation_and_hit_targets() {
        let mut exciter = HarmonicExciterView::new();
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch area)
        const { assert!(EXCITER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(EXCITER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency Conversion Roundtrip
        for test_f in [1000.0, 2500.0, 5000.0, 10000.0, 15000.0, 20000.0] {
            let norm = HarmonicExciterView::freq_to_normalized(test_f);
            assert!((0.0..=1.0).contains(&norm));
            let back = HarmonicExciterView::normalized_to_freq(norm);
            assert!(
                (back - test_f).abs() / test_f < 1e-4,
                "Frequency mismatch at {}",
                test_f
            );
        }

        // 3. Drive and Brilliance Conversions
        for drive in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let norm = HarmonicExciterView::drive_to_normalized(drive);
            assert!((0.0..=1.0).contains(&norm));
            let back = HarmonicExciterView::normalized_to_drive(norm);
            assert!((back - drive).abs() < 1e-4);
        }

        for brill in [0.0, 4.5, 9.0, 13.5, 18.0] {
            let norm = HarmonicExciterView::brilliance_to_normalized(brill);
            assert!((0.0..=1.0).contains(&norm));
            let back = HarmonicExciterView::normalized_to_brilliance(norm);
            assert!((back - brill).abs() < 1e-4);
        }

        // 4. Harmonic Response Evaluation
        exciter.crossover_freq_hz = 5000.0;
        exciter.drive_percent = 60.0;
        exciter.brilliance_db = 10.0;

        let low_resp = exciter.evaluate_harmonic_response(1000.0);
        let high_resp = exciter.evaluate_harmonic_response(12000.0);
        assert!(
            high_resp > low_resp,
            "High frequency harmonic excitement must exceed low frequency attenuation"
        );

        // 5. Exciter Modes
        for m in [
            ExciterMode::TapeHarmonics,
            ExciterMode::TubeEvenOrder,
            ExciterMode::TransistorOddOrder,
            ExciterMode::PsychoacousticAir,
        ] {
            exciter.mode = m;
            assert_eq!(exciter.mode, m);
        }

        // 6. Hit Testing on Exciter Puck
        exciter.harmonic_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(exciter.hit_test_harmonic_puck((center_x, center_y), canvas));
        assert!(!exciter.hit_test_harmonic_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = exciter.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1462_1467_resonance_suppressor_tracking_and_hit_targets() {
        let mut suppressor = ResonanceSuppressorView::new();
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(RESONANCE_NODE_HIT_RADIUS >= 22.0) };
        const { assert!(RESONANCE_NODE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Frequency & Depth Conversion Roundtrip
        for test_f in [20.0, 100.0, 1000.0, 5000.0, 10000.0, 20000.0] {
            let norm = ResonanceSuppressorView::freq_to_normalized(test_f);
            assert!((0.0..=1.0).contains(&norm));
            let back = ResonanceSuppressorView::normalized_to_freq(norm);
            assert!(
                (back - test_f).abs() / test_f < 1e-4,
                "Frequency mismatch at {}",
                test_f
            );
        }

        for depth in [0.0, 6.0, 12.0, 18.0, 24.0] {
            let norm = ResonanceSuppressorView::depth_to_normalized(depth);
            assert!((0.0..=1.0).contains(&norm));
            let back = ResonanceSuppressorView::normalized_to_depth(norm);
            assert!((back - depth).abs() < 1e-4);
        }

        // 3. Node Add / Remove Constraints
        assert_eq!(suppressor.nodes.len(), 4);
        assert!(suppressor.add_node(1200.0, 10.0, 8.0));
        assert_eq!(suppressor.nodes.len(), 5);

        while suppressor.nodes.len() < MAX_RESONANCE_NODES {
            assert!(suppressor.add_node(3000.0, 15.0, 10.0));
        }
        assert_eq!(suppressor.nodes.len(), MAX_RESONANCE_NODES);
        assert!(!suppressor.add_node(4000.0, 10.0, 6.0)); // Over limit should fail

        assert!(suppressor.remove_node(0));
        assert_eq!(suppressor.nodes.len(), MAX_RESONANCE_NODES - 1);

        // 4. Suppression Response Evaluation
        let center_f = suppressor.nodes[0].freq_hz;
        let notch_att = suppressor.evaluate_suppression_response(center_f);
        let off_att = suppressor.evaluate_suppression_response(center_f * 3.0);
        assert!(
            notch_att >= off_att,
            "Suppression response must peak at node center frequency"
        );

        // 5. Modes
        for m in [
            SuppressorMode::FastSurgical,
            SuppressorMode::MusicalSmooth,
            SuppressorMode::DeepHarmonicTame,
        ] {
            suppressor.mode = m;
            assert_eq!(suppressor.mode, m);
        }

        // 6. Hit Testing on Resonance Node
        let node0_f = suppressor.nodes[0].freq_hz;
        let node0_d = suppressor.nodes[0].depth_db;
        let nx = canvas.x + ResonanceSuppressorView::freq_to_normalized(node0_f) * canvas.width;
        let ny = canvas.y
            + (1.0 - ResonanceSuppressorView::depth_to_normalized(node0_d)) * canvas.height;
        assert!(suppressor.hit_test_node((nx, ny), canvas, 0));
        assert!(!suppressor.hit_test_node((nx + 100.0, ny + 100.0), canvas, 0));

        // 7. Deterministic ASCII Render
        let ascii = suppressor.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1463_optical_compressor_knee_transfer_and_hit_targets() {
        let mut comp = OpticalCompressorView::new();
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(COMPRESSOR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(COMPRESSOR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. dB & Ratio Conversion Roundtrip
        for db in [-60.0, -40.0, -20.0, -10.0, 0.0] {
            let norm = OpticalCompressorView::db_to_normalized(db);
            assert!((0.0..=1.0).contains(&norm));
            let back = OpticalCompressorView::normalized_to_db(norm);
            assert!((back - db).abs() < 1e-4);
        }

        for r in [1.0, 2.0, 4.0, 8.0, 20.0] {
            let norm = OpticalCompressorView::ratio_to_normalized(r);
            assert!((0.0..=1.0).contains(&norm));
            let back = OpticalCompressorView::normalized_to_ratio(norm);
            assert!((back - r).abs() < 1e-4);
        }

        // 3. Soft Knee Transfer Curve Evaluation
        comp.threshold_db = -20.0;
        comp.ratio = 4.0;
        comp.knee_width_db = 10.0;

        // Below knee: linear 1:1
        let below_out = comp.evaluate_transfer_curve(-40.0);
        assert_eq!(below_out, -40.0);

        // Above knee: compressed by ratio 4:1
        let above_out = comp.evaluate_transfer_curve(0.0);
        let expected_above = -20.0 + (0.0 - (-20.0)) / 4.0; // -15.0 dB
        assert!((above_out - expected_above).abs() < 1e-3);

        // Inside knee: smooth transition
        let knee_mid = comp.evaluate_transfer_curve(-20.0);
        assert!(knee_mid < -20.0 + 1.0 && knee_mid > -22.0);

        // 4. Topologies
        for topo in [
            CompressorTopology::OptoT4BTeletronix,
            CompressorTopology::VcaFastFeedForward,
            CompressorTopology::VariMuTubeVariableGain,
            CompressorTopology::FetPeakLimiter1176,
        ] {
            comp.topology = topo;
            assert_eq!(comp.topology, topo);
        }

        // 5. Hit Testing on Knee Puck
        comp.knee_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(comp.hit_test_knee_puck((center_x, center_y), canvas));
        assert!(!comp.hit_test_knee_puck((center_x + 100.0, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = comp.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1464_binaural_panner_hrtf_3d_orbit_and_hit_targets() {
        let mut panner = BinauralPannerView::new();
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(BINAURAL_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(BINAURAL_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth, Elevation & Distance Conversions
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = BinauralPannerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = BinauralPannerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = BinauralPannerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = BinauralPannerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        for dist in [0.1, 1.0, 2.5, 5.0, 10.0] {
            let norm = BinauralPannerView::distance_to_normalized(dist);
            assert!((0.0..=1.0).contains(&norm));
            let back = BinauralPannerView::normalized_to_distance(norm);
            assert!((back - dist).abs() < 1e-4);
        }

        // 3. Acoustic ITD & ILD Calculation
        let (itd_center, ild_center) = BinauralPannerView::calculate_interaural_cues(0.0, 0.0, 1.0);
        assert_eq!(itd_center, 0.0);
        assert_eq!(ild_center, 0.0);

        let (itd_right, ild_right) = BinauralPannerView::calculate_interaural_cues(90.0, 0.0, 1.0);
        assert!(
            itd_right > 500.0,
            "Right-ear source must produce positive ITD"
        );
        assert!(
            ild_right > 10.0,
            "Right-ear source must produce positive ILD"
        );

        // 4. HRTF Models
        for mdl in [
            HrtfModel::KemarStandardDummy,
            HrtfModel::CustomSubjectModel,
            HrtfModel::SphericalHeadRayTraced,
            HrtfModel::NearFieldBinaural,
        ] {
            panner.model = mdl;
            assert_eq!(panner.model, mdl);
        }

        // 5. Hit Testing on Orbit Puck
        panner.azimuth_deg = 0.0;
        panner.distance_m = 5.0;
        let cx = canvas.x + canvas.width * 0.5;
        let cy = canvas.y + canvas.height * 0.5;
        let max_r = (canvas.width.min(canvas.height) * 0.42).max(10.0);
        let dist_norm = BinauralPannerView::distance_to_normalized(5.0);
        let r = 25.0 + dist_norm * (max_r - 25.0);
        let az_rad = (-90.0_f32).to_radians();
        let target_x = cx + az_rad.cos() * r;
        let target_y = cy + az_rad.sin() * r;

        assert!(panner.hit_test_orbit_puck((target_x, target_y), canvas));
        assert!(!panner.hit_test_orbit_puck((target_x + 100.0, target_y + 100.0), canvas));

        // 6. Deterministic ASCII Render
        let ascii = panner.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1465_polar_phase_correlator_and_hit_targets() {
        let mut correlator = PolarPhaseCorrelatorView::new();
        let canvas = Rect::new(20.0, 290.0, 760.0, 185.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(CORRELATOR_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(CORRELATOR_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Correlation & Balance Conversions
        for corr in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let norm = PolarPhaseCorrelatorView::correlation_to_normalized(corr);
            assert!((0.0..=1.0).contains(&norm));
            let back = PolarPhaseCorrelatorView::normalized_to_correlation(norm);
            assert!((back - corr).abs() < 1e-4);
        }

        for bal in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let norm = PolarPhaseCorrelatorView::balance_to_normalized(bal);
            assert!((0.0..=1.0).contains(&norm));
            let back = PolarPhaseCorrelatorView::normalized_to_balance(norm);
            assert!((back - bal).abs() < 1e-4);
        }

        // 3. Mono Compatibility Check
        correlator.correlation_overall = 0.85;
        correlator.band_correlations[0] = 0.95;
        assert!(correlator.is_mono_safe());

        correlator.correlation_overall = -0.3;
        assert!(!correlator.is_mono_safe());

        // 4. Ballistics Modes
        for mode in [
            PhaseBallisticsMode::FastPeakPhase,
            PhaseBallisticsMode::RmsIntegratedCoherence,
            PhaseBallisticsMode::LoudnessWeightedLeq,
        ] {
            correlator.mode = mode;
            assert_eq!(correlator.mode, mode);
        }

        // 5. Hit Testing on MS Handle
        correlator.ms_handle_pos = 0.5;
        let hx = canvas.x + 0.5 * canvas.width;
        let hy = canvas.y + canvas.height * 0.5;
        assert!(correlator.hit_test_ms_handle((hx, hy), canvas));
        assert!(!correlator.hit_test_ms_handle((hx, hy + 100.0), canvas));

        // 6. Deterministic ASCII Render
        let ascii = correlator.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1468_cross_os_dpi_scaling_and_wcag_contrast_compliance_tier59() {
        let palette = ContrastColorPalette::default();
        assert!(palette.is_wcag_aa_compliant());
        assert!(palette.is_wcag_aaa_compliant());

        let cyan = (0, 229, 255);
        let mint = (0, 255, 180);
        let gold = (255, 215, 0);
        let orange = (255, 107, 43);
        let bg = (12, 16, 26);

        assert!(ContrastColorPalette::contrast_ratio(cyan, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(mint, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(gold, bg) >= 4.5);
        assert!(ContrastColorPalette::contrast_ratio(orange, bg) >= 4.5);
    }
}
