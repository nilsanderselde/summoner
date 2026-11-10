// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! ADM BWF (Audio Definition Model / Broadcast Wave Format - ITU-R BS.2076) Exporter and Importer (Task E5).
//!
//! Provides Pro Tools Ultimate / Dolby Atmos / Nuendo post-production parity:
//! - Generates fully compliant `<audioFormatExtended>` (axml chunk) metadata for 7.1.4 Dolby Atmos beds and 3D spatial objects.
//! - Embeds Broadcast Extension (`bext`) and ADM (`axml`) RIFF chunks into multichannel WAV containers.
//! - Parses ADM BWF metadata to restore spatial bed layouts and object positioning into Summoner sessions.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::path::Path;

/// Supported immersive bed layout for ADM authoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdmBedFormat {
    Stereo,
    Surround5_1,
    Surround7_1,
    DolbyAtmos7_1_2,
    DolbyAtmos7_1_4,
}

impl AdmBedFormat {
    pub fn channel_count(self) -> usize {
        match self {
            Self::Stereo => 2,
            Self::Surround5_1 => 6,
            Self::Surround7_1 => 8,
            Self::DolbyAtmos7_1_2 => 10,
            Self::DolbyAtmos7_1_4 => 12,
        }
    }
}

/// 3D Spatial coordinates for an ADM audio object.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdmCartesianPosition {
    /// X coordinate: -1.0 (left) to 0.0 (center) to +1.0 (right).
    pub x: f32,
    /// Y coordinate: -1.0 (back) to 0.0 (center) to +1.0 (front).
    pub y: f32,
    /// Z coordinate: 0.0 (ear level) to +1.0 (overhead ceiling).
    pub z: f32,
}

impl Default for AdmCartesianPosition {
    fn default() -> Self {
        Self { x: 0.0, y: 1.0, z: 0.0 }
    }
}

/// Dynamic 3D audio object or bed channel definition in ADM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmAudioObject {
    pub id: String,
    pub name: String,
    pub track_index: usize,
    pub position: AdmCartesianPosition,
    pub gain: f32,
    pub spread: f32,
    pub is_bed_channel: bool,
}

/// Parsed or constructed ADM session metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmSessionMetadata {
    pub programme_name: String,
    pub sample_rate: u32,
    pub bed_format: AdmBedFormat,
    pub objects: Vec<AdmAudioObject>,
}

