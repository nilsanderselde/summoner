// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Tactile Automation Lane Bezier Curve Point Editor with Pinch-to-Zoom Scaling (Step 1365).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const NODE_VISUAL_RADIUS: f32 = 12.0;
pub const NODE_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target
pub const HANDLE_VISUAL_RADIUS: f32 = 8.0;
pub const HANDLE_HIT_RADIUS: f32 = 22.0;

/// Curve interpolation type between automation points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AutomationCurveType {
    #[default]
    Linear,
    Exponential {
        tension: f32,
    }, // -1.0 to +1.0
    Bezier {
        handle_out_y: f32,
        handle_in_y: f32,
    },
    Hold,
}

/// A single node point in the automation lane.
#[derive(Debug, Clone, PartialEq)]
pub struct AutomationNode {
    pub id: String,
    pub time_beats: f64, // Beat position [0.0 ..= total_beats]
    pub value: f32,      // Normalized parameter value [0.0 ..= 1.0]
    pub curve: AutomationCurveType,
    pub is_selected: bool,
}

impl AutomationNode {
    pub fn new(
        id: impl Into<String>,
        time_beats: f64,
        value: f32,
        curve: AutomationCurveType,
    ) -> Self {
        Self {
            id: id.into(),
            time_beats: time_beats.max(0.0),
            value: value.clamp(0.0, 1.0),
            curve,
            is_selected: false,
        }
    }
}

/// Tactile Automation Lane Bezier Curve Point Editor View (Step 1365).
#[derive(Debug, Clone)]
pub struct BezierAutomationEditorView {
    pub parameter_name: String,
    pub unit_name: String,
    pub min_display: f32,
    pub max_display: f32,
    pub total_beats: f64,
    pub zoom_x: f32, // 1.0 ..= 32.0x
    pub zoom_y: f32, // 1.0 ..= 4.0x
    pub scroll_x_beats: f64,
    pub nodes: Vec<AutomationNode>,
    pub selected_node_idx: Option<usize>,
    pub dragging_node_idx: Option<usize>,
    pub grid_snap_beats: f64, // 0.25 = 1/16th note
    pub color_palette: ContrastColorPalette,
}

impl Default for BezierAutomationEditorView {
    fn default() -> Self {
        Self::new("Track 1 - Filter Cutoff", "Hz", 20.0, 20000.0, 16.0)
    }
}

impl BezierAutomationEditorView {
    pub fn new(
        param_name: impl Into<String>,
        unit: impl Into<String>,
        min_disp: f32,
        max_disp: f32,
        total_beats: f64,
    ) -> Self {
        let mut view = Self {
            parameter_name: param_name.into(),
            unit_name: unit.into(),
            min_display: min_disp,
            max_display: max_disp,
            total_beats: total_beats.max(4.0),
            zoom_x: 1.0_f32,
            zoom_y: 1.0_f32,
            scroll_x_beats: 0.0,
            nodes: Vec::new(),
            selected_node_idx: None,
            dragging_node_idx: None,
            grid_snap_beats: 0.25,
            color_palette: ContrastColorPalette::default(),
        };

        // Populate initial automation curve
        view.nodes.push(AutomationNode::new(
            "n0",
            0.0,
            0.20_f32,
            AutomationCurveType::Exponential { tension: 0.5_f32 },
        ));
        view.nodes.push(AutomationNode::new(
            "n1",
            4.0,
            0.85_f32,
            AutomationCurveType::Bezier {
                handle_out_y: 0.95_f32,
                handle_in_y: 0.65_f32,
            },
        ));
        view.nodes.push(AutomationNode::new(
            "n2",
            8.0,
            0.35_f32,
            AutomationCurveType::Exponential { tension: -0.4_f32 },
        ));
        view.nodes.push(AutomationNode::new(
            "n3",
            12.0,
            0.90_f32,
            AutomationCurveType::Linear,
        ));
        view.nodes.push(AutomationNode::new(
            "n4",
            16.0,
            0.10_f32,
            AutomationCurveType::Hold,
        ));

        view
    }

    /// Calculate visible beat range [start_beat, end_beat].
    pub fn visible_beat_range(&self) -> (f64, f64) {
        let visible_duration = self.total_beats / self.zoom_x.max(1.0_f32) as f64;
        let start = self
            .scroll_x_beats
            .clamp(0.0, (self.total_beats - visible_duration).max(0.0));
        let end = (start + visible_duration).min(self.total_beats);
        (start, end)
    }

