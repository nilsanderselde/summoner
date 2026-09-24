// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #6 Integration & Regression Tests:
//! 1. Curated Factory Preset & Soundbank Catalog Completeness (47+ Presets).
//! 2. General MIDI Soundbank Alignment (WISHLIST.md).
//! 3. Chiptune & Tracker Preset System Alignment (WISHLIST.md).
//! 4. 5 Full-Scale Multi-Track Factory Demo Templates (Physical, 19-EDO, Bohlen-Pierce, Chiptune, GM).
//! 5. GUI Top Bar & Asset Browser Integration with One-Click Preset & Template Loading.
//! 6. Zero-Allocation Real-Time Audio Streaming Convergence with ParamBus under AllocGuard.

#[cfg(test)]
pub mod pure_tests {
    use crate::dsp_node_ui::DspNodeRegistry;
    use crate::factory_presets::{
        all_demo_templates, all_factory_presets, find_demo_template, find_preset,
        presets_by_category, FactoryPresetCategory,
    };

    #[test]
    fn test_factory_presets_inventory_and_completeness() {
        let registry = DspNodeRegistry::new();
        let presets = all_factory_presets();
        assert!(
            presets.len() >= 45,
            "Factory presets catalog must contain >= 45 curated presets, found {}",
            presets.len()
        );

        for p in presets {
            assert!(!p.id.is_empty(), "Preset ID must not be empty");
            assert!(!p.name.is_empty(), "Preset Name must not be empty for {}", p.id);
            assert!(!p.description.is_empty(), "Description must not be empty for {}", p.id);
            assert!(!p.tags.is_empty(), "Tags must not be empty for {}", p.id);
            assert!(
                p.macro_tone >= 0.0 && p.macro_tone <= 1.0,
                "Macro tone must be in 0..1 for {}",
                p.id
            );
            assert!(
                p.macro_space >= 0.0 && p.macro_space <= 1.0,
                "Macro space must be in 0..1 for {}",
                p.id
            );
            assert!(
                p.macro_punch >= 0.0 && p.macro_punch <= 1.0,
                "Macro punch must be in 0..1 for {}",
                p.id
            );
            assert!(
                p.macro_character >= 0.0 && p.macro_character <= 1.0,
                "Macro character must be in 0..1 for {}",
                p.id
            );
            assert!(
                p.params.len() >= 5,
                "Preset {} must configure >= 5 parameters, found {}",
                p.id,
                p.params.len()
            );

            // Verify node kind exists in DspNodeRegistry
            assert!(
                registry.get(p.target_node_kind).is_some(),
                "Target DSP node {} for preset {} must exist in DspNodeRegistry",
                p.target_node_kind,
                p.id
            );
        }
    }

    #[test]
    fn test_general_midi_soundbank_coverage() {
        let gm_presets = presets_by_category(FactoryPresetCategory::GeneralMidi);
        assert_eq!(
            gm_presets.len(),
            10,
            "Must have 10 General MIDI flagship soundbank presets"
        );

        let expected_gm = [
            ("gm_grand_piano", "General MIDI Acoustic Grand Piano"),
            ("gm_electric_piano", "General MIDI Electric Piano 1 (Rhodes)"),
            ("gm_drawbar_organ", "General MIDI Drawbar Organ"),
            ("gm_church_organ", "General MIDI Church Pipe Organ"),
            ("gm_nylon_guitar", "General MIDI Nylon String Guitar"),
            ("gm_rock_guitar", "General MIDI Overdriven Rock Guitar"),
            ("gm_acoustic_bass", "General MIDI Acoustic Bass"),
            ("gm_synth_brass", "General MIDI Synth Brass 1"),
            ("gm_shakuhachi", "General MIDI Shakuhachi Flute"),
            ("gm_standard_drums", "General MIDI Standard Drum Kit 1"),
        ];

        for (id, name) in expected_gm {
            let p = find_preset(id).unwrap_or_else(|| panic!("GM preset {} not found", id));
            assert_eq!(p.name, name);
            assert_eq!(p.category, FactoryPresetCategory::GeneralMidi);
        }
    }

    #[test]
    fn test_chiptune_and_tracker_coverage() {
        let chip_presets = presets_by_category(FactoryPresetCategory::ChiptuneTracker);
        assert_eq!(
            chip_presets.len(),
            5,
            "Must have 5 chiptune and tracker presets"
        );

        let expected_chip = [
            "nes_pulse_lead",
            "gameboy_triangle_bass",
            "sid_6581_arp",
            "fasttracker_arp_slicer",
            "chiptune_noise_drums",
        ];

        for id in expected_chip {
            assert!(
                find_preset(id).is_some(),
                "Chiptune preset {} must be findable",
                id
            );
        }
    }

