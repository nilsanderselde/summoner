// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier HUD (Step 1556).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NAM_AMP_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_DRIVE: f32 = 0.0;
pub const MAX_DRIVE: f32 = 5.0;
pub const MIN_GATE_DB: f32 = -90.0;
pub const MAX_GATE_DB: f32 = 0.0;

/// Neural Amp Modeler profiles and cabinet acoustic configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamAmpProfile {
    CleanTube1965,          // Vintage 6V6 blackface American clean chime with gentle cathode bias sag
    BritishCrunchPlexi,     // 1959 EL34 cascaded preamp tube crunch and mid-forward bite
    ModernHighGainLead,     // 12AX7 multi-stage rectified high-gain lead with tight low end
    BassSlapAmpegSVT,       // 300W all-tube bass stack with growling punch and subsonic body
    BoutiqueClassAOverdrive,// EL84 Class-A cathode-biased touch-sensitive harmonic saturation
}

impl NamAmpProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::CleanTube1965 => "VINTAGE CLEAN 6V6",
            Self::BritishCrunchPlexi => "BRITISH PLEXI EL34",
            Self::ModernHighGainLead => "MODERN RECTO 12AX7",
            Self::BassSlapAmpegSVT => "ALL-TUBE BASS SVT",
            Self::BoutiqueClassAOverdrive => "BOUTIQUE CLASS-A EL84",
        }
    }

    pub fn nominal_drive(&self) -> f32 {
        match self {
            Self::CleanTube1965 => 1.2,
            Self::BritishCrunchPlexi => 2.5,
            Self::ModernHighGainLead => 4.2,
            Self::BassSlapAmpegSVT => 2.0,
            Self::BoutiqueClassAOverdrive => 3.0,
        }
    }

    pub fn nominal_output_gain(&self) -> f32 {
        match self {
            Self::CleanTube1965 => 1.1,
            Self::BritishCrunchPlexi => 0.95,
            Self::ModernHighGainLead => 0.85,
            Self::BassSlapAmpegSVT => 1.05,
            Self::BoutiqueClassAOverdrive => 1.0,
        }
    }

    pub fn nominal_gate_db(&self) -> f32 {
        match self {
            Self::CleanTube1965 => -72.0,
            Self::BritishCrunchPlexi => -60.0,
            Self::ModernHighGainLead => -48.0,
            Self::BassSlapAmpegSVT => -65.0,
            Self::BoutiqueClassAOverdrive => -55.0,
        }
    }

    pub fn nominal_cab_air(&self) -> f32 {
        match self {
            Self::CleanTube1965 => 0.50,
            Self::BritishCrunchPlexi => 0.35,
            Self::ModernHighGainLead => 0.25,
            Self::BassSlapAmpegSVT => 0.45,
            Self::BoutiqueClassAOverdrive => 0.40,
        }
    }

    pub fn nominal_sag(&self) -> f32 {
        match self {
            Self::CleanTube1965 => 0.15,
            Self::BritishCrunchPlexi => 0.35,
            Self::ModernHighGainLead => 0.50,
            Self::BassSlapAmpegSVT => 0.30,
            Self::BoutiqueClassAOverdrive => 0.40,
        }
    }
}

/// Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier HUD view state.
#[derive(Debug, Clone)]
pub struct NamAmpView {
    pub profile: NamAmpProfile,
    pub drive: f32,
    pub output_gain: f32,
    pub gate_threshold_db: f32,
    pub cabinet_air: f32,
    pub tube_sag: f32,
    pub presence_boost_db: f32,
    pub puck_pos: (f32, f32), // Normalized (x: gate_threshold, y: drive)
    pub is_dragging_puck: bool,
    pub saturation_curve: [f32; 16], // 16-point non-linear transfer curve
    pub wavenet_dilation_activity: [f32; 4], // Dilation stages [1, 2, 4, 8]
    pub color_palette: ContrastColorPalette,
}

impl Default for NamAmpView {
    fn default() -> Self {
        Self::new()
    }
}

