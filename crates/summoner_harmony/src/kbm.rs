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

pub struct KbmMapping {
    pub map_size: usize,
    pub first_midi_note: i32,
    pub last_midi_note: i32,
    pub middle_note: i32,
    pub reference_note: i32,
    pub reference_freq: f64,
    pub scale_degree: i32,
    pub mapping: Vec<Option<i32>>,
}

impl KbmMapping {
    pub fn parse(content: &str) -> Result<Self, &'static str> {
        let mut lines = content
            .lines()
            .filter(|l| !l.trim().starts_with('!') && !l.trim().is_empty());

        let map_size: usize = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let first_midi_note: i32 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let last_midi_note: i32 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let middle_note: i32 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let reference_note: i32 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let reference_freq: f64 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;
        let scale_degree: i32 = lines
            .next()
            .ok_or("err")?
            .trim()
            .parse()
            .map_err(|_| "err")?;

        let mut mapping = Vec::new();
        for _ in 0..map_size {
            let val_str = lines.next().ok_or("err")?.trim();
            if val_str.to_lowercase() == "x" {
                mapping.push(None);
            } else {
                let deg: i32 = val_str.parse().map_err(|_| "err")?;
                mapping.push(Some(deg));
            }
        }

        Ok(Self {
            map_size,
            first_midi_note,
            last_midi_note,
            middle_note,
            reference_note,
            reference_freq,
            scale_degree,
            mapping,
        })
    }
}