    #[test]
    fn test_microtonal_and_physical_modeling_coverage() {
        let micro_presets = presets_by_category(FactoryPresetCategory::MicrotonalHarmony);
        assert!(micro_presets.len() >= 5);

        assert_eq!(find_preset("19_edo_neutral_pad").unwrap().tuning_edo, Some(19));
        assert_eq!(find_preset("31_edo_justness_lead").unwrap().tuning_edo, Some(31));
        assert_eq!(find_preset("53_edo_makam_drone").unwrap().tuning_edo, Some(53));
        assert_eq!(find_preset("22_edo_porcupine_bass").unwrap().tuning_edo, Some(22));
        assert!(find_preset("bp_gamma_chime").is_some());

        let phys_presets = presets_by_category(FactoryPresetCategory::PhysicalModeling);
        assert!(phys_presets.len() >= 12);
        assert!(find_preset("steinway_grand_piano").is_some());
        assert!(find_preset("shakuhachi_flute").is_some());
        assert!(find_preset("ravi_sitar").is_some());
        assert!(find_preset("stradivarius_cello").is_some());
        assert!(find_preset("hammond_b3_organ").is_some());
        assert!(find_preset("hohner_clavinet_d6").is_some());
        assert!(find_preset("rosewood_marimba").is_some());
        assert!(find_preset("glass_armonica").is_some());
    }

    #[test]
    fn test_factory_demo_templates_inventory() {
        let templates = all_demo_templates();
        assert_eq!(templates.len(), 5, "Must have 5 full factory demo templates");

        let expected_templates = [
            ("physical_showcase", "Physical Modeling Acoustic Showcase", 110.0),
            ("19_edo_odyssey", "19-EDO Microtonal Odyssey", 128.0),
            ("bohlen_pierce", "Bohlen-Pierce Tritave Ambient", 88.0),
            ("chiptune_anthem", "Chiptune Tracker 8-Bit Anthem", 144.0),
            ("gm_quintet", "General MIDI Standard Quintet", 120.0),
        ];

        for (id, title, bpm) in expected_templates {
            let tmpl = find_demo_template(id)
                .unwrap_or_else(|| panic!("Demo template {} not found", id));
            assert_eq!(tmpl.title, title);
            assert_eq!(tmpl.bpm, bpm);
            assert!(
                tmpl.tracks.len() >= 4,
                "Template {} must have >= 4 tracks",
                id
            );
            for t in tmpl.tracks {
                assert!(!t.name.is_empty());
                assert!(find_preset(t.preset_id).is_some());
            }
        }
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use eframe::egui;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::factory_presets::find_preset;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_asset_browser::BrowserCategory;

    #[test]
    fn test_asset_browser_soundbank_category() {
        let cat = BrowserCategory::Soundbanks;
        assert_eq!(cat.name(), "Soundbanks");
        assert_eq!(cat.icon(), "📦");

        let folders = cat.default_folders();
        assert!(folders.len() >= 7, "Soundbanks category must have >= 7 sub-folders");
        let folder_names: Vec<&str> = folders.iter().map(|(n, _)| *n).collect();
        assert!(folder_names.contains(&"Physical Modeling"));
        assert!(folder_names.contains(&"General MIDI Soundbank"));
        assert!(folder_names.contains(&"Chiptune & Tracker"));
        assert!(folder_names.contains(&"Microtonal & Harmony"));
        assert!(folder_names.contains(&"Starter Demo Templates"));
    }

    #[test]
    fn test_apply_factory_preset_populates_device_rack_and_macros() {
        let mut view = AwardWinningGuiView::default();

        let ok = view.apply_factory_preset("steinway_grand_piano", None);
        assert!(ok, "apply_factory_preset should succeed for steinway_grand_piano");

        assert_eq!(view.top_bar_state.selected_preset, "Steinway Concert Grand Piano");
        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("GrandPianoModel"));
        assert_eq!(view.device_rack_state.device_name, "Steinway Concert Grand Piano");

        // Verify macro knobs synced
        assert_eq!(view.top_bar_state.macro_tone, 0.72);
        assert_eq!(view.top_bar_state.macro_space, 0.40);
        assert_eq!(view.top_bar_state.macro_punch, 0.65);
        assert_eq!(view.top_bar_state.macro_character, 0.50);

        // Verify device rack parameters populated
        assert!(view.device_rack_state.node_param_values.contains_key("felt_hardness"));
        assert_eq!(view.device_rack_state.node_param_values.get("felt_hardness"), Some(&0.65));
        assert_eq!(view.device_rack_state.node_param_values.get("soundboard_resonance"), Some(&0.82));

        // Verify inspector synced
        assert_eq!(view.inspector_state.selected_node_kind.as_deref(), Some("GrandPianoModel"));
        assert_eq!(view.inspector_state.target_name, "Steinway Concert Grand Piano");
        assert_eq!(view.inspector_state.node_param_values.get("felt_hardness"), Some(&0.65));
    }

