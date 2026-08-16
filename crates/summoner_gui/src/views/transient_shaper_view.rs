// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Multi-Band Audio Transient Shaper with Dynamic Frequency Split Crossovers (Step 1381).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const CROSSOVER_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch box
pub const CROSSOVER_HANDLE_VISUAL_WIDTH: f32 = 6.0;

/// Processing mode for a transient shaper frequency band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandMuteSolo {
    Normal,
    Mute,
    Solo,
    Bypass,
}

/// Single frequency band parameters in the multi-band transient shaper.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientBand {
    pub name: String,
    pub min_freq_hz: f32,
    pub max_freq_hz: f32,
    pub attack_gain_db: f32,   // -12.0 ..= +12.0 dB
    pub sustain_gain_db: f32,  // -12.0 ..= +12.0 dB
    pub attack_time_ms: f32,   // 1.0 ..= 100.0 ms
    pub sustain_decay_ms: f32, // 10.0 ..= 500.0 ms
    pub output_level_db: f32,  // -24.0 ..= +12.0 dB
    pub mode: BandMuteSolo,
    pub detected_transient_peak: f32, // 0.0 ..= 1.0 for real-time visualization
    pub detected_sustain_level: f32,  // 0.0 ..= 1.0
}

impl TransientBand {
    pub fn new(name: impl Into<String>, min_freq: f32, max_freq: f32) -> Self {
        Self {
            name: name.into(),
            min_freq_hz: min_freq.clamp(20.0, 20000.0),
            max_freq_hz: max_freq.clamp(20.0, 20000.0),
            attack_gain_db: 0.0,
            sustain_gain_db: 0.0,
            attack_time_ms: 20.0,
            sustain_decay_ms: 120.0,
            output_level_db: 0.0,
            mode: BandMuteSolo::Normal,
            detected_transient_peak: 0.0,
            detected_sustain_level: 0.0,
        }
    }
}

