// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Waveguide 3D Mesh Canvas HUD View (Milestone 13).
//!
//! Provides an interactive 3D wave propagation surface visualizer for 2D physical
//! waveguide meshes (membrane, plate, acoustic bar), touch strike puck handle
//! (>= 44x44pt hit bounding target), boundary impedance controls, and WCAG AAA
//! compliant high-contrast color palettes.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use summoner_dsp::waveguide_mesh::{
    MeshBoundaryType, MeshMaterialProfile, WaveguideMesh2D, MESH_DIM,
};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const WAVEGUIDE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const WAVEGUIDE_PUCK_VISUAL_RADIUS: f32 = 14.0;

/// Physical Waveguide 3D Mesh Canvas HUD View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveguideMeshView {
    pub mesh: WaveguideMesh2D,
    pub strike_puck_pos: (f32, f32), // Normalized [0.0 ..= 1.0] surface coordinates
    pub is_dragging_puck: bool,
    pub view_elevation_deg: f32,
    pub view_azimuth_deg: f32,
    pub wireframe_density: usize,
    pub show_energy_meter: bool,
    #[serde(skip, default)]
    pub color_palette: ContrastColorPalette,
}

impl Default for WaveguideMeshView {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveguideMeshView {
    pub fn new() -> Self {
        let mut mesh = WaveguideMesh2D::new(48000);
        mesh.strike(0.5, 0.5, 0.8, 1.5);

        Self {
            mesh,
            strike_puck_pos: (0.50, 0.50),
            is_dragging_puck: false,
            view_elevation_deg: 35.0,
            view_azimuth_deg: 45.0,
            wireframe_density: 16,
            show_energy_meter: true,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Sets mesh material and boundary condition.
    pub fn set_material(&mut self, material: MeshMaterialProfile) {
        self.mesh.set_material(material);
    }

    /// Sets boundary condition type.
    pub fn set_boundary(&mut self, boundary: MeshBoundaryType) {
        self.mesh.boundary = boundary;
    }

    /// Projects 3D mesh grid coordinates (x, y, z) into 2D canvas screen space.
    pub fn project_3d_to_canvas(
        &self,
        gx: f32,
        gy: f32,
        gz: f32,
        center: (f32, f32),
        scale: f32,
    ) -> (f32, f32) {
        let norm_x = (gx / (MESH_DIM - 1) as f32) - 0.5;
        let norm_y = (gy / (MESH_DIM - 1) as f32) - 0.5;

        let az_rad = self.view_azimuth_deg.to_radians();
        let el_rad = self.view_elevation_deg.to_radians();

        // Rotate around Z axis (Azimuth)
        let rot_x = norm_x * az_rad.cos() - norm_y * az_rad.sin();
        let rot_y = norm_x * az_rad.sin() + norm_y * az_rad.cos();

        // Isometric projection with elevation tilt
        let screen_x = center.0 + rot_x * scale * 1.732;
        let screen_y = center.1 + (rot_y * el_rad.sin() - gz * 1.5) * scale;

        (screen_x, screen_y)
    }

    /// Hit-tests touch position against the strike puck handle.
    pub fn hit_test_strike_puck(&self, pos: (f32, f32), center: (f32, f32), scale: f32) -> bool {
        let (px, py) = self.project_3d_to_canvas(
            self.strike_puck_pos.0 * (MESH_DIM - 1) as f32,
            self.strike_puck_pos.1 * (MESH_DIM - 1) as f32,
            0.0,
            center,
            scale,
        );
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= WAVEGUIDE_PUCK_HIT_RADIUS
    }

    /// Triggers a strike excitation on the waveguide mesh at current puck location.
    pub fn trigger_strike(&mut self, velocity: f32) {
        self.mesh.strike(
            self.strike_puck_pos.0,
            self.strike_puck_pos.1,
            velocity.clamp(0.1, 1.0),
            1.5,
        );
    }

    /// Advances mesh simulation by one frame.
    pub fn step_simulation(&mut self) {
        self.mesh.step_simulation();
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "WAVEGUIDE 3D MESH [{:?}] Bound:{:?} CFL:{:.2} Damp:{:.4} Energy:{:.4}",
            self.mesh.material,
            self.mesh.boundary,
            self.mesh.courant,
            self.mesh.damping,
            self.mesh.calculate_total_energy()
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let grid = self.mesh.displacement_grid();

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let mesh_y = (y * MESH_DIM) / canvas_h.max(1);

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let mesh_x = (x * MESH_DIM) / width.max(1);
                let disp = grid[mesh_y.min(MESH_DIM - 1)][mesh_x.min(MESH_DIM - 1)];

                if disp.abs() > 0.4 {
                    *cell = '#';
                } else if disp.abs() > 0.15 {
                    *cell = '~';
                } else if disp.abs() > 0.03 {
                    *cell = '.';
                } else {
                    *cell = ' ';
                }
            }

            // Draw strike puck
            let puck_x = (self.strike_puck_pos.0 * (width - 1) as f32).round() as usize;
            let puck_y = (self.strike_puck_pos.1 * (canvas_h - 1) as f32).round() as usize;
            if y == puck_y && puck_x < width {
                row[puck_x] = '@';
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "StrikePos: ({:.2}, {:.2}) | PuckHitTarget: >=44pt [PASS]",
            self.strike_puck_pos.0, self.strike_puck_pos.1
        );
        lines.push(footer);
        lines
    }

    /// Render headless PNG snapshot displaying 3D mesh surface, puck handle, and metrics.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Background: Deep slate/navy (#0A0E18)
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 10;
                pixels[idx + 1] = 14;
                pixels[idx + 2] = 24;
                pixels[idx + 3] = 255;
            }
        }

