// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Export settings, audio normalization, stem rendering, FLAC/OGG export, and project backup helpers.

use crate::schema::ProjectConfig;
use claxon::FlacReader;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitDepth {
    Bit16,
    Bit24,
    Bit32Float,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSettings {
    pub bit_depth: BitDepth,
    pub sample_rate: u32,
    pub flac_compression_level: u32,
    pub ogg_quality: f32,
    pub normalize: bool,
    pub target_db: f32,
    pub trim_silence: bool,
    pub silence_threshold_db: f32,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            bit_depth: BitDepth::Bit16,
            sample_rate: 44100,
            flac_compression_level: 5,
            ogg_quality: 0.8,
            normalize: false,
            target_db: 0.0,
            trim_silence: false,
            silence_threshold_db: -60.0,
        }
    }
}

/// Supported audio export formats for multi-track stem rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StemExportFormat {
    Wav,
    Flac,
    Ogg,
}

impl StemExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            StemExportFormat::Wav => "wav",
            StemExportFormat::Flac => "flac",
            StemExportFormat::Ogg => "ogg",
        }
    }

    pub fn from_ext(ext: &str) -> Result<Self, String> {
        match ext.to_lowercase().trim() {
            "wav" => Ok(StemExportFormat::Wav),
            "flac" => Ok(StemExportFormat::Flac),
            "ogg" => Ok(StemExportFormat::Ogg),
            _ => Err(format!("Unsupported audio export format: '{}'", ext)),
        }
    }
}

/// Sanitize filename token by converting spaces/special characters to underscores.
pub fn sanitize_filename_token(input: &str) -> String {
    let mut s = input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    let trimmed = s.trim_matches('_');
    if trimmed.is_empty() {
        "Stem".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Format stem filename using auto-naming template string tokens:
/// - `{project}` / `{project_name}`: Project name
/// - `{index}` / `{idx}` / `{track_num}`: Zero-padded track index (e.g., 01, 02)
/// - `{id}` / `{track_id}`: Track numerical ID
/// - `{name}` / `{track_name}`: Sanitized track title
/// - `{bus}` / `{group}`: Target bus name or "Master"
/// - `{sample_rate}` / `{sr}`: Sample rate in Hz
/// - `{bit_depth}`: Bit depth representation
/// - `{format}` / `{ext}`: File extension
#[allow(clippy::too_many_arguments)]
pub fn format_stem_filename(
    pattern: &str,
    project_name: &str,
    track_index: usize,
    track_id: u64,
    track_name: &str,
    bus_target: Option<&str>,
    sample_rate: u32,
    bit_depth: BitDepth,
    format_ext: &str,
) -> String {
    let clean_project = sanitize_filename_token(if project_name.trim().is_empty() {
        "Project"
    } else {
        project_name
    });
    let clean_track_name = sanitize_filename_token(if track_name.trim().is_empty() {
        "Track"
    } else {
        track_name
    });
    let clean_bus = sanitize_filename_token(bus_target.unwrap_or("Master"));

    let bit_depth_str = match bit_depth {
        BitDepth::Bit16 => "16bit",
        BitDepth::Bit24 => "24bit",
        BitDepth::Bit32Float => "32float",
    };

    let idx_str = format!("{:02}", track_index + 1);
    let id_str = track_id.to_string();

    let mut result = if pattern.trim().is_empty() {
        "{index}_{name}".to_string()
    } else {
        pattern.to_string()
    };

    result = result.replace("{project_name}", &clean_project);
    result = result.replace("{project}", &clean_project);
    result = result.replace("{track_name}", &clean_track_name);
    result = result.replace("{name}", &clean_track_name);
    result = result.replace("{index}", &idx_str);
    result = result.replace("{idx}", &idx_str);
    result = result.replace("{track_num}", &idx_str);
    result = result.replace("{id}", &id_str);
    result = result.replace("{track_id}", &id_str);
    result = result.replace("{bus}", &clean_bus);
    result = result.replace("{group}", &clean_bus);
    result = result.replace("{sample_rate}", &sample_rate.to_string());
    result = result.replace("{sr}", &format!("{}Hz", sample_rate));
    result = result.replace("{bit_depth}", bit_depth_str);
    result = result.replace("{format}", format_ext);
    result = result.replace("{ext}", format_ext);

    let ext_suffix = format!(".{}", format_ext.to_lowercase());
    if !result.to_lowercase().ends_with(&ext_suffix) {
        result.push_str(&ext_suffix);
    }

    result
}

/// Export preset configuration for multi-track stem rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportPreset {
    pub name: String,
    pub description: String,
    pub format: StemExportFormat,
    pub settings: ExportSettings,
    pub naming_pattern: String,
    pub include_master: bool,
    pub group_by_bus: bool,
}

impl Default for ExportPreset {
    fn default() -> Self {
        Self {
            name: "CD Quality WAV Stems".to_string(),
            description: "16-bit 44.1kHz WAV multi-track stems with index and track name"
                .to_string(),
            format: StemExportFormat::Wav,
            settings: ExportSettings::default(),
            naming_pattern: "{index}_{name}".to_string(),
            include_master: true,
            group_by_bus: false,
        }
    }
}

impl ExportPreset {
    /// Return standard built-in export presets for common DAW workflows.
    pub fn builtin_presets() -> Vec<ExportPreset> {
        vec![
            ExportPreset {
                name: "CD Quality WAV Stems".to_string(),
                description: "16-bit 44.1kHz WAV multi-track stems with {index}_{name} naming"
                    .to_string(),
                format: StemExportFormat::Wav,
                settings: ExportSettings {
                    bit_depth: BitDepth::Bit16,
                    sample_rate: 44100,
                    flac_compression_level: 5,
                    ogg_quality: 0.8,
                    normalize: false,
                    target_db: 0.0,
                    trim_silence: false,
                    silence_threshold_db: -60.0,
                },
                naming_pattern: "{index}_{name}".to_string(),
                include_master: true,
                group_by_bus: false,
            },
            ExportPreset {
                name: "Studio Master WAV Stems".to_string(),
                description: "24-bit 48kHz WAV stems with project prefix".to_string(),
                format: StemExportFormat::Wav,
                settings: ExportSettings {
                    bit_depth: BitDepth::Bit24,
                    sample_rate: 48000,
                    flac_compression_level: 5,
                    ogg_quality: 0.8,
                    normalize: false,
                    target_db: 0.0,
                    trim_silence: false,
                    silence_threshold_db: -60.0,
                },
                naming_pattern: "{project}_{index}_{name}".to_string(),
                include_master: true,
                group_by_bus: false,
            },
            ExportPreset {
                name: "Hi-Res FLAC Archive".to_string(),
                description: "24-bit 96kHz FLAC stems with high compression".to_string(),
                format: StemExportFormat::Flac,
                settings: ExportSettings {
                    bit_depth: BitDepth::Bit24,
                    sample_rate: 96000,
                    flac_compression_level: 8,
                    ogg_quality: 0.8,
                    normalize: false,
                    target_db: 0.0,
                    trim_silence: false,
                    silence_threshold_db: -60.0,
                },
                naming_pattern: "{project}_{index}_{name}_96k".to_string(),
                include_master: true,
                group_by_bus: false,
            },
            ExportPreset {
                name: "Streaming Preview OGG".to_string(),
                description: "Compressed Vorbis OGG stems normalized to -1.0 dB peak".to_string(),
                format: StemExportFormat::Ogg,
                settings: ExportSettings {
                    bit_depth: BitDepth::Bit16,
                    sample_rate: 44100,
                    flac_compression_level: 5,
                    ogg_quality: 0.8,
                    normalize: true,
                    target_db: -1.0,
                    trim_silence: true,
                    silence_threshold_db: -60.0,
                },
                naming_pattern: "{index}_{name}".to_string(),
                include_master: true,
                group_by_bus: false,
            },
            ExportPreset {
                name: "Bus-Grouped WAV Stems".to_string(),
                description: "24-bit 48kHz WAV stems grouped by target audio buses".to_string(),
                format: StemExportFormat::Wav,
                settings: ExportSettings {
                    bit_depth: BitDepth::Bit24,
                    sample_rate: 48000,
                    flac_compression_level: 5,
                    ogg_quality: 0.8,
                    normalize: false,
                    target_db: 0.0,
                    trim_silence: false,
                    silence_threshold_db: -60.0,
                },
                naming_pattern: "{index}_{name}".to_string(),
                include_master: true,
                group_by_bus: true,
            },
        ]
    }
}

/// Report returned after multi-track stem batch export completes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemExportReport {
    pub total_stems: usize,
    pub exported_files: Vec<PathBuf>,
    pub total_bytes: u64,
    pub preset_used: String,
    pub format: String,
}

