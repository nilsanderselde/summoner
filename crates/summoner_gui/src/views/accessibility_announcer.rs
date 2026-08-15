// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Accessibility Screen Reader Narration Cues & Keyboard Focus Ring (Step 1343).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DEFAULT_RING_THICKNESS: f32 = 3.0;
pub const DEFAULT_RING_OFFSET: f32 = 3.0;

/// Semantic accessibility UI roles for screen readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibleRole {
    Button,
    Slider,
    Toggle,
    RackSlot,
    TabItem,
    TextInput,
    Dialog,
}

impl AccessibleRole {
    pub fn role_name(&self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Slider => "Slider",
            Self::Toggle => "Toggle Switch",
            Self::RackSlot => "DSP Rack Slot",
            Self::TabItem => "Navigation Tab",
            Self::TextInput => "Text Field",
            Self::Dialog => "Modal Dialog",
        }
    }
}

/// Narration priority levels for screen readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrationPriority {
    Assertive, // Interrupted immediately (e.g. Peak Clip, Overload)
    Polite,    // Spoken after current queue item (e.g. Value Slider change)
}

/// A single screen reader spoken cue.
#[derive(Debug, Clone, PartialEq)]
pub struct NarrationCue {
    pub text: String,
    pub priority: NarrationPriority,
    pub timestamp_ms: u64,
}

impl NarrationCue {
    pub fn new(text: impl Into<String>, priority: NarrationPriority, timestamp_ms: u64) -> Self {
        Self {
            text: text.into(),
            priority,
            timestamp_ms,
        }
    }
}

/// Focusable accessible element metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct FocusableElement {
    pub id: String,
    pub label: String,
    pub role: AccessibleRole,
    pub value_text: String,
    pub bounds: Rect,
    pub tab_index: usize,
    pub help_text: String,
}

impl FocusableElement {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        role: AccessibleRole,
        value_text: impl Into<String>,
        bounds: Rect,
        tab_index: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            role,
            value_text: value_text.into(),
            bounds: bounds.enforce_min_hit_target(MIN_HIT_TARGET_PT),
            tab_index,
            help_text: String::new(),
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help_text = help.into();
        self
    }

    /// Spoken description when element receives focus
    pub fn spoken_description(&self) -> String {
        if self.value_text.is_empty() {
            format!("{}, {}", self.label, self.role.role_name())
        } else {
            format!(
                "{}, {}, value: {}",
                self.label,
                self.role.role_name(),
                self.value_text
            )
        }
    }
}

/// Accessibility Announcer & Keyboard Focus Manager View (Step 1343).
#[derive(Debug, Clone)]
pub struct AccessibilityAnnouncerView {
    pub elements: Vec<FocusableElement>,
    pub focused_index: Option<usize>,
    pub narration_queue: Vec<NarrationCue>,
    pub screen_reader_enabled: bool,
    pub high_contrast_mode: bool,
    pub focus_ring_thickness: f32,
    pub focus_ring_offset: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for AccessibilityAnnouncerView {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityAnnouncerView {
    pub fn new() -> Self {
        let mut view = Self {
            elements: Vec::new(),
            focused_index: Some(0),
            narration_queue: Vec::new(),
            screen_reader_enabled: true,
            high_contrast_mode: true,
            focus_ring_thickness: DEFAULT_RING_THICKNESS,
            focus_ring_offset: DEFAULT_RING_OFFSET,
            color_palette: ContrastColorPalette::default(),
        };

        // Initialize with default standard focusable controls
        view.elements.push(FocusableElement::new(
            "play_btn",
            "Play / Pause",
            AccessibleRole::Button,
            "Playing",
            Rect::new(20.0, 60.0, 80.0, 44.0),
            0,
        ));
        view.elements.push(FocusableElement::new(
            "tempo_slider",
            "Project BPM Tempo",
            AccessibleRole::Slider,
            "128.0 BPM",
            Rect::new(110.0, 60.0, 140.0, 44.0),
            1,
        ));
        view.elements.push(FocusableElement::new(
            "master_fader",
            "Master Volume Fader",
            AccessibleRole::Slider,
            "-0.5 dBFS",
            Rect::new(260.0, 60.0, 160.0, 44.0),
            2,
        ));
        view.elements.push(FocusableElement::new(
            "filter_cutoff",
            "SVF Filter Cutoff Frequency",
            AccessibleRole::Slider,
            "2.4 kHz (72%)",
            Rect::new(430.0, 60.0, 180.0, 44.0),
            3,
        ));

        // Initial welcome cue
        view.narration_queue.push(NarrationCue::new(
            "Summoner DAW Accessibility System active. Screen reader narration online.",
            NarrationPriority::Polite,
            0,
        ));

        view
    }

    /// Register a new focusable element
    pub fn register_element(&mut self, element: FocusableElement) {
        self.elements.push(element);
    }

    /// Advance keyboard focus to next item (Tab)
    pub fn focus_next(&mut self) -> Option<String> {
        if self.elements.is_empty() {
            return None;
        }
        let next_idx = match self.focused_index {
            Some(curr) => (curr + 1) % self.elements.len(),
            None => 0,
        };
        self.focused_index = Some(next_idx);
        let desc = self.elements[next_idx].spoken_description();
        self.queue_narration(&desc, NarrationPriority::Polite, 0);
        Some(self.elements[next_idx].id.clone())
    }

    /// Advance keyboard focus to previous item (Shift+Tab)
    pub fn focus_prev(&mut self) -> Option<String> {
        if self.elements.is_empty() {
            return None;
        }
        let prev_idx = match self.focused_index {
            Some(0) | None => self.elements.len() - 1,
            Some(curr) => curr - 1,
        };
        self.focused_index = Some(prev_idx);
        let desc = self.elements[prev_idx].spoken_description();
        self.queue_narration(&desc, NarrationPriority::Polite, 0);
        Some(self.elements[prev_idx].id.clone())
    }

    /// Queue a narration cue
    pub fn queue_narration(
        &mut self,
        text: impl Into<String>,
        priority: NarrationPriority,
        timestamp_ms: u64,
    ) {
        let cue = NarrationCue::new(text, priority, timestamp_ms);
        if priority == NarrationPriority::Assertive {
            self.narration_queue.insert(0, cue);
        } else {
            self.narration_queue.push(cue);
        }
    }

    /// Calculate focus ring bounding box enclosing element
    pub fn calculate_focus_ring_rect(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.x - self.focus_ring_offset,
            bounds.y - self.focus_ring_offset,
            bounds.width + self.focus_ring_offset * 2.0,
            bounds.height + self.focus_ring_offset * 2.0,
        )
    }

    /// Currently focused element reference
    pub fn current_focused_element(&self) -> Option<&FocusableElement> {
        self.focused_index.and_then(|idx| self.elements.get(idx))
    }

    /// Render deterministic ASCII representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[ACCESSIBILITY SCREEN READER & FOCUS RING]\n");
        out.push_str(&format!(
            "Screen Reader: {} | High Contrast: {} | Registered Elements: {}\n",
            if self.screen_reader_enabled {
                "ON"
            } else {
                "OFF"
            },
            if self.high_contrast_mode { "ON" } else { "OFF" },
            self.elements.len()
        ));

        if let Some(focused) = self.current_focused_element() {
            let ring = self.calculate_focus_ring_rect(focused.bounds);
            out.push_str(&format!(
                "Active Focus: [{}] '{}' | Value: '{}' | Focus Ring: ({:.0}, {:.0}, {:.0}x{:.0})\n",
                focused.role.role_name(),
                focused.label,
                focused.value_text,
                ring.x,
                ring.y,
                ring.width,
                ring.height
            ));
        }

        out.push_str("Narration Speech Queue:\n");
        for (i, cue) in self.narration_queue.iter().enumerate() {
            out.push_str(&format!(
                "  #{}: [{:?}] {}\n",
                i + 1,
                cue.priority,
                cue.text
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl AccessibilityAnnouncerView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("ACCESSIBILITY SCREEN READER & FOCUS NAVIGATION");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hc_label = if self.high_contrast_mode {
                        "WCAG AAA (7:1): ON"
                    } else {
                        "WCAG AA (4.5:1)"
                    };
                    let hc_btn =
                        egui::Button::new(egui::RichText::new(hc_label).size(13.0).strong().color(
                            if self.high_contrast_mode {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            },
                        ))
                        .min_size(Vec2::new(MIN_HIT_TARGET_PT * 3.0, MIN_HIT_TARGET_PT))
                        .fill(if self.high_contrast_mode {
                            Color32::from_rgb(0, 255, 255)
                        } else {
                            Color32::from_rgb(35, 50, 75)
                        });

                    if ui.add(hc_btn).clicked() {
                        self.high_contrast_mode = !self.high_contrast_mode;
                    }
                });
            });