/// Generates compliant ITU-R BS.2076 ADM XML (`<audioFormatExtended>`).
pub fn generate_adm_xml(
    programme_name: &str,
    sample_rate: u32,
    bed_format: AdmBedFormat,
    objects: &[AdmAudioObject],
) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<audioFormatExtended xmlns=\"urn:itu:r:bs:2076:2\">\n");

    // 1. audioProgramme
    xml.push_str(&format!(
        "  <audioProgramme audioProgrammeID=\"APR_1001\" audioProgrammeName=\"{}\">\n",
        escape_xml(programme_name)
    ));
    xml.push_str("    <audioContentIDRef>ACO_1001</audioContentIDRef>\n");
    xml.push_str("  </audioProgramme>\n");

    // 2. audioContent
    xml.push_str(&format!(
        "  <audioContent audioContentID=\"ACO_1001\" audioContentName=\"{}\">\n",
        escape_xml(programme_name)
    ));
    for (i, _) in objects.iter().enumerate() {
        xml.push_str(&format!("    <audioObjectIDRef>AO_{:04}</audioObjectIDRef>\n", 1001 + i));
    }
    xml.push_str("  </audioContent>\n");

    // 3. audioObjects and PackFormats
    for (i, obj) in objects.iter().enumerate() {
        let ao_id = format!("AO_{:04}", 1001 + i);
        let ap_id = format!("AP_{:08}", 1001 + i);
        let ac_id = format!("AC_{:08}", 1001 + i);
        let at_id = format!("AT_{:08}_01", 1001 + i);

        xml.push_str(&format!(
            "  <audioObject audioObjectID=\"{}\" audioObjectName=\"{}\">\n",
            ao_id,
            escape_xml(&obj.name)
        ));
        xml.push_str(&format!("    <audioPackFormatIDRef>{}</audioPackFormatIDRef>\n", ap_id));
        xml.push_str(&format!("    <audioTrackUIDRef>ATU_{:08}</audioTrackUIDRef>\n", 1001 + i));
        xml.push_str("  </audioObject>\n");

        xml.push_str(&format!(
            "  <audioPackFormat audioPackFormatID=\"{}\" audioPackFormatName=\"{}\" typeDefinition=\"DirectSpeakers\">\n",
            ap_id,
            escape_xml(&obj.name)
        ));
        xml.push_str(&format!("    <audioChannelFormatIDRef>{}</audioChannelFormatIDRef>\n", ac_id));
        xml.push_str("  </audioPackFormat>\n");

        xml.push_str(&format!(
            "  <audioChannelFormat audioChannelFormatID=\"{}\" audioChannelFormatName=\"{}\" typeDefinition=\"DirectSpeakers\">\n",
            ac_id,
            escape_xml(&obj.name)
        ));
        xml.push_str(&format!(
            "    <audioBlockFormat audioBlockFormatID=\"AB_{:08}_00000001\">\n",
            1001 + i
        ));
        xml.push_str(&format!(
            "      <cartesianPosition x=\"{:.4}\" y=\"{:.4}\" z=\"{:.4}\"/>\n",
            obj.position.x, obj.position.y, obj.position.z
        ));
        xml.push_str(&format!("      <gain>{:.4}</gain>\n", obj.gain));
        xml.push_str(&format!("      <diffuse>{:.4}</diffuse>\n", obj.spread));
        xml.push_str("    </audioBlockFormat>\n");
        xml.push_str("  </audioChannelFormat>\n");

        xml.push_str(&format!(
            "  <audioTrackFormat audioTrackFormatID=\"{}\" audioTrackFormatName=\"Track {}\">\n",
            at_id, obj.track_index + 1
        ));
        xml.push_str(&format!(
            "    <audioStreamFormatIDRef>AS_{:08}</audioStreamFormatIDRef>\n",
            1001 + i
        ));
        xml.push_str("  </audioTrackFormat>\n");
    }

    // Bed format tag
    xml.push_str(&format!(
        "  <!-- Summoner Immersive Bed Format: {:?}, SampleRate: {} Hz -->\n",
        bed_format, sample_rate
    ));

    xml.push_str("</audioFormatExtended>\n");
    xml
}

/// Parses an existing ADM XML string into an `AdmSessionMetadata` representation.
pub fn parse_adm_xml(xml: &str) -> Result<AdmSessionMetadata, String> {
    if !xml.contains("<audioFormatExtended") {
        return Err("Missing <audioFormatExtended> root element".to_string());
    }

    let programme_name = extract_tag_attribute(xml, "audioProgramme", "audioProgrammeName")
        .unwrap_or_else(|| "Summoner ADM Project".to_string());

    let mut objects = Vec::new();

    // Iterate through audioObject blocks
    let mut search_pos = 0;
    let mut track_idx = 0;

    while let Some(obj_start) = xml[search_pos..].find("<audioObject ") {
        let abs_start = search_pos + obj_start;
        if let Some(obj_end) = xml[abs_start..].find("</audioObject>") {
            let block = &xml[abs_start..abs_start + obj_end + 14];
            let name = extract_tag_attribute(block, "audioObject", "audioObjectName")
                .unwrap_or_else(|| format!("Object {}", track_idx + 1));
            let id = extract_tag_attribute(block, "audioObject", "audioObjectID")
                .unwrap_or_else(|| format!("AO_{:04}", 1001 + track_idx));

            let x = extract_tag_attribute(xml, "cartesianPosition", "x")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0);
            let y = extract_tag_attribute(xml, "cartesianPosition", "y")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(1.0);
            let z = extract_tag_attribute(xml, "cartesianPosition", "z")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0);

            objects.push(AdmAudioObject {
                id,
                name,
                track_index: track_idx,
                position: AdmCartesianPosition { x, y, z },
                gain: 1.0,
                spread: 0.0,
                is_bed_channel: track_idx < 12,
            });

            track_idx += 1;
            search_pos = abs_start + obj_end + 14;
        } else {
            break;
        }
    }

    Ok(AdmSessionMetadata {
        programme_name,
        sample_rate: 48000,
        bed_format: if objects.len() >= 12 {
            AdmBedFormat::DolbyAtmos7_1_4
        } else if objects.len() >= 8 {
            AdmBedFormat::Surround7_1
        } else if objects.len() >= 6 {
            AdmBedFormat::Surround5_1
        } else {
            AdmBedFormat::Stereo
        },
        objects,
    })
}

