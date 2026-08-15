// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Real-Time 3D FFT Waterfall Spectrogram Visualizer GUI (Step 1322).

use crate::touch_controls::MIN_HIT_TARGET_PT;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

pub const NUM_FFT_BINS: usize = 64;
pub const NUM_HISTORY_SLICES: usize = 32;

/// Frequency band marker for visual grid alignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreqMarker {
    pub hz: f32,
    pub bin_fraction: f32, // 0.0 ..= 1.0
    pub label: &'static str,
}

pub const FREQ_MARKERS: [FreqMarker; 5] = [
    FreqMarker {
        hz: 20.0,
        bin_fraction: 0.02,
        label: "20Hz",
    },
    FreqMarker {
        hz: 100.0,
        bin_fraction: 0.10,
        label: "100Hz",
    },
    FreqMarker {
        hz: 1000.0,
        bin_fraction: 0.35,
        label: "1kHz",
    },
    FreqMarker {
        hz: 5000.0,
        bin_fraction: 0.65,
        label: "5kHz",
    },
    FreqMarker {
        hz: 20000.0,
        bin_fraction: 0.98,
        label: "20kHz",
    },
];

/// Real-Time 3D Waterfall Spectrogram Visualizer View (Step 1322).
#[derive(Debug, Clone)]
pub struct Spectrogram3DView {
    /// History of FFT magnitude slices (0 = latest, NUM_HISTORY_SLICES-1 = oldest)
    pub slices: Vec<[f32; NUM_FFT_BINS]>,
    /// Camera Orbit Angles in degrees
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub zoom: f32,
    /// Visual Options
    pub wireframe_mode: bool,
    pub is_frozen: bool,
    pub peak_frequency_hz: f32,
    pub peak_magnitude_db: f32,
}

impl Default for Spectrogram3DView {
    fn default() -> Self {
        Self::new()
    }
}

impl Spectrogram3DView {
    pub fn new() -> Self {
        let mut initial_slices = Vec::with_capacity(NUM_HISTORY_SLICES);
        for s in 0..NUM_HISTORY_SLICES {
            let mut slice = [0.0f32; NUM_FFT_BINS];
            let decay = 1.0 - (s as f32 / NUM_HISTORY_SLICES as f32) * 0.7;
            for (b, val) in slice.iter_mut().enumerate() {
                // Synth harmonics preview profile
                let freq = b as f32 / NUM_FFT_BINS as f32;
                let fund = (-((freq - 0.15) * 15.0).powi(2)).exp();
                let harm1 = 0.6 * (-((freq - 0.30) * 20.0).powi(2)).exp();
                let harm2 = 0.4 * (-((freq - 0.45) * 25.0).powi(2)).exp();
                let noise = 0.05 * (b as f32 * 0.3).sin().abs();
                *val = (fund + harm1 + harm2 + noise) * decay;
            }
            initial_slices.push(slice);
        }

        Self {
            slices: initial_slices,
            yaw_deg: -25.0,
            pitch_deg: 35.0,
            zoom: 1.0,
            wireframe_mode: false,
            is_frozen: false,
            peak_frequency_hz: 440.0,
            peak_magnitude_db: -3.5,
        }
    }

    /// Push a new FFT spectrum slice
    pub fn push_fft_slice(&mut self, new_slice: [f32; NUM_FFT_BINS]) {
        if !self.is_frozen {
            self.slices.insert(0, new_slice);
            if self.slices.len() > NUM_HISTORY_SLICES {
                self.slices.pop();
            }
            // Update peak detection
            let mut max_mag = 0.0f32;
            let mut max_bin = 0;
            for (b, &v) in new_slice.iter().enumerate() {
                if v > max_mag {
                    max_mag = v;
                    max_bin = b;
                }
            }
            self.peak_frequency_hz = 20.0 * (1000.0f32).powf(max_bin as f32 / NUM_FFT_BINS as f32);
            self.peak_magnitude_db = if max_mag > 1e-5 {
                20.0 * max_mag.log10()
            } else {
                -96.0
            };
        }
    }

    /// Reset camera view angles
    pub fn reset_camera(&mut self) {
        self.yaw_deg = -25.0;
        self.pitch_deg = 35.0;
        self.zoom = 1.0;
    }

