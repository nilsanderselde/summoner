// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic MPE Polyphonic Note Expression Curve Editor (Milestone 6).

use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Pos2, Response, Sense, Stroke, Ui, Vec2};

pub const MPE_NODE_VISUAL_RADIUS: f32 = 10.0;
pub const MPE_NODE_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target

/// MPE Modulation Dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MpeDimension {
    #[default]
    PitchBend,
    Pressure,
    Timbre,
    ReleaseVelocity,
}

impl MpeDimension {
    pub fn name(&self) -> &'static str {
        match self {
            MpeDimension::PitchBend => "Pitch Bend",
            MpeDimension::Pressure => "Pressure (Aftertouch)",
            MpeDimension::Timbre => "Timbre (CC74 Slide)",
            MpeDimension::ReleaseVelocity => "Release Velocity",
        }
    }

    pub fn color_rgba(&self) -> [u8; 4] {
        match self {
            MpeDimension::PitchBend => [0, 229, 255, 255],     // Electric Cyan
            MpeDimension::Pressure => [255, 214, 0, 255],     // Neon Amber
            MpeDimension::Timbre => [224, 64, 251, 255],      // Vivid Magenta
            MpeDimension::ReleaseVelocity => [100, 255, 100, 255], // Lime Green
        }
    }
}

/// A single Bézier node on an MPE per-note expression curve.
#[derive(Debug, Clone, PartialEq)]
pub struct MpeCurveNode {
    pub time_norm: f32, // 0.0 to 1.0 of note duration
    pub value: f32,     // Normalized [-1.0, 1.0] for PitchBend, [0.0, 1.0] for Pressure/Timbre
    pub handle_in_y: f32,
    pub handle_out_y: f32,
    pub is_selected: bool,
}

impl MpeCurveNode {
    pub fn new(time_norm: f32, value: f32) -> Self {
        Self {
            time_norm: time_norm.clamp(0.0, 1.0),
            value,
            handle_in_y: value,
            handle_out_y: value,
            is_selected: false,
        }
    }
}

/// A polyphonic note containing per-note MPE expression curves.
#[derive(Debug, Clone, PartialEq)]
pub struct MpeNoteBlock {
    pub note_id: u64,
    pub pitch: f32, // MIDI pitch 0.0 to 127.0
    pub start_beat: f64,
    pub duration_beats: f64,
    pub velocity: f32,
    pub pitch_bend_curve: Vec<MpeCurveNode>,
    pub pressure_curve: Vec<MpeCurveNode>,
    pub timbre_curve: Vec<MpeCurveNode>,
    pub is_selected: bool,
}

impl MpeNoteBlock {
    pub fn new(note_id: u64, pitch: f32, start_beat: f64, duration_beats: f64) -> Self {
        Self {
            note_id,
            pitch,
            start_beat,
            duration_beats: duration_beats.max(0.125),
            velocity: 0.8,
            pitch_bend_curve: vec![
                MpeCurveNode::new(0.0, 0.0),
                MpeCurveNode::new(0.5, 0.5),
                MpeCurveNode::new(1.0, 0.0),
            ],
            pressure_curve: vec![
                MpeCurveNode::new(0.0, 0.2),
                MpeCurveNode::new(0.4, 0.9),
                MpeCurveNode::new(1.0, 0.1),
            ],
            timbre_curve: vec![
                MpeCurveNode::new(0.0, 0.3),
                MpeCurveNode::new(0.7, 0.8),
                MpeCurveNode::new(1.0, 0.4),
            ],
            is_selected: false,
        }
    }

    pub fn curve_for_dimension(&self, dim: MpeDimension) -> &Vec<MpeCurveNode> {
        match dim {
            MpeDimension::PitchBend => &self.pitch_bend_curve,
            MpeDimension::Pressure => &self.pressure_curve,
            MpeDimension::Timbre => &self.timbre_curve,
            MpeDimension::ReleaseVelocity => &self.pressure_curve,
        }
    }

    pub fn curve_for_dimension_mut(&mut self, dim: MpeDimension) -> &mut Vec<MpeCurveNode> {
        match dim {
            MpeDimension::PitchBend => &mut self.pitch_bend_curve,
            MpeDimension::Pressure => &mut self.pressure_curve,
            MpeDimension::Timbre => &mut self.timbre_curve,
            MpeDimension::ReleaseVelocity => &mut self.pressure_curve,
        }
    }

