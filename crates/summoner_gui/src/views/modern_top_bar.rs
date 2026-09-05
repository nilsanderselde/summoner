// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modern Top Bar with Global Transport, Stats Readout, VU Meters & Mode Switching.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModernViewTab {
    #[default]
    Arranger,
    PianoRoll,
    Modular,
    Mixer,
    Performance,
}

impl ModernViewTab {
    pub fn name(&self) -> &'static str {
        match self {
            ModernViewTab::Arranger => "ARRANGER",
            ModernViewTab::PianoRoll => "PIANO ROLL",
            ModernViewTab::Modular => "MODULAR",
            ModernViewTab::Mixer => "MIXER",
            ModernViewTab::Performance => "STAGE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernTopBarState {
    pub bpm: f64,
    pub key_signature: String,
    pub time_signature: String,
    pub cpu_percent: f32,
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub master_peak_db: f32,
    pub active_tab: ModernViewTab,
    #[serde(default = "default_selected_preset")]
    pub selected_preset: String,
    #[serde(default = "default_available_presets")]
    pub available_presets: Vec<String>,
    #[serde(default = "default_master_gain")]
    pub master_gain: f32,
    #[serde(default = "default_macro_tone")]
    pub macro_tone: f32,
    #[serde(default = "default_macro_space")]
    pub macro_space: f32,
    #[serde(default = "default_macro_punch")]
    pub macro_punch: f32,
    #[serde(default = "default_macro_character")]
    pub macro_character: f32,
    #[serde(default = "default_true_novice")]
    pub is_novice_macro_visible: bool,
}

fn default_master_gain() -> f32 {
    1.0
}

fn default_macro_tone() -> f32 {
    0.65
}

fn default_macro_space() -> f32 {
    0.40
}

fn default_macro_punch() -> f32 {
    0.55
}

fn default_macro_character() -> f32 {
    0.50
}

fn default_true_novice() -> bool {
    true
}

fn default_selected_preset() -> String {
    "Init Synth 1".to_string()
}

fn default_available_presets() -> Vec<String> {
    vec![
        "Init Synth 1".to_string(),
        "Aether Warm Pad".to_string(),
        "808 Sub Kick".to_string(),
        "Vintage Tape Lead".to_string(),
        "Karplus Acoustic Pluck".to_string(),
        "Neural Vocal Demucs".to_string(),
        "Ambient Crystal Bells".to_string(),
        "Lo-Fi Breakbeat".to_string(),
        "Cathedral Pipe Organ".to_string(),
        "Cosmic Shockwave Reverb".to_string(),
        "Neuro HRV Bio-Sync".to_string(),
        "Analog Tape Stop".to_string(),
        "Orchestral Timpani Drum".to_string(),
        "Concert Rosewood Marimba".to_string(),
        "Bourbonnais Vielle Gurdy".to_string(),
        "Silk String Japanese Koto".to_string(),
        "Hindustani Ravi Sitar".to_string(),
        "Rhodes Classic Mark I".to_string(),
        "Hurdy-Gurdy Crank Wheel".to_string(),
        "Concert Snare Drum Rattle".to_string(),
        "Indian Raga Chikari Drone".to_string(),
        "Franklin Water Armonica".to_string(),
        "Concert Grand Felt Hammer".to_string(),
        "Sitka Spruce Resonance".to_string(),
        "Hohner D6 Funk Clavinet".to_string(),
        "Indian Tarab Sympathetic".to_string(),
        "Neural AI Room Impulse".to_string(),
        "AI Voice-Leading Master".to_string(),
        "Quantum Hyperbolic Reverb".to_string(),
        "Neuro Relaxation Feedback".to_string(),
        "Acoustic Cloaking Soundfield".to_string(),
        "Sub-Harmonic Quantum Tunnel".to_string(),
        "EBU R128 Master Broadcast".to_string(),
        "Dolby Atmos Bed Splitter".to_string(),
        "Procedural Raytraced Hall".to_string(),
        "Polymetric Euclidean Groove".to_string(),
        "Markov Generative Matrix".to_string(),
        "3D Atmos Trajectory Orbit".to_string(),
        "Concert Grand Piano Escapement".to_string(),
        "Bowed Violin Kinematics".to_string(),
        "Concert Acoustic Harp Arpeggio".to_string(),
        "Hurdy Gurdy Chien Drone".to_string(),
        "Vocal Tract Formant Shaper".to_string(),
        "Electric Piano Tine Saturation".to_string(),
        "Karplus Plucked Nylon Resonator".to_string(),
        "Analog Reel-to-Reel Tape Saturation".to_string(),
        "Shoebox Early Acoustic Reflections".to_string(),
        "MPE Polyphonic Gesture Router".to_string(),
        "Multitrack Ambient Soundscape Loop".to_string(),
        "Harmonium Free-Reed Expression Swell".to_string(),
        "Glass Armonica Celestial Shimmer".to_string(),
        "High-Gain WaveNet Tube Overdrive".to_string(),
        "Concert Hall Bowed Double Bass".to_string(),
        "Tuned Orchestral Timpani Membrane".to_string(),
        "Vintage 91-Wheel Jazz Tonewheel Organ".to_string(),
        "Master Brickwall True-Peak Limiter".to_string(),
        "Concert Grand Imperial Steinway".to_string(),
        "Stevie 70s Clavinet Wah Funk".to_string(),
        "Plate Tank Mechanical Reverb".to_string(),
        "Zen Bamboo Flute Breath".to_string(),
        "Neo-Riemannian Cadence Graph".to_string(),
        "Bjorklund Euclidean Rhythm Engine".to_string(),
        "Microtonal 31-EDO Harmonic Scale".to_string(),
        "Acoustic Flamenco Guitar Strummer".to_string(),
        "Just Intonation Concordance Lattice".to_string(),
        "Bowed Cello Friction Orbit".to_string(),
        "Moog 4-Pole Ladder Self-Oscillation".to_string(),
        "Leslie 122 Dual Rotor Doppler".to_string(),
        "Neural High-Gain Amp Stack".to_string(),
        "Spectrogram Visual Audio Canvas".to_string(),
        "Cathedral Pipe Organ Windchest".to_string(),
        "Tactile 16-Pad Groove Machine".to_string(),
        "Benjamin Franklin Glass Armonica".to_string(),
        "Multi-Zone SFZ Orchestra Sample Patch".to_string(),
        "Lua Real-Time DSP Script Controller".to_string(),
        "Zero-Latency Plugin Sandbox Guard".to_string(),
        "Ultra-Low Latency Opus Audio Relay".to_string(),
        "Real-Time Session Telemetry & Headroom HUD".to_string(),
        "Interactive Lua DSP Profiler & Flamegraph".to_string(),
        "Git DAG Branching & Undo Timeline HUD".to_string(),
        "Live Session Matrix Clip Launcher".to_string(),
        "AI Spectral Unmasking & Mix Balance HUD".to_string(),
        "Hexagonal Wicki-Hayden Isomorphic Keyboard".to_string(),
        "Polymetric Euclidean Tracker Groove Matrix".to_string(),
        "WASM & AudioWorklet Standalone Packager".to_string(),
        "ITU-R BS.2076 ADM Dolby Atmos Spatial Exporter".to_string(),
        "AI Harmonic Cadence & Leading Tone Suggester".to_string(),
        "Adaptive Automation Bézier Spline Thinning HUD".to_string(),
        "Wolfram Rule 30 Cellular Rhythm Matrix".to_string(),
        "ONNX Neural Diatonic Melody Composer".to_string(),
        "Demucs 4-Stem Deep Learning Splitter".to_string(),
        "Standalone CLAP Audio Plugin Bundler".to_string(),
        "Privacy-Preserving Federated Mix Engine".to_string(),
        "Sample-Accurate Multi-Take Studio Vocal Comp".to_string(),
        "Dynamic L1/L2 Cadence Tree Path Resolver".to_string(),
        "44-Cylinder Acoustic Vocal Tract Synthesizer".to_string(),
        "WebAssembly AudioWorklet Real-Time Bundle".to_string(),
        "Psychoacoustic Lipshitz Mastering Dither".to_string(),
        "Multi-Band Attack & Sustain Transient Sculptor".to_string(),
        "Spherical Harmonic 3D Binaural HRTF Soundfield".to_string(),
        "Real-Time CRDT Multi-User Session".to_string(),
        "Ultrasonic Phased-Array Levitation Soundfield".to_string(),
        "Non-Linear Follow Action Matrix Sequencer".to_string(),
        "Concert Grand Piano Escapement Trajectory".to_string(),
        "Neural Amp Modeler WaveNet Studio Rig".to_string(),
        "Visual Spectrogram Image Sonification Canvas".to_string(),
        "Multi-Track DAW Transport & Graph Engine".to_string(),
        "Polyphonic Voice Pool & Micro-Timing Sequencer".to_string(),
        "Live Session Clip Matrix & Scene Launcher".to_string(),
        "Vocal Multi-Take Comping & Crossfade Editor".to_string(),
        "Generative Markov Algorithmic Melodic Mutator".to_string(),
        "Broadcast Multi-Stem Master Export Station".to_string(),
        "Concert Grand Piano Acoustic Resonance Model".to_string(),
        "Cathedral Pipe Organ Tracker Wind Chest".to_string(),
        "Hurdy-Gurdy Rosin Friction & Drone Resonator".to_string(),
        "Continuous Human Breath & Wind Controller".to_string(),
        "Hurdy-Gurdy Continuous Crank & Buzzing Chien Gesture".to_string(),
        "6-DOF Acoustic Articulation Bowing Dynamics Bus".to_string(),
        "Phonetic IPA Vowel Formant Trajectory Matrix".to_string(),
        "Embedded Hardware Supervisor & Thermal Throttling".to_string(),
        "Live Session Lossless WAV Disk Recorder Station".to_string(),
        "AI Multi-Track Frequency Collision & Masking Inspector".to_string(),
        "Acoustic Tonehole 3-Port Scattering Wave Matrix".to_string(),
        "Quantum Bloch Sphere Density Matrix Tomography".to_string(),
        "Neo-Riemannian Tonnetz Harmonic Matrix Synthesizer".to_string(),
        "CLAP Sub-Sample Precision Automation & MPE Rig".to_string(),
        "Multi-Velocity Layered Acoustic Studio Drum Station".to_string(),
        "3D Binaural Spatial Soundfield Vector Tracker".to_string(),
    ]
}

impl Default for ModernTopBarState {
    fn default() -> Self {
        Self {
            bpm: 120.00,
            key_signature: "Am".to_string(),
            time_signature: "4/4".to_string(),
            cpu_percent: 12.0,
            is_playing: false,
            is_recording: false,
            is_looping: true,
            master_peak_db: 1.2,
            active_tab: ModernViewTab::Arranger,
            selected_preset: default_selected_preset(),
            available_presets: default_available_presets(),
            master_gain: default_master_gain(),
            macro_tone: default_macro_tone(),
            macro_space: default_macro_space(),
            macro_punch: default_macro_punch(),
            macro_character: default_macro_character(),
            is_novice_macro_visible: default_true_novice(),
        }
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_top_bar(
    ui: &mut egui::Ui,
    state: &mut ModernTopBarState,
    on_menu_click: impl FnOnce(),
) {
    let bar_height = 48.0;
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), bar_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            // Background frame
            let full_rect = ui.available_rect_before_wrap();
            let rect = Rect::from_min_size(full_rect.min, Vec2::new(ui.available_width(), bar_height));
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(10, 14, 24));
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)),
            );

            ui.add_space(12.0);

            // Brand Logo & Menu Button
            let logo_resp = ui.button(RichText::new("⚡").font(FontId::proportional(18.0)).color(Color32::from_rgb(56, 189, 248)));
            if logo_resp.clicked() {
                on_menu_click();
            }

            let menu_btn = ui.add(
                egui::Button::new(RichText::new("Menu").font(FontId::proportional(12.0)).color(Color32::from_rgb(220, 230, 245)))
                    .fill(Color32::from_rgb(24, 34, 52))
                    .rounding(Rounding::same(4.0))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
            );
            if menu_btn.clicked() {
                // Open menu
            }

            ui.add_space(12.0);

            // Quick Preset Selector (Novice Top-Level UX)
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 20, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Preset:").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                        egui::ComboBox::from_id_source("top_bar_preset_selector")
                            .selected_text(RichText::new(&state.selected_preset).font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(56, 189, 248)))
                            .show_ui(ui, |ui| {
                                for preset in &state.available_presets {
                                    let is_sel = state.selected_preset == *preset;
                                    if ui.selectable_label(is_sel, preset).clicked() {
                                        state.selected_preset = preset.clone();
                                    }
                                }
                            });
                    });
                });

            ui.add_space(14.0);

            // Transport Stats Group (BPM, Key, Sig, CPU)
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 20, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // BPM
                        ui.vertical(|ui| {
                            ui.label(RichText::new("BPM").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(format!("{:.2}", state.bpm)).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // Key
                        ui.vertical(|ui| {
                            ui.label(RichText::new("KEY").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(&state.key_signature).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // Time Sig
                        ui.vertical(|ui| {
                            ui.label(RichText::new("SIG").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(&state.time_signature).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                        });

                        ui.add_space(8.0);

                        // CPU Meter
                        ui.vertical(|ui| {
                            ui.label(RichText::new("CPU").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{:.0}%", state.cpu_percent)).font(FontId::proportional(11.0)).color(Color32::from_rgb(56, 189, 248)));
                                let (cpu_resp, cpu_painter) = ui.allocate_painter(Vec2::new(24.0, 4.0), egui::Sense::hover());
                                cpu_painter.rect_filled(cpu_resp.rect, 1.0, Color32::from_rgb(24, 34, 52));
                                let fill_w = (state.cpu_percent / 100.0).clamp(0.0, 1.0) * 24.0;
                                cpu_painter.rect_filled(
                                    Rect::from_min_size(cpu_resp.rect.min, Vec2::new(fill_w, 4.0)),
                                    1.0,
                                    Color32::from_rgb(16, 185, 129),
                                );
                            });
                        });
                    });
                });

            ui.add_space(16.0);

            // Transport Control Buttons (Play, Pause, Record, Loop)
            ui.horizontal(|ui| {
                // Play Button
                let play_fill = if state.is_playing {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(24, 34, 52)
                };
                let play_text = if state.is_playing { Color32::BLACK } else { Color32::from_rgb(56, 189, 248) };
                let play_btn = ui.add(
                    egui::Button::new(RichText::new("▶").font(FontId::proportional(13.0)).color(play_text))
                        .fill(play_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                );
                if play_btn.clicked() {
                    state.is_playing = !state.is_playing;
                }

                // Pause / Stop
                let stop_btn = ui.add(
                    egui::Button::new(RichText::new("⏸").font(FontId::proportional(12.0)).color(Color32::from_rgb(200, 215, 235)))
                        .fill(Color32::from_rgb(24, 34, 52))
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                );
                if stop_btn.clicked() {
                    state.is_playing = false;
                }

                // Record (Red Pill)
                let rec_fill = if state.is_recording {
                    Color32::from_rgb(239, 68, 68)
                } else {
                    Color32::from_rgb(45, 20, 25)
                };
                let rec_btn = ui.add(
                    egui::Button::new(RichText::new("🔴").font(FontId::proportional(11.0)).color(Color32::from_rgb(239, 68, 68)))
                        .fill(rec_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0_f32, if state.is_recording { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(80, 25, 30) }))
                );
                if rec_btn.clicked() {
                    state.is_recording = !state.is_recording;
                }

                // Loop Toggle
                let loop_fill = if state.is_looping {
                    Color32::from_rgba_unmultiplied(56, 189, 248, 40)
                } else {
                    Color32::from_rgb(24, 34, 52)
                };
                let loop_stroke = if state.is_looping {
                    Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248))
                } else {
                    Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74))
                };
                let loop_btn = ui.add(
                    egui::Button::new(RichText::new("🔁").font(FontId::proportional(11.0)).color(Color32::from_rgb(56, 189, 248)))
                        .fill(loop_fill)
                        .min_size(Vec2::new(32.0, 28.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(loop_stroke)
                );
                if loop_btn.clicked() {
                    state.is_looping = !state.is_looping;
                }
            });

            // Novice High-Level Macro Strip (Tone, Space, Punch, Character)
            if state.is_novice_macro_visible {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(14, 20, 32))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            draw_top_bar_macro_dial(ui, "Tone", &mut state.macro_tone, Color32::from_rgb(56, 189, 248), "High-level brightness / frequency tone shaper");
                            ui.add_space(2.0);
                            draw_top_bar_macro_dial(ui, "Space", &mut state.macro_space, Color32::from_rgb(99, 102, 241), "High-level spatial depth, reverb & delay diffusion");
                            ui.add_space(2.0);
                            draw_top_bar_macro_dial(ui, "Punch", &mut state.macro_punch, Color32::from_rgb(239, 68, 68), "High-level transient impact, dynamics & attack");
                            ui.add_space(2.0);
                            draw_top_bar_macro_dial(ui, "Char", &mut state.macro_character, Color32::from_rgb(245, 158, 11), "High-level harmonic warmth, saturation & color");
                        });
                    });
            }

            ui.add_space(10.0);

            // Master Volume Fader & Multi-Segment Gradient VU Meter
            ui.horizontal(|ui| {
                ui.label(RichText::new("Master").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));

                // Tactile Master Volume Slider
                let mut norm_gain = (state.master_gain / 2.0).clamp(0.0, 1.0);
                let (gain_resp, gain_painter) = ui.allocate_painter(Vec2::new(42.0, 10.0), egui::Sense::click_and_drag());
                let gain_rect = gain_resp.rect;
                if gain_resp.dragged() {
                    let d_x = ui.input(|i| i.pointer.delta().x);
                    norm_gain = (norm_gain + d_x * 0.015).clamp(0.0, 1.0);
                    state.master_gain = norm_gain * 2.0;
                }
                gain_painter.rect_filled(gain_rect, 2.0, Color32::from_rgb(14, 20, 32));
                gain_painter.rect_stroke(gain_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));
                let fill_w = gain_rect.width() * norm_gain;
                if fill_w > 0.0 {
                    gain_painter.rect_filled(
                        Rect::from_min_size(gain_rect.min, Vec2::new(fill_w, gain_rect.height())),
                        2.0,
                        Color32::from_rgb(56, 189, 248),
                    );
                }

                let (vu_resp, vu_painter) = ui.allocate_painter(Vec2::new(80.0, 10.0), egui::Sense::hover());
                let vu_rect = vu_resp.rect;
                vu_painter.rect_filled(vu_rect, 2.0, Color32::from_rgb(14, 20, 32));
                vu_painter.rect_stroke(vu_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));

                // Draw gradient bars: Green (0..60%) -> Yellow (60..80%) -> Orange (80..95%) -> Pink (95..100%)
                let segments = [
                    (0.0, 0.60, Color32::from_rgb(16, 185, 129)),
                    (0.60, 0.80, Color32::from_rgb(234, 179, 8)),
                    (0.80, 0.95, Color32::from_rgb(249, 115, 22)),
                    (0.95, 1.00, Color32::from_rgb(236, 72, 153)),
                ];
                for (s_start, s_end, color) in segments {
                    let seg_left = vu_rect.left() + (vu_rect.width() * s_start);
                    let seg_w = vu_rect.width() * (s_end - s_start);
                    let seg_rect = Rect::from_min_size(egui::pos2(seg_left, vu_rect.top() + 1.0), Vec2::new(seg_w, vu_rect.height() - 2.0));
                    vu_painter.rect_filled(seg_rect, 1.0, color);
                }

                // Peak dB readout
                let peak_color = if state.master_peak_db > 0.0 {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(16, 185, 129)
                };
                ui.label(RichText::new(format!("+{:.1}dB", state.master_peak_db)).font(FontId::proportional(10.0)).strong().color(peak_color));
            });

            // Right-aligned Tab Mode Switcher
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);

                let tabs = [
                    ModernViewTab::Mixer,
                    ModernViewTab::Modular,
                    ModernViewTab::PianoRoll,
                    ModernViewTab::Arranger,
                ];

                for tab in tabs {
                    let is_active = state.active_tab == tab;
                    let (fill, stroke_col, text_col) = if is_active {
                        (
                            Color32::from_rgba_unmultiplied(56, 189, 248, 30),
                            Color32::from_rgb(56, 189, 248),
                            Color32::from_rgb(56, 189, 248),
                        )
                    } else {
                        (
                            Color32::from_rgb(18, 24, 36),
                            Color32::from_rgb(36, 50, 74),
                            Color32::from_rgb(148, 163, 184),
                        )
                    };

                    let btn = ui.add(
                        egui::Button::new(RichText::new(tab.name()).font(FontId::proportional(11.0)).strong().color(text_col))
                            .fill(fill)
                            .rounding(Rounding::same(4.0))
                            .stroke(Stroke::new(if is_active { 1.5_f32 } else { 1.0_f32 }, stroke_col))
                            .min_size(Vec2::new(72.0, 26.0))
                    );
                    if btn.clicked() {
                        state.active_tab = tab;
                    }
                }
            });
        },
    );
}