            ui.add_space(8.0);

            // Active Focused Element Overview Card
            if let Some(focused) = self.current_focused_element() {
                egui::Frame::none()
                    .fill(Color32::from_rgb(16, 25, 42))
                    .stroke(Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 255)))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "KEYBOARD FOCUSED: {} [{}]",
                                        focused.label,
                                        focused.role.role_name()
                                    ))
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(0, 255, 255)),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Current Value: {}",
                                        focused.value_text
                                    ))
                                    .size(13.0)
                                    .color(Color32::from_rgb(255, 215, 0)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Tab Index: #{}",
                                            focused.tab_index + 1
                                        ))
                                        .size(12.0)
                                        .color(Color32::from_rgb(180, 205, 235)),
                                    );
                                },
                            );
                        });
                    });
            }

            ui.add_space(10.0);

            // Tab Navigation Navigation Controls Bar
            ui.horizontal(|ui| {
                let prev_btn = egui::Button::new(
                    egui::RichText::new("< Shift+Tab (Prev)")
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(220, 235, 255)),
                )
                .min_size(Vec2::new(MIN_HIT_TARGET_PT * 3.5, MIN_HIT_TARGET_PT))
                .fill(Color32::from_rgb(35, 48, 72));

                if ui.add(prev_btn).clicked() {
                    self.focus_prev();
                }

                ui.add_space(8.0);

                let next_btn = egui::Button::new(
                    egui::RichText::new("Tab (Next) >")
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(220, 235, 255)),
                )
                .min_size(Vec2::new(MIN_HIT_TARGET_PT * 3.5, MIN_HIT_TARGET_PT))
                .fill(Color32::from_rgb(35, 48, 72));

                if ui.add(next_btn).clicked() {
                    self.focus_next();
                }
            });

            ui.add_space(12.0);

            // Screen Reader Narration Cues Feed
            ui.label(
                egui::RichText::new("LIVE SCREEN READER SPEECH CUES:")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(200, 220, 245)),
            );

            for cue in self.narration_queue.iter().rev().take(6) {
                let priority_color = match cue.priority {
                    NarrationPriority::Assertive => Color32::from_rgb(255, 107, 43),
                    NarrationPriority::Polite => Color32::from_rgb(0, 229, 255),
                };

                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 26, 40))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("[{:?}]", cue.priority))
                                    .size(11.0)
                                    .strong()
                                    .color(priority_color),
                            );
                            ui.label(
                                egui::RichText::new(&cue.text)
                                    .size(12.0)
                                    .color(Color32::from_rgb(240, 245, 255)),
                            );
                        });
                    });

                ui.add_space(4.0);
            }
        })
        .response
    }
}