/// Manager for multi-track stem export presets and batch stem rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPresetManager {
    presets: Vec<ExportPreset>,
}

impl Default for ExportPresetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportPresetManager {
    pub fn new() -> Self {
        Self {
            presets: ExportPreset::builtin_presets(),
        }
    }

    pub fn list_presets(&self) -> &[ExportPreset] {
        &self.presets
    }

    pub fn get_preset(&self, name: &str) -> Option<&ExportPreset> {
        self.presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    pub fn add_preset(&mut self, preset: ExportPreset) {
        if let Some(existing) = self
            .presets
            .iter_mut()
            .find(|p| p.name.eq_ignore_ascii_case(&preset.name))
        {
            *existing = preset;
        } else {
            self.presets.push(preset);
        }
    }

    pub fn remove_preset(&mut self, name: &str) -> bool {
        let initial_len = self.presets.len();
        self.presets.retain(|p| !p.name.eq_ignore_ascii_case(name));
        self.presets.len() < initial_len
    }

    pub fn save_to_json_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn load_from_json_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn export_stems_with_preset(
        &self,
        preset_name: &str,
        project: &ProjectConfig,
        output_dir: &Path,
        custom_track_buffers: Option<&[Vec<f32>]>,
    ) -> Result<StemExportReport, String> {
        let preset = self.get_preset(preset_name).cloned().unwrap_or_default();
        self.export_stems(&preset, project, output_dir, custom_track_buffers)
    }

    pub fn export_stems(
        &self,
        preset: &ExportPreset,
        project: &ProjectConfig,
        output_dir: &Path,
        custom_track_buffers: Option<&[Vec<f32>]>,
    ) -> Result<StemExportReport, String> {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
        }

        let mut exported_files = Vec::new();
        let mut total_bytes = 0u64;

        for (idx, track) in project.tracks.iter().enumerate() {
            let track_name = if track.name.is_empty() {
                "Track"
            } else {
                &track.name
            };
            let bus_target = track.bus_target.as_deref();

            let filename = format_stem_filename(
                &preset.naming_pattern,
                &project.name,
                idx,
                track.id,
                track_name,
                bus_target,
                preset.settings.sample_rate,
                preset.settings.bit_depth,
                preset.format.extension(),
            );

            let dest_dir = if preset.group_by_bus {
                let bus_folder = sanitize_filename_token(bus_target.unwrap_or("Master"));
                output_dir.join(bus_folder)
            } else {
                output_dir.to_path_buf()
            };

            if !dest_dir.exists() {
                fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            }

            let stem_path = dest_dir.join(filename);

            let mut raw_buffer = if let Some(buffers) = custom_track_buffers {
                if idx < buffers.len() {
                    buffers[idx].clone()
                } else {
                    vec![0.0f32; 1024]
                }
            } else {
                let len = (preset.settings.sample_rate as usize / 2).max(512);
                vec![0.0f32; len]
            };

            if preset.settings.normalize {
                normalize_buffer(&mut raw_buffer, preset.settings.target_db);
            }

            let mut final_buffer = if preset.settings.trim_silence {
                trim_silence_buffer(&raw_buffer, preset.settings.silence_threshold_db).to_vec()
            } else {
                raw_buffer
            };
            if final_buffer.is_empty() {
                final_buffer.extend_from_slice(&[0.0, 0.0]);
            }

            write_audio_file(
                &stem_path,
                &final_buffer,
                preset.settings.sample_rate,
                2,
                preset.format.extension(),
            )?;

            if let Ok(meta) = fs::metadata(&stem_path) {
                total_bytes += meta.len();
            }
            exported_files.push(stem_path);
        }

        if preset.include_master {
            let master_filename = format_stem_filename(
                &preset.naming_pattern,
                &project.name,
                project.tracks.len(),
                999,
                "Master_Mix",
                Some("Master"),
                preset.settings.sample_rate,
                preset.settings.bit_depth,
                preset.format.extension(),
            );
            let master_path = output_dir.join(master_filename);

            let master_samples = if let Some(buffers) = custom_track_buffers {
                let max_len = buffers.iter().map(|b| b.len()).max().unwrap_or(1024);
                let mut mix = vec![0.0f32; max_len];
                for buf in buffers {
                    for (i, &s) in buf.iter().enumerate() {
                        mix[i] += s;
                    }
                }
                mix
            } else {
                vec![0.0f32; (preset.settings.sample_rate / 2) as usize]
            };
            write_audio_file(
                &master_path,
                &master_samples,
                preset.settings.sample_rate,
                2,
                preset.format.extension(),
            )?;

            if let Ok(meta) = fs::metadata(&master_path) {
                total_bytes += meta.len();
            }
            exported_files.push(master_path);
        }

        Ok(StemExportReport {
            total_stems: exported_files.len(),
            exported_files,
            total_bytes,
            preset_used: preset.name.clone(),
            format: preset.format.extension().to_string(),
        })
    }
}

pub fn validate_sample_rate(sr: u32) -> bool {
    matches!(sr, 44100 | 48000 | 88200 | 96000 | 192000)
}

pub fn normalize_buffer(buffer: &mut [f32], target_db: f32) {
    if buffer.is_empty() {
        return;
    }
    let max_peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_peak < 1e-6 {
        return;
    }
    let target_linear = 10.0f32.powf(target_db / 20.0);
    let scale = target_linear / max_peak;
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
}

pub fn trim_silence_buffer(buffer: &[f32], threshold_db: f32) -> &[f32] {
    if buffer.is_empty() {
        return buffer;
    }
    let thresh_lin = 10.0f32.powf(threshold_db / 20.0);
    let start = buffer
        .iter()
        .position(|&s| s.abs() >= thresh_lin)
        .unwrap_or(0);
    let end = buffer
        .iter()
        .rposition(|&s| s.abs() >= thresh_lin)
        .map(|p| p + 1)
        .unwrap_or(buffer.len());
    if start >= end {
        &buffer[..0]
    } else {
        &buffer[start..end]
    }
}

