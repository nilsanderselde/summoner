// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modern Top Bar with Global Transport, Stats Readout, VU Meters & Mode Switching.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModernViewTab {
    #[default]
    Arranger,
    PianoRoll,
    Modular,
    Mixer,
    Performance,
}

impl ModernViewTab {
    pub fn name(&self) -> &'static str {
        match self {
            ModernViewTab::Arranger => "ARRANGER",
            ModernViewTab::PianoRoll => "PIANO ROLL",
            ModernViewTab::Modular => "MODULAR",
            ModernViewTab::Mixer => "MIXER",
            ModernViewTab::Performance => "STAGE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModernTopBarState {
    pub bpm: f64,
    pub key_signature: String,
    pub time_signature: String,
    pub cpu_percent: f32,
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub master_peak_db: f32,
    pub active_tab: ModernViewTab,
}

impl Default for ModernTopBarState {
    fn default() -> Self {
        Self {
            bpm: 120.00,
            key_signature: "Am".to_string(),
            time_signature: "4/4".to_string(),
            cpu_percent: 12.0,
            is_playing: false,
            is_recording: false,
            is_looping: true,
            master_peak_db: 1.2,
            active_tab: ModernViewTab::Arranger,
        }
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_top_bar(
    ui: &mut egui::Ui,
    state: &mut ModernTopBarState,
    on_menu_click: impl FnOnce(),
) {
    let bar_height = 48.0;
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), bar_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            // Background frame
            let full_rect = ui.available_rect_before_wrap();
            let rect = Rect::from_min_size(full_rect.min, Vec2::new(ui.available_width(), bar_height));
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(10, 14, 24));
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)),
            );

            ui.add_space(12.0);

            // Brand Logo & Menu Button
            let logo_resp = ui.button(RichText::new("⚡").font(FontId::proportional(18.0)).color(Color32::from_rgb(56, 189, 248)));
            if logo_resp.clicked() {
                on_menu_click();
            }

            let menu_btn = ui.add(
                egui::Button::new(RichText::new("Menu").font(FontId::proportional(12.0)).color(Color32::from_rgb(220, 230, 245)))
                    .fill(Color32::from_rgb(24, 34, 52))
                    .rounding(Rounding::same(4.0))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
            );
            if menu_btn.clicked() {
                // Open menu
            }

            ui.add_space(16.0);

            // Transport Stats Group (BPM, Key, Sig, CPU)
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 20, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // BPM
                        ui.vertical(|ui| {
                            ui.label(RichText::new("BPM").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(format!("{:.2}", state.bpm)).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // Key
                        ui.vertical(|ui| {
                            ui.label(RichText::new("KEY").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(&state.key_signature).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // Time Sig
                        ui.vertical(|ui| {
                            ui.label(RichText::new("SIG").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(&state.time_signature).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // CPU Meter
                        ui.vertical(|ui| {
                            ui.label(RichText::new("CPU").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{:.0}%", state.cpu_percent)).font(FontId::proportional(11.0)).color(Color32::from_rgb(56, 189, 248)));
                                let (cpu_resp, cpu_painter) = ui.allocate_painter(Vec2::new(24.0, 4.0), egui::Sense::hover());
                                cpu_painter.rect_filled(cpu_resp.rect, 1.0, Color32::from_rgb(24, 34, 52));
                                let fill_w = (state.cpu_percent / 100.0).clamp(0.0, 1.0) * 24.0;
                                cpu_painter.rect_filled(
                                    Rect::from_min_size(cpu_resp.rect.min, Vec2::new(fill_w, 4.0)),
                                    1.0,
                                    Color32::from_rgb(16, 185, 129),
                                );
                            });
                        });
                    });
                });

            ui.add_space(16.0);

            // Transport Control Buttons (Play, Pause, Record, Loop)
            ui.horizontal(|ui| {
                // Play Button
                let play_fill = if state.is_playing {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(24, 34, 52)
                };
                let play_text = if state.is_playing { Color32::BLACK } else { Color32::from_rgb(56, 189, 248) };
                let play_btn = ui.add(
                    egui::Button::new(RichText::new("▶").font(FontId::proportional(13.0)).color(play_text))
                        .fill(play_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                );
                if play_btn.clicked() {
                    state.is_playing = !state.is_playing;
                }

                // Pause / Stop
                let stop_btn = ui.add(
                    egui::Button::new(RichText::new("⏸").font(FontId::proportional(12.0)).color(Color32::from_rgb(200, 215, 235)))
                        .fill(Color32::from_rgb(24, 34, 52))
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                );
                if stop_btn.clicked() {
                    state.is_playing = false;
                }

                // Record (Red Pill)
                let rec_fill = if state.is_recording {
                    Color32::from_rgb(239, 68, 68)
                } else {
                    Color32::from_rgb(45, 20, 25)
                };
                let rec_btn = ui.add(
                    egui::Button::new(RichText::new("🔴").font(FontId::proportional(11.0)).color(Color32::from_rgb(239, 68, 68)))
                        .fill(rec_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, if state.is_recording { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(80, 25, 30) }))
                );
                if rec_btn.clicked() {
                    state.is_recording = !state.is_recording;
                }

                // Loop Toggle
                let loop_fill = if state.is_looping {
                    Color32::from_rgba_unmultiplied(56, 189, 248, 40)
                } else {
                    Color32::from_rgb(24, 34, 52)
                };
                let loop_stroke = if state.is_looping {
                    Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248))
                } else {
                    Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74))
                };
                let loop_btn = ui.add(
                    egui::Button::new(RichText::new("🔁").font(FontId::proportional(11.0)).color(Color32::from_rgb(56, 189, 248)))
                        .fill(loop_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(loop_stroke)
                );
                if loop_btn.clicked() {
                    state.is_looping = !state.is_looping;
                }
            });

            ui.add_space(16.0);

            // Master Volume & Multi-Segment Gradient VU Meter
            ui.horizontal(|ui| {
                ui.label(RichText::new("Master Vol VU Meters").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));

                let (vu_resp, vu_painter) = ui.allocate_painter(Vec2::new(130.0, 10.0), egui::Sense::hover());
                let vu_rect = vu_resp.rect;
                vu_painter.rect_filled(vu_rect, 2.0, Color32::from_rgb(14, 20, 32));
                vu_painter.rect_stroke(vu_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));

                // Draw gradient bars: Green (0..60%) -> Yellow (60..80%) -> Orange (80..95%) -> Pink (95..100%)
                let segments = [
                    (0.0, 0.60, Color32::from_rgb(16, 185, 129)),
                    (0.60, 0.80, Color32::from_rgb(234, 179, 8)),
                    (0.80, 0.95, Color32::from_rgb(249, 115, 22)),
                    (0.95, 1.00, Color32::from_rgb(236, 72, 153)),
                ];
                for (s_start, s_end, color) in segments {
                    let seg_left = vu_rect.left() + (vu_rect.width() * s_start);
                    let seg_w = vu_rect.width() * (s_end - s_start);
                    let seg_rect = Rect::from_min_size(egui::pos2(seg_left, vu_rect.top() + 1.0), Vec2::new(seg_w, vu_rect.height() - 2.0));
                    vu_painter.rect_filled(seg_rect, 1.0, color);
                }

                // Peak dB readout
                let peak_color = if state.master_peak_db > 0.0 {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(16, 185, 129)
                };
                ui.label(RichText::new(format!("+{:.1}dB", state.master_peak_db)).font(FontId::proportional(10.0)).strong().color(peak_color));
            });

            // Right-aligned Tab Mode Switcher
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);

                let tabs = [
                    ModernViewTab::Mixer,
                    ModernViewTab::Modular,
                    ModernViewTab::PianoRoll,
                    ModernViewTab::Arranger,
                ];

                for tab in tabs {
                    let is_active = state.active_tab == tab;
                    let (fill, stroke_col, text_col) = if is_active {
                        (
                            Color32::from_rgba_unmultiplied(56, 189, 248, 30),
                            Color32::from_rgb(56, 189, 248),
                            Color32::from_rgb(56, 189, 248),
                        )
                    } else {
                        (
                            Color32::from_rgb(18, 24, 36),
                            Color32::from_rgb(36, 50, 74),
                            Color32::from_rgb(148, 163, 184),
                        )
                    };

                    let btn = ui.add(
                        egui::Button::new(RichText::new(tab.name()).font(FontId::proportional(11.0)).strong().color(text_col))
                            .fill(fill)
                            .rounding(Rounding::same(4.0))
                            .stroke(Stroke::new(if is_active { 1.5_f32 } else { 1.0_f32 }, stroke_col))
                            .min_size(Vec2::new(72.0, 26.0))
                    );
                    if btn.clicked() {
                        state.active_tab = tab;
                    }
                }
            });
        },
    );
}
