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

pub mod lod;
pub mod renderer;
pub mod visualizer;
pub mod ascii_renderer;

#[cfg(feature = "gui")]
pub mod command_palette;
#[cfg(feature = "gui")]
pub mod app;
#[cfg(feature = "gui")]
pub mod views;
#[cfg(feature = "gui")]
pub mod stage_view;

#[cfg(feature = "gui")]
pub fn launch(project: summoner_project::schema::ProjectConfig, param_bus: std::sync::Arc<summoner_core::param_bus::ParamBus>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Summoner DAW",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::SummonerApp::new(project, param_bus)))),
    ).unwrap();
}