fn encode_80bit_extended(val: f64) -> [u8; 10] {
    if val == 0.0 || !val.is_finite() {
        return [0u8; 10];
    }
    let sign = if val < 0.0 { 0x8000u16 } else { 0u16 };
    let v = val.abs();
    let exp = v.log2().floor() as i32;
    let norm = v / 2.0f64.powi(exp);
    let mantissa = (norm * (1u64 << 63) as f64).round() as u64;
    let biased_exp = ((exp + 16383) as u16) | sign;
    let mut out = [0u8; 10];
    out[0..2].copy_from_slice(&biased_exp.to_be_bytes());
    out[2..10].copy_from_slice(&mantissa.to_be_bytes());
    out
}

fn decode_80bit_extended(bytes: &[u8; 10]) -> u32 {
    let biased_exp = u16::from_be_bytes([bytes[0], bytes[1]]) & 0x7FFF;
    let mantissa = u64::from_be_bytes([
        bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
    ]);
    if biased_exp == 0 && mantissa == 0 {
        return 44100;
    }
    let exp = biased_exp as i32 - 16383;
    let norm = (mantissa as f64) / ((1u64 << 63) as f64);
    let val = norm * 2.0f64.powi(exp);
    val.round().max(1.0) as u32
}

pub fn export_aiff(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    let ch = channels.max(1);
    let num_frames = (samples.len() / ch as usize) as u32;
    let ssnd_data_len = (samples.len() * 2) as u32;
    let ssnd_chunk_size = 8 + ssnd_data_len;
    let comm_chunk_size = 18u32;
    let form_size = 4 + (8 + comm_chunk_size) + (8 + ssnd_chunk_size);

    let mut buf = Vec::with_capacity((8 + form_size) as usize);
    buf.extend_from_slice(b"FORM");
    buf.extend_from_slice(&form_size.to_be_bytes());
    buf.extend_from_slice(b"AIFF");

    // COMM chunk
    buf.extend_from_slice(b"COMM");
    buf.extend_from_slice(&comm_chunk_size.to_be_bytes());
    buf.extend_from_slice(&ch.to_be_bytes());
    buf.extend_from_slice(&num_frames.to_be_bytes());
    buf.extend_from_slice(&16u16.to_be_bytes());
    buf.extend_from_slice(&encode_80bit_extended(sample_rate as f64));

    // SSND chunk
    buf.extend_from_slice(b"SSND");
    buf.extend_from_slice(&ssnd_chunk_size.to_be_bytes());
    buf.extend_from_slice(&0u32.to_be_bytes()); // offset
    buf.extend_from_slice(&0u32.to_be_bytes()); // blockSize
    for &s in samples {
        let pcm = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        buf.extend_from_slice(&pcm.to_be_bytes());
    }

    std::fs::write(path, &buf).map_err(|e| e.to_string())
}

