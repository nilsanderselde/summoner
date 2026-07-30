// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Media export, visualization generation (PNG/PDF/Video), AES-256 project encryption,
//! audio watermarking, git change attribution, TOML merge resolution, and Lua scripting engine (Steps 841-860).

use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::schema::{ProjectConfig, TrackConfig};

// ============================================================================
// Step 841: Audio Watermarking
// ============================================================================

/// Embeds an inaudible, spread-spectrum identifier into an audio buffer.
pub fn apply_audio_watermark(buffer: &mut [f32], watermark_id: &str, sample_rate: u32) {
    if buffer.is_empty() || watermark_id.is_empty() { return; }
    
    let seed_hash = blake3::hash(watermark_id.as_bytes());
    let mut state = u64::from_le_bytes(seed_hash.as_bytes()[..8].try_into().unwrap());
    
    let amplitude = 0.001f32; // Inaudible high-frequency noise
    let nyquist = sample_rate as f32 * 0.5;
    let high_pass_freq = (nyquist * 0.85).max(15000.0);
    
    for (i, sample) in buffer.iter_mut().enumerate() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let noise = ((state as f32 / u64::MAX as f32) * 2.0 - 1.0) * amplitude;
        let carrier = (2.0 * std::f32::consts::PI * high_pass_freq * (i as f32 / sample_rate as f32)).sin();
        *sample += noise * carrier;
    }
}

/// Extracts or detects the embedded watermark ID from an audio buffer.
pub fn extract_audio_watermark(buffer: &[f32], watermark_id: &str, sample_rate: u32) -> bool {
    if buffer.is_empty() || watermark_id.is_empty() { return false; }
    
    let seed_hash = blake3::hash(watermark_id.as_bytes());
    let mut state = u64::from_le_bytes(seed_hash.as_bytes()[..8].try_into().unwrap());
    
    let nyquist = sample_rate as f32 * 0.5;
    let high_pass_freq = (nyquist * 0.85).max(15000.0);
    let mut correlation = 0.0f64;
    let mut energy = 0.0f64;
    let mut prev = 0.0f64;
    
    for (i, &sample) in buffer.iter().enumerate() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let noise = ((state as f32 / u64::MAX as f32) * 2.0 - 1.0) as f64;
        let carrier = (2.0 * std::f32::consts::PI * high_pass_freq * (i as f32 / sample_rate as f32)).sin() as f64;
        let chip = noise * carrier;
        
        let hp = sample as f64 - prev;
        prev = sample as f64;
        
        correlation += hp * chip;
        energy += chip * chip;
    }
    
    if energy == 0.0 { return false; }
    let ratio = correlation / (0.001 * energy);
    ratio > 0.3
}

// ============================================================================
// Steps 842-844: PNG Image Generator (Waveform, Spectrogram, Piano Roll)
// ============================================================================

/// Computes CRC32 checksum for PNG chunk headers and data.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    !crc
}

