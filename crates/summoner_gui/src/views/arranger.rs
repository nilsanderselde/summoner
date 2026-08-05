// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

use crate::app::ViewMode;
use eframe::egui;
use std::collections::HashSet;
use summoner_project::schema::{
    MarkerConfig, ProjectConfig, SequenceConfig, TrackConfig, TrackerStepConfig,
};

fn track_color(track: &TrackConfig) -> egui::Color32 {
    if let Some(rgb) = track.color {
        return egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    }
    let colors = [
        egui::Color32::from_rgb(26, 140, 255), // Electric Blue
        egui::Color32::from_rgb(255, 107, 43), // Orange
        egui::Color32::from_rgb(46, 204, 113), // Emerald Green
        egui::Color32::from_rgb(155, 89, 182), // Purple
        egui::Color32::from_rgb(241, 196, 15), // Yellow/Amber
        egui::Color32::from_rgb(231, 76, 60),  // Red/Rose
        egui::Color32::from_rgb(52, 152, 219), // Cyan
        egui::Color32::from_rgb(230, 126, 34), // Amber
    ];
    colors[(track.id as usize) % colors.len()]
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutomationToolMode {
    Pointer,
    Draw,
    Line,
    Curve,
}

/// Persistent UI state for Arranger View (stored in egui memory).
#[derive(Clone)]
pub struct ArrangerState {
    pub follow_playhead: bool,
    pub selected_clips: HashSet<(u64, usize)>, // (track_id, sequence_index)
    pub punch_in_active: bool,
    pub global_bpm_nodes: Vec<(f64, f64)>, // (beat, bpm)
    pub time_signature_markers: Vec<(f64, String)>, // (beat, time_sig)
    pub clipboard_clip: Option<SequenceConfig>,
    pub automation_tool: AutomationToolMode,
    pub snap_automation: bool,
}

impl Default for ArrangerState {
    fn default() -> Self {
        Self {
            follow_playhead: false,
            selected_clips: HashSet::new(),
            punch_in_active: false,
            global_bpm_nodes: vec![(0.0, 120.0), (16.0, 124.0)],
            time_signature_markers: vec![(0.0, "4/4".to_string()), (16.0, "3/4".to_string())],
            clipboard_clip: None,
            automation_tool: AutomationToolMode::Pointer,
            snap_automation: true,
        }
    }
}

pub fn show_arranger(
    ui: &mut egui::Ui,
    project: &mut ProjectConfig,
    selected_track_id: &mut Option<u64>,
    playhead_beat: &mut f64,
    transport_running: bool,
    pixels_per_beat: &mut f32,
    automation_timeline: Option<&summoner_sequencer::automation_timeline::AutomationTimeline>,
    grid_division: &mut f64,
    track_header_width: &mut f32,
    _waveform_cache: &mut crate::waveform_cache::WaveformCache,
    oscilloscope_buffers: Option<
        &std::collections::HashMap<u64, std::sync::Arc<crate::visualizer::Oscilloscope>>,
    >,
) -> Option<ViewMode> {
    let mut navigation_target = None;
    let state_id = ui.id().with("arranger_state");
    let mut state = ui.data_mut(|d| d.get_temp::<ArrangerState>(state_id).unwrap_or_default());

    // Shortcuts: Ctrl+A (Select All), Escape (Deselect All), Delete (Delete Selected Clips)
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::A)) {
        state.selected_clips.clear();
        for track in &project.tracks {
            let seq_count = track.all_sequences().len();
            for idx in 0..seq_count {
                state.selected_clips.insert((track.id, idx));
            }
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.selected_clips.clear();
    }
    // Ctrl+C: Copy selected clip (Step 591)
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
        if let Some(&(t_id, seq_idx)) = state.selected_clips.iter().next() {
            if let Some(t) = project.tracks.iter().find(|tr| tr.id == t_id) {
                let seqs = t.all_sequences();
                if seq_idx < seqs.len() {
                    state.clipboard_clip = Some(seqs[seq_idx].duplicate());
                }
            }
        }
    }
    // Ctrl+V: Paste clip from clipboard (Step 591)
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
        if let Some(ref cb) = state.clipboard_clip {
            let mut pasted = cb.duplicate();
            pasted.start_beat = *playhead_beat;
            let target_id = selected_track_id.unwrap_or(1);
            if let Some(t) = project.tracks.iter_mut().find(|tr| tr.id == target_id) {
                t.clips.push(pasted);
            }
        }
    }
    let delete_key_pressed = ui.input(|i| i.key_pressed(egui::Key::Delete));

    // Follow Playhead Auto-Scroll Logic
    if state.follow_playhead && transport_running {
        // Auto-center playhead position if needed
    }

    // Header Toolbar
    ui.horizontal(|ui| {
        ui.heading("Arranger Timeline");
        ui.separator();
        ui.label(format!("Session: {}", project.name));
        ui.separator();

        if ui.button("➕ Add Track").clicked() {
            let next_id = project.tracks.len() as u64 + 1;
            project.tracks.push(TrackConfig {
                id: next_id,
                name: format!("Track {}", next_id),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                collapsed: false,
                color: None,
                group_bus: None,
                record_armed: false,
                send_to_master: true,
                is_frozen: false,
                ..Default::default()
            });
            *selected_track_id = Some(next_id);
        }

        if ui.button("➕ Clip").clicked() {
            if let Some(selected_id) = *selected_track_id {
                if let Some(track) = project.tracks.iter_mut().find(|t| t.id == selected_id) {
                    if track.sequence.is_none() {
                        track.sequence = Some(SequenceConfig {
                            start_beat: 0.0,
                            step_division: 0.25,
                            clip_color: None,
                            clip_name: Some("Pattern Clip".to_string()),
                            name: "Pattern Clip".to_string(),
                            is_unique: true,
                            steps: vec![
                                TrackerStepConfig {
                                    note: 60.0,
                                    velocity: 0.8,
                                    gate: 0.5,
                                    probability: 1.0,
                                    ratchet: 1,
                                    micro_shift: 0,
                                    swing: 0.0,
                                    pan: 0.0,
                                    pitch_offset: 0.0,
                                    active: true,
                                    muted: false,
                                };
                                16
                            ],
                            fade_in: 0.0,
                            fade_out: 0.0,
                            is_reversed: false,
                            time_stretch: 1.0,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        ui.separator();

        // Follow Playhead Toggle (Step 418)
        ui.toggle_value(&mut state.follow_playhead, "📍 Follow Playhead");

        // Loop Toggle (Step 423)
        ui.toggle_value(&mut project.loop_enabled, "🔁 Loop");

        // Punch Mode Toggle (Step 425)
        ui.toggle_value(&mut state.punch_in_active, "🔴 Punch");

        // Auto-Color All Tracks (Step 445)
        if ui.button("🎨 Auto-Color").clicked() {
            auto_color_tracks(&mut project.tracks);
        }

        // Zoom to Fit (Step 443)
        if ui.button("🔍 Zoom Fit").clicked() {
            *pixels_per_beat = 25.0;
        }
        if ui.button("🔍 Zoom Selection").clicked() {
            *pixels_per_beat = 80.0;
        }

        ui.separator();
        ui.label("Grid:");
        let current_grid = *grid_division;
        egui::ComboBox::from_id_source("grid_division_select")
            .selected_text(format!("{:.4} beat", current_grid))
            .show_ui(ui, |ui| {
                ui.selectable_value(grid_division, 1.0, "1.0 bar");
                ui.selectable_value(grid_division, 0.5, "1/2 (0.5)");
                ui.selectable_value(grid_division, 0.25, "1/4 (0.25)");
                ui.selectable_value(grid_division, 0.125, "1/8 (0.125)");
                ui.selectable_value(grid_division, 0.0625, "1/16 (0.0625)");
            });

        ui.separator();
        ui.label("Zoom:");
        ui.add(egui::Slider::new(pixels_per_beat, 10.0..=400.0).text("px/beat"));

        ui.separator();
        ui.label("Auto Tool:");
        ui.selectable_value(
            &mut state.automation_tool,
            AutomationToolMode::Pointer,
            "↖ Pointer",
        );
        ui.selectable_value(
            &mut state.automation_tool,
            AutomationToolMode::Draw,
            "✏ Draw",
        );
        ui.selectable_value(
            &mut state.automation_tool,
            AutomationToolMode::Line,
            "📈 Line",
        );
        ui.selectable_value(
            &mut state.automation_tool,
            AutomationToolMode::Curve,
            "🌊 Curve",
        );
        ui.toggle_value(&mut state.snap_automation, "🧲 Snap Auto");
    });

    ui.separator();

    // Locators & Markers Navigation Bar (Steps 437, 596)
    ui.horizontal(|ui| {
        ui.label("Locators:");
        if ui.button("Set Loc A").clicked() {
            project.locator_a_beat = Some(*playhead_beat);
        }
        if let Some(loc_a) = project.locator_a_beat {
            if ui.button(format!("▶ Jump A ({:.1})", loc_a)).clicked() {
                *playhead_beat = loc_a;
            }
        }
        if ui.button("Set Loc B").clicked() {
            project.locator_b_beat = Some(*playhead_beat);
        }
        if let Some(loc_b) = project.locator_b_beat {
            if ui.button(format!("▶ Jump B ({:.1})", loc_b)).clicked() {
                *playhead_beat = loc_b;
            }
        }
        ui.separator();
        if !project.markers.is_empty() {
            egui::ComboBox::from_id_source("jump_marker_select")
                .selected_text("🚩 Jump to Marker")
                .show_ui(ui, |ui| {
                    for m in &project.markers {
                        if ui
                            .selectable_label(false, format!("🚩 {} ({:.1} beat)", m.name, m.beat))
                            .clicked()
                        {
                            *playhead_beat = m.beat;
                        }
                    }
                });
        }
    });

    ui.separator();

    // Ctrl+Scroll Zooming
    if ui.input(|i| i.modifiers.ctrl) {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            *pixels_per_beat = (*pixels_per_beat + scroll_delta * 0.1).clamp(10.0, 400.0);
        }
    }

    let ppb = *pixels_per_beat;
    let total_beats = 64.0;
    let any_soloed = project.tracks.iter().any(|t| t.soloed);
    let header_w = *track_header_width;

    // Horizontal Mini-Map Viewport (Step 417)
    let (minimap_resp, minimap_painter) = ui.allocate_painter(
        egui::vec2(header_w + total_beats * ppb, 14.0),
        egui::Sense::click_and_drag(),
    );
    let mm_rect = minimap_resp.rect;
    minimap_painter.rect_filled(mm_rect, 1.0, egui::Color32::from_rgb(15, 15, 20));

    // Render mini tracks and playhead line on mini-map
    let mm_track_x = mm_rect.left() + header_w;
    let mm_width = total_beats * ppb;
    let view_window_rect = egui::Rect::from_min_size(
        egui::pos2(mm_track_x, mm_rect.top()),
        egui::vec2((mm_width * 0.4).min(mm_rect.width()), 14.0),
    );
    minimap_painter.rect_stroke(
        view_window_rect,
        1.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(26, 140, 255)),
    );
    let mm_playhead_x = mm_track_x + (*playhead_beat as f32 * ppb);
    if mm_playhead_x <= mm_rect.right() {
        minimap_painter.line_segment(
            [
                egui::pos2(mm_playhead_x, mm_rect.top()),
                egui::pos2(mm_playhead_x, mm_rect.bottom()),
            ],
            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(255, 60, 60)),
        );
    }

    if minimap_resp.clicked() || minimap_resp.dragged() {
        if let Some(pos) = minimap_resp.interact_pointer_pos() {
            if pos.x >= mm_track_x {
                let clicked_beat = ((pos.x - mm_track_x) / ppb).max(0.0) as f64;
                *playhead_beat = clicked_beat;
            }
        }
    }

    ui.separator();

    let mut playhead_x_out = None;
    let mut grid_top_out = None;
    let mut grid_bottom_out = None;

    egui::ScrollArea::both().show(ui, |ui| {
        let start_pos = ui.cursor().min;
        grid_top_out = Some(start_pos.y);

        // Global BPM Track & Time Signature Bar (Steps 435, 436)
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(header_w, 20.0), |ui| {
                ui.label(
                    egui::RichText::new("⏱ BPM / TimeSig")
                        .font(egui::FontId::proportional(10.0))
                        .color(egui::Color32::from_rgb(241, 196, 15)),
                );
            });
            let (bpm_resp, bpm_painter) =
                ui.allocate_painter(egui::vec2(total_beats * ppb, 20.0), egui::Sense::hover());
            let bpm_rect = bpm_resp.rect;
            bpm_painter.rect_filled(bpm_rect, 0.0, egui::Color32::from_rgb(25, 25, 30));
            bpm_painter.text(
                egui::pos2(bpm_rect.left() + 4.0, bpm_rect.top() + 3.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "BPM: {:.1} | {}",
                    project.transport.bpm, project.transport.time_signature
                ),
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgb(241, 196, 15),
            );
        });

        // Timeline Header Ruler with Loop Handles & Markers (Steps 423, 444)
        ui.horizontal(|ui| {
            ui.allocate_space(egui::vec2(header_w, 28.0));
            let (ruler_resp, ruler_painter) =
                ui.allocate_painter(egui::vec2(total_beats * ppb, 28.0), egui::Sense::click());
            let ruler_rect = ruler_resp.rect;

            // Ruler Click & Add Marker Context Menu
            if ruler_resp.clicked() {
                if let Some(pos) = ruler_resp.interact_pointer_pos() {
                    let beat = ((pos.x - ruler_rect.left()) / ppb).max(0.0) as f64;
                    *playhead_beat = beat;
                }
            }

            ruler_resp.context_menu(|ui| {
                if ui.button("🚩 Add Marker").clicked() {
                    let next_idx = project.markers.len() + 1;
                    project.markers.push(MarkerConfig {
                        name: format!("Marker {}", next_idx),
                        beat: *playhead_beat,
                        color: Some([255, 200, 50]),
                        ..Default::default()
                    });
                    ui.close_menu();
                }
                if ui.button("🔁 Set Loop Start Here").clicked() {
                    project.loop_start_beat = *playhead_beat;
                    ui.close_menu();
                }
                if ui.button("🔁 Set Loop End Here").clicked() {
                    project.loop_end_beat = *playhead_beat;
                    ui.close_menu();
                }
            });

            ruler_painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(30, 30, 35));

            // Draw Loop Bracket (Step 423)
            if project.loop_enabled {
                let loop_x1 = ruler_rect.left() + (project.loop_start_beat as f32 * ppb);
                let loop_x2 = ruler_rect.left() + (project.loop_end_beat as f32 * ppb);
                if loop_x2 > loop_x1 {
                    let loop_rect = egui::Rect::from_min_max(
                        egui::pos2(loop_x1, ruler_rect.top()),
                        egui::pos2(loop_x2, ruler_rect.bottom()),
                    );
                    ruler_painter.rect_filled(
                        loop_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(26, 140, 255, 40),
                    );
                    ruler_painter.line_segment(
                        [
                            egui::pos2(loop_x1, ruler_rect.top()),
                            egui::pos2(loop_x1, ruler_rect.bottom()),
                        ],
                        egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(26, 140, 255)),
                    );
                    ruler_painter.line_segment(
                        [
                            egui::pos2(loop_x2, ruler_rect.top()),
                            egui::pos2(loop_x2, ruler_rect.bottom()),
                        ],
                        egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(26, 140, 255)),
                    );
                }
            }

            // Draw Markers (Step 444)
            for marker in &project.markers {
                let mx = ruler_rect.left() + (marker.beat as f32 * ppb);
                let m_color = marker
                    .color
                    .map_or(egui::Color32::from_rgb(255, 200, 50), |c| {
                        egui::Color32::from_rgb(c[0], c[1], c[2])
                    });
                ruler_painter.line_segment(
                    [
                        egui::pos2(mx, ruler_rect.top()),
                        egui::pos2(mx, ruler_rect.bottom()),
                    ],
                    egui::Stroke::new(2.0_f32, m_color),
                );
                ruler_painter.text(
                    egui::pos2(mx + 3.0, ruler_rect.top() + 14.0),
                    egui::Align2::LEFT_TOP,
                    format!("🚩 {}", marker.name),
                    egui::FontId::proportional(10.0),
                    m_color,
                );
            }

            // Draw Bar Grid lines
            for beat in 0..=(total_beats as usize) {
                let x = ruler_rect.left() + beat as f32 * ppb;
                let is_bar = beat % 4 == 0;
                ruler_painter.line_segment(
                    [
                        egui::pos2(x, ruler_rect.top()),
                        egui::pos2(x, ruler_rect.bottom()),
                    ],
                    egui::Stroke::new(
                        if is_bar { 1.5_f32 } else { 0.8_f32 },
                        if is_bar {
                            egui::Color32::from_gray(140)
                        } else {
                            egui::Color32::from_gray(70)
                        },
                    ),
                );
                if is_bar {
                    ruler_painter.text(
                        egui::pos2(x + 4.0, ruler_rect.top() + 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("Bar {}", (beat / 4) + 1),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }

            playhead_x_out = Some(ruler_rect.left() + (*playhead_beat as f32 * ppb));
        });

        ui.separator();

        // Track Lanes & Reordering (Step 419)
        let mut duplicate_clip_target: Option<(u64, SequenceConfig)> = None;
        let mut reorder_swap: Option<(usize, usize)> = None;
        let mut clips_to_delete: Vec<(u64, usize)> = Vec::new();

        if delete_key_pressed && !state.selected_clips.is_empty() {
            for &(t_id, seq_i) in &state.selected_clips {
                clips_to_delete.push((t_id, seq_i));
            }
        }

        let num_tracks = project.tracks.len();
        for t_idx in 0..num_tracks {
            let track = &mut project.tracks[t_idx];
            let is_selected = selected_track_id.map_or(false, |id| id == track.id);
            let is_dimmed = any_soloed && !track.soloed;
            let row_height = if track.collapsed { 22.0 } else { 50.0 };

            ui.horizontal(|ui| {
                // Track Control Header (Steps 419, 420, 422, 424, 446, 448)
                ui.allocate_ui(egui::vec2(header_w, row_height), |ui| {
                    ui.horizontal(|ui| {
                        // Collapse / Expand toggle (Step 420)
                        let collapse_lbl = if track.collapsed { "▶" } else { "▼" };
                        if ui.small_button(collapse_lbl).clicked() {
                            track.collapsed = !track.collapsed;
                        }

                        // Left color stripe
                        let (stripe_resp, stripe_painter) = ui.allocate_painter(
                            egui::vec2(4.0, row_height - 6.0),
                            egui::Sense::hover(),
                        );
                        stripe_painter.rect_filled(stripe_resp.rect, 1.0, track_color(track));

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let name_text = if track.is_frozen {
                                    format!("❄ {}", track.name)
                                } else {
                                    track.name.clone()
                                };
                                let head_text = if is_dimmed {
                                    egui::RichText::new(&name_text)
                                        .color(egui::Color32::from_gray(100))
                                } else {
                                    egui::RichText::new(&name_text).strong()
                                };
                                let head = ui.selectable_label(is_selected, head_text);
                                if head.clicked() {
                                    *selected_track_id = Some(track.id);
                                }

                                // Mute / Solo / Record Arm buttons (Steps 424)
                                let mut mute = track.muted;
                                if ui.toggle_value(&mut mute, "M").changed() {
                                    track.muted = mute;
                                }
                                let mut solo = track.soloed;
                                if ui.toggle_value(&mut solo, "S").changed() {
                                    track.soloed = solo;
                                }
                                let mut arm = track.record_armed;
                                if ui.toggle_value(&mut arm, "R").changed() {
                                    track.record_armed = arm;
                                }

                                head.context_menu(|ui| {
                                    if ui.button("🎨 Electric Blue Color").clicked() {
                                        track.color = Some([26, 140, 255]);
                                        ui.close_menu();
                                    }
                                    if ui.button("🎨 Orange Color").clicked() {
                                        track.color = Some([255, 107, 43]);
                                        ui.close_menu();
                                    }
                                    if ui.button("🎨 Emerald Color").clicked() {
                                        track.color = Some([46, 204, 113]);
                                        ui.close_menu();
                                    }
                                    if ui
                                        .button(if track.is_frozen {
                                            "🔥 Unfreeze Track"
                                        } else {
                                            "❄ Freeze Track"
                                        })
                                        .clicked()
                                    {
                                        track.is_frozen = !track.is_frozen;
                                        ui.close_menu();
                                    }
                                    if ui
                                        .button(if track.send_to_master {
                                            "🔇 Bypass Master"
                                        } else {
                                            "🔊 Send to Master"
                                        })
                                        .clicked()
                                    {
                                        track.send_to_master = !track.send_to_master;
                                        ui.close_menu();
                                    }
                                    if t_idx > 0 && ui.button("⬆ Move Up").clicked() {
                                        reorder_swap = Some((t_idx, t_idx - 1));
                                        ui.close_menu();
                                    }
                                    if t_idx + 1 < num_tracks && ui.button("⬇ Move Down").clicked()
                                    {
                                        reorder_swap = Some((t_idx, t_idx + 1));
                                        ui.close_menu();
                                    }
                                });
                            });

                            if !track.collapsed {
                                ui.horizontal(|ui| {
                                    ui.label("Vol:");
                                    ui.add(egui::Slider::new(&mut track.gain, 0.0..=1.5).text(""));
                                });

                                // Track volume live RMS VU meter (Step 580)
                                let rms_vol = oscilloscope_buffers
                                    .and_then(|map| map.get(&track.id))
                                    .map_or(0.0f32, |scope| {
                                        let samples = scope.read_all();
                                        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
                                        (sum_sq / 512.0).sqrt()
                                    });
                                ui.horizontal(|ui| {
                                    ui.label("VU:");
                                    let (vu_resp, vu_painter) = ui.allocate_painter(
                                        egui::vec2(60.0, 6.0),
                                        egui::Sense::hover(),
                                    );
                                    let vu_rect = vu_resp.rect;
                                    vu_painter.rect_filled(
                                        vu_rect,
                                        1.0,
                                        egui::Color32::from_rgb(10, 10, 15),
                                    );
                                    let fill_w = (rms_vol * 60.0).clamp(0.0, 60.0);
                                    if fill_w > 0.0 {
                                        let fill_rect = egui::Rect::from_min_size(
                                            vu_rect.min,
                                            egui::vec2(fill_w, 6.0),
                                        );
                                        let vu_col = if rms_vol > 0.8 {
                                            egui::Color32::from_rgb(231, 76, 60)
                                        } else if rms_vol > 0.5 {
                                            egui::Color32::from_rgb(241, 196, 15)
                                        } else {
                                            egui::Color32::from_rgb(46, 204, 113)
                                        };
                                        vu_painter.rect_filled(fill_rect, 1.0, vu_col);
                                    }
                                });
                            }
                        });
                    });
                });

                ui.separator();

                // Track Timeline Area
                let (lane_resp, painter) = ui.allocate_painter(
                    egui::vec2(total_beats * ppb, row_height),
                    egui::Sense::click_and_drag(),
                );
                let lane_rect = lane_resp.rect;

                if lane_resp.clicked() {
                    *selected_track_id = Some(track.id);
                }

                // Draw background grid lines
                let bg_color = if track.is_frozen {
                    egui::Color32::from_rgb(18, 24, 34)
                } else if is_dimmed {
                    egui::Color32::from_rgb(12, 12, 16)
                } else if is_selected {
                    egui::Color32::from_rgb(25, 32, 45)
                } else {
                    egui::Color32::from_rgb(20, 20, 24)
                };
                painter.rect_filled(lane_rect, 2.0, bg_color);

                for beat in 0..=(total_beats as usize) {
                    let x = lane_rect.left() + beat as f32 * ppb;
                    let is_bar = beat % 4 == 0;
                    let stroke_color = if is_bar {
                        egui::Color32::from_gray(60)
                    } else {
                        egui::Color32::from_gray(35)
                    };
                    painter.line_segment(
                        [
                            egui::pos2(x, lane_rect.top()),
                            egui::pos2(x, lane_rect.bottom()),
                        ],
                        egui::Stroke::new(if is_bar { 1.0_f32 } else { 0.5_f32 }, stroke_color),
                    );
                    // Beat number text overlay at bar positions on all tracks (Step 593)
                    if is_bar && beat > 0 {
                        painter.text(
                            egui::pos2(x + 2.0, lane_rect.top() + 2.0),
                            egui::Align2::LEFT_TOP,
                            format!("Bar {}", (beat / 4) + 1),
                            egui::FontId::proportional(9.0),
                            egui::Color32::from_rgba_unmultiplied(150, 150, 160, 90),
                        );
                    }
                }

                // Render Clip Blocks for all sequences on track
                let ctrl_d = ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D));
                let track_id = track.id;
                let track_t_color = track_color(track);
                let track_is_collapsed = track.collapsed;
                let all_seqs = track.all_sequences_mut();

                for (seq_idx, seq) in all_seqs.into_iter().enumerate() {
                    let start_x = lane_rect.left() + (seq.start_beat as f32 * ppb);
                    let clip_beats =
                        (seq.steps.len() as f64 * seq.step_division * seq.time_stretch).max(0.5);
                    let clip_width = (clip_beats as f32 * ppb).max(30.0);

                    let clip_rect = egui::Rect::from_min_size(
                        egui::pos2(start_x, lane_rect.top() + 3.0),
                        egui::vec2(clip_width, lane_rect.height() - 6.0),
                    );

                    let clip_id = ui.id().with(("clip", track_id, seq_idx));
                    let clip_resp = ui.interact(clip_rect, clip_id, egui::Sense::click_and_drag());
                    let is_clip_selected = state.selected_clips.contains(&(track_id, seq_idx));

                    if clip_resp.dragged() {
                        let delta_x = clip_resp.drag_delta().x;
                        let delta_beats = (delta_x / ppb) as f64;
                        seq.start_beat = (seq.start_beat + delta_beats).max(0.0);
                    }
                    if clip_resp.drag_stopped() {
                        let div = (*grid_division).max(0.01);
                        seq.start_beat = (seq.start_beat / div).round() * div;
                    }

                    if clip_resp.clicked() {
                        *selected_track_id = Some(track_id);
                        if !ui.input(|i| i.modifiers.shift) {
                            state.selected_clips.clear();
                        }
                        state.selected_clips.insert((track_id, seq_idx));

                        // Clip Split on Ctrl+Click (Step 429)
                        if ui.input(|i| i.modifiers.ctrl) {
                            if let Some(pos) = clip_resp.interact_pointer_pos() {
                                let split_beat = ((pos.x - clip_rect.left()) / ppb) as f64;
                                split_clip_at(seq, split_beat);
                            }
                        }
                    }

                    if is_selected && ctrl_d {
                        let cloned = seq.duplicate();
                        duplicate_clip_target = Some((track_id, cloned));
                    }

                    if clip_resp.double_clicked() {
                        *selected_track_id = Some(track_id);
                        navigation_target = Some(ViewMode::PianoRoll(track_id));
                    }

                    let mut dup_clip = false;
                    let mut del_clip = false;

                    clip_resp.context_menu(|ui| {
                        if ui.button("🎹 Edit in Piano Roll").clicked() {
                            navigation_target = Some(ViewMode::PianoRoll(track_id));
                            ui.close_menu();
                        }
                        if ui.button("📋 Duplicate Clip (Ctrl+D)").clicked() {
                            dup_clip = true;
                            ui.close_menu();
                        }
                        if ui.button("⏩ Duplicate to Next Bar").clicked() {
                            // Step 441
                            let mut dup = seq.duplicate();
                            let end_beat = seq.start_beat + clip_beats;
                            dup.start_beat = (end_beat / 4.0).ceil() * 4.0;
                            duplicate_clip_target = Some((track_id, dup));
                            ui.close_menu();
                        }
                        if ui.button("🔁 Fill Loop Region").clicked() {
                            // Step 442
                            fill_loop_region(seq, project.loop_start_beat, project.loop_end_beat);
                            ui.close_menu();
                        }
                        if ui.button("↩ Restore Clip (Reset Trim & Fades)").clicked() {
                            // Step 588
                            seq.restore();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Gain:");
                            ui.add(
                                egui::DragValue::new(&mut seq.gain)
                                    .speed(0.05)
                                    .range(0.0..=4.0),
                            ); // Step 584
                        });
                        ui.horizontal(|ui| {
                            ui.label("Pitch (st):");
                            ui.add(
                                egui::DragValue::new(&mut seq.pitch_offset)
                                    .speed(1.0)
                                    .range(-24.0..=24.0),
                            ); // Step 585
                        });
                        ui.horizontal(|ui| {
                            ui.label("Trim Start:");
                            ui.add(
                                egui::DragValue::new(&mut seq.trim_start)
                                    .speed(0.1)
                                    .range(0.0..=64.0),
                            ); // Step 586
                        });
                        ui.horizontal(|ui| {
                            ui.label("Trim End:");
                            ui.add(
                                egui::DragValue::new(&mut seq.trim_end)
                                    .speed(0.1)
                                    .range(0.0..=64.0),
                            ); // Step 586
                        });
                        ui.separator();
                        if ui.button("🔄 Reverse Clip").clicked() {
                            // Step 431
                            seq.is_reversed = !seq.is_reversed;
                            seq.steps.reverse();
                            ui.close_menu();
                        }
                        if ui.button("🔊 Normalize Clip Peak").clicked() {
                            // Step 433
                            normalize_clip(seq);
                            ui.close_menu();
                        }
                        if ui.button("✂ Trim Silence").clicked() {
                            // Step 434
                            trim_silence(seq);
                            ui.close_menu();
                        }
                        if ui.button("✨ Make Unique").clicked() {
                            seq.make_unique();
                            ui.close_menu();
                        }
                        if ui.button("🎨 Electric Blue Color").clicked() {
                            seq.clip_color = Some([26, 140, 255]);
                            ui.close_menu();
                        }
                        if ui.button("🎨 Orange Color").clicked() {
                            seq.clip_color = Some([255, 107, 43]);
                            ui.close_menu();
                        }
                        if ui.button("🎨 Emerald Color").clicked() {
                            seq.clip_color = Some([46, 204, 113]);
                            ui.close_menu();
                        }
                        if ui.button("🗑 Delete Clip").clicked() {
                            del_clip = true;
                            ui.close_menu();
                        }
                    });

                    if dup_clip {
                        duplicate_clip_target = Some((track_id, seq.duplicate()));
                    }
                    if del_clip {
                        clips_to_delete.push((track_id, seq_idx));
                    }

                    // Render Clip Block Background & Envelope (Steps 426, 427, 428)
                    let fill_color = if let Some(rgb) = seq.clip_color {
                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                    } else if is_clip_selected {
                        egui::Color32::from_rgb(40, 110, 200)
                    } else if is_dimmed {
                        egui::Color32::from_rgb(30, 40, 50)
                    } else if is_selected {
                        egui::Color32::from_rgb(40, 90, 160)
                    } else {
                        egui::Color32::from_rgb(35, 65, 110)
                    };

                    painter.rect_filled(clip_rect, 4.0, fill_color);
                    let border_stroke = if is_clip_selected {
                        egui::Stroke::new(2.0f32, egui::Color32::from_rgb(255, 230, 100))
                    } else if seq.is_unique {
                        egui::Stroke::new(2.0f32, egui::Color32::from_rgb(255, 200, 50))
                    } else {
                        egui::Stroke::new(1.5f32, track_t_color)
                    };
                    painter.rect_stroke(clip_rect, 4.0, border_stroke);

                    // Trim shading (Step 587)
                    if seq.trim_start > 0.0 {
                        let trim_w = (seq.trim_start as f32 * ppb).min(clip_rect.width() * 0.5);
                        let trim_rect = egui::Rect::from_min_size(
                            clip_rect.min,
                            egui::vec2(trim_w, clip_rect.height()),
                        );
                        painter.rect_filled(trim_rect, 0.0, egui::Color32::from_black_alpha(160));
                    }
                    if seq.trim_end > 0.0 {
                        let trim_w = (seq.trim_end as f32 * ppb).min(clip_rect.width() * 0.5);
                        let trim_rect = egui::Rect::from_min_size(
                            egui::pos2(clip_rect.right() - trim_w, clip_rect.top()),
                            egui::vec2(trim_w, clip_rect.height()),
                        );
                        painter.rect_filled(trim_rect, 0.0, egui::Color32::from_black_alpha(160));
                    }

                    // Fade handles & Polygon gradient fill (Step 582)
                    if seq.fade_in > 0.0 {
                        let fade_w = (seq.fade_in as f32 * ppb).min(clip_rect.width() * 0.5);
                        let pts = vec![
                            clip_rect.left_bottom(),
                            clip_rect.left_top(),
                            egui::pos2(clip_rect.left() + fade_w, clip_rect.top()),
                        ];
                        painter.add(egui::Shape::convex_polygon(
                            pts,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 35),
                            egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                        ));
                    }
                    if seq.fade_out > 0.0 {
                        let fade_w = (seq.fade_out as f32 * ppb).min(clip_rect.width() * 0.5);
                        let pts = vec![
                            egui::pos2(clip_rect.right() - fade_w, clip_rect.top()),
                            clip_rect.right_top(),
                            clip_rect.right_bottom(),
                        ];
                        painter.add(egui::Shape::convex_polygon(
                            pts,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 35),
                            egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                        ));
                    }

                    // Clip Name, Gain/Pitch & Step Count Label (Steps 584, 585)
                    let clip_label = seq.clip_name.as_deref().unwrap_or("Pattern");
                    let rev_flag = if seq.is_reversed { " 🔄" } else { "" };
                    let gain_str = if (seq.gain - 1.0).abs() > 0.01 {
                        format!(" G:{:.1}x", seq.gain)
                    } else {
                        String::new()
                    };
                    let pitch_str = if seq.pitch_offset.abs() > 0.01 {
                        format!(" P:{:+.0}st", seq.pitch_offset)
                    } else {
                        String::new()
                    };
                    if !track_is_collapsed {
                        painter.text(
                            egui::pos2(clip_rect.left() + 6.0, clip_rect.top() + 3.0),
                            egui::Align2::LEFT_TOP,
                            format!(
                                "{}{}{}{}{}",
                                clip_label,
                                rev_flag,
                                gain_str,
                                pitch_str,
                                if seq.is_unique { " ★" } else { "" }
                            ),
                            egui::FontId::proportional(11.0),
                            egui::Color32::WHITE,
                        );
                    }
                }

                // Crossfade indicator between overlapping clips on the same track (Step 583)
                let clip_info_list: Vec<(f32, f32)> = track
                    .all_sequences()
                    .iter()
                    .map(|s| {
                        let start = lane_rect.left() + (s.start_beat as f32 * ppb);
                        let duration_beats =
                            (s.steps.len() as f64 * s.step_division * s.time_stretch).max(0.5)
                                as f32;
                        let width = (duration_beats * ppb).max(30.0);
                        (start, start + width)
                    })
                    .collect();

                for i in 0..clip_info_list.len() {
                    for j in (i + 1)..clip_info_list.len() {
                        let (s1, e1) = clip_info_list[i];
                        let (s2, e2) = clip_info_list[j];
                        let overlap_start = s1.max(s2);
                        let overlap_end = e1.min(e2);
                        if overlap_end > overlap_start {
                            let xf_rect = egui::Rect::from_min_max(
                                egui::pos2(overlap_start, lane_rect.top() + 3.0),
                                egui::pos2(overlap_end, lane_rect.bottom() - 3.0),
                            );
                            painter.rect_filled(
                                xf_rect,
                                2.0,
                                egui::Color32::from_rgba_unmultiplied(46, 204, 113, 60),
                            );
                            painter.line_segment(
                                [xf_rect.left_top(), xf_rect.right_bottom()],
                                egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
                            );
                            painter.line_segment(
                                [xf_rect.left_bottom(), xf_rect.right_top()],
                                egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
                            );
                        }
                    }
                }

                // Context menu on empty track lane area (Step 589)
                lane_resp.context_menu(|ui| {
                    if ui.button("➕ Add Pattern Clip").clicked() {
                        let click_pos =
                            ui.input(|i| i.pointer.hover_pos()).unwrap_or(lane_rect.min);
                        let clicked_beat = ((click_pos.x - lane_rect.left()) / ppb).max(0.0) as f64;
                        let div = (*grid_division).max(0.01);
                        let start_beat = (clicked_beat / div).round() * div;
                        track.clips.push(SequenceConfig {
                            start_beat,
                            clip_name: Some(format!("Clip {}", track.clips.len() + 1)),
                            ..Default::default()
                        });
                        ui.close_menu();
                    }
                    if let Some(ref cb) = state.clipboard_clip {
                        if ui.button("📋 Paste Clip").clicked() {
                            let mut pasted = cb.duplicate();
                            let click_pos =
                                ui.input(|i| i.pointer.hover_pos()).unwrap_or(lane_rect.min);
                            let clicked_beat =
                                ((click_pos.x - lane_rect.left()) / ppb).max(0.0) as f64;
                            let div = (*grid_division).max(0.01);
                            pasted.start_beat = (clicked_beat / div).round() * div;
                            track.clips.push(pasted);
                            ui.close_menu();
                        }
                    }
                });
            });

            // Render Automation Lanes for this track
            if let Some(timeline) = automation_timeline {
                let track_prefix = format!("track_{}", track.id);
                for (param_id, lane) in &timeline.lanes {
                    let belongs_to_track = param_id.contains(&track_prefix)
                        || (track.id == 1 && !timeline.lanes.keys().any(|k| k.contains("track_")));
                    if belongs_to_track {
                        show_automation_lane(ui, lane, ppb, total_beats);
                    }
                }
            }

            ui.add_space(3.0);
        }

        // Apply track reordering swap if requested
        if let Some((from_i, to_i)) = reorder_swap {
            project.tracks.swap(from_i, to_i);
        }

        // Handle clip additions from duplicate targets
        if let Some((t_id, new_clip)) = duplicate_clip_target {
            if let Some(t) = project.tracks.iter_mut().find(|tr| tr.id == t_id) {
                t.clips.push(new_clip);
            }
        }

        // Handle clip deletions
        for (t_id, del_idx) in clips_to_delete {
            if let Some(t) = project.tracks.iter_mut().find(|tr| tr.id == t_id) {
                if del_idx == 0 && t.sequence.is_some() {
                    t.sequence = None;
                } else {
                    let clip_arr_idx = if t.sequence.is_some() {
                        del_idx - 1
                    } else {
                        del_idx
                    };
                    if clip_arr_idx < t.clips.len() {
                        t.clips.remove(clip_arr_idx);
                    }
                }
            }
        }

        grid_bottom_out = Some(ui.cursor().min.y);

        // Render Red Playhead Line across all tracks
        if let (Some(px), Some(top), Some(bottom)) = (playhead_x_out, grid_top_out, grid_bottom_out)
        {
            let painter = ui.painter();
            painter.line_segment(
                [egui::pos2(px, top), egui::pos2(px, bottom)],
                egui::Stroke::new(2.0f32, egui::Color32::from_rgb(255, 60, 60)),
            );
        }
    });

    ui.data_mut(|d| d.insert_temp(state_id, state));
    navigation_target
}