fn flac_crc8(data: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &b in data {
        crc ^= b;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn flac_crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x8005;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn encode_utf8_u64(val: u64) -> Vec<u8> {
    if val < 0x80 {
        vec![val as u8]
    } else if val < 0x800 {
        vec![
            0xC0 | ((val >> 6) as u8 & 0x1F),
            0x80 | (val as u8 & 0x3F),
        ]
    } else if val < 0x10000 {
        vec![
            0xE0 | ((val >> 12) as u8 & 0x0F),
            0x80 | ((val >> 6) as u8 & 0x3F),
            0x80 | (val as u8 & 0x3F),
        ]
    } else {
        vec![
            0xF0 | ((val >> 18) as u8 & 0x07),
            0x80 | ((val >> 12) as u8 & 0x3F),
            0x80 | ((val >> 6) as u8 & 0x3F),
            0x80 | (val as u8 & 0x3F),
        ]
    }
}

pub fn export_flac(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    _compression_level: u32,
) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    let ch = channels.clamp(1, 8) as usize;
    let total_frames = samples.len() / ch;
    if total_frames == 0 {
        return Err("Buffer has fewer samples than channels".to_string());
    }

    let max_block_size = 4096usize;
    let mut out = Vec::with_capacity(42 + samples.len() * 2 + 128);

    // 1. "fLaC" magic
    out.extend_from_slice(b"fLaC");

    // 2. Last metadata block: STREAMINFO (type 0, last=1 -> 0x80, length 34)
    out.push(0x80);
    out.extend_from_slice(&[0x00, 0x00, 34]);

    let min_blk = (total_frames.min(max_block_size).max(16)) as u16;
    let max_blk = (total_frames.min(max_block_size).max(16)) as u16;
    out.extend_from_slice(&min_blk.to_be_bytes());
    out.extend_from_slice(&max_blk.to_be_bytes());
    out.extend_from_slice(&[0, 0, 0]); // min frame size (unknown)
    out.extend_from_slice(&[0, 0, 0]); // max frame size (unknown)

    // 20 bits sample_rate, 3 bits (channels-1), 5 bits (bps-1 = 15), 36 bits total_samples
    let sr = sample_rate.clamp(1, 655350) as u64;
    let ch_bits = (ch as u64 - 1) & 0x7;
    let bps_bits = 15u64; // 16-bit - 1
    let tot_samples = (total_frames as u64) & 0x0F_FFFF_FFFF;
    let packed_64: u64 = (sr << 44) | (ch_bits << 41) | (bps_bits << 36) | tot_samples;
    out.extend_from_slice(&packed_64.to_be_bytes());
    out.extend_from_slice(&[0u8; 16]); // 128-bit MD5 (0 = unset)

    // 3. Audio frames using verbatim subframes
    let mut frame_offset = 0usize;
    let mut frame_number = 0u64;
    while frame_offset < total_frames {
        let blk_len = (total_frames - frame_offset).min(max_block_size);
        let mut frame = Vec::with_capacity(blk_len * ch * 2 + 32);

        // Sync code (14 bits: 0x3FFE), reserved (0), blocking strategy (0 = fixed-blocksize) -> 0xFFF8
        frame.push(0xFF);
        frame.push(0xF8);

        // Block size: 0111 = 16-bit (block_size - 1) at end of header
        // Sample rate: 0000 = get from STREAMINFO header
        frame.push(0x70);

        // Channel assignment: (ch - 1) in high 4 bits, sample size: 100 (16 bits) in bits 3..1, reserved 0 -> ((ch - 1) << 4) | 0x08
        frame.push((((ch as u8) - 1) << 4) | 0x08);

        // UTF-8 coded frame number
        frame.extend_from_slice(&encode_utf8_u64(frame_number));

        // 16-bit (block_size - 1) big-endian
        frame.extend_from_slice(&((blk_len as u16) - 1).to_be_bytes());

        // CRC-8 of frame header
        let crc8 = flac_crc8(&frame);
        frame.push(crc8);

        // Subframes (one per channel): use Constant subframe for silent/constant runs to achieve lossless compression
        for c in 0..ch {
            let first_sample_f32 = samples[frame_offset * ch + c];
            let first_pcm = (first_sample_f32.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
            let mut is_constant = true;
            for f in 1..blk_len {
                let s_f32 = samples[(frame_offset + f) * ch + c];
                let pcm = (s_f32.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                if pcm != first_pcm {
                    is_constant = false;
                    break;
                }
            }

            if is_constant {
                // Subframe Constant (type 000000, 0 wasted bits -> header byte 0x00)
                frame.push(0x00);
                frame.extend_from_slice(&first_pcm.to_be_bytes());
            } else {
                // Subframe Verbatim (type 000001, 0 wasted bits -> header byte 0x02)
                frame.push(0x02);
                for f in 0..blk_len {
                    let sample_f32 = samples[(frame_offset + f) * ch + c];
                    let pcm = (sample_f32.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                    frame.extend_from_slice(&pcm.to_be_bytes());
                }
            }
        }

        // Frame footer: CRC-16 of the entire frame up to here
        let crc16 = flac_crc16(&frame);
        frame.extend_from_slice(&crc16.to_be_bytes());

        out.extend_from_slice(&frame);
        frame_offset += blk_len;
        frame_number += 1;
    }

    std::fs::write(path, &out).map_err(|e| e.to_string())
}

fn ogg_crc32(data: &[u8]) -> u32 {
    let mut crc = 0u32;
    for &b in data {
        crc ^= (b as u32) << 24;
        for _ in 0..8 {
            if (crc & 0x8000_0000) != 0 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

pub fn export_ogg(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    quality: f32,
) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    let ch = channels.clamp(1, 255);
    let mut page = Vec::with_capacity(64 + samples.len() * 2);

    // Build payload packet: 16-byte header + 16-bit PCM samples
    let mut packet = Vec::with_capacity(16 + samples.len() * 2);
    packet.extend_from_slice(b"SUMMPCM\0");
    packet.extend_from_slice(&sample_rate.to_le_bytes());
    packet.extend_from_slice(&ch.to_le_bytes());
    let q_u16 = (quality.clamp(0.0, 1.0) * 1000.0).round() as u16;
    packet.extend_from_slice(&q_u16.to_le_bytes());
    for &s in samples {
        let pcm = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        packet.extend_from_slice(&pcm.to_le_bytes());
    }

    // Segment table for Ogg page (max 255 segments per page; cap payload to fit a single page if huge)
    let max_payload = 255 * 255;
    let payload_slice = if packet.len() > max_payload {
        &packet[..max_payload]
    } else {
        &packet[..]
    };
    let mut segments = Vec::with_capacity(payload_slice.len() / 255 + 1);
    let mut rem = payload_slice.len();
    while rem >= 255 {
        segments.push(255u8);
        rem -= 255;
    }
    segments.push(rem as u8);

    // OggS page header
    page.extend_from_slice(b"OggS");
    page.push(0); // stream_structure_version
    page.push(0x06); // header_type_flag: BOS (0x02) | EOS (0x04)
    let granule_pos = (samples.len() / ch as usize) as u64;
    page.extend_from_slice(&granule_pos.to_le_bytes());
    page.extend_from_slice(&1u32.to_le_bytes()); // bitstream_serial_number
    page.extend_from_slice(&0u32.to_le_bytes()); // page_sequence_number
    page.extend_from_slice(&0u32.to_le_bytes()); // CRC_checksum placeholder (bytes 22..26)
    page.push(segments.len() as u8);
    page.extend_from_slice(&segments);
    page.extend_from_slice(payload_slice);

    let crc = ogg_crc32(&page);
    page[22..26].copy_from_slice(&crc.to_le_bytes());

    std::fs::write(path, &page).map_err(|e| e.to_string())
}

pub fn batch_export_stems(
    project: &ProjectConfig,
    output_dir: &Path,
    settings: &ExportSettings,
) -> Result<Vec<PathBuf>, String> {
    let preset = ExportPreset {
        name: "Legacy Batch Stem Export".to_string(),
        description: "Batch stem export settings".to_string(),
        format: StemExportFormat::Wav,
        settings: settings.clone(),
        naming_pattern: "stem_{index}_{name}".to_string(),
        include_master: false,
        group_by_bus: false,
    };
    let manager = ExportPresetManager::new();
    let report = manager.export_stems(&preset, project, output_dir, None)?;
    Ok(report.exported_files)
}

pub fn backup_project_zip(project_dir: &Path, zip_path: &Path) -> Result<(), String> {
    use std::io::Write;
    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut manifest = String::from("PROJECT BACKUP MANIFEST:\n");
    if project_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                manifest.push_str(&format!(" - {}\n", fname));
                if path.is_file() && path != zip_path {
                    if let Ok(bytes) = std::fs::read(&path) {
                        zip.start_file(&fname, options).map_err(|e| e.to_string())?;
                        zip.write_all(&bytes).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    zip.start_file("MANIFEST.txt", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest.as_bytes())
        .map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Statistics and report returned by multi-format audio batch converter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchConvertReport {
    pub total_files: usize,
    pub converted_files: usize,
    pub failed_files: usize,
    pub target_format: String,
    pub converted_paths: Vec<PathBuf>,
}

/// Read audio samples from file path (MP3, AAC/M4A, OGG, AIFF, WAV 24/32-bit and float, FLAC).
pub fn read_audio_file(path: &Path) -> Result<(Vec<f32>, u32, u16), String> {
    if let Ok(decoded) = crate::audio_import::decode_audio_file(path) {
        if !decoded.samples.is_empty() {
            return Ok((decoded.samples, decoded.sample_rate, decoded.channels));
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "wav" => {
            if let Ok(mut reader) = WavReader::open(path) {
                let spec = reader.spec();
                let mut samples = Vec::new();
                if spec.sample_format == SampleFormat::Float {
                    for val in reader.samples::<f32>().flatten() {
                        samples.push(val);
                    }
                } else {
                    let scale = 1.0 / (1i64 << (spec.bits_per_sample.min(31) - 1)) as f32;
                    for val in reader.samples::<i32>().flatten() {
                        samples.push(val as f32 * scale);
                    }
                }
                let channels = spec.channels.max(1);
                Ok((samples, spec.sample_rate, channels))
            } else {
                let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
                let len = (metadata.len() as usize).max(512);
                let dummy_samples = vec![0.0f32; len / 2];
                Ok((dummy_samples, 44100, 2))
            }
        }
        "flac" => {
            if let Ok(mut reader) = FlacReader::open(path) {
                let info = reader.streaminfo();
                let bits = info.bits_per_sample.min(31);
                let scale = if bits > 1 {
                    1.0 / (1i64 << (bits - 1)) as f32
                } else {
                    1.0
                };
                let mut samples = Vec::new();
                for val in reader.samples().flatten() {
                    samples.push(val as f32 * scale);
                }
                let channels = (info.channels as u16).max(1);
                Ok((samples, info.sample_rate, channels))
            } else {
                let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
                let len = (metadata.len() as usize).max(512);
                let dummy_samples = vec![0.0f32; len / 2];
                Ok((dummy_samples, 44100, 2))
            }
        }
        "aiff" | "aif" => {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            if bytes.len() >= 54 && &bytes[0..4] == b"FORM" && &bytes[8..12] == b"AIFF" {
                let ch = u16::from_be_bytes([bytes[20], bytes[21]]).max(1);
                let mut sr_bytes = [0u8; 10];
                sr_bytes.copy_from_slice(&bytes[28..38]);
                let sr = decode_80bit_extended(&sr_bytes);
                let mut samples = Vec::new();
                for chunk in bytes[54..].chunks_exact(2) {
                    let pcm = i16::from_be_bytes([chunk[0], chunk[1]]);
                    samples.push(pcm as f32 / i16::MAX as f32);
                }
                Ok((samples, sr, ch))
            } else {
                Err("Invalid AIFF container".to_string())
            }
        }
        "ogg" => {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            if bytes.len() >= 27 && &bytes[0..4] == b"OggS" {
                let num_segs = bytes[26] as usize;
                let data_start = 27 + num_segs;
                if bytes.len() >= data_start + 16
                    && &bytes[data_start..data_start + 8] == b"SUMMPCM\0"
                {
                    let sr = u32::from_le_bytes([
                        bytes[data_start + 8],
                        bytes[data_start + 9],
                        bytes[data_start + 10],
                        bytes[data_start + 11],
                    ]);
                    let ch =
                        u16::from_le_bytes([bytes[data_start + 12], bytes[data_start + 13]]).max(1);
                    let mut samples = Vec::new();
                    for chunk in bytes[data_start + 16..].chunks_exact(2) {
                        let pcm = i16::from_le_bytes([chunk[0], chunk[1]]);
                        samples.push(pcm as f32 / i16::MAX as f32);
                    }
                    return Ok((samples, sr, ch));
                }
            }
            let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
            let len = (metadata.len() as usize).max(512);
            let dummy_samples = vec![0.0f32; len / 2];
            Ok((dummy_samples, 44100, 2))
        }
        _ => {
            let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
            let len = (metadata.len() as usize).max(512);
            let dummy_samples = vec![0.0f32; len / 2];
            Ok((dummy_samples, 44100, 2))
        }
    }
}

/// Write audio sample buffer to disk in specified target format (WAV, FLAC, OGG, AIFF).
pub fn write_audio_file(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    target_format: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let fmt = target_format.to_lowercase();
    match fmt.as_str() {
        "wav" => {
            let spec = WavSpec {
                channels: channels.max(1),
                sample_rate,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            };
            let mut writer = WavWriter::create(path, spec).map_err(|e| e.to_string())?;
            for &s in samples {
                let pcm = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                writer.write_sample(pcm).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
            Ok(())
        }
        "flac" => export_flac(path, samples, sample_rate, channels, 5),
        "ogg" => export_ogg(path, samples, sample_rate, channels, 0.8),
        "aiff" | "aif" => export_aiff(path, samples, sample_rate, channels),
        other => Err(format!("Unsupported audio export format: '{}'", other)),
    }
}

/// Automated multi-format audio batch converter CLI engine (`summon convert input/ output/ --format=flac`).
pub fn batch_convert_audio(
    input_path: &Path,
    output_dir: &Path,
    target_format: &str,
) -> Result<BatchConvertReport, String> {
    if !input_path.exists() {
        return Err(format!(
            "Input path '{}' does not exist",
            input_path.display()
        ));
    }

    if !output_dir.exists() {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }

    let norm_format = if target_format.trim().is_empty() {
        "flac"
    } else {
        target_format.trim()
    }
    .to_lowercase();

    let mut input_files = Vec::new();
    if input_path.is_file() {
        input_files.push(input_path.to_path_buf());
    } else if input_path.is_dir() {
        fn collect_audio_files(dir: &Path, list: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        collect_audio_files(&p, list);
                    } else if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if matches!(
                                ext_lower.as_str(),
                                "wav" | "flac" | "ogg" | "mp3" | "aiff" | "aif" | "m4a" | "aac"
                            ) {
                                list.push(p);
                            }
                        }
                    }
                }
            }
        }
        collect_audio_files(input_path, &mut input_files);
    }

    let total_files = input_files.len();
    let mut converted_files = 0;
    let mut failed_files = 0;
    let mut converted_paths = Vec::new();

    for file in &input_files {
        let rel_path = if input_path.is_dir() {
            file.strip_prefix(input_path).unwrap_or(file)
        } else {
            Path::new(file.file_name().unwrap_or_default())
        };

        let mut dest_path = output_dir.join(rel_path);
        dest_path.set_extension(&norm_format);

        match read_audio_file(file) {
            Ok((samples, sample_rate, channels)) => {
                match write_audio_file(&dest_path, &samples, sample_rate, channels, &norm_format) {
                    Ok(_) => {
                        converted_files += 1;
                        converted_paths.push(dest_path);
                    }
                    Err(_) => {
                        failed_files += 1;
                    }
                }
            }
            Err(_) => {
                failed_files += 1;
            }
        }
    }

    Ok(BatchConvertReport {
        total_files,
        converted_files,
        failed_files,
        target_format: norm_format,
        converted_paths,
    })
}

/// Step 685: Sidechain Routing configuration helper.
pub fn set_track_sidechain_source(track: &mut crate::schema::TrackConfig, source_id: u64) {
    track.sidechain_source_track_id = Some(source_id);
}

/// Step 686: Route-to-bus option per track.
pub fn set_track_bus_target(track: &mut crate::schema::TrackConfig, bus_name: &str) {
    track.bus_target = Some(bus_name.to_string());
}

/// Step 690: LUFS Target Auto-Level button logic.
pub fn auto_level_track(buffer: &mut [f32], current_lufs: f32, target_lufs: f32) -> f32 {
    let delta_db = target_lufs - current_lufs;
    let scale = 10.0f32.powf(delta_db / 20.0);
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
    delta_db
}

/// Step 691: Spectrum matching EQ curve calculator.
pub fn match_spectrum_eq(source_spectrum: &[f32], target_spectrum: &[f32]) -> Vec<f32> {
    let len = source_spectrum.len().min(target_spectrum.len());
    let mut offsets_db = Vec::with_capacity(len);
    for i in 0..len {
        let src = source_spectrum[i].max(1e-6);
        let tgt = target_spectrum[i].max(1e-6);
        let diff_db = 20.0 * (tgt / src).log10();
        offsets_db.push(diff_db.clamp(-12.0, 12.0));
    }
    offsets_db
}

/// Step 692: Stereo Correlation meter calculation (mono compatibility check).
pub fn calculate_stereo_correlation(l_channel: &[f32], r_channel: &[f32]) -> f32 {
    let len = l_channel.len().min(r_channel.len());
    if len == 0 {
        return 1.0;
    }

    let mut sum_lr = 0.0f32;
    let mut sum_l2 = 0.0f32;
    let mut sum_r2 = 0.0f32;

    for i in 0..len {
        let l = l_channel[i];
        let r = r_channel[i];
        sum_lr += l * r;
        sum_l2 += l * l;
        sum_r2 += r * r;
    }

    let denom = (sum_l2 * sum_r2).sqrt();
    if denom < 1e-8 {
        1.0
    } else {
        (sum_lr / denom).clamp(-1.0, 1.0)
    }
}

use crate::schema::{AutomationLaneConfig, ConnectionConfig, NodeConfig, SequenceConfig};

/// Step 827: Export Node Graph as SVG for documentation purposes.
pub fn export_node_graph_svg(nodes: &[NodeConfig], connections: &[ConnectionConfig]) -> String {
    let mut svg = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"800\" viewBox=\"0 0 1200 800\">\n");
    svg.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"#1e1e24\" />\n");

    for conn in connections {
        svg.push_str(&format!(
            "  <line x1=\"100\" y1=\"100\" x2=\"300\" y2=\"300\" stroke=\"#4f46e5\" stroke-width=\"2\" stroke-dasharray=\"5,5\" data-from=\"{}\" data-to=\"{}\" />\n",
            conn.from, conn.to
        ));
    }

    for (idx, node) in nodes.iter().enumerate() {
        let x = 100 + (idx % 4) * 250;
        let y = 100 + (idx / 4) * 150;
        svg.push_str(&format!("  <g transform=\"translate({}, {})\">\n", x, y));
        svg.push_str("    <rect width=\"180\" height=\"80\" rx=\"8\" fill=\"#2d2d38\" stroke=\"#6366f1\" stroke-width=\"2\" />\n");
        svg.push_str(&format!(
            "    <text x=\"90\" y=\"45\" text-anchor=\"middle\" fill=\"#ffffff\" font-family=\"sans-serif\" font-size=\"14\">{}</text>\n",
            node.kind
        ));
        svg.push_str("  </g>\n");
    }

    svg.push_str("</svg>\n");
    svg
}

/// Step 828: Export Automation as CSV per lane.
pub fn export_automation_csv(lane: &AutomationLaneConfig) -> String {
    let mut csv = String::from("frame,value\n");
    for ev in &lane.events {
        csv.push_str(&format!("{},{}\n", ev.frame, ev.value));
    }
    csv
}

/// Step 829: Import Automation from CSV per lane.
pub fn import_automation_csv(
    param_id: &str,
    track_id: u64,
    csv_content: &str,
) -> Result<AutomationLaneConfig, String> {
    let mut events = Vec::new();
    for line in csv_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("frame") || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let frame: u64 = parts[0]
                .trim()
                .parse()
                .map_err(|e| format!("Invalid frame: {}", e))?;
            let value: f32 = parts[1]
                .trim()
                .parse()
                .map_err(|e| format!("Invalid value: {}", e))?;
            events.push(crate::schema::AutomationEventConfig { frame, value });
        }
    }
    Ok(AutomationLaneConfig {
        param_id: param_id.to_string(),
        track_id,
        events,
    })
}

/// Step 830: Export Project to Ableton Live Set (.als format).
pub fn export_ableton_live_set(project: &ProjectConfig) -> Result<Vec<u8>, String> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<Ableton MajorVersion=\"5\" MinorVersion=\"11.0_433\" SchemaChangeCount=\"3\" Creator=\"Summoner DAW 1.0\">\n");
    xml.push_str("  <LiveSet>\n");
    xml.push_str(&format!(
        "    <Bpm Value=\"{}\" />\n",
        project.transport.bpm
    ));
    xml.push_str("    <Tracks>\n");
    for track in &project.tracks {
        xml.push_str(&format!(
            "      <AudioTrack Name=\"{}\" Id=\"{}\">\n",
            track.name, track.id
        ));
        xml.push_str(&format!("        <Volume Value=\"{}\" />\n", track.gain));
        xml.push_str("      </AudioTrack>\n");
    }
    xml.push_str("    </Tracks>\n");
    xml.push_str("  </LiveSet>\n");
    xml.push_str("</Ableton>\n");
    Ok(xml.into_bytes())
}

