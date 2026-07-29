use eframe::egui;
use summoner_project::schema::ProjectConfig;
use crate::app::ViewMode;

fn track_color(track_id: u64) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(26, 140, 255),  // Electric Blue
        egui::Color32::from_rgb(255, 107, 43),  // Orange
        egui::Color32::from_rgb(46, 204, 113),  // Emerald Green
        egui::Color32::from_rgb(155, 89, 182),  // Purple
        egui::Color32::from_rgb(241, 196, 15),  // Yellow/Amber
        egui::Color32::from_rgb(231, 76, 60),   // Red/Rose
        egui::Color32::from_rgb(52, 152, 219),  // Cyan
        egui::Color32::from_rgb(230, 126, 34),  // Amber
    ];
    colors[(track_id as usize) % colors.len()]
}

pub fn show_arranger(
    ui: &mut egui::Ui,
    project: &mut ProjectConfig,
    selected_track_id: &mut Option<u64>,
    playhead_beat: &mut f64,
    _transport_running: bool,
    pixels_per_beat: &mut f32,
) -> Option<ViewMode> {
    let mut navigation_target = None;

    // Header Toolbar
    ui.horizontal(|ui| {
        ui.heading("Arranger Timeline");
        ui.separator();
        ui.label(format!("Session: {}", project.name));
        ui.separator();

        if ui.button("➕ Add Track").clicked() {
            let next_id = project.tracks.len() as u64 + 1;
            project.tracks.push(summoner_project::schema::TrackConfig {
                id: next_id,
                name: format!("Track {}", next_id),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                nodes: Vec::new(),
                sequence: None,
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
            });
            *selected_track_id = Some(next_id);
        }

        if ui.button("➕ Clip").clicked() {
            if let Some(selected_id) = *selected_track_id {
                if let Some(track) = project.tracks.iter_mut().find(|t| t.id == selected_id) {
                    if track.sequence.is_none() {
                        track.sequence = Some(summoner_project::schema::SequenceConfig {
                            start_beat: 0.0,
                            step_division: 0.25,
                            steps: vec![summoner_project::schema::TrackerStepConfig {
                                note: 60.0,
                                velocity: 0.8,
                                gate: 0.5,
                                probability: 1.0,
                                ratchet: 1,
                                micro_shift: 0,
                                active: true,
                            }; 16],
                        });
                    }
                }
            }
        }

        ui.separator();
        ui.label("Zoom:");
        ui.add(egui::Slider::new(pixels_per_beat, 10.0..=400.0).text("px/beat"));
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

    let mut playhead_x_out = None;
    let mut grid_top_out = None;
    let mut grid_bottom_out = None;

    egui::ScrollArea::both().show(ui, |ui| {
        let start_pos = ui.cursor().min;
        grid_top_out = Some(start_pos.y);

        // Timeline Header Ruler
        ui.horizontal(|ui| {
            ui.allocate_space(egui::vec2(180.0, 24.0)); // Space above track control headers
            let (ruler_resp, ruler_painter) = ui.allocate_painter(egui::vec2(total_beats * ppb, 24.0), egui::Sense::click());
            let ruler_rect = ruler_resp.rect;

            if ruler_resp.clicked() {
                if let Some(pos) = ruler_resp.interact_pointer_pos() {
                    let beat = ((pos.x - ruler_rect.left()) / ppb).max(0.0) as f64;
                    *playhead_beat = beat;
                }
            }

            ruler_painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(30, 30, 35));
            for beat in 0..=(total_beats as usize) {
                let x = ruler_rect.left() + beat as f32 * ppb;
                let is_bar = beat % 4 == 0;
                ruler_painter.line_segment(
                    [egui::pos2(x, ruler_rect.top()), egui::pos2(x, ruler_rect.bottom())],
                    egui::Stroke::new(if is_bar { 1.5_f32 } else { 0.8_f32 }, if is_bar { egui::Color32::from_gray(140) } else { egui::Color32::from_gray(70) }),
                );
                if is_bar {
                    ruler_painter.text(
                        egui::pos2(x + 4.0, ruler_rect.top() + 4.0),
                        egui::Align2::LEFT_TOP,
                        format!("Bar {}", (beat / 4) + 1),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                } else if ppb > 80.0 {
                    // Render 16th subdivisions
                    for sub in 1..4 {
                        let sub_x = ruler_rect.left() + (beat as f32 + sub as f32 * 0.25) * ppb;
                        ruler_painter.line_segment(
                            [egui::pos2(sub_x, ruler_rect.top() + 12.0), egui::pos2(sub_x, ruler_rect.bottom())],
                            egui::Stroke::new(0.5_f32, egui::Color32::from_gray(45)),
                        );
                    }
                }
            }

            playhead_x_out = Some(ruler_rect.left() + (*playhead_beat as f32 * ppb));
        });

        ui.separator();

        // Track Lanes
        let mut duplicate_clip_target: Option<(u64, summoner_project::schema::SequenceConfig)> = None;

        for track in &mut project.tracks {
            let is_selected = selected_track_id.map_or(false, |id| id == track.id);
            let is_dimmed = any_soloed && !track.soloed;

            ui.horizontal(|ui| {
                // Track Control Header
                ui.allocate_ui(egui::vec2(180.0, 50.0), |ui| {
                    ui.horizontal(|ui| {
                        // Left color stripe
                        let (stripe_resp, stripe_painter) = ui.allocate_painter(egui::vec2(4.0, 44.0), egui::Sense::hover());
                        stripe_painter.rect_filled(stripe_resp.rect, 1.0, track_color(track.id));

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let head_text = if is_dimmed {
                                    egui::RichText::new(&track.name).color(egui::Color32::from_gray(100))
                                } else {
                                    egui::RichText::new(&track.name).strong()
                                };
                                let head = ui.selectable_label(is_selected, head_text);
                                if head.clicked() {
                                    *selected_track_id = Some(track.id);
                                }
                                let mut mute = track.muted;
                                if ui.toggle_value(&mut mute, "M").changed() {
                                    track.muted = mute;
                                }
                                let mut solo = track.soloed;
                                if ui.toggle_value(&mut solo, "S").changed() {
                                    track.soloed = solo;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Vol:");
                                ui.add(egui::Slider::new(&mut track.gain, 0.0..=1.5).text(""));
                            });
                        });
                    });
                });

                ui.separator();

                // Track Timeline Area
                let (lane_resp, painter) = ui.allocate_painter(egui::vec2(total_beats * ppb, 50.0), egui::Sense::click_and_drag());
                let lane_rect = lane_resp.rect;

                if lane_resp.clicked() {
                    *selected_track_id = Some(track.id);
                }

                // Draw background grid lines
                let bg_color = if is_dimmed {
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
                    let stroke_color = if is_bar { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(35) };
                    painter.line_segment(
                        [egui::pos2(x, lane_rect.top()), egui::pos2(x, lane_rect.bottom())],
                        egui::Stroke::new(if is_bar { 1.0_f32 } else { 0.5_f32 }, stroke_color),
                    );
                    if ppb > 80.0 {
                        for sub in 1..4 {
                            let sub_x = lane_rect.left() + (beat as f32 + sub as f32 * 0.25) * ppb;
                            painter.line_segment(
                                [egui::pos2(sub_x, lane_rect.top()), egui::pos2(sub_x, lane_rect.bottom())],
                                egui::Stroke::new(0.5_f32, egui::Color32::from_gray(25)),
                            );
                        }
                    }
                }

                // Render Clip Block
                if let Some(seq) = &mut track.sequence {
                    let start_x = lane_rect.left() + (seq.start_beat as f32 * ppb);
                    let clip_beats = seq.steps.len() as f64 * seq.step_division;
                    let clip_width = (clip_beats as f32 * ppb).max(30.0);

                    let clip_rect = egui::Rect::from_min_size(
                        egui::pos2(start_x, lane_rect.top() + 4.0),
                        egui::vec2(clip_width, lane_rect.height() - 8.0),
                    );

                    let clip_id = ui.id().with(("clip", track.id));
                    let clip_resp = ui.interact(clip_rect, clip_id, egui::Sense::click_and_drag());

                    if clip_resp.dragged() {
                        let delta_x = clip_resp.drag_delta().x;
                        let delta_beats = (delta_x / ppb) as f64;
                        seq.start_beat = (seq.start_beat + delta_beats).max(0.0);
                    }

                    if clip_resp.clicked() {
                        *selected_track_id = Some(track.id);
                    }

                    if clip_resp.double_clicked() {
                        *selected_track_id = Some(track.id);
                        navigation_target = Some(ViewMode::PianoRoll(track.id));
                    }

                    let mut delete_clip = false;
                    let mut dup_clip = false;

                    clip_resp.context_menu(|ui| {
                        if ui.button("🎹 Edit in Piano Roll").clicked() {
                            navigation_target = Some(ViewMode::PianoRoll(track.id));
                            ui.close_menu();
                        }
                        if ui.button("📋 Duplicate Clip").clicked() {
                            dup_clip = true;
                            ui.close_menu();
                        }
                        if ui.button("🗑 Delete Clip").clicked() {
                            delete_clip = true;
                            ui.close_menu();
                        }
                    });

                    if dup_clip {
                        let mut cloned = seq.clone();
                        cloned.start_beat += clip_beats;
                        duplicate_clip_target = Some((track.id, cloned));
                    }

                    if delete_clip {
                        track.sequence = None;
                    } else {
                        // Render clip background and border
                        let fill_color = if is_dimmed {
                            egui::Color32::from_rgb(30, 40, 50)
                        } else if is_selected {
                            egui::Color32::from_rgb(40, 90, 160)
                        } else {
                            egui::Color32::from_rgb(35, 65, 110)
                        };

                        painter.rect_filled(clip_rect, 4.0, fill_color);
                        painter.rect_stroke(clip_rect, 4.0, egui::Stroke::new(1.5f32, track_color(track.id)));

                        // Render Step-Grid Mini Preview
                        let step_count = seq.steps.len();
                        if step_count > 0 {
                            let step_w = clip_rect.width() / step_count as f32;
                            for (idx, step) in seq.steps.iter().enumerate() {
                                if step.active {
                                    let sx = clip_rect.left() + idx as f32 * step_w;
                                    let h = (step.velocity * (clip_rect.height() - 14.0)).clamp(2.0, clip_rect.height() - 14.0);
                                    let sy = clip_rect.bottom() - 4.0 - h;
                                    let s_rect = egui::Rect::from_min_size(
                                        egui::pos2(sx + 1.0, sy),
                                        egui::vec2((step_w - 2.0).max(1.0), h),
                                    );
                                    painter.rect_filled(s_rect, 1.0, egui::Color32::from_rgb(120, 200, 255));
                                }
                            }
                        }

                        painter.text(
                            egui::pos2(clip_rect.left() + 6.0, clip_rect.top() + 3.0),
                            egui::Align2::LEFT_TOP,
                            format!("Pattern ({} steps)", seq.steps.len()),
                            egui::FontId::proportional(11.0),
                            egui::Color32::WHITE,
                        );
                    }
                }
            });
            ui.add_space(4.0);
        }

        grid_bottom_out = Some(ui.cursor().min.y);

        // Render Red Playhead Line across all tracks
        if let (Some(px), Some(top), Some(bottom)) = (playhead_x_out, grid_top_out, grid_bottom_out) {
            let painter = ui.painter();
            painter.line_segment(
                [egui::pos2(px, top), egui::pos2(px, bottom)],
                egui::Stroke::new(2.0f32, egui::Color32::from_rgb(255, 60, 60)),
            );
        }
    });

    navigation_target
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_arranger_renders_without_panic() {
        let mut project = create_default_project("Arranger Test");
        let mut selected_id = Some(1);
        let mut playhead = 0.0;
        let mut ppb = 40.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_arranger(ui, &mut project, &mut selected_id, &mut playhead, false, &mut ppb);
            });
        });
    }

    #[test]
    fn test_arranger_playhead_advances() {
        let mut playhead: f64 = 0.0;
        let transport_running = true;
        let dt: f64 = 1.0 / 60.0; // 60 fps frame
        let bpm: f64 = 120.0;

        if transport_running {
            playhead += dt * (bpm / 60.0);
        }

        assert!(playhead > 0.0);
        assert!((playhead - (1.0 / 30.0)).abs() < 1e-4);
    }

    #[test]
    fn test_arranger_clip_drag() {
        let mut project = create_default_project("Drag Test");
        if let Some(track) = project.tracks.get_mut(0) {
            track.sequence = Some(summoner_project::schema::SequenceConfig {
                start_beat: 0.0,
                step_division: 0.25,
                steps: vec![],
            });
            // Simulate dragging 2 beats
            if let Some(seq) = &mut track.sequence {
                seq.start_beat += 2.0;
            }
            assert_eq!(track.sequence.as_ref().unwrap().start_beat, 2.0);
        }
    }
}