/// Interactive Multi-Band Audio Transient Shaper View (Step 1381).
#[derive(Debug, Clone)]
pub struct TransientShaperView {
    pub bands: [TransientBand; 3],  // Low, Mid, High
    pub crossover_low_mid_hz: f32,  // 40.0 ..= 1000.0 Hz (default 250 Hz)
    pub crossover_mid_high_hz: f32, // 1000.0 ..= 12000.0 Hz (default 3500 Hz)
    pub active_band_idx: usize,
    pub dragging_crossover_idx: Option<usize>, // 0: low-mid, 1: mid-high
    pub global_input_gain_db: f32,             // -24.0 ..= +12.0 dB
    pub global_output_gain_db: f32,            // -24.0 ..= +12.0 dB
    pub global_mix_pct: f32,                   // 0.0 ..= 100.0%
    pub clip_limit_enabled: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientShaperView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientShaperView {
    pub const MIN_FREQ_HZ: f32 = 20.0;
    pub const MAX_FREQ_HZ: f32 = 20000.0;

    pub fn new() -> Self {
        let low_band = TransientBand::new("LOW", Self::MIN_FREQ_HZ, 250.0);
        let mid_band = TransientBand::new("MID", 250.0, 3500.0);
        let high_band = TransientBand::new("HIGH", 3500.0, Self::MAX_FREQ_HZ);

        Self {
            bands: [low_band, mid_band, high_band],
            crossover_low_mid_hz: 250.0,
            crossover_mid_high_hz: 3500.0,
            active_band_idx: 1, // Default focus on MID
            dragging_crossover_idx: None,
            global_input_gain_db: 0.0,
            global_output_gain_db: 0.0,
            global_mix_pct: 100.0,
            clip_limit_enabled: true,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Map log frequency (20 Hz - 20 kHz) to normalized horizontal position [0.0 ..= 1.0].
    pub fn freq_to_norm_x(freq_hz: f32) -> f32 {
        let clamped = freq_hz.clamp(Self::MIN_FREQ_HZ, Self::MAX_FREQ_HZ);
        let log_min = Self::MIN_FREQ_HZ.log10();
        let log_max = Self::MAX_FREQ_HZ.log10();
        let log_val = clamped.log10();
        ((log_val - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    }

    /// Map normalized horizontal position [0.0 ..= 1.0] to log frequency (20 Hz - 20 kHz).
    pub fn norm_x_to_freq(norm_x: f32) -> f32 {
        let clamped = norm_x.clamp(0.0, 1.0);
        let log_min = Self::MIN_FREQ_HZ.log10();
        let log_max = Self::MAX_FREQ_HZ.log10();
        let log_val = log_min + clamped * (log_max - log_min);
        10.0_f32
            .powf(log_val)
            .clamp(Self::MIN_FREQ_HZ, Self::MAX_FREQ_HZ)
    }

    /// Convert frequency to canvas X pixel coordinate.
    pub fn freq_to_screen_x(&self, freq_hz: f32, canvas: Rect) -> f32 {
        canvas.x + Self::freq_to_norm_x(freq_hz) * canvas.width
    }

    /// Convert canvas X pixel coordinate to frequency in Hz.
    pub fn screen_x_to_freq(&self, screen_x: f32, canvas: Rect) -> f32 {
        if canvas.width <= 0.0 {
            return Self::MIN_FREQ_HZ;
        }
        let norm_x = ((screen_x - canvas.x) / canvas.width).clamp(0.0, 1.0);
        Self::norm_x_to_freq(norm_x)
    }

    /// Update crossover splits while ensuring minimum spacing (at least 100 Hz gap).
    pub fn set_crossover_low_mid(&mut self, freq_hz: f32) {
        let max_allowed = (self.crossover_mid_high_hz - 100.0).max(60.0);
        self.crossover_low_mid_hz = freq_hz.clamp(40.0, max_allowed);
        self.bands[0].max_freq_hz = self.crossover_low_mid_hz;
        self.bands[1].min_freq_hz = self.crossover_low_mid_hz;
    }

    pub fn set_crossover_mid_high(&mut self, freq_hz: f32) {
        let min_allowed = (self.crossover_low_mid_hz + 100.0).min(18000.0);
        self.crossover_mid_high_hz = freq_hz.clamp(min_allowed, 18000.0);
        self.bands[1].max_freq_hz = self.crossover_mid_high_hz;
        self.bands[2].min_freq_hz = self.crossover_mid_high_hz;
    }

    /// Hit-test crossover split handles with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_crossover_handle(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        if pos.1 < canvas.y - 10.0 || pos.1 > canvas.y + canvas.height + 10.0 {
            return None;
        }

        let low_mid_x = self.freq_to_screen_x(self.crossover_low_mid_hz, canvas);
        if (pos.0 - low_mid_x).abs() <= CROSSOVER_HANDLE_HIT_RADIUS {
            return Some(0);
        }

        let mid_high_x = self.freq_to_screen_x(self.crossover_mid_high_hz, canvas);
        if (pos.0 - mid_high_x).abs() <= CROSSOVER_HANDLE_HIT_RADIUS {
            return Some(1);
        }

        None
    }

    /// Hit-test band regions on canvas to select active band for editing.
    pub fn hit_test_band_region(&self, screen_x: f32, canvas: Rect) -> usize {
        let freq = self.screen_x_to_freq(screen_x, canvas);
        if freq < self.crossover_low_mid_hz {
            0
        } else if freq < self.crossover_mid_high_hz {
            1
        } else {
            2
        }
    }

    /// Feed real-time transient detection metrics for visualization.
    pub fn update_band_meters(&mut self, band_idx: usize, peak: f32, sustain: f32) {
        if let Some(band) = self.bands.get_mut(band_idx) {
            band.detected_transient_peak = peak.clamp(0.0, 1.0);
            band.detected_sustain_level = sustain.clamp(0.0, 1.0);
        }
    }

    /// Deterministic ASCII render of the 3-band split frequency spectrum.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec![' '; width];
        let lm_norm = Self::freq_to_norm_x(self.crossover_low_mid_hz);
        let mh_norm = Self::freq_to_norm_x(self.crossover_mid_high_hz);

        let lm_pos = ((lm_norm * (width - 1) as f32).round() as usize).min(width - 1);
        let mh_pos = ((mh_norm * (width - 1) as f32).round() as usize).min(width - 1);

        for (i, item) in buf.iter_mut().enumerate() {
            if i < lm_pos {
                *item = 'L';
            } else if i == lm_pos {
                *item = '|';
            } else if i < mh_pos {
                *item = 'M';
            } else if i == mh_pos {
                *item = '|';
            } else {
                *item = 'H';
            }
        }
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl TransientShaperView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("MULTI-BAND AUDIO TRANSIENT SHAPER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Low/Mid: {:.0} Hz | Mid/High: {:.0} Hz",
                        self.crossover_low_mid_hz, self.crossover_mid_high_hz
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();
                ui.checkbox(&mut self.clip_limit_enabled, "Soft Clip Limiter");
            });

            ui.add_space(6.0);

            // 2. Multi-Band Interactive Crossover Frequency Canvas
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 160.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());
            let canvas = Rect::new(
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

            // Frequency Grid Lines (100Hz, 1kHz, 10kHz)
            let freq_guides = [
                50.0_f32,
                100.0_f32,
                500.0_f32,
                1000.0_f32,
                5000.0_f32,
                10000.0_f32,
            ];
            for f in freq_guides {
                let gx = self.freq_to_screen_x(f, canvas);
                painter.line_segment(
                    [
                        egui::pos2(gx, canvas.y),
                        egui::pos2(gx, canvas.y + canvas.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 70)),
                );
                let label = if f >= 1000.0_f32 {
                    format!("{:.0}k", f / 1000.0_f32)
                } else {
                    format!("{:.0}", f)
                };
                painter.text(
                    egui::pos2(gx + 3.0_f32, canvas.y + canvas.height - 12.0_f32),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::proportional(10.0_f32),
                    Color32::from_rgb(130, 155, 185),
                );
            }

            // Band Fill Regions
            let lm_x = self.freq_to_screen_x(self.crossover_low_mid_hz, canvas);
            let mh_x = self.freq_to_screen_x(self.crossover_mid_high_hz, canvas);

            let band_colors = [
                Color32::from_rgba_unmultiplied(0, 229, 255, 35), // Low
                Color32::from_rgba_unmultiplied(255, 215, 0, 35), // Mid
                Color32::from_rgba_unmultiplied(255, 107, 43, 35), // High
            ];

            let band_rects = [
                egui::Rect::from_min_max(
                    egui::pos2(canvas.x, canvas.y),
                    egui::pos2(lm_x, canvas.y + canvas.height),
                ),
                egui::Rect::from_min_max(
                    egui::pos2(lm_x, canvas.y),
                    egui::pos2(mh_x, canvas.y + canvas.height),
                ),
                egui::Rect::from_min_max(
                    egui::pos2(mh_x, canvas.y),
                    egui::pos2(canvas.x + canvas.width, canvas.y + canvas.height),
                ),
            ];

            for (idx, r) in band_rects.iter().enumerate() {
                painter.rect_filled(*r, 0.0_f32, band_colors[idx]);
                if idx == self.active_band_idx {
                    painter.rect_stroke(
                        *r,
                        0.0_f32,
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)),
                    );
                }
                // Band Label on canvas
                let center_x = (r.min.x + r.max.x) * 0.5_f32;
                painter.text(
                    egui::pos2(center_x, canvas.y + 20.0_f32),
                    egui::Align2::CENTER_CENTER,
                    format!("BAND {}: {}", idx + 1, self.bands[idx].name),
                    egui::FontId::proportional(13.0_f32),
                    if idx == self.active_band_idx {
                        Color32::from_rgb(255, 255, 255)
                    } else {
                        Color32::from_rgb(180, 200, 225)
                    },
                );

                // Transient Attack & Sustain Readout
                painter.text(
                    egui::pos2(center_x, canvas.y + 44.0_f32),
                    egui::Align2::CENTER_CENTER,
                    format!(
                        "Att: {:+.1}dB | Sus: {:+.1}dB",
                        self.bands[idx].attack_gain_db, self.bands[idx].sustain_gain_db
                    ),
                    egui::FontId::proportional(11.0_f32),
                    Color32::from_rgb(200, 220, 240),
                );
            }

