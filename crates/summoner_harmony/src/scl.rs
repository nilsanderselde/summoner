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

pub struct SclTuning {
    pub title: String,
    pub num_notes: usize,
    pub cents_or_ratios: Vec<f64>,
}

impl SclTuning {
    pub fn parse(content: &str) -> Result<Self, &'static str> {
        let mut lines = content.lines().filter(|l| !l.trim().starts_with('!') && !l.trim().is_empty());
        let title = lines.next().ok_or("Missing title")?.trim().to_string();
        let num_notes_str = lines.next().ok_or("Missing number of notes")?;
        let num_notes = num_notes_str.trim().parse::<usize>().map_err(|_| "Invalid number of notes")?;
        
        let mut cents_or_ratios = Vec::new();
        for line in lines.take(num_notes) {
            let val_str = line.split_whitespace().next().unwrap_or("");
            if val_str.contains('.') {
                // Cents
                let cents: f64 = val_str.parse().map_err(|_| "Invalid cents value")?;
                cents_or_ratios.push(cents);
            } else if val_str.contains('/') {
                // Ratio
                let mut parts = val_str.split('/');
                let num: f64 = parts.next().unwrap().parse().map_err(|_| "Invalid ratio num")?;
                let den: f64 = parts.next().unwrap_or("1").parse().map_err(|_| "Invalid ratio den")?;
                let cents = 1200.0 * (num / den).log2();
                cents_or_ratios.push(cents);
            } else {
                // Integer ratio
                let num: f64 = val_str.parse().map_err(|_| "Invalid integer ratio")?;
                let cents = 1200.0 * num.log2();
                cents_or_ratios.push(cents);
            }
        }
        
        Ok(Self {
            title,
            num_notes,
            cents_or_ratios,
        })
    }
}
