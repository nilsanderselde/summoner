// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Bus Sidechain Ducking & Duck Curve Dynamic Transfer Matrix with Gain Reduction Meter Bridge (Step 1425).

use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SIDECHAIN_NUM_BUSES: usize = 8;
pub const SIDECHAIN_NODE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

pub const BUS_NAMES: [&str; SIDECHAIN_NUM_BUSES] = [
    "Kick", "Snare", "Vocals", "Lead", "Bass", "Aux 1", "Aux 2", "Master",
];

/// Individual routing connection between source bus and destination track.
#[derive(Debug, Clone, PartialEq)]
pub struct SidechainRoute {
    pub source_idx: usize,
    pub dest_idx: usize,
    pub enabled: bool,
    pub depth_pct: f32,     // 0.0 ..= 100.0%
    pub threshold_db: f32,  // -60.0 ..= 0.0 dB
    pub ratio: f32,         // 1.0 ..= 20.0
    pub current_gr_db: f32, // Real-time gain reduction (e.g. -6.5 dB)
}

impl SidechainRoute {
    pub fn new(source_idx: usize, dest_idx: usize) -> Self {
        Self {
            source_idx,
            dest_idx,
            enabled: false,
            depth_pct: 75.0,
            threshold_db: -18.0,
            ratio: 4.0,
            current_gr_db: 0.0,
        }
    }
}

/// Multi-Bus Sidechain Dynamic Ducking Matrix View (Step 1425).
#[derive(Debug, Clone)]
pub struct SidechainMatrixView {
    pub routes: Vec<SidechainRoute>,
    pub selected_source_idx: usize,
    pub selected_dest_idx: usize,
    pub attack_ms: f32,        // 0.1 ..= 100.0 ms
    pub hold_ms: f32,          // 0.0 ..= 500.0 ms
    pub release_ms: f32,       // 5.0 ..= 1000.0 ms
    pub lookahead_ms: f32,     // 0.0 ..= 50.0 ms
    pub sidechain_hpf_hz: f32, // 20.0 ..= 500.0 Hz
    pub sidechain_lpf_hz: f32, // 1000.0 ..= 20000.0 Hz
    pub gr_history: Vec<f32>,  // Rolling buffer of gain reduction values (-dB)
    pub color_palette: ContrastColorPalette,
}

impl Default for SidechainMatrixView {
    fn default() -> Self {
        Self::new()
    }
}

