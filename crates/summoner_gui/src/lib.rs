// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! GPU-accelerated UI & LOD rendering engine for Summoner DAW.

#![allow(clippy::new_without_default)]

pub mod ascii_renderer;
pub mod docking_layout;
pub mod gpu_waveform;
pub mod hud_overlay;
pub mod layout_math;
pub mod lod;
pub mod oscilloscope_view;
pub mod platform;
pub mod renderer;
pub mod touch_controls;
pub mod visualizer;
pub mod waveform_cache;

#[cfg(feature = "gui")]
pub mod app;
#[cfg(feature = "gui")]
pub mod command_palette;
#[cfg(feature = "gui")]
pub mod param_controls;
#[cfg(feature = "gui")]
pub mod patch_matrix;
#[cfg(feature = "gui")]
pub mod stage_view;
#[cfg(feature = "gui")]
pub mod theme;
#[cfg(feature = "gui")]
pub mod tier32_tests;
#[cfg(feature = "gui")]
pub mod tier33_advanced_tests;
#[cfg(feature = "gui")]
pub mod tier33_tests;
#[cfg(feature = "gui")]
pub mod tier34_tests;
pub mod tier35_tests;
pub mod tier36_tests;
pub mod tier37_tests;
pub mod tier38_tests;
pub mod tier39_tests;
pub mod tier40_tests;
pub mod tier41_tests;
pub mod tier43_tests;
pub mod tier44_tests;
pub mod tier45_tests;
pub mod tier46_tests;
pub mod tier47_tests;
#[cfg(feature = "gui")]
pub mod tier48_tests;
#[cfg(feature = "gui")]
pub mod tier49_tests;
#[cfg(feature = "gui")]
pub mod tier50_tests;
#[cfg(feature = "gui")]
pub mod tier51_tests;
#[cfg(feature = "gui")]
pub mod touch_gestures;
#[cfg(feature = "gui")]
pub mod transport_bar;
#[cfg(feature = "gui")]
pub mod views;

#[cfg(feature = "gui")]
pub use views::accessibility_announcer::AccessibilityAnnouncerView;
#[cfg(feature = "gui")]
pub use views::detachable_window_manager::DetachableWindowManagerView;
#[cfg(feature = "gui")]
pub use views::dpi_scale_panel::DpiScalePanelView;
#[cfg(feature = "gui")]
pub use views::dsp_rack_dock::DspRackDockView;
#[cfg(feature = "gui")]
pub use views::harmonic_tension_map::HarmonicTensionMapView;
#[cfg(feature = "gui")]
pub use views::keybinding_editor::KeybindingEditorView;
#[cfg(feature = "gui")]
pub use views::live_macro_rack::LiveMacroRackView;
#[cfg(feature = "gui")]
pub use views::macro_rotary_dial::MacroRotaryDialView;
#[cfg(feature = "gui")]
pub use views::meter_bridge_view::MeterBridgeView;
#[cfg(feature = "gui")]
pub use views::sample_editor_view;
#[cfg(feature = "gui")]
pub use views::sample_editor_view::SampleEditorView;
#[cfg(feature = "gui")]
pub use views::spatial_panner_view::SpatialPannerView;
#[cfg(feature = "gui")]
pub use views::spectrogram_3d_view::Spectrogram3DView;

#[cfg(feature = "gui")]
pub fn launch(
    project: summoner_project::schema::ProjectConfig,
    param_bus: std::sync::Arc<summoner_core::param_bus::ParamBus>,
) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Summoner DAW",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::SummonerApp::new(project, param_bus)))),
    )
    .unwrap();
}
