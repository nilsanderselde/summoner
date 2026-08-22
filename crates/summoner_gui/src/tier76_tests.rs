// Summoner DAW - Tier 76 GUI Milestones Unit Test Suite (Steps 1631-1640)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::midside_focuser_view::{
        MidSideFocuserView, MidSideProfileType, MIDSIDE_FOCUSER_PUCK_HIT_RADIUS,
    };
    use crate::views::nhk222_immersion_view::{
        Nhk222ImmersionView, Nhk222ProfileType, NHK222_PUCK_HIT_RADIUS,
    };
    use crate::views::shakuhachi_view::{
        ShakuhachiFingeringNote, ShakuhachiLengthType, ShakuhachiView, SHAKUHACHI_PUCK_HIT_RADIUS,
    };
    use crate::views::spectral_flatness_view::{
        SpectralFlatnessView, SpectralProfileType, SPECTRAL_FLATNESS_PUCK_HIT_RADIUS,
    };
    use crate::views::wavefront_reflection_view::{
        RoomArchitectureType, WavefrontReflectionView, WAVEFRONT_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1631_1636_shakuhachi_bamboo_flute_modes_and_hit_targets() {
        let mut flute = ShakuhachiView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(SHAKUHACHI_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SHAKUHACHI_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Airjet Velocity Conversion Roundtrip
        for v in [4.0, 11.0, 14.5, 16.5, 19.0, 24.0, 42.0] {
            let norm = ShakuhachiView::velocity_to_normalized(v);
            assert!((0.0..=1.0).contains(&norm));
            let back = ShakuhachiView::normalized_to_velocity(norm);
            assert!((back - v).abs() < 1e-4, "Velocity mismatch at {}", v);
        }

        // 3. Utaguchi Angle Conversion Roundtrip
        for angle in [10.0, 34.0, 38.0, 40.0, 44.0, 48.0, 60.0] {
            let norm = ShakuhachiView::angle_to_normalized(angle);
            assert!((0.0..=1.0).contains(&norm));
            let back = ShakuhachiView::normalized_to_angle(norm);
            assert!((back - angle).abs() < 1e-4, "Angle mismatch at {}", angle);
        }

        // 4. Length Types and Nominal Values
        for ltype in [
            ShakuhachiLengthType::IchishakuHassun,
            ShakuhachiLengthType::NishakuYonsun,
            ShakuhachiLengthType::IchishakuRokusun,
            ShakuhachiLengthType::NishakuIssun,
            ShakuhachiLengthType::SanShakuKyotaku,
        ] {
            flute.set_length_type(ltype);
            let v = ltype.nominal_jet_velocity_mps();
            let ang = ltype.nominal_utaguchi_angle_deg();
            let len = ltype.nominal_length_cm();
            let chiff = ltype.nominal_chiff_noise();
            let q = ltype.nominal_bore_q();

            assert!((4.0..=42.0).contains(&v));
            assert!((10.0..=60.0).contains(&ang));
            assert!((45.0..=95.0).contains(&len));
            assert!((0.10..=0.60).contains(&chiff));
            assert!((30.0..=70.0).contains(&q));
            assert!(!ltype.flute_name().is_empty());
        }

        // 5. Fingering Notes & Open Hole Ratios
        for fn_note in [
            ShakuhachiFingeringNote::Ro,
            ShakuhachiFingeringNote::Tsu,
            ShakuhachiFingeringNote::Re,
            ShakuhachiFingeringNote::Chi,
            ShakuhachiFingeringNote::Ri,
        ] {
            flute.set_fingering(fn_note);
            let ratio = fn_note.open_hole_ratio();
            assert!((0.0..=1.0).contains(&ratio));
            assert!(!fn_note.note_name().is_empty());
        }

        // 6. Physics Simulation Verification
        flute.set_length_type(ShakuhachiLengthType::IchishakuHassun);
        flute.jet_velocity_mps = 19.0;
        flute.utaguchi_angle_deg = 38.0;
        flute.meri_kari_cents = -50.0;
        flute.update_shakuhachi_simulation();
        assert!(flute.modal_amplitudes[0] > 0.4); // Fundamental Ro f0
        assert!(flute.modal_amplitudes[4] > 0.1); // Blowing edge chiff noise
        assert!(flute.modal_amplitudes[7] > 0.3); // Meri-kari inflection

        // 7. Hit Testing on Puck
        flute.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(flute.hit_test_shakuhachi_puck((center_x, center_y), canvas));
        assert!(!flute.hit_test_shakuhachi_puck((center_x + 100.0, center_y), canvas));

        // 8. ASCII Fallback Rendering
        let ascii = flute.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('S')));
    }

    #[test]
    fn test_step_1632_1637_spectral_flatness_tonality_and_hit_targets() {
        let mut spec = SpectralFlatnessView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(SPECTRAL_FLATNESS_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(SPECTRAL_FLATNESS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. HNR dB Conversion Roundtrip
        for hnr in [-20.0, -6.0, 0.0, 4.0, 15.0, 18.0, 32.0, 40.0] {
            let norm = SpectralFlatnessView::hnr_to_normalized(hnr);
            assert!((0.0..=1.0).contains(&norm));
            let back = SpectralFlatnessView::normalized_to_hnr(norm);
            assert!((back - hnr).abs() < 1e-4, "HNR mismatch at {}", hnr);
        }

        // 3. Profiles and Nominal Values
        for prof in [
            SpectralProfileType::PureToneLead,
            SpectralProfileType::HarmonicPolyChoir,
            SpectralProfileType::PercussiveTransient,
            SpectralProfileType::AmbientAtmosphere,
            SpectralProfileType::MasteringFullMix,
        ] {
            spec.set_profile(prof);
            let sfm = prof.nominal_sfm();
            let hnr = prof.nominal_hnr_db();
            let entropy = prof.nominal_wiener_entropy();
            let tonality = prof.nominal_tonality_factor();

            assert!((0.0..=1.0).contains(&sfm));
            assert!((-20.0..=40.0).contains(&hnr));
            assert!((0.0..=1.0).contains(&entropy));
            assert!((0.0..=1.0).contains(&tonality));
            assert!(!prof.profile_name().is_empty());
        }

        // 4. Subband Flatness Curves
        spec.set_profile(SpectralProfileType::PureToneLead);
        assert!(spec.tonality_factor > 0.8);
        assert!(spec.spectral_flatness_measure < 0.15);
        assert!(spec.bark_band_flatness[0] < spec.bark_band_flatness[7]);

        spec.set_profile(SpectralProfileType::AmbientAtmosphere);
        assert!(spec.tonality_factor < 0.2);
        assert!(spec.spectral_flatness_measure > 0.85);

        // 5. Hit Testing on Puck
        spec.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(spec.hit_test_flatness_puck((center_x, center_y), canvas));
        assert!(!spec.hit_test_flatness_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Fallback Rendering
        let ascii = spec.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('F')));
    }

    #[test]
    fn test_step_1633_midside_focuser_dynamic_punch_and_hit_targets() {
        let mut ms = MidSideFocuserView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(MIDSIDE_FOCUSER_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(MIDSIDE_FOCUSER_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Cutoff Hz and Punch dB Roundtrips
        for hz in [20.0, 80.0, 100.0, 120.0, 140.0, 150.0, 300.0] {
            let norm = MidSideFocuserView::cutoff_to_normalized(hz);
            assert!((0.0..=1.0).contains(&norm));
            let back = MidSideFocuserView::normalized_to_cutoff(norm);
            assert!((back - hz).abs() < 1e-4, "Cutoff mismatch at {}", hz);
        }

        for db in [0.0, 1.5, 2.0, 2.5, 4.0, 6.5, 12.0] {
            let norm = MidSideFocuserView::punch_to_normalized(db);
            assert!((0.0..=1.0).contains(&norm));
            let back = MidSideFocuserView::normalized_to_punch(norm);
            assert!((back - db).abs() < 1e-4, "Punch mismatch at {}", db);
        }

        // 3. Profiles and Nominal Values
        for prof in [
            MidSideProfileType::ClubSubTightener,
            MidSideProfileType::VinylMasterMonofier,
            MidSideProfileType::AcousticLiveFocus,
            MidSideProfileType::DynamicEdmSlam,
            MidSideProfileType::BroadcastClaritySafe,
        ] {
            ms.set_profile(prof);
            let hz = prof.nominal_mono_cutoff_hz();
            let punch = prof.nominal_mid_punch_db();
            let width = prof.nominal_side_width();
            let taps = prof.nominal_fir_taps();

            assert!((20.0..=300.0).contains(&hz));
            assert!((0.0..=12.0).contains(&punch));
            assert!((0.0..=2.0).contains(&width));
            assert!(taps >= 128);
            assert!(!prof.profile_name().is_empty());
        }

        // 4. Stereo Correlation & Band Energies
        ms.set_profile(MidSideProfileType::ClubSubTightener);
        assert!(ms.stereo_correlation > 0.5);
        assert!(ms.band_energies[0] > 0.8); // Sub-mono energy

        // 5. Hit Testing on Puck
        ms.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(ms.hit_test_focuser_puck((center_x, center_y), canvas));
        assert!(!ms.hit_test_focuser_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Fallback Rendering
        let ascii = ms.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('M')));
    }

    #[test]
    fn test_step_1634_wavefront_reflection_raytracing_and_hit_targets() {
        let mut wf = WavefrontReflectionView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(WAVEFRONT_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(WAVEFRONT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Architectures and Nominal Values
        for arch in [
            RoomArchitectureType::ConcertHallHorseshoe,
            RoomArchitectureType::ShoeboxStudioControl,
            RoomArchitectureType::CathedralStoneVault,
            RoomArchitectureType::ModularVocalBooth,
            RoomArchitectureType::AsymmetricalGallery,
        ] {
            wf.set_architecture(arch);
            let s = arch.nominal_scattering();
            let a = arch.nominal_absorption();
            let r = arch.nominal_ray_count();
            let win = arch.nominal_early_window_ms();

            assert!((0.0..=1.0).contains(&s));
            assert!((0.0..=1.0).contains(&a));
            assert!((100..=5000).contains(&r));
            assert!((10.0..=150.0).contains(&win));
            assert!(!arch.architecture_name().is_empty());
        }

        // 3. Early Reflection Decay Taps
        wf.set_architecture(RoomArchitectureType::ConcertHallHorseshoe);
        assert!(wf.reflection_energy_taps[0] > wf.reflection_energy_taps[15]);
        assert!(wf.reflection_energy_taps[0] > 0.5);

        // 4. Hit Testing on Puck
        wf.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(wf.hit_test_wavefront_puck((center_x, center_y), canvas));
        assert!(!wf.hit_test_wavefront_puck((center_x + 100.0, center_y), canvas));

        // 5. ASCII Fallback Rendering
        let ascii = wf.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('W')));
    }

    #[test]
    fn test_step_1635_nhk222_3layer_immersion_and_hit_targets() {
        let mut nhk = Nhk222ImmersionView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement
        const { assert!(NHK222_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(NHK222_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth & Elevation Roundtrips
        for az in [-180.0, -90.0, -45.0, -30.0, 0.0, 45.0, 60.0, 180.0] {
            let norm = Nhk222ImmersionView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222ImmersionView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        for el in [-90.0, -45.0, 0.0, 15.0, 20.0, 35.0, 45.0, 75.0, 90.0] {
            let norm = Nhk222ImmersionView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222ImmersionView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4, "Elevation mismatch at {}", el);
        }

        // 3. Profiles and Nominal Values
        for prof in [
            Nhk222ProfileType::OrchestralSymphony222,
            Nhk222ProfileType::StadiumSportsLive222,
            Nhk222ProfileType::TheatricalFilmEpic,
            Nhk222ProfileType::DomePlanetarium,
            Nhk222ProfileType::Binaural222Headphones,
        ] {
            nhk.set_profile(prof);
            let az = prof.nominal_azimuth_deg();
            let el = prof.nominal_elevation_deg();
            let dist = prof.nominal_distance_m();

            assert!((-180.0..=180.0).contains(&az));
            assert!((-90.0..=90.0).contains(&el));
            assert!((0.5..=20.0).contains(&dist));
            assert!(!prof.profile_name().is_empty());
        }

        // 4. 24 Speaker Energies & Layer Balance
        nhk.set_profile(Nhk222ProfileType::DomePlanetarium);
        assert!(nhk.top_layer_energy > nhk.bottom_layer_energy);
        assert_eq!(nhk.speaker_energies.len(), 24);

        // 5. Hit Testing on Puck
        nhk.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(nhk.hit_test_nhk222_puck((center_x, center_y), canvas));
        assert!(!nhk.hit_test_nhk222_puck((center_x + 100.0, center_y), canvas));

        // 6. ASCII Fallback Rendering
        let ascii = nhk.render_ascii(40, 15);
        assert_eq!(ascii.len(), 15);
        assert!(ascii[0].starts_with('+'));
        assert!(ascii.iter().any(|line| line.contains('N')));
    }
}
