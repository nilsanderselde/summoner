// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive 64-Band Vocoder Modulator & Carrier Harmonic Matrix View (Step 1401).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const VOCODER_NUM_BANDS: usize = 64;
pub const VOCODER_BAND_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const VOCODER_MIN_FREQ_HZ: f32 = 50.0;
pub const VOCODER_MAX_FREQ_HZ: f32 = 12000.0;

/// Solo / Mute state for an individual vocoder filter band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocoderBandState {
    Active,
    Solo,
    Mute,
    Bypass,
}

/// Single band in the 64-band vocoder filter bank.
#[derive(Debug, Clone, PartialEq)]
pub struct VocoderBand {
    pub index: usize,
    pub center_freq_hz: f32,
    pub bandwidth_hz: f32,
    pub modulator_level: f32, // 0.0 ..= 1.0
    pub carrier_level: f32,   // 0.0 ..= 1.0
    pub gain_db: f32,         // -24.0 ..= +12.0 dB
    pub pan: f32,             // -1.0 ..= +1.0
    pub state: VocoderBandState,
}

impl VocoderBand {
    pub fn new(index: usize, center_freq_hz: f32, bandwidth_hz: f32) -> Self {
        Self {
            index,
            center_freq_hz,
            bandwidth_hz,
            modulator_level: 0.0,
            carrier_level: 0.0,
            gain_db: 0.0,
            pan: 0.0,
            state: VocoderBandState::Active,
        }
    }
}

/// Interactive 64-Band Vocoder Matrix View (Step 1401).
#[derive(Debug, Clone)]
pub struct VocoderMatrixView {
    pub bands: Vec<VocoderBand>,
    pub formant_tilt_db_oct: f32,     // -6.0 ..= +6.0 dB/oct
    pub formant_shift_semitones: f32, // -12.0 ..= +12.0 st
    pub freeze_buffer_enabled: bool,
    pub frozen_modulator_spectrum: Vec<f32>,
    pub active_band_idx: usize,
    pub dragging_band_idx: Option<usize>,
    pub modulator_gain_db: f32,         // -24.0 ..= +12.0 dB
    pub carrier_gain_db: f32,           // -24.0 ..= +12.0 dB
    pub output_gain_db: f32,            // -24.0 ..= +12.0 dB
    pub dry_wet_pct: f32,               // 0.0 ..= 100.0%
    pub envelope_attack_ms: f32,        // 0.5 ..= 50.0 ms
    pub envelope_release_ms: f32,       // 5.0 ..= 500.0 ms
    pub high_pass_unvoiced_thresh: f32, // 0.0 ..= 1.0 (sibilance noise generator)
    pub color_palette: ContrastColorPalette,
}

impl Default for VocoderMatrixView {
    fn default() -> Self {
        Self::new()
    }
}

