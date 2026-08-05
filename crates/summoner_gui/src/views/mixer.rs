use crate::visualizer::{show_spectrum, SpectrumAnalyzer};
use eframe::egui;
use std::collections::HashMap;
use summoner_project::schema::{NodeConfig, ProjectConfig};

/// Internal GUI state for Console Mixer view (peak hold, master fader, popup state).
#[derive(Debug, Clone)]
pub struct MixerState {
    pub master_gain: f32,
    pub master_pan: f32,
    pub master_muted: bool,
    pub peak_holds: HashMap<u64, f32>, // track_id (0 for Master) -> peak_level (0.0 ..= 1.0)
    pub fx_popup_track_id: Option<u64>,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            master_gain: 1.0,
            master_pan: 0.0,
            master_muted: false,
            peak_holds: HashMap::new(),
            fx_popup_track_id: None,
        }
    }
}

fn track_color(track_id: u64) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(26, 140, 255), // Electric Blue
        egui::Color32::from_rgb(255, 107, 43), // Orange Accent
        egui::Color32::from_rgb(155, 89, 182), // Purple
        egui::Color32::from_rgb(46, 204, 113), // Emerald Green
        egui::Color32::from_rgb(241, 196, 15), // Yellow
        egui::Color32::from_rgb(231, 76, 60),  // Crimson
        egui::Color32::from_rgb(52, 152, 219), // Cyan
        egui::Color32::from_rgb(230, 126, 34), // Amber
    ];
    colors[(track_id as usize) % colors.len()]
}

pub fn show_mixer(
    ui: &mut egui::Ui,
    project: &mut ProjectConfig,
    selected_track_id: &mut Option<u64>,
    spectrum: Option<&SpectrumAnalyzer>,
) {
    let state_id = ui.id().with("mixer_state");
    let mut state = ui
        .data_mut(|d| d.get_temp::<MixerState>(state_id))
        .unwrap_or_default();

    show_mixer_impl(ui, project, selected_track_id, spectrum, &mut state);

    ui.data_mut(|d| d.insert_temp(state_id, state));
}

