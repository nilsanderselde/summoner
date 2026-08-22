// Summoner DAW - Tier 64 GUI Milestones Unit Test Suite (Steps 1511-1520)

#[cfg(test)]
mod tests {
    use crate::layout_math::Rect;
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::binaural_brir_view::{
        BinauralBrirView, RoomAcousticProfile, BRIR_SOURCE_PUCK_HIT_RADIUS,
    };
    use crate::views::bowed_string_view::{
        BowedStringView, StringMaterial, BOWED_STRING_PUCK_HIT_RADIUS,
    };
    use crate::views::dialog_gating_view::{
        DialogGatingView, DialogLoudnessStandard, DIALOG_GATING_PUCK_HIT_RADIUS,
    };
    use crate::views::granular_freeze_view::{
        GrainEnvelopeWindow, GranularFreezeView, GRANULAR_FREEZE_PUCK_HIT_RADIUS,
    };
    use crate::views::multiband_clipper_view::{
        ClipperCurveMode, MultibandClipperView, CLIPPER_KNEE_PUCK_HIT_RADIUS,
    };

    #[test]
    fn test_step_1511_1516_bowed_string_friction_and_hit_targets() {
        let mut bowed = BowedStringView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(BOWED_STRING_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(BOWED_STRING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Bow Speed Conversion Roundtrip
        for speed in [0.01, 0.15, 0.45, 1.00, 1.75, 2.00] {
            let norm = BowedStringView::speed_to_normalized(speed);
            assert!((0.0..=1.0).contains(&norm));
            let back = BowedStringView::normalized_to_speed(norm);
            assert!((back - speed).abs() < 1e-4, "Speed mismatch at {}", speed);
        }

        // 3. Bow Force Conversion Roundtrip
        for force in [0.05, 0.50, 1.25, 2.80, 4.20, 5.00] {
            let norm = BowedStringView::force_to_normalized(force);
            assert!((0.0..=1.0).contains(&norm));
            let back = BowedStringView::normalized_to_force(norm);
            assert!((back - force).abs() < 1e-4, "Force mismatch at {}", force);
        }

        // 4. Bridge Proximity Beta Conversion Roundtrip
        for beta in [0.02, 0.08, 0.12, 0.25, 0.40, 0.50] {
            let norm = BowedStringView::beta_to_normalized(beta);
            assert!((0.0..=1.0).contains(&norm));
            let back = BowedStringView::normalized_to_beta(norm);
            assert!((back - beta).abs() < 1e-4, "Beta mismatch at {}", beta);
        }

        // 5. Schelleng Limits and Stability
        let (f_min, f_max) = bowed.schelleng_limits();
        assert!(
            f_min < f_max,
            "Schelleng F_min ({}) must be < F_max ({})",
            f_min,
            f_max
        );
        assert!(bowed.helmholtz_stability_score > 0.0);

        // 6. String Material Selection
        for mat in [
            StringMaterial::SteelCore,
            StringMaterial::GutCore,
            StringMaterial::SyntheticCore,
            StringMaterial::NylonWound,
            StringMaterial::TungstenHeavy,
        ] {
            bowed.material = mat;
            let (mu_s, mu_d) = mat.friction_coefficients();
            assert!(mu_s > mu_d, "Static friction must exceed dynamic friction");
        }

        // 7. String Displacement Evaluation
        let disp_center = bowed.evaluate_string_displacement(0.5, 0.25);
        let disp_nut = bowed.evaluate_string_displacement(0.0, 0.25);
        assert_eq!(disp_nut, 0.0, "Displacement at nut boundary must be zero");
        assert!(disp_center.abs() > 0.0);

        // 8. Hit Testing on Bow Puck
        bowed.bow_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(bowed.hit_test_bow_puck((center_x, center_y), canvas));
        assert!(!bowed.hit_test_bow_puck((center_x + 100.0, center_y), canvas));

        // 9. Deterministic ASCII Render
        let ascii = bowed.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1512_binaural_brir_spatializer() {
        let mut brir = BinauralBrirView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(BRIR_SOURCE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(BRIR_SOURCE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Azimuth Conversion Roundtrip
        for az in [-180.0, -90.0, -45.0, 0.0, 45.0, 90.0, 180.0] {
            let norm = BinauralBrirView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = BinauralBrirView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4, "Azimuth mismatch at {}", az);
        }

        // 3. Distance Conversion Roundtrip
        for dist in [0.20, 1.00, 2.50, 5.00, 8.50, 10.00] {
            let norm = BinauralBrirView::distance_to_normalized(dist);
            assert!((0.0..=1.0).contains(&norm));
            let back = BinauralBrirView::normalized_to_distance(norm);
            assert!((back - dist).abs() < 1e-4, "Distance mismatch at {}", dist);
        }

        // 4. Woodworth ITD & ILD Acoustics
        brir.azimuth_deg = 90.0; // Source at 90 deg right
        brir.update_binaural_acoustics();
        assert!(
            brir.itd_microseconds > 500.0,
            "ITD at 90 deg must be > 500us (Woodworth)"
        );
        assert!(brir.ild_decibels > 10.0, "ILD at 90 deg must be > 10dB");

        // 5. Room Acoustic Profiles
        for prof in [
            RoomAcousticProfile::ConcertHall,
            RoomAcousticProfile::ScoringStage,
            RoomAcousticProfile::CathedralSpace,
            RoomAcousticProfile::DryStudio,
            RoomAcousticProfile::IntimateChamber,
        ] {
            brir.room_profile = prof;
            assert!(prof.rt60_seconds() > 0.0);
            assert!(prof.early_reflection_delay_ms() > 0.0);
        }

        // 6. Early Reflection Decay Sequence
        let (d0, a0) = brir.evaluate_early_reflection(0);
        let (d5, a5) = brir.evaluate_early_reflection(5);
        assert!(d0 < d5, "Initial reflection must precede late reflections");
        assert!(
            a0 > a5,
            "Initial reflection amplitude must exceed late reflections"
        );

        // 7. Hit Testing on Source Puck
        brir.source_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(brir.hit_test_source_puck((center_x, center_y), canvas));
        assert!(!brir.hit_test_source_puck((center_x + 100.0, center_y), canvas));

        // 8. Deterministic ASCII Render
        let ascii = brir.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1513_1517_multiband_clipper_transfer_functions_and_hit_targets() {
        let mut clipper = MultibandClipperView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(CLIPPER_KNEE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(CLIPPER_KNEE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Threshold Conversion Roundtrip
        for thresh in [-24.0, -18.0, -12.0, -6.0, -3.0, 0.0] {
            let norm = MultibandClipperView::threshold_to_normalized(thresh);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandClipperView::normalized_to_threshold(norm);
            assert!(
                (back - thresh).abs() < 1e-4,
                "Threshold mismatch at {}",
                thresh
            );
        }

        // 3. Ceiling Conversion Roundtrip
        for ceil in [-12.0, -8.0, -5.0, -2.0, -0.5, 0.0] {
            let norm = MultibandClipperView::ceiling_to_normalized(ceil);
            assert!((0.0..=1.0).contains(&norm));
            let back = MultibandClipperView::normalized_to_ceiling(norm);
            assert!((back - ceil).abs() < 1e-4, "Ceiling mismatch at {}", ceil);
        }

        // 4. Non-Linear Transfer Curve Modes
        for mode in [
            ClipperCurveMode::SoftKneeCubic,
            ClipperCurveMode::HyperbolicTanh,
            ClipperCurveMode::HardBrickwall,
            ClipperCurveMode::QuinticSmooth,
            ClipperCurveMode::AsymmetricTube,
        ] {
            clipper.curve_mode = mode;
            let out_pass = clipper.evaluate_transfer_curve(-20.0);
            let out_clip = clipper.evaluate_transfer_curve(0.0);
            assert_eq!(out_pass, -20.0, "Linear passband must preserve input level");
            assert!(out_clip <= 0.0, "Clipped output must not exceed ceiling");
        }

        // 5. 4-Band Crossover Verification
        assert_eq!(clipper.bands.len(), 4);
        assert!(clipper.bands[0].crossover_high_hz < clipper.bands[1].crossover_high_hz);

        // 6. Hit Testing on Knee Puck
        clipper.knee_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(clipper.hit_test_knee_puck((center_x, center_y), canvas));
        assert!(!clipper.hit_test_knee_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = clipper.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1514_granular_spectral_cloud_freeze() {
        let mut granular = GranularFreezeView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(GRANULAR_FREEZE_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(GRANULAR_FREEZE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Grain Size Conversion Roundtrip
        for size in [10.0, 50.0, 120.0, 250.0, 400.0, 500.0] {
            let norm = GranularFreezeView::size_to_normalized(size);
            assert!((0.0..=1.0).contains(&norm));
            let back = GranularFreezeView::normalized_to_size(norm);
            assert!((back - size).abs() < 1e-4, "Size mismatch at {}", size);
        }

        // 3. Pitch Spray Conversion Roundtrip
        for spray in [-24.0, -12.0, -5.0, 0.0, 7.0, 18.0, 24.0] {
            let norm = GranularFreezeView::pitch_to_normalized(spray);
            assert!((0.0..=1.0).contains(&norm));
            let back = GranularFreezeView::normalized_to_pitch(norm);
            assert!((back - spray).abs() < 1e-4, "Pitch mismatch at {}", spray);
        }

        // 4. Window Envelope Profiles
        for win in [
            GrainEnvelopeWindow::HannSmooth,
            GrainEnvelopeWindow::BlackmanHarris,
            GrainEnvelopeWindow::GaussianBell,
            GrainEnvelopeWindow::TukeyTapered,
            GrainEnvelopeWindow::TrapezoidSharp,
        ] {
            granular.window_type = win;
            let center_val = granular.evaluate_window_envelope(0.5);
            let edge_val = granular.evaluate_window_envelope(0.0);
            assert!(
                center_val > 0.8,
                "Center of window envelope must be near 1.0"
            );
            assert!(edge_val <= 0.1, "Edge of window envelope must be near 0.0");
        }

        // 5. Active Grain Particles
        for i in 0..8 {
            let p = granular.evaluate_grain_particle(i);
            assert!((0.0..=1.0).contains(&p.pos_norm));
            assert!((0.0..=1.0).contains(&p.age_norm));
        }

        // 6. Hit Testing on Freeze Puck
        granular.cloud_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(granular.hit_test_freeze_puck((center_x, center_y), canvas));
        assert!(!granular.hit_test_freeze_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = granular.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }

    #[test]
    fn test_step_1515_dialog_gating_and_speech_loudness() {
        let mut gating = DialogGatingView::new();
        let canvas = Rect::new(20.0, 104.0, 760.0, 236.0);

        // 1. Hit Target Enforcement
        const { assert!(DIALOG_GATING_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(DIALOG_GATING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. LUFS Conversion Roundtrip
        for lufs in [-40.0, -32.0, -27.0, -23.0, -16.0, -14.0, -10.0] {
            let norm = DialogGatingView::lufs_to_normalized(lufs);
            assert!((0.0..=1.0).contains(&norm));
            let back = DialogGatingView::normalized_to_lufs(norm);
            assert!((back - lufs).abs() < 1e-4, "LUFS mismatch at {}", lufs);
        }

        // 3. Standards Definitions
        for std in [
            DialogLoudnessStandard::EbuR128,
            DialogLoudnessStandard::AtscA85,
            DialogLoudnessStandard::NetflixOtt,
            DialogLoudnessStandard::StreamingMusic,
            DialogLoudnessStandard::PodcastSpeech,
        ] {
            gating.standard = std;
            assert!(std.target_integrated_lufs() <= -10.0);
            assert!(std.true_peak_ceiling_dbtp() <= 0.0);
        }

        // 4. ITU-R BS.1770-4 K-Weighting Filter Response
        let resp_100hz = gating.evaluate_k_weighting_response_db(100.0);
        let resp_5khz = gating.evaluate_k_weighting_response_db(5000.0);
        assert!(
            resp_5khz > resp_100hz,
            "K-weighting high-shelving must boost high frequencies"
        );

        // 5. Gating Calculations
        gating.update_gating_calculations();
        assert!(gating.gating_delta_lu >= 0.0);

        // 6. Hit Testing on Dialog Puck
        gating.dialog_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(gating.hit_test_dialog_puck((center_x, center_y), canvas));
        assert!(!gating.hit_test_dialog_puck((center_x + 100.0, center_y), canvas));

        // 7. Deterministic ASCII Render
        let ascii = gating.render_ascii(40, 10);
        assert_eq!(ascii.len(), 10);
    }
}