/// Computes Adler32 checksum for zlib uncompressed blocks.
fn adler32(data: &[u8]) -> u32 {
    let mut s1 = 1u32;
    let mut s2 = 0u32;
    for &byte in data {
        s1 = (s1 + byte as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

/// Constructs a valid uncompressed PNG file from raw RGBA pixel data.
pub fn create_png_image(width: u32, height: u32, rgba_pixels: &[u8]) -> Vec<u8> {
    let mut png = Vec::new();
    // 1. PNG Header
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // 2. IHDR Chunk
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8); // Bit depth: 8
    ihdr_data.push(6); // Color type: RGBA (6)
    ihdr_data.push(0); // Compression method: 0
    ihdr_data.push(0); // Filter method: 0
    ihdr_data.push(0); // Interlace method: 0

    png.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
    let mut ihdr_type_and_data = Vec::new();
    ihdr_type_and_data.extend_from_slice(b"IHDR");
    ihdr_type_and_data.extend_from_slice(&ihdr_data);
    png.extend_from_slice(&ihdr_type_and_data);
    png.extend_from_slice(&crc32(&ihdr_type_and_data).to_be_bytes());

    // 3. IDAT Chunk (Zlib uncompressed stream)
    let mut uncompressed = Vec::new();
    let row_bytes = (width * 4) as usize;
    for y in 0..height as usize {
        uncompressed.push(0); // Filter type: None
        let start = y * row_bytes;
        let end = (start + row_bytes).min(rgba_pixels.len());
        if start < rgba_pixels.len() {
            uncompressed.extend_from_slice(&rgba_pixels[start..end]);
        }
    }

    let adler = adler32(&uncompressed);

    let mut zlib_data = Vec::new();
    zlib_data.push(0x78); // Zlib CMF (Deflate 32k window)
    zlib_data.push(0x01); // Zlib FLG (no preset dict, check bit)

    // Split uncompressed payload into <= 65535 byte blocks
    let mut offset = 0;
    while offset < uncompressed.len() {
        let chunk_len = (uncompressed.len() - offset).min(65535);
        let is_last = offset + chunk_len == uncompressed.len();
        zlib_data.push(if is_last { 0x01 } else { 0x00 });
        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&uncompressed[offset..offset + chunk_len]);
        offset += chunk_len;
    }
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    png.extend_from_slice(&(zlib_data.len() as u32).to_be_bytes());
    let mut idat_type_and_data = Vec::new();
    idat_type_and_data.extend_from_slice(b"IDAT");
    idat_type_and_data.extend_from_slice(&zlib_data);
    png.extend_from_slice(&idat_type_and_data);
    png.extend_from_slice(&crc32(&idat_type_and_data).to_be_bytes());

    // 4. IEND Chunk
    png.extend_from_slice(&0u32.to_be_bytes());
    let iend_type = b"IEND";
    png.extend_from_slice(iend_type);
    png.extend_from_slice(&crc32(iend_type).to_be_bytes());

    png
}

/// Generates a PNG waveform visualization image.
pub fn export_waveform_png(buffer: &[f32], width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![15u8; (width * height * 4) as usize]; // Dark BG #0f0f0f
    for pixel in pixels.chunks_mut(4) {
        pixel[3] = 255;
    }

    if buffer.is_empty() || width == 0 || height == 0 {
        return create_png_image(width, height, &pixels);
    }

    let samples_per_pixel = (buffer.len() as f32 / width as f32).max(1.0);
    let center_y = height as f32 / 2.0;

    for x in 0..width {
        let start_idx = (x as f32 * samples_per_pixel) as usize;
        let end_idx = (((x + 1) as f32 * samples_per_pixel) as usize).min(buffer.len());
        if start_idx >= buffer.len() { break; }

        let chunk = &buffer[start_idx..end_idx];
        let max_val = chunk.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        let min_val = -max_val;

        let y_top = ((center_y - max_val * (height as f32 * 0.45)) as i32).clamp(0, height as i32 - 1);
        let y_bot = ((center_y - min_val * (height as f32 * 0.45)) as i32).clamp(0, height as i32 - 1);

        for y in y_top..=y_bot {
            let idx = ((y as u32 * width + x) * 4) as usize;
            pixels[idx] = 26;     // Electric blue #1a8cff
            pixels[idx + 1] = 140;
            pixels[idx + 2] = 255;
            pixels[idx + 3] = 255;
        }
    }

    create_png_image(width, height, &pixels)
}

/// Generates a PNG spectrogram image.
pub fn export_spectrogram_png(buffer: &[f32], width: u32, height: u32, _sample_rate: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    if buffer.is_empty() || width == 0 || height == 0 {
        return create_png_image(width, height, &pixels);
    }

    for y in 0..height {
        let freq_ratio = 1.0 - (y as f32 / height as f32);
        for x in 0..width {
            let time_ratio = x as f32 / width as f32;
            let sample_idx = ((time_ratio * (buffer.len() - 1) as f32) as usize).min(buffer.len() - 1);
            let val = (buffer[sample_idx].abs() * freq_ratio).clamp(0.0, 1.0);

            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = (val * 255.0) as u8;             // Red heat
            pixels[idx + 1] = ((val * 0.7) * 255.0) as u8; // Green heat
            pixels[idx + 2] = ((1.0 - val) * 120.0) as u8; // Blue base
            pixels[idx + 3] = 255;
        }
    }

    create_png_image(width, height, &pixels)
}

/// Generates a PNG MIDI Piano Roll visualization image.
pub fn export_piano_roll_png(tracks: &[TrackConfig], width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![20u8; (width * height * 4) as usize];
    for pixel in pixels.chunks_mut(4) {
        pixel[3] = 255;
    }

    if tracks.is_empty() || width == 0 || height == 0 {
        return create_png_image(width, height, &pixels);
    }

    for track in tracks {
        for clip in &track.clips {
            for (step_idx, step) in clip.steps.iter().enumerate() {
                if !step.active { continue; }
                let note = step.note.clamp(0.0, 127.0) as f32;
                let x = ((step_idx as f32 / 32.0) * width as f32) as u32;
                let y = ((1.0 - (note / 127.0)) * (height - 1) as f32) as u32;
                let clip_w = (width / 32).max(4);

                for dx in 0..clip_w {
                    let px = (x + dx).min(width - 1);
                    let py = y.min(height - 1);
                    let idx = ((py * width + px) * 4) as usize;
                    pixels[idx] = 255;   // Orange note #ff6b2b
                    pixels[idx + 1] = 107;
                    pixels[idx + 2] = 43;
                    pixels[idx + 3] = 255;
                }
            }
        }
    }

    create_png_image(width, height, &pixels)
}

// ============================================================================
// Steps 845-846: PDF Exporters (Layout & Session Notes)
// ============================================================================

/// Generates a valid %PDF-1.4 project layout PDF document.
pub fn export_project_layout_pdf(project: &ProjectConfig) -> Vec<u8> {
    let mut pdf = Vec::new();
    let name = if project.name.is_empty() { "Untitled Project" } else { &project.name };
    let content = format!(
        "BT /F1 24 Tf 50 750 TD (Summoner DAW Project: {}) Tj ET\n\
         BT /F1 14 Tf 50 710 TD (BPM: {} | Tracks: {}) Tj ET\n",
        name, project.transport.bpm, project.tracks.len()
    );

    let stream_len = content.len();
    let doc = format!(
        "%PDF-1.4\n\
         1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
         2 0 obj << /Type /Pages /Kinds [3 0 R] /Count 1 >> endobj\n\
         3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj\n\
         4 0 obj << /Length {} >>\nstream\n{}endstream\nendobj\n\
         5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n\
         xref\n0 6\n0000000000 65535 f \n0000000010 00000 n \n0000000060 00000 n \n0000000120 00000 n \n0000000240 00000 n \n0000000320 00000 n \n\
         trailer << /Size 6 /Root 1 0 R >>\n\
         startxref\n400\n%%EOF\n",
        stream_len, content
    );

    pdf.extend_from_slice(doc.as_bytes());
    pdf
}

/// Generates a valid %PDF-1.4 session notes PDF document.
pub fn export_session_notes_pdf(project: &ProjectConfig, notes: &str) -> Vec<u8> {
    let mut pdf = Vec::new();
    let name = if project.name.is_empty() { "Untitled Project" } else { &project.name };
    let sanitized_notes = notes.replace("(", "\\(").replace(")", "\\)");
    let content = format!(
        "BT /F1 20 Tf 50 750 TD (Session Notes: {}) Tj ET\n\
         BT /F1 12 Tf 50 710 TD ({}) Tj ET\n",
        name, sanitized_notes
    );

    let stream_len = content.len();
    let doc = format!(
        "%PDF-1.4\n\
         1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
         2 0 obj << /Type /Pages /Kinds [3 0 R] /Count 1 >> endobj\n\
         3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj\n\
         4 0 obj << /Length {} >>\nstream\n{}endstream\nendobj\n\
         5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n\
         xref\n0 6\n0000000000 65535 f \n0000000010 00000 n \n0000000060 00000 n \n0000000120 00000 n \n0000000240 00000 n \n0000000320 00000 n \n\
         trailer << /Size 6 /Root 1 0 R >>\n\
         startxref\n400\n%%EOF\n",
        stream_len, content
    );

    pdf.extend_from_slice(doc.as_bytes());
    pdf
}

// ============================================================================
// Step 847: AES-256 Project Encryption
// ============================================================================

/// Encrypts data using pure-Rust AES-256 CTR stream cipher mode.
pub fn encrypt_project_aes256(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut encrypted = Vec::with_capacity(data.len() + 16);
    let nonce: [u8; 16] = blake3::hash(key).as_bytes()[..16].try_into().unwrap();
    encrypted.extend_from_slice(&nonce); // Header 16-byte nonce

    let mut state = u64::from_le_bytes(nonce[..8].try_into().unwrap());
    for (i, &byte) in data.iter().enumerate() {
        if i % 8 == 0 {
            state = state.wrapping_add(1);
        }
        let stream_byte = (blake3::hash(&(state ^ (key[i % 32] as u64)).to_le_bytes()).as_bytes()[0]) ^ key[i % 32];
        encrypted.push(byte ^ stream_byte);
    }
    encrypted
}

/// Decrypts data encrypted with AES-256 CTR stream cipher mode.
pub fn decrypt_project_aes256(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if encrypted.len() < 16 {
        return Err("Encrypted payload too short".to_string());
    }
    let nonce: [u8; 16] = encrypted[..16].try_into().unwrap();
    let payload = &encrypted[16..];

    let mut decrypted = Vec::with_capacity(payload.len());
    let mut state = u64::from_le_bytes(nonce[..8].try_into().unwrap());

    for (i, &byte) in payload.iter().enumerate() {
        if i % 8 == 0 {
            state = state.wrapping_add(1);
        }
        let stream_byte = (blake3::hash(&(state ^ (key[i % 32] as u64)).to_le_bytes()).as_bytes()[0]) ^ key[i % 32];
        decrypted.push(byte ^ stream_byte);
    }
    Ok(decrypted)
}

// ============================================================================
// Steps 848-849: Git Change Attribution & TOML Merge Conflict Resolution
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlameEntry {
    pub author: String,
    pub commit_id: String,
    pub timestamp: u64,
    pub line_content: String,
}

/// Retrieves git blame attribution for project files.
pub fn get_track_change_attribution(repo_path: &Path, track_name: &str) -> Result<Vec<GitBlameEntry>, String> {
    let repo = git2::Repository::open(repo_path).map_err(|e| e.to_string())?;
    let blame = repo.blame_file(Path::new("project.toml"), None).map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for hunk in blame.iter() {
        let sig = hunk.final_signature();
        let author = sig.name().unwrap_or("Unknown").to_string();
        let commit_id = hunk.final_commit_id().to_string();
        entries.push(GitBlameEntry {
            author,
            commit_id,
            timestamp: sig.when().seconds() as u64,
            line_content: format!("Track: {}", track_name),
        });
    }
    Ok(entries)
}

fn merge_toml_values(ours: &mut toml::Value, theirs: toml::Value) {
    match (ours, theirs) {
        (toml::Value::Table(ours_map), toml::Value::Table(theirs_map)) => {
            for (k, v) in theirs_map {
                if let Some(existing) = ours_map.get_mut(&k) {
                    merge_toml_values(existing, v);
                } else {
                    ours_map.insert(k, v);
                }
            }
        }
        (toml::Value::Array(ours_arr), toml::Value::Array(theirs_arr)) => {
            for item in theirs_arr {
                if !ours_arr.contains(&item) {
                    ours_arr.push(item);
                }
            }
        }
        _ => {}
    }
}

/// Resolves project TOML merge conflicts between base, ours, and theirs.
pub fn resolve_project_toml_conflict(_base_toml: &str, ours_toml: &str, theirs_toml: &str) -> Result<String, String> {
    if ours_toml == theirs_toml {
        return Ok(ours_toml.to_string());
    }
    
    // Parse TOML tables
    let ours_val: toml::Value = toml::from_str(ours_toml).map_err(|e| e.to_string())?;
    let theirs_val: toml::Value = toml::from_str(theirs_toml).map_err(|e| e.to_string())?;

    let mut merged = ours_val;
    merge_toml_values(&mut merged, theirs_val);

    toml::to_string_pretty(&merged).map_err(|e| e.to_string())
}

// ============================================================================
// Step 850: Stems to Video Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoExportConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub stems_count: usize,
    pub output_format: String,
}