            // Draw Crossover Vertical Split Handles (>=44pt Touch Targets)
            let handles = [(lm_x, 0), (mh_x, 1)];
            for &(hx, _c_idx) in &handles {
                // Split line
                painter.line_segment(
                    [
                        egui::pos2(hx, canvas.y),
                        egui::pos2(hx, canvas.y + canvas.height),
                    ],
                    Stroke::new(
                        CROSSOVER_HANDLE_VISUAL_WIDTH,
                        Color32::from_rgb(0, 229, 255),
                    ),
                );
                // Hit target puck at center
                let handle_center = egui::pos2(hx, canvas.y + canvas.height * 0.5_f32);
                painter.circle_filled(handle_center, 14.0_f32, Color32::from_rgb(0, 229, 255));
                painter.circle_filled(handle_center, 4.0_f32, Color32::from_rgb(10, 14, 22));
                // Outer touch bounding ring (22pt radius = 44x44pt touch area)
                painter.circle_stroke(
                    handle_center,
                    CROSSOVER_HANDLE_HIT_RADIUS,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
                );
            }

            // Handle Crossover Drag Interactions
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(h_idx) = self.hit_test_crossover_handle((pos.x, pos.y), canvas) {
                        self.dragging_crossover_idx = Some(h_idx);
                    } else {
                        self.active_band_idx = self.hit_test_band_region(pos.x, canvas);
                    }
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let freq = self.screen_x_to_freq(pos.x, canvas);
                    match self.dragging_crossover_idx {
                        Some(0) => self.set_crossover_low_mid(freq),
                        Some(1) => self.set_crossover_mid_high(freq),
                        _ => {}
                    }
                }
            }

            if response.drag_stopped() {
                self.dragging_crossover_idx = None;
            }

            ui.add_space(10.0);

            // 3. Active Band Parameter Cards & Sliders (>=44pt Touch Targets)
            let curr_band = &mut self.bands[self.active_band_idx];
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "SELECTED BAND: {} ({:.0} Hz - {:.0} Hz)",
                            curr_band.name, curr_band.min_freq_hz, curr_band.max_freq_hz
                        ))
                        .color(Color32::from_rgb(255, 215, 0))
                        .strong(),
                    );
                    ui.separator();

                    // Band Mute/Solo/Bypass Buttons (>=44x44pt Touch Targets)
                    let modes = [
                        (BandMuteSolo::Normal, "NORM"),
                        (BandMuteSolo::Solo, "SOLO"),
                        (BandMuteSolo::Mute, "MUTE"),
                        (BandMuteSolo::Bypass, "BYPASS"),
                    ];
                    for (m, lbl) in modes {
                        let is_act = curr_band.mode == m;
                        let btn = egui::Button::new(
                            egui::RichText::new(lbl)
                                .color(if is_act {
                                    Color32::from_rgb(10, 14, 22)
                                } else {
                                    Color32::from_rgb(220, 235, 255)
                                })
                                .strong(),
                        )
                        .min_size(Vec2::new(60.0, MIN_HIT_TARGET_PT))
                        .fill(if is_act {
                            Color32::from_rgb(0, 229, 255)
                        } else {
                            Color32::from_rgb(30, 40, 60)
                        });

                        if ui.add(btn).clicked() {
                            curr_band.mode = m;
                        }
                    }
                });

                ui.add_space(8.0);

                // Per-band sliders
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Attack Gain").strong());
                        ui.add(
                            egui::Slider::new(&mut curr_band.attack_gain_db, -12.0..=12.0)
                                .text("dB"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sustain Gain").strong());
                        ui.add(
                            egui::Slider::new(&mut curr_band.sustain_gain_db, -12.0..=12.0)
                                .text("dB"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Attack Time").strong());
                        ui.add(
                            egui::Slider::new(&mut curr_band.attack_time_ms, 1.0..=100.0)
                                .text("ms"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sustain Decay").strong());
                        ui.add(
                            egui::Slider::new(&mut curr_band.sustain_decay_ms, 10.0..=500.0)
                                .text("ms"),
                        );
                    });
                });
            });

            ui.add_space(8.0);

            // 4. Global Controls Bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Input Gain").strong());
                ui.add(egui::Slider::new(&mut self.global_input_gain_db, -24.0..=12.0).text("dB"));

                ui.separator();
                ui.label(egui::RichText::new("Dry/Wet Mix").strong());
                ui.add(egui::Slider::new(&mut self.global_mix_pct, 0.0..=100.0).text("%"));

                ui.separator();
                ui.label(egui::RichText::new("Output Gain").strong());
                ui.add(egui::Slider::new(&mut self.global_output_gain_db, -24.0..=12.0).text("dB"));
            });
        });
    }
}