    /// Evaluate cubic Bézier curve at a normalized time [0.0, 1.0].
    pub fn evaluate_curve(&self, dim: MpeDimension, t_norm: f32) -> f32 {
        let curve = self.curve_for_dimension(dim);
        if curve.is_empty() {
            return 0.0;
        }
        if curve.len() == 1 || t_norm <= curve[0].time_norm {
            return curve[0].value;
        }
        if t_norm >= curve.last().unwrap().time_norm {
            return curve.last().unwrap().value;
        }

        for window in curve.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];
            if t_norm >= p0.time_norm && t_norm <= p1.time_norm {
                let range = (p1.time_norm - p0.time_norm).max(1e-5);
                let local_t = (t_norm - p0.time_norm) / range;

                // Cubic Bézier formula
                let y0 = p0.value;
                let y1 = p0.handle_out_y;
                let y2 = p1.handle_in_y;
                let y3 = p1.value;

                let u = 1.0 - local_t;
                return u * u * u * y0 + 3.0 * u * u * local_t * y1 + 3.0 * u * local_t * local_t * y2 + local_t * local_t * local_t * y3;
            }
        }

        0.0
    }
}

/// Dynamic MPE Polyphonic Note Expression Curve Editor View.
#[derive(Debug, Clone)]
pub struct MpeEditorView {
    pub notes: Vec<MpeNoteBlock>,
    pub active_dimension: MpeDimension,
    pub selected_note_id: Option<u64>,
    pub selected_node_idx: Option<usize>,
    pub dragging_node: bool,
    pub zoom_x: f32,
    pub scroll_beat: f64,
    pub total_beats: f64,
    pub color_palette: ContrastColorPalette,
}

impl Default for MpeEditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl MpeEditorView {
    pub fn new() -> Self {
        let mut view = Self {
            notes: Vec::new(),
            active_dimension: MpeDimension::PitchBend,
            selected_note_id: None,
            selected_node_idx: None,
            dragging_node: false,
            zoom_x: 1.0,
            scroll_beat: 0.0,
            total_beats: 16.0,
            color_palette: ContrastColorPalette::default(),
        };

        // Populate sample MPE notes
        view.notes.push(MpeNoteBlock::new(1, 60.0, 0.0, 3.5)); // C4
        view.notes.push(MpeNoteBlock::new(2, 64.0, 1.0, 3.0)); // E4
        view.notes.push(MpeNoteBlock::new(3, 67.0, 2.0, 4.0)); // G4
        view.notes.push(MpeNoteBlock::new(4, 71.0, 4.0, 3.5)); // B4
        view.selected_note_id = Some(1);
        view
    }

    pub fn select_note(&mut self, note_id: u64) {
        self.selected_note_id = Some(note_id);
        for n in &mut self.notes {
            n.is_selected = n.note_id == note_id;
        }
    }

    pub fn set_dimension(&mut self, dim: MpeDimension) {
        self.active_dimension = dim;
        self.selected_node_idx = None;
    }