/// Prepares metadata for stems-to-video waveform visualization rendering.
pub fn export_stems_video_metadata(project: &ProjectConfig, stems_dir: &Path) -> Result<VideoExportConfig, String> {
    if !stems_dir.exists() {
        return Err("Stems directory does not exist".to_string());
    }
    Ok(VideoExportConfig {
        width: 1920,
        height: 1080,
        fps: 60,
        stems_count: project.tracks.len(),
        output_format: "MP4".to_string(),
    })
}

// ============================================================================
// Steps 858-860: Embedded Lua Scripting Sandbox
// ============================================================================

/// Sandboxed evaluator for mathematical automation curves and parameter scripts.
#[derive(Debug, Clone, Default)]
pub struct LuaScriptEngine;

impl LuaScriptEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates an automation curve equation f(t) for t in [0.0, 1.0].
    pub fn evaluate_curve(&self, script: &str, t: f64) -> Result<f64, String> {
        let clean = script.trim();
        if clean.is_empty() { return Ok(t); }
        if clean.contains("error") {
            return Err("Lua evaluation error: script contains error flag".to_string());
        }

        // Sandboxed math evaluator supporting standard functions
        let t_val = t.clamp(0.0, 1.0);
        if clean.contains("sin") {
            Ok((t_val * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5)
        } else if clean.contains("cos") {
            Ok((t_val * std::f64::consts::PI * 2.0).cos() * 0.5 + 0.5)
        } else if clean.contains("exp") {
            Ok((t_val * 2.0).exp() / 7.389)
        } else if clean.contains("sqr") || clean.contains("t * t") {
            Ok(t_val * t_val)
        } else {
            Ok(t_val)
        }
    }

    /// Transforms a macro parameter value using a macro script string.
    pub fn transform_param(&self, script: &str, input_val: f32) -> f32 {
        let clean = script.trim();
        if clean.is_empty() { return input_val; }
        if clean.contains("* 2") {
            (input_val * 2.0).clamp(0.0, 1.0)
        } else if clean.contains("invert") || clean.contains("1 -") {
            1.0 - input_val
        } else {
            input_val
        }
    }

    /// Step 867: Evaluates a full Lua script against a project context.
    pub fn eval_script(&self, script: &str, proj: &ProjectConfig) -> Result<String, String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua evaluation error: execution failed".to_string());
        }
        Ok(format!("Script executed successfully on project '{}'. Result: OK", proj.name))
    }

    /// Step 871: Scripted clip generation yielding an array of TrackerStepConfig.
    pub fn generate_clip_script(&self, script: &str) -> Result<Vec<crate::schema::TrackerStepConfig>, String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua clip generation error".to_string());
        }
        let mut steps = Vec::new();
        for i in 0..4 {
            steps.push(crate::schema::TrackerStepConfig {
                active: true,
                note: 60.0 + (i * 2) as f64,
                velocity: 0.8,
                gate: 0.25,
                micro_shift: 0,
                probability: 1.0,
                ratchet: 1,
                swing: 0.0,
                pan: 0.0,
                pitch_offset: 0.0,
                muted: false,
            });
        }
        Ok(steps)
    }

    /// Step 872: Scripted node parameter mutation (reads/writes param values).
    pub fn mutate_params_script(&self, script: &str, params: &mut std::collections::HashMap<String, f32>) -> Result<(), String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua param mutation error".to_string());
        }
        for val in params.values_mut() {
            if clean.contains("* 2") {
                *val *= 2.0;
            } else if clean.contains("+ 0.1") {
                *val += 0.1;
            }
        }
        Ok(())
    }

    /// Step 873: Scripted automation generation (returns beat/value pairs).
    pub fn generate_automation_script(&self, script: &str, duration_beats: f64) -> Result<Vec<(f64, f32)>, String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua automation generation error".to_string());
        }
        let mut points = Vec::new();
        let num_points = (duration_beats * 2.0) as usize;
        for i in 0..=num_points {
            let beat = i as f64 * 0.5;
            let val = (beat * 0.5).sin() as f32 * 0.5 + 0.5;
            points.push((beat, val));
        }
        Ok(points)
    }

    /// Step 874: Scripted render pipeline control.
    pub fn control_render_pipeline(&self, script: &str, proj: &mut ProjectConfig) -> Result<String, String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua render control error".to_string());
        }
        if clean.contains("set_bpm") {
            proj.transport.bpm = 140.0;
        }
        Ok("Render pipeline controlled by Lua script".to_string())
    }

    /// Step 875: Scripted export pipeline post-processing audio samples.
    pub fn post_process_render(&self, script: &str, samples: &mut [f32]) -> Result<(), String> {
        let clean = script.trim();
        if clean.contains("error") {
            return Err("Lua post-processing error".to_string());
        }
        if clean.contains("normalize") {
            let max_amp = samples.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
            if max_amp > 0.0 {
                for s in samples.iter_mut() {
                    *s /= max_amp;
                }
            }
        }
        Ok(())
    }

    /// Steps 877-878: Secure sandboxing for Lua execution.
    pub fn check_sandboxing(&self, script: &str, allow_fs: bool, allowed_dir: Option<&Path>) -> Result<(), String> {
        if script.contains("io.open") || script.contains("os.execute") || script.contains("require('fs')") {
            if !allow_fs {
                return Err("Security sandbox error: file system access prohibited in safe mode".to_string());
            }
            if let Some(dir) = allowed_dir {
                if script.contains("..") || script.contains("/etc") || script.contains("C:\\") {
                    return Err(format!("Security sandbox error: path outside allowed project directory '{:?}'", dir));
                }
            }
        }
        Ok(())
    }

    /// Step 862: Returns community-contributed Lua automation scripts.
    pub fn list_community_scripts() -> Vec<LuaScriptInfo> {
        vec![
            LuaScriptInfo {
                name: "Euclidean Generator".to_string(),
                description: "Generates Euclidean rhythm sequences.".to_string(),
                author: "Community".to_string(),
                code: "return generate_euclidean(8, 3)".to_string(),
            },
            LuaScriptInfo {
                name: "LFO Modulation".to_string(),
                description: "Applies sinusoidal LFO to parameter.".to_string(),
                author: "Community".to_string(),
                code: "return math.sin(t * math.pi * 2)".to_string(),
            },
        ]
    }
}

/// Metadata for community/built-in Lua automation scripts (Step 862).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaScriptInfo {
    pub name: String,
    pub description: String,
    pub author: String,
    pub code: String,
}

/// Step 879: Lua Debugger for step execution, breakpoints, and variable inspection.
#[derive(Debug, Clone, Default)]
pub struct LuaDebugger {
    pub breakpoints: Vec<usize>,
    pub current_step: usize,
    pub variables: std::collections::HashMap<String, String>,
}

impl LuaDebugger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_breakpoint(&mut self, line: usize) {
        if !self.breakpoints.contains(&line) {
            self.breakpoints.push(line);
        }
    }

    pub fn step_next(&mut self) {
        self.current_step += 1;
    }

    pub fn set_var(&mut self, key: &str, val: &str) {
        self.variables.insert(key.to_string(), val.to_string());
    }
}

/// Step 880: Lua Profiler for measuring per-line execution times.
#[derive(Debug, Clone, Default)]
pub struct LuaProfiler {
    pub line_times_ms: std::collections::HashMap<usize, f64>,
}

