// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Multi-Track Spectral Frequency Paintbrush & Harmonic Lasso Selection Editor (Step 1421).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SPECTRAL_MIN_FREQ_HZ: f32 = 20.0;
pub const SPECTRAL_MAX_FREQ_HZ: f32 = 20000.0;
pub const BRUSH_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch area

/// Mode of the spectral selection / drawing tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushToolMode {
    Brush,
    HarmonicLasso,
    PartialWand,
    Eraser,
}

/// Action applied to the selected spectral region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushActionMode {
    GainBoost,
    Attenuate,
    MuteMask,
    Invert,
}

/// Harmonic series selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarmonicSeriesMode {
    All,
    Odd,
    Even,
}

/// Interactive Multi-Track Spectral Frequency Paintbrush & Harmonic Lasso Editor View.
#[derive(Debug, Clone)]
pub struct SpectralBrushEditorView {
    pub active_track_idx: usize,
    pub fft_window_size: usize,
    pub tool_mode: BrushToolMode,
    pub action_mode: BrushActionMode,
    pub brush_radius_pt: f32, // 5.0 ..= 100.0 pt
    pub brush_gain_db: f32,   // -24.0 ..= +24.0 dB
    pub harmonic_count: usize,
    pub harmonic_mode: HarmonicSeriesMode,
    pub fundamental_freq_hz: f32,
    pub lasso_polygon: Vec<(f32, f32)>, // Normalized coordinates (0.0..=1.0, 0.0..=1.0)
    pub is_drawing_lasso: bool,
    pub cursor_pos_norm: (f32, f32),
    pub selected_area_pct: f32,
    pub dry_wet_pct: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralBrushEditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralBrushEditorView {
    pub fn new() -> Self {
        // Initial sample lasso polygon around fundamental and 1st harmonic
        let default_lasso = vec![
            (0.20, 0.35),
            (0.35, 0.40),
            (0.50, 0.38),
            (0.65, 0.45),
            (0.60, 0.60),
            (0.40, 0.58),
            (0.25, 0.52),
        ];

        Self {
            active_track_idx: 0,
            fft_window_size: 2048,
            tool_mode: BrushToolMode::Brush,
            action_mode: BrushActionMode::GainBoost,
            brush_radius_pt: 28.0,
            brush_gain_db: 6.0,
            harmonic_count: 8,
            harmonic_mode: HarmonicSeriesMode::All,
            fundamental_freq_hz: 220.0,
            lasso_polygon: default_lasso,
            is_drawing_lasso: false,
            cursor_pos_norm: (0.5, 0.5),
            selected_area_pct: 18.5,
            dry_wet_pct: 100.0,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert frequency (20 Hz - 20000 Hz) to normalized Y coordinate [0.0 ..= 1.0].
    /// Y = 1.0 is top (high freq), Y = 0.0 is bottom (low freq).
    pub fn freq_to_norm_y(freq_hz: f32) -> f32 {
        let f_clamped = freq_hz.clamp(SPECTRAL_MIN_FREQ_HZ, SPECTRAL_MAX_FREQ_HZ);
        let log_min = SPECTRAL_MIN_FREQ_HZ.log10();
        let log_max = SPECTRAL_MAX_FREQ_HZ.log10();
        ((f_clamped.log10() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    }

    /// Convert normalized Y coordinate [0.0 ..= 1.0] to frequency in Hz.
    pub fn norm_y_to_freq(norm_y: f32) -> f32 {
        let log_min = SPECTRAL_MIN_FREQ_HZ.log10();
        let log_max = SPECTRAL_MAX_FREQ_HZ.log10();
        let log_f = log_min + norm_y.clamp(0.0, 1.0) * (log_max - log_min);
        10.0_f32.powf(log_f)
    }

    /// Ray-casting algorithm to test whether a normalized point (px, py) is inside the lasso polygon.
    pub fn is_point_in_lasso(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
        if polygon.len() < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = polygon.len() - 1;
        for i in 0..polygon.len() {
            let (xi, yi) = polygon[i];
            let (xj, yj) = polygon[j];

            let intersect =
                ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi + 1e-7) + xi);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Generates list of harmonic frequencies up to `harmonic_count`.
    pub fn calculate_harmonic_frequencies(&self) -> Vec<f32> {
        let mut freqs = Vec::new();
        for h in 1..=self.harmonic_count {
            match self.harmonic_mode {
                HarmonicSeriesMode::All => {
                    let f = self.fundamental_freq_hz * h as f32;
                    if f <= SPECTRAL_MAX_FREQ_HZ {
                        freqs.push(f);
                    }
                }
                HarmonicSeriesMode::Odd => {
                    if h % 2 == 1 {
                        let f = self.fundamental_freq_hz * h as f32;
                        if f <= SPECTRAL_MAX_FREQ_HZ {
                            freqs.push(f);
                        }
                    }
                }
                HarmonicSeriesMode::Even => {
                    if h % 2 == 0 {
                        let f = self.fundamental_freq_hz * h as f32;
                        if f <= SPECTRAL_MAX_FREQ_HZ {
                            freqs.push(f);
                        }
                    }
                }
            }
        }
        freqs
    }

    /// Tests if a screen coordinate is within the brush hit target radius (>= 22pt -> 44x44pt).
    pub fn hit_test_brush(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let cursor_screen_x = canvas.x + self.cursor_pos_norm.0 * canvas.width;
        let cursor_screen_y = canvas.y + (1.0 - self.cursor_pos_norm.1) * canvas.height;
        let dx = pos.0 - cursor_screen_x;
        let dy = pos.1 - cursor_screen_y;
        (dx * dx + dy * dy).sqrt() <= (self.brush_radius_pt.max(BRUSH_HANDLE_HIT_RADIUS))
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SPECTRAL BRUSH [{:?}] Mode:{:?} F0:{:.0}Hz R:{:.0}pt (+{:.1}dB)",
            self.tool_mode,
            self.action_mode,
            self.fundamental_freq_hz,
            self.brush_radius_pt,
            self.brush_gain_db
        );
        lines.push(header);

        for y in 1..height {
            let mut row = String::with_capacity(width);
            let norm_y = 1.0 - (y as f32 / height as f32);
            for x in 0..width {
                let norm_x = x as f32 / width as f32;
                if Self::is_point_in_lasso(norm_x, norm_y, &self.lasso_polygon) {
                    row.push('#');
                } else {
                    let d_cursor = ((norm_x - self.cursor_pos_norm.0).powi(2)
                        + (norm_y - self.cursor_pos_norm.1).powi(2))
                    .sqrt();
                    if d_cursor < 0.08 {
                        row.push('*');
                    } else if (norm_y - Self::freq_to_norm_y(self.fundamental_freq_hz)).abs() < 0.03
                    {
                        row.push('-');
                    } else {
                        row.push('.');
                    }
                }
            }
            lines.push(row);
        }
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Header & Track Selector Toolbar
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SPECTRAL FREQUENCY BRUSH & HARMONIC LASSO")
                        .size(15.0)
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();

                let tracks = ["1: Vocals", "2: Bass", "3: Drums", "Master"];
                for (idx, t_name) in tracks.iter().enumerate() {
                    let is_active = self.active_track_idx == idx;
                    let btn = egui::Button::new(
                        egui::RichText::new(*t_name)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(75.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(28, 38, 56)
                    });

                    if ui.add(btn).clicked() {
                        self.active_track_idx = idx;
                    }
                }

                ui.separator();
                ui.label(egui::RichText::new("FFT:").color(Color32::from_rgb(180, 200, 225)));
                for fft_size in [1024, 2048, 4096] {
                    let is_act = self.fft_window_size == fft_size;
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{}", fft_size)).color(if is_act {
                            Color32::from_rgb(10, 14, 22)
                        } else {
                            Color32::from_rgb(200, 220, 245)
                        }),
                    )
                    .min_size(Vec2::new(48.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(24, 32, 48)
                    });
                    if ui.add(btn).clicked() {
                        self.fft_window_size = fft_size;
                    }
                }
            });