pub fn show_mixer_impl(
    ui: &mut egui::Ui,
    project: &mut ProjectConfig,
    selected_track_id: &mut Option<u64>,
    spectrum: Option<&SpectrumAnalyzer>,
    state: &mut MixerState,
) {
    ui.heading("Console Mixer");
    ui.separator();

    let any_soloed = project.tracks.iter().any(|t| t.soloed);

    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for track in &mut project.tracks {
                let is_selected = selected_track_id.map_or(false, |id| id == track.id);
                let is_dimmed = any_soloed && !track.soloed;

                let bg_color = if is_selected {
                    egui::Color32::from_rgb(35, 45, 60)
                } else if is_dimmed {
                    egui::Color32::from_rgb(18, 18, 22)
                } else {
                    egui::Color32::from_rgb(25, 25, 30)
                };

                egui::Frame::window(ui.style())
                    .fill(bg_color)
                    .show(ui, |ui| {
                        ui.set_width(125.0);
                        ui.vertical_centered(|ui| {
                            // Top track color indicator stripe
                            let (stripe_rect, _) = ui
                                .allocate_exact_size(egui::vec2(115.0, 4.0), egui::Sense::hover());
                            ui.painter()
                                .rect_filled(stripe_rect, 2.0, track_color(track.id));

                            ui.add_space(4.0);

                            // Track header button
                            let head_btn = ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&track.name).strong().size(13.0),
                            );
                            if head_btn.clicked() {
                                *selected_track_id = Some(track.id);
                            }

                            ui.separator();

                            // Mute & Solo toggles
                            ui.horizontal(|ui| {
                                let mut mute = track.muted;
                                if ui
                                    .toggle_value(&mut mute, egui::RichText::new("M").strong())
                                    .changed()
                                {
                                    track.muted = mute;
                                }
                                let mut solo = track.soloed;
                                if ui
                                    .toggle_value(&mut solo, egui::RichText::new("S").strong())
                                    .changed()
                                {
                                    track.soloed = solo;
                                }
                            });

                            ui.add_space(4.0);

                            // Send level slider
                            ui.label(egui::RichText::new("Send").size(11.0));
                            ui.add(egui::Slider::new(&mut track.send_level, 0.0..=1.0).text(""));

                            ui.add_space(6.0);

                            // Gain Fader + Vertical VU meter strip with peak hold
                            ui.label(egui::RichText::new("Gain").size(11.0));
                            ui.horizontal(|ui| {
                                let gain_slider = egui::Slider::new(&mut track.gain, 0.0..=2.0)
                                    .orientation(egui::SliderOrientation::Vertical)
                                    .text("");
                                ui.add(gain_slider);

                                // VU Meter bar beside fader
                                let (meter_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(12.0, 100.0),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter();
                                painter.rect_filled(
                                    meter_rect,
                                    2.0,
                                    egui::Color32::from_rgb(12, 12, 16),
                                );

                                let current_level = if track.muted || is_dimmed {
                                    0.0f32
                                } else {
                                    (track.gain * 0.65).min(1.0)
                                };

                                if current_level > 0.0 {
                                    let fill_h = current_level * meter_rect.height();
                                    let fill_r = egui::Rect::from_min_max(
                                        egui::pos2(meter_rect.min.x, meter_rect.max.y - fill_h),
                                        meter_rect.max,
                                    );
                                    let meter_color = if current_level > 0.85 {
                                        egui::Color32::from_rgb(235, 60, 60)
                                    } else if current_level > 0.7 {
                                        egui::Color32::from_rgb(235, 180, 40)
                                    } else {
                                        egui::Color32::from_rgb(40, 200, 80)
                                    };
                                    painter.rect_filled(fill_r, 1.0, meter_color);
                                }

                                // Peak hold line decaying over time
                                let entry = state.peak_holds.entry(track.id).or_insert(0.0);
                                if current_level >= *entry {
                                    *entry = current_level;
                                } else {
                                    *entry = (*entry - 0.01).max(0.0);
                                }
                                let peak_val = *entry;
                                if peak_val > 0.0 {
                                    let peak_y = meter_rect.max.y - peak_val * meter_rect.height();
                                    painter.line_segment(
                                        [
                                            egui::pos2(meter_rect.min.x, peak_y),
                                            egui::pos2(meter_rect.max.x, peak_y),
                                        ],
                                        egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
                                    );
                                }
                            });

                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} dB",
                                    20.0 * track.gain.max(0.0001).log10()
                                ))
                                .size(11.0),
                            );

                            ui.add_space(4.0);

                            // Pan Knob / Slider
                            ui.label(egui::RichText::new("Pan").size(11.0));
                            ui.add(egui::Slider::new(&mut track.pan, -1.0..=1.0).text(""));

                            ui.separator();

                            // Insert FX Button & count
                            if ui.button("➕ Insert FX").clicked() {
                                state.fx_popup_track_id =
                                    if state.fx_popup_track_id == Some(track.id) {
                                        None
                                    } else {
                                        Some(track.id)
                                    };
                            }
                            ui.label(
                                egui::RichText::new(format!("{} FX Nodes", track.nodes.len()))
                                    .size(11.0),
                            );

                            ui.separator();
                            ui.collapsing("Spectrum", |ui| {
                                let dummy_spec = SpectrumAnalyzer::new();
                                let spec_ref = spectrum.unwrap_or(&dummy_spec);
                                show_spectrum(ui, spec_ref, 115.0, 40.0);
                            });
                        });
                    });

                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Master Bus Channel Strip at Right End
            egui::Frame::window(ui.style())
                .fill(egui::Color32::from_rgb(30, 32, 42))
                .stroke(egui::Stroke::new(
                    1.5_f32,
                    egui::Color32::from_rgb(255, 180, 40),
                ))
                .show(ui, |ui| {
                    ui.set_width(125.0);
                    ui.vertical_centered(|ui| {
                        // Master Color Stripe (Gold/Amber)
                        let (stripe_rect, _) =
                            ui.allocate_exact_size(egui::vec2(115.0, 4.0), egui::Sense::hover());
                        ui.painter().rect_filled(
                            stripe_rect,
                            2.0,
                            egui::Color32::from_rgb(255, 180, 40),
                        );

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Master")
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::from_rgb(255, 180, 40)),
                        );
                        ui.separator();

                        // Master Mute
                        let mut m_mute = state.master_muted;
                        if ui
                            .toggle_value(&mut m_mute, egui::RichText::new("M").strong())
                            .changed()
                        {
                            state.master_muted = m_mute;
                        }

                        ui.add_space(18.0);

                        // Master Gain Fader & VU Meter with peak hold
                        ui.label(egui::RichText::new("Master Gain").size(11.0));
                        ui.horizontal(|ui| {
                            let g_slider = egui::Slider::new(&mut state.master_gain, 0.0..=2.0)
                                .orientation(egui::SliderOrientation::Vertical)
                                .text("");
                            ui.add(g_slider);

                            let (meter_rect, _) = ui
                                .allocate_exact_size(egui::vec2(12.0, 100.0), egui::Sense::hover());
                            let painter = ui.painter();
                            painter.rect_filled(
                                meter_rect,
                                2.0,
                                egui::Color32::from_rgb(12, 12, 16),
                            );

                            let m_level = if state.master_muted {
                                0.0
                            } else {
                                (state.master_gain * 0.65).min(1.0)
                            };
                            if m_level > 0.0 {
                                let fill_h = m_level * meter_rect.height();
                                let fill_r = egui::Rect::from_min_max(
                                    egui::pos2(meter_rect.min.x, meter_rect.max.y - fill_h),
                                    meter_rect.max,
                                );
                                painter.rect_filled(
                                    fill_r,
                                    1.0,
                                    egui::Color32::from_rgb(255, 180, 40),
                                );
                            }

                            // Peak hold for master (id 0)
                            let m_entry = state.peak_holds.entry(0).or_insert(0.0);
                            if m_level >= *m_entry {
                                *m_entry = m_level;
                            } else {
                                *m_entry = (*m_entry - 0.01).max(0.0);
                            }
                            let peak_val = *m_entry;
                            if peak_val > 0.0 {
                                let peak_y = meter_rect.max.y - peak_val * meter_rect.height();
                                painter.line_segment(
                                    [
                                        egui::pos2(meter_rect.min.x, peak_y),
                                        egui::pos2(meter_rect.max.x, peak_y),
                                    ],
                                    egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
                                );
                            }
                        });

                        ui.label(
                            egui::RichText::new(format!(
                                "{:.1} dB",
                                20.0 * state.master_gain.max(0.0001).log10()
                            ))
                            .size(11.0),
                        );

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Pan").size(11.0));
                        ui.add(egui::Slider::new(&mut state.master_pan, -1.0..=1.0).text(""));

                        ui.separator();
                        ui.collapsing("Spectrum", |ui| {
                            let dummy_spec = SpectrumAnalyzer::new();
                            let spec_ref = spectrum.unwrap_or(&dummy_spec);
                            show_spectrum(ui, spec_ref, 115.0, 40.0);
                        });
                    });
                });
        });
    });

    // Effect selector popup window
    if let Some(target_tid) = state.fx_popup_track_id {
        egui::Window::new("Insert Effect Node")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("Select effect node to insert:"));
                let effects = [
                    "BiquadFilter",
                    "DistortionNode",
                    "ChorusNode",
                    "FlangerNode",
                    "PhaserNode",
                    "DelayNode",
                    "ReverbNode",
                    "CompressorNode",
                ];
                for fx_kind in effects {
                    if ui.button(fx_kind).clicked() {
                        if let Some(tr) = project.tracks.iter_mut().find(|t| t.id == target_tid) {
                            tr.nodes.push(NodeConfig {
                                kind: fx_kind.to_string(),
                                params: HashMap::new(),
                                plugin_state: None,
                            });
                        }
                        state.fx_popup_track_id = None;
                    }
                }
                if ui.button("Close").clicked() {
                    state.fx_popup_track_id = None;
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_mixer_view_renders_without_panic() {
        let mut project = create_default_project("Test Session");
        let mut selected_track_id = None;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_mixer(ui, &mut project, &mut selected_track_id, None);
            });
        });
    }

    #[test]
    fn test_mixer_solo_toggle() {
        let mut project = create_default_project("Solo Test");
        let mut selected_track_id = None;
        let mut state = MixerState::default();

        assert!(!project.tracks[0].soloed);

        // Toggle solo on track 0
        project.tracks[0].soloed = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_mixer_impl(ui, &mut project, &mut selected_track_id, None, &mut state);
            });
        });

        assert!(project.tracks[0].soloed);
    }

    #[test]
    fn test_mixer_send_level_renders() {
        let mut project = create_default_project("Send Test");
        let mut selected_track_id = None;
        let mut state = MixerState::default();

        project.tracks[0].send_level = 0.75;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_mixer_impl(ui, &mut project, &mut selected_track_id, None, &mut state);
            });
        });

        assert_eq!(project.tracks[0].send_level, 0.75);
    }
}