    #[test]
    fn test_apply_factory_preset_live_param_bus_zero_alloc() {
        let mut view = AwardWinningGuiView::default();
        let mut bus = ParamBus::new();
        view.selected_track_idx = 0; // Track 1 -> param base 1000

        // Pre-register track 1 parameters
        for p in 0..64 {
            bus.register(ParamId(1000 + p), 0.0);
        }
        for p in 0..16 {
            bus.register(ParamId(1100 + p), 0.0);
        }

        let preset = find_preset("gm_grand_piano").expect("gm_grand_piano must exist");

        // Execute under AllocGuard to guarantee zero heap allocations on the audio thread
        {
            let _guard = AllocGuard::new();
            let ok = view.apply_factory_preset("gm_grand_piano", Some(&bus));
            assert!(ok);
        }

        // Verify parameters were written to ParamBus
        assert_eq!(bus.get(ParamId(1100)), Some(preset.macro_tone)); // Tone / Cutoff
        assert_eq!(bus.get(ParamId(1102)), Some(preset.macro_space)); // Space / Decay
        assert_eq!(bus.get(ParamId(1105)), Some(preset.macro_punch)); // Punch / Drive
        assert_eq!(bus.get(ParamId(1000)), Some(0.80)); // hammer_velocity
        assert_eq!(bus.get(ParamId(1002)), Some(0.70)); // brilliance
    }

    #[test]
    fn test_load_factory_demo_template_instantiates_multitrack_session() {
        let mut view = AwardWinningGuiView::default();

        let ok = view.load_factory_demo_template("19_edo_odyssey");
        assert!(ok, "Loading 19_edo_odyssey should succeed");

        assert_eq!(view.top_bar_state.bpm, 128.0);
        assert_eq!(view.top_bar_state.key_signature, "19-EDO C");
        assert!(view.inspector_state.scale_name.contains("19-EDO"));

        // Verify tracks
        assert_eq!(view.tracks.len(), 4);
        assert_eq!(view.tracks[0].name, "19-EDO Neutral Pad");
        assert_eq!(view.tracks[1].name, "19-EDO Sub Bass");
        assert_eq!(view.tracks[2].name, "Quartertone Pluck");
        assert_eq!(view.tracks[3].name, "53-EDO Makam Lead");

        // Verify lead track notes were loaded into piano roll
        assert_eq!(view.piano_roll_notes.len(), 6);
        assert_eq!(view.piano_roll_notes[0].pitch_idx, 0);
        assert_eq!(view.piano_roll_notes[1].pitch_idx, 6);

        // Verify lead track instrument was loaded into rack
        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("EdoTuning"));
    }

    #[test]
    fn test_top_bar_pending_demo_template_processing() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        // Queue a demo template via top bar state
        view.top_bar_state.pending_demo_template = Some("physical_showcase".to_string());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Pending flag was consumed
        assert!(view.top_bar_state.pending_demo_template.is_none());

        // Showcase loaded
        assert_eq!(view.top_bar_state.bpm, 110.0);
        assert_eq!(view.tracks.len(), 5);
        assert_eq!(view.tracks[0].name, "Steinway Grand Piano");
        assert_eq!(view.tracks[1].name, "Stradivarius Cello");
        assert_eq!(view.tracks[2].name, "Bamboo Shakuhachi");
    }

    #[test]
    fn test_asset_browser_callback_loads_preset_and_template() {
        let mut view = AwardWinningGuiView::default();
        let _ctx = egui::Context::default();

        // Simulate asset selection from asset browser
        view.asset_browser_state.selected_category = crate::views::modern_asset_browser::BrowserCategory::Soundbanks;

        // 1. Loading preset from browser
        let preset_asset = "Hindustani Ravi Shankar Sitar";
        let ok = view.apply_factory_preset(preset_asset, None);
        assert!(ok);
        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("JawariBridge"));

        // 2. Loading demo template from browser
        let tmpl_asset = "Chiptune Tracker 8-Bit Anthem";
        let tmpl = crate::factory_presets::find_demo_template(tmpl_asset).expect("Chiptune anthem must exist");
        let ok_t = view.load_factory_demo_template(tmpl.id);
        assert!(ok_t);
        assert_eq!(view.top_bar_state.bpm, 144.0);
        assert_eq!(view.tracks[0].name, "NES 8-Bit Pulse Lead");
    }
}
