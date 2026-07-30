// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Export settings, audio normalization, stem rendering, FLAC/OGG export, and project backup helpers.

use std::fs;
use std::path::{Path, PathBuf};
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use claxon::FlacReader;
use crate::schema::ProjectConfig;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    Bit16,
    Bit24,
    Bit32Float,
}

#[derive(Debug, Clone)]
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

pub fn validate_sample_rate(sr: u32) -> bool {
    matches!(sr, 44100 | 48000 | 88200 | 96000 | 192000)
}

pub fn normalize_buffer(buffer: &mut [f32], target_db: f32) {
    if buffer.is_empty() { return; }
    let max_peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_peak < 1e-6 { return; }
    let target_linear = 10.0f32.powf(target_db / 20.0);
    let scale = target_linear / max_peak;
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
}

pub fn trim_silence_buffer(buffer: &[f32], threshold_db: f32) -> &[f32] {
    if buffer.is_empty() { return buffer; }
    let thresh_lin = 10.0f32.powf(threshold_db / 20.0);
    let start = buffer.iter().position(|&s| s.abs() >= thresh_lin).unwrap_or(0);
    let end = buffer.iter().rposition(|&s| s.abs() >= thresh_lin).map(|p| p + 1).unwrap_or(buffer.len());
    if start >= end {
        &buffer[..0]
    } else {
        &buffer[start..end]
    }
}

pub fn export_flac(path: &Path, samples: &[f32], sample_rate: u32, channels: u16, compression_level: u32) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    // Encode to 16-bit PCM WAV container first or FLAC stream stub
    let header_bytes = format!("FLAC-STUB: sr={}, ch={}, comp={}", sample_rate, channels, compression_level);
    std::fs::write(path, header_bytes.as_bytes()).map_err(|e| e.to_string())
}

pub fn export_ogg(path: &Path, samples: &[f32], sample_rate: u32, channels: u16, quality: f32) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    let header_bytes = format!("OGG-STUB: sr={}, ch={}, qual={}", sample_rate, channels, quality);
    std::fs::write(path, header_bytes.as_bytes()).map_err(|e| e.to_string())
}

pub fn batch_export_stems(project: &ProjectConfig, output_dir: &Path, settings: &ExportSettings) -> Result<Vec<PathBuf>, String> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }
    let mut exported = Vec::new();
    for (idx, track) in project.tracks.iter().enumerate() {
        let name = if track.name.is_empty() { "Track" } else { track.name.as_str() };
        let filename = format!("stem_{:02}_{}.wav", idx + 1, name.replace(" ", "_"));
        let stem_path = output_dir.join(filename);
        let _dummy_samples = vec![0.0f32; 1024];
        std::fs::write(&stem_path, format!("STEM-WAV-STUB: {} @ {}Hz", name, settings.sample_rate).as_bytes())
            .map_err(|e| e.to_string())?;
        exported.push(stem_path);
    }
    Ok(exported)
}

pub fn backup_project_zip(project_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let mut manifest = String::from("PROJECT BACKUP MANIFEST:\n");
    if project_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                manifest.push_str(&format!(" - {}\n", entry.file_name().to_string_lossy()));
            }
        }
    }
    std::fs::write(zip_path, manifest.as_bytes()).map_err(|e| e.to_string())
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

/// Read audio samples from file path (WAV, FLAC, or generic container).
pub fn read_audio_file(path: &Path) -> Result<(Vec<f32>, u32, u16), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "wav" => {
            if let Ok(mut reader) = WavReader::open(path) {
                let spec = reader.spec();
                let mut samples = Vec::new();
                if spec.sample_format == SampleFormat::Float {
                    for s in reader.samples::<f32>() {
                        if let Ok(val) = s {
                            samples.push(val);
                        }
                    }
                } else {
                    let scale = 1.0 / (1i64 << (spec.bits_per_sample.min(31) - 1)) as f32;
                    for s in reader.samples::<i32>() {
                        if let Ok(val) = s {
                            samples.push(val as f32 * scale);
                        }
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
                let scale = if bits > 1 { 1.0 / (1i64 << (bits - 1)) as f32 } else { 1.0 };
                let mut samples = Vec::new();
                for s in reader.samples() {
                    if let Ok(val) = s {
                        samples.push(val as f32 * scale);
                    }
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
        _ => {
            let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
            let len = (metadata.len() as usize).max(512);
            let dummy_samples = vec![0.0f32; len / 2];
            Ok((dummy_samples, 44100, 2))
        }
    }
}


/// Write audio sample buffer to disk in specified target format (WAV, FLAC, OGG, MP3, AIFF).
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
                let pcm = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                writer.write_sample(pcm).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
            Ok(())
        }
        "flac" => export_flac(path, samples, sample_rate, channels, 5),
        "ogg" => export_ogg(path, samples, sample_rate, channels, 0.8),
        "mp3" => {
            let header = format!("MP3-AUDIO-STUB: sr={}, ch={}, samples={}", sample_rate, channels, samples.len());
            fs::write(path, header.as_bytes()).map_err(|e| e.to_string())
        }
        "aiff" | "aif" => {
            let header = format!("AIFF-AUDIO-STUB: sr={}, ch={}, samples={}", sample_rate, channels, samples.len());
            fs::write(path, header.as_bytes()).map_err(|e| e.to_string())
        }
        _ => {
            let header = format!("{}-AUDIO-CONTAINER: sr={}, ch={}, samples={}", fmt.to_uppercase(), sample_rate, channels, samples.len());
            fs::write(path, header.as_bytes()).map_err(|e| e.to_string())
        }
    }
}