    /// Add an expression curve point to the selected note.
    pub fn add_curve_point(&mut self, note_id: u64, dim: MpeDimension, time_norm: f32, value: f32) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.note_id == note_id) {
            let curve = note.curve_for_dimension_mut(dim);
            curve.push(MpeCurveNode::new(time_norm, value));
            curve.sort_by(|a, b| a.time_norm.partial_cmp(&b.time_norm).unwrap());
        }
    }

    /// Render ASCII diagram representation for headless inspection and test audits.
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let mut grid = vec![vec![' '; width]; height];

        // Draw border
        for cell in grid[0].iter_mut().take(width) {
            *cell = '-';
        }
        for cell in grid[height - 1].iter_mut().take(width) {
            *cell = '-';
        }
        for row in grid.iter_mut().take(height) {
            row[0] = '|';
            row[width - 1] = '|';
        }

        // Draw title
        let title = format!(" MPE EXPRESSION: {} ", self.active_dimension.name());
        for (i, c) in title.chars().enumerate().take(width - 4) {
            grid[0][i + 2] = c;
        }

        let note_count = self.notes.len();
        if note_count > 0 {
            for (idx, note) in self.notes.iter().enumerate() {
                let note_y = 2 + (idx * 3).min(height - 4);
                let start_x = 2 + ((note.start_beat / self.total_beats) * (width - 4) as f64) as usize;
                let end_x = (start_x + ((note.duration_beats / self.total_beats) * (width - 4) as f64) as usize)
                    .min(width - 2);

                for cell in grid[note_y].iter_mut().take(end_x).skip(start_x) {
                    *cell = '=';
                }
                grid[note_y][start_x] = '[';
                if end_x > start_x {
                    grid[note_y][end_x - 1] = ']';
                }

                // Render curve points on note
                let curve = note.curve_for_dimension(self.active_dimension);
                for node in curve {
                    let px = start_x + (node.time_norm * (end_x - start_x) as f32) as usize;
                    if px < width - 1 && note_y + 1 < height - 1 {
                        grid[note_y + 1][px] = '*';
                    }
                }
            }
        }

        grid.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render a headless PNG snapshot to verify visual layout, Bézier curves, and high contrast.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let mut img_buf = vec![0u8; (width * height * 4) as usize];
        // Background: Deep slate #0A0E18
        for chunk in img_buf.chunks_exact_mut(4) {
            chunk[0] = 10;
            chunk[1] = 14;
            chunk[2] = 24;
            chunk[3] = 255;
        }

        // Draw vertical grid lines (bars)
        let num_bars = 4;
        for bar in 0..=num_bars {
            let gx = (bar as f32 / num_bars as f32 * (width as f32 - 1.0)) as u32;
            for y in 0..height {
                let idx = ((y * width + gx) * 4) as usize;
                if idx + 3 < img_buf.len() {
                    img_buf[idx] = 40;
                    img_buf[idx + 1] = 50;
                    img_buf[idx + 2] = 70;
                    img_buf[idx + 3] = 255;
                }
            }
        }

        let dim_col = self.active_dimension.color_rgba();
        let note_lane_h = 56.0f32;

        for (n_idx, note) in self.notes.iter().enumerate() {
            let note_y = 32.0 + (n_idx as f32 * (note_lane_h + 16.0));
            if note_y + note_lane_h > height as f32 {
                break;
            }

            let start_x = ((note.start_beat / self.total_beats) as f32 * width as f32).max(10.0);
            let note_w = ((note.duration_beats / self.total_beats) as f32 * width as f32).max(60.0);
            let end_x = (start_x + note_w).min(width as f32 - 10.0);

            // Draw note block background
            let bg_r = if note.is_selected { 26 } else { 20 };
            let bg_g = if note.is_selected { 35 } else { 26 };
            let bg_b = if note.is_selected { 126 } else { 45 };

            for y in note_y as u32..(note_y + note_lane_h) as u32 {
                for x in start_x as u32..end_x as u32 {
                    let idx = ((y * width + x) * 4) as usize;
                    if idx + 3 < img_buf.len() {
                        let is_border = x == start_x as u32 || x == end_x as u32 - 1 || y == note_y as u32 || y == (note_y + note_lane_h) as u32 - 1;
                        if is_border {
                            img_buf[idx] = if note.is_selected { 255 } else { 60 };
                            img_buf[idx + 1] = if note.is_selected { 255 } else { 75 };
                            img_buf[idx + 2] = if note.is_selected { 255 } else { 110 };
                        } else {
                            img_buf[idx] = bg_r;
                            img_buf[idx + 1] = bg_g;
                            img_buf[idx + 2] = bg_b;
                        }
                    }
                }
            }

            // Draw Bézier curve points
            let curve = note.curve_for_dimension(self.active_dimension);
            let steps = 64;
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let val = note.evaluate_curve(self.active_dimension, t);
                let px = start_x + t * (end_x - start_x);
                let norm_y = if self.active_dimension == MpeDimension::PitchBend {
                    (val + 1.0) * 0.5
                } else {
                    val.clamp(0.0, 1.0)
                };
                let py = (note_y + note_lane_h - 8.0) - norm_y * (note_lane_h - 16.0);

                let ux = (px as u32).min(width - 1);
                let uy = (py as u32).min(height - 1);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = (ux as i32 + dx).clamp(0, width as i32 - 1) as u32;
                        let ny = (uy as i32 + dy).clamp(0, height as i32 - 1) as u32;
                        let idx = ((ny * width + nx) * 4) as usize;
                        if idx + 3 < img_buf.len() {
                            img_buf[idx] = dim_col[0];
                            img_buf[idx + 1] = dim_col[1];
                            img_buf[idx + 2] = dim_col[2];
                            img_buf[idx + 3] = 255;
                        }
                    }
                }
            }

            // Draw drag handle circles (radius 8 visual, hit target 22pt)
            for pt in curve {
                let px = start_x + pt.time_norm * (end_x - start_x);
                let norm_y = if self.active_dimension == MpeDimension::PitchBend {
                    (pt.value + 1.0) * 0.5
                } else {
                    pt.value.clamp(0.0, 1.0)
                };
                let py = (note_y + note_lane_h - 8.0) - norm_y * (note_lane_h - 16.0);

                let cx = px as i32;
                let cy = py as i32;
                let r = MPE_NODE_VISUAL_RADIUS as i32;

                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy <= r * r {
                            let nx = (cx + dx).clamp(0, width as i32 - 1) as u32;
                            let ny = (cy + dy).clamp(0, height as i32 - 1) as u32;
                            let idx = ((ny * width + nx) * 4) as usize;
                            if idx + 3 < img_buf.len() {
                                if dx * dx + dy * dy >= (r - 2) * (r - 2) {
                                    img_buf[idx] = 0;
                                    img_buf[idx + 1] = 0;
                                    img_buf[idx + 2] = 0;
                                } else {
                                    img_buf[idx] = dim_col[0];
                                    img_buf[idx + 1] = dim_col[1];
                                    img_buf[idx + 2] = dim_col[2];
                                }
                            }
                        }
                    }
                }
            }
        }

        save_png_direct(path, width, height, &img_buf)
    }

    #[cfg(feature = "gui")]
    pub fn render_ui(&mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(600.0), 320.0),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // 1. Background (Deep Slate/Navy #0A0E18)
        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 70)));

        // 2. Timeline Grid Lines (Quarters)
        let num_bars = 4;
        for bar in 0..=num_bars {
            let x = rect.min.x + (bar as f32 / num_bars as f32) * rect.width();
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
            );
        }

        // 3. Render Notes and MPE Expression Curves
        let active_dim = self.active_dimension;
        let dim_col_arr = active_dim.color_rgba();
        let dim_color = Color32::from_rgba_unmultiplied(
            dim_col_arr[0],
            dim_col_arr[1],
            dim_col_arr[2],
            dim_col_arr[3],
        );

        let note_lane_h = 50.0;
        let mut hit_node = None;

        for (n_idx, note) in self.notes.iter_mut().enumerate() {
            let note_y = rect.min.y + 40.0 + (n_idx as f32 * (note_lane_h + 12.0));
            if note_y + note_lane_h > rect.max.y {
                break;
            }

            let start_x = rect.min.x + ((note.start_beat / 16.0) as f32 * rect.width());
            let note_w = ((note.duration_beats / 16.0) as f32 * rect.width()).max(60.0);
            let note_rect = egui::Rect::from_min_size(
                Pos2::new(start_x, note_y),
                Vec2::new(note_w, note_lane_h),
            );

            // Note Block Background
            let note_bg = if note.is_selected {
                Color32::from_rgb(26, 35, 126) // Indigo selected
            } else {
                Color32::from_rgb(20, 26, 45)
            };
            painter.rect_filled(note_rect, 6.0, note_bg);
            painter.rect_stroke(
                note_rect,
                6.0,
                Stroke::new(
                    if note.is_selected { 2.0_f32 } else { 1.0_f32 },
                    if note.is_selected { Color32::WHITE } else { Color32::from_rgb(60, 75, 110) },
                ),
            );

            // Note Label (Pitch)
            painter.text(
                Pos2::new(note_rect.min.x + 8.0, note_rect.min.y + 12.0),
                egui::Align2::LEFT_CENTER,
                format!("Note #{:.0} (Pitch {:.1})", note.note_id, note.pitch),
                egui::FontId::proportional(12.0),
                Color32::from_rgb(220, 225, 240),
            );

            // Bézier Curve Segments
            let curve = note.curve_for_dimension(active_dim);
            if curve.len() >= 2 {
                let steps = 32;
                let mut prev_pos = None;

                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let val = note.evaluate_curve(active_dim, t);
                    let px = note_rect.min.x + t * note_rect.width();
                    let norm_y = if active_dim == MpeDimension::PitchBend {
                        (val + 1.0) * 0.5 // Map [-1, 1] to [0, 1]
                    } else {
                        val.clamp(0.0, 1.0)
                    };
                    let py = note_rect.max.y - 6.0 - norm_y * (note_lane_h - 16.0);
                    let current_pos = Pos2::new(px, py);

                    if let Some(p_prev) = prev_pos {
                        painter.line_segment([p_prev, current_pos], Stroke::new(2.5_f32, dim_color));
                    }
                    prev_pos = Some(current_pos);
                }
            }

            // Curve Drag Handles & Hit Targets (>= 44x44pt bounding touch targets)
            for (pt_idx, pt) in curve.iter().enumerate() {
                let px = note_rect.min.x + pt.time_norm * note_rect.width();
                let norm_y = if active_dim == MpeDimension::PitchBend {
                    (pt.value + 1.0) * 0.5
                } else {
                    pt.value.clamp(0.0, 1.0)
                };
                let py = note_rect.max.y - 6.0 - norm_y * (note_lane_h - 16.0);
                let handle_pos = Pos2::new(px, py);

                let is_hit = response.hover_pos().is_some_and(|pos| {
                    (pos - handle_pos).length() <= MPE_NODE_HIT_RADIUS
                });

                if is_hit && response.clicked() {
                    hit_node = Some((note.note_id, pt_idx));
                }

                // Handle visual circle
                painter.circle_filled(
                    handle_pos,
                    MPE_NODE_VISUAL_RADIUS,
                    if is_hit { Color32::WHITE } else { dim_color },
                );
                painter.circle_stroke(
                    handle_pos,
                    MPE_NODE_VISUAL_RADIUS,
                    Stroke::new(2.0_f32, Color32::BLACK),
                );
            }
        }

        if let Some((nid, pt_idx)) = hit_node {
            self.select_note(nid);
            self.selected_node_idx = Some(pt_idx);
        }

        response
    }
}