    /// Map (time_beats, value) to screen coordinate in points.
    pub fn time_value_to_screen(&self, time: f64, val: f32, canvas: Rect) -> (f32, f32) {
        let (start, end) = self.visible_beat_range();
        let range = (end - start).max(0.001);
        let norm_x = ((time - start) / range).clamp(0.0, 1.0) as f32;
        let norm_y = val.clamp(0.0_f32, 1.0_f32);

        let sx = canvas.x + norm_x * canvas.width;
        let sy = canvas.y + (1.0_f32 - norm_y) * canvas.height;
        (sx, sy)
    }

    /// Map screen coordinate in points to (time_beats, value).
    pub fn screen_to_time_value(&self, screen_pos: (f32, f32), canvas: Rect) -> (f64, f32) {
        let (start, end) = self.visible_beat_range();
        let range = (end - start).max(0.001);

        let norm_x = ((screen_pos.0 - canvas.x) / canvas.width.max(1.0_f32)).clamp(0.0, 1.0) as f64;
        let norm_y = (1.0_f32 - (screen_pos.1 - canvas.y) / canvas.height.max(1.0_f32))
            .clamp(0.0_f32, 1.0_f32);

        let time = start + norm_x * range;
        (time, norm_y)
    }

    /// Snap beat position to current grid resolution.
    pub fn snap_beat(&self, time: f64) -> f64 {
        if self.grid_snap_beats <= 0.0 {
            return time.clamp(0.0, self.total_beats);
        }
        let half = self.grid_snap_beats * 0.5;
        let snapped = ((time + half) / self.grid_snap_beats).floor() * self.grid_snap_beats;
        snapped.clamp(0.0, self.total_beats)
    }

    /// Hit test node points with >=44x44pt bounding touch target.
    pub fn hit_test_node(&self, screen_pos: (f32, f32), canvas: Rect) -> Option<usize> {
        for (idx, node) in self.nodes.iter().enumerate() {
            let (nx, ny) = self.time_value_to_screen(node.time_beats, node.value, canvas);
            let dx = screen_pos.0 - nx;
            let dy = screen_pos.1 - ny;
            if (dx * dx + dy * dy).sqrt() <= NODE_HIT_RADIUS {
                return Some(idx);
            }
        }
        None
    }

    /// Evaluate automation curve at given time beat.
    pub fn evaluate_curve_at(&self, time: f64) -> f32 {
        if self.nodes.is_empty() {
            return 0.0_f32;
        }
        if time <= self.nodes[0].time_beats {
            return self.nodes[0].value;
        }
        if time >= self.nodes.last().unwrap().time_beats {
            return self.nodes.last().unwrap().value;
        }

        // Find surrounding segment [n_prev, n_next]
        for i in 0..self.nodes.len() - 1 {
            let n0 = &self.nodes[i];
            let n1 = &self.nodes[i + 1];
            if time >= n0.time_beats && time <= n1.time_beats {
                let dt = (n1.time_beats - n0.time_beats).max(0.0001);
                let t = ((time - n0.time_beats) / dt) as f32; // [0.0 ..= 1.0]

                return match n0.curve {
                    AutomationCurveType::Linear => n0.value + t * (n1.value - n0.value),
                    AutomationCurveType::Exponential { tension } => {
                        let sign = if tension >= 0.0_f32 {
                            1.0_f32
                        } else {
                            -1.0_f32
                        };
                        let exponent = 1.0_f32 + tension.abs() * 4.0_f32;
                        let shaped_t = if sign > 0.0_f32 {
                            t.powf(exponent)
                        } else {
                            1.0_f32 - (1.0_f32 - t).powf(exponent)
                        };
                        n0.value + shaped_t * (n1.value - n0.value)
                    }
                    AutomationCurveType::Bezier {
                        handle_out_y,
                        handle_in_y,
                    } => {
                        // Cubic Bezier interpolation: P0=n0.value, P1=handle_out_y, P2=handle_in_y, P3=n1.value
                        let p0 = n0.value;
                        let p1 = handle_out_y;
                        let p2 = handle_in_y;
                        let p3 = n1.value;
                        let inv_t = 1.0_f32 - t;
                        inv_t.powi(3) * p0
                            + 3.0_f32 * inv_t.powi(2) * t * p1
                            + 3.0_f32 * inv_t * t.powi(2) * p2
                            + t.powi(3) * p3
                    }
                    AutomationCurveType::Hold => n0.value,
                };
            }
        }
        self.nodes.last().unwrap().value
    }