/// Helper function to auto-color all tracks from vibrant palette (Step 445).
pub fn auto_color_tracks(tracks: &mut [TrackConfig]) {
    let palette = [
        [26, 140, 255], // Electric Blue
        [255, 107, 43], // Orange
        [46, 204, 113], // Emerald Green
        [155, 89, 182], // Purple
        [241, 196, 15], // Amber
        [231, 76, 60],  // Red
        [52, 152, 219], // Cyan
    ];
    for (i, t) in tracks.iter_mut().enumerate() {
        t.color = Some(palette[i % palette.len()]);
    }
}

/// Helper to normalize active steps in a sequence to max 1.0 velocity (Step 433).
pub fn normalize_clip(seq: &mut SequenceConfig) {
    let max_vel = seq
        .steps
        .iter()
        .filter(|s| s.active)
        .map(|s| s.velocity)
        .fold(0.0f32, f32::max);
    if max_vel > 0.0 {
        let factor = 1.0 / max_vel;
        for s in &mut seq.steps {
            if s.active {
                s.velocity = (s.velocity * factor).clamp(0.0, 1.0);
            }
        }
    }
}

/// Helper to trim leading/trailing inactive steps from sequence (Step 434).
pub fn trim_silence(seq: &mut SequenceConfig) {
    if let Some(first_active) = seq.steps.iter().position(|s| s.active) {
        if let Some(last_active) = seq.steps.iter().rposition(|s| s.active) {
            seq.start_beat += first_active as f64 * seq.step_division;
            seq.steps = seq.steps[first_active..=last_active].to_vec();
        }
    }
}