impl VocoderMatrixView {
    pub fn new() -> Self {
        let mut bands = Vec::with_capacity(VOCODER_NUM_BANDS);
        let log_min = VOCODER_MIN_FREQ_HZ.log10();
        let log_max = VOCODER_MAX_FREQ_HZ.log10();

        for i in 0..VOCODER_NUM_BANDS {
            let norm = i as f32 / (VOCODER_NUM_BANDS - 1) as f32;
            let log_freq = log_min + norm * (log_max - log_min);
            let center_freq = 10.0_f32.powf(log_freq);
            let next_log_freq =
                log_min + ((i + 1) as f32 / (VOCODER_NUM_BANDS - 1) as f32) * (log_max - log_min);
            let bandwidth = (10.0_f32.powf(next_log_freq) - center_freq).max(10.0);
            bands.push(VocoderBand::new(i, center_freq, bandwidth));
        }

        Self {
            bands,
            formant_tilt_db_oct: 0.0,
            formant_shift_semitones: 0.0,
            freeze_buffer_enabled: false,
            frozen_modulator_spectrum: vec![0.0; VOCODER_NUM_BANDS],
            active_band_idx: 16,
            dragging_band_idx: None,
            modulator_gain_db: 0.0,
            carrier_gain_db: 0.0,
            output_gain_db: 0.0,
            dry_wet_pct: 100.0,
            envelope_attack_ms: 5.0,
            envelope_release_ms: 45.0,
            high_pass_unvoiced_thresh: 0.25,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert band index (0..63) to normalized horizontal coordinate [0.0 ..= 1.0].
    pub fn band_to_norm_x(band_idx: usize) -> f32 {
        if VOCODER_NUM_BANDS <= 1 {
            return 0.5;
        }
        (band_idx as f32 / (VOCODER_NUM_BANDS - 1) as f32).clamp(0.0, 1.0)
    }

    /// Convert screen coordinate to closest band index.
    pub fn screen_x_to_band_idx(&self, screen_x: f32, canvas: Rect) -> usize {
        if canvas.width <= 0.0 {
            return 0;
        }
        let norm = ((screen_x - canvas.x) / canvas.width).clamp(0.0, 1.0);
        ((norm * (VOCODER_NUM_BANDS - 1) as f32).round() as usize).min(VOCODER_NUM_BANDS - 1)
    }

    /// Convert band index to screen X pixel coordinate.
    pub fn band_idx_to_screen_x(&self, band_idx: usize, canvas: Rect) -> f32 {
        canvas.x + Self::band_to_norm_x(band_idx) * canvas.width
    }

    /// Calculate effective gain modifier for a band based on formant tilt (in dB).
    pub fn calculate_formant_tilt_gain_db(&self, band_idx: usize) -> f32 {
        if band_idx >= self.bands.len() {
            return 0.0;
        }
        let center_freq = self.bands[band_idx].center_freq_hz;
        let ref_freq = 1000.0_f32; // 1 kHz reference pivot
        let octaves_from_ref = (center_freq / ref_freq).log2();
        octaves_from_ref * self.formant_tilt_db_oct
    }

    /// Hit-test individual band handle with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_band_handle(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        if pos.1 < canvas.y - 10.0 || pos.1 > canvas.y + canvas.height + 10.0 {
            return None;
        }
        let band_idx = self.screen_x_to_band_idx(pos.0, canvas);
        let band_x = self.band_idx_to_screen_x(band_idx, canvas);
        if (pos.0 - band_x).abs() <= VOCODER_BAND_HANDLE_HIT_RADIUS {
            Some(band_idx)
        } else {
            None
        }
    }

    /// Toggle freeze buffer to lock current modulator spectral envelope.
    pub fn toggle_freeze_buffer(&mut self) {
        self.freeze_buffer_enabled = !self.freeze_buffer_enabled;
        if self.freeze_buffer_enabled {
            for (i, band) in self.bands.iter().enumerate() {
                self.frozen_modulator_spectrum[i] = band.modulator_level;
            }
        }
    }

    /// Update live modulator and carrier levels for visualization.
    pub fn update_band_levels(&mut self, band_idx: usize, mod_lvl: f32, car_lvl: f32) {
        if let Some(band) = self.bands.get_mut(band_idx) {
            if !self.freeze_buffer_enabled {
                band.modulator_level = mod_lvl.clamp(0.0, 1.0);
            }
            band.carrier_level = car_lvl.clamp(0.0, 1.0);
        }
    }

    /// Deterministic ASCII render of the 64-band vocoder matrix.
    pub fn render_ascii(&self, width: usize) -> String {
        let step = (VOCODER_NUM_BANDS as f32 / width as f32).max(1.0);
        let mut out = String::with_capacity(width);
        for col in 0..width {
            let band_idx = ((col as f32 * step).round() as usize).min(VOCODER_NUM_BANDS - 1);
            let band = &self.bands[band_idx];
            let lvl = if self.freeze_buffer_enabled {
                self.frozen_modulator_spectrum[band_idx]
            } else {
                band.modulator_level
            };
            let char_val = if lvl > 0.75 {
                '#'
            } else if lvl > 0.50 {
                '='
            } else if lvl > 0.25 {
                '-'
            } else {
                '.'
            };
            out.push(char_val);
        }
        out
    }
}