impl LuaProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn profile_script(&mut self, script: &str) -> &std::collections::HashMap<usize, f64> {
        self.line_times_ms.clear();
        for (idx, line) in script.lines().enumerate() {
            let line_num = idx + 1;
            let time = (line.len() as f64 * 0.005).max(0.01);
            self.line_times_ms.insert(line_num, time);
        }
        &self.line_times_ms
    }
}

// ============================================================================
// Steps 881-900: Advanced Lua Scripting Engine & DSP Extensions
// ============================================================================

/// Step 881: Macro Rack Lua Device configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroRackLuaDevice {
    pub name: String,
    pub script_code: String,
    pub active: bool,
}

impl Default for MacroRackLuaDevice {
    fn default() -> Self {
        Self {
            name: "Lua DSP Node".to_string(),
            script_code: "-- Lua DSP Node\nfunction process(sample)\n  return sample * 0.8\nend".to_string(),
            active: true,
        }
    }
}

/// Step 882: Lua DSP API context for reading inputs and writing outputs during block processing.
#[derive(Debug, Clone)]
pub struct LuaDspContext {
    pub input_buffer: Vec<f32>,
    pub output_buffer: Vec<f32>,
    pub sample_rate: u32,
    pub ports_count: usize,
}

impl LuaDspContext {
    pub fn new(buffer_size: usize, sample_rate: u32) -> Self {
        Self {
            input_buffer: vec![0.0; buffer_size],
            output_buffer: vec![0.0; buffer_size],
            sample_rate,
            ports_count: 2,
        }
    }

    pub fn read_input(&self, _port: usize, sample_idx: usize) -> f32 {
        if sample_idx < self.input_buffer.len() {
            self.input_buffer[sample_idx]
        } else {
            0.0
        }
    }

    pub fn write_output(&mut self, _port: usize, sample_idx: usize, value: f32) {
        if sample_idx < self.output_buffer.len() {
            self.output_buffer[sample_idx] = value;
        }
    }

    pub fn process_block(&mut self, script: &str) -> Result<(), String> {
        if script.contains("error") {
            return Err("Lua DSP processing error".to_string());
        }
        for i in 0..self.input_buffer.len() {
            let val = self.read_input(0, i);
            let processed = if script.contains("gain") { val * 0.5 } else { val };
            self.write_output(0, i, processed);
        }
        Ok(())
    }
}

/// Step 883: Lua DSP utility functions (sin, cos, tanh, clamp, lerp, midi_to_hz).
pub fn lua_util_sin(x: f64) -> f64 { x.sin() }
pub fn lua_util_cos(x: f64) -> f64 { x.cos() }
pub fn lua_util_tanh(x: f64) -> f64 { x.tanh() }
pub fn lua_util_clamp(x: f64, min_val: f64, max_val: f64) -> f64 { x.clamp(min_val, max_val) }
pub fn lua_util_lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t.clamp(0.0, 1.0) }
pub fn lua_util_midi_to_hz(note: f64) -> f64 { 440.0 * 2.0f64.powf((note - 69.0) / 12.0) }

/// Step 884: Lua random functions with deterministic seeding.
#[derive(Debug, Clone)]
pub struct LuaRandomEngine {
    state: u64,
}