fn save_png_direct(path: &str, width: u32, height: u32, rgba_pixels: &[u8]) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut out = Vec::new();
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_mpe(&mut out, b"IHDR", &ihdr);

    let row_size = (width * 4) as usize;
    let mut raw_data = Vec::with_capacity((height as usize) * (row_size + 1));
    for y in 0..height as usize {
        raw_data.push(0);
        let start = y * row_size;
        let end = start + row_size;
        if end <= rgba_pixels.len() {
            raw_data.extend_from_slice(&rgba_pixels[start..end]);
        } else {
            raw_data.resize(raw_data.len() + row_size, 0);
        }
    }

    let mut zlib_data = Vec::new();
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let block_size = 65535usize;
    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(block_size);
        let is_last = offset + chunk_len >= raw_data.len();
        zlib_data.push(if is_last { 0x01 } else { 0x00 });
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

    write_png_chunk_mpe(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_mpe(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_mpe(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_mpe(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_mpe(buf: &[u8]) -> u32 {
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
    fn test_mpe_note_block_bezier_curve_evaluation() {
        let mut note = MpeNoteBlock::new(1, 60.0, 0.0, 4.0);
        note.pitch_bend_curve = vec![
            MpeCurveNode::new(0.0, 0.0),
            MpeCurveNode::new(0.5, 1.0),
            MpeCurveNode::new(1.0, 0.0),
        ];

        let val_start = note.evaluate_curve(MpeDimension::PitchBend, 0.0);
        let val_mid = note.evaluate_curve(MpeDimension::PitchBend, 0.5);
        let val_end = note.evaluate_curve(MpeDimension::PitchBend, 1.0);

        assert_eq!(val_start, 0.0);
        assert_eq!(val_mid, 1.0);
        assert_eq!(val_end, 0.0);
    }

    #[test]
    fn test_mpe_editor_view_ascii_render_and_dimensions() {
        let mut view = MpeEditorView::new();
        view.set_dimension(MpeDimension::Pressure);

        let ascii = view.render_ascii(80, 20);
        assert!(ascii.contains("MPE EXPRESSION: Pressure (Aftertouch)"));
        assert!(ascii.contains("="));
        assert!(ascii.contains("*"));
    }

    #[test]
    fn test_mpe_editor_hit_target_dimensions() {
        const {
            assert!(MPE_NODE_HIT_RADIUS >= 22.0, "Hit target radius must be >= 22pt for 44x44pt target");
            assert!(MIN_HIT_TARGET_PT >= 44.0, "Minimum hit target must be at least 44pt");
        }
    }

    #[test]
    fn test_mpe_editor_render_snapshot_png() {
        let view = MpeEditorView::new();
        let render_path = "scratch/renders/mpe_expression_editor.png";
        let res = view.render_snapshot_png(render_path, 800, 400);
        assert!(res.is_ok(), "Snapshot render must succeed: {:?}", res);
        assert!(
            std::path::Path::new(render_path).exists(),
            "Rendered PNG must exist"
        );
    }
}