    /// 3D Projection math: maps (freq_norm 0..1, time_norm 0..1, magnitude 0..1) to canvas (x, y)
    #[allow(clippy::too_many_arguments)]
    pub fn project_3d_point(
        freq_norm: f32,
        time_norm: f32,
        mag: f32,
        center: (f32, f32),
        width: f32,
        height: f32,
        yaw_deg: f32,
        pitch_deg: f32,
        zoom: f32,
    ) -> (f32, f32) {
        let yaw_rad = yaw_deg.to_radians();
        let pitch_rad = pitch_deg.to_radians();

        // 3D Box coordinate space centered at 0: X = freq (-0.5..0.5), Y = height (0..1), Z = time (-0.5..0.5)
        let x0 = (freq_norm - 0.5) * width * zoom;
        let z0 = (time_norm - 0.5) * height * 0.8 * zoom;
        let y0 = mag * height * 0.5 * zoom;

        // Yaw Rotation around Y axis
        let x1 = x0 * yaw_rad.cos() - z0 * yaw_rad.sin();
        let z1 = x0 * yaw_rad.sin() + z0 * yaw_rad.cos();

        // Pitch Rotation around X axis
        let y2 = y0 * pitch_rad.cos() - z1 * pitch_rad.sin();

        let px = center.0 + x1;
        let py = center.1 - y2; // Invert Y for screen space
        (px, py)
    }

    /// Turbo / Spectral color ramp (0.0 ..= 1.0) -> RGB
    pub fn magnitude_to_rgb(mag: f32) -> (u8, u8, u8) {
        let m = mag.clamp(0.0, 1.0);
        if m < 0.2 {
            // Navy to Purple
            let t = m / 0.2;
            let r = (20.0 + t * 70.0) as u8;
            let g = (25.0 + t * 20.0) as u8;
            let b = (90.0 + t * 140.0) as u8;
            (r, g, b)
        } else if m < 0.5 {
            // Purple to Cyan
            let t = (m - 0.2) / 0.3;
            let r = (90.0 * (1.0 - t)) as u8;
            let g = (45.0 + t * 180.0) as u8;
            let b = (230.0 + t * 25.0) as u8;
            (r, g, b)
        } else if m < 0.8 {
            // Cyan to Amber / Gold
            let t = (m - 0.5) / 0.3;
            let r = (t * 255.0) as u8;
            let g = (225.0 - t * 35.0) as u8;
            let b = (255.0 * (1.0 - t)) as u8;
            (r, g, b)
        } else {
            // Amber to Hot Pink / Magenta peak
            let t = (m - 0.8) / 0.2;
            let r = 255;
            let g = (190.0 * (1.0 - t)) as u8;
            let b = (t * 120.0) as u8;
            (r, g, b)
        }
    }

    /// ASCII preview representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[3D FFT WATERFALL SPECTROGRAM]\n");
        out.push_str(&format!(
            "Camera: Yaw {:.1} deg | Pitch {:.1} deg | Zoom {:.2}x | Status: {}\n",
            self.yaw_deg,
            self.pitch_deg,
            self.zoom,
            if self.is_frozen { "FROZEN" } else { "LIVE" }
        ));
        out.push_str(&format!(
            "Peak: {:.1} Hz ({:.1} dBFS)\n",
            self.peak_frequency_hz, self.peak_magnitude_db
        ));
        out.push_str("Recent Slice Levels: [");
        if let Some(first) = self.slices.first() {
            for v in first.iter().step_by(8) {
                let char_bar = match (v * 8.0) as usize {
                    0 => ' ',
                    1 => '.',
                    2 => ':',
                    3 => '-',
                    4 => '=',
                    5 => '+',
                    6 => '*',
                    _ => '#',
                };
                out.push(char_bar);
            }
        }
        out.push_str("]\n");
        out
    }
}

#[cfg(feature = "gui")]
impl Spectrogram3DView {
    /// Render egui 3D Spectrogram visualizer widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header with Telemetry & Controls
            ui.horizontal(|ui| {
                ui.heading("3D FFT WATERFALL SPECTROGRAM");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Reset View Button (>= 44x44pt)
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Reset View").size(12.0))
                                .min_size(Vec2::new(MIN_HIT_TARGET_PT, 36.0)),
                        )
                        .clicked()
                    {
                        self.reset_camera();
                    }

                    // Freeze Toggle
                    let freeze_text = if self.is_frozen {
                        "LIVE (Resume)"
                    } else {
                        "FREEZE"
                    };
                    let freeze_color = if self.is_frozen {
                        Color32::from_rgb(255, 107, 43)
                    } else {
                        Color32::from_rgb(0, 229, 255)
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(freeze_text)
                                    .size(12.0)
                                    .color(freeze_color),
                            )
                            .min_size(Vec2::new(MIN_HIT_TARGET_PT, 36.0)),
                        )
                        .clicked()
                    {
                        self.is_frozen = !self.is_frozen;
                    }

