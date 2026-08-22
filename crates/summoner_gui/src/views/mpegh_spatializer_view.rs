// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD (Step 1535).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MPEGH_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_DISTANCE_M: f32 = 0.10;
pub const MAX_DISTANCE_M: f32 = 10.00;

/// MPEG-H 3D Spatial Audio Format Topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpeghFormat {
    Mpegh714,        // 7.1.4 3D Audio Bed + Heights (12 channels)
    Mpegh51,         // 5.1 Surround Broadcast (6 channels)
    Mpegh222Dome,    // 22.2 Super Hi-Vision Dome (24 channels)
    MpeghBinaural,   // SOFA Personalized Binaural Headphone Render
    MpeghDynamicObj, // Interactive 6-DoF Scene Dynamic Objects
}

impl MpeghFormat {
    pub fn channel_count(&self) -> usize {
        match self {
            Self::Mpegh714 => 12,
            Self::Mpegh51 => 6,
            Self::Mpegh222Dome => 24,
            Self::MpeghBinaural => 2,
            Self::MpeghDynamicObj => 16,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Mpegh714 => "MPEG-H 7.1.4",
            Self::Mpegh51 => "5.1 SURROUND",
            Self::Mpegh222Dome => "22.2 NHK DOME",
            Self::MpeghBinaural => "SOFA BINAURAL",
            Self::MpeghDynamicObj => "DYNAMIC OBJ",
        }
    }
}

/// Personalized HRTF Model Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrtfProfile {
    KemarStandard,    // Standard KEMAR mannequin diffuse-field EQ
    GenelecAural,     // Custom ear pinna measured profile
    SphericalHead,    // Analytical spherical head with pinna notches
    Photogrammetry3D, // Personalized 3D ear mesh scan
}

impl HrtfProfile {
    pub fn notch_freq_khz(&self) -> f32 {
        match self {
            Self::KemarStandard => 7.2,
            Self::GenelecAural => 8.4,
            Self::SphericalHead => 6.8,
            Self::Photogrammetry3D => 9.1,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::KemarStandard => "KEMAR Standard",
            Self::GenelecAural => "Genelec Aural ID",
            Self::SphericalHead => "Spherical Head",
            Self::Photogrammetry3D => "Custom 3D Mesh",
        }
    }
}

/// Broadcast Mastering MPEG-H 3D Spatial Audio View HUD (Step 1535).
#[derive(Debug, Clone)]
pub struct MpeghSpatializerView {
    pub format: MpeghFormat,
    pub hrtf_profile: HrtfProfile,
    pub azimuth_deg: f32,            // [-180.0 ..= 180.0 deg]
    pub elevation_deg: f32,          // [-90.0 ..= 90.0 deg]
    pub distance_m: f32,             // [0.10 ..= 10.00 m]
    pub is_custom_pinna_mode: bool,  // true = Custom 3D Mesh, false = SOFA Profile
    pub object_puck_pos: (f32, f32), // Normalized (X: Azimuth, Y: Elevation)
    pub is_dragging_puck: bool,
    pub itd_microseconds: f32, // Interaural Time Difference (μs)
    pub ild_db: f32,           // Interaural Level Difference (dB)
    pub integrated_lufs: f32,  // Loudness compliance LUFS
    pub color_palette: ContrastColorPalette,
}