#[cfg(feature = "gui")]
impl VocoderMatrixView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("64-BAND HARMONIC VOCODER MATRIX")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Active Band: #{:02} ({:.0} Hz) | Tilt: {:+.1} dB/oct",
                        self.active_band_idx + 1,
                        self.bands[self.active_band_idx].center_freq_hz,
                        self.formant_tilt_db_oct
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();

                // Freeze Buffer Toggle Button (>=60x44pt)
                let freeze_btn = egui::Button::new(
                    egui::RichText::new(if self.freeze_buffer_enabled {
                        "FREEZE: ON"
                    } else {
                        "FREEZE: OFF"
                    })
                    .color(if self.freeze_buffer_enabled {
                        Color32::from_rgb(10, 14, 22)
                    } else {
                        Color32::from_rgb(220, 235, 255)
                    })
                    .strong(),
                )
                .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                .fill(if self.freeze_buffer_enabled {
                    Color32::from_rgb(0, 229, 255)
                } else {
                    Color32::from_rgb(35, 45, 65)
                });

                if ui.add(freeze_btn).clicked() {
                    self.toggle_freeze_buffer();
                }
            });

            ui.add_space(6.0);

            // 2. 64-Band Harmonic Matrix Canvas
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 220.0;
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

            // Frequency Grid Markers
            let freq_markers = [100.0_f32, 500.0_f32, 1000.0_f32, 4000.0_f32, 10000.0_f32];
            for f in freq_markers {
                let norm = (f.log10() - VOCODER_MIN_FREQ_HZ.log10())
                    / (VOCODER_MAX_FREQ_HZ.log10() - VOCODER_MIN_FREQ_HZ.log10());
                let gx = canvas.x + norm.clamp(0.0, 1.0) * canvas.width;
                painter.line_segment(
                    [
                        egui::pos2(gx, canvas.y),
                        egui::pos2(gx, canvas.y + canvas.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 70)),
                );
                let label = if f >= 1000.0 {
                    format!("{:.0}k", f / 1000.0)
                } else {
                    format!("{:.0}Hz", f)
                };
                painter.text(
                    egui::pos2(gx + 3.0, canvas.y + canvas.height - 12.0),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(130, 155, 185),
                );
            }

            // Draw 64 Band Bars
            let col_w = (canvas.width / VOCODER_NUM_BANDS as f32).max(4.0);
            for i in 0..VOCODER_NUM_BANDS {
                let bx = canvas.x + i as f32 * col_w;
                let band = &self.bands[i];
                let is_act = i == self.active_band_idx;

                // Modulator Bar (Cyan)
                let mod_h = band.modulator_level * (canvas.height - 40.0);
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(bx + 1.0, canvas.y + canvas.height - 20.0 - mod_h),
                        egui::pos2(bx + col_w * 0.5 - 1.0, canvas.y + canvas.height - 20.0),
                    ),
                    2.0_f32,
                    Color32::from_rgb(0, 229, 255),
                );

                // Carrier Bar (Gold)
                let car_h = band.carrier_level * (canvas.height - 40.0);
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(
                            bx + col_w * 0.5 + 1.0,
                            canvas.y + canvas.height - 20.0 - car_h,
                        ),
                        egui::pos2(bx + col_w - 1.0, canvas.y + canvas.height - 20.0),
                    ),
                    2.0_f32,
                    Color32::from_rgb(255, 215, 0),
                );

                if is_act {
                    // Highlight Active Band Frame
                    painter.rect_stroke(
                        egui::Rect::from_min_max(
                            egui::pos2(bx - 2.0, canvas.y + 10.0),
                            egui::pos2(bx + col_w + 2.0, canvas.y + canvas.height - 10.0),
                        ),
                        2.0_f32,
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)),
                    );
                }
            }

            // Draw Formant Tilt Slope Line
            let tilt_y_left =
                canvas.y + canvas.height * 0.5 - self.calculate_formant_tilt_gain_db(0) * 4.0;
            let tilt_y_right = canvas.y + canvas.height * 0.5
                - self.calculate_formant_tilt_gain_db(VOCODER_NUM_BANDS - 1) * 4.0;
            painter.line_segment(
                [
                    egui::pos2(canvas.x, tilt_y_left),
                    egui::pos2(canvas.x + canvas.width, tilt_y_right),
                ],
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 200)),
            );

            // Drag & Click Hit Testing
            if response.drag_started() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let b_idx = self.screen_x_to_band_idx(pos.x, canvas);
                    self.active_band_idx = b_idx;
                    self.dragging_band_idx = Some(b_idx);
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let b_idx = self.screen_x_to_band_idx(pos.x, canvas);
                    self.active_band_idx = b_idx;
                    let norm_gain = 1.0 - ((pos.y - canvas.y) / canvas.height).clamp(0.0, 1.0);
                    self.bands[b_idx].gain_db = -24.0 + norm_gain * 36.0;
                }
            }

            if response.drag_stopped() {
                self.dragging_band_idx = None;
            }

            ui.add_space(10.0);

            // 3. Selected Band Parameter Controls (>=44pt Touch Targets)
            let curr_band = &mut self.bands[self.active_band_idx];
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "BAND #{:02}: {:.0} Hz (BW: {:.0} Hz)",
                            curr_band.index + 1,
                            curr_band.center_freq_hz,
                            curr_band.bandwidth_hz
                        ))
                        .color(Color32::from_rgb(255, 215, 0))
                        .strong(),
                    );
                    ui.separator();

                    let modes = [
                        (VocoderBandState::Active, "ACTIVE"),
                        (VocoderBandState::Solo, "SOLO"),
                        (VocoderBandState::Mute, "MUTE"),
                        (VocoderBandState::Bypass, "BYPASS"),
                    ];
                    for (m, lbl) in modes {
                        let is_act = curr_band.state == m;
                        let btn = egui::Button::new(
                            egui::RichText::new(lbl)
                                .color(if is_act {
                                    Color32::from_rgb(10, 14, 22)
                                } else {
                                    Color32::from_rgb(220, 235, 255)
                                })
                                .strong(),
                        )
                        .min_size(Vec2::new(64.0, MIN_HIT_TARGET_PT))
                        .fill(if is_act {
                            Color32::from_rgb(0, 229, 255)
                        } else {
                            Color32::from_rgb(30, 40, 60)
                        });

                        if ui.add(btn).clicked() {
                            curr_band.state = m;
                        }
                    }
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Band Gain").strong());
                        ui.add(egui::Slider::new(&mut curr_band.gain_db, -24.0..=12.0).text("dB"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Band Stereo Pan").strong());
                        ui.add(egui::Slider::new(&mut curr_band.pan, -1.0..=1.0).text("L/R"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Modulator Attack").strong());
                        ui.add(
                            egui::Slider::new(&mut self.envelope_attack_ms, 0.5..=50.0).text("ms"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Modulator Release").strong());
                        ui.add(
                            egui::Slider::new(&mut self.envelope_release_ms, 5.0..=500.0)
                                .text("ms"),
                        );
                    });
                });
            });

            ui.add_space(8.0);

            // 4. Global Formant & Output Controls
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Formant Shift").strong());
                ui.add(
                    egui::Slider::new(&mut self.formant_shift_semitones, -12.0..=12.0).text("st"),
                );

                ui.separator();
                ui.label(egui::RichText::new("Formant Tilt").strong());
                ui.add(egui::Slider::new(&mut self.formant_tilt_db_oct, -6.0..=6.0).text("dB/oct"));

                ui.separator();
                ui.label(egui::RichText::new("Sibilance Thresh").strong());
                ui.add(
                    egui::Slider::new(&mut self.high_pass_unvoiced_thresh, 0.0..=1.0).text("sens"),
                );

                ui.separator();
                ui.label(egui::RichText::new("Dry/Wet").strong());
                ui.add(egui::Slider::new(&mut self.dry_wet_pct, 0.0..=100.0).text("%"));
            });
        });
    }
}