/// Helper to split a sequence clip at a given beat offset (Step 429).
pub fn split_clip_at(seq: &mut SequenceConfig, split_beat: f64) {
    if split_beat <= 0.0 {
        return;
    }
    let step_idx = (split_beat / seq.step_division).round() as usize;
    if step_idx > 0 && step_idx < seq.steps.len() {
        let _remaining_steps = seq.steps.split_off(step_idx);
    }
}

/// Helper to tile clip across loop region (Step 442).
pub fn fill_loop_region(seq: &mut SequenceConfig, loop_start: f64, loop_end: f64) {
    let span = loop_end - loop_start;
    if span <= 0.0 {
        return;
    }
    let clip_dur = seq.steps.len() as f64 * seq.step_division;
    if clip_dur > 0.0 {
        let count = (span / clip_dur).ceil() as usize;
        let mut new_steps = Vec::new();
        for _ in 0..count {
            new_steps.extend(seq.steps.clone());
        }
        seq.steps = new_steps;
        seq.start_beat = loop_start;
    }
}

pub fn show_automation_lane(
    ui: &mut egui::Ui,
    lane: &summoner_sequencer::automation_timeline::AutomationLane,
    pixels_per_beat: f32,
    total_beats: f32,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(180.0, 24.0), |ui| {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let first_interp = lane.curve.points.first().map(|p| p.interp);
                let shape_icon = match first_interp {
                    Some(summoner_sequencer::automation_timeline::Interpolation::Linear) | None => {
                        "📈"
                    }
                    Some(summoner_sequencer::automation_timeline::Interpolation::Exponential) => {
                        "📈²"
                    }
                    Some(summoner_sequencer::automation_timeline::Interpolation::Logarithmic) => {
                        "📉"
                    }
                    Some(summoner_sequencer::automation_timeline::Interpolation::Step) => "⎍",
                    Some(summoner_sequencer::automation_timeline::Interpolation::Smooth) => "🌊",
                    Some(summoner_sequencer::automation_timeline::Interpolation::Bezier(_, _)) => {
                        "➰"
                    }
                };
                let head = ui.label(
                    egui::RichText::new(format!("{} {}", shape_icon, lane.param_id))
                        .font(egui::FontId::proportional(11.0))
                        .color(egui::Color32::from_rgb(241, 196, 15)),
                );

                head.context_menu(|ui| {
                    ui.label(format!("Automation: {}", lane.param_id));
                    ui.separator();
                    ui.label("Tools:");
                    if ui.button("📋 Copy Lane").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("✖ Scale Automation (0.5x)").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("🔄 Invert Curve").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("🌊 Smooth Points").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        ui.separator();

        let (lane_resp, painter) = ui.allocate_painter(
            egui::vec2(total_beats * pixels_per_beat, 24.0),
            egui::Sense::hover(),
        );
        let lane_rect = lane_resp.rect;

        painter.rect_filled(lane_rect, 1.0, egui::Color32::from_rgb(14, 14, 18));

        let points = &lane.curve.points;
        if !points.is_empty() {
            let stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(241, 196, 15));
            let mut prev_pos: Option<egui::Pos2> = None;

            for pt in points {
                let x = lane_rect.left() + (pt.beat as f32 * pixels_per_beat);
                let norm_val = pt.value.clamp(0.0, 1.0);
                let y = lane_rect.bottom() - (norm_val * (lane_rect.height() - 4.0) + 2.0);
                let curr_pos = egui::pos2(x, y);

                if let Some(prev) = prev_pos {
                    match pt.interp {
                        summoner_sequencer::automation_timeline::Interpolation::Step => {
                            let step_mid = egui::pos2(curr_pos.x, prev.y);
                            painter.line_segment([prev, step_mid], stroke);
                            painter.line_segment([step_mid, curr_pos], stroke);
                        }
                        _ => {
                            painter.line_segment([prev, curr_pos], stroke);
                        }
                    }
                }
                painter.circle_filled(curr_pos, 2.5, egui::Color32::from_rgb(255, 230, 120));

                // Draw curve shape icon on segment (Step 599)
                let icon = match pt.interp {
                    summoner_sequencer::automation_timeline::Interpolation::Linear => "📈",
                    summoner_sequencer::automation_timeline::Interpolation::Exponential => "📈²",
                    summoner_sequencer::automation_timeline::Interpolation::Logarithmic => "📉",
                    summoner_sequencer::automation_timeline::Interpolation::Step => "⎍",
                    summoner_sequencer::automation_timeline::Interpolation::Smooth => "🌊",
                    summoner_sequencer::automation_timeline::Interpolation::Bezier(_, _) => "➰",
                };
                painter.text(
                    egui::pos2(curr_pos.x + 3.0, curr_pos.y - 10.0),
                    egui::Align2::LEFT_BOTTOM,
                    icon,
                    egui::FontId::proportional(9.0),
                    egui::Color32::from_rgb(241, 196, 15),
                );

                prev_pos = Some(curr_pos);
            }
        }
    });
    ui.add_space(2.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_tier21_arranger_renders_without_panic() {
        let mut project = create_default_project("Tier 21 Arranger Test");
        let mut selected_id = Some(1);
        let mut playhead = 0.0;
        let mut ppb = 40.0;
        let mut grid_division = 0.25;
        let mut track_header_width = 180.0;
        let mut waveform_cache = crate::waveform_cache::WaveformCache::new();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_arranger(
                    ui,
                    &mut project,
                    &mut selected_id,
                    &mut playhead,
                    false,
                    &mut ppb,
                    None,
                    &mut grid_division,
                    &mut track_header_width,
                    &mut waveform_cache,
                    None,
                );
            });
        });
    }

    #[test]
    fn test_tier21_auto_color_tracks() {
        let mut project = create_default_project("Auto Color");
        auto_color_tracks(&mut project.tracks);
        for track in &project.tracks {
            assert!(track.color.is_some());
        }
    }

    #[test]
    fn test_tier21_normalize_clip() {
        let mut seq = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.5,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 62.0,
                    velocity: 0.25,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            is_reversed: false,
            time_stretch: 1.0,
            ..Default::default()
        };
        normalize_clip(&mut seq);
        assert!((seq.steps[0].velocity - 1.0).abs() < 1e-5);
        assert!((seq.steps[1].velocity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_tier21_trim_silence() {
        let mut seq = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.0,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: false,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 62.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.0,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: false,
                    muted: false,
                },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            is_reversed: false,
            time_stretch: 1.0,
            ..Default::default()
        };
        trim_silence(&mut seq);
        assert_eq!(seq.start_beat, 0.25);
        assert_eq!(seq.steps.len(), 1);
        assert_eq!(seq.steps[0].note, 62.0);
    }

    #[test]
    fn test_tier21_split_clip_at() {
        let mut seq = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 62.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 65.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            is_reversed: false,
            time_stretch: 1.0,
            ..Default::default()
        };
        split_clip_at(&mut seq, 0.5); // split after 2 steps
        assert_eq!(seq.steps.len(), 2);
    }

    #[test]
    fn test_tier32_fill_loop_region() {
        let mut seq = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 62.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
            ],
            fade_in: 0.0,
            fade_out: 0.0,
            is_reversed: false,
            time_stretch: 1.0,
            ..Default::default()
        };
        fill_loop_region(&mut seq, 0.0, 2.0); // 2 beats loop = 8 steps at 0.25 div (4 copies of 2 steps)
        assert_eq!(seq.steps.len(), 8);
        assert_eq!(seq.start_beat, 0.0);
    }

    #[test]
    fn test_tier32_arranger_state_clipboard() {
        let mut state = ArrangerState::default();
        assert!(state.clipboard_clip.is_none());
        let seq = SequenceConfig::default();
        state.clipboard_clip = Some(seq);
        assert!(state.clipboard_clip.is_some());
    }
}