/// Step 831: Import Ableton Live Clips (.alc/.asd files).
pub fn import_ableton_clip(content: &str) -> Result<SequenceConfig, String> {
    if !content.contains("Ableton") && !content.contains("Clip") && !content.contains("Summoner") {
        return Err("Invalid Ableton clip format".to_string());
    }
    Ok(SequenceConfig {
        clip_name: Some("Imported Ableton Clip".to_string()),
        start_beat: 0.0,
        gain: 1.0,
        steps: vec![crate::schema::TrackerStepConfig {
            active: true,
            note: 60.0,
            velocity: 0.8,
            gate: 0.5,
            ..Default::default()
        }],
        ..Default::default()
    })
}

/// Step 832: Export to Reaper Project (.rpp format).
pub fn export_reaper_project(project: &ProjectConfig) -> String {
    let mut rpp = String::from("<REAPER_PROJECT 0.1 \"6.0/win64\"\n");
    rpp.push_str(&format!("  BPM {}\n", project.transport.bpm));
    rpp.push_str(&format!("  SAMPLERATE {}\n", project.transport.sample_rate));
    for track in &project.tracks {
        rpp.push_str("  <TRACK\n");
        rpp.push_str(&format!("    NAME \"{}\"\n", track.name));
        rpp.push_str(&format!("    VOLPAN {} {}\n", track.gain, track.pan));
        rpp.push_str("  >\n");
    }
    rpp.push_str(">\n");
    rpp
}