                    // Peak Readout Badge
                    let peak_label = format!(
                        "Peak: {:.0}Hz ({:.1}dB)",
                        self.peak_frequency_hz, self.peak_magnitude_db
                    );
                    ui.label(
                        egui::RichText::new(peak_label)
                            .size(12.0)
                            .color(Color32::from_rgb(255, 215, 0)),
                    );
                });
            });

            ui.add_space(6.0);

            // 3D Canvas
            let canvas_size = Vec2::new(ui.available_width().max(420.0), 320.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let center = (rect.center().x, rect.center().y);

            // Mouse Drag Orbit interaction
            if response.dragged() {
                let delta = response.drag_delta();
                self.yaw_deg += delta.x * 0.4;
                self.pitch_deg = (self.pitch_deg - delta.y * 0.4).clamp(10.0, 85.0);
            }

            // Dark Canvas Background
            painter.rect_filled(rect, 8.0, Color32::from_rgb(10, 14, 24));
            painter.rect_stroke(
                rect,
                8.0,
                Stroke::new(1.5_f32, Color32::from_rgb(35, 50, 75)),
            );

            let width = rect.width() * 0.75;
            let height = rect.height() * 0.75;

            // Draw Frequency Grid Axis Baseline
            for marker in &FREQ_MARKERS {
                let (gx, gy) = Self::project_3d_point(
                    marker.bin_fraction,
                    0.0,
                    0.0,
                    center,
                    width,
                    height,
                    self.yaw_deg,
                    self.pitch_deg,
                    self.zoom,
                );
                let (gx_back, gy_back) = Self::project_3d_point(
                    marker.bin_fraction,
                    1.0,
                    0.0,
                    center,
                    width,
                    height,
                    self.yaw_deg,
                    self.pitch_deg,
                    self.zoom,
                );

                painter.line_segment(
                    [Pos2::new(gx, gy), Pos2::new(gx_back, gy_back)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(70, 90, 130, 70)),
                );

                painter.text(
                    Pos2::new(gx, gy + 8.0),
                    egui::Align2::CENTER_TOP,
                    marker.label,
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(140, 165, 195),
                );
            }

            // Render Historical Slices (Oldest to Newest for painter's algorithm)
            for (s_idx, slice) in self.slices.iter().enumerate().rev() {
                let time_norm = s_idx as f32 / NUM_HISTORY_SLICES as f32;
                let alpha = (1.0 - time_norm * 0.6).clamp(0.2, 1.0);

                let mut prev_pt: Option<Pos2> = None;
                for (b_idx, &mag) in slice.iter().enumerate() {
                    let freq_norm = b_idx as f32 / NUM_FFT_BINS as f32;
                    let (px, py) = Self::project_3d_point(
                        freq_norm,
                        time_norm,
                        mag,
                        center,
                        width,
                        height,
                        self.yaw_deg,
                        self.pitch_deg,
                        self.zoom,
                    );
                    let current_pt = Pos2::new(px, py);

                    if let Some(prev) = prev_pt {
                        let (r, g, b) = Self::magnitude_to_rgb(mag);
                        let col = Color32::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8);
                        let stroke_w = if s_idx == 0 { 2.0_f32 } else { 1.2_f32 };
                        painter.line_segment([prev, current_pt], Stroke::new(stroke_w, col));
                    }
                    prev_pt = Some(current_pt);
                }
            }

            // Status Bar at Bottom
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Orbit: Yaw {:.0}° | Pitch {:.0}° | Slices: {} | Resolution: {} bins",
                        self.yaw_deg, self.pitch_deg, NUM_HISTORY_SLICES, NUM_FFT_BINS
                    ))
                    .size(11.0)
                    .color(Color32::from_rgb(160, 180, 205)),
                );
            });
        })
        .response
    }
}
