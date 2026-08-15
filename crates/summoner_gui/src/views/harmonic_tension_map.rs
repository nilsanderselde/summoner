// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Chord Progression Builder & Harmonic Tension Heatmap (Step 1345).

use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

/// Chord quality classification for harmonic analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    MajorTriad,
    MinorTriad,
    MajorSeventh,
    MinorSeventh,
    DominantSeventh,
    DiminishedSeventh,
    AugmentedSeventh,
    AlteredDominant,
}

impl ChordQuality {
    pub fn display_suffix(&self) -> &'static str {
        match self {
            Self::MajorTriad => "",
            Self::MinorTriad => "m",
            Self::MajorSeventh => "maj7",
            Self::MinorSeventh => "m7",
            Self::DominantSeventh => "7",
            Self::DiminishedSeventh => "dim7",
            Self::AugmentedSeventh => "aug7",
            Self::AlteredDominant => "7alt",
        }
    }

    /// Base harmonic tension factor (0.0 consonance to 1.0 maximum dissonance)
    pub fn base_tension(&self) -> f32 {
        match self {
            Self::MajorTriad => 0.05,
            Self::MinorTriad => 0.12,
            Self::MajorSeventh => 0.15,
            Self::MinorSeventh => 0.25,
            Self::DominantSeventh => 0.65,
            Self::AugmentedSeventh => 0.75,
            Self::DiminishedSeventh => 0.85,
            Self::AlteredDominant => 0.95,
        }
    }
}

/// A single chord block in the progression.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordBlock {
    pub root_note: String,
    pub quality: ChordQuality,
    pub roman_numeral: String,
    pub tension_score: f32, // 0.0 ..= 1.0
    pub duration_beats: f32,
}

impl ChordBlock {
    pub fn new(
        root_note: impl Into<String>,
        quality: ChordQuality,
        roman_numeral: impl Into<String>,
        duration_beats: f32,
    ) -> Self {
        let tension = quality.base_tension();
        Self {
            root_note: root_note.into(),
            quality,
            roman_numeral: roman_numeral.into(),
            tension_score: tension,
            duration_beats,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}{}", self.root_note, self.quality.display_suffix())
    }
}

/// Interactive Chord Progression Builder & Harmonic Tension Heatmap View (Step 1345).
#[derive(Debug, Clone)]
pub struct HarmonicTensionMapView {
    pub chords: Vec<ChordBlock>,
    pub selected_index: Option<usize>,
    pub key_root: String,
    pub edo_division: u16,
    pub color_palette: ContrastColorPalette,
}

impl Default for HarmonicTensionMapView {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonicTensionMapView {
    pub fn new() -> Self {
        let mut view = Self {
            chords: Vec::new(),
            selected_index: Some(0),
            key_root: "C".into(),
            edo_division: 12,
            color_palette: ContrastColorPalette::default(),
        };

        // Standard jazz ii - V - I - VI turnaround progression
        view.chords
            .push(ChordBlock::new("D", ChordQuality::MinorSeventh, "ii7", 4.0));
        view.chords.push(ChordBlock::new(
            "G",
            ChordQuality::DominantSeventh,
            "V7",
            4.0,
        ));
        view.chords.push(ChordBlock::new(
            "C",
            ChordQuality::MajorSeventh,
            "Imaj7",
            4.0,
        ));
        view.chords.push(ChordBlock::new(
            "A",
            ChordQuality::AlteredDominant,
            "VI7alt",
            4.0,
        ));

        view
    }

    pub fn add_chord(&mut self, chord: ChordBlock) {
        self.chords.push(chord);
    }

    pub fn remove_chord(&mut self, index: usize) -> Option<ChordBlock> {
        if index < self.chords.len() {
            Some(self.chords.remove(index))
        } else {
            None
        }
    }

    /// Maps harmonic tension score [0.0 ..= 1.0] to WCAG AA RGB color ramp
    pub fn tension_to_rgb(tension: f32) -> (u8, u8, u8) {
        let t = tension.clamp(0.0, 1.0);
        if t < 0.3 {
            // Low tension: Cyan / Green Consonance
            let frac = t / 0.3;
            (
                (0.0 + frac * 40.0) as u8,
                (229.0 + frac * 26.0) as u8,
                (255.0 - frac * 75.0) as u8,
            )
        } else if t < 0.7 {
            // Mid tension: Amber / Yellow Dominant
            let frac = (t - 0.3) / 0.4;
            (
                (40.0 + frac * 215.0) as u8,
                (255.0 - frac * 40.0) as u8,
                (180.0 - frac * 180.0) as u8,
            )
        } else {
            // High tension: Hot Coral / Magenta Altered Dissonance
            let frac = (t - 0.7) / 0.3;
            (
                255,
                (215.0 - frac * 170.0) as u8,
                (0.0 + frac * 120.0) as u8,
            )
        }
    }

    /// Render deterministic ASCII representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[INTERACTIVE CHORD PROGRESSION & TENSION MAP]\n");
        out.push_str(&format!(
            "Key: {} Major | Tuning: {}-EDO | Progression Length: {} chords\n",
            self.key_root,
            self.edo_division,
            self.chords.len()
        ));

        out.push_str("Chords & Tension Scores:\n");
        for (i, c) in self.chords.iter().enumerate() {
            let rgb = Self::tension_to_rgb(c.tension_score);
            out.push_str(&format!(
                "  #{}: {} ({}) | Tension: {:.0}% | RGB: ({}, {}, {})\n",
                i + 1,
                c.full_name(),
                c.roman_numeral,
                c.tension_score * 100.0,
                rgb.0,
                rgb.1,
                rgb.2
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl HarmonicTensionMapView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("HARMONIC TENSION MAP & PROGRESSION BUILDER");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Key: {} | Tuning: {}-EDO",
                            self.key_root, self.edo_division
                        ))
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(0, 229, 255)),
                    );
                });
            });

            ui.add_space(10.0);

            // Progression Chord Blocks Row
            ui.label(
                egui::RichText::new("PROGRESSION BLOCKS:")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(200, 220, 245)),
            );

            ui.horizontal(|ui| {
                for (i, chord) in self.chords.iter().enumerate() {
                    let rgb = Self::tension_to_rgb(chord.tension_score);
                    let tension_col = Color32::from_rgb(rgb.0, rgb.1, rgb.2);
                    let is_sel = self.selected_index == Some(i);

                    let card_bg = if is_sel {
                        Color32::from_rgb(25, 36, 56)
                    } else {
                        Color32::from_rgb(18, 24, 38)
                    };

                    let stroke_width: f32 = if is_sel { 2.0 } else { 1.0 };

                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(Stroke::new(stroke_width, tension_col))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(chord.full_name())
                                        .size(15.0)
                                        .strong()
                                        .color(Color32::from_rgb(240, 245, 255)),
                                );
                                ui.label(
                                    egui::RichText::new(&chord.roman_numeral)
                                        .size(12.0)
                                        .color(Color32::from_rgb(150, 180, 220)),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:.0}% Tension",
                                        chord.tension_score * 100.0
                                    ))
                                    .size(11.0)
                                    .strong()
                                    .color(tension_col),
                                );
                            });
                        });

                    ui.add_space(10.0);
                }
            });
        })
        .response
    }
}