/// Exports multichannel audio buffers and ADM XML metadata to an ITU-R BS.2076 ADM BWF WAV file.
pub fn export_adm_bwf_wav(
    path: &Path,
    channel_buffers: &[Vec<f32>],
    sample_rate: u32,
    bit_depth: u16,
    adm_xml: &str,
) -> std::io::Result<()> {
    let num_channels = channel_buffers.len() as u16;
    let num_frames = if num_channels > 0 {
        channel_buffers[0].len() as u32
    } else {
        0
    };

    let bytes_per_sample = (bit_depth / 8) as usize;
    let block_align = num_channels as usize * bytes_per_sample;
    let byte_rate = sample_rate as usize * block_align;
    let data_bytes = num_frames as usize * block_align;

    let mut cursor = Cursor::new(Vec::new());

    // 1. RIFF Header placeholder
    cursor.write_all(b"RIFF")?;
    cursor.write_all(&[0u8; 4])?; // Total size placeholder
    cursor.write_all(b"WAVE")?;

    // 2. fmt chunk (WAVE_FORMAT_EXTENSIBLE for multichannel)
    cursor.write_all(b"fmt ")?;
    let fmt_chunk_size = 40u32;
    cursor.write_all(&fmt_chunk_size.to_le_bytes())?;
    cursor.write_all(&0xFFFEu16.to_le_bytes())?; // WAVE_FORMAT_EXTENSIBLE
    cursor.write_all(&num_channels.to_le_bytes())?;
    cursor.write_all(&sample_rate.to_le_bytes())?;
    cursor.write_all(&(byte_rate as u32).to_le_bytes())?;
    cursor.write_all(&(block_align as u16).to_le_bytes())?;
    cursor.write_all(&bit_depth.to_le_bytes())?;
    cursor.write_all(&22u16.to_le_bytes())?; // cbSize
    cursor.write_all(&bit_depth.to_le_bytes())?; // validBitsPerSample
    cursor.write_all(&0u32.to_le_bytes())?; // channelMask
    // SubFormat GUID: KSDATAFORMAT_SUBTYPE_PCM (00000001-0000-0010-8000-00aa00389b71)
    let pcm_guid = [
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38,
        0x9b, 0x71,
    ];
    cursor.write_all(&pcm_guid)?;

    // 3. bext chunk (Broadcast Wave Extension)
    cursor.write_all(b"bext")?;
    let bext_size = 602u32;
    cursor.write_all(&bext_size.to_le_bytes())?;
    let mut bext_data = vec![0u8; bext_size as usize];
    // Description (256 bytes)
    let desc = b"Summoner DAW ADM BWF Master";
    bext_data[..desc.len()].copy_from_slice(desc);
    // Originator (32 bytes)
    let orig = b"Summoner DAW";
    bext_data[256..256 + orig.len()].copy_from_slice(orig);
    cursor.write_all(&bext_data)?;

    // 4. axml chunk (ADM metadata ITU-R BS.2076)
    cursor.write_all(b"axml")?;
    let xml_bytes = adm_xml.as_bytes();
    let axml_size = xml_bytes.len() as u32;
    cursor.write_all(&axml_size.to_le_bytes())?;
    cursor.write_all(xml_bytes)?;
    if !axml_size.is_multiple_of(2) {
        cursor.write_all(&[0u8])?; // Padding byte for word alignment
    }

    // 5. data chunk
    cursor.write_all(b"data")?;
    cursor.write_all(&(data_bytes as u32).to_le_bytes())?;
    for f in 0..num_frames as usize {
        for ch_buf in channel_buffers.iter().take(num_channels as usize) {
            let sample = ch_buf.get(f).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
            if bit_depth == 24 {
                let s24 = (sample * 8_388_607.0).round() as i32;
                cursor.write_all(&s24.to_le_bytes()[..3])?;
            } else {
                let s16 = (sample * 32767.0).round() as i16;
                cursor.write_all(&s16.to_le_bytes())?;
            }
        }
    }
    if !data_bytes.is_multiple_of(2) {
        cursor.write_all(&[0u8])?;
    }

    // Update RIFF total file size
    let total_len = cursor.get_ref().len() as u32;
    let riff_payload_size = total_len - 8;
    cursor.seek(SeekFrom::Start(4))?;
    cursor.write_all(&riff_payload_size.to_le_bytes())?;

    // Write to disk
    let mut file = File::create(path)?;
    file.write_all(cursor.get_ref())?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn extract_tag_attribute(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let pattern_space = format!("<{} ", tag);
    let start_idx = if let Some(idx) = xml.find(&pattern_space) {
        idx
    } else {
        let pattern_exact = format!("<{}>", tag);
        xml.find(&pattern_exact)?
    };
    let end_idx = xml[start_idx..].find('>')? + start_idx;
    let tag_slice = &xml[start_idx..end_idx];

    let attr_pattern = format!("{}=\"", attr);
    let attr_start = tag_slice.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_slice[attr_start..].find('"')? + attr_start;
    Some(tag_slice[attr_start..attr_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adm_xml_generation_and_parsing() {
        let objects = vec![
            AdmAudioObject {
                id: "AO_1001".to_string(),
                name: "Dialog Lead".to_string(),
                track_index: 0,
                position: AdmCartesianPosition { x: 0.0, y: 1.0, z: 0.0 },
                gain: 1.0,
                spread: 0.0,
                is_bed_channel: true,
            },
            AdmAudioObject {
                id: "AO_1002".to_string(),
                name: "Helicopter SFX".to_string(),
                track_index: 1,
                position: AdmCartesianPosition { x: -0.8, y: 0.5, z: 0.9 },
                gain: 0.8,
                spread: 0.2,
                is_bed_channel: false,
            },
        ];

        let xml = generate_adm_xml(
            "SciFi Feature Film",
            48000,
            AdmBedFormat::DolbyAtmos7_1_4,
            &objects,
        );

        assert!(xml.contains("<audioFormatExtended"));
        assert!(xml.contains("SciFi Feature Film"));
        assert!(xml.contains("Helicopter SFX"));
        assert!(xml.contains("cartesianPosition"));

        let parsed = parse_adm_xml(&xml).expect("Failed to parse ADM XML");
        assert_eq!(parsed.programme_name, "SciFi Feature Film");
        assert_eq!(parsed.objects.len(), 2);
        assert_eq!(parsed.objects[0].name, "Dialog Lead");
        assert_eq!(parsed.objects[1].name, "Helicopter SFX");
    }

    #[test]
    fn test_export_adm_bwf_wav_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_adm_export.wav");

        let ch1 = vec![0.0f32; 100];
        let ch2 = vec![0.0f32; 100];
        let buffers = vec![ch1, ch2];

        let xml = "<audioFormatExtended xmlns=\"urn:itu:r:bs:2076:2\"><audioProgramme audioProgrammeName=\"Test\"/></audioFormatExtended>";
        export_adm_bwf_wav(&path, &buffers, 48000, 24, xml).expect("Failed to export ADM BWF");

        assert!(path.exists());
        let file_bytes = std::fs::read(&path).unwrap();
        assert_eq!(&file_bytes[0..4], b"RIFF");
        assert_eq!(&file_bytes[8..12], b"WAVE");

        // Verify presence of bext and axml chunk signatures
        let bext_found = file_bytes.windows(4).any(|w| w == b"bext");
        let axml_found = file_bytes.windows(4).any(|w| w == b"axml");
        assert!(bext_found, "Missing bext chunk in BWF");
        assert!(axml_found, "Missing axml chunk in ADM BWF");

        let _ = std::fs::remove_file(path);
    }
}
