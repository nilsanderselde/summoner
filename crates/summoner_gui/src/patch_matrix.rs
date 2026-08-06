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

//! Interactive Modular Synthesizer Patch Matrix & Cable Routing View widget (`PatchMatrixView`).

use std::collections::HashMap;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Rect, Sense, Stroke, Vec2};

/// Source node category for modular synthesizer modulation routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SourceKind {
    Lfo,
    Envelope,
    Sequencer,
    MidiCc,
    Custom(String),
}

/// Source node definition in the patch matrix grid.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceNode {
    pub id: String,
    pub name: String,
    pub kind: SourceKind,
    pub rgb: (u8, u8, u8),
    pub current_signal: f32,
}

impl SourceNode {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: SourceKind) -> Self {
        let (r, g, b) = match &kind {
            SourceKind::Lfo => (0, 220, 255),        // Cyan
            SourceKind::Envelope => (255, 140, 0),   // Orange/Amber
            SourceKind::Sequencer => (50, 220, 100), // Emerald Green
            SourceKind::MidiCc => (190, 90, 255),    // Purple
            SourceKind::Custom(_) => (255, 215, 0),  // Gold
        };
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            rgb: (r, g, b),
            current_signal: 0.5,
        }
    }

    pub fn with_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.rgb = (r, g, b);
        self
    }
}

/// Destination target node in the patch matrix grid.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DestNode {
    pub id: String,
    pub name: String,
    pub group: String,
}

impl DestNode {
    pub fn new(id: impl Into<String>, name: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            group: group.into(),
        }
    }
}

/// A connection pin node state in the patch matrix grid.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchConnection {
    pub active: bool,
    pub intensity: f32,
    pub muted: bool,
    pub inverted: bool,
}

impl Default for PatchConnection {
    fn default() -> Self {
        Self {
            active: false,
            intensity: 1.0,
            muted: false,
            inverted: false,
        }
    }
}

/// Active routed signal path representation for signal flow lookup and inspection.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutedSignal {
    pub source_id: String,
    pub dest_id: String,
    pub intensity: f32,
    pub effective_level: f32,
}

/// Interactive Modular Synthesizer Patch Matrix & Cable Routing View Widget (`PatchMatrixView`).
#[derive(Debug, Clone)]
pub struct PatchMatrixView {
    pub sources: Vec<SourceNode>,
    pub destinations: Vec<DestNode>,
    pub connections: HashMap<(String, String), PatchConnection>,
    pub min_hit_target_size: f32,
    pub hovered_pin: Option<(String, String)>,
    pub selected_pin: Option<(String, String)>,
}

impl Default for PatchMatrixView {
    fn default() -> Self {
        Self::with_default_nodes()
    }
}