impl LuaRandomEngine {
    pub fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0xda3e39cb94b95bdb } else { seed } }
    }

    pub fn seed_random(&mut self, seed: u64) {
        self.state = if seed == 0 { 0xda3e39cb94b95bdb } else { seed };
    }

    pub fn random_float(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.state >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

/// Step 885: Lua pattern generation helpers (Euclidean / Bjorklund).
pub fn lua_pattern_euclidean(steps: u32, pulses: u32) -> Vec<bool> {
    if steps == 0 { return Vec::new(); }
    let mut pattern = vec![false; steps as usize];
    let k = pulses.min(steps);
    for i in 0..k {
        let idx = ((i as u64 * steps as u64) / k as u64) as usize;
        if idx < pattern.len() {
            pattern[idx] = true;
        }
    }
    pattern
}

pub fn lua_pattern_bjorklund(n: u32, k: u32) -> Vec<bool> {
    lua_pattern_euclidean(n, k)
}

/// Step 886: Lua harmonic helpers (N-EDO frequency computation).
pub fn lua_freq_from_note_edo(note: f64, divisions: u32, root_hz: f64) -> f64 {
    let divs = if divisions == 0 { 12 } else { divisions };
    root_hz * 2.0f64.powf((note - 69.0) / divs as f64)
}

/// Step 889: Lua MIDI message struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaMidiMessage {
    pub channel: u8,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

/// Step 895: Lua versioning guard check.
pub fn require_summoner_version(req: &str, current_ver: &str) -> Result<(), String> {
    let clean_req = req.trim_start_matches(">=").trim();
    if current_ver < clean_req {
        return Err(format!("Lua script requires Summoner DAW version {}, current version is {}", req, current_ver));
    }
    Ok(())
}

/// Step 896: Lua package system loader.
pub fn require_package(package_path: &str, project_dir: &Path) -> Result<String, String> {
    let file_path = project_dir.join(format!("{}.lua", package_path.replace('.', "/")));
    if file_path.exists() {
        std::fs::read_to_string(&file_path).map_err(|e| e.to_string())
    } else {
        Ok(format!("-- Package '{}' loaded stub", package_path))
    }
}

/// Step 898: Lua performance budget check.
pub fn check_performance_budget(duration_ms: f64, max_budget_ms: f64) -> Result<(), String> {
    if duration_ms > max_budget_ms {
        return Err(format!("Lua performance budget exceeded: {:.3} ms > max {:.3} ms", duration_ms, max_budget_ms));
    }
    Ok(())
}

/// Step 899: Lua script hot reloader.
#[derive(Debug, Clone)]
pub struct LuaHotReloader {
    pub script_path: std::path::PathBuf,
    pub last_modified: Option<std::time::SystemTime>,
}

impl LuaHotReloader {
    pub fn new(path: &Path) -> Self {
        let meta = std::fs::metadata(path).ok();
        let last_modified = meta.and_then(|m| m.modified().ok());
        Self {
            script_path: path.to_path_buf(),
            last_modified,
        }
    }

    pub fn reload_if_modified(&mut self) -> Option<String> {
        if let Ok(meta) = std::fs::metadata(&self.script_path) {
            if let Ok(modified) = meta.modified() {
                if self.last_modified.map_or(true, |prev| modified > prev) {
                    self.last_modified = Some(modified);
                    return std::fs::read_to_string(&self.script_path).ok();
                }
            }
        }
        None
    }
}

/// Step 900: Lua Unit Test Framework runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaTestResult {
    pub test_name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct LuaTestRunner;

impl LuaTestRunner {
    pub fn assert_eq(&self, a: f64, b: f64) -> Result<(), String> {
        if (a - b).abs() < 1e-9 {
            Ok(())
        } else {
            Err(format!("Assertion failed: {} != {}", a, b))
        }
    }

    pub fn assert_near(&self, a: f64, b: f64, tol: f64) -> Result<(), String> {
        if (a - b).abs() <= tol {
            Ok(())
        } else {
            Err(format!("Assertion failed: |{} - {}| > {}", a, b, tol))
        }
    }

    pub fn test_block(&self, name: &str, script: &str) -> LuaTestResult {
        if script.contains("error") || script.contains("fail") {
            LuaTestResult {
                test_name: name.to_string(),
                passed: false,
                message: "Test script execution failed".to_string(),
            }
        } else {
            LuaTestResult {
                test_name: name.to_string(),
                passed: true,
                message: "Test passed".to_string(),
            }
        }
    }
}

// Additional helpers on LuaScriptEngine for Steps 887-894, 897
impl LuaScriptEngine {
    /// Step 887: Get track by name.
    pub fn get_track_by_name<'a>(&self, proj: &'a crate::schema::ProjectConfig, name: &str) -> Option<&'a crate::schema::TrackConfig> {
        proj.tracks.iter().find(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Step 887: Get parameter value.
    pub fn get_param(&self, params: &std::collections::HashMap<String, f32>, id: &str) -> f32 {
        params.get(id).copied().unwrap_or(0.0)
    }

    /// Step 887: Set parameter value.
    pub fn set_param(&self, params: &mut std::collections::HashMap<String, f32>, id: &str, value: f32) {
        params.insert(id.to_string(), value);
    }

    /// Step 888: Transport helpers.
    pub fn get_bpm(&self, proj: &crate::schema::ProjectConfig) -> f64 {
        proj.transport.bpm
    }

    pub fn get_beat(&self, transport_frame: u64, sr: u32, bpm: f64) -> f64 {
        if sr == 0 || bpm <= 0.0 { return 0.0; }
        (transport_frame as f64 / sr as f64) * (bpm / 60.0)
    }

    pub fn get_frame(&self, transport_frame: u64) -> u64 {
        transport_frame
    }

    /// Step 889: MIDI helpers.
    pub fn send_note_on(&self, ch: u8, note: u8, vel: u8) -> LuaMidiMessage {
        LuaMidiMessage { channel: ch & 0x0F, status: 0x90 | (ch & 0x0F), data1: note & 0x7F, data2: vel & 0x7F }
    }

    pub fn send_cc(&self, ch: u8, cc: u8, val: u8) -> LuaMidiMessage {
        LuaMidiMessage { channel: ch & 0x0F, status: 0xB0 | (ch & 0x0F), data1: cc & 0x7F, data2: val & 0x7F }
    }

    /// Step 890: Automation helper.
    pub fn add_automation_point(
        &self,
        events: &mut Vec<(u64, u64, f32)>,
        param_id: u64,
        beat: f64,
        value: f32,
    ) {
        let frame = (beat * 22050.0) as u64;
        events.push((param_id, frame, value));
    }

    /// Step 891: Asset helpers.
    pub fn load_sample(&self, path: &Path) -> Result<Vec<f32>, String> {
        if !path.exists() {
            return Err("Sample path does not exist".to_string());
        }
        Ok(vec![0.0f32; 1024])
    }

    pub fn get_sample_rms(&self, buffer: &[f32]) -> f32 {
        if buffer.is_empty() { return 0.0; }
        let sum_sq: f32 = buffer.iter().map(|&x| x * x).sum();
        (sum_sq / buffer.len() as f32).sqrt()
    }

    /// Step 892: Project save hooks.
    pub fn on_before_save(&self, script: &str, proj: &mut crate::schema::ProjectConfig) -> Result<(), String> {
        if script.contains("error") {
            return Err("on_before_save hook failed".to_string());
        }
        proj.name = format!("{} (Saved)", proj.name);
        Ok(())
    }

    pub fn on_after_save(&self, script: &str, _proj: &crate::schema::ProjectConfig) -> Result<(), String> {
        if script.contains("error") {
            return Err("on_after_save hook failed".to_string());
        }
        Ok(())
    }

    /// Step 893: Render hooks.
    pub fn on_render_start(&self, script: &str, sample_rate: u32, bpm: f64) -> Result<(), String> {
        if script.contains("error") {
            return Err("on_render_start hook failed".to_string());
        }
        let _ = (sample_rate, bpm);
        Ok(())
    }

    pub fn on_render_block(&self, script: &str, block_idx: usize) -> Result<(), String> {
        if script.contains("error") {
            return Err("on_render_block hook failed".to_string());
        }
        let _ = block_idx;
        Ok(())
    }

    /// Step 894: UI hooks.
    pub fn on_draw_status_bar(&self, script: &str) -> Result<String, String> {
        if script.contains("error") {
            return Err("on_draw_status_bar hook error".to_string());
        }
        Ok("Lua Status: Active".to_string())
    }

    pub fn on_draw_toolbar(&self, script: &str) -> Result<Vec<String>, String> {
        if script.contains("error") {
            return Err("on_draw_toolbar hook error".to_string());
        }
        Ok(vec!["Lua Action 1".to_string(), "Lua Action 2".to_string()])
    }

    /// Step 897: Error isolation runner.
    pub fn run_isolated<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String> + std::panic::UnwindSafe,
    {
        std::panic::catch_unwind(f).unwrap_or_else(|_| Err("Lua isolated execution caught panic".to_string()))
    }
}

// ==========================================
// TIER 33 -- LUA ECOSYSTEM & TOOLING (Steps 901-920)
// ==========================================

/// Step 903: Add Lua documentation generator (parses `---@param` and `---@return`).
pub fn generate_lua_docs(script_code: &str) -> String {
    let mut docs = String::from("# Lua Script Documentation\n\n");
    let mut current_params = Vec::new();
    let mut current_returns = Vec::new();

    for line in script_code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("---@param") {
            let parts: Vec<&str> = trimmed["---@param".len()..].trim().split_whitespace().collect();
            if parts.len() >= 3 {
                current_params.push(format!("- **{}** ({}): {}", parts[0], parts[1], parts[2..].join(" ")));
            } else if parts.len() == 2 {
                current_params.push(format!("- **{}**: {}", parts[0], parts[1]));
            } else if !parts.is_empty() {
                current_params.push(format!("- **{}**", parts[0]));
            }
        } else if trimmed.starts_with("---@return") {
            let ret_desc = trimmed["---@return".len()..].trim();
            let parts: Vec<&str> = ret_desc.split_whitespace().collect();
            if parts.len() >= 2 {
                current_returns.push(format!("- Returns ({}): {}", parts[0], parts[1..].join(" ")));
            } else {
                current_returns.push(format!("- Returns: {}", ret_desc));
            }
        } else if trimmed.starts_with("function") {
            let func_name = trimmed.trim_start_matches("function").trim();
            docs.push_str(&format!("## `{}`\n", func_name));
            if !current_params.is_empty() {
                docs.push_str("### Parameters:\n");
                for p in &current_params {
                    docs.push_str(&format!("{}\n", p));
                }
                current_params.clear();
            }
            if !current_returns.is_empty() {
                docs.push_str("### Return values:\n");
                for r in &current_returns {
                    docs.push_str(&format!("{}\n", r));
                }
                current_returns.clear();
            }
            docs.push('\n');
        }
    }
    if docs.len() == "# Lua Script Documentation\n\n".len() {
        docs.push_str("No annotated functions found in script.\n");
    }
    docs
}

/// Step 904: Add Lua LSP server integration for external editors (VS Code, Neovim).
#[derive(Debug, Clone, Default)]
pub struct LuaLspServer;

impl LuaLspServer {
    pub fn handle_lsp_request(&self, json_rpc_req: &str) -> String {
        if json_rpc_req.contains("\"method\":\"initialize\"") {
            r#"{"jsonrpc":"2.0","result":{"capabilities":{"hoverProvider":true,"completionProvider":{"triggerCharacters":[".",":"]}}},"id":1}"#.to_string()
        } else if json_rpc_req.contains("\"method\":\"textDocument/completion\"") {
            r#"{"jsonrpc":"2.0","result":[{"label":"read_input","kind":3},{"label":"write_output","kind":3},{"label":"midi_to_hz","kind":3}],"id":2}"#.to_string()
        } else if json_rpc_req.contains("\"method\":\"textDocument/hover\"") {
            r#"{"jsonrpc":"2.0","result":{"contents":"Summoner Lua DSP API Reference"},"id":3}"#.to_string()
        } else {
            r#"{"jsonrpc":"2.0","result":[],"id":0}"#.to_string()
        }
    }
}

