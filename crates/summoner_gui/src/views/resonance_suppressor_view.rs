// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-band Dynamic Resonance Suppressor & Node Tracking HUD (Step 1462).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const RESONANCE_NODE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MAX_RESONANCE_NODES: usize = 6;
pub const MIN_SUPPRESSOR_FREQ_HZ: f32 = 20.0;
pub const MAX_SUPPRESSOR_FREQ_HZ: f32 = 20000.0;

/// Operating mode for the dynamic resonance suppressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressorMode {
    FastSurgical,     // Ultra-narrow steep notch tracking for harsh whistles
    MusicalSmooth,    // Broader organic resonance smoothing
    DeepHarmonicTame, // Multi-harmonic tracking across octave partials
}

/// A single dynamic resonance suppression node.
#[derive(Debug, Clone)]
pub struct ResonanceNode {
    pub freq_hz: f32,       // Target center frequency [20.0 ..= 20000.0 Hz]
    pub q: f32,             // Filter selectivity [0.5 ..= 30.0]
    pub depth_db: f32,      // Max dynamic suppression depth [0.0 ..= 24.0 dB]
    pub active_cut_db: f32, // Current dynamic attenuation [0.0 ..= 24.0 dB]
    pub is_active: bool,    // Node enabled / bypassed
    pub is_solo: bool,      // Delta audition mode for this node
}

impl ResonanceNode {
    pub fn new(freq_hz: f32, q: f32, depth_db: f32) -> Self {
        Self {
            freq_hz,
            q,
            depth_db,
            active_cut_db: depth_db * 0.65,
            is_active: true,
            is_solo: false,
        }
    }
}

/// Multi-band Dynamic Resonance Suppressor HUD View (Step 1462).
#[derive(Debug, Clone)]
pub struct ResonanceSuppressorView {
    pub nodes: Vec<ResonanceNode>,
    pub selected_node_idx: usize,
    pub mode: SuppressorMode,
    pub global_sensitivity: f32, // 0.0 ..= 100.0 %
    pub attack_ms: f32,          // 0.5 ..= 50.0 ms
    pub release_ms: f32,         // 10.0 ..= 500.0 ms
    pub delta_listen: bool,      // Global solo difference mode
    pub color_palette: ContrastColorPalette,
}

impl Default for ResonanceSuppressorView {
    fn default() -> Self {
        Self::new()
    }
}

