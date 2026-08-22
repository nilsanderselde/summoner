// Summoner DAW - Tier 73 GUI Milestones Unit Test Suite (Steps 1601-1610)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::auditory_roughness_view::{
        AuditoryRoughnessView, RoughnessStandard, ROUGHNESS_PUCK_HIT_RADIUS,
    };
    use crate::views::hoa5_binaural_view::{
        Hoa5BinauralView, Hoa5Profile, HOA5_PUCK_HIT_RADIUS, HOA5_TOTAL_CHANNELS,
    };
    use crate::views::neural_dereverb_view::{
        DereverbModel, NeuralDereverbView, DEREVERB_PUCK_HIT_RADIUS,
    };
    use crate::views::steelpan_drum_view::{
        SteelpanDrumView, SteelpanType, STEELPAN_PUCK_HIT_RADIUS,
    };
    use crate::views::tape_flux_master_view::{
        TapeFluxMasterView, TapeFormulation, TAPE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1601_1606_steelpan_annular_ring_modes_and_hit_targets() {
        let mut steelpan = SteelpanDrumView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(STEELPAN_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(STEELPAN_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Strike Radial Position Conversion Roundtrip
        for r in [0.05, 0.25, 0.45, 0.70, 0.95] {
            let norm = SteelpanDrumView::radial_to_normalized(r);
            assert!((0.0..=1.0).contains(&norm));
            let back = SteelpanDrumView::normalized_to_radial(norm);
            assert!((back - r).abs() < 1e-4, "Radial pos mismatch at {}", r);
        }

        // 3. Strike Velocity Conversion Roundtrip
        for vel in [0.10, 0.35, 0.75, 0.90, 1.00] {
            let norm = SteelpanDrumView::vel_to_normalized(vel);
            assert!((0.0..=1.0).contains(&norm));
            let back = SteelpanDrumView::normalized_to_vel(norm);
            assert!((back - vel).abs() < 1e-4, "Velocity mismatch at {}", vel);
        }

        // 4. Steelpan Types and Nominal Values
        for ptype in [
            SteelpanType::LeadTenorPan,
            SteelpanType::DoubleSecondsPan,
            SteelpanType::DoubleGuitarPan,
            SteelpanType::TripleCellosPan,
            SteelpanType::SixBassPan,
        ] {
            steelpan.set_pan_type(ptype);
            let r = ptype.nominal_radial_pos();
            let v = ptype.nominal_strike_velocity();
            let rings = ptype.nominal_rings();
            let gauge = ptype.nominal_gauge_mm();
            let decay = ptype.nominal_damping_s();
            assert!((0.05..=0.95).contains(&r));
            assert!((0.10..=1.00).contains(&v));
            assert!((3..=6).contains(&rings));
            assert!((0.8..=2.0).contains(&gauge));
            assert!((0.5..=5.0).contains(&decay));
            assert!(!ptype.pan_name().is_empty());
        }

        // 5. Physics Simulation Verification
        steelpan.set_pan_type(SteelpanType::LeadTenorPan);
        steelpan.strike_radial_pos = 0.45;
        steelpan.strike_velocity = 0.75;
        steelpan.update_modal_simulation();
        assert!(steelpan.modal_amplitudes[0] > 0.3); // Fundamental
        assert!(steelpan.modal_amplitudes[1] > 0.3); // Octave
        assert!(steelpan.modal_amplitudes[5] > 0.1); // Inter-note coupling

        // 6. Hit Testing on Puck
        steelpan.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(steelpan.hit_test_steelpan_puck((center_x, center_y), canvas));
        assert!(!steelpan.hit_test_steelpan_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = steelpan.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1602_1607_auditory_roughness_sensory_dissonance_and_hit_targets() {
        let mut roughness = AuditoryRoughnessView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(ROUGHNESS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(ROUGHNESS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Center Frequency Conversion Roundtrip
        for f in [50.0, 250.0, 440.0, 1000.0, 5000.0] {
            let norm = AuditoryRoughnessView::freq_to_normalized(f);
            assert!((0.0..=1.0).contains(&norm));
            let back = AuditoryRoughnessView::normalized_to_freq(norm);
            assert!((back - f).abs() / f < 1e-3, "Freq mismatch at {}", f);
        }

        // 3. Interval Semitones Conversion Roundtrip
        for s in [0.0, 1.4, 3.5, 7.0, 12.0, 14.0] {
            let norm = AuditoryRoughnessView::interval_to_normalized(s);
            assert!((0.0..=1.0).contains(&norm));
            let back = AuditoryRoughnessView::normalized_to_interval(norm);
            assert!((back - s).abs() < 1e-4, "Interval mismatch at {}", s);
        }

        // 4. Standards and Nominal Values
        for std in [
            RoughnessStandard::PlompLevelt1965,
            RoughnessStandard::KameokaKuriyagawa1969,
            RoughnessStandard::FastlZwicker2007,
            RoughnessStandard::Vassilakis2001,
            RoughnessStandard::SetharesMicrotonal,
        ] {
            roughness.set_standard(std);
            let f = std.nominal_center_freq_hz();
            let semi = std.nominal_interval_semitones();
            let parts = std.nominal_partials();
            let mod_rate = std.nominal_mod_rate_hz();
            assert!((50.0..=5000.0).contains(&f));
            assert!((0.0..=14.0).contains(&semi));
            assert!((1..=32).contains(&parts));
            assert!((10.0..=150.0).contains(&mod_rate));
            assert!(!std.standard_name().is_empty());
        }

        // 5. Critical Bands Roughness Simulation
        roughness.set_standard(RoughnessStandard::PlompLevelt1965);
        roughness.center_freq_hz = 440.0;
        roughness.interval_semitones = 1.4;
        roughness.update_roughness_simulation();
        assert!(roughness.sensory_dissonance_index > 0.3);
        assert!(roughness.roughness_asper > 0.5);
        assert_eq!(roughness.critical_band_roughness.len(), 8);

        // 6. Hit Testing
        roughness.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(roughness.hit_test_roughness_puck((center_x, center_y), canvas));
        assert!(!roughness.hit_test_roughness_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = roughness.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1603_tape_flux_master_hysteresis_and_hit_targets() {
        let mut tape = TapeFluxMasterView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(TAPE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(TAPE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Flux Drive Roundtrip
        for d in [-6.0, 0.0, 6.0, 9.0, 18.0] {
            let norm = TapeFluxMasterView::drive_to_normalized(d);
            assert!((0.0..=1.0).contains(&norm));
            let back = TapeFluxMasterView::normalized_to_drive(norm);
            assert!((back - d).abs() < 1e-4, "Drive mismatch at {}", d);
        }

        // 3. Bias Trim Roundtrip
        for b in [-6.0, -2.0, 0.0, 3.0, 6.0] {
            let norm = TapeFluxMasterView::bias_to_normalized(b);
            assert!((0.0..=1.0).contains(&norm));
            let back = TapeFluxMasterView::normalized_to_bias(norm);
            assert!((back - b).abs() < 1e-4, "Bias mismatch at {}", b);
        }

        // 4. Formulations
        for form in [
            TapeFormulation::Ampex456GrandMaster,
            TapeFormulation::StuderA800MasterTape,
            TapeFormulation::QuantegyGP9,
            TapeFormulation::CassetteTypeIVMetal,
            TapeFormulation::VintageTubeTape1958,
        ] {
            tape.set_formulation(form);
            let d = form.nominal_flux_drive_db();
            let b = form.nominal_bias_trim_db();
            let ips = form.nominal_ips_speed();
            let bump = form.nominal_head_bump_hz();
            assert!((-6.0..=18.0).contains(&d));
            assert!((-6.0..=6.0).contains(&b));
            assert!((3.75..=30.0).contains(&ips));
            assert!((30.0..=120.0).contains(&bump));
            assert!(!form.formulation_name().is_empty());
        }

        // 5. Hysteresis Loop & Spectrum Simulation
        tape.set_formulation(TapeFormulation::StuderA800MasterTape);
        tape.flux_drive_db = 9.0;
        tape.bias_trim_db = 3.0;
        tape.update_hysteresis_simulation();
        assert_eq!(tape.hysteresis_loop_pts.len(), 16);
        assert_eq!(tape.harmonic_spectrum.len(), 6);
        assert!(tape.thd_distortion_pct > 0.0);

        // 6. Hit Testing
        tape.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(tape.hit_test_tape_puck((center_x, center_y), canvas));
        assert!(!tape.hit_test_tape_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = tape.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1604_neural_dereverb_room_impulse_and_hit_targets() {
        let mut dereverb = NeuralDereverbView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(DEREVERB_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DEREVERB_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Suppression Depth Roundtrip
        for depth in [0.0, 6.0, 18.0, 24.0, 36.0] {
            let norm = NeuralDereverbView::depth_to_normalized(depth);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralDereverbView::normalized_to_depth(norm);
            assert!((back - depth).abs() < 1e-4, "Depth mismatch at {}", depth);
        }

        // 3. Direct / Reverberant Ratio Roundtrip
        for drr in [-12.0, -6.0, 0.0, 6.0, 24.0] {
            let norm = NeuralDereverbView::drr_to_normalized(drr);
            assert!((0.0..=1.0).contains(&norm));
            let back = NeuralDereverbView::normalized_to_drr(norm);
            assert!((back - drr).abs() < 1e-4, "DRR mismatch at {}", drr);
        }

        // 4. Models
        for model in [
            DereverbModel::NeuralSpectralMaskUNet,
            DereverbModel::WeightedPredictionErrorWPE,
            DereverbModel::DiffusionDeconvolution,
            DereverbModel::CathedralAcousticHall,
            DereverbModel::ConferenceRoomAutomix,
        ] {
            dereverb.set_model(model);
            let s = model.nominal_suppression_db();
            let drr = model.nominal_drr_db();
            let rt = model.nominal_rt60_s();
            assert!((0.0..=36.0).contains(&s));
            assert!((-12.0..=24.0).contains(&drr));
            assert!((0.3..=6.0).contains(&rt));
            assert!(!model.model_name().is_empty());
        }

        // 5. Energy Decay & Spectral Mask Simulation
        dereverb.set_model(DereverbModel::NeuralSpectralMaskUNet);
        dereverb.suppression_depth_db = 18.0;
        dereverb.direct_to_reverberant_ratio_db = 6.0;
        dereverb.update_dereverb_simulation();
        assert_eq!(dereverb.energy_decay_curve.len(), 16);
        assert_eq!(dereverb.spectral_mask_bands.len(), 8);
        assert!(dereverb.energy_decay_curve[0] > dereverb.energy_decay_curve[15]);

        // 6. Hit Testing
        dereverb.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(dereverb.hit_test_dereverb_puck((center_x, center_y), canvas));
        assert!(!dereverb.hit_test_dereverb_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = dereverb.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }

    #[test]
    fn test_step_1605_hoa5_36_channel_ambisonics_and_hit_targets() {
        let mut hoa5 = Hoa5BinauralView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(HOA5_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(HOA5_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Roundtrip
        for az in [-180.0, -90.0, 0.0, 45.0, 180.0] {
            let norm = Hoa5BinauralView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa5BinauralView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        // 3. Elevation Roundtrip
        for el in [-90.0, -45.0, 0.0, 30.0, 90.0] {
            let norm = Hoa5BinauralView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa5BinauralView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4, "Elevation mismatch at {}", el);
        }

        // 4. Profiles
        for prof in [
            Hoa5Profile::Hoa5thOrderSpherical36Ch,
            Hoa5Profile::MaxReSphericalEnergy,
            Hoa5Profile::InPhaseOptimalDecoder,
            Hoa5Profile::BinauralKEMAR5thOrder,
            Hoa5Profile::DolbyAtmosBed36Virtual,
        ] {
            hoa5.set_profile(prof);
            let az = prof.nominal_azimuth_deg();
            let el = prof.nominal_elevation_deg();
            let dist = prof.nominal_distance_m();
            let focus = prof.nominal_energy_focus();
            assert!((-180.0..=180.0).contains(&az));
            assert!((-90.0..=90.0).contains(&el));
            assert!((0.5..=10.0).contains(&dist));
            assert!((0.0..=1.0).contains(&focus));
            assert!(!prof.profile_name().is_empty());
        }

        // 5. 36-Channel Spherical Harmonics Simulation
        hoa5.set_profile(Hoa5Profile::Hoa5thOrderSpherical36Ch);
        hoa5.azimuth_deg = 45.0;
        hoa5.elevation_deg = 20.0;
        hoa5.update_hoa5_simulation();
        assert_eq!(hoa5.spherical_harmonics_levels.len(), HOA5_TOTAL_CHANNELS);
        assert_eq!(HOA5_TOTAL_CHANNELS, 36); // (5+1)^2 = 36 channels
        assert!(hoa5.spherical_harmonics_levels[0] > 0.5); // Monopole W

        // 6. Hit Testing
        hoa5.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(hoa5.hit_test_hoa5_puck((center_x, center_y), canvas));
        assert!(!hoa5.hit_test_hoa5_puck((center_x + 100.0, center_y), canvas));

        // 7. ASCII Render
        let ascii = hoa5.render_ascii(60, 15);
        assert_eq!(ascii.len(), 15);
        assert_eq!(ascii[0].len(), 60);
    }
}