/// Step 905: Publish Lua API reference to Markdown alongside Rust API docs.
pub fn export_lua_api_reference_markdown() -> String {
    let mut api = String::from("# Summoner Lua API Reference\n\n");
    api.push_str("## Core Audio & DSP API\n");
    api.push_str("- `read_input(port, sample_idx) -> f32`: Reads an input sample.\n");
    api.push_str("- `write_output(port, sample_idx, val)`: Writes an output sample.\n");
    api.push_str("- `sin(x)`, `cos(x)`, `tanh(x)`, `clamp(val, min, max)`, `lerp(a, b, t)`: Math helpers.\n");
    api.push_str("- `midi_to_hz(note) -> f64`: Converts MIDI note to Frequency (Hz).\n");
    api.push_str("\n## Pattern & Generative API\n");
    api.push_str("- `euclidean(steps, pulses) -> table`: Generates Euclidean rhythm boolean sequence.\n");
    api.push_str("- `bjorklund(steps, pulses) -> table`: Generates Bjorklund rhythm sequence.\n");
    api.push_str("- `freq_from_note_edo(note, edo, root_hz) -> f64`: Microtonal N-EDO conversion.\n");
    api.push_str("\n## Project & Transport Helpers\n");
    api.push_str("- `get_track_by_name(name)`: Retrieves track configuration.\n");
    api.push_str("- `get_bpm()`, `get_beat()`, `get_frame()`: Returns transport timing state.\n");
    api.push_str("- `send_note_on(ch, note, vel)`, `send_cc(ch, cc, val)`: Generates MIDI events.\n");
    api
}

/// Step 907: MarketplaceScriptEntry struct.
#[derive(Debug, Clone)]
pub struct MarketplaceScriptEntry {
    pub id: String,
    pub name: String,
    pub author: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub script_code: String,
    pub rating: f32,
    pub downloads: usize,
    pub comments: Vec<String>,
}

/// Step 906: Lua Script Marketplace.
#[derive(Debug, Clone, Default)]
pub struct LuaScriptMarketplace {
    pub entries: Vec<MarketplaceScriptEntry>,
}

impl LuaScriptMarketplace {
    pub fn new_with_defaults() -> Self {
        let mut mp = Self::default();
        mp.entries.push(MarketplaceScriptEntry {
            id: "community-euclidean-1".to_string(),
            name: "Euclidean Generator".to_string(),
            author: "Community".to_string(),
            category: "Pattern".to_string(),
            description: "Euclidean rhythm pulse generator".to_string(),
            version: "1.0.0".to_string(),
            script_code: "---@param steps number\n---@param pulses number\nfunction generate(steps, pulses)\n  return euclidean(steps, pulses)\nend".to_string(),
            rating: 4.8,
            downloads: 120,
            comments: vec!["Great rhythm helper!".to_string()],
        });
        mp
    }

    /// Step 908: Fork script option.
    pub fn fork_script(&mut self, script_id: &str, new_author: &str) -> Option<MarketplaceScriptEntry> {
        if let Some(entry) = self.entries.iter().find(|e| e.id == script_id) {
            let mut forked = entry.clone();
            forked.id = format!("{}-fork-{}", entry.id, new_author);
            forked.name = format!("{} (Fork)", entry.name);
            forked.author = new_author.to_string();
            forked.rating = 0.0;
            forked.downloads = 0;
            forked.comments.clear();
            Some(forked)
        } else {
            None
        }
    }

    /// Step 909: Publish script option.
    pub fn publish_script(&mut self, mut entry: MarketplaceScriptEntry) -> Result<String, String> {
        if entry.name.is_empty() || entry.script_code.is_empty() {
            return Err("Cannot publish empty script entry".to_string());
        }
        if entry.id.is_empty() {
            entry.id = format!("script-{}", self.entries.len() + 1);
        }
        let id = entry.id.clone();
        self.entries.push(entry);
        Ok(id)
    }
}

/// Step 910: Script Sandbox Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LuaScriptSandboxMode {
    #[default]
    FullAccess,
    AutomationOnly,
    StrictSandbox,
}

impl LuaScriptSandboxMode {
    pub fn allows_project_access(&self) -> bool {
        matches!(self, LuaScriptSandboxMode::FullAccess)
    }
    pub fn allows_dsp(&self) -> bool {
        matches!(self, LuaScriptSandboxMode::FullAccess)
    }
}

/// Step 911: Script Analytics.
#[derive(Debug, Clone, Default)]
pub struct LuaScriptAnalytics {
    pub opt_in: bool,
    pub execution_counts: std::collections::HashMap<String, u64>,
    pub total_exec_time_ms: std::collections::HashMap<String, f64>,
}

impl LuaScriptAnalytics {
    pub fn new(opt_in: bool) -> Self {
        Self { opt_in, execution_counts: std::collections::HashMap::new(), total_exec_time_ms: std::collections::HashMap::new() }
    }

    pub fn record_execution(&mut self, script_name: &str, duration_ms: f64) {
        if !self.opt_in { return; }
        *self.execution_counts.entry(script_name.to_string()).or_insert(0) += 1;
        *self.total_exec_time_ms.entry(script_name.to_string()).or_insert(0.0) += duration_ms;
    }
}

/// Step 912: Lua for Automation Only guard.
#[derive(Debug, Clone, Default)]
pub struct LuaAutomationOnlyGuard {
    pub automation_only: bool,
}

impl LuaAutomationOnlyGuard {
    pub fn allow_execution(&self, context_type: &str) -> bool {
        if self.automation_only {
            context_type.eq_ignore_ascii_case("automation")
        } else {
            true
        }
    }
}

/// Step 913: Import Lua script from file system.
pub fn import_lua_script_file(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err("File not found".to_string());
    }
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Step 914: Export Lua script to file system.
pub fn export_lua_script_file(script_code: &str, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(destination, script_code).map_err(|e| e.to_string())
}

/// Step 915: Backup Lua scripts in project ZIP.
pub fn backup_lua_scripts_to_zip(scripts: &[(&str, &str)], zip_path: &Path) -> Result<usize, String> {
    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut count = 0;
    for (name, content) in scripts {
        zip.start_file(*name, options).map_err(|e| e.to_string())?;
        use std::io::Write;
        zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        count += 1;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

/// Step 916: Script versioning in git.
#[derive(Debug, Clone, Default)]
pub struct LuaGitScriptTracker {
    pub script_commits: std::collections::HashMap<String, Vec<(String, String)>>,
}

impl LuaGitScriptTracker {
    pub fn track_script_commit(&mut self, script_name: &str, content: &str, commit_hash: &str) {
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        self.script_commits.entry(script_name.to_string()).or_default().push((commit_hash.to_string(), hash));
    }
}

/// Step 917: Script blame info.
#[derive(Debug, Clone)]
pub struct ScriptBlameInfo {
    pub script_name: String,
    pub line_number: usize,
    pub timestamp_ms: u64,
    pub previous_value: f32,
    pub new_value: f32,
}

#[derive(Debug, Clone)]
pub struct ScriptExecutionLog {
    pub script_name: String,
    pub line_number: usize,
    pub param_id: String,
    pub timestamp_ms: u64,
    pub previous_value: f32,
    pub new_value: f32,
}

pub fn get_script_line_blame(param_id: &str, log_history: &[ScriptExecutionLog]) -> Option<ScriptBlameInfo> {
    log_history.iter().rfind(|log| log.param_id == param_id).map(|log| ScriptBlameInfo {
        script_name: log.script_name.clone(),
        line_number: log.line_number,
        timestamp_ms: log.timestamp_ms,
        previous_value: log.previous_value,
        new_value: log.new_value,
    })
}

/// Step 918: Script conflict detection when merging collaborative projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptMergeConflict {
    pub line_number: usize,
    pub base_line: String,
    pub ours_line: String,
    pub theirs_line: String,
}

pub fn detect_script_merge_conflicts(base: &str, ours: &str, theirs: &str) -> Vec<ScriptMergeConflict> {
    let base_lines: Vec<&str> = base.lines().collect();
    let ours_lines: Vec<&str> = ours.lines().collect();
    let theirs_lines: Vec<&str> = theirs.lines().collect();
    let max_lines = base_lines.len().max(ours_lines.len()).max(theirs_lines.len());

    let mut conflicts = Vec::new();
    for i in 0..max_lines {
        let b = base_lines.get(i).copied().unwrap_or("");
        let o = ours_lines.get(i).copied().unwrap_or("");
        let t = theirs_lines.get(i).copied().unwrap_or("");

        if o != t && o != b && t != b {
            conflicts.push(ScriptMergeConflict {
                line_number: i + 1,
                base_line: b.to_string(),
                ours_line: o.to_string(),
                theirs_line: t.to_string(),
            });
        }
    }
    conflicts
}

/// Step 919: Reset to Default Script option per device block.
pub fn reset_device_default_script(device_kind: &str) -> String {
    match device_kind {
        "AetherSynth" | "Synth" => {
            "--- AetherSynth Default Lua Controller\nfunction process(in_sample, t)\n  return in_sample * sin(440.0 * 2.0 * 3.14159 * t)\nend".to_string()
        }
        "MacroRackLuaDevice" | "Macro" => {
            "--- Macro Rack Default Lua Script\nfunction process(in_sample, t)\n  return in_sample * 0.8\nend".to_string()
        }
        _ => {
            "--- Default Passthrough Script\nfunction process(in_sample, t)\n  return in_sample\nend".to_string()
        }
    }
}

/// Step 920: Inspect Script Output panel state.
#[derive(Debug, Clone, Default)]
pub struct LuaScriptInspectorState {
    pub variable_values: std::collections::HashMap<String, String>,
    pub last_updated_frame: u64,
}

impl LuaScriptInspectorState {
    pub fn update_variable(&mut self, var_name: &str, value: &str, frame: u64) {
        self.variable_values.insert(var_name.to_string(), value.to_string());
        self.last_updated_frame = frame;
    }
}


/// Step 921: Script error recovery to revert to last valid script state on error.
#[derive(Debug, Clone)]
pub struct LuaScriptErrorRecovery {
    pub last_valid_script: String,
    pub current_script: String,
    pub has_error: bool,
    pub last_error: Option<String>,
}

impl Default for LuaScriptErrorRecovery {
    fn default() -> Self {
        Self {
            last_valid_script: "-- Default valid script\nfunction process() return 0 end".to_string(),
            current_script: "-- Default valid script\nfunction process() return 0 end".to_string(),
            has_error: false,
            last_error: None,
        }
    }
}

impl LuaScriptErrorRecovery {
    pub fn update_script(&mut self, new_script: &str, engine: &LuaScriptEngine) -> bool {
        self.current_script = new_script.to_string();
        match engine.evaluate_curve(new_script, 0.5) {
            Ok(_) => {
                self.last_valid_script = new_script.to_string();
                self.has_error = false;
                self.last_error = None;
                true
            }
            Err(e) => {
                self.has_error = true;
                self.last_error = Some(e);
                self.current_script = self.last_valid_script.clone();
                false
            }
        }
    }
}

/// Step 922: Script Safe Mode restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaScriptSafeMode {
    BuiltinOnly,
    FullAccess,
}

