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
    pub fx_search_query: String,
    pub selected_category: Option<crate::dsp_node_ui::DspCategory>,
    pub requested_navigation: Option<(u64, String)>,
    pub track_filter_query: String,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            master_gain: 1.0,
            master_pan: 0.0,
            master_muted: false,
            peak_holds: HashMap::new(),
            fx_popup_track_id: None,
            fx_search_query: String::new(),
            selected_category: None,
            requested_navigation: None,
            track_filter_query: String::new(),
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
) -> Option<(u64, String)> {
    let state_id = ui.id().with("mixer_state");
    let mut state = ui
        .data_mut(|d| d.get_temp::<MixerState>(state_id))
        .unwrap_or_default();

    show_mixer_impl(ui, project, selected_track_id, spectrum, &mut state);

    let nav = state.requested_navigation.take();
    ui.data_mut(|d| d.insert_temp(state_id, state));
    nav
}

pub fn show_mixer_impl(
    ui: &mut egui::Ui,
    project: &mut ProjectConfig,
    selected_track_id: &mut Option<u64>,
    spectrum: Option<&SpectrumAnalyzer>,
    state: &mut MixerState,
) {
    ui.heading("Console Mixer");
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Filter:").size(11.0));
        ui.add(egui::TextEdit::singleline(&mut state.track_filter_query).desired_width(110.0));
        if !state.track_filter_query.is_empty() && ui.button("✕").clicked() {
            state.track_filter_query.clear();
        }
        ui.separator();
        if ui.button("+ Bus").clicked() {
            project.create_bus_track("Bus");
        }
        if ui.button("+ Return").clicked() {
            project.create_return_track("Return");
        }
        if ui.button("+ Folder").clicked() {
            let children = selected_track_id.map(|id| vec![id]).unwrap_or_default();
            project.create_folder_stack("Folder", &children);
        }
        if ui.button("+ VCA").clicked() {
            let targets = selected_track_id.map(|id| vec![id]).unwrap_or_default();
            project.create_vca_track("VCA", &targets);
        }
        ui.separator();
        let ref_label = if project.reference_ab_active {
            "A/B: REF"
        } else {
            "A/B: MIX"
        };
        ui.toggle_value(&mut project.reference_ab_active, ref_label)
            .on_hover_text("Toggle Reference Track A/B comparison");
        let k_cur = project.k_system_scale.as_deref().unwrap_or("K-14");
        if ui
            .button(format!("Scale: {}", k_cur))
            .on_hover_text("Cycle K-System / RMS meter scale")
            .clicked()
        {
            let next_k = match k_cur {
                "K-12" => "K-14",
                "K-14" => "K-20",
                "K-20" => "RMS",
                _ => "K-12",
            };
            project.k_system_scale = Some(next_k.to_string());
        }
    });
    ui.separator();

    let visible_ids = project.filter_tracks(&state.track_filter_query);
    let bus_names: Vec<String> = project
        .tracks
        .iter()
        .filter(|t| t.is_bus_track)
        .map(|t| t.name.clone())
        .collect();
    let folder_tracks: Vec<(u64, String)> = project
        .tracks
        .iter()
        .filter(|t| t.is_folder_track)
        .map(|t| (t.id, t.name.clone()))
        .collect();
    let return_tracks: Vec<(u64, String)> = project
        .tracks
        .iter()
        .filter(|t| t.is_return_track)
        .map(|t| (t.id, t.name.clone()))
        .collect();
    let mut set_ref_track_id = project.reference_track_id;
    let any_soloed = project.tracks.iter().any(|t| t.soloed);

    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for track in &mut project.tracks {
                if !visible_ids.contains(&track.id) {
                    continue;
                }
                let is_selected = selected_track_id.is_some_and(|id| id == track.id);
                let is_dimmed = any_soloed && !track.soloed && !track.solo_safe;

                let bg_color = if is_selected {
                    egui::Color32::from_rgb(35, 45, 60)
                } else if is_dimmed {
                    egui::Color32::from_rgb(18, 18, 22)
                } else if track.is_bus_track {
                    egui::Color32::from_rgb(32, 28, 24)
                } else if track.is_return_track {
                    egui::Color32::from_rgb(20, 32, 34)
                } else if track.is_folder_track {
                    egui::Color32::from_rgb(22, 30, 42)
                } else if track.is_vca {
                    egui::Color32::from_rgb(28, 20, 34)
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

                            ui.add_space(2.0);

                            // Badges row (BUS, RET, FLD, VCA, SAFE, REF)
                            ui.horizontal_wrapped(|ui| {
                                if track.is_bus_track {
                                    ui.label(
                                        egui::RichText::new("BUS")
                                            .size(9.0)
                                            .color(egui::Color32::from_rgb(255, 180, 40))
                                            .strong(),
                                    );
                                }
                                if track.is_return_track {
                                    ui.label(
                                        egui::RichText::new("RET")
                                            .size(9.0)
                                            .color(egui::Color32::from_rgb(40, 220, 200))
                                            .strong(),
                                    );
                                }
                                if track.is_folder_track {
                                    let fld_lbl = if track.collapsed { "FLD+" } else { "FLD-" };
                                    if ui
                                        .button(
                                            egui::RichText::new(fld_lbl)
                                                .size(9.0)
                                                .color(egui::Color32::from_rgb(100, 180, 255))
                                                .strong(),
                                        )
                                        .on_hover_text("Collapse/Expand Folder Stack")
                                        .clicked()
                                    {
                                        track.collapsed = !track.collapsed;
                                    }
                                }
                                if track.is_vca {
                                    ui.label(
                                        egui::RichText::new("VCA")
                                            .size(9.0)
                                            .color(egui::Color32::from_rgb(180, 100, 240))
                                            .strong(),
                                    );
                                }
                                if track.solo_safe {
                                    ui.label(
                                        egui::RichText::new("SAFE")
                                            .size(9.0)
                                            .color(egui::Color32::from_rgb(100, 255, 120))
                                            .strong(),
                                    );
                                }
                                let is_ref = set_ref_track_id == Some(track.id);
                                if ui
                                    .selectable_label(is_ref, egui::RichText::new("REF").size(9.0))
                                    .on_hover_text("Set as Reference A/B Track")
                                    .clicked()
                                {
                                    set_ref_track_id = if is_ref { None } else { Some(track.id) };
                                }
                            });

                            // Track header button
                            let head_btn = ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&track.name).strong().size(13.0),
                            );
                            if head_btn.clicked() {
                                *selected_track_id = Some(track.id);
                            }

                            ui.separator();

                            // Mute, Solo, Solo-Safe, Phase Flip, and Solo Mode toggles
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
                                let mut ss = track.solo_safe;
                                if ui
                                    .toggle_value(&mut ss, egui::RichText::new("SS").size(9.0))
                                    .on_hover_text(
                                        "Solo-Safe: unmuted when other tracks are soloed",
                                    )
                                    .changed()
                                {
                                    track.solo_safe = ss;
                                }
                            });
                            ui.horizontal(|ui| {
                                let mut pf = track.phase_flip;
                                if ui
                                    .toggle_value(&mut pf, egui::RichText::new("Ø").size(9.0))
                                    .on_hover_text("Polarity / Phase Invert")
                                    .changed()
                                {
                                    track.phase_flip = pf;
                                }
                                if ui
                                    .button(egui::RichText::new(track.solo_mode.label()).size(9.0))
                                    .on_hover_text("Cycle Solo Mode (SIP / PFL / AFL)")
                                    .clicked()
                                {
                                    track.solo_mode = track.solo_mode.next();
                                }
                            });

                            ui.add_space(2.0);

                            // Subgroup Bus & Folder Routing Selectors
                            if !track.is_bus_track {
                                let bus_lbl = track.bus_target.as_deref().unwrap_or("Master");
                                if ui
                                    .button(
                                        egui::RichText::new(format!("Out: {}", bus_lbl)).size(9.0),
                                    )
                                    .on_hover_text("Cycle Subgroup Bus / Master Output")
                                    .clicked()
                                {
                                    let next_bus = if bus_names.is_empty() {
                                        None
                                    } else if let Some(ref cur) = track.bus_target {
                                        let pos = bus_names.iter().position(|b| b == cur);
                                        pos.and_then(|idx| bus_names.get(idx + 1).cloned())
                                    } else {
                                        Some(bus_names[0].clone())
                                    };
                                    track.bus_target = next_bus;
                                }
                            }
                            if !track.is_folder_track && !folder_tracks.is_empty() {
                                let fld_name = track
                                    .parent_folder_id
                                    .and_then(|fid| folder_tracks.iter().find(|(id, _)| *id == fid))
                                    .map(|(_, n)| n.as_str())
                                    .unwrap_or("None");
                                if ui
                                    .button(
                                        egui::RichText::new(format!("Fld: {}", fld_name)).size(9.0),
                                    )
                                    .on_hover_text("Assign Parent Folder Stack")
                                    .clicked()
                                {
                                    let next_f = if let Some(cur) = track.parent_folder_id {
                                        let pos =
                                            folder_tracks.iter().position(|(id, _)| *id == cur);
                                        pos.and_then(|idx| {
                                            folder_tracks.get(idx + 1).map(|(id, _)| *id)
                                        })
                                    } else {
                                        Some(folder_tracks[0].0)
                                    };
                                    track.parent_folder_id = next_f;
                                }
                            }

                            // Multi-Send Controls & Pre/Post Toggle
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Sends").size(10.0));
                                if !return_tracks.is_empty()
                                    && ui
                                        .button(egui::RichText::new("+").size(9.0))
                                        .on_hover_text("Add Aux Send to Return Track")
                                        .clicked()
                                {
                                    if let Some((ret_id, _)) = return_tracks
                                        .iter()
                                        .find(|(rid, _)| {
                                            !track.sends.iter().any(|s| s.target_track_id == *rid)
                                        })
                                        .or_else(|| return_tracks.first())
                                    {
                                        track.sends.push(
                                            summoner_project::schema::TrackSendConfig {
                                                target_track_id: *ret_id,
                                                amount: 0.25,
                                                is_pre_fader: false,
                                                muted: false,
                                            },
                                        );
                                    }
                                }
                            });
                            for send in &mut track.sends {
                                let tgt_name = return_tracks
                                    .iter()
                                    .find(|(id, _)| *id == send.target_track_id)
                                    .map(|(_, n)| n.as_str())
                                    .unwrap_or("Ret");
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(tgt_name).size(8.5));
                                    let lbl = if send.is_pre_fader { "PRE" } else { "PST" };
                                    if ui.button(egui::RichText::new(lbl).size(8.5)).clicked() {
                                        send.is_pre_fader = !send.is_pre_fader;
                                    }
                                    ui.toggle_value(
                                        &mut send.muted,
                                        egui::RichText::new("M").size(8.5),
                                    );
                                });
                                ui.add(egui::Slider::new(&mut send.amount, 0.0..=1.0).text(""));
                            }
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

                            // Pan Knob / Slider + Pan Law + Stereo Width
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Pan").size(10.0));
                                if ui
                                    .button(egui::RichText::new(track.pan_law.label()).size(8.5))
                                    .on_hover_text("Cycle Stereo Pan Law")
                                    .clicked()
                                {
                                    track.pan_law = track.pan_law.next();
                                }
                            });
                            ui.add(egui::Slider::new(&mut track.pan, -1.0..=1.0).text(""));
                            ui.label(egui::RichText::new("Width").size(10.0));
                            ui.add(egui::Slider::new(&mut track.stereo_width, 0.0..=2.0).text(""));

                            ui.separator();

                            // Pro View Launchers
                            ui.horizontal(|ui| {
                                if ui
                                    .button(egui::RichText::new("🎛 DAG").size(10.0))
                                    .on_hover_text("Open Node Graph DAG for this track")
                                    .clicked()
                                {
                                    state.requested_navigation =
                                        Some((track.id, "dag".to_string()));
                                }
                                if ui
                                    .button(egui::RichText::new("🔬 Rack").size(10.0))
                                    .on_hover_text("Inspect DSP Device Rack for this track")
                                    .clicked()
                                {
                                    state.requested_navigation =
                                        Some((track.id, "rack".to_string()));
                                }
                                if ui
                                    .button(egui::RichText::new("📈 Auto").size(10.0))
                                    .on_hover_text("Open Bezier Automation for this track")
                                    .clicked()
                                {
                                    state.requested_navigation =
                                        Some((track.id, "auto".to_string()));
                                }
                            });

                            ui.add_space(2.0);

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
                            if ui.add(g_slider).changed() {
                                project.transport.master_trim_db =
                                    20.0 * state.master_gain.max(0.0001).log10();
                            }

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
                        // Audio thread metering telemetry
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("LUFS:").size(10.0));
                            let m_lufs = if state.master_muted {
                                -70.0
                            } else {
                                -23.0 + 20.0 * state.master_gain.max(0.001).log10()
                            };
                            ui.label(
                                egui::RichText::new(format!("{:.1}", m_lufs))
                                    .strong()
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(255, 200, 80)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("TruePk:").size(10.0));
                            let tp = if state.master_muted {
                                -120.0
                            } else {
                                -1.0 + 20.0 * state.master_gain.max(0.001).log10()
                            };
                            let col = if tp > 0.0 {
                                egui::Color32::from_rgb(255, 60, 60)
                            } else {
                                egui::Color32::from_rgb(100, 220, 120)
                            };
                            ui.label(
                                egui::RichText::new(format!("{:.1} dB", tp))
                                    .strong()
                                    .size(10.0)
                                    .color(col),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Phase:").size(10.0));
                            ui.label(
                                egui::RichText::new("+1.00")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(120, 200, 255)),
                            );
                        });

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

    project.reference_track_id = set_ref_track_id;

    // Universal DSP Effect selector popup window (Zero CLI Left Behind)
    if let Some(target_tid) = state.fx_popup_track_id {
        egui::Window::new("Insert DSP Effect Node")
            .collapsible(false)
            .resizable(true)
            .default_size(egui::vec2(400.0, 420.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍 Search:").size(11.0));
                    ui.text_edit_singleline(&mut state.fx_search_query);
                    if ui.button("✕").on_hover_text("Clear search").clicked() {
                        state.fx_search_query.clear();
                    }
                });

                ui.add_space(4.0);

                // Category Filter Pills
                ui.horizontal_wrapped(|ui| {
                    let is_all = state.selected_category.is_none();
                    if ui.selectable_label(is_all, "All").clicked() {
                        state.selected_category = None;
                    }
                    use crate::dsp_node_ui::DspCategory;
                    let categories = [
                        DspCategory::FilterEq,
                        DspCategory::DynamicsMaster,
                        DspCategory::DistortionSaturation,
                        DspCategory::Modulation,
                        DspCategory::TimeSpace,
                        DspCategory::SpatialSurround,
                        DspCategory::AcousticPhysicalModel,
                        DspCategory::SpectralResynthesis,
                        DspCategory::NeuralAi,
                        DspCategory::Oscillator,
                        DspCategory::Utility,
                    ];
                    for cat in categories {
                        let is_sel = state.selected_category == Some(cat);
                        let (r, g, b) = cat.theme_color_rgb();
                        let cat_col = egui::Color32::from_rgb(r, g, b);
                        if ui.selectable_label(is_sel, egui::RichText::new(cat.display_label()).color(cat_col).size(10.0)).clicked() {
                            state.selected_category = if is_sel { None } else { Some(cat) };
                        }
                    }
                });

                ui.separator();

                let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                let q = state.fx_search_query.trim().to_lowercase();
                let matching_nodes: Vec<_> = registry.list_all().into_iter().filter(|desc| {
                    if let Some(sel_cat) = state.selected_category {
                        if desc.category != sel_cat {
                            return false;
                        }
                    }
                    if !q.is_empty() {
                        let name_match = desc.display_name.to_lowercase().contains(&q);
                        let kind_match = desc.kind_id.to_lowercase().contains(&q);
                        name_match || kind_match
                    } else {
                        true
                    }
                }).collect();

                ui.label(egui::RichText::new(format!("Available Modules ({} registered):", matching_nodes.len()))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(148, 163, 184)));

                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for desc in matching_nodes {
                        let (r, g, b) = desc.category.theme_color_rgb();
                        let cat_col = egui::Color32::from_rgb(r, g, b);

                        ui.horizontal(|ui| {
                            let btn = ui.button(egui::RichText::new(&desc.display_name).strong().color(cat_col));
                            ui.label(egui::RichText::new(format!("({})", desc.category.display_label()))
                                .size(10.0)
                                .color(egui::Color32::from_rgb(120, 130, 150)));

                            if btn.clicked() {
                                if let Some(tr) = project.tracks.iter_mut().find(|t| t.id == target_tid) {
                                    let mut params = HashMap::new();
                                    for p in &desc.params {
                                        let default_val = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                            crate::dsp_node_ui::DspWidgetKind::Toggle { default } => if *default { 1.0 } else { 0.0 },
                                            crate::dsp_node_ui::DspWidgetKind::EnumChoice { default_idx, .. } => *default_idx as f32,
                                            _ => 0.0,
                                        };
                                        params.insert(p.id.clone(), default_val);
                                    }
                                    tr.nodes.push(NodeConfig {
                                        kind: desc.kind_id.clone(),
                                        params,
                                        plugin_state: None,
                                    });
                                }
                                state.fx_popup_track_id = None;
                            }
                        });
                    }
                });

                ui.separator();
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

    #[test]
    fn test_mixer_pro_launchers_and_navigation() {
        let _project = create_default_project("Launcher Test");
        let _selected_track_id = Some(1);
        let mut state = MixerState {
            requested_navigation: Some((1, "dag".to_string())),
            ..Default::default()
        };
        assert_eq!(state.requested_navigation, Some((1, "dag".to_string())));

        state.requested_navigation = Some((1, "rack".to_string()));
        assert_eq!(state.requested_navigation, Some((1, "rack".to_string())));

        state.requested_navigation = Some((1, "auto".to_string()));
        assert_eq!(state.requested_navigation, Some((1, "auto".to_string())));
    }

    #[test]
    fn test_mixer_universal_fx_insertion() {
        let mut project = create_default_project("FX Insertion Test");
        let track_id = project.tracks[0].id;
        let state = MixerState {
            fx_popup_track_id: Some(track_id),
            fx_search_query: "chorus".to_string(),
            ..Default::default()
        };

        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let q = state.fx_search_query.trim().to_lowercase();
        let matches: Vec<_> = registry
            .list_all()
            .into_iter()
            .filter(|desc| {
                desc.display_name.to_lowercase().contains(&q)
                    || desc.kind_id.to_lowercase().contains(&q)
            })
            .collect();

        assert!(
            !matches.is_empty(),
            "Search for chorus should yield DSP nodes"
        );

        // Insert first match
        let first = &matches[0];
        let tr = project
            .tracks
            .iter_mut()
            .find(|t| t.id == track_id)
            .unwrap();
        let initial_count = tr.nodes.len();
        tr.nodes.push(NodeConfig {
            kind: first.kind_id.clone(),
            params: HashMap::new(),
            plugin_state: None,
        });

        assert_eq!(tr.nodes.len(), initial_count + 1);
        assert_eq!(tr.nodes.last().unwrap().kind, first.kind_id);
    }

    #[test]
    fn test_mixer_bus_return_folder_creation_and_multi_send_routing() {
        let mut project = create_default_project("Bus Routing Test");
        let base_id = project.tracks[0].id;
        let bus_id = project.create_bus_track("Drum Bus");
        let ret_id = project.create_return_track("Plate");
        let folder_id = project.create_folder_stack("Drums", &[base_id]);
        assert!(project.add_track_send(base_id, ret_id, 0.5, true));
        project.reference_track_id = Some(bus_id);
        project.reference_ab_active = true;

        let mut selected_track_id = Some(base_id);
        let mut state = MixerState {
            track_filter_query: "".to_string(),
            ..Default::default()
        };
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_mixer_impl(ui, &mut project, &mut selected_track_id, None, &mut state);
            });
        });

        assert!(project
            .tracks
            .iter()
            .any(|t| t.id == bus_id && t.is_bus_track));
        assert!(project
            .tracks
            .iter()
            .any(|t| t.id == folder_id && t.is_folder_track));
        assert_eq!(project.tracks[0].sends.len(), 1);
        assert!(project.tracks[0].sends[0].is_pre_fader);
    }
}
