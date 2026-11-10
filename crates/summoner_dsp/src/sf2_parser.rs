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

//! SoundFont 2 (SF2) binary parser according to the SoundFont 2.04 specification.
//!
//! Parses RIFF sfbk files containing INFO, sdta (sample audio), and pdta (preset & instrument hydra)
//! chunks, and maps presets to `MultiSampleBank` and `SampleRegion`s.

use crate::sampler::{LoopMode, MultiSampleBank, SampleBuffer, SampleRegion};
use std::sync::Arc;
use summoner_core::audio::Sample;

/// Information metadata parsed from the SF2 `INFO` list.
#[derive(Debug, Clone, Default)]
pub struct Sf2Info {
    pub soundfont_version: (u16, u16),
    pub wave_pool_version: (u16, u16),
    pub name: String,
    pub engine: String,
    pub author: String,
    pub copyright: String,
    pub comments: String,
}

/// A parsed audio sample entry in the SF2 file.
#[derive(Debug, Clone)]
pub struct Sf2Sample {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub loop_start: usize,
    pub loop_end: usize,
    pub sample_rate: u32,
    pub original_pitch: u8,
    pub pitch_correction_cents: i8,
    pub sample_type: u16,
    pub audio: Arc<SampleBuffer>,
}

/// An SF2 Instrument Zone defining key/vel mapping and generator parameters.
#[derive(Debug, Clone)]
pub struct Sf2InstrumentZone {
    pub key_range: (u8, u8),
    pub vel_range: (u8, u8),
    pub sample_index: Option<usize>,
    pub overriding_root_key: Option<u8>,
    pub loop_mode: LoopMode,
    pub coarse_tune_semitones: i16,
    pub fine_tune_cents: i16,
}

/// An SF2 Instrument definition.
#[derive(Debug, Clone)]
pub struct Sf2Instrument {
    pub name: String,
    pub zones: Vec<Sf2InstrumentZone>,
}

/// An SF2 Preset Zone pointing to an instrument.
#[derive(Debug, Clone)]
pub struct Sf2PresetZone {
    pub key_range: (u8, u8),
    pub vel_range: (u8, u8),
    pub instrument_index: Option<usize>,
}

/// An SF2 Preset definition.
#[derive(Debug, Clone)]
pub struct Sf2Preset {
    pub name: String,
    pub preset_number: u16,
    pub bank_number: u16,
    pub zones: Vec<Sf2PresetZone>,
}

/// A fully parsed SoundFont 2 file.
#[derive(Debug, Clone)]
pub struct Sf2SoundFont {
    pub info: Sf2Info,
    pub samples: Vec<Sf2Sample>,
    pub instruments: Vec<Sf2Instrument>,
    pub presets: Vec<Sf2Preset>,
}