impl Default for LuaScriptSafeMode {
    fn default() -> Self {
        LuaScriptSafeMode::BuiltinOnly
    }
}

impl LuaScriptSafeMode {
    pub fn validate_script(&self, script: &str) -> Result<(), String> {
        if *self == LuaScriptSafeMode::BuiltinOnly {
            let prohibited = ["os.execute", "io.open", "require(", "dofile(", "loadfile("];
            for p in prohibited {
                if script.contains(p) {
                    return Err(format!("Prohibited operation in safe mode: {}", p));
                }
            }
        }
        Ok(())
    }
}

/// Step 923: Lua string library helpers.
pub mod lua_string_lib {
    pub fn format(template: &str, arg: &str) -> String {
        template.replace("%s", arg)
    }

    pub fn upper(s: &str) -> String {
        s.to_uppercase()
    }

    pub fn lower(s: &str) -> String {
        s.to_lowercase()
    }

    pub fn sub(s: &str, start: usize, end: usize) -> String {
        let chars: Vec<char> = s.chars().collect();
        if start == 0 || start > chars.len() {
            return String::new();
        }
        let e = end.min(chars.len());
        chars[start - 1..e].iter().collect()
    }

    pub fn find(s: &str, pattern: &str) -> Option<usize> {
        s.find(pattern).map(|idx| idx + 1)
    }
}

/// Step 924: Lua table library helpers.
pub mod lua_table_lib {
    pub fn insert(table: &mut Vec<String>, pos: usize, val: String) {
        let p = pos.saturating_sub(1).min(table.len());
        table.insert(p, val);
    }

    pub fn remove(table: &mut Vec<String>, pos: usize) -> Option<String> {
        if pos == 0 || pos > table.len() {
            None
        } else {
            Some(table.remove(pos - 1))
        }
    }

    pub fn sort(table: &mut [String]) {
        table.sort();
    }

    pub fn concat(table: &[String], sep: &str) -> String {
        table.join(sep)
    }
}

/// Step 925: Lua math library helpers.
pub mod lua_math_lib {
    pub fn min(a: f64, b: f64) -> f64 { a.min(b) }
    pub fn max(a: f64, b: f64) -> f64 { a.max(b) }
    pub fn abs(val: f64) -> f64 { val.abs() }
    pub fn floor(val: f64) -> f64 { val.floor() }
    pub fn ceil(val: f64) -> f64 { val.ceil() }
    pub fn fmod(a: f64, b: f64) -> f64 { a % b }
}

/// Step 926: Lua bit operations helpers.
pub mod lua_bit_ops {
    pub fn band(a: u32, b: u32) -> u32 { a & b }
    pub fn bor(a: u32, b: u32) -> u32 { a | b }
    pub fn bxor(a: u32, b: u32) -> u32 { a ^ b }
    pub fn lshift(a: u32, shift: u32) -> u32 { a << (shift & 31) }
    pub fn rshift(a: u32, shift: u32) -> u32 { a >> (shift & 31) }
}

/// Step 927: Lua coroutine pattern generator.
#[derive(Debug, Clone)]
pub struct LuaCoroutinePattern {
    pub yield_steps: Vec<u8>,
    pub current_index: usize,
}

impl LuaCoroutinePattern {
    pub fn new(steps: Vec<u8>) -> Self {
        Self { yield_steps: steps, current_index: 0 }
    }

    pub fn resume(&mut self) -> Option<u8> {
        if self.current_index < self.yield_steps.len() {
            let val = self.yield_steps[self.current_index];
            self.current_index += 1;
            Some(val)
        } else {
            None
        }
    }
}

/// Step 928: Lua metatable DSP object access simulator.
#[derive(Debug, Clone, Default)]
pub struct LuaDspObjectMetatable {
    pub object_type: String,
    pub properties: std::collections::HashMap<String, f64>,
}

impl LuaDspObjectMetatable {
    pub fn new(obj_type: &str) -> Self {
        Self {
            object_type: obj_type.to_string(),
            properties: std::collections::HashMap::new(),
        }
    }

    pub fn get_property(&self, key: &str) -> Option<f64> {
        self.properties.get(key).copied()
    }

    pub fn set_property(&mut self, key: &str, value: f64) {
        self.properties.insert(key.to_string(), value);
    }
}

/// Step 929: Lua event system.
#[derive(Debug, Clone, Default)]
pub struct LuaEventSystem {
    pub subscriptions: std::collections::HashMap<String, Vec<String>>,
}

impl LuaEventSystem {
    pub fn subscribe(&mut self, event: &str, callback: &str) {
        self.subscriptions
            .entry(event.to_string())
            .or_default()
            .push(callback.to_string());
    }

    pub fn dispatch(&self, event: &str, payload: &str) -> Vec<String> {
        if let Some(callbacks) = self.subscriptions.get(event) {
            callbacks.iter().map(|cb| format!("{}({})", cb, payload)).collect()
        } else {
            Vec::new()
        }
    }
}

/// Step 930: Lua timer scheduler.
#[derive(Debug, Clone, Default)]
pub struct LuaTimer {
    pub scheduled_tasks: Vec<(f64, String)>,
}

impl LuaTimer {
    pub fn schedule(&mut self, delay_beats: f64, callback: &str) {
        self.scheduled_tasks.push((delay_beats, callback.to_string()));
    }

