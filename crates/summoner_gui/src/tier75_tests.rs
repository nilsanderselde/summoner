// Summoner DAW - Tier 75 GUI Milestones Unit Test Suite (Steps 1621-1630)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::atmos_proximity_view::{
        AtmosProximityProfile, AtmosProximityView, ATMOS_PROX_PUCK_HIT_RADIUS,
    };
    use crate::views::concordance_lattice_view::{
        ConcordanceLatticeView, TuningSystemType, LATTICE_PUCK_HIT_RADIUS,
    };
    use crate::views::diffractive_propagation_view::{
        DiffractionModelType, DiffractivePropagationView, DIFFRACTION_PUCK_HIT_RADIUS,
    };
    use crate::views::transient_reconstructor_view::{
        ReconstructProfileType, TransientReconstructorView, RECONSTRUCT_PUCK_HIT_RADIUS,
    };
    use crate::views::turkish_ney_view::{
        BashpareHornType, TurkishNeyType, TurkishNeyView, NEY_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1621_1626_turkish_ney_embouchure_vortex_and_hit_targets() {
        let mut ney = TurkishNeyView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(NEY_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NEY_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Airjet Velocity Conversion Roundtrip
        for v in [5.0, 15.0, 18.5, 22.0, 26.0, 45.0] {
            let norm = TurkishNeyView::velocity_to_normalized(v);
            assert!((0.0..=1.0).contains(&norm));
            let back = TurkishNeyView::normalized_to_velocity(norm);
            assert!((back - v).abs() < 1e-4, "Velocity mismatch at {}", v);
        }

        // 3. Embouchure Angle Conversion Roundtrip
        for angle in [15.0, 36.0, 38.0, 40.0, 42.0, 65.0] {
            let norm = TurkishNeyView::angle_to_normalized(angle);
            assert!((0.0..=1.0).contains(&norm));
            let back = TurkishNeyView::normalized_to_angle(norm);
            assert!((back - angle).abs() < 1e-4, "Angle mismatch at {}", angle);
        }

        // 4. Instrument Types and Nominal Values
        for ntype in [
            TurkishNeyType::MansurNey,
            TurkishNeyType::KizNey,
            TurkishNeyType::BolahenkNey,
            TurkishNeyType::SupurdeNey,
            TurkishNeyType::SahNey,
        ] {
            ney.set_ney_type(ntype);
            let v = ntype.nominal_jet_velocity_mps();
            let ang = ntype.nominal_embouchure_angle_deg();
            let len = ntype.nominal_bore_length_cm();
            let noise = ntype.nominal_turbulence_noise();
            let loss = ntype.nominal_bore_loss();
            let q = ntype.nominal_acoustic_q();

            assert!((5.0..=45.0).contains(&v));
            assert!((15.0..=65.0).contains(&ang));
            assert!((45.0..=88.0).contains(&len));
            assert!((0.10..=0.50).contains(&noise));
            assert!((0.01..=0.20).contains(&loss));
            assert!((30.0..=70.0).contains(&q));
            assert!(!ntype.instrument_name().is_empty());
        }

        // 5. Başpare Mouthpiece Horn Coupling
        for btype in [
            BashpareHornType::WaterBuffaloHorn,
            BashpareHornType::DelrinAcoustic,
            BashpareHornType::WalnutHardwood,
            BashpareHornType::EbonyReeded,
        ] {
            ney.bashpare_type = btype;
            let c = btype.impedance_coupling();
            assert!((0.80..=1.25).contains(&c));
            assert!(!btype.name().is_empty());
        }

        // 6. Physics Simulation Verification
        ney.set_ney_type(TurkishNeyType::MansurNey);
        ney.jet_velocity_mps = 18.5;
        ney.embouchure_angle_deg = 42.0;
        ney.update_ney_simulation();
        assert!(ney.modal_amplitudes[0] > 0.4); // Fundamental Rast f0
        assert!(ney.modal_amplitudes[4] > 0.1); // Vortex turbulence noise
        assert!(ney.modal_amplitudes[6] > 0.5); // Horn mouthpiece coupling

        // 7. Hit Testing on Puck
        ney.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(ney.hit_test_ney_puck((center_x, center_y), canvas));
        assert!(!ney.hit_test_ney_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = ney.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1622_1627_concordance_lattice_sensory_roughness_and_hit_targets() {
        let mut lattice = ConcordanceLatticeView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(LATTICE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(LATTICE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Cents Conversion Roundtrip
        for cents in [0.0, 100.0, 386.314, 700.0, 701.955, 1200.0] {
            let norm = ConcordanceLatticeView::cents_to_normalized(cents);
            assert!((0.0..=1.0).contains(&norm));
            let back = ConcordanceLatticeView::normalized_to_cents(norm);
            assert!((back - cents).abs() < 1e-3, "Cents mismatch at {}", cents);
        }

        // 3. Tuning System Types
        for sys in [
            TuningSystemType::TwelveTET,
            TuningSystemType::JustIntonationPure,
            TuningSystemType::Pythagorean3Limit,
            TuningSystemType::QuarterCommaMeantone,
            TuningSystemType::Ottoman53TET,
        ] {
            lattice.set_tuning_system(sys);
            let fifth = sys.nominal_fifth_cents();
            let third = sys.nominal_major_third_cents();
            let rough = sys.baseline_roughness();

            assert!((690.0..=705.0).contains(&fifth));
            assert!((380.0..=410.0).contains(&third));
            assert!((0.05..=0.55).contains(&rough));
            assert!(!sys.system_name().is_empty());
        }

        // 4. Plomp-Levelt Roughness Function
        let r_unison = ConcordanceLatticeView::plomp_levelt_roughness(440.0, 440.0);
        let r_dissonant = ConcordanceLatticeView::plomp_levelt_roughness(440.0, 466.16); // Minor second
        let r_fifth = ConcordanceLatticeView::plomp_levelt_roughness(440.0, 660.0); // Perfect fifth
        assert_eq!(r_unison, 0.0);
        assert!(r_dissonant > r_fifth);

        // 5. Hit Testing on Puck
        lattice.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(lattice.hit_test_lattice_puck((center_x, center_y), canvas));
        assert!(!lattice.hit_test_lattice_puck((center_x + 100.0, center_y), canvas));

        // 6. Deterministic ASCII Render
        let ascii = lattice.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1623_1627_transient_reconstructor_lookahead_and_hit_targets() {
        let mut recon = TransientReconstructorView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(RECONSTRUCT_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(RECONSTRUCT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Lookahead ms & Crest dB Roundtrips
        for la in [0.5, 2.5, 4.0, 6.0, 15.0] {
            let norm = TransientReconstructorView::lookahead_to_normalized(la);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientReconstructorView::normalized_to_lookahead(norm);
            assert!((back - la).abs() < 1e-4, "Lookahead mismatch at {}", la);
        }

        for db in [0.0, 2.0, 3.5, 6.0, 12.0] {
            let norm = TransientReconstructorView::crest_to_normalized(db);
            assert!((0.0..=1.0).contains(&norm));
            let back = TransientReconstructorView::normalized_to_crest(norm);
            assert!((back - db).abs() < 1e-4, "Crest dB mismatch at {}", db);
        }

        // 3. Profiles
        for prof in [
            ReconstructProfileType::PunchyMaster,
            ReconstructProfileType::MicroTransientRestorer,
            ReconstructProfileType::DrumStemExpander,
            ReconstructProfileType::AcousticGuitarExciter,
            ReconstructProfileType::TransparentAirCeiling,
        ] {
            recon.set_profile(prof);
            assert!((0.5..=15.0).contains(&recon.lookahead_ms));
            assert!((0.0..=12.0).contains(&recon.crest_factor_boost_db));
            assert!((0.1..=1.0).contains(&recon.transient_sensitivity));
            assert!(recon.fir_taps >= 128);
            assert!(!prof.profile_name().is_empty());
        }

        // 4. Hit Testing and ASCII Render
        recon.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(recon.hit_test_reconstruct_puck((center_x, center_y), canvas));
        assert!(!recon.hit_test_reconstruct_puck((center_x + 100.0, center_y), canvas));

        let ascii = recon.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1624_1627_diffractive_propagation_wavefronts_and_hit_targets() {
        let mut diff = DiffractivePropagationView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(DIFFRACTION_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DIFFRACTION_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Angle and Distance Roundtrips
        for ang in [0.0, 45.0, 75.0, 95.0, 120.0, 180.0] {
            let norm = DiffractivePropagationView::angle_to_normalized(ang);
            assert!((0.0..=1.0).contains(&norm));
            let back = DiffractivePropagationView::normalized_to_angle(norm);
            assert!((back - ang).abs() < 1e-4, "Angle mismatch at {}", ang);
        }

        for d in [0.5, 2.8, 4.5, 6.5, 25.0] {
            let norm = DiffractivePropagationView::distance_to_normalized(d);
            assert!((0.0..=1.0).contains(&norm));
            let back = DiffractivePropagationView::normalized_to_distance(norm);
            assert!((back - d).abs() < 1e-4, "Distance mismatch at {}", d);
        }

        // 3. Models
        for m in [
            DiffractionModelType::SphericalHelmholtzKirchhoff,
            DiffractionModelType::BiotTolstoyMedwinEdge,
            DiffractionModelType::NeuralLatentWavefield,
            DiffractionModelType::ThinWedgeShadow,
            DiffractionModelType::CurvedPillarScattering,
        ] {
            diff.set_model(m);
            assert!((0.0..=180.0).contains(&diff.diffraction_angle_deg));
            assert!((0.5..=25.0).contains(&diff.source_distance_m));
            assert!((0.05..=0.95).contains(&diff.boundary_absorption_alpha));
            assert!(!m.model_name().is_empty());
        }

        // 4. Hit Testing and ASCII Render
        diff.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(diff.hit_test_diffraction_puck((center_x, center_y), canvas));
        assert!(!diff.hit_test_diffraction_puck((center_x + 100.0, center_y), canvas));

        let ascii = diff.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1625_1627_atmos_proximity_panner_and_hit_targets() {
        let mut atmos = AtmosProximityView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(ATMOS_PROX_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ATMOS_PROX_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth and Elevation Roundtrips
        for az in [-180.0, -90.0, -35.0, 0.0, 45.0, 180.0] {
            let norm = AtmosProximityView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosProximityView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        for el in [-90.0, -45.0, 0.0, 25.0, 60.0, 90.0] {
            let norm = AtmosProximityView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosProximityView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4, "Elevation mismatch at {}", el);
        }

        // 3. Proximity Bass Boost and Profiles
        atmos.set_profile(AtmosProximityProfile::VR6DOFObjectSpatial); // 0.4m
        assert!(atmos.distance_m < 1.0);
        assert!(atmos.proximity_bass_boost_db > 3.0); // Boost applied for < 1m

        atmos.set_profile(AtmosProximityProfile::StadiumBroadcastImmersive); // 12.0m
        assert_eq!(atmos.proximity_bass_boost_db, 0.0);
        assert!(atmos.air_absorption_loss_db > 1.0);

        // 4. Hit Testing and ASCII Render
        atmos.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(atmos.hit_test_atmos_puck((center_x, center_y), canvas));
        assert!(!atmos.hit_test_atmos_puck((center_x + 100.0, center_y), canvas));

        let ascii = atmos.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
