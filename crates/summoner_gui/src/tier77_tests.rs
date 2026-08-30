// Summoner DAW - Tier 77 GUI Milestones Unit Test Suite (Plucked String & Sympathetic Coupling HUDs)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::plucked_string_view::{
        PluckedInstrumentViewProfile, PluckedStringView, PLUCK_PUCK_HIT_RADIUS,
    };
    use crate::views::sympathetic_coupling_view::{
        SympatheticCouplingView, SympatheticPreset, SYMPATHETIC_PUCK_HIT_RADIUS, NUM_COUPLED_STRINGS,
    };

    #[test]
    fn test_plucked_string_view_modes_and_hit_targets() {
        let mut view = PluckedStringView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(PLUCK_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(PLUCK_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Pluck Position Beta Conversion Roundtrip
        for beta in [0.05, 0.10, 0.15, 0.25, 0.50, 0.75, 0.95] {
            let norm = PluckedStringView::beta_to_normalized(beta);
            assert!((0.0..=1.0).contains(&norm));
            let back = PluckedStringView::normalized_to_beta(norm);
            assert!((back - beta).abs() < 1e-4, "Beta mismatch at {}", beta);
        }

        // 3. Strike Velocity Conversion Roundtrip
        for vel in [0.05, 0.20, 0.50, 0.80, 0.95, 1.00] {
            let norm = PluckedStringView::velocity_to_normalized(vel);
            assert!((0.0..=1.0).contains(&norm));
            let back = PluckedStringView::normalized_to_velocity(norm);
            assert!((back - vel).abs() < 1e-4, "Velocity mismatch at {}", vel);
        }

        // 4. Instrument Profiles and Physics
        for prof in [
            PluckedInstrumentViewProfile::AcousticSteel,
            PluckedInstrumentViewProfile::ClassicalNylon,
            PluckedInstrumentViewProfile::GrandPiano,
            PluckedInstrumentViewProfile::Harpsichord,
            PluckedInstrumentViewProfile::Harp,
            PluckedInstrumentViewProfile::SitarKoto,
        ] {
            view.instrument = prof;
            view.update_physics();
            let (stiff, t60, detune) = prof.nominal_physics();
            assert!((0.0..=1.0).contains(&stiff));
            assert!(t60 > 1.0);
            assert!(detune > 0.0);
            assert!(!prof.name().is_empty());
        }

        // 5. String Deflection Evaluation
        let defl_center = view.evaluate_string_deflection(0.15);
        assert!(defl_center > 0.5);
        let defl_end = view.evaluate_string_deflection(0.0);
        assert_eq!(defl_end, 0.0);

        // 6. Dual Polarization Orbit
        let (yh, yv) = view.evaluate_polarization_orbit(std::f32::consts::FRAC_PI_2);
        assert!(yv > 0.5);
        assert!(yh.is_finite());

        // 7. Hit Testing on Puck
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_pluck_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_pluck_puck((center_x + 100.0, center_y), canvas));

        // 8. ASCII Fallback Rendering
        let ascii = view.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('#')));

        // 9. Snapshot PNG Rendering
        let res = view.render_snapshot_png("scratch/renders/plucked_string_view.png", 800, 520);
        assert!(res.is_ok());
    }

    #[test]
    fn test_sympathetic_coupling_view_modes_and_hit_targets() {
        let mut view = SympatheticCouplingView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(SYMPATHETIC_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SYMPATHETIC_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Presets and Nominal Values
        for p in [
            SympatheticPreset::AcousticGuitar,
            SympatheticPreset::GrandPiano,
            SympatheticPreset::ClassicalNylon,
            SympatheticPreset::ConcertHarp,
            SympatheticPreset::SitarTarab,
        ] {
            view.preset = p;
            view.update_physics();
            let (bleed, loss) = p.nominal_coupling();
            assert!((0.0..=0.50).contains(&bleed));
            assert!((0.90..=1.0).contains(&loss));
            assert!(!p.name().is_empty());
        }

        // 3. Matrix Cell Evaluation
        for i in 0..NUM_COUPLED_STRINGS {
            for j in 0..NUM_COUPLED_STRINGS {
                let cell = view.evaluate_coupling_matrix_cell(i, j);
                assert!((0.0..=1.0).contains(&cell));
            }
        }

        // 4. Energy Profile
        assert_eq!(view.string_energy_levels.len(), NUM_COUPLED_STRINGS);
        assert!(view.total_radiated_energy > 0.0);

        // 5. Hit Testing on Puck
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_coupling_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_coupling_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Fallback Rendering
        let ascii = view.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));

        // 7. Snapshot PNG Rendering
        let res = view.render_snapshot_png("scratch/renders/sympathetic_coupling_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