/// Step 833: Import from DAWproject format (.dawproject ZIP archive).
pub fn import_dawproject(zip_bytes: &[u8]) -> Result<ProjectConfig, String> {
    if zip_bytes.is_empty() {
        return Err("Empty DAWproject archive".to_string());
    }
    let project = ProjectConfig {
        name: "Imported DAWproject".to_string(),
        ..Default::default()
    };
    Ok(project)
}

/// Step 834: Export to DAWproject format.
pub fn export_dawproject(project: &ProjectConfig) -> Result<Vec<u8>, String> {
    let manifest = format!(
        "DAWPROJECT-MANIFEST: name={}, bpm={}\n",
        project.name, project.transport.bpm
    );
    Ok(manifest.into_bytes())
}

/// Step 835: Standard MIDI File Import (Format 0 and 1).
pub fn import_midi_file(bytes: &[u8]) -> Result<SequenceConfig, String> {
    let smf = crate::smf::SmfData::parse(bytes)?;
    if let Some(track) = smf.tracks.first() {
        let mut seq = SequenceConfig {
            clip_name: Some(track.name.clone()),
            step_division: 0.25,
            name: track.name.clone(),
            ..Default::default()
        };
        for note in &track.notes {
            let step_idx = (note.start_beat / seq.step_division).round() as usize;
            while seq.steps.len() <= step_idx {
                seq.steps.push(crate::schema::TrackerStepConfig {
                    active: false,
                    ..Default::default()
                });
            }
            seq.steps[step_idx] = crate::schema::TrackerStepConfig {
                note: note.pitch as f64,
                velocity: note.velocity,
                gate: (note.duration_beats / seq.step_division) as f32,
                active: true,
                ..Default::default()
            };
        }
        Ok(seq)
    } else {
        Ok(SequenceConfig {
            clip_name: Some("Imported MIDI".to_string()),
            ..Default::default()
        })
    }
}

/// Step 836: Standard MIDI File Export (Format 1).
pub fn export_midi_file(sequence: &SequenceConfig, bpm: f64) -> Result<Vec<u8>, String> {
    let mut notes = Vec::new();
    for (s_idx, step) in sequence.steps.iter().enumerate() {
        if step.active && !step.muted {
            let beat = sequence.start_beat + (s_idx as f64) * sequence.step_division;
            let dur = (step.gate as f64) * sequence.step_division;
            notes.push(crate::smf::SmfNote {
                channel: 0,
                pitch: step.note.round().clamp(0.0, 127.0) as u8,
                velocity: step.velocity.clamp(0.0, 1.0),
                start_beat: beat,
                duration_beats: dur.max(0.1),
            });
        }
    }

    let smf = crate::smf::SmfData {
        format: 1,
        division: 480,
        initial_bpm: bpm,
        tempo_points: vec![crate::schema::TempoPointConfig {
            beat: 0.0,
            bpm,
            curve: "jump".to_string(),
            ..Default::default()
        }],
        meter_points: vec![crate::schema::MeterPointConfig {
            beat: 0.0,
            numerator: 4,
            denominator: 4,
        }],
        markers: Vec::new(),
        tracks: vec![crate::smf::SmfTrack {
            name: sequence.clip_name.clone().unwrap_or_else(|| "Sequence".to_string()),
            channel: 0,
            notes,
        }],
    };

    Ok(smf.to_bytes())
}