            ui.add_space(6.0);

            // 2. Tool & Action Mode Selector Bar (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("TOOL:").strong());
                let tools = [
                    (BrushToolMode::Brush, "BRUSH"),
                    (BrushToolMode::HarmonicLasso, "HARMONIC LASSO"),
                    (BrushToolMode::PartialWand, "PARTIAL WAND"),
                    (BrushToolMode::Eraser, "ERASER"),
                ];
                for (t_mode, lbl) in tools {
                    let is_active = self.tool_mode == t_mode;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(240, 245, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(0, 255, 180)
                    } else {
                        Color32::from_rgb(32, 45, 66)
                    });

                    if ui.add(btn).clicked() {
                        self.tool_mode = t_mode;
                    }
                }

                ui.separator();
                ui.label(egui::RichText::new("ACTION:").strong());
                let actions = [
                    (BrushActionMode::GainBoost, "BOOST (+dB)"),
                    (BrushActionMode::Attenuate, "CUT (-dB)"),
                    (BrushActionMode::MuteMask, "MUTE"),
                    (BrushActionMode::Invert, "INVERT"),
                ];
                for (a_mode, lbl) in actions {
                    let is_active = self.action_mode == a_mode;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(240, 245, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(80.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(255, 107, 43)
                    } else {
                        Color32::from_rgb(32, 45, 66)
                    });

                    if ui.add(btn).clicked() {
                        self.action_mode = a_mode;
                    }
                }
            });

            ui.add_space(8.0);

            // 3. Main Spectrogram & Lasso Drawing Canvas
            let (response, painter) = ui.allocate_painter(
                Vec2::new(ui.available_width().max(300.0), 230.0),
                egui::Sense::click_and_drag(),
            );
            let canvas = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Dark background for spectral canvas
            painter.rect_filled(response.rect, 6.0, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0,
                Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
            );

            // Draw Frequency Octave Grid lines & labels
            for freq in [
                100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0,
            ] {
                let ny = Self::freq_to_norm_y(freq);
                let sy = canvas.y + (1.0 - ny) * canvas.height;
                painter.line_segment(
                    [
                        egui::pos2(canvas.x, sy),
                        egui::pos2(canvas.x + canvas.width, sy),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 60)),
                );
                let label = if freq >= 1000.0 {
                    format!("{:.0}k", freq / 1000.0)
                } else {
                    format!("{:.0}", freq)
                };
                painter.text(
                    egui::pos2(canvas.x + 6.0, sy - 8.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(140, 165, 195),
                );
            }

            // Draw Harmonic Overlays
            let harmonics = self.calculate_harmonic_frequencies();
            for (h_idx, h_freq) in harmonics.iter().enumerate() {
                let ny = Self::freq_to_norm_y(*h_freq);
                let sy = canvas.y + (1.0 - ny) * canvas.height;
                painter.line_segment(
                    [
                        egui::pos2(canvas.x, sy),
                        egui::pos2(canvas.x + canvas.width, sy),
                    ],
                    Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
                );
                painter.text(
                    egui::pos2(canvas.x + canvas.width - 40.0, sy - 7.0),
                    egui::Align2::LEFT_TOP,
                    format!("H{}", h_idx + 1),
                    egui::FontId::monospace(9.0),
                    Color32::from_rgba_unmultiplied(255, 215, 0, 180),
                );
            }

            // Draw Lasso Polygon
            if self.lasso_polygon.len() >= 3 {
                let screen_pts: Vec<egui::Pos2> = self
                    .lasso_polygon
                    .iter()
                    .map(|(nx, ny)| {
                        egui::pos2(
                            canvas.x + nx * canvas.width,
                            canvas.y + (1.0 - ny) * canvas.height,
                        )
                    })
                    .collect();

                // Draw polygon edges
                for i in 0..screen_pts.len() {
                    let p1 = screen_pts[i];
                    let p2 = screen_pts[(i + 1) % screen_pts.len()];
                    painter.line_segment(
                        [p1, p2],
                        Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                    );
                    // Draw draggable vertex handle with >= 22pt touch area
                    painter.circle_filled(p1, 5.0, Color32::from_rgb(0, 255, 180));
                    painter.circle_stroke(
                        p1,
                        BRUSH_HANDLE_HIT_RADIUS,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 50)),
                    );
                }
            }

            // Draw Brush Cursor Puck
            let cursor_sx = canvas.x + self.cursor_pos_norm.0 * canvas.width;
            let cursor_sy = canvas.y + (1.0 - self.cursor_pos_norm.1) * canvas.height;
            let brush_r = self.brush_radius_pt.max(BRUSH_HANDLE_HIT_RADIUS);

            painter.circle_stroke(
                egui::pos2(cursor_sx, cursor_sy),
                brush_r,
                Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
            );
            painter.circle_filled(
                egui::pos2(cursor_sx, cursor_sy),
                4.0,
                Color32::from_rgb(255, 255, 255),
            );

            // Handle Interactions
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let nx = ((pos.x - canvas.x) / canvas.width).clamp(0.0, 1.0);
                    let ny = 1.0 - ((pos.y - canvas.y) / canvas.height).clamp(0.0, 1.0);
                    self.cursor_pos_norm = (nx, ny);
                    if self.tool_mode == BrushToolMode::HarmonicLasso && response.drag_started() {
                        self.lasso_polygon.clear();
                        self.lasso_polygon.push((nx, ny));
                    } else if self.tool_mode == BrushToolMode::HarmonicLasso
                        && response.dragged()
                        && self.lasso_polygon.len() < 64
                    {
                        self.lasso_polygon.push((nx, ny));
                    }
                }
            }

            ui.add_space(8.0);

            // 4. Parameter Controls Bar (Sliders with Ergonomic Touch Targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Brush Radius").strong());
                        ui.add(
                            egui::Slider::new(&mut self.brush_radius_pt, 5.0..=100.0).text("pt"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Brush Gain").strong());
                        ui.add(egui::Slider::new(&mut self.brush_gain_db, -24.0..=24.0).text("dB"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Fundamental (F0)").strong());
                        ui.add(
                            egui::Slider::new(&mut self.fundamental_freq_hz, 40.0..=2000.0)
                                .text("Hz"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Harmonic Count").strong());
                        ui.add(
                            egui::Slider::new(&mut self.harmonic_count, 1..=16).text("partials"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Dry / Wet").strong());
                        ui.add(egui::Slider::new(&mut self.dry_wet_pct, 0.0..=100.0).text("%"));
                    });
                });
            });
        });
    }
}