impl PatchMatrixView {
    /// Create a new empty `PatchMatrixView`.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            destinations: Vec::new(),
            connections: HashMap::new(),
            min_hit_target_size: 44.0,
            hovered_pin: None,
            selected_pin: None,
        }
    }

    /// Create a `PatchMatrixView` pre-populated with standard synthesizer sources and targets.
    pub fn with_default_nodes() -> Self {
        let sources = vec![
            SourceNode::new("lfo1", "LFO 1 (Sine)", SourceKind::Lfo),
            SourceNode::new("lfo2", "LFO 2 (Triangle)", SourceKind::Lfo),
            SourceNode::new("env1", "Env 1 (Filter)", SourceKind::Envelope),
            SourceNode::new("env2", "Env 2 (Amp)", SourceKind::Envelope),
            SourceNode::new("seq1", "Step Sequencer", SourceKind::Sequencer),
            SourceNode::new("midi_mod", "MIDI Mod Wheel", SourceKind::MidiCc),
        ];

        let destinations = vec![
            DestNode::new("cutoff", "Filter Cutoff", "Filter"),
            DestNode::new("resonance", "Filter Res", "Filter"),
            DestNode::new("pitch", "Pitch / VCO", "Oscillator"),
            DestNode::new("amp", "Master Amp", "Amplifier"),
            DestNode::new("pan", "Stereo Pan", "Output"),
            DestNode::new("fx_wet", "FX Reverb Wet", "Effects"),
        ];

        let mut matrix = Self {
            sources,
            destinations,
            connections: HashMap::new(),
            min_hit_target_size: 44.0,
            hovered_pin: None,
            selected_pin: None,
        };

        matrix.connect("lfo1", "cutoff", 0.75);
        matrix.connect("env1", "cutoff", 1.0);
        matrix.connect("env2", "amp", 1.0);

        matrix
    }

    pub fn add_source(&mut self, source: SourceNode) {
        if !self.sources.iter().any(|s| s.id == source.id) {
            self.sources.push(source);
        }
    }

    pub fn add_destination(&mut self, dest: DestNode) {
        if !self.destinations.iter().any(|d| d.id == dest.id) {
            self.destinations.push(dest);
        }
    }

    /// Toggle connection state between source and destination.
    pub fn toggle_connection(&mut self, source_id: &str, dest_id: &str) -> bool {
        let key = (source_id.to_string(), dest_id.to_string());
        let conn = self.connections.entry(key).or_default();
        conn.active = !conn.active;
        conn.active
    }

    /// Connect a source to a destination with specified signal intensity (0.0 to 1.0).
    pub fn connect(&mut self, source_id: &str, dest_id: &str, intensity: f32) {
        let key = (source_id.to_string(), dest_id.to_string());
        let conn = self.connections.entry(key).or_default();
        conn.active = true;
        conn.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Disconnect a pin node.
    pub fn disconnect(&mut self, source_id: &str, dest_id: &str) {
        let key = (source_id.to_string(), dest_id.to_string());
        if let Some(conn) = self.connections.get_mut(&key) {
            conn.active = false;
        }
    }

    /// Check if connection exists and is actively routing signal.
    pub fn is_connected(&self, source_id: &str, dest_id: &str) -> bool {
        self.connections
            .get(&(source_id.to_string(), dest_id.to_string()))
            .map(|c| c.active && !c.muted)
            .unwrap_or(false)
    }

    /// Get intensity (0.0 to 1.0) of connection. Returns 0.0 if inactive.
    pub fn get_intensity(&self, source_id: &str, dest_id: &str) -> f32 {
        self.connections
            .get(&(source_id.to_string(), dest_id.to_string()))
            .filter(|c| c.active && !c.muted)
            .map(|c| c.intensity)
            .unwrap_or(0.0)
    }

    /// Set routing intensity for a connection.
    pub fn set_intensity(&mut self, source_id: &str, dest_id: &str, intensity: f32) {
        let key = (source_id.to_string(), dest_id.to_string());
        let conn = self.connections.entry(key).or_default();
        conn.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Lookup calculated signal flow level from source to destination.
    pub fn get_signal_flow(&self, source_id: &str, dest_id: &str) -> Option<f32> {
        let conn = self
            .connections
            .get(&(source_id.to_string(), dest_id.to_string()))?;
        if !conn.active || conn.muted {
            return None;
        }
        let src = self.sources.iter().find(|s| s.id == source_id)?;
        let val = src.current_signal * conn.intensity;
        Some(if conn.inverted { -val } else { val })
    }

    /// Get all active outgoing routes for a source.
    pub fn get_routes_for_source(&self, source_id: &str) -> Vec<(String, f32)> {
        self.destinations
            .iter()
            .filter_map(|dest| {
                let intensity = self.get_intensity(source_id, &dest.id);
                if intensity > 0.0 {
                    Some((dest.id.clone(), intensity))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all active incoming routes for a destination target.
    pub fn get_routes_for_dest(&self, dest_id: &str) -> Vec<(String, f32)> {
        self.sources
            .iter()
            .filter_map(|src| {
                let intensity = self.get_intensity(&src.id, dest_id);
                if intensity > 0.0 {
                    Some((src.id.clone(), intensity))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get total list of active routed signals across the entire matrix.
    pub fn get_active_routes(&self) -> Vec<RoutedSignal> {
        let mut routes = Vec::new();
        for src in &self.sources {
            for dest in &self.destinations {
                if let Some(level) = self.get_signal_flow(&src.id, &dest.id) {
                    let intensity = self.get_intensity(&src.id, &dest.id);
                    routes.push(RoutedSignal {
                        source_id: src.id.clone(),
                        dest_id: dest.id.clone(),
                        intensity,
                        effective_level: level,
                    });
                }
            }
        }
        routes
    }

    /// Update live signal level for a source node.
    pub fn update_source_signal(&mut self, source_id: &str, signal: f32) {
        if let Some(src) = self.sources.iter_mut().find(|s| s.id == source_id) {
            src.current_signal = signal.clamp(0.0, 1.0);
        }
    }
}

#[cfg(feature = "gui")]
impl PatchMatrixView {
    /// Render interactive Patch Matrix grid and signal routing view.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Modular Synth Patch Matrix & Routing");
            ui.add_space(8.0);

            let pin_size = self.min_hit_target_size.max(44.0);

            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("patch_matrix_grid")
                    .spacing([4.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(180.0, pin_size),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(egui::RichText::new("Source \\ Dest").strong().italics());
                            },
                        );

                        for dest in &self.destinations {
                            ui.allocate_ui_with_layout(
                                Vec2::new(pin_size, pin_size),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.label(egui::RichText::new(&dest.name).small().strong());
                                },
                            );
                        }
                        ui.end_row();

                        let sources_clone = self.sources.clone();
                        let dests_clone = self.destinations.clone();

                        for src in &sources_clone {
                            let (r, g, b) = src.rgb;
                            let src_color = Color32::from_rgb(r, g, b);

                            ui.allocate_ui_with_layout(
                                Vec2::new(180.0, pin_size),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let (rect, _) = ui
                                        .allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                                    ui.painter().circle_filled(rect.center(), 6.0, src_color);
                                    ui.add_space(4.0);

                                    ui.label(egui::RichText::new(&src.name).strong());

                                    let bar_w = 30.0;
                                    let (bar_rect, _) = ui
                                        .allocate_exact_size(Vec2::new(bar_w, 8.0), Sense::hover());
                                    ui.painter()
                                        .rect_filled(bar_rect, 2.0, Color32::from_gray(40));
                                    let fill_w = bar_w * src.current_signal.clamp(0.0, 1.0);
                                    let fill_rect =
                                        Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, 8.0));
                                    ui.painter().rect_filled(fill_rect, 2.0, src_color);
                                },
                            );

                            for dest in &dests_clone {
                                let key = (src.id.clone(), dest.id.clone());
                                let conn = self.connections.get(&key).cloned().unwrap_or_default();

                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(pin_size, pin_size),
                                    Sense::click_and_drag(),
                                );

                                let hit_target_ok = rect.width() >= 44.0 && rect.height() >= 44.0;
                                let is_hovered = response.hovered();
                                if is_hovered {
                                    self.hovered_pin = Some(key.clone());
                                }

                                if response.clicked() {
                                    self.toggle_connection(&src.id, &dest.id);
                                }

                                if response.dragged() {
                                    let delta_y = response.drag_delta().y;
                                    let new_intensity =
                                        (conn.intensity - delta_y * 0.01).clamp(0.0, 1.0);
                                    self.set_intensity(&src.id, &dest.id, new_intensity);
                                }

                                let painter = ui.painter();
                                let center = rect.center();

                                let bg_color = if is_hovered {
                                    Color32::from_rgb(50, 60, 80)
                                } else {
                                    Color32::from_rgb(25, 28, 36)
                                };
                                painter.rect_filled(rect, 6.0, bg_color);

                                let border_color = if conn.active {
                                    src_color
                                } else if is_hovered {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(60)
                                };
                                painter.rect_stroke(rect, 6.0, Stroke::new(1.5_f32, border_color));

                                if conn.active {
                                    let glow_radius = 8.0 + 8.0 * conn.intensity;
                                    let mut glow_color = src_color;
                                    glow_color = Color32::from_rgba_unmultiplied(
                                        glow_color.r(),
                                        glow_color.g(),
                                        glow_color.b(),
                                        (180.0 * conn.intensity) as u8,
                                    );
                                    painter.circle_filled(center, glow_radius, glow_color);

                                    painter.circle_filled(center, 6.0, Color32::WHITE);
                                    painter.circle_stroke(
                                        center,
                                        14.0,
                                        Stroke::new(2.0_f32, src_color),
                                    );

                                    if conn.muted {
                                        painter.line_segment(
                                            [
                                                center - Vec2::new(6.0, 6.0),
                                                center + Vec2::new(6.0, 6.0),
                                            ],
                                            Stroke::new(2.0_f32, Color32::RED),
                                        );
                                    }
                                } else {
                                    painter.circle_filled(center, 4.0, Color32::from_gray(10));
                                    painter.circle_stroke(
                                        center,
                                        4.0,
                                        Stroke::new(1.0_f32, Color32::from_gray(60)),
                                    );
                                }

                                response.on_hover_ui(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} ➔ {}",
                                            src.name, dest.name
                                        ))
                                        .strong(),
                                    );
                                    ui.label(format!(
                                        "Active: {}",
                                        if conn.active { "Yes" } else { "No" }
                                    ));
                                    ui.label(format!("Intensity: {:.0}%", conn.intensity * 100.0));
                                    ui.label(format!(
                                        "Hit Target Size: {:.0}x{:.0}pt ({})",
                                        rect.width(),
                                        rect.height(),
                                        if hit_target_ok {
                                            "OK >=44pt"
                                        } else {
                                            "WARN <44pt"
                                        }
                                    ));
                                    ui.label("Click to toggle | Drag vertical to adjust intensity");
                                });
                            }
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Active Cable Routings:").strong());
                let active_routes = self.get_active_routes();
                ui.label(format!("{} active connections", active_routes.len()));
            });

            ui.horizontal_wrapped(|ui| {
                for route in self.get_active_routes() {
                    let src = self.sources.iter().find(|s| s.id == route.source_id);
                    let dest = self.destinations.iter().find(|d| d.id == route.dest_id);
                    if let (Some(s), Some(d)) = (src, dest) {
                        let (r, g, b) = s.rgb;
                        let color = Color32::from_rgb(r, g, b);
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 5.0, color);
                            ui.label(format!(
                                "{} ➔ {}: {:.0}%",
                                s.name,
                                d.name,
                                route.intensity * 100.0
                            ));
                        });
                        ui.add_space(8.0);
                    }
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_matrix_initialization() {
        let matrix = PatchMatrixView::with_default_nodes();
        assert!(!matrix.sources.is_empty());
        assert!(!matrix.destinations.is_empty());
        assert!(matrix.min_hit_target_size >= 44.0);
    }

    #[test]
    fn test_node_patching_and_connection_toggling() {
        let mut matrix = PatchMatrixView::new();
        matrix.add_source(SourceNode::new("lfo1", "LFO 1", SourceKind::Lfo));
        matrix.add_destination(DestNode::new("cutoff", "Filter Cutoff", "Filter"));

        assert!(!matrix.is_connected("lfo1", "cutoff"));

        let state = matrix.toggle_connection("lfo1", "cutoff");
        assert!(state);
        assert!(matrix.is_connected("lfo1", "cutoff"));

        let state2 = matrix.toggle_connection("lfo1", "cutoff");
        assert!(!state2);
        assert!(!matrix.is_connected("lfo1", "cutoff"));
    }

    #[test]
    fn test_signal_flow_lookup_and_intensity() {
        let mut matrix = PatchMatrixView::new();
        matrix.add_source(SourceNode::new("lfo1", "LFO 1", SourceKind::Lfo));
        matrix.add_destination(DestNode::new("cutoff", "Filter Cutoff", "Filter"));

        matrix.update_source_signal("lfo1", 0.8);
        matrix.connect("lfo1", "cutoff", 0.5);

        assert_eq!(matrix.get_intensity("lfo1", "cutoff"), 0.5);
        let flow = matrix.get_signal_flow("lfo1", "cutoff");
        assert!(flow.is_some());
        let val = flow.unwrap();
        assert!((val - 0.4).abs() < 1e-4);

        let routes = matrix.get_routes_for_source("lfo1");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].0, "cutoff");
        assert_eq!(routes[0].1, 0.5);

        let dest_routes = matrix.get_routes_for_dest("cutoff");
        assert_eq!(dest_routes.len(), 1);
        assert_eq!(dest_routes[0].0, "lfo1");
    }

    #[test]
    fn test_active_routes_summary() {
        let matrix = PatchMatrixView::with_default_nodes();
        let routes = matrix.get_active_routes();
        assert!(routes.len() >= 3);
        assert!(routes
            .iter()
            .any(|r| r.source_id == "lfo1" && r.dest_id == "cutoff"));
    }

    #[test]
    fn test_hit_target_bounds() {
        let matrix = PatchMatrixView::default();
        assert!(matrix.min_hit_target_size >= 44.0);
    }
}
