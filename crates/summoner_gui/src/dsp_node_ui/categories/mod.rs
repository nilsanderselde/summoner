// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Category-partitioned registry modules for DSP node descriptors and UI builders.

use std::collections::HashMap;

use super::types::DspNodeDescriptor;
use super::DspNodeUi;

pub mod oscillator;
pub mod acoustic_physical_model;
pub mod filter_eq;
pub mod dynamics_master;
pub mod distortion_saturation;
pub mod modulation;
pub mod time_space;
pub mod spatial_surround;
pub mod spectral_resynthesis;
pub mod neural_ai;
pub mod sampler_slicer;
pub mod composite_synth;
pub mod utility;

pub(super) fn register_all_descriptors(descriptors: &mut HashMap<String, DspNodeDescriptor>) {
    oscillator::register_descriptors(descriptors);
    acoustic_physical_model::register_descriptors(descriptors);
    filter_eq::register_descriptors(descriptors);
    dynamics_master::register_descriptors(descriptors);
    distortion_saturation::register_descriptors(descriptors);
    modulation::register_descriptors(descriptors);
    time_space::register_descriptors(descriptors);
    spatial_surround::register_descriptors(descriptors);
    spectral_resynthesis::register_descriptors(descriptors);
    neural_ai::register_descriptors(descriptors);
    sampler_slicer::register_descriptors(descriptors);
    composite_synth::register_descriptors(descriptors);
    utility::register_descriptors(descriptors);
}

pub(super) fn try_create_node_ui(target_name: &str) -> Option<Box<dyn DspNodeUi>> {
    if let Some(node) = oscillator::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = acoustic_physical_model::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = filter_eq::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = dynamics_master::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = distortion_saturation::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = modulation::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = time_space::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = spatial_surround::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = spectral_resynthesis::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = neural_ai::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = sampler_slicer::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = composite_synth::create_node_ui(target_name) {
        return Some(node);
    }
    if let Some(node) = utility::create_node_ui(target_name) {
        return Some(node);
    }
    None
}
