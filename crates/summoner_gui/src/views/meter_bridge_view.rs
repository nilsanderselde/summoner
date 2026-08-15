// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Multi-Track Peak Metering Bridge Panel with Peak Hold Decay & Clip Indicators (Step 1324).

use crate::touch_controls::MIN_HIT_TARGET_PT;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

pub const DB_MIN: f32 = -60.0;
pub const DB_MAX: f32 = 6.0;
pub const METER_BAR_WIDTH: f32 = 14.0;
pub const METER_STRIP_WIDTH: f32 = 64.0;
pub const METER_HEIGHT: f32 = 220.0;

/// Peak / RMS channel meter state
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMeterState {
    pub track_id: u64,
    pub track_name: String,
    pub peak_l_db: f32,
    pub peak_r_db: f32,
    pub rms_l_db: f32,
    pub rms_r_db: f32,
    pub peak_hold_l_db: f32,
    pub peak_hold_r_db: f32,
    pub clipped_l: bool,
    pub clipped_r: bool,
    pub mute: bool,
    pub solo: bool,
}

impl ChannelMeterState {
    pub fn new(track_id: u64, track_name: impl Into<String>) -> Self {
        Self {
            track_id,
            track_name: track_name.into(),
            peak_l_db: -60.0,
            peak_r_db: -60.0,
            rms_l_db: -60.0,
            rms_r_db: -60.0,
            peak_hold_l_db: -60.0,
            peak_hold_r_db: -60.0,
            clipped_l: false,
            clipped_r: false,
            mute: false,
            solo: false,
        }
    }

    /// Update meter levels with peak hold decay
    pub fn update_levels(
        &mut self,
        peak_l: f32,
        peak_r: f32,
        rms_l: f32,
        rms_r: f32,
        decay_db: f32,
    ) {
        self.peak_l_db = peak_l.clamp(DB_MIN, DB_MAX);
        self.peak_r_db = peak_r.clamp(DB_MIN, DB_MAX);
        self.rms_l_db = rms_l.clamp(DB_MIN, DB_MAX);
        self.rms_r_db = rms_r.clamp(DB_MIN, DB_MAX);

        // Peak Hold L
        if self.peak_l_db >= self.peak_hold_l_db {
            self.peak_hold_l_db = self.peak_l_db;
        } else {
            self.peak_hold_l_db = (self.peak_hold_l_db - decay_db).max(self.peak_l_db);
        }

        // Peak Hold R
        if self.peak_r_db >= self.peak_hold_r_db {
            self.peak_hold_r_db = self.peak_r_db;
        } else {
            self.peak_hold_r_db = (self.peak_hold_r_db - decay_db).max(self.peak_r_db);
        }

        // Clip detection (> 0.0 dBFS)
        if self.peak_l_db >= 0.0 {
            self.clipped_l = true;
        }
        if self.peak_r_db >= 0.0 {
            self.clipped_r = true;
        }
    }

    /// Reset clip indicator
    pub fn reset_clip(&mut self) {
        self.clipped_l = false;
        self.clipped_r = false;
    }
}

/// Interactive Multi-Track Peak Metering Bridge View (Step 1324).
#[derive(Debug, Clone)]
pub struct MeterBridgeView {
    pub channels: Vec<ChannelMeterState>,
    pub master_channel: ChannelMeterState,
    pub peak_hold_decay_rate_db_per_frame: f32,
}

impl Default for MeterBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl MeterBridgeView {
    pub fn new() -> Self {
        let channels = vec![
            ChannelMeterState {
                track_id: 1,
                track_name: "Kick / Snare".into(),
                peak_l_db: -4.2,
                peak_r_db: -4.5,
                rms_l_db: -12.0,
                rms_r_db: -12.5,
                peak_hold_l_db: -1.2,
                peak_hold_r_db: -1.5,
                clipped_l: false,
                clipped_r: false,
                mute: false,
                solo: false,
            },
            ChannelMeterState {
                track_id: 2,
                track_name: "Sub Bass".into(),
                peak_l_db: -6.0,
                peak_r_db: -6.0,
                rms_l_db: -10.5,
                rms_r_db: -10.5,
                peak_hold_l_db: -3.0,
                peak_hold_r_db: -3.0,
                clipped_l: false,
                clipped_r: false,
                mute: false,
                solo: false,
            },
            ChannelMeterState {
                track_id: 3,
                track_name: "Lead Synth".into(),
                peak_l_db: -9.5,
                peak_r_db: -8.2,
                rms_l_db: -16.0,
                rms_r_db: -15.0,
                peak_hold_l_db: -6.0,
                peak_hold_r_db: -5.5,
                clipped_l: false,
                clipped_r: false,
                mute: false,
                solo: false,
            },
            ChannelMeterState {
                track_id: 4,
                track_name: "Reverb FX".into(),
                peak_l_db: -14.0,
                peak_r_db: -13.5,
                rms_l_db: -22.0,
                rms_r_db: -21.0,
                peak_hold_l_db: -11.0,
                peak_hold_r_db: -10.5,
                clipped_l: false,
                clipped_r: false,
                mute: false,
                solo: false,
            },
        ];

        let master_channel = ChannelMeterState {
            track_id: 0,
            track_name: "MASTER BUS".into(),
            peak_l_db: -1.8,
            peak_r_db: -1.5,
            rms_l_db: -8.5,
            rms_r_db: -8.0,
            peak_hold_l_db: -0.2,
            peak_hold_r_db: -0.1,
            clipped_l: false,
            clipped_r: false,
            mute: false,
            solo: false,
        };

        Self {
            channels,
            master_channel,
            peak_hold_decay_rate_db_per_frame: 0.15,
        }
    }