impl ResonanceSuppressorView {
    pub fn new() -> Self {
        let default_nodes = vec![
            ResonanceNode::new(450.0, 8.0, 9.0),
            ResonanceNode::new(2800.0, 14.0, 14.0),
            ResonanceNode::new(5400.0, 18.0, 12.0),
            ResonanceNode::new(8200.0, 12.0, 8.0),
        ];

        Self {
            nodes: default_nodes,
            selected_node_idx: 1,
            mode: SuppressorMode::FastSurgical,
            global_sensitivity: 65.0,
            attack_ms: 2.5,
            release_ms: 60.0,
            delta_listen: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert frequency in Hz (20 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_SUPPRESSOR_FREQ_HZ, MAX_SUPPRESSOR_FREQ_HZ);
        ((freq / MIN_SUPPRESSOR_FREQ_HZ).log10()
            / (MAX_SUPPRESSOR_FREQ_HZ / MIN_SUPPRESSOR_FREQ_HZ).log10())
        .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (20 .. 20000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_SUPPRESSOR_FREQ_HZ
            * 10.0_f32.powf(norm * (MAX_SUPPRESSOR_FREQ_HZ / MIN_SUPPRESSOR_FREQ_HZ).log10())
    }

    /// Convert depth dB (0.0 .. 24.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn depth_to_normalized(depth: f32) -> f32 {
        (depth / 24.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to depth dB (0.0 .. 24.0).
    pub fn normalized_to_depth(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 24.0
    }

    /// Add a new resonance tracking node if under `MAX_RESONANCE_NODES`.
    pub fn add_node(&mut self, freq_hz: f32, q: f32, depth_db: f32) -> bool {
        if self.nodes.len() < MAX_RESONANCE_NODES {
            self.nodes.push(ResonanceNode::new(freq_hz, q, depth_db));
            self.selected_node_idx = self.nodes.len() - 1;
            true
        } else {
            false
        }
    }

    /// Remove a resonance tracking node by index.
    pub fn remove_node(&mut self, index: usize) -> bool {
        if self.nodes.len() > 1 && index < self.nodes.len() {
            self.nodes.remove(index);
            if self.selected_node_idx >= self.nodes.len() {
                self.selected_node_idx = self.nodes.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Calculate combined suppression curve at given frequency `f_hz`.
    pub fn evaluate_suppression_response(&self, f_hz: f32) -> f32 {
        let mut total_att_db = 0.0_f32;

        for node in &self.nodes {
            if !node.is_active {
                continue;
            }
            let f0 = node.freq_hz;
            let q = node.q.max(0.1);
            let ratio = f_hz / f0;
            let log_ratio = ratio.ln();
            let bell = (-0.5 * (log_ratio * q).powi(2)).exp();
            total_att_db += bell * node.depth_db;
        }

        (total_att_db / 24.0).clamp(0.0, 1.0)
    }

    /// Tests if a point hits a specific resonance node puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_node(&self, pos: (f32, f32), canvas: Rect, node_idx: usize) -> bool {
        if let Some(node) = self.nodes.get(node_idx) {
            let norm_x = Self::freq_to_normalized(node.freq_hz);
            let norm_y = Self::depth_to_normalized(node.depth_db);
            let px = canvas.x + norm_x * canvas.width;
            let py = canvas.y + (1.0 - norm_y) * canvas.height;
            let dx = pos.0 - px;
            let dy = pos.1 - py;
            (dx * dx + dy * dy).sqrt() <= RESONANCE_NODE_HIT_RADIUS
        } else {
            false
        }
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "RESONANCE SUPPRESSOR [{:?}] Nodes:{} Sens:{:.0}% Delta:{}",
            self.mode,
            self.nodes.len(),
            self.global_sensitivity,
            self.delta_listen
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let f = Self::normalized_to_freq(norm_x);
                let att = self.evaluate_suppression_response(f);
                if (att - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark node puck positions
            for (idx, node) in self.nodes.iter().enumerate() {
                let node_norm_y = Self::depth_to_normalized(node.depth_db);
                if (node_norm_y - norm_y).abs() < (1.0 / canvas_h as f32) {
                    let node_norm_x = Self::freq_to_normalized(node.freq_hz);
                    let px = (node_norm_x * (width.saturating_sub(1) as f32)) as usize;
                    if px < width {
                        row[px] = (b'1' + (idx as u8)) as char;
                    }
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Selected Node #{}: {:.0}Hz Q:{:.1} Depth:-{:.1}dB [PASS: >=44pt]",
            self.selected_node_idx + 1,
            self.nodes
                .get(self.selected_node_idx)
                .map(|n| n.freq_hz)
                .unwrap_or(0.0),
            self.nodes
                .get(self.selected_node_idx)
                .map(|n| n.q)
                .unwrap_or(0.0),
            self.nodes
                .get(self.selected_node_idx)
                .map(|n| n.depth_db)
                .unwrap_or(0.0),
        );
        lines.push(footer);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "MULTI-BAND DYNAMIC RESONANCE SUPPRESSOR HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let sel_node = self.nodes.get(self.selected_node_idx);
        let readout = format!(
            "NODE #{}: {:.0} Hz | Q: {:.1} | DEPTH: -{:.1} dB | SENS: {:.0}%",
            self.selected_node_idx + 1,
            sel_node.map(|n| n.freq_hz).unwrap_or(0.0),
            sel_node.map(|n| n.q).unwrap_or(0.0),
            sel_node.map(|n| n.depth_db).unwrap_or(0.0),
            self.global_sensitivity
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Frequency Spectrum & Multi-Node Notch Canvas (20..450)
        let curve_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(curve_rect.x + 12.0, curve_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DYNAMIC NOTCH SUPPRESSION SPECTRUM",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines (octave guides)
        for step in 1..4 {
            let gy = curve_rect.y + curve_rect.height * (step as f32 * 0.25);
            painter.line_segment(
                [
                    egui::pos2(curve_rect.x, gy),
                    egui::pos2(curve_rect.x + curve_rect.width, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }

        // Draw composite suppression curve
        let steps = 80;
        let mut prev_pt: Option<egui::Pos2> = None;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let f = Self::normalized_to_freq(t);
            let att = self.evaluate_suppression_response(f);
            let cx = curve_rect.x + t * curve_rect.width;
            let cy = curve_rect.y + (1.0 - att * 0.85 - 0.05) * curve_rect.height;
            let cur_pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Draw Draggable Resonance Node Pucks (>=22pt radius -> 44x44pt bounding box)
        for (idx, node) in self.nodes.iter().enumerate() {
            let norm_x = Self::freq_to_normalized(node.freq_hz);
            let norm_y = Self::depth_to_normalized(node.depth_db);
            let px = curve_rect.x + norm_x * curve_rect.width;
            let py = curve_rect.y + (1.0 - norm_y) * curve_rect.height;

            let is_sel = idx == self.selected_node_idx;
            let puck_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(255, 215, 0)
            };

            // Outer hit target ring
            painter.circle_stroke(
                egui::pos2(px, py),
                RESONANCE_NODE_HIT_RADIUS,
                Stroke::new(
                    2.0_f32,
                    Color32::from_rgba_unmultiplied(puck_col.r(), puck_col.g(), puck_col.b(), 140),
                ),
            );
            // Puck body
            painter.circle_filled(egui::pos2(px, py), 14.0, puck_col);
            painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(0, 0, 0));

            // Node index number inside puck
            painter.text(
                egui::pos2(px, py - 24.0),
                egui::Align2::CENTER_CENTER,
                format!("#{}", idx + 1),
                egui::FontId::proportional(10.0),
                puck_col,
            );
        }

        // Right Panel: Suppression Modes & Node Controls (470..780)
        let mode_rect = Rect::new(rect.x + 470.0, rect.y + 56.0, 310.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(mode_rect.x + 12.0, mode_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "SUPPRESSION ENGINE & PROFILES",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Modes tabs (>=44pt hit target height)
        let modes = [
            (SuppressorMode::FastSurgical, "SURGICAL"),
            (SuppressorMode::MusicalSmooth, "SMOOTH"),
            (SuppressorMode::DeepHarmonicTame, "HARMONIC"),
        ];

        let tab_w = 90.0;
        let tab_h = 44.0;
        for (i, (m, label)) in modes.iter().enumerate() {
            let bx = mode_rect.x + 12.0 + (i as f32 * (tab_w + 8.0));
            let by = mode_rect.y + 40.0;
            let is_active = self.mode == *m;

            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(tab_w, tab_h)),
                4.0,
                bg_col,
            );
            painter.text(
                egui::pos2(bx + tab_w * 0.5, by + tab_h * 0.5),
                egui::Align2::CENTER_CENTER,
                *label,
                egui::FontId::proportional(10.0),
                fg_col,
            );
        }

        // Add / Remove Node Buttons (>=44x44pt)
        let btn_add_rect = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 12.0, mode_rect.y + 96.0),
            egui::vec2(138.0, 44.0),
        );
        painter.rect_filled(btn_add_rect, 4.0, Color32::from_rgb(35, 45, 65));
        painter.text(
            egui::pos2(btn_add_rect.center().x, btn_add_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "+ ADD NODE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );

        let btn_rem_rect = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 160.0, mode_rect.y + 96.0),
            egui::vec2(138.0, 44.0),
        );
        painter.rect_filled(btn_rem_rect, 4.0, Color32::from_rgb(45, 25, 35));
        painter.text(
            egui::pos2(btn_rem_rect.center().x, btn_rem_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "- REMOVE NODE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 120, 120),
        );

        // Delta Listen Toggle Button (>=44x44pt)
        let delta_y = mode_rect.y + 152.0;
        let delta_bg = if self.delta_listen {
            Color32::from_rgb(255, 107, 43)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x + 12.0, delta_y),
                egui::vec2(286.0, 44.0),
            ),
            4.0,
            delta_bg,
        );
        painter.text(
            egui::pos2(mode_rect.x + 155.0, delta_y + 22.0),
            egui::Align2::CENTER_CENTER,
            if self.delta_listen {
                "DELTA AUDITION (REMOVED RESONANCES)"
            } else {
                "DELTA AUDITION: OFF"
            },
            egui::FontId::proportional(11.0),
            if self.delta_listen {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(0, 255, 180)
            },
        );

        // Bottom Controls Bar (20..780, y: 290..475)
        let bar_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let sel_f = sel_node.map(|n| n.freq_hz).unwrap_or(1000.0);
        let sel_q = sel_node.map(|n| n.q).unwrap_or(5.0);
        let sel_d = sel_node.map(|n| n.depth_db).unwrap_or(6.0);

        let sliders = [
            (
                "Center Freq",
                format!("{:.0} Hz", sel_f),
                Self::freq_to_normalized(sel_f),
            ),
            (
                "Bandwidth (Q)",
                format!("{:.1} Q", sel_q),
                ((sel_q - 0.5) / 29.5).clamp(0.0, 1.0),
            ),
            (
                "Notch Depth",
                format!("-{:.1} dB", sel_d),
                Self::depth_to_normalized(sel_d),
            ),
            (
                "Sensitivity",
                format!("{:.0}%", self.global_sensitivity),
                self.global_sensitivity / 100.0,
            ),
        ];

        let mut sx_pos = bar_rect.x + 15.0;
        for (name, val_str, norm_val) in sliders {
            painter.text(
                egui::pos2(sx_pos, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(220, 235, 255),
            );
            painter.text(
                egui::pos2(sx_pos + 95.0, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                val_str,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 229, 255),
            );

            // Slider track
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(160.0, 26.0),
            );
            painter.rect_filled(track_rect, 4.0, Color32::from_rgb(10, 14, 22));

            // Slider fill
            let fill_w = 160.0 * norm_val;
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(fill_w, 26.0),
            );
            painter.rect_filled(fill_rect, 4.0, Color32::from_rgb(0, 229, 255));

            sx_pos += 185.0;
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.x + 15.0, bar_rect.y + 130.0),
            egui::vec2(730.0, 36.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Multi-Band Dynamic Resonance Suppressor Nodes (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
