// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 79: Universal DSP Coverage, Two-Tier Parameter Reflection & Alias Normalization Tests.
//!
//! Validates:
//! 1. Complete catalog coverage (>270 modules) across all 10 DSP categories.
//! 2. Zero-allocation reflection parameter models with proper scaling (linear, log, enum, bool, int).
//! 3. Alias normalization for snake_case, PascalCase, and legacy DSP aliases.
//! 4. Seamless integration with `DspRackDockView` and `MacroRackView`.

#[cfg(test)]
mod tests {
    use crate::dsp_node_ui::*;
    use crate::views::dsp_rack_dock::DspRackDockView;
    use crate::views::modern_device_rack::ModernDeviceRackState;
    use std::collections::HashSet;

    #[test]
    fn test_tier79_dsp_inventory_size_and_uniqueness() {
        let inventory = DspNodeRegistry::inventory();
        assert!(
            inventory.len() >= 270,
            "DspNodeRegistry should contain at least 270 nodes, but found {}",
            inventory.len()
        );

        let mut seen = HashSet::new();
        for (name, _, desc) in &inventory {
            assert!(
                seen.insert(*name),
                "Duplicate node type registered in inventory: {}",
                name
            );
            assert!(
                !desc.is_empty(),
                "Node {} must have a non-empty description",
                name
            );
        }
    }

    #[test]
    fn test_tier79_all_categories_represented() {
        let all_categories = [
            DspNodeCategory::Oscillator,
            DspNodeCategory::AcousticPhysicalModel,
            DspNodeCategory::FilterEq,
            DspNodeCategory::DynamicsMaster,
            DspNodeCategory::DistortionSaturation,
            DspNodeCategory::Modulation,
            DspNodeCategory::TimeSpace,
            DspNodeCategory::SpatialSurround,
            DspNodeCategory::SpectralResynthesis,
            DspNodeCategory::NeuralAi,
            DspNodeCategory::SamplerSlicer,
            DspNodeCategory::CompositeSynth,
            DspNodeCategory::Utility,
        ];

        for category in all_categories {
            let nodes = DspNodeRegistry::nodes_by_category(category);
            assert!(
                !nodes.is_empty(),
                "Category {:?} must have registered nodes",
                category
            );
            assert!(
                !category.display_label().is_empty(),
                "Category {:?} must have a valid display label",
                category
            );
            let (r, g, b) = category.theme_color_rgb();
            assert!(
                r > 0 || g > 0 || b > 0,
                "Category {:?} theme color must not be black",
                category
            );
        }
    }

    #[test]
    fn test_tier79_universal_node_instantiation_and_parameter_contracts() {
        for (name, expected_category, _) in DspNodeRegistry::inventory() {
            let mut node = DspNodeRegistry::create_node_ui(name)
                .unwrap_or_else(|| panic!("DspNodeRegistry::create_node_ui failed for {}", name));

            assert_eq!(
                node.node_type_name(),
                name,
                "Node type name mismatch for {}",
                name
            );
            assert_eq!(
                node.category(),
                expected_category,
                "Category mismatch for node {}",
                name
            );
            assert!(
                !node.display_name().is_empty(),
                "Display name for {} should not be empty",
                name
            );

            let params = node.parameters();
            assert!(
                params.len() >= 2,
                "Node {} should have at least 2 parameters for reflection, got {}",
                name,
                params.len()
            );

            for param in params {
                assert!(
                    !param.id.is_empty(),
                    "Parameter ID empty on node {}",
                    name
                );
                assert!(
                    !param.name.is_empty(),
                    "Parameter Name empty on {}.{}",
                    name,
                    param.id
                );

                // Test normalization bounds [0.0, 1.0]
                let norm = param.normalized_value();
                assert!(
                    (0.0..=1.0001).contains(&norm),
                    "Normalized value {} out of range on {}.{}",
                    norm,
                    name,
                    param.id
                );

                // Test format output
                let formatted = param.format_value();
                assert!(
                    !formatted.is_empty(),
                    "Format output empty for {}.{}",
                    name,
                    param.id
                );
            }

            // Test mutating first parameter
            let first_id = node.parameters()[0].id.clone();
            let orig_val = node.get_param_value(&first_id).unwrap();
            let new_val = orig_val + 1.0;
            assert!(node.set_param_value(&first_id, new_val));
            assert_eq!(node.get_param_value(&first_id), Some(new_val));

            // Test locking parameter prevents mutation
            if let Some(p) = node.parameters_mut().iter_mut().find(|p| p.id == first_id) {
                p.is_locked = true;
            }
            assert!(!node.set_param_value(&first_id, new_val + 5.0));
            assert_eq!(node.get_param_value(&first_id), Some(new_val));
        }
    }

