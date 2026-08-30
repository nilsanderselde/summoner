// Summoner DAW - Tier 79 GUI Milestones Unit Test Suite (Koto & Ji Bridge HUDs)

#[cfg(test)]
mod tests {
    use crate::touch_controls::MIN_HIT_TARGET_PT;
    use crate::views::ji_bridge_view::{
        JiBridgeView, JiBridgeViewProfile, JI_HANDLE_HIT_RADIUS, NUM_JI_BRIDGES,
    };
    use crate::views::koto_view::{
        KotoHudPreset, KotoView, KOTO_PUCK_HIT_RADIUS, MAX_OSHI_ITE_FORCE_N, MIN_OSHI_ITE_FORCE_N,
    };

    #[test]
    fn test_koto_view_hit_targets_and_presets() {
        let mut view = KotoView::new();

        // 1. Minimum Hit Target Enforcement (Radius >= 22pt -> 44x44pt touch bounding box)
        const { assert!(KOTO_PUCK_HIT_RADIUS >= 22.0) };
        const { assert!(KOTO_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Presets and acoustic parameter updates
        for preset in [
            KotoHudPreset::HirajoshiTraditionalSilk,
            KotoHudPreset::KokinJoshiUrbanClassical,
            KotoHudPreset::InSenContemporaryDramatic,
            KotoHudPreset::KumoiJoshiSpringRain,
            KotoHudPreset::RyukyuFestiveOkinawa,
            KotoHudPreset::ConcertGuzhengVirtuoso,
        ] {
            view.set_preset(preset);
            assert!(!preset.name().is_empty());
            assert!(!view.active_tuning_schema_name.is_empty());
            assert!(view.behind_bridge_bleed > 0.0);
            assert!(view.body_wood_resonance > 0.0);
        }

        // 3. Oshi-Ite pitch cents deflection calculation
        view.oshi_ite_force_n = MIN_OSHI_ITE_FORCE_N;
        assert_eq!(view.current_pitch_cents(), 0.0);

        view.oshi_ite_force_n = MAX_OSHI_ITE_FORCE_N;
        let max_cents = view.current_pitch_cents();
        assert!((max_cents - 400.0).abs() < 1.0, "Expected ~400 cents, got {}", max_cents);

        // 4. ASCII and snapshot rendering
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);

        let snap_res = view.render_snapshot_png("scratch/renders/koto_view.png", 800, 520);
        assert!(snap_res.is_ok());
    }

    #[test]
    fn test_ji_bridge_view_hit_targets_and_profiles() {
        let mut view = JiBridgeView::new();

        // 1. Minimum Hit Target Enforcement
        const { assert!(JI_HANDLE_HIT_RADIUS >= 22.0) };
        const { assert!(JI_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT) };

        // 2. Bridge count and position bounds
        assert_eq!(view.bridge_positions.len(), NUM_JI_BRIDGES);
        for &pos in &view.bridge_positions {
            assert!((0.15..=0.90).contains(&pos), "Bridge position {} out of range", pos);
        }

        // 3. Material profiles
        for profile in [
            JiBridgeViewProfile::PaulowniaHardwood,
            JiBridgeViewProfile::IvoryBone,
            JiBridgeViewProfile::RosewoodGuzheng,
            JiBridgeViewProfile::SyntheticDelrin,
            JiBridgeViewProfile::SmokedBamboo,
        ] {
            view.set_profile(profile);
            let (trans, refl) = profile.transmission_reflection();
            assert!(trans > 0.0 && trans < 0.25);
            assert!(refl > 0.90 && refl <= 1.0);
            assert!(!profile.name().is_empty());
        }

        // 4. ASCII and snapshot rendering
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);

        let snap_res = view.render_snapshot_png("scratch/renders/ji_bridge_view.png", 800, 520);
        assert!(snap_res.is_ok());
    }
}
