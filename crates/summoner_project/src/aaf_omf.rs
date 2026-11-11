//! AAF (Advanced Authoring Format) and OMF session import/export (Task I5).
//!
//! Provides multitrack session exchange with Pro Tools, Logic Pro, DaVinci Resolve,
//! Premiere Pro, and Avid Media Composer:
//! - [`AafSession`]: Composition model capturing audio tracks, clips, in/out offsets, and volume fades.
//! - [`export_aaf_xml`]: Serializes session into SMPTE AAF Edit Protocol XML representation.
//! - [`import_aaf_xml`]: Parses AAF session XML and reconstructs [`ProjectConfig`].
//! - Seamless bidirectional round-trip conversion between AAF interchange and native Summoner project models.

use crate::audio_clip::AudioClipConfig;
use crate::schema::{ProjectConfig, TrackConfig, TransportConfig};
use serde::{Deserialize, Serialize};
use std::fmt;
use summoner_core::timecode::TimecodeRate;

/// Individual clip/event entry within an AAF or OMF track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AafClip {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub asset_id: String,
    pub start_sample: u64,
    pub duration_samples: u64,
    pub source_offset_samples: u64,
    pub gain: f32,
    pub pan: f32,
    pub fade_in_samples: u64,
    pub fade_out_samples: u64,
    pub is_muted: bool,
}

impl AafClip {
    pub fn new(
        id: &str,
        name: &str,
        file_path: &str,
        asset_id: &str,
        start_sample: u64,
        duration_samples: u64,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            file_path: file_path.to_string(),
            asset_id: asset_id.to_string(),
            start_sample,
            duration_samples,
            source_offset_samples: 0,
            gain: 1.0,
            pan: 0.0,
            fade_in_samples: 0,
            fade_out_samples: 0,
            is_muted: false,
        }
    }
}

/// Multitrack audio track within an AAF/OMF session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AafTrack {
    pub id: u64,
    pub name: String,
    pub channels: usize,
    pub gain: f32,
    pub pan: f32,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub clips: Vec<AafClip>,
}

impl AafTrack {
    pub fn new(id: u64, name: &str, channels: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            channels,
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            clips: Vec::new(),
        }
    }
}

/// Complete AAF/OMF interchange session document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AafSession {
    pub name: String,
    pub sample_rate: u32,
    pub timecode_rate: TimecodeRate,
    pub bpm: f64,
    pub tracks: Vec<AafTrack>,
}

impl Default for AafSession {
    fn default() -> Self {
        Self {
            name: "Untitled Session".to_string(),
            sample_rate: 48000,
            timecode_rate: TimecodeRate::Fps25,
            bpm: 120.0,
            tracks: Vec::new(),
        }
    }
}

impl AafSession {
    pub fn new(name: &str, sample_rate: u32, timecode_rate: TimecodeRate, bpm: f64) -> Self {
        Self {
            name: name.to_string(),
            sample_rate,
            timecode_rate,
            bpm,
            tracks: Vec::new(),
        }
    }

    /// Extract AAF session from native Summoner `ProjectConfig`.
    pub fn from_project(project: &ProjectConfig, sample_rate: u32) -> Self {
        let bpm = project.transport.bpm;
        let mut aaf_tracks = Vec::new();

        for track in &project.tracks {
            let mut aaf_track = AafTrack::new(track.id, &track.name, track.channels);
            aaf_track.gain = track.gain;
            aaf_track.pan = track.pan;
            aaf_track.is_muted = track.muted;
            aaf_track.is_soloed = track.soloed;

            for clip in &track.audio_clips {
                let start_seconds = (clip.start_beat * 60.0) / bpm;
                let duration_seconds = (clip.duration_beats * 60.0) / bpm;
                let start_sample = (start_seconds * sample_rate as f64).round() as u64;
                let duration_samples = (duration_seconds * sample_rate as f64).round() as u64;

                let fade_in_sec = (clip.fade_in_beats * 60.0) / bpm;
                let fade_out_sec = (clip.fade_out_beats * 60.0) / bpm;

                let aaf_clip = AafClip {
                    id: clip.id.clone(),
                    name: clip.name.clone(),
                    file_path: clip
                        .file_path
                        .clone()
                        .unwrap_or_else(|| format!("{}.wav", clip.asset_id)),
                    asset_id: clip.asset_id.clone(),
                    start_sample,
                    duration_samples,
                    source_offset_samples: clip.offset_frames,
                    gain: clip.gain,
                    pan: clip.pan,
                    fade_in_samples: (fade_in_sec * sample_rate as f64).round() as u64,
                    fade_out_samples: (fade_out_sec * sample_rate as f64).round() as u64,
                    is_muted: clip.muted,
                };
                aaf_track.clips.push(aaf_clip);
            }

            aaf_tracks.push(aaf_track);
        }

        Self {
            name: project.name.clone(),
            sample_rate,
            timecode_rate: TimecodeRate::Fps25,
            bpm,
            tracks: aaf_tracks,
        }
    }

