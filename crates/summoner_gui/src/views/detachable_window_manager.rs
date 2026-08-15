// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Monitor Detachable Floating Window Management (Step 1342).

use crate::layout_math::{OperatingSystem, Rect};
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SNAP_EDGE_THRESHOLD: f32 = 16.0;

/// View types capable of being detached into floating multi-monitor windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetachableWindowType {
    MixerConsole,
    Spectrogram3D,
    NodeGraph,
    MacroRack,
    ArrangerTimeline,
    MeterBridge,
}

impl DetachableWindowType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MixerConsole => "Multi-Channel Mixer Console",
            Self::Spectrogram3D => "3D Waterfall Spectrogram",
            Self::NodeGraph => "Modular DSP Node Graph",
            Self::MacroRack => "Live Performance Macro Rack",
            Self::ArrangerTimeline => "Arranger & Automation Timeline",
            Self::MeterBridge => "Peak Metering Bridge",
        }
    }
}

/// Description of a connected physical monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub id: usize,
    pub name: String,
    pub bounds: Rect,
    pub dpi_scale: f32,
    pub is_primary: bool,
}

impl MonitorInfo {
    pub fn new(
        id: usize,
        name: impl Into<String>,
        bounds: Rect,
        dpi_scale: f32,
        is_primary: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            bounds,
            dpi_scale,
            is_primary,
        }
    }
}

/// State representation for an active detached floating window.
#[derive(Debug, Clone, PartialEq)]
pub struct DetachableWindowState {
    pub id: String,
    pub title: String,
    pub view_type: DetachableWindowType,
    pub is_detached: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub target_monitor_id: usize,
    pub window_bounds: Rect,
    pub min_size: (f32, f32),
}

impl DetachableWindowState {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        view_type: DetachableWindowType,
        initial_bounds: Rect,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            view_type,
            is_detached: true,
            is_minimized: false,
            is_maximized: false,
            target_monitor_id: 0,
            window_bounds: initial_bounds,
            min_size: (400.0, 300.0),
        }
    }

    /// Snaps window to edges of monitor if within threshold.
    pub fn apply_edge_snap(&mut self, monitor_bounds: Rect, threshold: f32) -> bool {
        let mut snapped = false;

        // Snap Left
        if (self.window_bounds.x - monitor_bounds.x).abs() <= threshold {
            self.window_bounds.x = monitor_bounds.x;
            snapped = true;
        }

        // Snap Top
        if (self.window_bounds.y - monitor_bounds.y).abs() <= threshold {
            self.window_bounds.y = monitor_bounds.y;
            snapped = true;
        }

        // Snap Right
        let right_diff = (self.window_bounds.max_x() - monitor_bounds.max_x()).abs();
        if right_diff <= threshold {
            self.window_bounds.x = monitor_bounds.max_x() - self.window_bounds.width;
            snapped = true;
        }

        // Snap Bottom
        let bottom_diff = (self.window_bounds.max_y() - monitor_bounds.max_y()).abs();
        if bottom_diff <= threshold {
            self.window_bounds.y = monitor_bounds.max_y() - self.window_bounds.height;
            snapped = true;
        }

        snapped
    }
}

/// Multi-Monitor Detachable Floating Window Manager View (Step 1342).
#[derive(Debug, Clone)]
pub struct DetachableWindowManagerView {
    pub os: OperatingSystem,
    pub monitors: Vec<MonitorInfo>,
    pub floating_windows: Vec<DetachableWindowState>,
    pub active_drag_window_id: Option<String>,
    pub color_palette: ContrastColorPalette,
}

impl DetachableWindowManagerView {
    pub fn new(os: OperatingSystem) -> Self {
        let mut view = Self {
            os,
            monitors: Vec::new(),
            floating_windows: Vec::new(),
            active_drag_window_id: None,
            color_palette: ContrastColorPalette::default(),
        };

        // Standard dual-monitor setup defaults
        view.monitors.push(MonitorInfo::new(
            0,
            "Primary Display (4K)",
            Rect::new(0.0, 0.0, 3840.0, 2160.0),
            if os == OperatingSystem::Windows {
                1.5
            } else {
                2.0
            },
            true,
        ));
        view.monitors.push(MonitorInfo::new(
            1,
            "Secondary Display (FHD)",
            Rect::new(3840.0, 0.0, 1920.0, 1080.0),
            1.0,
            false,
        ));

        // Initial default detached windows
        view.floating_windows.push(DetachableWindowState::new(
            "win_mixer",
            "Master Mixer Console",
            DetachableWindowType::MixerConsole,
            Rect::new(3880.0, 40.0, 1840.0, 1000.0),
        ));
        view.floating_windows.push(DetachableWindowState::new(
            "win_spectrogram",
            "3D Waterfall Spectrogram",
            DetachableWindowType::Spectrogram3D,
            Rect::new(100.0, 100.0, 960.0, 640.0),
        ));

        view
    }

    /// Detaches a window by type
    pub fn detach_view(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        view_type: DetachableWindowType,
    ) {
        let id_str = id.into();
        if !self.floating_windows.iter().any(|w| w.id == id_str) {
            let bounds = Rect::new(200.0, 200.0, 800.0, 500.0);
            self.floating_windows
                .push(DetachableWindowState::new(id_str, title, view_type, bounds));
        }
    }