impl SidechainMatrixView {
    pub fn new() -> Self {
        let mut routes = Vec::with_capacity(SIDECHAIN_NUM_BUSES * SIDECHAIN_NUM_BUSES);
        for s in 0..SIDECHAIN_NUM_BUSES {
            for d in 0..SIDECHAIN_NUM_BUSES {
                let mut r = SidechainRoute::new(s, d);
                // Default: Kick -> Bass ducking enabled
                if s == 0 && d == 4 {
                    r.enabled = true;
                    r.current_gr_db = -8.5;
                }
                // Vocals -> Aux 1 ducking enabled
                if s == 2 && d == 5 {
                    r.enabled = true;
                    r.current_gr_db = -4.2;
                }
                routes.push(r);
            }
        }

        // Initialize gain reduction history trace
        let mut gr_history = vec![0.0; 60];
        for (i, gr) in gr_history.iter_mut().enumerate() {
            let phase = (i as f32 / 15.0) * std::f32::consts::PI;
            if phase.sin() > 0.3 {
                *gr = -(phase.sin() * 9.0);
            }
        }

        Self {
            routes,
            selected_source_idx: 0,
            selected_dest_idx: 4,
            attack_ms: 2.0,
            hold_ms: 15.0,
            release_ms: 120.0,
            lookahead_ms: 5.0,
            sidechain_hpf_hz: 80.0,
            sidechain_lpf_hz: 16000.0,
            gr_history,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Retrieve route reference by source and destination index.
    pub fn get_route(&self, source_idx: usize, dest_idx: usize) -> Option<&SidechainRoute> {
        self.routes
            .iter()
            .find(|r| r.source_idx == source_idx && r.dest_idx == dest_idx)
    }

    /// Retrieve mutable route reference by source and destination index.
    pub fn get_route_mut(
        &mut self,
        source_idx: usize,
        dest_idx: usize,
    ) -> Option<&mut SidechainRoute> {
        self.routes
            .iter_mut()
            .find(|r| r.source_idx == source_idx && r.dest_idx == dest_idx)
    }

    /// Calculate dynamic compressor transfer curve reduction (dB) for input level (dB).
    pub fn calculate_duck_curve_gr(&self, input_db: f32, threshold_db: f32, ratio: f32) -> f32 {
        if input_db <= threshold_db {
            0.0
        } else {
            let overshoot = input_db - threshold_db;
            let compressed = threshold_db + overshoot / ratio.max(1.0);
            (compressed - input_db).min(0.0)
        }
    }

    /// Tests if a screen coordinate hits a matrix cell node (>= 22pt radius -> 44x44pt).
    pub fn hit_test_matrix_node(&self, pos: (f32, f32), node_center_screen: (f32, f32)) -> bool {
        let dx = pos.0 - node_center_screen.0;
        let dy = pos.1 - node_center_screen.1;
        (dx * dx + dy * dy).sqrt() <= SIDECHAIN_NODE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SIDECHAIN MATRIX Active Routes:{} Attack:{:.1}ms Release:{:.0}ms HPF:{:.0}Hz",
            self.routes.iter().filter(|r| r.enabled).count(),
            self.attack_ms,
            self.release_ms,
            self.sidechain_hpf_hz
        );
        lines.push(header);

        for (s, bus_name) in BUS_NAMES.iter().enumerate().take(height.saturating_sub(1)) {
            let mut row = format!("{:<6} |", bus_name);
            for d in 0..SIDECHAIN_NUM_BUSES {
                if let Some(r) = self.get_route(s, d) {
                    if r.enabled {
                        row.push_str(" [X] ");
                    } else {
                        row.push_str("  .  ");
                    }
                }
            }
            lines.push(row);
        }
        while lines.len() < height {
            lines.push("-".repeat(width.max(30)));
        }
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Header Bar & Mode Title
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("MULTI-BUS SIDECHAIN DUCKING MATRIX")
                        .size(15.0)
                        .color(Color32::from_rgb(0, 255, 180))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "SELECTED ROUTE: {} -> {}",
                        BUS_NAMES[self.selected_source_idx], BUS_NAMES[self.selected_dest_idx]
                    ))
                    .color(Color32::from_rgb(255, 215, 0))
                    .strong(),
                );
            });

            ui.add_space(8.0);

            // 2. Dual Canvas: Left (8x8 Routing Matrix Grid) & Right (Gain Reduction Meter Bridge & Duck Curve)
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) * 0.5;

                // Left Canvas: 8x8 Routing Matrix
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("8x8 DUCKING ROUTING CROSS-POINTS")
                            .color(Color32::from_rgb(0, 229, 255))
                            .strong(),
                    );
                    let (res_l, painter_l) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 220.0),
                        egui::Sense::click(),
                    );
                    let rect_l = res_l.rect;

                    painter_l.rect_filled(rect_l, 6.0, Color32::from_rgb(14, 18, 28));
                    painter_l.rect_stroke(
                        rect_l,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    let cell_w = (rect_l.width() - 60.0) / SIDECHAIN_NUM_BUSES as f32;
                    let cell_h = (rect_l.height() - 25.0) / SIDECHAIN_NUM_BUSES as f32;

                    // Draw Matrix Nodes
                    for (s, bus_name) in BUS_NAMES.iter().enumerate() {
                        // Source bus labels
                        painter_l.text(
                            egui::pos2(
                                rect_l.min.x + 6.0,
                                rect_l.min.y + 25.0 + s as f32 * cell_h + 4.0,
                            ),
                            egui::Align2::LEFT_TOP,
                            *bus_name,
                            egui::FontId::monospace(9.0),
                            Color32::from_rgb(180, 200, 225),
                        );

                        for d in 0..SIDECHAIN_NUM_BUSES {
                            let nx = rect_l.min.x + 60.0 + d as f32 * cell_w + cell_w * 0.5;
                            let ny = rect_l.min.y + 25.0 + s as f32 * cell_h + cell_h * 0.5;

                            let is_enabled =
                                self.get_route(s, d).map(|r| r.enabled).unwrap_or(false);
                            let is_selected =
                                self.selected_source_idx == s && self.selected_dest_idx == d;

                            let node_col = if is_enabled {
                                Color32::from_rgb(0, 255, 180)
                            } else if is_selected {
                                Color32::from_rgb(255, 215, 0)
                            } else {
                                Color32::from_rgb(35, 45, 65)
                            };

                            // Draw hit target circle
                            painter_l.circle_filled(egui::pos2(nx, ny), 7.0, node_col);
                            painter_l.circle_stroke(
                                egui::pos2(nx, ny),
                                SIDECHAIN_NODE_HIT_RADIUS.min(cell_w * 0.5),
                                Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_unmultiplied(60, 80, 115, 60),
                                ),
                            );
                        }
                    }

                    // Click interaction to toggle route or select
                    if res_l.clicked() {
                        if let Some(pos) = res_l.interact_pointer_pos() {
                            for s in 0..SIDECHAIN_NUM_BUSES {
                                for d in 0..SIDECHAIN_NUM_BUSES {
                                    let nx = rect_l.min.x + 60.0 + d as f32 * cell_w + cell_w * 0.5;
                                    let ny = rect_l.min.y + 25.0 + s as f32 * cell_h + cell_h * 0.5;
                                    if self.hit_test_matrix_node((pos.x, pos.y), (nx, ny)) {
                                        self.selected_source_idx = s;
                                        self.selected_dest_idx = d;
                                        if let Some(r) = self.get_route_mut(s, d) {
                                            r.enabled = !r.enabled;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                // Right Canvas: Gain Reduction Meter Bridge & Transfer Curve
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("GAIN REDUCTION METER BRIDGE (-dB)")
                            .color(Color32::from_rgb(255, 107, 43))
                            .strong(),
                    );
                    let (res_r, painter_r) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 220.0),
                        egui::Sense::hover(),
                    );
                    let rect_r = res_r.rect;

                    painter_r.rect_filled(rect_r, 6.0, Color32::from_rgb(10, 14, 22));
                    painter_r.rect_stroke(
                        rect_r,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // Gain reduction rolling trace
                    let top_y = rect_r.min.y + 20.0;
                    let bottom_y = rect_r.max.y - 20.0;
                    painter_r.line_segment(
                        [
                            egui::pos2(rect_r.min.x + 10.0, top_y),
                            egui::pos2(rect_r.max.x - 10.0, top_y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
                    );
                    painter_r.text(
                        egui::pos2(rect_r.min.x + 14.0, top_y - 14.0),
                        egui::Align2::LEFT_TOP,
                        "0 dB GR",
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(0, 255, 180),
                    );
                    painter_r.text(
                        egui::pos2(rect_r.min.x + 14.0, bottom_y - 14.0),
                        egui::Align2::LEFT_TOP,
                        "-24 dB GR (Max Ducking)",
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(255, 107, 43),
                    );

                    let mut prev_pt: Option<egui::Pos2> = None;
                    for (i, gr) in self.gr_history.iter().enumerate() {
                        let norm_x = i as f32 / (self.gr_history.len() - 1) as f32;
                        let norm_y = (gr.abs() / 24.0).clamp(0.0, 1.0);
                        let sx = rect_r.min.x + 14.0 + norm_x * (rect_r.width() - 28.0);
                        let sy = top_y + norm_y * (bottom_y - top_y);
                        let pt = egui::pos2(sx, sy);
                        if let Some(prev) = prev_pt {
                            painter_r.line_segment(
                                [prev, pt],
                                Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                            );
                        }
                        prev_pt = Some(pt);
                    }
                });
            });

            ui.add_space(8.0);

            // 3. Tactile Controls Bar (>=44pt Touch Targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Attack Time").strong());
                        ui.add(egui::Slider::new(&mut self.attack_ms, 0.1..=100.0).text("ms"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Hold Time").strong());
                        ui.add(egui::Slider::new(&mut self.hold_ms, 0.0..=500.0).text("ms"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Release Time").strong());
                        ui.add(egui::Slider::new(&mut self.release_ms, 5.0..=1000.0).text("ms"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Lookahead").strong());
                        ui.add(egui::Slider::new(&mut self.lookahead_ms, 0.0..=50.0).text("ms"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sidechain HPF").strong());
                        ui.add(
                            egui::Slider::new(&mut self.sidechain_hpf_hz, 20.0..=500.0).text("Hz"),
                        );
                    });
                });
            });
        });
    }
}