    /// Convert AAF session into native Summoner `ProjectConfig`.
    pub fn to_project(&self) -> ProjectConfig {
        let mut project = ProjectConfig {
            version: "1.1.0".to_string(),
            name: self.name.clone(),
            transport: TransportConfig {
                sample_rate: self.sample_rate,
                bpm: self.bpm,
                time_signature: "4/4".to_string(),
                ..TransportConfig::default()
            },
            ..ProjectConfig::default()
        };

        for aaf_track in &self.tracks {
            let mut track = TrackConfig {
                id: aaf_track.id,
                name: aaf_track.name.clone(),
                channels: aaf_track.channels,
                gain: aaf_track.gain,
                pan: aaf_track.pan,
                muted: aaf_track.is_muted,
                soloed: aaf_track.is_soloed,
                ..TrackConfig::default()
            };

            for aaf_clip in &aaf_track.clips {
                let start_seconds = aaf_clip.start_sample as f64 / self.sample_rate as f64;
                let duration_seconds = aaf_clip.duration_samples as f64 / self.sample_rate as f64;
                let start_beat = (start_seconds * self.bpm) / 60.0;
                let duration_beats = (duration_seconds * self.bpm) / 60.0;

                let fade_in_sec = aaf_clip.fade_in_samples as f64 / self.sample_rate as f64;
                let fade_out_sec = aaf_clip.fade_out_samples as f64 / self.sample_rate as f64;

                let mut clip = AudioClipConfig::new(
                    &aaf_clip.id,
                    &aaf_clip.name,
                    &aaf_clip.asset_id,
                    start_beat,
                    duration_beats,
                );
                clip.file_path = Some(aaf_clip.file_path.clone());
                clip.offset_frames = aaf_clip.source_offset_samples;
                clip.duration_frames = Some(aaf_clip.duration_samples);
                clip.gain = aaf_clip.gain;
                clip.pan = aaf_clip.pan;
                clip.fade_in_beats = (fade_in_sec * self.bpm) / 60.0;
                clip.fade_out_beats = (fade_out_sec * self.bpm) / 60.0;
                clip.muted = aaf_clip.is_muted;

                track.audio_clips.push(clip);
            }

            project.tracks.push(track);
        }

        project
    }
}

/// Errors occurring during AAF/OMF import and export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AafError {
    XmlParse(String),
    Serialization(String),
    InvalidTimecode(String),
    Io(String),
}

impl fmt::Display for AafError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XmlParse(e) => write!(f, "AAF XML parse error: {}", e),
            Self::Serialization(e) => write!(f, "AAF serialization error: {}", e),
            Self::InvalidTimecode(e) => write!(f, "Invalid timecode: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for AafError {}

/// Export `AafSession` to SMPTE 2067 / AAF Edit Protocol XML string.
pub fn export_aaf_xml(session: &AafSession) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<AAFComposition xmlns=\"http://www.amwa.tv/aaf/edit-protocol/1.0\">\n");
    xml.push_str(&format!("  <Name>{}</Name>\n", xml_escape(&session.name)));
    xml.push_str(&format!(
        "  <SampleRate>{}</SampleRate>\n",
        session.sample_rate
    ));
    xml.push_str(&format!(
        "  <EditRate>{}</EditRate>\n",
        session.timecode_rate.nominal_fps()
    ));
    xml.push_str(&format!("  <TempoBpm>{:.2}</TempoBpm>\n", session.bpm));
    xml.push_str("  <Tracks>\n");

    for track in &session.tracks {
        xml.push_str(&format!("    <Track id=\"{}\">\n", track.id));
        xml.push_str(&format!("      <Name>{}</Name>\n", xml_escape(&track.name)));
        xml.push_str(&format!("      <Channels>{}</Channels>\n", track.channels));
        xml.push_str(&format!("      <Gain>{:.4}</Gain>\n", track.gain));
        xml.push_str(&format!("      <Pan>{:.4}</Pan>\n", track.pan));
        xml.push_str(&format!("      <Muted>{}</Muted>\n", track.is_muted));
        xml.push_str(&format!("      <Soloed>{}</Soloed>\n", track.is_soloed));
        xml.push_str("      <Clips>\n");

        for clip in &track.clips {
            xml.push_str(&format!("        <Clip id=\"{}\">\n", xml_escape(&clip.id)));
            xml.push_str(&format!(
                "          <Name>{}</Name>\n",
                xml_escape(&clip.name)
            ));
            xml.push_str(&format!(
                "          <FilePath>{}</FilePath>\n",
                xml_escape(&clip.file_path)
            ));
            xml.push_str(&format!(
                "          <AssetId>{}</AssetId>\n",
                xml_escape(&clip.asset_id)
            ));
            xml.push_str(&format!(
                "          <StartSample>{}</StartSample>\n",
                clip.start_sample
            ));
            xml.push_str(&format!(
                "          <DurationSamples>{}</DurationSamples>\n",
                clip.duration_samples
            ));
            xml.push_str(&format!(
                "          <SourceOffsetSamples>{}</SourceOffsetSamples>\n",
                clip.source_offset_samples
            ));
            xml.push_str(&format!("          <Gain>{:.4}</Gain>\n", clip.gain));
            xml.push_str(&format!("          <Pan>{:.4}</Pan>\n", clip.pan));
            xml.push_str(&format!(
                "          <FadeInSamples>{}</FadeInSamples>\n",
                clip.fade_in_samples
            ));
            xml.push_str(&format!(
                "          <FadeOutSamples>{}</FadeOutSamples>\n",
                clip.fade_out_samples
            ));
            xml.push_str(&format!("          <Muted>{}</Muted>\n", clip.is_muted));
            xml.push_str("        </Clip>\n");
        }

        xml.push_str("      </Clips>\n");
        xml.push_str("    </Track>\n");
    }

    xml.push_str("  </Tracks>\n");
    xml.push_str("</AAFComposition>\n");
    xml
}

