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

//! DAW Transport Control Bar & Quick Access Header Toolbar widget (`TransportBarView`).

use std::time::Instant;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Rect, Sense, Stroke, Vec2};

/// Supported DAW Time Signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeSignature {
    FourFour,
    ThreeFour,
    SixEight,
    SevenEight,
}

impl TimeSignature {
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeSignature::FourFour => "4/4",
            TimeSignature::ThreeFour => "3/4",
            TimeSignature::SixEight => "6/8",
            TimeSignature::SevenEight => "7/8",
        }
    }

    pub fn beats_per_bar(&self) -> u32 {
        match self {
            TimeSignature::FourFour => 4,
            TimeSignature::ThreeFour => 3,
            TimeSignature::SixEight => 6,
            TimeSignature::SevenEight => 7,
        }
    }
}

/// GUI visual themes for theme switching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SelectedTheme {
    SummonerDark,
    LightTheme,
    HighContrast,
}

impl SelectedTheme {
    pub fn name(&self) -> &'static str {
        match self {
            SelectedTheme::SummonerDark => "Dark (Summoner)",
            SelectedTheme::LightTheme => "Light",
            SelectedTheme::HighContrast => "High Contrast",
        }
    }
}

/// DAW Transport Control Bar & Quick Access Header Toolbar Widget (`TransportBarView`).
pub struct TransportBarView {
    pub is_playing: bool,
    pub is_recording: bool,
    pub loop_enabled: bool,
    pub is_paused: bool,

    pub bpm: f32,
    pub time_signature: TimeSignature,

    pub tap_timestamps: Vec<Instant>,

    pub master_volume_db: f32,
    pub master_peak_l: f32,
    pub master_peak_r: f32,

    pub cpu_usage_pct: f32,
    pub memory_usage_mb: usize,

    pub current_theme: SelectedTheme,

    pub min_button_size: f32,
}

impl Default for TransportBarView {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            loop_enabled: false,
            is_paused: false,
            bpm: 120.0,
            time_signature: TimeSignature::FourFour,
            tap_timestamps: Vec::new(),
            master_volume_db: 0.0,
            master_peak_l: 0.0,
            master_peak_r: 0.0,
            cpu_usage_pct: 12.5,
            memory_usage_mb: 480,
            current_theme: SelectedTheme::SummonerDark,
            min_button_size: 44.0,
        }
    }
}

impl TransportBarView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.is_paused = false;
        }
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
    }

    pub fn toggle_record(&mut self) {
        self.is_recording = !self.is_recording;
    }

    pub fn toggle_loop(&mut self) {
        self.loop_enabled = !self.loop_enabled;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(20.0, 300.0);
    }

    /// Tap tempo calculation updating `bpm` based on real-time tap intervals.
    pub fn tap_tempo(&mut self) {
        let now = Instant::now();
        self.tap_timestamps
            .retain(|t| now.duration_since(*t).as_secs_f32() < 2.0);
        self.tap_timestamps.push(now);

        if self.tap_timestamps.len() >= 2 {
            let mut total_duration = 0.0;
            let count = self.tap_timestamps.len() - 1;
            for i in 0..count {
                let delta = self.tap_timestamps[i + 1]
                    .duration_since(self.tap_timestamps[i])
                    .as_secs_f32();
                total_duration += delta;
            }
            let avg_interval = total_duration / count as f32;
            if avg_interval > 0.05 {
                let calculated_bpm = 60.0 / avg_interval;
                self.set_bpm(calculated_bpm);
            }
        }
    }

    /// Calculate BPM directly from interval in milliseconds (useful for testing or MIDI clock sync).
    pub fn tap_tempo_with_interval_ms(&mut self, interval_ms: f32) {
        if interval_ms > 50.0 {
            let calculated_bpm = 60000.0 / interval_ms;
            self.set_bpm(calculated_bpm);
        }
    }

    /// Get platform-aware system toolbar padding / margin bounds.
    pub fn platform_aware_padding(&self) -> (f32, f32, f32, f32) {
        let display_server = crate::platform::detect_display_server();
        match display_server {
            crate::platform::DisplayServer::MacOsQuartz => (28.0, 12.0, 8.0, 12.0),
            crate::platform::DisplayServer::WindowsDesktop => (8.0, 12.0, 8.0, 12.0),
            crate::platform::DisplayServer::Wayland | crate::platform::DisplayServer::X11 => {
                (10.0, 10.0, 10.0, 10.0)
            }
            _ => (8.0, 8.0, 8.0, 8.0),
        }
    }
}