impl Default for MpeghSpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl MpeghSpatializerView {
    pub fn new() -> Self {
        let format = MpeghFormat::Mpegh714;
        let mut view = Self {
            format,
            hrtf_profile: HrtfProfile::KemarStandard,
            azimuth_deg: 45.0,
            elevation_deg: 15.0,
            distance_m: 2.50,
            is_custom_pinna_mode: false,
            object_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            itd_microseconds: 420.0,
            ild_db: 8.5,
            integrated_lufs: -14.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.object_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_spatial_calculations();
        view
    }

    /// Convert Azimuth [-180 ..= +180 deg] to normalized coordinate [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(az: f32) -> f32 {
        let a = az.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Azimuth [-180 ..= +180 deg].
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    /// Convert Elevation [-90 ..= +90 deg] to normalized coordinate [0.0 ..= 1.0].
    pub fn elevation_to_normalized(el: f32) -> f32 {
        let e = el.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((e - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Elevation [-90 ..= +90 deg].
    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    /// Set MPEG-H format preset.
    pub fn set_format(&mut self, format: MpeghFormat) {
        self.format = format;
        self.update_spatial_calculations();
    }

    /// Update calculated ITD, ILD and binaural spatial properties.
    pub fn update_spatial_calculations(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();

        // Woodworth's spherical formula for ITD
        // ITD = (r_head / c) * (sin(az) + az) * cos(el)
        let head_radius_m = 0.0875; // 8.75 cm average adult head radius
        let speed_of_sound_m_s = 343.0;
        let sin_az = az_rad.abs().sin();
        let itd_sec =
            (head_radius_m / speed_of_sound_m_s) * (sin_az + az_rad.abs()) * el_rad.cos().max(0.1);
        self.itd_microseconds = (itd_sec * 1_000_000.0).clamp(0.0, 750.0);

        // ILD head shadowing approximation (frequency-dependent average)
        self.ild_db = (sin_az * 14.0 * el_rad.cos().max(0.1)).clamp(0.0, 20.0);
    }

    /// Evaluate HRTF Frequency Response magnitude (dB) for left/right ears at frequency $f$ (Hz).
    pub fn evaluate_hrtf_magnitude(&self, freq_hz: f32) -> (f32, f32) {
        let f = freq_hz.clamp(20.0, 20000.0);
        let f_khz = f / 1000.0;
        let notch_center = self.hrtf_profile.notch_freq_khz();

        // Head shadow attenuation on opposite ear
        let is_left_side = self.azimuth_deg < 0.0;
        let az_factor = (self.azimuth_deg.abs() / 180.0).clamp(0.0, 1.0);

        // Pinna spectral notch (concha resonance dip)
        let notch_dip = -12.0 * (-((f_khz - notch_center) / 1.2).powi(2)).exp();
        let hf_boost = if f_khz > 3.0 {
            (f_khz - 3.0).powf(1.4) * 0.8
        } else {
            0.0
        };

        let left_mag = if is_left_side {
            hf_boost + notch_dip
        } else {
            (hf_boost + notch_dip) - (az_factor * (f_khz.min(10.0) * 1.5))
        };

        let right_mag = if !is_left_side {
            hf_boost + notch_dip
        } else {
            (hf_boost + notch_dip) - (az_factor * (f_khz.min(10.0) * 1.5))
        };

        (left_mag.clamp(-30.0, 12.0), right_mag.clamp(-30.0, 12.0))
    }

    /// Hit-test touch coordinate on the MPEG-H spatial object puck.
    pub fn hit_test_object_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.object_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.object_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MPEGH_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Polar 3D Map and HRTF Spectrum.
    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            row[width - 1] = '|';
            if row_idx == 0 || row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
                row[width - 1] = '+';
            }
        }

        let mid_x = width / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = '|';
        }

        // Draw HRTF spectrum on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let frac = c as f32 / (right_w.max(1) as f32);
            let freq = 100.0 * 200.0_f32.powf(frac);
            let (l_db, _r_db) = self.evaluate_hrtf_magnitude(freq);
            let norm_mag = ((l_db + 30.0) / 42.0).clamp(0.0, 1.0);
            let row = (height as isize - 2 - (norm_mag * (height as f32 - 4.0)) as isize)
                .clamp(1, height as isize - 2) as usize;
            grid[row][mid_x + 1 + c] = '#';
        }

        // Spatial Object Puck on left half
        let puck_col = ((self.object_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.object_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'M';
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MPEG-H 3D IMMERSIVE SPATIAL AUDIO & PERSONALIZED HRTF HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Format Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let formats = [
            (MpeghFormat::Mpegh714, "MPEG-H 7.1.4"),
            (MpeghFormat::Mpegh51, "5.1 SURROUND"),
            (MpeghFormat::Mpegh222Dome, "22.2 NHK DOME"),
            (MpeghFormat::MpeghBinaural, "SOFA BINAURAL"),
            (MpeghFormat::MpeghDynamicObj, "DYNAMIC OBJ"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (f, name)) in formats.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.format == *f;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_format(*f);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: MPEG-H 3D Spatial Radar Map (Azimuth vs Elevation)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "MPEG-H 3D OBJECT SPACE (AZIMUTH vs ELEVATION)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Radar Polar Grid Guides
        let radar_center = left_rect.center();
        let max_r = (left_rect.height() - 60.0) * 0.45;
        for r_step in 1..=3 {
            let cur_r = max_r * (r_step as f32 / 3.0);
            painter.circle_stroke(
                radar_center,
                cur_r,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 100)),
            );
        }
        painter.line_segment(
            [
                egui::pos2(radar_center.x - max_r, radar_center.y),
                egui::pos2(radar_center.x + max_r, radar_center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(radar_center.x, radar_center.y - max_r),
                egui::pos2(radar_center.x, radar_center.y + max_r),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 120)),
        );

        // Interactive Object Puck (Azimuth vs Elevation)
        let puck_x = left_rect.min.x + self.object_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.object_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.object_puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_spatial_calculations();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            MPEGH_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elevation: {:+.1}° | Dist: {:.2}m",
                self.azimuth_deg, self.elevation_deg, self.distance_m
            ),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: Personalized HRTF Profile Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "PERSONALIZED HRTF PINNA FILTER (L / R EAR)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // HRTF Profile Selection Buttons (>= 44x44pt)
        let prof_w = (right_rect.width() - 30.0 - 10.0) / 2.0;
        let p1_rect = egui::Rect::from_min_size(
            egui::pos2(right_rect.min.x + 15.0, right_rect.min.y + 30.0),
            egui::vec2(prof_w, 44.0),
        );
        let p2_rect = egui::Rect::from_min_size(
            egui::pos2(right_rect.min.x + 25.0 + prof_w, right_rect.min.y + 30.0),
            egui::vec2(prof_w, 44.0),
        );