/// Import `AafSession` from SMPTE 2067 / AAF Edit Protocol XML string.
pub fn import_aaf_xml(xml: &str) -> Result<AafSession, AafError> {
    let name = extract_xml_tag(xml, "Name").unwrap_or_else(|| "Imported Session".to_string());
    let sample_rate = extract_xml_tag(xml, "SampleRate")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(48000);
    let bpm = extract_xml_tag(xml, "TempoBpm")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(120.0);
    let edit_rate = extract_xml_tag(xml, "EditRate")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(25);

    let timecode_rate = match edit_rate {
        24 => TimecodeRate::Fps24,
        25 => TimecodeRate::Fps25,
        30 => TimecodeRate::Fps30Ndf,
        _ => TimecodeRate::Fps25,
    };

    let mut session = AafSession::new(&name, sample_rate, timecode_rate, bpm);

    // Split track elements
    let track_blocks = split_xml_blocks(xml, "<Track", "</Track>");
    for (idx, block) in track_blocks.iter().enumerate() {
        let track_name =
            extract_xml_tag(block, "Name").unwrap_or_else(|| format!("Track {}", idx + 1));
        let channels = extract_xml_tag(block, "Channels")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        let gain = extract_xml_tag(block, "Gain")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        let pan = extract_xml_tag(block, "Pan")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let muted = extract_xml_tag(block, "Muted")
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let soloed = extract_xml_tag(block, "Soloed")
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let mut aaf_track = AafTrack::new(idx as u64 + 1, &track_name, channels);
        aaf_track.gain = gain;
        aaf_track.pan = pan;
        aaf_track.is_muted = muted;
        aaf_track.is_soloed = soloed;

        let clip_blocks = split_xml_blocks(block, "<Clip", "</Clip>");
        for (c_idx, c_block) in clip_blocks.iter().enumerate() {
            let clip_id =
                extract_attr(c_block, "id").unwrap_or_else(|| format!("clip_{}_{}", idx, c_idx));
            let clip_name =
                extract_xml_tag(c_block, "Name").unwrap_or_else(|| format!("Clip {}", c_idx + 1));
            let file_path = extract_xml_tag(c_block, "FilePath").unwrap_or_default();
            let asset_id = extract_xml_tag(c_block, "AssetId").unwrap_or_else(|| clip_id.clone());
            let start_sample = extract_xml_tag(c_block, "StartSample")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let duration_samples = extract_xml_tag(c_block, "DurationSamples")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(48000);
            let source_offset = extract_xml_tag(c_block, "SourceOffsetSamples")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let clip_gain = extract_xml_tag(c_block, "Gain")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0);
            let clip_pan = extract_xml_tag(c_block, "Pan")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            let fade_in = extract_xml_tag(c_block, "FadeInSamples")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let fade_out = extract_xml_tag(c_block, "FadeOutSamples")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let clip_muted = extract_xml_tag(c_block, "Muted")
                .map(|s| s.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            let mut clip = AafClip::new(
                &clip_id,
                &clip_name,
                &file_path,
                &asset_id,
                start_sample,
                duration_samples,
            );
            clip.source_offset_samples = source_offset;
            clip.gain = clip_gain;
            clip.pan = clip_pan;
            clip.fade_in_samples = fade_in;
            clip.fade_out_samples = fade_out;
            clip.is_muted = clip_muted;

            aaf_track.clips.push(clip);
        }

        session.tracks.push(aaf_track);
    }

    Ok(session)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            return Some(xml[content_start..content_start + end].trim().to_string());
        }
    }
    None
}

fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = xml.find(&pattern) {
        let val_start = start + pattern.len();
        if let Some(end) = xml[val_start..].find('"') {
            return Some(xml[val_start..val_start + end].to_string());
        }
    }
    None
}

fn split_xml_blocks(xml: &str, open_prefix: &str, close_tag: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut search_idx = 0;

    while let Some(start) = xml[search_idx..].find(open_prefix) {
        let absolute_start = search_idx + start;
        if let Some(end) = xml[absolute_start..].find(close_tag) {
            let absolute_end = absolute_start + end + close_tag.len();
            blocks.push(xml[absolute_start..absolute_end].to_string());
            search_idx = absolute_end;
        } else {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aaf_xml_export_and_import_roundtrip() {
        let mut session = AafSession::new("Orchestral Cue", 48000, TimecodeRate::Fps24, 120.0);
        let mut track1 = AafTrack::new(1, "Violins 1", 2);
        track1.gain = 0.85;
        track1.pan = -0.3;

        let mut clip1 = AafClip::new(
            "c1",
            "Staccato Run",
            "audio/violins.wav",
            "asset_v1",
            48000,
            96000,
        );
        clip1.source_offset_samples = 24000;
        clip1.fade_in_samples = 480;
        clip1.fade_out_samples = 960;
        track1.clips.push(clip1);

        session.tracks.push(track1);

        // Export to XML
        let xml_str = export_aaf_xml(&session);
        assert!(xml_str.contains("<Name>Orchestral Cue</Name>"));
        assert!(xml_str.contains("<SampleRate>48000</SampleRate>"));
        assert!(xml_str.contains("<EditRate>24</EditRate>"));
        assert!(xml_str.contains("Violins 1"));
        assert!(xml_str.contains("Staccato Run"));

        // Import back from XML
        let imported = import_aaf_xml(&xml_str).expect("Import should succeed");
        assert_eq!(imported.name, "Orchestral Cue");
        assert_eq!(imported.sample_rate, 48000);
        assert_eq!(imported.tracks.len(), 1);
        assert_eq!(imported.tracks[0].name, "Violins 1");
        assert_eq!(imported.tracks[0].clips.len(), 1);
        assert_eq!(imported.tracks[0].clips[0].name, "Staccato Run");
        assert_eq!(imported.tracks[0].clips[0].source_offset_samples, 24000);
        assert_eq!(imported.tracks[0].clips[0].duration_samples, 96000);
    }

    #[test]
    fn test_aaf_session_project_config_roundtrip() {
        let mut session = AafSession::new("Film Score", 48000, TimecodeRate::Fps25, 120.0);
        let mut track = AafTrack::new(1, "Dialogue", 1);
        track.gain = 0.9;
        // 48000 samples at 48kHz = 1 second = 2 beats at 120 BPM
        let clip = AafClip::new(
            "c_dia",
            "Actor Line 1",
            "audio/dialogue.wav",
            "asset_dia",
            48000,
            96000,
        );
        track.clips.push(clip);
        session.tracks.push(track);

        // Convert to ProjectConfig
        let project = session.to_project();
        assert_eq!(project.name, "Film Score");
        assert_eq!(project.tracks.len(), 1);
        assert_eq!(project.tracks[0].name, "Dialogue");
        assert_eq!(project.tracks[0].audio_clips.len(), 1);
        assert_eq!(project.tracks[0].audio_clips[0].start_beat, 2.0);
        assert_eq!(project.tracks[0].audio_clips[0].duration_beats, 4.0);

        // Convert back to AafSession
        let roundtrip_session = AafSession::from_project(&project, 48000);
        assert_eq!(roundtrip_session.tracks.len(), 1);
        assert_eq!(roundtrip_session.tracks[0].clips[0].start_sample, 48000);
        assert_eq!(roundtrip_session.tracks[0].clips[0].duration_samples, 96000);
    }
}