    /// Re-attaches / docks a floating window back into main layout
    pub fn reattach_window(&mut self, id: &str) -> bool {
        if let Some(pos) = self.floating_windows.iter().position(|w| w.id == id) {
            self.floating_windows.remove(pos);
            true
        } else {
            false
        }
    }

    /// Calculate scaled bounds when transferring between monitors of differing DPI
    pub fn calculate_dpi_compensated_bounds(
        bounds: Rect,
        source_dpi: f32,
        target_dpi: f32,
    ) -> Rect {
        if source_dpi <= 0.0 || target_dpi <= 0.0 {
            return bounds;
        }
        let ratio = target_dpi / source_dpi;
        Rect::new(
            bounds.x,
            bounds.y,
            bounds.width * ratio,
            bounds.height * ratio,
        )
    }

    /// Render deterministic ASCII representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[DETACHABLE MULTI-MONITOR WINDOW MANAGER]\n");
        out.push_str(&format!(
            "OS Target: {:?} | Connected Monitors: {}\n",
            self.os,
            self.monitors.len()
        ));

        for mon in &self.monitors {
            out.push_str(&format!(
                "  Monitor #{}: {} | Bounds: ({:.0}, {:.0}, {:.0}x{:.0}) | Scale: {:.2}x | Primary: {}\n",
                mon.id, mon.name, mon.bounds.x, mon.bounds.y, mon.bounds.width, mon.bounds.height, mon.dpi_scale, mon.is_primary
            ));
        }

        out.push_str(&format!(
            "Detached Floating Windows ({}):\n",
            self.floating_windows.len()
        ));
        for win in &self.floating_windows {
            out.push_str(&format!(
                "  - [{}] '{}' ({}) | Bounds: ({:.0}, {:.0}, {:.0}x{:.0}) | Mon: #{}\n",
                win.id,
                win.title,
                win.view_type.display_name(),
                win.window_bounds.x,
                win.window_bounds.y,
                win.window_bounds.width,
                win.window_bounds.height,
                win.target_monitor_id
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl DetachableWindowManagerView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("MULTI-MONITOR DETACHABLE WINDOW MANAGER");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Edge Snapping: {}pt", SNAP_EDGE_THRESHOLD))
                            .size(12.0)
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                });
            });

            ui.add_space(8.0);

            // Connected Monitors Overview Card
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 26, 40))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)))
                .rounding(6.0)
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("DETECTED DISPLAY TOPOLOGY:")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(200, 220, 245)),
                    );
                    ui.horizontal(|ui| {
                        for mon in &self.monitors {
                            let mon_bg = if mon.is_primary {
                                Color32::from_rgb(30, 48, 75)
                            } else {
                                Color32::from_rgb(25, 34, 52)
                            };
                            egui::Frame::none()
                                .fill(mon_bg)
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 85, 120)))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(&mon.name)
                                                .size(12.0)
                                                .strong()
                                                .color(Color32::from_rgb(240, 245, 255)),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:.0}x{:.0} @ {:.2}x DPI",
                                                mon.bounds.width, mon.bounds.height, mon.dpi_scale
                                            ))
                                            .size(11.0)
                                            .color(Color32::from_rgb(0, 229, 255)),
                                        );
                                    });
                                });
                            ui.add_space(10.0);
                        }
                    });
                });

            ui.add_space(12.0);

            // Floating Window Table
            ui.label(
                egui::RichText::new("ACTIVE FLOATING DETACHED WINDOWS:")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(200, 220, 245)),
            );

            let mut win_to_reattach = None;

            for win in &mut self.floating_windows {
                egui::Frame::none()
                    .fill(Color32::from_rgb(22, 28, 44))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&win.title)
                                        .size(14.0)
                                        .strong()
                                        .color(Color32::from_rgb(240, 245, 255)),
                                );
                                ui.label(
                                    egui::RichText::new(win.view_type.display_name())
                                        .size(11.0)
                                        .color(Color32::from_rgb(140, 170, 210)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Reattach button (>=44x44pt)
                                    let reattach_btn = egui::Button::new(
                                        egui::RichText::new("Re-Attach to Main Dock")
                                            .size(12.0)
                                            .strong()
                                            .color(Color32::BLACK),
                                    )
                                    .min_size(Vec2::new(MIN_HIT_TARGET_PT * 3.5, MIN_HIT_TARGET_PT))
                                    .fill(Color32::from_rgb(0, 229, 255));

                                    if ui.add(reattach_btn).clicked() {
                                        win_to_reattach = Some(win.id.clone());
                                    }

                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Bounds: {:.0}x{:.0} at ({:.0}, {:.0})",
                                            win.window_bounds.width,
                                            win.window_bounds.height,
                                            win.window_bounds.x,
                                            win.window_bounds.y
                                        ))
                                        .size(11.0)
                                        .color(Color32::from_rgb(180, 200, 225)),
                                    );
                                },
                            );
                        });
                    });

                ui.add_space(8.0);
            }

            if let Some(id) = win_to_reattach {
                self.reattach_window(&id);
            }
        })
        .response
    }
}