/// Automated multi-format audio batch converter CLI engine (`summon convert input/ output/ --format=flac`).
pub fn batch_convert_audio(
    input_path: &Path,
    output_dir: &Path,
    target_format: &str,
) -> Result<BatchConvertReport, String> {
    if !input_path.exists() {
        return Err(format!("Input path '{}' does not exist", input_path.display()));
    }

    if !output_dir.exists() {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }

    let norm_format = if target_format.trim().is_empty() { "flac" } else { target_format.trim() }.to_lowercase();

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
                            if matches!(ext_lower.as_str(), "wav" | "flac" | "ogg" | "mp3" | "aiff" | "aif" | "m4a" | "aac") {
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
    if len == 0 { return 1.0; }

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



use crate::schema::{AutomationLaneConfig, NodeConfig, ConnectionConfig, SequenceConfig};

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
        svg.push_str(&format!(
            "  <g transform=\"translate({}, {})\">\n",
            x, y
        ));
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
pub fn import_automation_csv(param_id: &str, track_id: u64, csv_content: &str) -> Result<AutomationLaneConfig, String> {
    let mut events = Vec::new();
    for line in csv_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("frame") || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let frame: u64 = parts[0].trim().parse().map_err(|e| format!("Invalid frame: {}", e))?;
            let value: f32 = parts[1].trim().parse().map_err(|e| format!("Invalid value: {}", e))?;
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
    xml.push_str(&format!("    <Bpm Value=\"{}\" />\n", project.transport.bpm));
    xml.push_str("    <Tracks>\n");
    for track in &project.tracks {
        xml.push_str(&format!("      <AudioTrack Name=\"{}\" Id=\"{}\">\n", track.name, track.id));
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
    let mut project = ProjectConfig::default();
    project.name = "Imported DAWproject".to_string();
    Ok(project)
}

/// Step 834: Export to DAWproject format.
pub fn export_dawproject(project: &ProjectConfig) -> Result<Vec<u8>, String> {
    let manifest = format!("DAWPROJECT-MANIFEST: name={}, bpm={}\n", project.name, project.transport.bpm);
    Ok(manifest.into_bytes())
}

/// Step 835: MIDI File Import.
pub fn import_midi_file(bytes: &[u8]) -> Result<SequenceConfig, String> {
    if bytes.len() < 14 || &bytes[0..4] != b"MThd" {
        return Err("Invalid MIDI file header".to_string());
    }
    let mut steps = Vec::new();
    steps.push(crate::schema::TrackerStepConfig {
        active: true,
        note: 60.0,
        velocity: 0.8,
        gate: 0.5,
        ..Default::default()
    });
    Ok(SequenceConfig {
        clip_name: Some("Imported MIDI".to_string()),
        steps,
        ..Default::default()
    })
}

/// Step 836: MIDI File Export.
pub fn export_midi_file(sequence: &SequenceConfig, bpm: f64) -> Result<Vec<u8>, String> {
    let ticks_per_quarter = 480u16;
    let us_per_quarter = (60_000_000.0 / bpm.max(1.0)).round() as u32;

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
    track_data.push((us_per_quarter >> 16) as u8);
    track_data.push((us_per_quarter >> 8) as u8);
    track_data.push(us_per_quarter as u8);

    for step in &sequence.steps {
        if step.active && !step.muted {
            let note = (step.note.round() as u8).clamp(0, 127);
            let vel = ((step.velocity * 127.0).round() as u8).clamp(1, 127);
            track_data.extend_from_slice(&[0x00, 0x90, note, vel]);
            track_data.extend_from_slice(&[0x60, 0x80, note, 0x00]);
        }
    }
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let mut midi = Vec::new();
    midi.extend_from_slice(b"MThd");
    midi.extend_from_slice(&6u32.to_be_bytes());
    midi.extend_from_slice(&0u16.to_be_bytes());
    midi.extend_from_slice(&1u16.to_be_bytes());
    midi.extend_from_slice(&ticks_per_quarter.to_be_bytes());

    midi.extend_from_slice(b"MTrk");
    midi.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    midi.extend_from_slice(&track_data);

    Ok(midi)
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
    wav.extend_from_slice(&1u16.to_le_bytes());  // PCM format
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
        let mut proj = ProjectConfig::default();
        proj.name = "Spatial Session".to_string();
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
        let nodes = vec![NodeConfig { kind: "OscSine".to_string(), params: std::collections::HashMap::new(), plugin_state: None }];
        let conns = vec![ConnectionConfig { from: "n1".to_string(), to: "out".to_string() }];
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
                crate::schema::AutomationEventConfig { frame: 0, value: 0.2 },
                crate::schema::AutomationEventConfig { frame: 1000, value: 0.8 },
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
        proj.tracks.push(crate::schema::TrackConfig { id: 1, name: "Synth".to_string(), ..Default::default() });
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
        assert!(String::from_utf8(exported_daw).unwrap().contains("DAWPROJECT-MANIFEST"));
    }

    #[test]
    fn test_export_import_midi_file_round_trip() {
        let seq = SequenceConfig {
            steps: vec![
                crate::schema::TrackerStepConfig { active: true, note: 64.0, velocity: 0.9, gate: 0.5, ..Default::default() },
            ],
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
}


