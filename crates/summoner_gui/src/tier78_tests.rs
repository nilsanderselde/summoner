// Summoner DAW - Tier 78 GUI Milestones Unit Test Suite (Membrane Cavity & Idiophone Spectrum HUDs)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::idiophone_spectrum_view::{
        IdiophoneSpectrumView, IdiophoneViewProfile, IDIOPHONE_PUCK_HIT_RADIUS, NUM_VIEW_MODES,
    };
    use crate::views::membrane_cavity_view::{
        MembraneCavityView, MembraneInstrumentViewProfile, MEMBRANE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_membrane_cavity_view_modes_and_hit_targets() {
        let mut view = MembraneCavityView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(MEMBRANE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MEMBRANE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Radial Strike Position Conversion Roundtrip
        for r in [0.00, 0.15, 0.35, 0.50, 0.70, 0.85, 1.00] {
            let norm = MembraneCavityView::radius_to_normalized(r);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembraneCavityView::normalized_to_radius(norm);
            assert!((back - r).abs() < 1e-4, "Radius mismatch at {}", r);
        }

        // 3. Strike Velocity Conversion Roundtrip
        for vel in [0.05, 0.20, 0.50, 0.80, 0.95, 1.00] {
            let norm = MembraneCavityView::velocity_to_normalized(vel);
            assert!((0.0..=1.0).contains(&norm));
            let back = MembraneCavityView::normalized_to_velocity(norm);
            assert!((back - vel).abs() < 1e-4, "Velocity mismatch at {}", vel);
        }

        // 4. Instrument Profiles and Physics
        for prof in [
            MembraneInstrumentViewProfile::TimpaniKettle,
            MembraneInstrumentViewProfile::ConcertBassDrum,
            MembraneInstrumentViewProfile::SnareDrum,
            MembraneInstrumentViewProfile::TomTom,
            MembraneInstrumentViewProfile::BongosCongas,
            MembraneInstrumentViewProfile::DjembeFramedrum,
        ] {
            view.instrument = prof;
            view.update_physics();
            let (air, tens, t60, rim) = prof.nominal_physics();
            assert!((0.0..=1.0).contains(&air));
            assert!(tens > 0.0);
            assert!(t60 > 1.0);
            assert!(rim >= 0.0);
            assert!(!prof.name().is_empty());
        }

        // 5. 2D Bessel Membrane Displacement Evaluation
        let disp_center = view.evaluate_membrane_displacement(0.0, 0.0);
        assert!(disp_center.is_finite());
        let disp_edge = view.evaluate_membrane_displacement(1.0, 0.0);
        assert!(disp_edge.is_finite());

        // 6. Hit Testing on Puck
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_membrane_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_membrane_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Fallback Rendering
        let ascii = view.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('#')));

        // 8. Snapshot PNG Rendering
        let res = view.render_snapshot_png("scratch/renders/membrane_cavity_view.png", 800, 520);
        assert!(res.is_ok());
    }

    #[test]
    fn test_idiophone_spectrum_view_modes_and_hit_targets() {
        let mut view = IdiophoneSpectrumView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(IDIOPHONE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(IDIOPHONE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Presets and Nominal Values
        for p in [
            IdiophoneViewProfile::MarimbaWood,
            IdiophoneViewProfile::XylophoneRosewood,
            IdiophoneViewProfile::VibraphoneAluminum,
            IdiophoneViewProfile::SteelpanTrinidad,
            IdiophoneViewProfile::KalimbaMbira,
            IdiophoneViewProfile::GlockenspielBell,
        ] {
            view.instrument = p;
            view.update_physics();
            let (tube, trem, _h) = p.nominal_physics();
            assert!((0.0..=1.0).contains(&tube));
            assert!(trem >= 0.0);
            assert!(!p.name().is_empty());
        }

        // 3. Hardness and Velocity Roundtrips
        for h in [0.0, 0.25, 0.50, 0.75, 1.0] {
            let norm = IdiophoneSpectrumView::hardness_to_normalized(h);
            assert!((0.0..=1.0).contains(&norm));
            let back = IdiophoneSpectrumView::normalized_to_hardness(norm);
            assert!((back - h).abs() < 1e-4);
        }

        // 4. Modal Spectrum Evaluation
        let spectrum = view.evaluate_modal_spectrum();
        assert_eq!(spectrum.len(), NUM_VIEW_MODES);
        for &amp in &spectrum {
            assert!(amp > 0.0 && amp <= 1.0);
        }

        // 5. Hit Testing on Puck
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_idiophone_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_idiophone_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Fallback Rendering
        let ascii = view.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('#')));

        // 7. Snapshot PNG Rendering
        let res = view.render_snapshot_png("scratch/renders/idiophone_spectrum_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