impl Sf2SoundFont {
    /// Parse an SF2 file from a byte slice.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 {
            return Err("SF2 data too short for RIFF header".to_string());
        }

        // Verify "RIFF"
        if &bytes[0..4] != b"RIFF" {
            return Err("Not a valid RIFF file".to_string());
        }

        // Verify "sfbk"
        if &bytes[8..12] != b"sfbk" {
            return Err("Not a valid SoundFont 2 (sfbk) RIFF file".to_string());
        }

        let mut pos = 12;
        let mut info = Sf2Info::default();
        let mut raw_samples_pcm: Vec<i16> = Vec::new();
        let mut samples = Vec::new();
        let mut instruments = Vec::new();
        let mut presets = Vec::new();

        while pos + 8 <= bytes.len() {
            let chunk_id = &bytes[pos..pos + 4];
            let chunk_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            let chunk_end = (pos + chunk_size).min(bytes.len());
            let chunk_data = &bytes[pos..chunk_end];

            if chunk_id == b"LIST" && chunk_data.len() >= 4 {
                let list_type = &chunk_data[0..4];
                let sub_data = &chunk_data[4..];

                match list_type {
                    b"INFO" => {
                        info = parse_info_list(sub_data);
                    }
                    b"sdta" => {
                        raw_samples_pcm = parse_sdta_list(sub_data);
                    }
                    b"pdta" => {
                        let parsed_pdta = parse_pdta_list(sub_data, &raw_samples_pcm)?;
                        samples = parsed_pdta.0;
                        instruments = parsed_pdta.1;
                        presets = parsed_pdta.2;
                    }
                    _ => {}
                }
            }

            pos = chunk_end + (chunk_size % 2); // Word alignment
        }

        Ok(Sf2SoundFont {
            info,
            samples,
            instruments,
            presets,
        })
    }

    /// Build a `MultiSampleBank` from a preset index.
    pub fn build_bank_for_preset(&self, preset_idx: usize) -> Result<MultiSampleBank, String> {
        let preset = self
            .presets
            .get(preset_idx)
            .ok_or_else(|| format!("Preset index {} out of range", preset_idx))?;

        let mut bank = MultiSampleBank::new();

        for pzone in &preset.zones {
            if let Some(inst_idx) = pzone.instrument_index {
                if let Some(inst) = self.instruments.get(inst_idx) {
                    for izone in &inst.zones {
                        if let Some(sample_idx) = izone.sample_index {
                            if let Some(sample) = self.samples.get(sample_idx) {
                                // Compute intersection of preset and instrument ranges
                                let lokey = pzone.key_range.0.max(izone.key_range.0);
                                let hikey = pzone.key_range.1.min(izone.key_range.1);
                                let lovel = pzone.vel_range.0.max(izone.vel_range.0);
                                let hivel = pzone.vel_range.1.min(izone.vel_range.1);

                                if lokey <= hikey && lovel <= hivel {
                                    let root_key = izone
                                        .overriding_root_key
                                        .unwrap_or(sample.original_pitch);

                                    let mut region = SampleRegion::new(
                                        lokey,
                                        hikey,
                                        root_key,
                                        format!("{}_{}", inst.name, sample.name),
                                    );
                                    region.lovel = lovel;
                                    region.hivel = hivel;
                                    region.loop_mode = izone.loop_mode;
                                    region.loop_start = sample.loop_start;
                                    region.loop_end = sample.loop_end;
                                    region.buffer = Some(sample.audio.clone());

                                    bank.add_region(region);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(bank)
    }

    /// Build a `MultiSampleBank` by matching preset name (case-insensitive substring).
    pub fn build_bank_by_name(&self, name: &str) -> Result<MultiSampleBank, String> {
        let name_lower = name.to_lowercase();
        let idx = self
            .presets
            .iter()
            .position(|p| p.name.to_lowercase().contains(&name_lower))
            .ok_or_else(|| format!("No preset found matching '{}'", name))?;

        self.build_bank_for_preset(idx)
    }
}

fn parse_info_list(bytes: &[u8]) -> Sf2Info {
    let mut info = Sf2Info::default();
    let mut pos = 0;

    while pos + 8 <= bytes.len() {
        let sub_id = &bytes[pos..pos + 4];
        let sub_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let end = (pos + sub_size).min(bytes.len());
        let val_bytes = &bytes[pos..end];

        let val_str = String::from_utf8_lossy(val_bytes)
            .trim_end_matches('\0')
            .trim()
            .to_string();

        match sub_id {
            b"INAM" => info.name = val_str,
            b"ISFT" => info.engine = val_str,
            b"IENG" => info.author = val_str,
            b"ICOP" => info.copyright = val_str,
            b"ICMT" => info.comments = val_str,
            _ => {}
        }

        pos = end + (sub_size % 2);
    }

    info
}

fn parse_sdta_list(bytes: &[u8]) -> Vec<i16> {
    let mut pos = 0;
    while pos + 8 <= bytes.len() {
        let sub_id = &bytes[pos..pos + 4];
        let sub_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let end = (pos + sub_size).min(bytes.len());
        if sub_id == b"smpl" {
            let data = &bytes[pos..end];
            let num_samples = data.len() / 2;
            let mut pcm = Vec::with_capacity(num_samples);
            for i in 0..num_samples {
                let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
                pcm.push(sample);
            }
            return pcm;
        }

        pos = end + (sub_size % 2);
    }
    Vec::new()
}

struct RawGen {
    oper: u16,
    amount: u16,
}

#[derive(Default)]
struct ZoneAccumulator {
    key_range: (u8, u8),
    vel_range: (u8, u8),
    sample_id: Option<usize>,
    root_key: Option<u8>,
    loop_mode: LoopMode,
}

#[allow(clippy::type_complexity, clippy::needless_range_loop)]
fn parse_pdta_list(
    bytes: &[u8],
    raw_pcm: &[i16],
) -> Result<(Vec<Sf2Sample>, Vec<Sf2Instrument>, Vec<Sf2Preset>), String> {
    let mut chunks: std::collections::HashMap<[u8; 4], &[u8]> = std::collections::HashMap::new();

    let mut pos = 0;
    while pos + 8 <= bytes.len() {
        let sub_id: [u8; 4] = bytes[pos..pos + 4].try_into().unwrap();
        let sub_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let end = (pos + sub_size).min(bytes.len());
        chunks.insert(sub_id, &bytes[pos..end]);

        pos = end + (sub_size % 2);
    }

    // 1. Parse Sample Headers (shdr)
    let mut samples = Vec::new();
    if let Some(&shdr_bytes) = chunks.get(b"shdr") {
        let entry_size = 46;
        let count = shdr_bytes.len() / entry_size;
        for i in 0..count {
            let offset = i * entry_size;
            let name_raw = &shdr_bytes[offset..offset + 20];
            let name = String::from_utf8_lossy(name_raw)
                .trim_end_matches('\0')
                .trim()
                .to_string();

            if name == "EOS" {
                break;
            }

            let start = u32::from_le_bytes(shdr_bytes[offset + 20..offset + 24].try_into().unwrap()) as usize;
            let end = u32::from_le_bytes(shdr_bytes[offset + 24..offset + 28].try_into().unwrap()) as usize;
            let loop_start = u32::from_le_bytes(shdr_bytes[offset + 28..offset + 32].try_into().unwrap()) as usize;
            let loop_end = u32::from_le_bytes(shdr_bytes[offset + 32..offset + 36].try_into().unwrap()) as usize;
            let sample_rate = u32::from_le_bytes(shdr_bytes[offset + 36..offset + 40].try_into().unwrap());
            let original_pitch = shdr_bytes[offset + 40];
            let pitch_correction = shdr_bytes[offset + 41] as i8;
            let sample_type = u16::from_le_bytes(shdr_bytes[offset + 44..offset + 46].try_into().unwrap());

            // Extract audio frames
            let pcm_slice = if start < raw_pcm.len() && end <= raw_pcm.len() && start < end {
                &raw_pcm[start..end]
            } else {
                &[]
            };

            let f32_samples: Vec<Sample> = pcm_slice.iter().map(|&s| s as f32 / 32768.0).collect();
            let audio_buffer = Arc::new(SampleBuffer::new(f32_samples, sample_rate.max(8000), 1));

            samples.push(Sf2Sample {
                name,
                start,
                end,
                loop_start: loop_start.saturating_sub(start),
                loop_end: loop_end.saturating_sub(start),
                sample_rate,
                original_pitch,
                pitch_correction_cents: pitch_correction,
                sample_type,
                audio: audio_buffer,
            });
        }
    }

    // 2. Parse Instrument Generators (igen)
    let mut igens = Vec::new();
    if let Some(&igen_bytes) = chunks.get(b"igen") {
        for i in 0..(igen_bytes.len() / 4) {
            let offset = i * 4;
            let oper = u16::from_le_bytes(igen_bytes[offset..offset + 2].try_into().unwrap());
            let amount = u16::from_le_bytes(igen_bytes[offset + 2..offset + 4].try_into().unwrap());
            igens.push(RawGen { oper, amount });
        }
    }

    // 3. Parse Instrument Bags (ibag)
    let mut ibags = Vec::new();
    if let Some(&ibag_bytes) = chunks.get(b"ibag") {
        for i in 0..(ibag_bytes.len() / 4) {
            let offset = i * 4;
            let gen_ndx = u16::from_le_bytes(ibag_bytes[offset..offset + 2].try_into().unwrap()) as usize;
            ibags.push(gen_ndx);
        }
    }

    // 4. Parse Instruments (inst)
    let mut instruments = Vec::new();
    if let Some(&inst_bytes) = chunks.get(b"inst") {
        let entry_size = 22;
        let count = inst_bytes.len() / entry_size;
        let mut raw_insts = Vec::new();
        for i in 0..count {
            let offset = i * entry_size;
            let name = String::from_utf8_lossy(&inst_bytes[offset..offset + 20])
                .trim_end_matches('\0')
                .trim()
                .to_string();
            let bag_ndx = u16::from_le_bytes(inst_bytes[offset + 20..offset + 22].try_into().unwrap()) as usize;
            raw_insts.push((name, bag_ndx));
        }

        for i in 0..raw_insts.len().saturating_sub(1) {
            let name = raw_insts[i].0.clone();
            let bag_start = raw_insts[i].1;
            let bag_end = raw_insts[i + 1].1.min(ibags.len());

            let mut zones = Vec::new();
            for b in bag_start..bag_end {
                let gen_start = ibags[b];
                let gen_end = if b + 1 < ibags.len() { ibags[b + 1] } else { igens.len() };

                let mut accum = ZoneAccumulator {
                    key_range: (0, 127),
                    vel_range: (0, 127),
                    sample_id: None,
                    root_key: None,
                    loop_mode: LoopMode::NoLoop,
                };

                for g in gen_start..gen_end.min(igens.len()) {
                    let gen = &igens[g];
                    match gen.oper {
                        43 => { // keyRange
                            let lo = (gen.amount & 0xFF) as u8;
                            let hi = ((gen.amount >> 8) & 0xFF) as u8;
                            accum.key_range = (lo, hi);
                        }
                        44 => { // velRange
                            let lo = (gen.amount & 0xFF) as u8;
                            let hi = ((gen.amount >> 8) & 0xFF) as u8;
                            accum.vel_range = (lo, hi);
                        }
                        53 => { // sampleID
                            accum.sample_id = Some(gen.amount as usize);
                        }
                        54 => { // sampleModes
                            accum.loop_mode = match gen.amount & 0x03 {
                                1 => LoopMode::LoopContinuous,
                                3 => LoopMode::LoopRelease,
                                _ => LoopMode::NoLoop,
                            };
                        }
                        58 => { // overridingRootKey
                            accum.root_key = Some((gen.amount & 0x7F) as u8);
                        }
                        _ => {}
                    }
                }

                if accum.sample_id.is_some() {
                    zones.push(Sf2InstrumentZone {
                        key_range: accum.key_range,
                        vel_range: accum.vel_range,
                        sample_index: accum.sample_id,
                        overriding_root_key: accum.root_key,
                        loop_mode: accum.loop_mode,
                        coarse_tune_semitones: 0,
                        fine_tune_cents: 0,
                    });
                }
            }

            instruments.push(Sf2Instrument { name, zones });
        }
    }

    // 5. Parse Preset Generators (pgen)
    let mut pgens = Vec::new();
    if let Some(&pgen_bytes) = chunks.get(b"pgen") {
        for i in 0..(pgen_bytes.len() / 4) {
            let offset = i * 4;
            let oper = u16::from_le_bytes(pgen_bytes[offset..offset + 2].try_into().unwrap());
            let amount = u16::from_le_bytes(pgen_bytes[offset + 2..offset + 4].try_into().unwrap());
            pgens.push(RawGen { oper, amount });
        }
    }

    // 6. Parse Preset Bags (pbag)
    let mut pbags = Vec::new();
    if let Some(&pbag_bytes) = chunks.get(b"pbag") {
        for i in 0..(pbag_bytes.len() / 4) {
            let offset = i * 4;
            let gen_ndx = u16::from_le_bytes(pbag_bytes[offset..offset + 2].try_into().unwrap()) as usize;
            pbags.push(gen_ndx);
        }
    }

    // 7. Parse Presets (phdr)
    let mut presets = Vec::new();
    if let Some(&phdr_bytes) = chunks.get(b"phdr") {
        let entry_size = 38;
        let count = phdr_bytes.len() / entry_size;
        let mut raw_presets = Vec::new();

        for i in 0..count {
            let offset = i * entry_size;
            let name = String::from_utf8_lossy(&phdr_bytes[offset..offset + 20])
                .trim_end_matches('\0')
                .trim()
                .to_string();
            let preset_num = u16::from_le_bytes(phdr_bytes[offset + 20..offset + 22].try_into().unwrap());
            let bank_num = u16::from_le_bytes(phdr_bytes[offset + 22..offset + 24].try_into().unwrap());
            let bag_ndx = u16::from_le_bytes(phdr_bytes[offset + 24..offset + 26].try_into().unwrap()) as usize;

            raw_presets.push((name, preset_num, bank_num, bag_ndx));
        }

        for i in 0..raw_presets.len().saturating_sub(1) {
            let (name, preset_number, bank_number, bag_start) = raw_presets[i].clone();
            let bag_end = raw_presets[i + 1].3.min(pbags.len());

            let mut zones = Vec::new();
            for b in bag_start..bag_end {
                let gen_start = pbags[b];
                let gen_end = if b + 1 < pbags.len() { pbags[b + 1] } else { pgens.len() };

                let mut key_range = (0u8, 127u8);
                let mut vel_range = (0u8, 127u8);
                let mut instrument_index = None;

                for g in gen_start..gen_end.min(pgens.len()) {
                    let gen = &pgens[g];
                    match gen.oper {
                        41 => { // instrument
                            instrument_index = Some(gen.amount as usize);
                        }
                        43 => { // keyRange
                            let lo = (gen.amount & 0xFF) as u8;
                            let hi = ((gen.amount >> 8) & 0xFF) as u8;
                            key_range = (lo, hi);
                        }
                        44 => { // velRange
                            let lo = (gen.amount & 0xFF) as u8;
                            let hi = ((gen.amount >> 8) & 0xFF) as u8;
                            vel_range = (lo, hi);
                        }
                        _ => {}
                    }
                }

                if instrument_index.is_some() {
                    zones.push(Sf2PresetZone {
                        key_range,
                        vel_range,
                        instrument_index,
                    });
                }
            }

            presets.push(Sf2Preset {
                name,
                preset_number,
                bank_number,
                zones,
            });
        }
    }

    Ok((samples, instruments, presets))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthesizes a valid minimal SoundFont 2 binary chunk in memory for unit testing.
    fn create_test_sf2_bytes() -> Vec<u8> {
        let mut buf = Vec::new();

        // RIFF header placeholder
        buf.extend_from_slice(b"RIFF\0\0\0\0sfbk");

        // 1. INFO chunk
        let mut info_body = Vec::new();
        // INAM
        let name_bytes = b"Test Synth SoundFont\0";
        info_body.extend_from_slice(b"INAM");
        info_body.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        info_body.extend_from_slice(name_bytes);
        if name_bytes.len() % 2 != 0 {
            info_body.push(0);
        }

        buf.extend_from_slice(b"LIST");
        buf.extend_from_slice(&((info_body.len() + 4) as u32).to_le_bytes());
        buf.extend_from_slice(b"INFO");
        buf.extend_from_slice(&info_body);

        // 2. sdta chunk
        let mut sdta_body = Vec::new();
        // 100 samples of 16-bit PCM sine wave
        let pcm_samples: Vec<i16> = (0..100)
            .map(|i| ((i as f32 * 0.1).sin() * 16000.0) as i16)
            .collect();
        let mut pcm_bytes = Vec::new();
        for s in &pcm_samples {
            pcm_bytes.extend_from_slice(&s.to_le_bytes());
        }

        sdta_body.extend_from_slice(b"smpl");
        sdta_body.extend_from_slice(&(pcm_bytes.len() as u32).to_le_bytes());
        sdta_body.extend_from_slice(&pcm_bytes);

        buf.extend_from_slice(b"LIST");
        buf.extend_from_slice(&((sdta_body.len() + 4) as u32).to_le_bytes());
        buf.extend_from_slice(b"sdta");
        buf.extend_from_slice(&sdta_body);

        // 3. pdta chunk
        let mut pdta_body = Vec::new();

        // phdr (2 entries: Preset 0, EOS)
        let mut phdr = Vec::new();
        // Entry 0
        let mut p0 = vec![0u8; 38];
        p0[0..10].copy_from_slice(b"TestPreset");
        p0[20..22].copy_from_slice(&0u16.to_le_bytes()); // preset 0
        p0[22..24].copy_from_slice(&0u16.to_le_bytes()); // bank 0
        p0[24..26].copy_from_slice(&0u16.to_le_bytes()); // bag 0
        phdr.extend_from_slice(&p0);
        // EOS
        let mut peos = vec![0u8; 38];
        peos[0..3].copy_from_slice(b"EOP");
        peos[24..26].copy_from_slice(&1u16.to_le_bytes()); // bag 1
        phdr.extend_from_slice(&peos);

        pdta_body.extend_from_slice(b"phdr");
        pdta_body.extend_from_slice(&(phdr.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&phdr);

        // pbag (2 entries)
        let mut pbag = Vec::new();
        pbag.extend_from_slice(&0u16.to_le_bytes()); // gen 0
        pbag.extend_from_slice(&0u16.to_le_bytes()); // mod 0
        pbag.extend_from_slice(&1u16.to_le_bytes()); // gen 1
        pbag.extend_from_slice(&0u16.to_le_bytes()); // mod 0
        pdta_body.extend_from_slice(b"pbag");
        pdta_body.extend_from_slice(&(pbag.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&pbag);

        // pgen (1 entry: instrument 0)
        let mut pgen = Vec::new();
        pgen.extend_from_slice(&41u16.to_le_bytes()); // oper: instrument
        pgen.extend_from_slice(&0u16.to_le_bytes());  // inst 0
        pdta_body.extend_from_slice(b"pgen");
        pdta_body.extend_from_slice(&(pgen.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&pgen);

        // inst (2 entries: Inst 0, EOS)
        let mut inst = Vec::new();
        let mut i0 = vec![0u8; 22];
        i0[0..8].copy_from_slice(b"TestInst");
        i0[20..22].copy_from_slice(&0u16.to_le_bytes()); // bag 0
        inst.extend_from_slice(&i0);
        let mut ieos = vec![0u8; 22];
        ieos[0..3].copy_from_slice(b"EOI");
        ieos[20..22].copy_from_slice(&1u16.to_le_bytes()); // bag 1
        inst.extend_from_slice(&ieos);

        pdta_body.extend_from_slice(b"inst");
        pdta_body.extend_from_slice(&(inst.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&inst);

        // ibag (2 entries)
        let mut ibag = Vec::new();
        ibag.extend_from_slice(&0u16.to_le_bytes()); // gen 0
        ibag.extend_from_slice(&0u16.to_le_bytes()); // mod 0
        ibag.extend_from_slice(&1u16.to_le_bytes()); // gen 1
        ibag.extend_from_slice(&0u16.to_le_bytes()); // mod 0
        pdta_body.extend_from_slice(b"ibag");
        pdta_body.extend_from_slice(&(ibag.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&ibag);

        // igen (1 entry: sampleID 0)
        let mut igen = Vec::new();
        igen.extend_from_slice(&53u16.to_le_bytes()); // oper: sampleID
        igen.extend_from_slice(&0u16.to_le_bytes());  // sample 0
        pdta_body.extend_from_slice(b"igen");
        pdta_body.extend_from_slice(&(igen.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&igen);

        // shdr (2 entries: Sample 0, EOS)
        let mut shdr = Vec::new();
        let mut s0 = vec![0u8; 46];
        s0[0..10].copy_from_slice(b"TestSample");
        s0[20..24].copy_from_slice(&0u32.to_le_bytes());   // start 0
        s0[24..28].copy_from_slice(&100u32.to_le_bytes()); // end 100
        s0[28..32].copy_from_slice(&10u32.to_le_bytes());  // loop_start 10
        s0[32..36].copy_from_slice(&90u32.to_le_bytes());  // loop_end 90
        s0[36..40].copy_from_slice(&44100u32.to_le_bytes()); // sample_rate 44100
        s0[40] = 60; // original pitch C4
        s0[41] = 0;  // correction
        s0[44..46].copy_from_slice(&1u16.to_le_bytes()); // mono sample
        shdr.extend_from_slice(&s0);

        let mut seos = vec![0u8; 46];
        seos[0..3].copy_from_slice(b"EOS");
        shdr.extend_from_slice(&seos);

        pdta_body.extend_from_slice(b"shdr");
        pdta_body.extend_from_slice(&(shdr.len() as u32).to_le_bytes());
        pdta_body.extend_from_slice(&shdr);

        buf.extend_from_slice(b"LIST");
        buf.extend_from_slice(&((pdta_body.len() + 4) as u32).to_le_bytes());
        buf.extend_from_slice(b"pdta");
        buf.extend_from_slice(&pdta_body);

        // Update total RIFF size
        let total_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&total_size.to_le_bytes());

        buf
    }

    #[test]
    fn test_parse_sf2_binary() {
        let sf2_bytes = create_test_sf2_bytes();
        let sf2 = Sf2SoundFont::parse(&sf2_bytes).expect("Failed to parse synthesized SF2");

        assert_eq!(sf2.info.name, "Test Synth SoundFont");
        assert_eq!(sf2.samples.len(), 1);
        assert_eq!(sf2.samples[0].name, "TestSample");
        assert_eq!(sf2.samples[0].sample_rate, 44100);
        assert_eq!(sf2.samples[0].original_pitch, 60);
        assert_eq!(sf2.samples[0].audio.data.len(), 100);

        assert_eq!(sf2.instruments.len(), 1);
        assert_eq!(sf2.instruments[0].name, "TestInst");
        assert_eq!(sf2.presets.len(), 1);
        assert_eq!(sf2.presets[0].name, "TestPreset");

        // Build bank from preset
        let bank = sf2.build_bank_for_preset(0).expect("Failed to build bank");
        assert_eq!(bank.regions.len(), 1);
        assert_eq!(bank.regions[0].pitch_keycenter, 60);
        assert_eq!(bank.regions[0].lokey, 0);
        assert_eq!(bank.regions[0].hikey, 127);
    }
}