#[cfg(feature = "gui")]
pub fn draw_top_bar_macro_dial(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    accent: Color32,
    tooltip: &str,
) {
    let size = Vec2::new(34.0, 40.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.hovered() {
        resp = resp.on_hover_text(format!("{}\nMacro: {:.0}%", tooltip, *value * 100.0));
    }

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.015).clamp(0.0, 1.0);
    }

    let center = egui::pos2(rect.center().x, rect.top() + 14.0);
    let radius = 11.0;

    // Background circle
    painter.circle_filled(center, radius, Color32::from_rgb(12, 16, 26));
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));

    // Active arc ring (-135 deg to +135 deg)
    let start_angle = -std::f32::consts::PI * 0.75;
    let end_angle = start_angle + (*value * std::f32::consts::PI * 1.5);

    let arc_steps = 12;
    let mut prev_arc = None;
    for i in 0..=arc_steps {
        let a = start_angle + (i as f32 / arc_steps as f32) * (end_angle - start_angle);
        let pt = egui::pos2(center.x + a.cos() * (radius - 1.5), center.y + a.sin() * (radius - 1.5));
        if let Some(last) = prev_arc {
            painter.line_segment([last, pt], Stroke::new(2.0_f32, accent));
        }
        prev_arc = Some(pt);
    }

    // Pointer notch
    let notch_pt = egui::pos2(center.x + end_angle.cos() * (radius - 2.5), center.y + end_angle.sin() * (radius - 2.5));
    painter.line_segment([center, notch_pt], Stroke::new(1.5_f32, Color32::WHITE));

    // Label below
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 30.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(8.5),
        Color32::from_rgb(148, 163, 184),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_top_bar_defaults() {
        let state = ModernTopBarState::default();
        assert_eq!(state.bpm, 120.00);
        assert_eq!(state.key_signature, "Am");
        assert_eq!(state.selected_preset, "Init Synth 1");
        assert!(!state.available_presets.is_empty());
        assert_eq!(state.master_gain, 1.0);
        assert_eq!(state.macro_tone, 0.65);
        assert_eq!(state.macro_space, 0.40);
        assert_eq!(state.macro_punch, 0.55);
        assert_eq!(state.macro_character, 0.50);
        assert!(state.is_novice_macro_visible);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_top_bar_headless_rendering() {
        let mut state = ModernTopBarState::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_top_bar(ui, &mut state, || {});
            });
        });
    }
}