    /// Reset all clip indicators across all channels
    pub fn reset_all_clips(&mut self) {
        for ch in &mut self.channels {
            ch.reset_clip();
        }
        self.master_channel.reset_clip();
    }

    /// Convert dB level to normalized fraction (0.0 ..= 1.0) along meter height
    pub fn db_to_fraction(db: f32) -> f32 {
        if db <= DB_MIN {
            0.0
        } else if db >= DB_MAX {
            1.0
        } else {
            (db - DB_MIN) / (DB_MAX - DB_MIN)
        }
    }

    /// Get color for dB level in standard broadcast metering gradient
    pub fn db_to_color(db: f32) -> (u8, u8, u8) {
        if db >= 0.0 {
            (255, 40, 60) // Red clip / warning
        } else if db >= -6.0 {
            (255, 180, 20) // Amber / orange high level
        } else if db >= -18.0 {
            (240, 220, 40) // Yellow nominal level
        } else {
            (0, 220, 140) // Green safe level
        }
    }

    /// Render ASCII summary of meter bridge
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[MULTI-TRACK METER BRIDGE]\n");
        for ch in &self.channels {
            out.push_str(&format!(
                " - {:<14} : L: {:+5.1}dB (PeakHold: {:+5.1}dB) | R: {:+5.1}dB (PeakHold: {:+5.1}dB) [Clip: {}]\n",
                ch.track_name, ch.peak_l_db, ch.peak_hold_l_db, ch.peak_r_db, ch.peak_hold_r_db,
                if ch.clipped_l || ch.clipped_r { "CLIP" } else { "OK" }
            ));
        }
        out.push_str(&format!(
            " - {:<14} : L: {:+5.1}dB (PeakHold: {:+5.1}dB) | R: {:+5.1}dB (PeakHold: {:+5.1}dB) [Clip: {}]\n",
            self.master_channel.track_name,
            self.master_channel.peak_l_db,
            self.master_channel.peak_hold_l_db,
            self.master_channel.peak_r_db,
            self.master_channel.peak_hold_r_db,
            if self.master_channel.clipped_l || self.master_channel.clipped_r { "CLIP" } else { "OK" }
        ));
        out
    }
}