    /// Insert a new node at given time and value.
    pub fn insert_node(&mut self, time: f64, val: f32) -> usize {
        let snapped_time = self.snap_beat(time);
        let id = format!("n_{}_{}", snapped_time, self.nodes.len());
        let node = AutomationNode::new(id, snapped_time, val, AutomationCurveType::Linear);
        self.nodes.push(node);
        self.nodes
            .sort_by(|a, b| a.time_beats.partial_cmp(&b.time_beats).unwrap());
        self.nodes
            .iter()
            .position(|n| (n.time_beats - snapped_time).abs() < 0.001)
            .unwrap_or(0)
    }

    /// Delete node at index (preserves first and last bounds).
    pub fn delete_node(&mut self, idx: usize) -> bool {
        if self.nodes.len() > 2 && idx < self.nodes.len() {
            self.nodes.remove(idx);
            if self.selected_node_idx == Some(idx) {
                self.selected_node_idx = None;
            }
            true
        } else {
            false
        }
    }

    /// Apply pinch-to-zoom scaling factors.
    pub fn apply_pinch_zoom(&mut self, factor_x: f32, factor_y: f32) {
        self.zoom_x = (self.zoom_x * factor_x).clamp(1.0_f32, 32.0_f32);
        self.zoom_y = (self.zoom_y * factor_y).clamp(1.0_f32, 4.0_f32);
    }

    /// Formatted display value with unit for node.
    pub fn display_value_string(&self, val: f32) -> String {
        let disp = self.min_display + val * (self.max_display - self.min_display);
        format!("{:.1} {}", disp, self.unit_name)
    }

    /// Generate deterministic ASCII representation for verification.
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let mut grid = vec![vec![' '; width]; height];
        for col in 0..width {
            let t = (col as f64 / (width.saturating_sub(1)).max(1) as f64) * self.total_beats;
            let val = self.evaluate_curve_at(t);
            let row = ((1.0_f32 - val) * (height - 1) as f32).round() as usize;
            let row = row.min(height - 1);
            if let Some(r) = grid.get_mut(row) {
                if let Some(c) = r.get_mut(col) {
                    *c = '*';
                }
            }
        }
        for node in &self.nodes {
            let norm_x = (node.time_beats / self.total_beats).clamp(0.0, 1.0) as f32;
            let col = ((norm_x * (width - 1) as f32).round() as usize).min(width - 1);
            let row = ((1.0_f32 - node.value) * (height - 1) as f32).round() as usize;
            let row = row.min(height - 1);
            grid[row][col] = 'O';
        }
        let mut out = String::new();
        for r in grid {
            out.push_str(&r.into_iter().collect::<String>());
            out.push('\n');
        }
        out
    }
}

