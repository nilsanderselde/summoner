// Summoner DAW - Tier 71 GUI Milestones Unit Test Suite (Steps 1581-1590)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::dynamic_crest_shaper_view::{
        CrestTopology, DynamicCrestShaperView, CREST_PUCK_HIT_RADIUS,
    };
    use crate::views::mbira_kalimba_view::{MbiraKalimbaView, MbiraType, MBIRA_PUCK_HIT_RADIUS};
    use crate::views::neural_vocal_stylizer_view::{
        NeuralVocalStylizerView, VocalStyleModel, STYLIZE_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_debleed_view::{
        DebleedMode, SpectralDebleedView, DEBLEED_PUCK_HIT_RADIUS,
    };
    use crate::views::wfs_array_spatializer_view::{
        WfsArraySpatializerView, WfsGeometry, WFS_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1581_1586_mbira_kalimba_modal_dispersion_and_hit_targets() {
        let mut mbira = MbiraKalimbaView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(MBIRA_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MBIRA_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Pluck Force Conversion Roundtrip
        for force in [0.1, 0.9, 1.8, 2.4, 3.5, 5.0] {
            let norm = MbiraKalimbaView::force_to_normalized(force);
            assert!((0.0..=1.0).contains(&norm));
            let back = MbiraKalimbaView::normalized_to_force(norm);
            assert!(
                (back - force).abs() < 1e-4,
                "Pluck force mismatch at {}",
                force
            );
        }

        // 3. Buzz Intensity Conversion Roundtrip
        for buzz in [0.0, 0.05, 0.35, 0.50, 0.85, 1.0] {
            let norm = MbiraKalimbaView::buzz_to_normalized(buzz);
            assert!((0.0..=1.0).contains(&norm));
            let back = MbiraKalimbaView::normalized_to_buzz(norm);
            assert!((back - buzz).abs() < 1e-4, "Buzz mismatch at {}", buzz);
        }

        // 4. Instrument Types and Nominal Values
        for itype in [
            MbiraType::MbiraDzavadzimu,
            MbiraType::NyungaNyunga15,
            MbiraType::HughTraceyKalimba17,
            MbiraType::ArrayMbira5Octave,
            MbiraType::BassKalimbaElectrified,
        ] {
            mbira.set_instrument_type(itype);
            let f = itype.nominal_pluck_force_n();
            let b = itype.nominal_buzz_intensity();
            let t = itype.nominal_tine_count();
            let d = itype.nominal_decay_s();
            assert!((0.1..=5.0).contains(&f));
            assert!((0.0..=1.0).contains(&b));
            assert!((8..=32).contains(&t));
            assert!((0.5..=10.0).contains(&d));
            assert!(!itype.instrument_name().is_empty());
        }

        // 5. Physics Simulation Verification
        mbira.set_instrument_type(MbiraType::MbiraDzavadzimu);
        mbira.pluck_force_n = 2.4;
        mbira.buzz_intensity_pct = 0.85;
        mbira.update_physics_simulation();
        assert!(mbira.modal_amplitudes[0] > 0.5);
        assert!(mbira.modal_amplitudes[3] > 0.5);

        // 6. Hit Testing on Puck
        mbira.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(mbira.hit_test_mbira_puck((center_x, center_y), canvas));
        assert!(!mbira.hit_test_mbira_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = mbira.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1582_1587_spectral_debleed_separation_and_hit_targets() {
        let mut debleed = SpectralDebleedView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(DEBLEED_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DEBLEED_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-60.0, -42.0, -28.0, -15.0, 0.0] {
            let norm = SpectralDebleedView::thresh_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralDebleedView::normalized_to_thresh(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Thresh mismatch at {}",
                thresh
            );
        }

        // 3. Sharpness Conversion Roundtrip
        for sharpness in [0.5, 1.5, 2.4, 3.2, 4.0] {
            let norm = SpectralDebleedView::sharpness_to_normalized(sharpness);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralDebleedView::normalized_to_sharpness(norm);
            assert!(
                (back - sharpness).abs() < 1e-4,
                "Sharpness mismatch at {}",
                sharpness
            );
        }

        // 4. Debleed Modes
        for mode in [
            DebleedMode::DrumKitHiHatBleed,
            DebleedMode::VocalHeadphoneSpill,
            DebleedMode::LiveStageGuitarSpill,
            DebleedMode::AcousticPianoHammerDebleed,
            DebleedMode::OrchestralSectionIsolator,
        ] {
            debleed.set_debleed_mode(mode);
            let th = mode.nominal_thresh_db();
            let g = mode.nominal_mask_sharpness();
            assert!((-60.0..=0.0).contains(&th));
            assert!((0.5..=4.0).contains(&g));
            assert!(!mode.mode_name().is_empty());
        }

        // 5. Spectral Mask Attenuation Checks
        debleed.set_debleed_mode(DebleedMode::DrumKitHiHatBleed);
        debleed.update_spectral_mask();
        for &att in debleed.spectral_attenuations_db.iter() {
            assert!((-48.0..=0.0).contains(&att));
        }

        // 6. Hit Testing
        debleed.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(debleed.hit_test_debleed_puck((center_x, center_y), canvas));
        assert!(!debleed.hit_test_debleed_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = debleed.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1583_dynamic_crest_shaper_punch_and_hit_targets() {
        let mut crest = DynamicCrestShaperView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(CREST_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CREST_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Target Crest Conversion Roundtrip
        for target in [3.0, 8.0, 13.0, 16.5, 24.0] {
            let norm = DynamicCrestShaperView::crest_to_normalized(target);
            assert!((0.0..=1.0).contains(&norm));
            let back = DynamicCrestShaperView::normalized_to_crest(norm);
            assert!((back - target).abs() < 1e-4, "Crest mismatch at {}", target);
        }

        // 3. Expansion Ratio Conversion Roundtrip
        for ratio in [1.0, 1.8, 2.6, 3.2, 4.0] {
            let norm = DynamicCrestShaperView::ratio_to_normalized(ratio);
            assert!((0.0..=1.0).contains(&norm));
            let back = DynamicCrestShaperView::normalized_to_ratio(norm);
            assert!((back - ratio).abs() < 1e-4, "Ratio mismatch at {}", ratio);
        }

        // 4. Topologies
        for topo in [
            CrestTopology::PunchMaximizer,
            CrestTopology::DensityCompactor,
            CrestTopology::MultibandTransientLeveler,
            CrestTopology::AcousticDynamicPreserver,
            CrestTopology::BroadcastLoudnessSculptor,
        ] {
            crest.set_topology(topo);
            let tg = topo.nominal_crest_target_db();
            let exp = topo.nominal_expansion_ratio();
            assert!((3.0..=24.0).contains(&tg));
            assert!((1.0..=4.0).contains(&exp));
            assert!(!topo.topology_name().is_empty());
        }

        // 5. 4-Band Dynamics Simulation
        crest.set_topology(CrestTopology::PunchMaximizer);
        crest.update_dynamics_simulation();
        for &cf in crest.band_crest_factors_db.iter() {
            assert!((3.0..=24.0).contains(&cf));
        }

        // 6. Hit Testing
        crest.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(crest.hit_test_crest_puck((center_x, center_y), canvas));
        assert!(!crest.hit_test_crest_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = crest.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1584_neural_vocal_stylizer_ornaments_and_hit_targets() {
        let mut stylizer = NeuralVocalStylizerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(STYLIZE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(STYLIZE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Style Blend Conversion Roundtrip
        for blend in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = NeuralVocalStylizerView::blend_to_normalized(blend);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralVocalStylizerView::normalized_to_blend(norm);
            assert!((back - blend).abs() < 1e-4, "Blend mismatch at {}", blend);
        }

        // 3. Ornament Depth Conversion Roundtrip
        for ornament in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = NeuralVocalStylizerView::ornament_to_normalized(ornament);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralVocalStylizerView::normalized_to_ornament(norm);
            assert!(
                (back - ornament).abs() < 1e-4,
                "Ornament mismatch at {}",
                ornament
            );
        }

        // 4. Vocal Style Models
        for model in [
            VocalStyleModel::BelCantoOpera,
            VocalStyleModel::ContemporaryPopBelt,
            VocalStyleModel::BulgarianChoirOpenThroat,
            VocalStyleModel::TuvanThroatKargyraa,
            VocalStyleModel::GospelMelismaExpressive,
        ] {
            stylizer.set_vocal_model(model);
            let b = model.nominal_style_blend();
            let o = model.nominal_ornament_depth();
            assert!((0.0..=1.0).contains(&b));
            assert!((0.0..=1.0).contains(&o));
            assert!(!model.model_name().is_empty());
        }

        // 5. Radar Feature Simulation
        stylizer.set_vocal_model(VocalStyleModel::BelCantoOpera);
        stylizer.update_neural_simulation();
        for &val in stylizer.expression_radar_axes.iter() {
            assert!((0.0..=1.0).contains(&val));
        }

        // 6. Hit Testing
        stylizer.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(stylizer.hit_test_stylizer_puck((center_x, center_y), canvas));
        assert!(!stylizer.hit_test_stylizer_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = stylizer.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1585_wfs_array_spatializer_holographic_field_and_hit_targets() {
        let mut wfs = WfsArraySpatializerView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(WFS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(WFS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Lateral X Position Conversion Roundtrip
        for x in [-8.0, -4.0, 0.0, 3.5, 8.0] {
            let norm = WfsArraySpatializerView::x_to_normalized(x);
            assert!((0.0..=1.0).contains(&norm));
            let back = WfsArraySpatializerView::normalized_to_x(norm);
            assert!((back - x).abs() < 1e-4, "X position mismatch at {}", x);
        }

        // 3. Depth Y Position Conversion Roundtrip
        for y in [-5.0, -1.0, 0.0, 3.5, 10.0] {
            let norm = WfsArraySpatializerView::y_to_normalized(y);
            assert!((0.0..=1.0).contains(&norm));
            let back = WfsArraySpatializerView::normalized_to_y(norm);
            assert!((back - y).abs() < 1e-4, "Y position mismatch at {}", y);
        }

        // 4. Geometries
        for geom in [
            WfsGeometry::LinearFrontArray64,
            WfsGeometry::RectangularRoomArray128,
            WfsGeometry::CurvedStageProscenium32,
            WfsGeometry::DualLinearMasteringDesk48,
            WfsGeometry::HexagonalHolographicArray96,
        ] {
            wfs.set_array_geometry(geom);
            let ch = geom.nominal_channels();
            let fc = geom.nominal_aliasing_cutoff_hz();
            assert!((16..=256).contains(&ch));
            assert!((800.0..=8000.0).contains(&fc));
            assert!(!geom.geometry_name().is_empty());
        }

        // 5. Array Delay Calculation Verification
        wfs.set_array_geometry(WfsGeometry::LinearFrontArray64);
        wfs.source_x_m = 0.0;
        wfs.source_y_m = 3.5;
        wfs.update_wfs_simulation();
        assert!(!wfs.is_focused_source);
        for &del in wfs.array_delays_ms.iter() {
            assert!(del > 0.0 && del < 50.0);
        }

        // 6. Hit Testing
        wfs.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(wfs.hit_test_wfs_puck((center_x, center_y), canvas));
        assert!(!wfs.hit_test_wfs_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = wfs.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