#[cfg(feature = "gui")]
impl MeterBridgeView {
    /// Render egui Meter Bridge UI
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let decay = self.peak_hold_decay_rate_db_per_frame;
        for ch in &mut self.channels {
            ch.update_levels(ch.peak_l_db, ch.peak_r_db, ch.rms_l_db, ch.rms_r_db, decay);
        }
        self.master_channel.update_levels(
            self.master_channel.peak_l_db,
            self.master_channel.peak_r_db,
            self.master_channel.rms_l_db,
            self.master_channel.rms_r_db,
            decay,
        );

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("MULTI-TRACK PEAK METERING BRIDGE");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Reset All Clips").size(12.0))
                                .min_size(Vec2::new(MIN_HIT_TARGET_PT, 34.0)),
                        )
                        .clicked()
                    {
                        self.reset_all_clips();
                    }
                });
            });

            ui.add_space(8.0);

            // Channel Strips Container
            ui.horizontal(|ui| {
                // dB Scale Legend Column
                self.render_db_scale_column(ui);

                ui.add_space(6.0);

                // Track Meters
                for ch in &mut self.channels {
                    Self::render_channel_strip(ui, ch, false);
                    ui.add_space(4.0);
                }

                ui.separator();

                // Master Bus Meter
                Self::render_channel_strip(ui, &mut self.master_channel, true);
            });
        })
        .response
    }

    fn render_db_scale_column(&self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(32.0, METER_HEIGHT + 60.0), egui::Sense::hover());
        let rect = response.rect;
        let meter_top = rect.min.y + 40.0;
        let meter_bottom = meter_top + METER_HEIGHT;

        let marks = [
            6.0, 0.0, -3.0, -6.0, -12.0, -18.0, -24.0, -36.0, -48.0, -60.0,
        ];
        for &db in &marks {
            let frac = Self::db_to_fraction(db);
            let y = meter_bottom - frac * METER_HEIGHT;
            let label = if db == 0.0 {
                " 0".to_string()
            } else if db > 0.0 {
                format!("+{:.0}", db)
            } else {
                format!("{:.0}", db)
            };

            painter.line_segment(
                [Pos2::new(rect.max.x - 6.0, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0_f32, Color32::from_rgb(100, 120, 150)),
            );
            painter.text(
                Pos2::new(rect.max.x - 8.0, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(150, 175, 205),
            );
        }
    }

    fn render_channel_strip(ui: &mut egui::Ui, ch: &mut ChannelMeterState, is_master: bool) {
        ui.vertical(|ui| {
            ui.set_width(METER_STRIP_WIDTH);

            // Clip Indicator LED Button (>= 44x44pt hit target)
            let is_clipped = ch.clipped_l || ch.clipped_r;
            let clip_color = if is_clipped {
                Color32::from_rgb(255, 30, 40)
            } else {
                Color32::from_rgb(45, 55, 75)
            };
            let clip_text = if is_clipped { "CLIP" } else { "OK" };

            let clip_btn =
                egui::Button::new(egui::RichText::new(clip_text).size(11.0).strong().color(
                    if is_clipped {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(140, 160, 180)
                    },
                ))
                .fill(clip_color)
                .min_size(Vec2::new(MIN_HIT_TARGET_PT, 28.0));

            if ui.add(clip_btn).clicked() {
                ch.reset_clip();
            }

            ui.add_space(4.0);

            // Stereo Meter Bars (L & R)
            let (response, painter) = ui.allocate_painter(
                Vec2::new(METER_STRIP_WIDTH, METER_HEIGHT),
                egui::Sense::hover(),
            );
            let rect = response.rect;

            // Background
            painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 28));
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)),
            );

            let bar_w = METER_BAR_WIDTH;
            let left_x = rect.min.x + 12.0;
            let right_x = left_x + bar_w + 4.0;

            // Render Left Bar
            Self::draw_single_meter_bar(
                &painter,
                left_x,
                rect.min.y,
                bar_w,
                METER_HEIGHT,
                ch.peak_l_db,
                ch.peak_hold_l_db,
            );
            // Render Right Bar
            Self::draw_single_meter_bar(
                &painter,
                right_x,
                rect.min.y,
                bar_w,
                METER_HEIGHT,
                ch.peak_r_db,
                ch.peak_hold_r_db,
            );

            ui.add_space(4.0);

            // Track Label
            let label_col = if is_master {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };
            ui.label(
                egui::RichText::new(&ch.track_name)
                    .size(11.0)
                    .strong()
                    .color(label_col),
            );

            // Mute / Solo Buttons (Touch targets >= 44x44pt)
            if !is_master {
                ui.horizontal(|ui| {
                    let m_color = if ch.mute {
                        Color32::from_rgb(255, 60, 60)
                    } else {
                        Color32::from_rgb(45, 55, 75)
                    };
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("M").size(10.0))
                                .fill(m_color)
                                .min_size(Vec2::new(26.0, 26.0)),
                        )
                        .clicked()
                    {
                        ch.mute = !ch.mute;
                    }
                    let s_color = if ch.solo {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(45, 55, 75)
                    };
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("S").size(10.0).color(
                                if ch.solo {
                                    Color32::BLACK
                                } else {
                                    Color32::WHITE
                                },
                            ))
                            .fill(s_color)
                            .min_size(Vec2::new(26.0, 26.0)),
                        )
                        .clicked()
                    {
                        ch.solo = !ch.solo;
                    }
                });
            }
        });
    }

    fn draw_single_meter_bar(
        painter: &egui::Painter,
        x: f32,
        top_y: f32,
        width: f32,
        height: f32,
        peak_db: f32,
        peak_hold_db: f32,
    ) {
        let bottom_y = top_y + height;
        let frac = Self::db_to_fraction(peak_db);
        let bar_h = frac * height;
        let bar_top_y = bottom_y - bar_h;

        // Meter Track Background
        let track_rect = egui::Rect::from_min_size(Pos2::new(x, top_y), Vec2::new(width, height));
        painter.rect_filled(track_rect, 2.0, Color32::from_rgb(20, 25, 38));

        // Active Level Fill
        if bar_h > 1.0 {
            let (r, g, b) = Self::db_to_color(peak_db);
            let fill_rect =
                egui::Rect::from_min_size(Pos2::new(x, bar_top_y), Vec2::new(width, bar_h));
            painter.rect_filled(fill_rect, 2.0, Color32::from_rgb(r, g, b));
        }

        // Peak Hold Tick Line
        let hold_frac = Self::db_to_fraction(peak_hold_db);
        let hold_y = bottom_y - hold_frac * height;
        let (hr, hg, hb) = Self::db_to_color(peak_hold_db);
        painter.line_segment(
            [Pos2::new(x, hold_y), Pos2::new(x + width, hold_y)],
            Stroke::new(2.0_f32, Color32::from_rgb(hr, hg, hb)),
        );
    }
}
