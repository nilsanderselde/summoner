// Summoner DAW - Tier 67 GUI Milestones Unit Test Suite (Steps 1541-1550)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::auro3d_spatializer_view::{
        Auro3dSpatializerView, AuroFormat, AURO3D_PUCK_HIT_RADIUS,
    };
    use crate::views::membrane_plate_view::{
        BoundaryClamping, MembranePlateView, PlateProfile, MEMBRANE_PLATE_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_decompressor_view::{
        DecompressorPreset, MultibandDecompressorView, DECOMPRESSOR_PUCK_HIT_RADIUS,
    };
    use crate::views::neural_inpaint_view::{
        InpaintModel, NeuralInpaintView, INPAINT_PUCK_HIT_RADIUS,
    };
    use crate::views::phase_align_view::{
        MicPairPreset, PhaseAlignView, PHASE_ALIGN_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1541_1546_membrane_plate_modal_frequencies_and_hit_targets() {
        let mut plate = MembranePlateView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(MEMBRANE_PLATE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MEMBRANE_PLATE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Plate Thickness Conversion Roundtrip
        for thick in [0.5, 1.5, 3.2, 10.0, 25.0] {
            let norm = MembranePlateView::thickness_to_normalized(thick);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembranePlateView::normalized_to_thickness(norm);
            assert!(
                (back - thick).abs() < 1e-4,
                "Thickness mismatch at {}",
                thick
            );
        }

        // 3. Membrane Tension Conversion Roundtrip
        for tension in [100.0, 500.0, 3500.0, 7500.0, 10000.0] {
            let norm = MembranePlateView::tension_to_normalized(tension);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembranePlateView::normalized_to_tension(norm);
            assert!(
                (back - tension).abs() < 1e-4,
                "Tension mismatch at {}",
                tension
            );
        }

        // 4. Aspect Ratio Conversion Roundtrip
        for aspect in [0.5, 0.75, 1.0, 1.5, 2.0] {
            let norm = MembranePlateView::aspect_to_normalized(aspect);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembranePlateView::normalized_to_aspect(norm);
            assert!(
                (back - aspect).abs() < 1e-4,
                "Aspect mismatch at {}",
                aspect
            );
        }

        // 5. Plate Profiles
        for prof in [
            PlateProfile::CircularTympanum,
            PlateProfile::RectangularSteelPlate,
            PlateProfile::GongTamTam,
            PlateProfile::SnareBottomMylar,
            PlateProfile::MarimbaRosewoodBar,
        ] {
            plate.set_profile(prof);
            let f0 = prof.nominal_fundamental_hz();
            let th = prof.nominal_thickness_mm();
            let loss = prof.default_loss_factor();
            assert!(f0 > 20.0);
            assert!(th > 0.0);
            assert!(loss > 0.0 && loss < 0.1);
        }

        // 6. Boundary Edge Clamping Impedances
        for b in [
            BoundaryClamping::FreeEdge,
            BoundaryClamping::SimplySupported,
            BoundaryClamping::ClampedRigid,
        ] {
            plate.boundary = b;
            let imp = b.impedance_factor();
            assert!((0.1..=1.0).contains(&imp));
        }

        // 7. Spatial Displacement Evaluation
        let disp_center = plate.evaluate_spatial_displacement(0.5, 0.5);
        let disp_edge = plate.evaluate_spatial_displacement(0.95, 0.5);
        assert!(disp_center.is_finite());
        assert!(disp_edge.is_finite());

        // 8. Hit Testing on Strike Puck
        plate.strike_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(plate.hit_test_strike_puck((center_x, center_y), canvas));
        assert!(!plate.hit_test_strike_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = plate.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1542_1547_phase_align_group_delay_and_hit_targets() {
        let mut align = PhaseAlignView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(PHASE_ALIGN_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(PHASE_ALIGN_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Time Delay Conversion Roundtrip
        for delay in [-50.0, -12.5, 0.0, 2.45, 50.0] {
            let norm = PhaseAlignView::delay_to_normalized(delay);
            assert!((0.0..=1.0).contains(&norm));
            let back = PhaseAlignView::normalized_to_delay(norm);
            assert!((back - delay).abs() < 1e-4, "Delay mismatch at {}", delay);
        }

        // 3. Allpass Frequency Conversion Roundtrip
        for freq in [20.0, 85.0, 1000.0, 7500.0, 20000.0] {
            let norm = PhaseAlignView::freq_to_normalized(freq);
            assert!((0.0..=1.0).contains(&norm));
            let back = PhaseAlignView::normalized_to_freq(norm);
            assert!(
                (back - freq).abs() / freq < 1e-3,
                "Frequency mismatch at {}",
                freq
            );
        }

        // 4. Microphone Pair Presets
        for pr in [
            MicPairPreset::DrumKickInOut,
            MicPairPreset::SnareTopBottom,
            MicPairPreset::OverheadLeftRight,
            MicPairPreset::BassDiAndAmp,
            MicPairPreset::AcousticGuitarDual,
        ] {
            align.set_preset(pr);
            let d = pr.default_delay_ms();
            let f = pr.default_allpass_freq_hz();
            assert!((-50.0..=50.0).contains(&d));
            assert!((20.0..=20000.0).contains(&f));
        }

        // 5. Phase Shift & Comb Magnitude Response
        let shift_100 = align.evaluate_phase_shift(100.0);
        let comb_db = align.evaluate_comb_response_db(100.0);
        assert!(shift_100.is_finite());
        assert!((-36.0..=6.0).contains(&comb_db));

        // 6. Hit Testing on Phase Puck
        align.phase_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(align.hit_test_phase_puck((center_x, center_y), canvas));
        assert!(!align.hit_test_phase_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = align.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1543_multiband_decompressor_upward_expansion_and_hit_targets() {
        let mut decomp = MultibandDecompressorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(DECOMPRESSOR_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DECOMPRESSOR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-60.0, -40.0, -18.0, -6.0, 0.0] {
            let norm = MultibandDecompressorView::thresh_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandDecompressorView::normalized_to_thresh(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 3. Ratio Conversion Roundtrip
        for ratio in [1.0, 1.5, 2.0, 3.2, 4.0] {
            let norm = MultibandDecompressorView::ratio_to_normalized(ratio);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandDecompressorView::normalized_to_ratio(norm);
            assert!((back - ratio).abs() < 1e-4, "Ratio mismatch at {}", ratio);
        }

        // 4. Range Conversion Roundtrip
        for range in [0.0, 3.0, 6.0, 12.0, 18.0] {
            let norm = MultibandDecompressorView::range_to_normalized(range);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandDecompressorView::normalized_to_range(norm);
            assert!((back - range).abs() < 1e-4, "Range mismatch at {}", range);
        }

        // 5. Presets
        for pr in [
            DecompressorPreset::MasterDynRestoration,
            DecompressorPreset::DrumTransientRescue,
            DecompressorPreset::SlapBassPunch,
            DecompressorPreset::VocalAirRestoration,
            DecompressorPreset::OrchestralOpen,
        ] {
            decomp.set_preset(pr);
            let th = pr.default_threshold_db();
            let r = pr.default_ratio();
            assert!((-60.0..=0.0).contains(&th));
            assert!((1.0..=4.0).contains(&r));
        }

        // 6. Upward Expansion Transfer Function Evaluation
        let below = decomp.evaluate_expansion_curve(0, -40.0);
        let above = decomp.evaluate_expansion_curve(0, -6.0);
        assert_eq!(below, -40.0); // No boost below threshold
        assert!(above >= -6.0); // Upward expansion boost

        // 7. Hit Testing on De-Compressor Puck
        decomp.decompressor_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(decomp.hit_test_decompressor_puck((center_x, center_y), canvas));
        assert!(!decomp.hit_test_decompressor_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = decomp.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1544_neural_inpaint_generative_diffusion_and_hit_targets() {
        let mut inpaint = NeuralInpaintView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(INPAINT_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(INPAINT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Time Conversion Roundtrip
        for time in [0.0, 50.0, 250.0, 420.0, 500.0] {
            let norm = NeuralInpaintView::time_to_normalized(time);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralInpaintView::normalized_to_time(norm);
            assert!((back - time).abs() < 1e-4, "Time mismatch at {}", time);
        }

        // 3. Frequency Conversion Roundtrip
        for freq in [20.0, 250.0, 1000.0, 8000.0, 20000.0] {
            let norm = NeuralInpaintView::freq_to_normalized(freq);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralInpaintView::normalized_to_freq(norm);
            assert!(
                (back - freq).abs() / freq < 1e-3,
                "Frequency mismatch at {}",
                freq
            );
        }

        // 4. Inpaint Models
        for model in [
            InpaintModel::DropoutRepair,
            InpaintModel::SpectralDeClick,
            InpaintModel::PlosiveThump,
            InpaintModel::MicClipRestore,
            InpaintModel::StemBleedEraser,
        ] {
            inpaint.set_model(model);
            let steps = model.default_diffusion_steps();
            let guide = model.default_guidance_scale();
            assert!((5..=50).contains(&steps));
            assert!((1.0..=10.0).contains(&guide));
        }

        // 5. 2D Spectrogram Mask Weight Evaluation
        let inside =
            inpaint.evaluate_inpaint_mask(inpaint.mask_center_time_ms, inpaint.mask_center_freq_hz);
        let outside = inpaint.evaluate_inpaint_mask(0.0, 20.0);
        assert!((inside - 1.0).abs() < 1e-4);
        assert_eq!(outside, 0.0);

        // 6. Hit Testing on Inpaint Puck
        inpaint.inpaint_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(inpaint.hit_test_inpaint_puck((center_x, center_y), canvas));
        assert!(!inpaint.hit_test_inpaint_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = inpaint.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1545_auro3d_spatializer_tri_level_and_hit_targets() {
        let mut auro = Auro3dSpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(AURO3D_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(AURO3D_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, 0.0, 35.0, 180.0] {
            let norm = Auro3dSpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Auro3dSpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-3, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Conversion Roundtrip
        for el in [-30.0, 0.0, 28.0, 60.0, 90.0] {
            let norm = Auro3dSpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Auro3dSpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-3, "Elevation mismatch at {}", el);
        }

        // 4. Auro-3D Formats
        for fmt in [
            AuroFormat::Auro131,
            AuroFormat::Auro111,
            AuroFormat::Auro101,
            AuroFormat::Auro91,
            AuroFormat::AuroMaxBinaural,
        ] {
            auro.set_format(fmt);
            let ch = fmt.channel_count();
            assert!(ch > 0);
        }

        // 5. Tri-Level Energy Distribution
        auro.elevation_deg = 0.0;
        auro.update_spatial_energies();
        assert_eq!(auro.bed_layer_energy, 1.0);
        assert_eq!(auro.height_layer_energy, 0.0);

        auro.elevation_deg = 30.0;
        auro.update_spatial_energies();
        assert_eq!(auro.height_layer_energy, 1.0);

        // 6. Cartesian 3D Coordinates
        let (x, y, z) = auro.evaluate_cartesian_position();
        assert!(x.is_finite() && y.is_finite() && z.is_finite());

        // 7. Hit Testing on Auro Puck
        auro.auro_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(auro.hit_test_auro_puck((center_x, center_y), canvas));
        assert!(!auro.hit_test_auro_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = auro.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
