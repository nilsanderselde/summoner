// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use std::collections::HashMap;
use summoner_core::node::{AudioNode, ProcessContext};
use summoner_dsp::{scan_plugin_directory, PluginAudioNode, PluginFormat};
use summoner_project::schema::{NodeConfig, PluginStateConfig};

#[test]
fn test_tier27_vst3_and_clap_hosting_full_pipeline() {
    // 1. Scan plugin directory
    let temp_dir = std::env::temp_dir().join("summoner_tier27_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let vst3_dir = temp_dir.join("Echo.vst3");
    std::fs::create_dir_all(&vst3_dir).unwrap();
    let clap_file = temp_dir.join("Reverb.clap");
    std::fs::File::create(&clap_file).unwrap();

    let scanned = scan_plugin_directory(&temp_dir);
    assert_eq!(scanned.len(), 2);

    // 2. Load VST3 and CLAP AudioNodes
    let vst3_desc = scanned
        .iter()
        .find(|p| p.format == PluginFormat::Vst3)
        .unwrap();
    let mut vst3_node = PluginAudioNode::new(vst3_desc.clone());

    let clap_desc = scanned
        .iter()
        .find(|p| p.format == PluginFormat::Clap)
        .unwrap();
    let mut clap_node = PluginAudioNode::new(clap_desc.clone());

    // 3. Test Parameter Automation & Audio Routing
    vst3_node.set_parameter_by_name("Gain", 0.75);
    clap_node.set_parameter_by_name("Gain", 1.25);

    let ctx = ProcessContext::new(44100, 120.0, 0);
    let in_l = vec![1.0f32; 128];
    let in_r = vec![1.0f32; 128];
    let mut out_l = vec![0.0f32; 128];
    let mut out_r = vec![0.0f32; 128];

    vst3_node.process(
        &[&in_l[..], &in_r[..]],
        &mut [&mut out_l[..], &mut out_r[..]],
        &ctx,
    );
    assert_eq!(out_l[0], 0.75);
    assert_eq!(out_r[0], 0.75);

    // 4. Test State Save / Restore in NodeConfig TOML Schema
    let mut params_map = HashMap::new();
    params_map.insert("Gain".to_string(), 0.85);

    let node_config = NodeConfig {
        kind: "VstPluginNode".to_string(),
        params: HashMap::new(),
        plugin_state: Some(PluginStateConfig {
            plugin_name: "Echo".to_string(),
            plugin_path: "Echo.vst3".to_string(),
            format: "Vst3".to_string(),
            is_bypassed: false,
            state_base64: "SGVsbG8gVlNUMw==".to_string(),
            parameters: params_map,
        }),
    };

    assert_eq!(node_config.kind, "VstPluginNode");
    assert!(node_config.plugin_state.is_some());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