#[cfg(feature = "gui")]
impl TransportBarView {
    /// Render DAW Transport Control Bar toolbar.
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let (top_pad, right_pad, bottom_pad, left_pad) = self.platform_aware_padding();

        let frame = egui::Frame::none()
            .fill(ui.visuals().window_fill)
            .inner_margin(egui::Margin {
                top: top_pad,
                right: right_pad,
                bottom: bottom_pad,
                left: left_pad,
            })
            .stroke(Stroke::new(1.0_f32, Color32::from_gray(50)));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let btn_size = Vec2::new(
                    self.min_button_size.max(44.0),
                    self.min_button_size.max(44.0),
                );

                ui.group(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

                    let play_color = if self.is_playing {
                        Color32::from_rgb(0, 220, 100)
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(egui::RichText::new("▶ Play").strong().color(
                                if self.is_playing {
                                    Color32::BLACK
                                } else {
                                    Color32::WHITE
                                },
                            ))
                            .fill(play_color),
                        )
                        .clicked()
                    {
                        self.toggle_play();
                    }

                    if ui
                        .add_sized(btn_size, egui::Button::new("⏹ Stop"))
                        .clicked()
                    {
                        self.stop();
                    }

                    let rec_color = if self.is_recording {
                        Color32::from_rgb(255, 50, 50)
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(egui::RichText::new("⏺ Rec").strong().color(
                                if self.is_recording {
                                    Color32::WHITE
                                } else {
                                    Color32::RED
                                },
                            ))
                            .fill(rec_color),
                        )
                        .clicked()
                    {
                        self.toggle_record();
                    }

                    let loop_color = if self.loop_enabled {
                        Color32::from_rgb(26, 140, 255)
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(egui::RichText::new("🔁 Loop").strong().color(
                                if self.loop_enabled {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(180)
                                },
                            ))
                            .fill(loop_color),
                        )
                        .clicked()
                    {
                        self.toggle_loop();
                    }
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Tempo:").strong());
                    ui.add(
                        egui::DragValue::new(&mut self.bpm)
                            .range(20.0..=300.0)
                            .speed(0.5)
                            .suffix(" BPM"),
                    );

                    if ui
                        .add_sized(
                            Vec2::new(70.0, self.min_button_size.max(44.0)),
                            egui::Button::new("👆 Tap"),
                        )
                        .clicked()
                    {
                        self.tap_tempo();
                    }

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Sig:").strong());
                    egui::ComboBox::from_id_source("time_sig_combo")
                        .selected_text(self.time_signature.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.time_signature,
                                TimeSignature::FourFour,
                                "4/4",
                            );
                            ui.selectable_value(
                                &mut self.time_signature,
                                TimeSignature::ThreeFour,
                                "3/4",
                            );
                            ui.selectable_value(
                                &mut self.time_signature,
                                TimeSignature::SixEight,
                                "6/8",
                            );
                            ui.selectable_value(
                                &mut self.time_signature,
                                TimeSignature::SevenEight,
                                "7/8",
                            );
                        });
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Master:").strong());
                    ui.add(
                        egui::Slider::new(&mut self.master_volume_db, -60.0..=6.0).suffix(" dB"),
                    );

                    let meter_w = 60.0;
                    let meter_h = 14.0;
                    let (rect_l, _) =
                        ui.allocate_exact_size(Vec2::new(meter_w, meter_h), Sense::hover());
                    ui.painter()
                        .rect_filled(rect_l, 2.0, Color32::from_gray(30));
                    let fill_l = (rect_l.width() * self.master_peak_l.clamp(0.0, 1.0)).max(0.0);
                    let fill_rect_l = Rect::from_min_size(rect_l.min, Vec2::new(fill_l, meter_h));
                    let l_color = if self.master_peak_l > 0.95 {
                        Color32::RED
                    } else {
                        Color32::GREEN
                    };
                    ui.painter().rect_filled(fill_rect_l, 2.0, l_color);

                    let (rect_r, _) =
                        ui.allocate_exact_size(Vec2::new(meter_w, meter_h), Sense::hover());
                    ui.painter()
                        .rect_filled(rect_r, 2.0, Color32::from_gray(30));
                    let fill_r = (rect_r.width() * self.master_peak_r.clamp(0.0, 1.0)).max(0.0);
                    let fill_rect_r = Rect::from_min_size(rect_r.min, Vec2::new(fill_r, meter_h));
                    let r_color = if self.master_peak_r > 0.95 {
                        Color32::RED
                    } else {
                        Color32::GREEN
                    };
                    ui.painter().rect_filled(fill_rect_r, 2.0, r_color);
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("CPU: {:.1}%", self.cpu_usage_pct)).small(),
                        );
                        let cpu_color = if self.cpu_usage_pct >= 85.0 {
                            Color32::RED
                        } else if self.cpu_usage_pct >= 60.0 {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        };
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(80.0, 6.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, Color32::from_gray(30));
                        let fill_rect = Rect::from_min_size(
                            rect.min,
                            Vec2::new(
                                rect.width() * (self.cpu_usage_pct / 100.0).clamp(0.0, 1.0),
                                6.0,
                            ),
                        );
                        ui.painter().rect_filled(fill_rect, 2.0, cpu_color);

                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format!("RAM: {} MB", self.memory_usage_mb))
                                .small(),
                        );
                    });
                });

                ui.add_space(12.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Theme:").strong());
                    let prev_theme = self.current_theme;
                    egui::ComboBox::from_id_source("theme_switcher_combo")
                        .selected_text(self.current_theme.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_theme,
                                SelectedTheme::SummonerDark,
                                "Dark (Summoner)",
                            );
                            ui.selectable_value(
                                &mut self.current_theme,
                                SelectedTheme::LightTheme,
                                "Light",
                            );
                            ui.selectable_value(
                                &mut self.current_theme,
                                SelectedTheme::HighContrast,
                                "High Contrast",
                            );
                        });

                    if self.current_theme != prev_theme {
                        match self.current_theme {
                            SelectedTheme::SummonerDark => {
                                crate::theme::apply_summoner_theme(ctx, 14.0)
                            }
                            SelectedTheme::LightTheme => crate::theme::apply_light_theme(ctx, 14.0),
                            SelectedTheme::HighContrast => {
                                crate::theme::apply_high_contrast_theme(ctx, 16.0)
                            }
                        }
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_toggles() {
        let mut transport = TransportBarView::new();
        assert!(!transport.is_playing);
        assert!(!transport.is_recording);
        assert!(!transport.loop_enabled);

        transport.toggle_play();
        assert!(transport.is_playing);

        transport.toggle_record();
        assert!(transport.is_recording);

        transport.toggle_loop();
        assert!(transport.loop_enabled);

        transport.stop();
        assert!(!transport.is_playing);
    }

    #[test]
    fn test_bpm_setting_and_tap_tempo() {
        let mut transport = TransportBarView::new();
        transport.set_bpm(120.0);
        assert_eq!(transport.bpm, 120.0);

        // 500ms interval corresponds to 120 BPM
        transport.tap_tempo_with_interval_ms(500.0);
        assert_eq!(transport.bpm, 120.0);

        // 400ms interval corresponds to 150 BPM
        transport.tap_tempo_with_interval_ms(400.0);
        assert_eq!(transport.bpm, 150.0);

        // Clamping bounds check
        transport.set_bpm(500.0);
        assert_eq!(transport.bpm, 300.0);
        transport.set_bpm(5.0);
        assert_eq!(transport.bpm, 20.0);
    }

    #[test]
    fn test_time_signature_properties() {
        assert_eq!(TimeSignature::FourFour.display_name(), "4/4");
        assert_eq!(TimeSignature::FourFour.beats_per_bar(), 4);
        assert_eq!(TimeSignature::ThreeFour.display_name(), "3/4");
        assert_eq!(TimeSignature::ThreeFour.beats_per_bar(), 3);
        assert_eq!(TimeSignature::SixEight.display_name(), "6/8");
        assert_eq!(TimeSignature::SixEight.beats_per_bar(), 6);
        assert_eq!(TimeSignature::SevenEight.display_name(), "7/8");
        assert_eq!(TimeSignature::SevenEight.beats_per_bar(), 7);
    }

    #[test]
    fn test_button_hit_target_bounds() {
        let transport = TransportBarView::default();
        assert!(transport.min_button_size >= 44.0);
    }

    #[test]
    fn test_platform_aware_padding() {
        let transport = TransportBarView::new();
        let (t, r, b, l) = transport.platform_aware_padding();
        assert!(t >= 8.0);
        assert!(r >= 8.0);
        assert!(b >= 8.0);
        assert!(l >= 8.0);
    }

    #[test]
    fn test_selected_theme_names() {
        assert_eq!(SelectedTheme::SummonerDark.name(), "Dark (Summoner)");
        assert_eq!(SelectedTheme::LightTheme.name(), "Light");
        assert_eq!(SelectedTheme::HighContrast.name(), "High Contrast");
    }
}