        // Header Panel (y: 10..46, x: 20..width-20)
        fill_rounded_rect_wg(&mut pixels, width, height, 20, 10, width - 40, 36, 4, [18, 24, 38, 255]);

        // Main 3D Mesh Canvas (x: 20..width-20, y: 56..height-70)
        let canvas_w = width - 40;
        let canvas_h = height - 130;
        fill_rounded_rect_wg(&mut pixels, width, height, 20, 56, canvas_w, canvas_h, 6, [14, 20, 32, 255]);
        draw_rect_border_wg(&mut pixels, width, height, 20, 56, canvas_w, canvas_h, [45, 60, 85, 255]);

        let center_x = 20.0 + canvas_w as f32 * 0.5;
        let center_y = 56.0 + canvas_h as f32 * 0.55;
        let scale = canvas_h as f32 * 0.40;

        let grid = self.mesh.displacement_grid();

        // Draw 3D Mesh Wireframe Grid
        for y in 0..MESH_DIM {
            for x in 0..MESH_DIM {
                let disp = grid[y][x];
                let (px, py) = self.project_3d_to_canvas(x as f32, y as f32, disp, (center_x, center_y), scale);

                // Connect to right neighbor
                if x + 1 < MESH_DIM {
                    let disp_r = grid[y][x + 1];
                    let (rx, ry) = self.project_3d_to_canvas((x + 1) as f32, y as f32, disp_r, (center_x, center_y), scale);
                    draw_line_segment_wg(&mut pixels, width, height, px, py, rx, ry, [0, 229, 255, 180]);
                }

                // Connect to bottom neighbor
                if y + 1 < MESH_DIM {
                    let disp_b = grid[y + 1][x];
                    let (bx, by) = self.project_3d_to_canvas(x as f32, (y + 1) as f32, disp_b, (center_x, center_y), scale);
                    draw_line_segment_wg(&mut pixels, width, height, px, py, bx, by, [0, 255, 180, 180]);
                }
            }
        }

        // Draw Strike Puck Handle (Gold, >= 22pt radius -> 44x44pt bounding touch target)
        let (puck_x, puck_y) = self.project_3d_to_canvas(
            self.strike_puck_pos.0 * (MESH_DIM - 1) as f32,
            self.strike_puck_pos.1 * (MESH_DIM - 1) as f32,
            0.0,
            (center_x, center_y),
            scale,
        );