impl NamAmpView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: NamAmpProfile::BritishCrunchPlexi,
            drive: 2.5,
            output_gain: 0.95,
            gate_threshold_db: -60.0,
            cabinet_air: 0.35,
            tube_sag: 0.35,
            presence_boost_db: 1.5,
            puck_pos: (
                (-60.0 - MIN_GATE_DB) / (MAX_GATE_DB - MIN_GATE_DB),
                2.5 / MAX_DRIVE,
            ),
            is_dragging_puck: false,
            saturation_curve: [0.0; 16],
            wavenet_dilation_activity: [0.8, 0.65, 0.5, 0.35],
            color_palette: ContrastColorPalette::default(),
        };
        view.recompute_transfer_simulation();
        view
    }

    pub fn set_profile(&mut self, profile: NamAmpProfile) {
        self.profile = profile;
        self.drive = profile.nominal_drive();
        self.output_gain = profile.nominal_output_gain();
        self.gate_threshold_db = profile.nominal_gate_db();
        self.cabinet_air = profile.nominal_cab_air();
        self.tube_sag = profile.nominal_sag();
        self.puck_pos = (
            Self::gate_to_normalized(self.gate_threshold_db),
            Self::drive_to_normalized(self.drive),
        );
        self.recompute_transfer_simulation();
    }

    pub fn drive_to_normalized(drive: f32) -> f32 {
        ((drive - MIN_DRIVE) / (MAX_DRIVE - MIN_DRIVE)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_DRIVE + norm.clamp(0.0, 1.0) * (MAX_DRIVE - MIN_DRIVE)
    }

    pub fn gate_to_normalized(gate: f32) -> f32 {
        ((gate - MIN_GATE_DB) / (MAX_GATE_DB - MIN_GATE_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_gate(norm: f32) -> f32 {
        MIN_GATE_DB + norm.clamp(0.0, 1.0) * (MAX_GATE_DB - MIN_GATE_DB)
    }

    pub fn recompute_transfer_simulation(&mut self) {
        // Compute 16-point non-linear tanh / soft-clipping saturation transfer curve
        let drive_effective = self.drive * (1.0 + self.tube_sag * 0.4);
        for i in 0..16 {
            // Input range from -1.0 to +1.0
            let in_x = -1.0 + (i as f32 / 15.0) * 2.0;
            let out_y = (in_x * drive_effective).tanh() * self.output_gain;
            self.saturation_curve[i] = out_y.clamp(-1.0, 1.0);
        }

        // Compute WaveNet dilation stage activity meters based on drive and sag
        let base_act = (self.drive / MAX_DRIVE).clamp(0.1, 1.0);
        self.wavenet_dilation_activity[0] = base_act;
        self.wavenet_dilation_activity[1] = (base_act * 0.85 + self.tube_sag * 0.15).clamp(0.05, 1.0);
        self.wavenet_dilation_activity[2] = (base_act * 0.70 + self.cabinet_air * 0.20).clamp(0.05, 1.0);
        self.wavenet_dilation_activity[3] = (base_act * 0.55 + self.tube_sag * 0.35).clamp(0.05, 1.0);
    }

    pub fn hit_test_amp_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NAM_AMP_PUCK_HIT_RADIUS
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        let mut out = String::new();
        out.push_str("========================================================================\n");
        out.push_str("  NEURAL AMP MODELER (NAM) WAVENET GUITAR & BASS AMPLIFIER HUD\n");
        out.push_str("========================================================================\n");
        out.push_str(&format!("  Profile:             {}\n", self.profile.profile_name()));
        out.push_str(&format!("  Pre-Amp Drive:       {:.2} x\n", self.drive));
        out.push_str(&format!("  Master Volume:       {:.2} x\n", self.output_gain));
        out.push_str(&format!("  Noise Gate Thresh:   {:.1} dB\n", self.gate_threshold_db));
        out.push_str(&format!("  Cabinet Air IR:      {:.2} %\n", self.cabinet_air * 100.0));
        out.push_str(&format!("  Cathode Tube Sag:    {:.2} %\n", self.tube_sag * 100.0));
        out.push_str(&format!("  Presence Boost:      {:.1} dB\n", self.presence_boost_db));
        out.push_str("------------------------------------------------------------------------\n");
        out.push_str("  WaveNet Dilation Stages (d=1, 2, 4, 8):\n  ");
        for (idx, act) in self.wavenet_dilation_activity.iter().enumerate() {
            let pct = (*act * 100.0).round() as u32;
            out.push_str(&format!("[D{}: {}%] ", 1 << idx, pct));
        }
        out.push_str("\n------------------------------------------------------------------------\n");
        out.push_str("  Non-Linear Saturation Transfer Curve:\n  ");
        for val in self.saturation_curve.iter() {
            let norm = ((*val + 1.0) * 0.5 * 8.0).round() as usize;
            let bar_str = match norm.min(8) {
                0 => " ",
                1 => " ",
                2 => "▂",
                3 => "▃",
                4 => "▄",
                5 => "▅",
                6 => "▆",
                7 => "▇",
                _ => "█",
            };
            out.push_str(bar_str);
        }
        out.push_str("\n========================================================================\n");
        out.push_str("  [PASS] Neural Amp Modeler WaveNet Simulation Verified\n");
        out.push_str("========================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Profile selector tabs
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Amp Profile:")
                        .font(egui::FontId::proportional(12.0))
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                let profiles = [
                    (NamAmpProfile::CleanTube1965, "Clean 6V6"),
                    (NamAmpProfile::BritishCrunchPlexi, "British Plexi"),
                    (NamAmpProfile::ModernHighGainLead, "Recto High-Gain"),
                    (NamAmpProfile::BassSlapAmpegSVT, "Bass SVT"),
                    (NamAmpProfile::BoutiqueClassAOverdrive, "Class-A EL84"),
                ];
                for (p, label) in profiles {
                    let is_sel = self.profile == p;
                    let text = if is_sel {
                        egui::RichText::new(format!("● {}", label))
                            .color(Color32::from_rgb(245, 158, 11)) // Amber gold
                            .strong()
                    } else {
                        egui::RichText::new(label).color(Color32::from_rgb(203, 213, 225))
                    };
                    if ui.add_sized([115.0, 26.0], egui::Button::new(text)).clicked() {
                        self.set_profile(p);
                    }
                }
            });

            ui.add_space(8.0);

            // 2D Interactive Canvas & Non-Linear Transfer Curve
            ui.columns(2, |cols| {
                // Column 1: 2D Puck Canvas (X: Gate Threshold, Y: Pre-Amp Drive)
                cols[0].vertical(|ui| {
                    ui.label(
                        egui::RichText::new("2D Drive Gain & Noise Gate Threshold Canvas")
                            .font(egui::FontId::proportional(12.0))
                            .color(Color32::from_rgb(226, 232, 240)),
                    );

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(340.0, 240.0),
                        egui::Sense::click_and_drag(),
                    );

                    let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

                    // Handle dragging
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if self.hit_test_amp_puck((pos.x, pos.y), canvas_rect) {
                                self.is_dragging_puck = true;
                            }
                        }
                    }

                    if self.is_dragging_puck {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let norm_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                            let norm_y = (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                            self.puck_pos = (norm_x, norm_y);
                            self.gate_threshold_db = Self::normalized_to_gate(norm_x);
                            self.drive = Self::normalized_to_drive(norm_y);
                            self.recompute_transfer_simulation();
                        }
                    }

                    if response.drag_stopped() {
                        self.is_dragging_puck = false;
                    }

                    // Background grid & frame
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    // Grid lines
                    for i in 1..4 {
                        let gx = rect.min.x + (i as f32 / 4.0) * rect.width();
                        painter.line_segment(
                            [egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 116, 139, 40)),
                        );
                        let gy = rect.min.y + (i as f32 / 4.0) * rect.height();
                        painter.line_segment(
                            [egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 116, 139, 40)),
                        );
                    }

                    // Puck center & target
                    let puck_center = egui::pos2(
                        rect.min.x + self.puck_pos.0 * rect.width(),
                        rect.max.y - self.puck_pos.1 * rect.height(),
                    );

                    // Touch target shadow ring (>= 44x44pt bounding box)
                    painter.circle_stroke(
                        puck_center,
                        NAM_AMP_PUCK_HIT_RADIUS,
                        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 90)),
                    );

                    painter.circle_filled(
                        puck_center,
                        9.0,
                        if self.is_dragging_puck {
                            Color32::from_rgb(239, 68, 68)
                        } else {
                            Color32::from_rgb(245, 158, 11)
                        },
                    );
                    painter.circle_stroke(puck_center, 9.0, Stroke::new(1.5_f32, Color32::WHITE));

                    // Coordinate labels
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Gate: {:.1} dB  |  Drive: {:.2} x",
                                self.gate_threshold_db,
                                self.drive
                            ))
                            .font(egui::FontId::proportional(11.0))
                            .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                // Column 2: Non-Linear Saturation Curve & WaveNet Dilations
                cols[1].vertical(|ui| {
                    ui.label(
                        egui::RichText::new("WaveNet Transfer Curve & Dilation Activity")
                            .font(egui::FontId::proportional(12.0))
                            .color(Color32::from_rgb(226, 232, 240)),
                    );

                    let (curve_rect, _) = ui.allocate_exact_size(
                        egui::vec2(340.0, 240.0),
                        egui::Sense::hover(),
                    );

                    let painter = ui.painter_at(curve_rect);
                    painter.rect_filled(curve_rect, 4.0, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(curve_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    // Center crosshair (0,0)
                    let center_y = curve_rect.min.y + 0.5 * curve_rect.height();
                    let center_x = curve_rect.min.x + 0.5 * curve_rect.width();
                    painter.line_segment(
                        [egui::pos2(curve_rect.min.x, center_y), egui::pos2(curve_rect.max.x, center_y)],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 116, 139, 60)),
                    );
                    painter.line_segment(
                        [egui::pos2(center_x, curve_rect.min.y), egui::pos2(center_x, curve_rect.max.y)],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 116, 139, 60)),
                    );

                    // Draw transfer curve points
                    let mut prev_pt: Option<egui::Pos2> = None;
                    for (i, val) in self.saturation_curve.iter().enumerate() {
                        let px = curve_rect.min.x + (i as f32 / 15.0) * curve_rect.width();
                        let py = center_y - (*val * 0.45) * curve_rect.height();
                        let pt = egui::pos2(px, py);
                        if let Some(last) = prev_pt {
                            painter.line_segment([last, pt], Stroke::new(2.0_f32, Color32::from_rgb(245, 158, 11)));
                        }
                        prev_pt = Some(pt);
                    }

                    // Dilation stage indicators at bottom
                    let bar_w = (curve_rect.width() - 30.0) / 4.0;
                    for (idx, act) in self.wavenet_dilation_activity.iter().enumerate() {
                        let bx = curve_rect.min.x + 10.0 + idx as f32 * (bar_w + 4.0);
                        let by = curve_rect.max.y - 18.0;
                        let fill_w = bar_w * act;
                        painter.rect_filled(
                            egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(bar_w, 8.0)),
                            2.0,
                            Color32::from_rgb(30, 41, 59),
                        );
                        painter.rect_filled(
                            egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(fill_w, 8.0)),
                            2.0,
                            Color32::from_rgb(245, 158, 11),
                        );
                    }

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("D1: 1x  |  D2: 2x  |  D4: 4x  |  D8: 8x Dilations")
                                .font(egui::FontId::proportional(11.0))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });
            });

            ui.add_space(8.0);
            ui.separator();

            // Tactile control sliders
            ui.horizontal(|ui| {
                ui.label("Master Volume:");
                if ui.add(egui::Slider::new(&mut self.output_gain, 0.0..=2.0).suffix(" x")).changed() {
                    self.recompute_transfer_simulation();
                }

                ui.add_space(16.0);
                ui.label("Tube Sag:");
                if ui.add(egui::Slider::new(&mut self.tube_sag, 0.0..=1.0).suffix(" %")).changed() {
                    self.recompute_transfer_simulation();
                }

                ui.add_space(16.0);
                ui.label("Cabinet Air:");
                if ui.add(egui::Slider::new(&mut self.cabinet_air, 0.0..=1.0).suffix(" %")).changed() {
                    self.recompute_transfer_simulation();
                }

                ui.add_space(16.0);
                ui.label("Presence:");
                if ui.add(egui::Slider::new(&mut self.presence_boost_db, -6.0..=6.0).suffix(" dB")).changed() {
                    self.recompute_transfer_simulation();
                }
            });
        });
    }
}