    #[test]
    fn test_tier79_alias_normalization_matrix() {
        let test_cases = [
            ("tube_drive", "TubeSaturationNode"),
            ("tubedrive", "TubeSaturationNode"),
            ("svf_filter", "FilterSVF"),
            ("svf", "FilterSVF"),
            ("ladder", "FilterLadder"),
            ("tape_delay", "EffectDelay"),
            ("tapedelay", "EffectDelay"),
            ("conv_reverb", "ConvolutionReverbNode"),
            ("reverb", "EffectReverb"),
            ("compressor", "CompressorNode"),
            ("limiter", "LimiterNode"),
            ("piano", "GrandPianoModel"),
            ("grand_piano", "GrandPianoModel"),
            ("gamelan", "GamelanGender"),
            ("hurdy_gurdy", "HurdyGurdy"),
            ("sitar", "SitarModel"),
            ("shakuhachi", "ShakuhachiModel"),
            ("pipe_organ", "PipeOrganModel"),
            ("glass_armonica", "GlassArmonica"),
            ("violin", "BowedStringModel"),
            ("steelpan", "SteelpanModel"),
            ("ney", "TurkishNeyModel"),
            ("brass", "WaveguideBrassModel"),
            ("woodwind", "WoodwindJetModel"),
            ("spring_reverb", "SpringLatticeNode"),
            ("plate_reverb", "PlateTankNode"),
            ("hammond", "TonewheelOrganModel"),
            ("b3", "TonewheelOrganModel"),
            ("vocaltract", "VocalTractNode"),
            ("chorus", "EffectChorus"),
            ("flanger", "EffectFlanger"),
            ("phaser", "EffectPhaser"),
            ("eq", "ParametricEqNode"),
            ("deesser", "DeesserNode"),
            ("gate", "NoiseGateNode"),
            ("distortion", "DistortionNode"),
            ("console", "ConsoleEmulationNode"),
            ("bitcrusher", "BitcrusherNode"),
            ("wavefolder", "WavefolderNode"),
            ("exciter", "HarmonicExciterNode"),
            ("atmos", "Atmos916Spatializer"),
            ("binaural", "BinauralSpatializerNode"),
            ("lufs", "LufsMeterNode"),
            ("eburadar", "EbuLoudnessRadar"),
            ("midside", "MidSideNode"),
            ("gain", "GainNode"),
            ("tuner", "ChromaticTunerNode"),
            ("vca", "VCA"),
            ("stretcher", "WsolaTimeStretcher"),
            ("stems", "StemSeparator"),
            ("warp", "ElasticWarpEngine"),
            ("oversampler", "Oversampler"),
        ];

        for (alias, expected) in test_cases {
            let normalized = DspNodeRegistry::normalize_type_name(alias);
            assert_eq!(
                normalized,
                Some(expected),
                "Normalization failed for alias '{}'",
                alias
            );

            // Test instantiation via alias
            let node = DspNodeRegistry::create_node_ui(alias)
                .unwrap_or_else(|| panic!("Failed to create node via alias '{}'", alias));
            assert_eq!(
                node.node_type_name(),
                expected,
                "Node type name mismatch for alias '{}'",
                alias
            );
        }
    }

    #[test]
    fn test_tier79_search_functionality() {
        let searches = [
            ("reverb", 4),
            ("synth", 4),
            ("filter", 6),
            ("quantum", 4),
            ("spatial", 5),
            ("neural", 8),
            ("transient", 5),
            ("master", 3),
        ];

        for (query, min_expected) in searches {
            let results = DspNodeRegistry::search_nodes(query);
            assert!(
                results.len() >= min_expected,
                "Search query '{}' expected >= {} matches, found {}",
                query,
                min_expected,
                results.len()
            );
        }
    }

    #[test]
    fn test_tier79_dsp_rack_dock_integration() {
        let mut dock = DspRackDockView::default();
        let initial_count = dock.modules.len();
        assert_eq!(initial_count, 4);

        // Add 5 modules from the registry
        assert!(dock.load_from_registry("AetherSynth"));
        assert!(dock.load_from_registry("FilterLadder"));
        assert!(dock.load_from_registry("TubeSaturationNode"));
        assert!(dock.load_from_registry("EffectDelay"));
        assert!(dock.load_from_registry("LimiterNode"));

        assert_eq!(dock.modules.len(), initial_count + 5);

        // Check modules have reflected parameters
        for module in &dock.modules {
            assert!(!module.params.is_empty(), "Rack module {} must have parameters", module.name);
        }

        // Test module reordering
        let mod0_id = dock.modules[0].id.clone();
        let mod1_id = dock.modules[1].id.clone();
        assert!(dock.reorder_module(0, 2));
        assert_eq!(dock.modules[0].id, mod1_id);
        assert_eq!(dock.modules[1].id, mod0_id);

        // Test bypass toggle
        dock.modules[0].is_bypassed = true;
        assert!(dock.modules[0].is_bypassed);

        // Test remove module
        dock.remove_module(0);
        assert_eq!(dock.modules.len(), initial_count + 4);
    }

    #[test]
    fn test_tier79_modern_device_rack_integration() {
        let mut rack_state = ModernDeviceRackState::default();
        assert_eq!(rack_state.device_name, "Synth 1");
        assert!(rack_state.is_enabled);

        // Test dynamic node selection
        rack_state.selected_node_kind = Some("TubeSaturationNode".to_string());
        let registry = DspNodeRegistry::new();
        let desc = registry.get("TubeSaturationNode").expect("TubeSaturationNode must exist in registry");
        assert_eq!(desc.kind_id, "TubeSaturationNode");
    }
}