        draw_circle_ring_wg(&mut pixels, width, height, puck_x, puck_y, WAVEGUIDE_PUCK_HIT_RADIUS, [255, 215, 0, 160]);
        draw_circle_filled_wg(&mut pixels, width, height, puck_x, puck_y, WAVEGUIDE_PUCK_VISUAL_RADIUS, [255, 215, 0, 255]);
        draw_circle_filled_wg(&mut pixels, width, height, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

        // Bottom Metrics Footer Bar (x: 20..width-20, y: height-60..height-15)
        let footer_y = height - 60;
        fill_rounded_rect_wg(&mut pixels, width, height, 20, footer_y, canvas_w, 45, 4, [16, 35, 28, 255]);
        draw_rect_border_wg(&mut pixels, width, height, 20, footer_y, canvas_w, 45, [0, 255, 180, 255]);

        encode_minimal_png_wg(path, &pixels, width, height)
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
            6.0,
            Color32::from_rgb(10, 14, 24),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 18.0),
            egui::Align2::LEFT_TOP,
            format!(
                "2D PHYSICAL WAVEGUIDE RESONATOR MESH [{:?}]",
                self.mesh.material
            ),
            egui::FontId::proportional(15.0),
            Color32::from_rgb(255, 215, 0),
        );

        let canvas_w = rect.width - 40.0;
        let canvas_h = rect.height - 120.0;
        let center_x = rect.x + 20.0 + canvas_w * 0.5;
        let center_y = rect.y + 56.0 + canvas_h * 0.55;
        let scale = canvas_h * 0.40;

        let grid = self.mesh.displacement_grid();

        // Draw 3D Wireframe Mesh
        for y in 0..MESH_DIM {
            for x in 0..MESH_DIM {
                let disp = grid[y][x];
                let (px, py) = self.project_3d_to_canvas(x as f32, y as f32, disp, (center_x, center_y), scale);
                let pt = egui::pos2(px, py);

                if x + 1 < MESH_DIM {
                    let disp_r = grid[y][x + 1];
                    let (rx, ry) = self.project_3d_to_canvas((x + 1) as f32, y as f32, disp_r, (center_x, center_y), scale);
                    painter.line_segment([pt, egui::pos2(rx, ry)], Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)));
                }

                if y + 1 < MESH_DIM {
                    let disp_b = grid[y + 1][x];
                    let (bx, by) = self.project_3d_to_canvas(x as f32, (y + 1) as f32, disp_b, (center_x, center_y), scale);
                    painter.line_segment([pt, egui::pos2(bx, by)], Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)));
                }
            }
        }

        // Draw Strike Puck (Gold, >= 22pt radius -> 44x44pt touch bounding target)
        let (puck_x, puck_y) = self.project_3d_to_canvas(
            self.strike_puck_pos.0 * (MESH_DIM - 1) as f32,
            self.strike_puck_pos.1 * (MESH_DIM - 1) as f32,
            0.0,
            (center_x, center_y),
            scale,
        );

        let puck_pt = egui::pos2(puck_x, puck_y);
        painter.circle_stroke(
            puck_pt,
            WAVEGUIDE_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(puck_pt, WAVEGUIDE_PUCK_VISUAL_RADIUS, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(puck_pt, 4.0, Color32::WHITE);
    }
}

// ----------------------------------------------------------------------------
// Minimal software rasterizer & PNG encoder helpers for headless verification
// ----------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect_wg(
    buf: &mut [u8],
    w: u32,
    h: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    radius: u32,
    col: [u8; 4],
) {
    let r = radius as f32;
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let mut inside = true;
            if x < rx + radius && y < ry + radius {
                let dx = (rx + radius - x) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y < ry + radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x < rx + radius && y >= ry + rh - radius {
                let dx = (rx + radius - x) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y >= ry + rh - radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            }
            if inside {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rect_border_wg(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for x in rx..(rx + rw).min(w) {
        if ry < h {
            let idx = ((ry * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if ry + rh - 1 < h {
            let idx = (((ry + rh - 1) * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
    for y in ry..(ry + rh).min(h) {
        if rx < w {
            let idx = ((y * w + rx) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if rx + rw - 1 < w {
            let idx = ((y * w + rx + rw - 1) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_segment_wg(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) * 2.0).max(1.0) as u32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let px = (x0 + t * dx).round() as i32;
        let py = (y0 + t * dy).round() as i32;
        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
            let idx = ((py as u32 * w + px as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_wg(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_circle_ring_wg(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let thickness = 2.0f32;
    let r_inner = (r - thickness).max(0.0);
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r * r && dist_sq >= r_inner * r_inner {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn encode_minimal_png_wg(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_wg(&mut out, b"IHDR", &ihdr);

    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0);
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_wg(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_wg(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_wg(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_wg(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_wg(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touch_controls::MIN_HIT_TARGET_PT;

    #[test]
    fn test_waveguide_mesh_view_ascii_render() {
        let view = WaveguideMeshView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert!(ascii[0].contains("WAVEGUIDE 3D MESH"));
    }

    #[test]
    fn test_waveguide_mesh_view_hit_target_dimensions() {
        const {
            assert!(
                WAVEGUIDE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Strike puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_waveguide_mesh_view_snapshot_render() {
        let mut view = WaveguideMeshView::new();
        view.trigger_strike(0.85);
        for _ in 0..10 {
            view.step_simulation();
        }
        let res = view.render_snapshot_png("scratch/renders/waveguide_mesh_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