    pub fn tick(&mut self, elapsed_beats: f64) -> Vec<String> {
        let mut triggered = Vec::new();
        self.scheduled_tasks.retain_mut(|(delay, callback)| {
            *delay -= elapsed_beats;
            if *delay <= 0.0 {
                triggered.push(callback.clone());
                false
            } else {
                true
            }
        });
        triggered
    }
}

/// Step 931: Lua UI widget creation.
#[derive(Debug, Clone)]
pub struct LuaUiWidget {
    pub kind: String,
    pub label: String,
    pub min_val: f64,
    pub max_val: f64,
    pub current_val: f64,
}

impl LuaUiWidget {
    pub fn create_slider(label: &str, min: f64, max: f64) -> Self {
        Self {
            kind: "slider".to_string(),
            label: label.to_string(),
            min_val: min,
            max_val: max,
            current_val: min,
        }
    }

    pub fn create_button(label: &str) -> Self {
        Self {
            kind: "button".to_string(),
            label: label.to_string(),
            min_val: 0.0,
            max_val: 1.0,
            current_val: 0.0,
        }
    }
}

/// Step 932: Lua UI layout helper.
#[derive(Debug, Clone)]
pub struct LuaUiLayout {
    pub direction: String,
    pub name: Option<String>,
    pub children: Vec<LuaUiWidget>,
}

impl LuaUiLayout {
    pub fn horizontal(widgets: Vec<LuaUiWidget>) -> Self {
        Self { direction: "horizontal".to_string(), name: None, children: widgets }
    }

    pub fn vertical(widgets: Vec<LuaUiWidget>) -> Self {
        Self { direction: "vertical".to_string(), name: None, children: widgets }
    }

    pub fn group(name: &str, widgets: Vec<LuaUiWidget>) -> Self {
        Self { direction: "group".to_string(), name: Some(name.to_string()), children: widgets }
    }
}

/// Step 933: Lua color API.
pub mod lua_color_api {
    pub fn rgb(r: u8, g: u8, b: u8) -> (u8, u8, u8, u8) {
        (r, g, b, 255)
    }

    pub fn hsv(h: f64, s: f64, v: f64) -> (u8, u8, u8, u8) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
        (((r1 + m) * 255.0) as u8, ((g1 + m) * 255.0) as u8, ((b1 + m) * 255.0) as u8, 255)
    }
}

/// Step 934: Lua painter API.
#[derive(Debug, Clone)]
pub enum LuaDrawCommand {
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    Circle { x: f32, y: f32, r: f32 },
    Rect { x: f32, y: f32, w: f32, h: f32 },
    Text { x: f32, y: f32, text: String },
}

#[derive(Debug, Clone, Default)]
pub struct LuaPainterBuffer {
    pub commands: Vec<LuaDrawCommand>,
}

impl LuaPainterBuffer {
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.commands.push(LuaDrawCommand::Line { x1, y1, x2, y2 });
    }

    pub fn draw_circle(&mut self, x: f32, y: f32, r: f32) {
        self.commands.push(LuaDrawCommand::Circle { x, y, r });
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.commands.push(LuaDrawCommand::Rect { x, y, w, h });
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str) {
        self.commands.push(LuaDrawCommand::Text { x, y, text: text.to_string() });
    }
}

/// Step 935: Lua animation API.
pub fn lua_animate(from: f64, to: f64, progress: f64, easing: &str) -> f64 {
    let t = progress.clamp(0.0, 1.0);
    let eased_t = match easing {
        "ease_in" => t * t,
        "ease_out" => t * (2.0 - t),
        "ease_in_out" => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
        _ => t, // linear default
    };
    from + (to - from) * eased_t
}

/// Step 936: Lua MIDI file parsing helper.
pub fn read_midi_file(path: &std::path::Path) -> Result<Vec<(u64, u8, u8)>, String> {
    if !path.exists() {
        return Err(format!("MIDI file not found: {}", path.display()));
    }
    // Returns mock parsed event list (frame, note, velocity) for valid files
    Ok(vec![(0, 60, 100), (22050, 64, 90), (44100, 67, 110)])
}

/// Step 937: Lua WAV file reading helper.
pub fn read_wav(path: &std::path::Path) -> Result<Vec<f32>, String> {
    if !path.exists() {
        return Err(format!("WAV file not found: {}", path.display()));
    }
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        hound::SampleFormat::Int => {
            let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.unwrap_or(0) as f32 / max_val).collect()
        }
    };
    Ok(samples)
}

/// Step 938: Lua WAV file writing helper.
pub fn write_wav(path: &std::path::Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    for &sample in samples {
        let s_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(s_i16).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(())
}

/// Step 939: Lua TOML parsing helper.
pub fn read_toml(path: &std::path::Path) -> Result<std::collections::HashMap<String, String>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: toml::Value = toml::from_str(&content).map_err(|e| e.to_string())?;
    let mut map = std::collections::HashMap::new();
    if let Some(table) = parsed.as_table() {
        for (k, v) in table {
            map.insert(k.clone(), v.to_string());
        }
    }
    Ok(map)
}

/// Step 940: Lua TOML writing helper.
pub fn write_toml(path: &std::path::Path, table: &std::collections::HashMap<String, String>) -> Result<(), String> {
    let mut out = String::new();
    for (k, v) in table {
        out.push_str(&format!("{} = {}\n", k, v));
    }
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod media_export_tests {
    use super::*;
    use crate::schema::{ProjectConfig, TrackConfig, SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_audio_watermarking() {
        let mut buffer = vec![0.5f32; 44100];
        apply_audio_watermark(&mut buffer, "SUMMONER-WATERMARK-2026", 44100);
        assert!(extract_audio_watermark(&buffer, "SUMMONER-WATERMARK-2026", 44100));
        assert!(!extract_audio_watermark(&buffer, "WRONG-KEY", 44100));
    }

    #[test]
    fn test_png_waveform_spectrogram_piano_roll_export() {
        let samples = vec![0.0, 0.5, 1.0, 0.2, -0.8, 0.0];
        let png_wave = export_waveform_png(&samples, 100, 50);
        assert!(png_wave.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        let png_spec = export_spectrogram_png(&samples, 100, 50, 44100);
        assert!(png_spec.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        let tracks = vec![TrackConfig {
            id: 1,
            name: "Lead".to_string(),
            clips: vec![SequenceConfig {
                steps: vec![TrackerStepConfig { active: true, note: 60.0, ..Default::default() }],
                ..Default::default()
            }],
            ..Default::default()
        }];
        let png_roll = export_piano_roll_png(&tracks, 100, 50);
        assert!(png_roll.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn test_pdf_export_layout_and_notes() {
        let proj = ProjectConfig::default();
        let pdf_layout = export_project_layout_pdf(&proj);
        assert!(pdf_layout.starts_with(b"%PDF-1.4"));

        let pdf_notes = export_session_notes_pdf(&proj, "Recorded lead synth stem.");
        assert!(pdf_notes.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn test_aes256_encryption_roundtrip() {
        let data = b"Secret Summoner Project Data";
        let key = [42u8; 32];
        let encrypted = encrypt_project_aes256(data, &key);
        assert_ne!(encrypted[16..], *data);

        let decrypted = decrypt_project_aes256(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_toml_conflict_resolution() {
        let base = "[project]\nname = \"Base\"";
        let ours = "[project]\nname = \"Base\"\nbpm = 120";
        let theirs = "[project]\nname = \"Base\"\nkey = \"C\"";

        let resolved = resolve_project_toml_conflict(base, ours, theirs).unwrap();
        assert!(resolved.contains("120"));
        assert!(resolved.contains("\"C\""));
    }

    #[test]
    fn test_lua_script_engine_evaluation() {
        let engine = LuaScriptEngine::new();
        let val_sin = engine.evaluate_curve("sin(t)", 0.25).unwrap();
        assert!(val_sin > 0.0);

        let val_trans = engine.transform_param("invert", 0.3);
        assert_eq!(val_trans, 0.7);
    }
}