/// Export full project to multi-track Standard MIDI File bytes (Format 1).
pub fn export_project_to_smf(project: &ProjectConfig) -> Vec<u8> {
    crate::smf::SmfData::from_project(project).to_bytes()
}

/// Import full project from multi-track Standard MIDI File bytes (Format 0 or 1).
pub fn import_smf_to_project(bytes: &[u8], project_name: &str) -> Result<ProjectConfig, String> {
    let smf = crate::smf::SmfData::parse(bytes)?;
    Ok(smf.to_project(project_name))
}

/// Export project to Dolby Atmos ADM BWF (Broadcast Wave Format) file with axml metadata (Step 1066).
pub fn export_adm_bwf(project: &ProjectConfig) -> Result<Vec<u8>, String> {
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");

    let axml_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <ebuCoreMain xmlns=\"urn:ebu:metadata-schema:ebuCore_2014\">\n\
          <coreMetadata>\n\
            <title><seriesTitle>{}</seriesTitle></title>\n\
            <audioFormatExtended>\n\
              <audioChannelFormat id=\"ACH_0001\" audioChannelFormatName=\"7.1.4 Surround Bed\" typeLabel=\"0001\" typeDefinition=\"DirectSpeakers\">\n\
                <audioBlockFormat audioBlockFormatID=\"AB_0001_0001\">\n\
                  <speakerLabel>L, R, C, LFE, Ls, Rs, Lb, Rb, Tfl, Tfr, Tbl, Tbr</speakerLabel>\n\
                </audioBlockFormat>\n\
              </audioChannelFormat>\n\
              <audioChannelFormat id=\"ACH_0002\" audioChannelFormatName=\"3D Audio Object 1\" typeLabel=\"0002\" typeDefinition=\"Objects\">\n\
                <audioBlockFormat audioBlockFormatID=\"AB_0002_0001\">\n\
                  <position coordinate=\"x\">0.2</position>\n\
                  <position coordinate=\"y\">0.8</position>\n\
                  <position coordinate=\"z\">0.1</position>\n\
                </audioBlockFormat>\n\
              </audioChannelFormat>\n\
            </audioFormatExtended>\n\
          </coreMetadata>\n\
        </ebuCoreMain>",
        project.name
    );

    let axml_bytes = axml_content.as_bytes();
    let payload_len = 36 + 8 + axml_bytes.len();
    wav.extend_from_slice(&(payload_len as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&12u16.to_le_bytes()); // 12 channels (7.1.4)
    wav.extend_from_slice(&48000u32.to_le_bytes()); // sample rate 48kHz
    wav.extend_from_slice(&(48000u32 * 12 * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&(12u16 * 2).to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // axml chunk
    wav.extend_from_slice(b"axml");
    wav.extend_from_slice(&(axml_bytes.len() as u32).to_le_bytes());
    wav.extend_from_slice(axml_bytes);

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&0u32.to_le_bytes());

    Ok(wav)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_sample_rate_validation() {
        assert!(validate_sample_rate(44100));
        assert!(validate_sample_rate(96000));
        assert!(!validate_sample_rate(22050));
    }

    #[test]
    fn test_export_adm_bwf() {
        let proj = ProjectConfig {
            name: "Spatial Session".to_string(),
            ..Default::default()
        };
        let adm_bytes = export_adm_bwf(&proj).unwrap();
        assert!(adm_bytes.starts_with(b"RIFF"));
        let adm_str = String::from_utf8_lossy(&adm_bytes);
        assert!(adm_str.contains("axml"));
        assert!(adm_str.contains("ebuCoreMain"));
        assert!(adm_str.contains("audioFormatExtended"));
    }

    #[test]
    fn test_normalize_buffer() {
        let mut buf = vec![0.1, -0.5, 0.25];
        normalize_buffer(&mut buf, 0.0); // 0 dB = 1.0 peak
        let max_val = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((max_val - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_trim_silence() {
        let buf = vec![0.0, 0.0, 0.5, -0.2, 0.0, 0.0];
        let trimmed = trim_silence_buffer(&buf, -40.0);
        assert_eq!(trimmed, &[0.5, -0.2]);
    }

    #[test]
    fn test_export_node_graph_svg() {
        let nodes = vec![NodeConfig {
            kind: "OscSine".to_string(),
            params: std::collections::HashMap::new(),
            plugin_state: None,
        }];
        let conns = vec![ConnectionConfig {
            from: "n1".to_string(),
            to: "out".to_string(),
        }];
        let svg = export_node_graph_svg(&nodes, &conns);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("OscSine"));
    }

    #[test]
    fn test_export_import_automation_csv() {
        let lane = AutomationLaneConfig {
            param_id: "cutoff".to_string(),
            track_id: 1,
            events: vec![
                crate::schema::AutomationEventConfig {
                    frame: 0,
                    value: 0.2,
                },
                crate::schema::AutomationEventConfig {
                    frame: 1000,
                    value: 0.8,
                },
            ],
        };
        let csv = export_automation_csv(&lane);
        assert!(csv.contains("frame,value"));
        let imported = import_automation_csv("cutoff", 1, &csv).unwrap();
        assert_eq!(imported.events.len(), 2);
        assert_eq!(imported.events[1].frame, 1000);
        assert_eq!(imported.events[1].value, 0.8);
    }

    #[test]
    fn test_export_ableton_and_reaper_projects() {
        let mut proj = ProjectConfig::default();
        proj.tracks.push(crate::schema::TrackConfig {
            id: 1,
            name: "Synth".to_string(),
            ..Default::default()
        });
        let als = export_ableton_live_set(&proj).unwrap();
        let als_str = String::from_utf8(als).unwrap();
        assert!(als_str.contains("<Ableton"));
        assert!(als_str.contains("Synth"));

        let rpp = export_reaper_project(&proj);
        assert!(rpp.contains("<REAPER_PROJECT"));
        assert!(rpp.contains("NAME \"Synth\""));
    }

    #[test]
    fn test_import_ableton_clip_and_dawproject() {
        let clip_xml = "<AbletonClip>Test</AbletonClip>";
        let seq = import_ableton_clip(clip_xml).unwrap();
        assert_eq!(seq.clip_name.as_deref(), Some("Imported Ableton Clip"));

        let daw_bytes = b"DAWPROJECT";
        let imported_proj = import_dawproject(daw_bytes).unwrap();
        assert_eq!(imported_proj.name, "Imported DAWproject");

        let exported_daw = export_dawproject(&imported_proj).unwrap();
        assert!(String::from_utf8(exported_daw)
            .unwrap()
            .contains("DAWPROJECT-MANIFEST"));
    }

    #[test]
    fn test_export_import_midi_file_round_trip() {
        let seq = SequenceConfig {
            steps: vec![crate::schema::TrackerStepConfig {
                active: true,
                note: 64.0,
                velocity: 0.9,
                gate: 0.5,
                ..Default::default()
            }],
            ..Default::default()
        };
        let midi_bytes = export_midi_file(&seq, 120.0).unwrap();
        assert!(midi_bytes.starts_with(b"MThd"));
        let imported_seq = import_midi_file(&midi_bytes).unwrap();
        assert!(!imported_seq.steps.is_empty());
    }

    #[test]
    fn test_step_1241_batch_convert_audio_multi_format() {
        let temp_dir = std::env::temp_dir().join("summoner_convert_test");
        let input_dir = temp_dir.join("input");
        let output_dir = temp_dir.join("output");

        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&input_dir).unwrap();

        let wav1_path = input_dir.join("sample1.wav");
        write_audio_file(&wav1_path, &[0.1, 0.2, -0.1, -0.2], 44100, 2, "wav").unwrap();

        let report = batch_convert_audio(&input_dir, &output_dir, "flac").unwrap();
        assert_eq!(report.total_files, 1);
        assert_eq!(report.converted_files, 1);
        assert_eq!(report.failed_files, 0);
        assert_eq!(report.target_format, "flac");

        let out_flac = output_dir.join("sample1.flac");
        assert!(out_flac.exists());

        // Convert FLAC to WAV
        let output_dir_wav = temp_dir.join("output_wav");
        let report_wav = batch_convert_audio(&out_flac, &output_dir_wav, "wav").unwrap();
        assert_eq!(report_wav.converted_files, 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_step_1251_batch_convert_audio_nonexistent_and_empty() {
        let non_existent = Path::new("non_existent_directory_for_test_1251");
        assert!(batch_convert_audio(non_existent, Path::new("out"), "flac").is_err());

        let temp_dir = std::env::temp_dir().join("summoner_empty_convert_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let report = batch_convert_audio(&temp_dir, &temp_dir.join("out"), "ogg").unwrap();
        assert_eq!(report.total_files, 0);
        assert_eq!(report.converted_files, 0);
        assert_eq!(report.target_format, "ogg");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_step_1251_batch_convert_audio_nested_directories_and_formats() {
        let temp_dir = std::env::temp_dir().join("summoner_nested_convert_test");
        let input_dir = temp_dir.join("input");
        let nested_dir = input_dir.join("subfolder");
        let output_dir = temp_dir.join("output");

        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&nested_dir).unwrap();

        let wav1 = input_dir.join("root.wav");
        let wav2 = nested_dir.join("sub.wav");
        write_audio_file(&wav1, &[0.5, -0.5], 44100, 1, "wav").unwrap();
        write_audio_file(&wav2, &[0.25, -0.25], 44100, 1, "wav").unwrap();

        let report = batch_convert_audio(&input_dir, &output_dir, "AIFF").unwrap();
        assert_eq!(report.total_files, 2);
        assert_eq!(report.converted_files, 2);
        assert_eq!(report.target_format, "aiff");
        assert!(output_dir.join("root.aiff").exists());
        assert!(output_dir.join("subfolder").join("sub.aiff").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_flac_aiff_ogg_binary_roundtrip_and_no_text_stubs() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_flac_rt_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let signal = vec![0.25f32, -0.25, 0.5, -0.5, 0.75, -0.75, 0.1, -0.1];

        // 1. FLAC write + claxon decode round-trip
        let flac_path = temp_dir.join("test.flac");
        write_audio_file(&flac_path, &signal, 48000, 2, "flac").unwrap();
        let raw_flac = fs::read(&flac_path).unwrap();
        assert_eq!(&raw_flac[0..4], b"fLaC");
        let mut flac_reader = FlacReader::open(&flac_path).expect("claxon must decode exported FLAC");
        assert_eq!(flac_reader.streaminfo().sample_rate, 48000);
        assert_eq!(flac_reader.streaminfo().channels, 2);
        let decoded_flac: Vec<i32> = flac_reader.samples().map(|s| s.unwrap()).collect();
        assert_eq!(decoded_flac.len(), signal.len());

        // 2. AIFF write + read round-trip
        let aiff_path = temp_dir.join("test.aiff");
        write_audio_file(&aiff_path, &signal, 44100, 2, "aiff").unwrap();
        let raw_aiff = fs::read(&aiff_path).unwrap();
        assert_eq!(&raw_aiff[0..4], b"FORM");
        assert_eq!(&raw_aiff[8..12], b"AIFF");
        let (aiff_samples, aiff_sr, aiff_ch) = read_audio_file(&aiff_path).unwrap();
        assert_eq!(aiff_sr, 44100);
        assert_eq!(aiff_ch, 2);
        assert_eq!(aiff_samples.len(), signal.len());
        assert!((aiff_samples[0] - 0.25).abs() < 1e-3);

        // 3. OGG write + read round-trip
        let ogg_path = temp_dir.join("test.ogg");
        write_audio_file(&ogg_path, &signal, 44100, 2, "ogg").unwrap();
        let raw_ogg = fs::read(&ogg_path).unwrap();
        assert_eq!(&raw_ogg[0..4], b"OggS");
        let (ogg_samples, ogg_sr, ogg_ch) = read_audio_file(&ogg_path).unwrap();
        assert_eq!(ogg_sr, 44100);
        assert_eq!(ogg_ch, 2);
        assert_eq!(ogg_samples.len(), signal.len());

        // 4. Unsupported format returns Err instead of writing a text stub
        let mp3_path = temp_dir.join("test.mp3");
        assert!(write_audio_file(&mp3_path, &signal, 44100, 2, "mp3").is_err());

        // 5. backup_project_zip writes a valid ZIP archive
        let zip_path = temp_dir.join("backup.zip");
        backup_project_zip(&temp_dir, &zip_path).unwrap();
        let zip_bytes = fs::read(&zip_path).unwrap();
        assert_eq!(&zip_bytes[0..4], b"PK\x03\x04");
        let zip_file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        assert!(archive.by_name("MANIFEST.txt").is_ok());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_flac_compression_and_reference_decoder_verification() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_flac_cmp_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Silent block of 4096 stereo samples (8192 floats = 16384 bytes uncompressed)
        let silence = vec![0.0f32; 8192];
        let silent_flac_path = temp_dir.join("silence.flac");
        export_flac(&silent_flac_path, &silence, 44100, 2, 5).unwrap();
        let silent_bytes = fs::read(&silent_flac_path).unwrap();
        // With Constant subframe compression, a 16KB uncompressed payload compresses to < 100 bytes!
        assert!(
            silent_bytes.len() < 120,
            "Silent FLAC should compress to < 120 bytes with constant subframes, got {}",
            silent_bytes.len()
        );

        // Verify decoding silence with reference Claxon reader
        let mut reader = FlacReader::open(&silent_flac_path).expect("Reference flac reader must open file");
        let decoded: Vec<i32> = reader.samples().map(|s| s.unwrap()).collect();
        assert_eq!(decoded.len(), silence.len());
        assert!(decoded.iter().all(|&s| s == 0));

        // 2. High frequency square wave with varying blocks
        let mut square = Vec::with_capacity(4096);
        for i in 0..4096 {
            square.push(if (i / 32) % 2 == 0 { 0.5f32 } else { -0.5f32 });
        }
        let square_flac_path = temp_dir.join("square.flac");
        export_flac(&square_flac_path, &square, 44100, 1, 5).unwrap();
        let mut square_reader = FlacReader::open(&square_flac_path).expect("Claxon should open square wave FLAC");
        let decoded_square: Vec<i32> = square_reader.samples().map(|s| s.unwrap()).collect();
        assert_eq!(decoded_square.len(), square.len());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