#[cfg(feature = "gui")]
impl BezierAutomationEditorView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("BEZIER & AUTOMATION LANE EDITOR")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(&self.parameter_name)
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(format!("Zoom X: {:.1}x", self.zoom_x));
            });

            // 2. Automation Curve Canvas
            let canvas_w = ui.available_width().max(650.0_f32);
            let canvas_h = 240.0_f32;

            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());
            let canvas_rect = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Canvas Background
            painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(40, 55, 80)),
            );

            // Value Grid Lines (0%, 25%, 50%, 75%, 100%)
            for g in 0..=4 {
                let norm_y = g as f32 * 0.25_f32;
                let y = response.rect.max.y - norm_y * canvas_h;
                painter.line_segment(
                    [
                        egui::pos2(response.rect.min.x, y),
                        egui::pos2(response.rect.max.x, y),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 70, 95, 70)),
                );
                painter.text(
                    egui::pos2(response.rect.min.x + 8.0_f32, y - 4.0_f32),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{:.0}%", norm_y * 100.0_f32),
                    egui::FontId::proportional(9.0_f32),
                    Color32::from_rgb(120, 145, 175),
                );
            }

            // Handle Touch / Drag Interactions
            if let Some(pos) = response.hover_pos() {
                let pt = (pos.x, pos.y);
                if response.drag_started() {
                    if let Some(idx) = self.hit_test_node(pt, canvas_rect) {
                        self.dragging_node_idx = Some(idx);
                        self.selected_node_idx = Some(idx);
                    }
                }
            }

            if response.dragged() {
                if let (Some(drag_idx), Some(pos)) = (self.dragging_node_idx, response.hover_pos())
                {
                    let (raw_time, val) = self.screen_to_time_value((pos.x, pos.y), canvas_rect);
                    let snapped_time = self.snap_beat(raw_time);
                    if drag_idx < self.nodes.len() {
                        self.nodes[drag_idx].time_beats = snapped_time;
                        self.nodes[drag_idx].value = val;
                    }
                }
            }

            if response.drag_stopped() {
                self.dragging_node_idx = None;
            }

            // Draw Continuous Evaluated Curve Path (100 sample segments)
            let num_segments = 100;
            let (start_beat, end_beat) = self.visible_beat_range();
            let duration = (end_beat - start_beat).max(0.001);

            let mut prev_pt = None;
            for s in 0..=num_segments {
                let beat = start_beat + (s as f64 / num_segments as f64) * duration;
                let val = self.evaluate_curve_at(beat);
                let (sx, sy) = self.time_value_to_screen(beat, val, canvas_rect);
                let current_pt = egui::pos2(sx, sy);

                if let Some(p) = prev_pt {
                    painter.line_segment(
                        [p, current_pt],
                        Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                    );
                }
                prev_pt = Some(current_pt);
            }

            // Draw Nodes and Handles (>=44x44pt Hit Target Bounds)
            for (idx, node) in self.nodes.iter().enumerate() {
                let (nx, ny) = self.time_value_to_screen(node.time_beats, node.value, canvas_rect);
                let center_pos = egui::pos2(nx, ny);
                let is_selected = self.selected_node_idx == Some(idx);

                // Node touch bounding indicator if selected
                if is_selected {
                    let touch_box = egui::Rect::from_center_size(
                        center_pos,
                        Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT),
                    );
                    painter.rect_stroke(
                        touch_box,
                        4.0_f32,
                        Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)),
                    );
                }

                // Node body
                let node_col = if is_selected {
                    Color32::from_rgb(255, 215, 0)
                } else {
                    Color32::from_rgb(0, 255, 180)
                };

                painter.circle_filled(center_pos, NODE_VISUAL_RADIUS, node_col);
                painter.circle_stroke(
                    center_pos,
                    NODE_VISUAL_RADIUS,
                    Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)),
                );
            }

            ui.add_space(10.0_f32);

            // 3. Node Inspector & Actions Panel
            let mut delete_node_idx = None;
            let mut change_curve = None;

            if let Some(idx) = self.selected_node_idx {
                if let Some(node) = self.nodes.get(idx) {
                    let node_time = node.time_beats;
                    let node_val = node.value;
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "NODE #{}: Beat {:.2} | Value: {:.1}%",
                                    idx,
                                    node_time,
                                    node_val * 100.0_f32
                                ))
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                            );
                            ui.separator();

                            if ui.button("Linear").clicked() {
                                change_curve = Some((idx, AutomationCurveType::Linear));
                            }
                            if ui.button("Exponential").clicked() {
                                change_curve = Some((
                                    idx,
                                    AutomationCurveType::Exponential { tension: 0.5_f32 },
                                ));
                            }
                            if ui.button("Bezier").clicked() {
                                change_curve = Some((
                                    idx,
                                    AutomationCurveType::Bezier {
                                        handle_out_y: (node_val + 0.1_f32).min(1.0_f32),
                                        handle_in_y: (node_val - 0.1_f32).max(0.0_f32),
                                    },
                                ));
                            }
                            if ui.button("Hold").clicked() {
                                change_curve = Some((idx, AutomationCurveType::Hold));
                            }

                            ui.separator();
                            if ui.button("Delete Node").clicked() {
                                delete_node_idx = Some(idx);
                            }
                        });
                    });
                }
            }

            if let Some((idx, new_type)) = change_curve {
                if idx < self.nodes.len() {
                    self.nodes[idx].curve = new_type;
                }
            }
            if let Some(idx) = delete_node_idx {
                self.delete_node(idx);
            }
        });
    }
}
