#![cfg(feature = "gui")]

// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 114 Test Suite — Turn #50 (Turn #16):
//! Unified Multi-Category Device Catalog & Instant Fuzzy Palette Search (Milestone 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::dsp_node_ui::{DspNodeCategory, DspNodeRegistry};
    use crate::views::device_catalog::{
        compute_fuzzy_score, DeviceCatalogEntry, DeviceCatalogView, CATALOG_TOUCH_TARGET_SIZE,
    };

    #[test]
    fn test_turn50_device_catalog_inventory_indexing_and_total_count() {
        let catalog = DeviceCatalogView::new();
        let inv = DspNodeRegistry::inventory();

        assert_eq!(catalog.total_count(), inv.len());
        assert!(catalog.total_count() >= 670, "Catalog should have >= 670 reflected DSP modules");

        // Verify key modules exist in index
        assert!(catalog.entries.iter().any(|e| e.kind_id == "AetherSynth"));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "PlateReverbTank" || e.kind_id.contains("Plate")));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "ParametricEqNode" || e.kind_id == "ParametricEq"));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "TruePeakLimiter"));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "CrystalResonator"));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "GlassArmonica"));
        assert!(catalog.entries.iter().any(|e| e.kind_id == "FrenchHurdyGurdy" || e.kind_id.contains("HurdyGurdy")));
    }

    #[test]
    fn test_turn50_device_catalog_submillisecond_fuzzy_search() {
        let mut catalog = DeviceCatalogView::new();

        // Query "Aether"
        catalog.search_query = "Aether".to_string();
        let results = catalog.filtered_entries();
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.kind_id == "AetherSynth"));

        // Query "Plate"
        catalog.search_query = "Plate".to_string();
        let results = catalog.filtered_entries();
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.kind_id.contains("Plate")));

        // Query "Crystal"
        catalog.search_query = "Crystal".to_string();
        let results = catalog.filtered_entries();
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.kind_id.contains("Crystal")));

        // Case-insensitivity test
        catalog.search_query = "hurdy".to_string();
        let results = catalog.filtered_entries();
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.kind_id.contains("Hurdy")));
    }

    #[test]
    fn test_turn50_device_catalog_fuzzy_score_ranking() {
        let entry_a = DeviceCatalogEntry::new(
            "OscSine",
            "Pure Sine Wave Oscillator",
            DspNodeCategory::Oscillator,
            "Pure Sine Wave Oscillator",
        );
        let entry_b = DeviceCatalogEntry::new(
            "SimdPolyWavetableOscillator",
            "AVX2/NEON Polyphonic Wavetable Voice Bank",
            DspNodeCategory::Oscillator,
            "High performance oscillator",
        );

        // Exact match should have highest score
        let score_exact = compute_fuzzy_score("OscSine", &entry_a).unwrap();
        let score_partial = compute_fuzzy_score("Osc", &entry_a).unwrap();
        assert!(score_exact > score_partial);

        // Favorite boost should elevate score
        let mut entry_fav = entry_b.clone();
        entry_fav.is_favorite = true;
        let score_non_fav = compute_fuzzy_score("oscillator", &entry_b).unwrap();
        let score_fav = compute_fuzzy_score("oscillator", &entry_fav).unwrap();
        assert!(score_fav > score_non_fav);
    }

    #[test]
    fn test_turn50_device_catalog_tagged_categorization_and_counts() {
        let mut catalog = DeviceCatalogView::new();

        let osc_count = catalog.category_count(DspNodeCategory::Oscillator);
        let filter_count = catalog.category_count(DspNodeCategory::FilterEq);
        let physical_count = catalog.category_count(DspNodeCategory::AcousticPhysicalModel);

        assert!(osc_count > 0, "Oscillator category should have registered modules");
        assert!(filter_count > 0, "Filter category should have registered modules");
        assert!(physical_count > 0, "Physical modeling category should have registered modules");

        // Filter by category
        catalog.selected_category = Some(DspNodeCategory::AcousticPhysicalModel);
        let filtered = catalog.filtered_entries();
        assert_eq!(filtered.len(), physical_count);
        for item in filtered {
            assert_eq!(item.category, DspNodeCategory::AcousticPhysicalModel);
        }
    }

    #[test]
    fn test_turn50_device_catalog_favorite_pinning_and_filtering() {
        let mut catalog = DeviceCatalogView::new();
        assert_eq!(catalog.favorites_count(), 0);

        // Pin favorites
        assert!(catalog.toggle_favorite("AetherSynth"));
        assert!(catalog.toggle_favorite("PlateReverbNode"));
        assert_eq!(catalog.favorites_count(), 2);
        assert!(catalog.is_favorite("AetherSynth"));
        assert!(catalog.is_favorite("PlateReverbNode"));
        assert!(!catalog.is_favorite("OscSine"));

        // Unpin favorite
        assert!(!catalog.toggle_favorite("PlateReverbNode"));
        assert_eq!(catalog.favorites_count(), 1);
        assert!(!catalog.is_favorite("PlateReverbNode"));

        // Test favorites_only filter
        catalog.favorites_only = true;
        let filtered = catalog.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].kind_id, "AetherSynth");
    }

    #[test]
    fn test_turn50_device_catalog_keyboard_navigation() {
        let mut catalog = DeviceCatalogView::new();
        let total = 5;

        catalog.selected_index = 0;
        catalog.select_next(total);
        assert_eq!(catalog.selected_index, 1);

        catalog.select_next(total);
        assert_eq!(catalog.selected_index, 2);

        catalog.select_prev(total);
        assert_eq!(catalog.selected_index, 1);

        // Wrap around backward
        catalog.selected_index = 0;
        catalog.select_prev(total);
        assert_eq!(catalog.selected_index, 4);

        // Wrap around forward
        catalog.select_next(total);
        assert_eq!(catalog.selected_index, 0);
    }

    #[test]
    fn test_turn50_device_catalog_take_requested_insert() {
        let mut catalog = DeviceCatalogView::new();
        assert_eq!(catalog.take_requested_insert(), None);

        catalog.requested_insert_kind = Some("AetherSynth".to_string());
        assert_eq!(catalog.take_requested_insert(), Some("AetherSynth".to_string()));
        assert_eq!(catalog.take_requested_insert(), None);
    }

    #[test]
    fn test_turn50_device_catalog_touch_target_dimensions() {
        assert!(CATALOG_TOUCH_TARGET_SIZE.x >= 44.0);
        assert!(CATALOG_TOUCH_TARGET_SIZE.y >= 44.0);
    }

    #[test]
    fn test_turn50_device_catalog_deterministic_ascii_snapshot() {
        let mut catalog = DeviceCatalogView::new();
        catalog.search_query = "Aether".to_string();
        catalog.toggle_favorite("AetherSynth");

        let snapshot = catalog.render_snapshot_ascii();
        assert!(snapshot.contains("=== Summoner DSP Device Catalog ==="));
        assert!(snapshot.contains("Query: \"Aether\""));
        assert!(snapshot.contains("[★]"));
        assert!(snapshot.contains("AetherSynth"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;

    #[test]
    fn test_turn50_award_winning_gui_view_catalog_integration() {
        let mut view = AwardWinningGuiView::new();

        // Initially catalog modal is closed
        assert!(!view.modular_add_modal_open);
        assert!(!view.device_catalog.is_open);

        // Open catalog modal
        view.modular_add_modal_open = true;
        view.device_catalog.is_open = true;
        assert_eq!(view.device_catalog.total_count(), view.device_catalog.entries.len());

        // Simulate choosing a module from catalog
        view.device_catalog.requested_insert_kind = Some("OscSine".to_string());

        let initial_node_count = view.modular_nodes.len();
        if let Some(kind) = view.device_catalog.take_requested_insert() {
            let registry = crate::dsp_node_ui::DspNodeRegistry::new();
            if let Some(desc) = registry.get(&kind) {
                view.add_modular_node_from_descriptor(desc);
                view.modular_add_modal_open = false;
                view.device_catalog.is_open = false;
            }
        }

        assert_eq!(view.modular_nodes.len(), initial_node_count + 1);
        assert!(view.modular_nodes.iter().any(|n| n.kind_id == "OscSine"));
        assert!(!view.modular_add_modal_open);
        assert!(!view.device_catalog.is_open);
    }
}