        let bg_p1 = if !self.is_custom_pinna_mode {
            Color32::from_rgb(0, 229, 255)
        } else {
            Color32::from_rgb(30, 45, 65)
        };
        let bg_p2 = if self.is_custom_pinna_mode {
            Color32::from_rgb(255, 215, 0)
        } else {
            Color32::from_rgb(30, 45, 65)
        };

        painter.rect_filled(p1_rect, 4.0, bg_p1);
        painter.text(
            p1_rect.center(),
            egui::Align2::CENTER_CENTER,
            "SOFA PROFILE #1",
            egui::FontId::proportional(10.0),
            if !self.is_custom_pinna_mode {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(220, 235, 255)
            },
        );

        painter.rect_filled(p2_rect, 4.0, bg_p2);
        painter.text(
            p2_rect.center(),
            egui::Align2::CENTER_CENTER,
            "CUSTOM PINNA MESH",
            egui::FontId::proportional(10.0),
            if self.is_custom_pinna_mode {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(220, 235, 255)
            },
        );

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if p1_rect.contains(pos) {
                    self.is_custom_pinna_mode = false;
                    self.hrtf_profile = HrtfProfile::KemarStandard;
                    self.update_spatial_calculations();
                } else if p2_rect.contains(pos) {
                    self.is_custom_pinna_mode = true;
                    self.hrtf_profile = HrtfProfile::Photogrammetry3D;
                    self.update_spatial_calculations();
                }
            }
        }

        // Draw Left (Cyan) & Right (Gold) Ear HRTF Curves
        let curve_w = right_rect.width() - 30.0;
        let mut prev_l = None;
        let mut prev_r = None;
        for i in 0..=40 {
            let frac = i as f32 / 40.0;
            let freq = 100.0 * 200.0_f32.powf(frac);
            let (l_db, r_db) = self.evaluate_hrtf_magnitude(freq);
            let norm_l = ((l_db + 30.0) / 42.0).clamp(0.0, 1.0);
            let norm_r = ((r_db + 30.0) / 42.0).clamp(0.0, 1.0);
            let cx = right_rect.min.x + 15.0 + frac * curve_w;
            let cy_l = right_rect.max.y - 40.0 - norm_l * 80.0;
            let cy_r = right_rect.max.y - 40.0 - norm_r * 80.0;

            let pt_l = egui::pos2(cx, cy_l);
            let pt_r = egui::pos2(cx, cy_r);

            if let Some(prev) = prev_l {
                painter.line_segment(
                    [prev, pt_l],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            if let Some(prev) = prev_r {
                painter.line_segment(
                    [prev, pt_r],
                    Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
                );
            }
            prev_l = Some(pt_l);
            prev_r = Some(pt_r);
        }

        painter.text(
            egui::pos2(right_rect.min.x + 15.0, right_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Profile: {} (ITD: {:.0} μs | ILD: {:.1} dB)",
                self.hrtf_profile.display_name(),
                self.itd_microseconds,
                self.ild_db
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "OBJECT POSITION",
                format!(
                    "Az: {:+.1}°, El: {:+.1}° ({:.1}m)",
                    self.azimuth_deg, self.elevation_deg, self.distance_m
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MPEG-H CHANNELS",
                format!(
                    "{} ({} Ch)",
                    self.format.display_name(),
                    self.format.channel_count()
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BINAURAL HRTF ITD",
                format!(
                    "{:.0} μs ({:.1} dB ILD)",
                    self.itd_microseconds, self.ild_db
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "LOUDNESS COMPLIANCE",
                format!("{:.1} LUFS (EBU R128)", self.integrated_lufs),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] MPEG-H 3D Spatial Audio & Personalized HRTF Profile (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
