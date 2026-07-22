// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

pub struct AsciiRenderer;

impl AsciiRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_vu_meter(&self, level: f32, width: usize) -> String {
        let level = level.clamp(0.0, 1.0);
        let filled = (level * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);

        let mut meter = String::with_capacity(width + 2);
        meter.push('[');
        for _ in 0..filled {
            meter.push('#');
        }
        for _ in 0..empty {
            meter.push('-');
        }
        meter.push(']');
        meter
    }

    pub fn render_waveform(&self, buffer: &[f32], width: usize, height: usize) -> String {
        if buffer.is_empty() || width == 0 || height == 0 {
            return String::new();
        }

        let mut out = String::new();
        let chunk_size = (buffer.len() as f32 / width as f32).ceil() as usize;

        // Simplify by creating a 1D vertical peak representation for each column
        // A full terminal waveform would be 2D. We'll do a simple multi-line approach.
        let mut columns = Vec::new();
        for chunk in buffer.chunks(chunk_size) {
            let max_val = chunk.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
            columns.push(max_val.clamp(0.0, 1.0));
        }

        let half_h = height / 2;

        for r in 0..height {
            let row_val = 1.0 - (r as f32 / (height - 1) as f32); // 1.0 to 0.0 top to bottom
            let normalized_y = (row_val - 0.5) * 2.0; // 1.0 to -1.0

            for col in &columns {
                // simple filled block if val >= |y|
                if col >= &normalized_y.abs() {
                    out.push('*');
                } else {
                    if r == half_h {
                        out.push('-');
                    } else {
                        out.push(' ');
                    }
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn render_track_header(&self, track_id: u64, name: &str, level: f32) -> String {
        format!("Track {} | {:<15} | {}", track_id, name, self.render_vu_meter(level, 20))
    }
}
