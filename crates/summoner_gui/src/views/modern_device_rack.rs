// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Reusable Device Box Primitive with Rotary Knobs, Faders, Live CRT Oscilloscope & Filter HUD.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

pub const MIN_KNOB_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target

/// A visual representation of a single DSP device slot in a track's processing chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RackChainDeviceVisual {
    pub kind: String,
    pub display_name: String,
    pub is_bypassed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernDeviceRackState {
    pub device_name: String,
    #[serde(default)]
    pub selected_node_kind: Option<String>,
    #[serde(default)]
    pub node_param_values: std::collections::HashMap<String, f32>,
    pub is_enabled: bool,
    pub is_minimized: bool,
    // Knobs
    pub cutoff: f32,
    pub resonance: f32,
    pub decay: f32,
    pub env_decay: f32,
    pub mod_amt: f32,
    pub drive: f32,
    // Faders
    pub osc_mix: f32,
    pub shape: f32,
    pub volume: f32,
    // Filter Curve Nodes (4 node frequencies 0.0..1.0 and gains)
    pub filter_nodes: [(f32, f32); 4],
    // LFO params
    pub lfo_speed: f32,
    pub lfo_depth: f32,
    #[serde(default)]
    pub is_expanded_params: bool,
    #[serde(default)]
    pub requested_automation_param: Option<String>,
    #[serde(default)]
    pub chain_devices: Vec<RackChainDeviceVisual>,
    #[serde(default)]
    pub selected_chain_idx: usize,
    #[serde(default)]
    pub requested_open_crystal_hud: bool,
    #[serde(default)]
    pub requested_open_armonica_hud: bool,
    #[serde(default)]
    pub requested_open_hurdy_gurdy_hud: bool,
    #[serde(default)]
    pub requested_open_trompette_hud: bool,
    #[serde(default)]
    pub requested_open_hoa5_radar_hud: bool,
    #[serde(default)]
    pub requested_open_neural_morph_hud: bool,
    #[serde(default)]
    pub requested_open_stems_export_modal: bool,
    #[serde(default)]
    pub requested_open_bowed_string_hud: bool,
    #[serde(default)]
    pub requested_open_shakuhachi_hud: bool,
    #[serde(default)]
    pub requested_open_sitar_hud: bool,
    #[serde(default)]
    pub requested_open_turkish_ney_hud: bool,
    #[serde(default)]
    pub requested_open_steelpan_hud: bool,
    #[serde(default)]
    pub requested_open_mbira_hud: bool,
    #[serde(default)]
    pub requested_open_koto_hud: bool,
    #[serde(default)]
    pub requested_open_gamelan_hud: bool,
    #[serde(default)]
    pub requested_open_dulcimer_hud: bool,
    #[serde(default)]
    pub requested_open_clavinet_hud: bool,
    #[serde(default)]
    pub requested_open_grand_piano_hud: bool,
    #[serde(default)]
    pub requested_open_pipe_organ_hud: bool,
    #[serde(default)]
    pub requested_open_electric_piano_hud: bool,
    #[serde(default)]
    pub requested_open_tonewheel_organ_hud: bool,
    #[serde(default)]
    pub requested_open_waveguide_brass_hud: bool,
    #[serde(default)]
    pub requested_open_vocal_tract_hud: bool,
    #[serde(default)]
    pub requested_open_rotary_speaker_hud: bool,
    #[serde(default)]
    pub requested_open_free_reed_hud: bool,
    #[serde(default)]
    pub requested_open_granular_cloud_hud: bool,
    #[serde(default)]
    pub requested_open_spring_lattice_hud: bool,
    #[serde(default)]
    pub requested_open_waveguide_mesh_hud: bool,
    #[serde(default)]
    pub requested_open_plucked_string_hud: bool,
    #[serde(default)]
    pub requested_open_bellows_hud: bool,
    #[serde(default)]
    pub requested_open_jawari_bridge_hud: bool,
    #[serde(default)]
    pub requested_open_soundboard_hud: bool,
    #[serde(default)]
    pub requested_open_sympathetic_hud: bool,
    #[serde(default)]
    pub requested_open_woodwind_jet_hud: bool,
    #[serde(default)]
    pub requested_open_tonehole_matrix_hud: bool,
    #[serde(default)]
    pub requested_open_plate_dispersion_hud: bool,
    #[serde(default)]
    pub requested_open_membrane_cavity_hud: bool,
    #[serde(default)]
    pub requested_open_embouchure_angle_hud: bool,
    #[serde(default)]
    pub requested_open_friction_orbit_hud: bool,
    #[serde(default)]
    pub requested_open_rank_voicing_hud: bool,
    #[serde(default)]
    pub requested_open_tine_resonator_hud: bool,
    #[serde(default)]
    pub requested_open_spring_reverb_hud: bool,
    #[serde(default)]
    pub requested_open_comb_resonator_hud: bool,
    #[serde(default)]
    pub requested_open_wavefront_reflection_hud: bool,
    #[serde(default)]
    pub requested_open_diffractive_propagation_hud: bool,
    #[serde(default)]
    pub requested_open_membrane_resonator_hud: bool,
    #[serde(default)]
    pub requested_open_membrane_plate_hud: bool,
    #[serde(default)]
    pub requested_open_idiophone_spectrum_hud: bool,
    #[serde(default)]
    pub requested_open_sonar_hydrophone_hud: bool,
    #[serde(default)]
    pub requested_open_hoa5_binaural_hud: bool,
    #[serde(default)]
    pub requested_open_mpegh_3d_hud: bool,
    #[serde(default)]
    pub requested_open_atmos_916_hud: bool,
    #[serde(default)]
    pub requested_open_auro3d_hud: bool,
    #[serde(default)]
    pub requested_open_nhk222_hud: bool,
    #[serde(default)]
    pub requested_open_wfs_array_hud: bool,
    #[serde(default)]
    pub requested_open_binaural_panner_hud: bool,
    #[serde(default)]
    pub requested_open_binaural_brir_hud: bool,
    #[serde(default)]
    pub requested_open_continuous_doppler_hud: bool,
    #[serde(default)]
    pub requested_open_spatial_room_hud: bool,
    #[serde(default)]
    pub requested_open_rotary_doppler_hud: bool,
    #[serde(default)]
    pub requested_open_raytraced_reverb_hud: bool,
    #[serde(default)]
    pub requested_open_atmos_surround_hud: bool,
    #[serde(default)]
    pub requested_open_atmos_proximity_hud: bool,
    #[serde(default)]
    pub requested_open_mpegh_trajectory_hud: bool,
    #[serde(default)]
    pub requested_open_nhk222_immersion_hud: bool,
    #[serde(default)]
    pub requested_open_hoa4_spatializer_hud: bool,
    #[serde(default)]
    pub requested_open_mpegh_spatializer_hud: bool,
    #[serde(default)]
    pub requested_open_spatial_trajectory_hud: bool,
    #[serde(default)]
    pub requested_open_ambisonic_radar_hud: bool,
    #[serde(default)]
    pub requested_open_hoa_spatializer_hud: bool,
    #[serde(default)]
    pub requested_open_neural_timbre_morph_hud: bool,
    #[serde(default)]
    pub requested_open_neural_radiance_hud: bool,
    #[serde(default)]
    pub requested_open_neural_vocal_stylizer_hud: bool,
    #[serde(default)]
    pub requested_open_neural_speech_to_singing_hud: bool,
    #[serde(default)]
    pub requested_open_neural_choir_formant_hud: bool,
    #[serde(default)]
    pub requested_open_neural_vocoder_morph_hud: bool,
    #[serde(default)]
    pub requested_open_neural_wavetable_hud: bool,
    #[serde(default)]
    pub requested_open_neural_phoneme_hud: bool,
    #[serde(default)]
    pub requested_open_neural_dereverb_hud: bool,
    #[serde(default)]
    pub requested_open_neural_super_resolution_hud: bool,
    #[serde(default)]
    pub requested_open_nam_amp_hud: bool,
    #[serde(default)]
    pub requested_open_neural_inpaint_hud: bool,
    #[serde(default)]
    pub requested_open_neural_timbre_hud: bool,
    #[serde(default)]
    pub requested_open_bitcrusher_morph_hud: bool,
    #[serde(default)]
    pub requested_open_bbd_chorus_hud: bool,
    #[serde(default)]
    pub requested_open_tube_bias_hud: bool,
    #[serde(default)]
    pub requested_open_tape_saturation_hud: bool,
    #[serde(default)]
    pub requested_open_ladder_filter_hud: bool,
    #[serde(default)]
    pub requested_open_granular_freeze_hud: bool,
    #[serde(default)]
    pub requested_open_auto_wah_hud: bool,
    #[serde(default)]
    pub requested_open_formant_filter_hud: bool,
    #[serde(default)]
    pub requested_open_oversampled_limiter_hud: bool,
    #[serde(default)]
    pub requested_open_vari_mu_master_hud: bool,
}

impl Default for ModernDeviceRackState {
    fn default() -> Self {
        Self {
            device_name: "Synth 1".to_string(),
            selected_node_kind: Some("AetherSynth".to_string()),
            node_param_values: std::collections::HashMap::new(),
            is_enabled: true,
            is_minimized: false,
            is_expanded_params: false,
            requested_automation_param: None,
            chain_devices: Vec::new(),
            selected_chain_idx: 0,
            requested_open_crystal_hud: false,
            requested_open_armonica_hud: false,
            requested_open_hurdy_gurdy_hud: false,
            requested_open_trompette_hud: false,
            requested_open_hoa5_radar_hud: false,
            requested_open_neural_morph_hud: false,
            requested_open_stems_export_modal: false,
            requested_open_bowed_string_hud: false,
            requested_open_shakuhachi_hud: false,
            requested_open_sitar_hud: false,
            requested_open_turkish_ney_hud: false,
            requested_open_steelpan_hud: false,
            requested_open_mbira_hud: false,
            requested_open_koto_hud: false,
            requested_open_gamelan_hud: false,
            requested_open_dulcimer_hud: false,
            requested_open_clavinet_hud: false,
            requested_open_grand_piano_hud: false,
            requested_open_pipe_organ_hud: false,
            requested_open_electric_piano_hud: false,
            requested_open_tonewheel_organ_hud: false,
            requested_open_waveguide_brass_hud: false,
            requested_open_vocal_tract_hud: false,
            requested_open_rotary_speaker_hud: false,
            requested_open_free_reed_hud: false,
            requested_open_granular_cloud_hud: false,
            requested_open_spring_lattice_hud: false,
            requested_open_waveguide_mesh_hud: false,
            requested_open_plucked_string_hud: false,
            requested_open_bellows_hud: false,
            requested_open_jawari_bridge_hud: false,
            requested_open_soundboard_hud: false,
            requested_open_sympathetic_hud: false,
            requested_open_woodwind_jet_hud: false,
            requested_open_tonehole_matrix_hud: false,
            requested_open_plate_dispersion_hud: false,
            requested_open_membrane_cavity_hud: false,
            requested_open_embouchure_angle_hud: false,
            requested_open_friction_orbit_hud: false,
            requested_open_rank_voicing_hud: false,
            requested_open_tine_resonator_hud: false,
            requested_open_spring_reverb_hud: false,
            requested_open_comb_resonator_hud: false,
            requested_open_wavefront_reflection_hud: false,
            requested_open_diffractive_propagation_hud: false,
            requested_open_membrane_resonator_hud: false,
            requested_open_membrane_plate_hud: false,
            requested_open_idiophone_spectrum_hud: false,
            requested_open_sonar_hydrophone_hud: false,
            requested_open_hoa5_binaural_hud: false,
            requested_open_mpegh_3d_hud: false,
            requested_open_atmos_916_hud: false,
            requested_open_auro3d_hud: false,
            requested_open_nhk222_hud: false,
            requested_open_wfs_array_hud: false,
            requested_open_binaural_panner_hud: false,
            requested_open_binaural_brir_hud: false,
            requested_open_continuous_doppler_hud: false,
            requested_open_spatial_room_hud: false,
            requested_open_rotary_doppler_hud: false,
            requested_open_raytraced_reverb_hud: false,
            requested_open_atmos_surround_hud: false,
            requested_open_atmos_proximity_hud: false,
            requested_open_mpegh_trajectory_hud: false,
            requested_open_nhk222_immersion_hud: false,
            requested_open_hoa4_spatializer_hud: false,
            requested_open_mpegh_spatializer_hud: false,
            requested_open_spatial_trajectory_hud: false,
            requested_open_ambisonic_radar_hud: false,
            requested_open_hoa_spatializer_hud: false,
            requested_open_neural_timbre_morph_hud: false,
            requested_open_neural_radiance_hud: false,
            requested_open_neural_vocal_stylizer_hud: false,
            requested_open_neural_speech_to_singing_hud: false,
            requested_open_neural_choir_formant_hud: false,
            requested_open_neural_vocoder_morph_hud: false,
            requested_open_neural_wavetable_hud: false,
            requested_open_neural_phoneme_hud: false,
            requested_open_neural_dereverb_hud: false,
            requested_open_neural_super_resolution_hud: false,
            requested_open_nam_amp_hud: false,
            requested_open_neural_inpaint_hud: false,
            requested_open_neural_timbre_hud: false,
            requested_open_bitcrusher_morph_hud: false,
            requested_open_bbd_chorus_hud: false,
            requested_open_tube_bias_hud: false,
            requested_open_tape_saturation_hud: false,
            requested_open_ladder_filter_hud: false,
            requested_open_granular_freeze_hud: false,
            requested_open_auto_wah_hud: false,
            requested_open_formant_filter_hud: false,
            requested_open_oversampled_limiter_hud: false,
            requested_open_vari_mu_master_hud: false,
            cutoff: 0.65,
            resonance: 0.45,
            decay: 0.50,
            env_decay: 0.35,
            mod_amt: 0.60,
            drive: 0.40,
            osc_mix: 0.75,
            shape: 0.50,
            volume: 0.85,
            filter_nodes: [
                (0.15, 0.70),
                (0.40, 0.65),
                (0.65, 0.55),
                (0.90, 0.20),
            ],
            lfo_speed: 0.40,
            lfo_depth: 0.60,
        }
    }
}

impl ModernDeviceRackState {
    /// Ensure the chain has at least one device matching the currently selected node kind.
    pub fn ensure_chain(&mut self) {
        if self.chain_devices.is_empty() {
            let kind = self.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
            let name = self.device_name.clone();
            self.chain_devices.push(RackChainDeviceVisual {
                kind,
                display_name: name,
                is_bypassed: !self.is_enabled,
            });
            self.selected_chain_idx = 0;
        }
    }

    /// Select a device slot in the track chain.
    pub fn select_device(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if idx < self.chain_devices.len() {
            self.selected_chain_idx = idx;
            let dev = &self.chain_devices[idx];
            self.selected_node_kind = Some(dev.kind.clone());
            self.device_name = dev.display_name.clone();
            self.is_enabled = !dev.is_bypassed;
            true
        } else {
            false
        }
    }

    /// Add a new DSP device module to the end of the track chain and select it.
    pub fn add_device(&mut self, kind: &str, display_name: &str) -> usize {
        self.ensure_chain();
        self.chain_devices.push(RackChainDeviceVisual {
            kind: kind.to_string(),
            display_name: display_name.to_string(),
            is_bypassed: false,
        });
        let new_idx = self.chain_devices.len() - 1;
        self.select_device(new_idx);
        new_idx
    }

    /// Remove a DSP device from the chain (keeping at least 1 device).
    pub fn remove_device(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if self.chain_devices.len() > 1 && idx < self.chain_devices.len() {
            self.chain_devices.remove(idx);
            if self.selected_chain_idx >= self.chain_devices.len() {
                self.selected_chain_idx = self.chain_devices.len() - 1;
            }
            let sel = self.selected_chain_idx;
            self.select_device(sel);
            true
        } else {
            false
        }
    }

    /// Move a device within the chain from `from` to `to`.
    pub fn move_device(&mut self, from: usize, to: usize) -> bool {
        self.ensure_chain();
        if from < self.chain_devices.len() && to < self.chain_devices.len() && from != to {
            let dev = self.chain_devices.remove(from);
            self.chain_devices.insert(to, dev);
            self.selected_chain_idx = to;
            true
        } else {
            false
        }
    }

    /// Toggle bypass state for a device in the chain.
    pub fn toggle_device_bypass(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if let Some(dev) = self.chain_devices.get_mut(idx) {
            dev.is_bypassed = !dev.is_bypassed;
            if idx == self.selected_chain_idx {
                self.is_enabled = !dev.is_bypassed;
            }
            true
        } else {
            false
        }
    }

    /// Request live parameter automation for a given parameter ID (e.g. from right-click or auto button).
    pub fn request_automation(&mut self, param_id: &str) {
        if self.selected_chain_idx > 0 && !param_id.starts_with("node_") && !param_id.starts_with("master_node_") {
            self.requested_automation_param = Some(format!("node_{}_{}", self.selected_chain_idx, param_id));
        } else {
            self.requested_automation_param = Some(param_id.to_string());
        }
    }

    /// Reset standard device rack knobs to factory default values.
    pub fn reset_knob_defaults(&mut self) {
        self.cutoff = 0.65;
        self.resonance = 0.45;
        self.decay = 0.50;
        self.env_decay = 0.35;
        self.mod_amt = 0.60;
        self.drive = 0.40;
        self.osc_mix = 0.75;
        self.shape = 0.50;
        self.volume = 0.85;
        self.lfo_speed = 0.40;
        self.lfo_depth = 0.60;
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_device_rack(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    oscilloscope_data: Option<&[f32]>,
) {
    show_modern_device_rack_with_context(ui, state, oscilloscope_data, None, 1);
}

#[cfg(feature = "gui")]
pub fn show_modern_device_rack_with_context(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    oscilloscope_data: Option<&[f32]>,
    param_bus: Option<&summoner_core::param_bus::ParamBus>,
    track_id: u64,
) {
    state.ensure_chain();
    state.node_param_values.entry("cutoff".to_string()).or_insert(state.cutoff);
    state.node_param_values.entry("resonance".to_string()).or_insert(state.resonance);
    state.node_param_values.entry("decay".to_string()).or_insert(state.decay);
    state.node_param_values.entry("drive".to_string()).or_insert(state.drive);

    let registry = crate::dsp_node_ui::DspNodeRegistry::new();
    let cur_selection = state.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
    let opt_desc = registry.get(&cur_selection);
    let (r, g, b) = opt_desc.map(|d| d.category.color_rgb()).unwrap_or((56, 189, 248));
    let cat_accent = Color32::from_rgb(r, g, b);

    // Collapsed Drawer Mode (Click-to-Expand Pro Rack Drawer)
    if state.is_minimized {
        let collapsed_h = 36.0;
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 20, 32))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.set_height(collapsed_h);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("▲ EXPAND RACK").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).clicked() {
                        state.is_minimized = false;
                    }
                    if ui.button(RichText::new("⊞ PRO").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        state.is_minimized = false;
                        state.is_expanded_params = true;
                    }
                    if ui.button(RichText::new("📈 AUTO").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8))).on_hover_text("Open Live Bézier Automation Lane for primary parameter").clicked() {
                        let primary = opt_desc.and_then(|d| d.params.first()).map(|p| p.id.clone()).unwrap_or_else(|| "cutoff".to_string());
                        state.requested_automation_param = Some(primary);
                    }
                    let is_crystal = cur_selection == "CrystalResonator" || cur_selection.contains("Crystal") || cur_selection.contains("Bowl");
                    let is_armonica = cur_selection == "GlassArmonica" || cur_selection == "FranklinGlassArmonica" || cur_selection == "ArmonicaChassisResonator" || cur_selection.contains("Armonica");
                    let is_hurdy = cur_selection == "FrenchHurdyGurdy" || cur_selection.contains("HurdyGurdy") || cur_selection.contains("Vielle");
                    let is_trompette = cur_selection == "TrompetteChienBridge" || cur_selection == "TrompetteBridge" || cur_selection.contains("Trompette") || cur_selection.contains("Chien");
                    let is_hoa = cur_selection == "AmbisonicRadarSpatializer" || cur_selection == "HoaSpatializer" || cur_selection == "Hoa5BinauralSpatializer" || cur_selection == "Hoa5RadarView" || cur_selection.contains("Ambisonic") || cur_selection.contains("Hoa");
                    let is_bowed = cur_selection.contains("Bowed") || cur_selection.contains("Violin") || cur_selection.contains("Cello") || cur_selection == "BowedStringNode" || cur_selection == "BowedString";
                    let is_shakuhachi = cur_selection.contains("Shakuhachi") || cur_selection.contains("BambooFlute") || cur_selection == "ShakuhachiNode" || cur_selection == "Shakuhachi";
                    let is_sitar = cur_selection.contains("Sitar") || cur_selection == "SitarNode" || cur_selection == "SitarModel" || cur_selection.contains("Jawari");
                    let is_turkish_ney = cur_selection.contains("TurkishNey") || cur_selection.contains("Ney") || cur_selection == "TurkishNeyNode";
                    let is_steelpan = cur_selection.contains("Steelpan") || cur_selection.contains("SteelDrum") || cur_selection == "SteelpanNode" || cur_selection == "SteelpanModel";
                    let is_mbira = cur_selection.contains("Mbira") || cur_selection.contains("Kalimba") || cur_selection == "MbiraNode" || cur_selection == "KalimbaNode" || cur_selection == "MbiraKalimbaModel" || cur_selection == "MbiraModel";
                    let is_koto = cur_selection.contains("Koto") || cur_selection.contains("Guzheng") || cur_selection == "KotoNode" || cur_selection == "KotoSynthesizer" || cur_selection == "KotoStringVoice" || cur_selection == "KotoSoundboard";
                    let is_gamelan = cur_selection.contains("Gamelan") || cur_selection.contains("Gender") || cur_selection.contains("Jegogan") || cur_selection.contains("Metallophone") || cur_selection == "GamelanGender" || cur_selection == "AcousticBalineseGamelanJegoganBronzeBar";
                    let is_dulcimer = cur_selection.contains("Dulcimer") || cur_selection.contains("Cimbalom") || cur_selection.contains("Santur") || cur_selection.contains("Yangqin") || cur_selection.contains("Psaltery") || cur_selection == "DulcimerCimbalomModel";
                    let is_clavinet = cur_selection.contains("Clavinet") || cur_selection.contains("Clav") || cur_selection == "ClavinetNode" || cur_selection == "ClavinetD6";
                    let is_electric_piano = cur_selection.contains("ElectricPiano") || cur_selection == "ElectricPianoModel" || cur_selection == "ElectricPianoProfile" || cur_selection.contains("Rhodes") || cur_selection.contains("Wurlitzer");
                    let is_tonewheel_organ = cur_selection.contains("Tonewheel") || cur_selection == "TonewheelOrgan" || cur_selection == "TonewheelOrganModel" || cur_selection == "TonewheelOrganNode" || cur_selection == "TonewheelOrganProfile" || cur_selection.contains("Hammond") || cur_selection.contains("Drawbar");
                    let is_piano = !is_electric_piano && (cur_selection.contains("Piano") || cur_selection.contains("GrandPiano") || cur_selection == "GrandPianoModel" || cur_selection == "ConcertGrandPianoNode");
                    let is_pipe_organ = !is_tonewheel_organ && (cur_selection.contains("PipeOrgan") || cur_selection.contains("Organ") || cur_selection.contains("Windchest") || cur_selection == "PipeOrganModel");
                    let is_waveguide_brass = cur_selection.contains("WaveguideBrass") || cur_selection.contains("BrassInstrument") || cur_selection.contains("AcousticBrass") || cur_selection == "WaveguideBrassModel" || cur_selection == "WaveguideBrassNode" || cur_selection == "BrassInstrumentProfile";
                    let is_vocal_tract = cur_selection.contains("VocalTract") || cur_selection.contains("VocalTractModel") || cur_selection.contains("VowelSpace") || cur_selection.contains("VocalFormant") || cur_selection == "VocalTractNode" || cur_selection == "VocalTractProfile";
                    let is_rotary_speaker = cur_selection.contains("Rotary") || cur_selection.contains("Leslie") || cur_selection.contains("DopplerSpeaker") || cur_selection == "RotarySpeakerNode" || cur_selection == "DualRotorLeslieDopplerSpeaker" || cur_selection == "RotaryCabinetModel";
                    let is_free_reed = cur_selection.contains("FreeReed") || cur_selection.contains("Harmonium") || cur_selection.contains("Accordion") || cur_selection.contains("Bandoneon") || cur_selection.contains("Bayan") || cur_selection.contains("Cassotto") || cur_selection == "FreeReedNode" || cur_selection == "FreeReedVoice" || cur_selection == "FreeReedOscillator";
                    let is_granular = cur_selection.contains("Granular") || cur_selection.contains("Grain") || cur_selection == "GranularSynthNode" || cur_selection == "GranularFreezeNode" || cur_selection == "GranularPitchShifter";
                    let is_spring_lattice = cur_selection.contains("Spring") || cur_selection.contains("Lattice") || cur_selection == "SpringLatticeNode" || cur_selection == "SpringReverb" || cur_selection == "SpringTankModel" || cur_selection == "SpringMassLattice";
                    let is_waveguide_mesh = cur_selection.contains("WaveguideMesh") || cur_selection.contains("MeshPlate") || cur_selection.contains("MeshMembrane") || cur_selection == "WaveguideMeshNode" || cur_selection == "WaveguideMesh2D" || cur_selection == "RectilinearWaveguideMesh" || cur_selection == "WaveguideMeshBoundary" || cur_selection == "WaveguideMesh";
                    let is_plucked_string = cur_selection.contains("PluckedString") || cur_selection.contains("CommutedPlucked") || cur_selection == "PluckedStringNode" || cur_selection == "CommutedPluckedStringNode" || cur_selection == "CommutedPluckedString" || cur_selection == "PluckedStringWaveguide" || cur_selection == "KarplusStrong";
                    let is_bellows = cur_selection.contains("Bellows") || cur_selection.contains("Cassotto") || cur_selection == "PneumaticBellows" || cur_selection == "CassottoChamber" || cur_selection == "AccordionBellows" || cur_selection == "BellowsDynamics";
                    let is_jawari_bridge = cur_selection.contains("Jawari") || cur_selection.contains("Tarab") || cur_selection == "JawariBridge" || cur_selection == "TarabResonatorBank" || cur_selection == "TarabStringResonator" || cur_selection == "SitarGourdBody" || cur_selection == "SitarBridge";
                    let is_soundboard = cur_selection.contains("Soundboard") || cur_selection.contains("SoundboardBridge") || cur_selection == "SoundboardBridgeModel" || cur_selection == "PianoSoundboard" || cur_selection == "SpruceSoundboard";
                    let is_sympathetic = cur_selection.contains("Sympathetic") || cur_selection.contains("SympatheticCoupling") || cur_selection == "SympatheticCouplingModel" || cur_selection == "SympatheticStringMatrix" || cur_selection == "DroneStrings";
                    let is_woodwind_jet = cur_selection.contains("WoodwindJet") || cur_selection.contains("FluteJet") || cur_selection.contains("AirJet") || cur_selection.contains("WoodwindModel") || cur_selection == "WoodwindJetView" || cur_selection == "AeroacousticFluteJetExciter" || cur_selection == "AcousticConcertFluteAirJetModel" || cur_selection == "WoodwindJetNode";
                    let is_tonehole_matrix = cur_selection.contains("Tonehole") || cur_selection.contains("BoreModel") || cur_selection == "ToneholeMatrixView" || cur_selection == "ToneholeMatrix" || cur_selection == "ToneholeGrid" || cur_selection == "WoodwindToneholeGrid" || cur_selection == "ToneholeLattice";
                    let is_plate_dispersion = cur_selection.contains("PlateTank") || cur_selection.contains("PlateReverb") || cur_selection.contains("PlateDispersion") || cur_selection == "PlateTankNode" || cur_selection == "PlateDispersionView" || cur_selection == "PlateReverbNode" || cur_selection.contains("EMT140");
                    let is_membrane_resonator = cur_selection.contains("MembraneResonator") || cur_selection == "MembraneResonatorView" || cur_selection == "MembraneResonatorNode";
                    let is_membrane_plate = cur_selection.contains("MembranePlate") || cur_selection == "MembranePlateView" || cur_selection == "MembranePlateModel";
                    let is_membrane_cavity = (cur_selection.contains("PercussionMembrane") || cur_selection.contains("MembranePercussion") || cur_selection.contains("MembraneCavity") || cur_selection == "PercussionMembrane" || cur_selection == "MembraneCavityView" || cur_selection == "MembranePercussionModel") && !is_membrane_resonator && !is_membrane_plate;
                    let is_embouchure_angle = cur_selection.contains("EmbouchureAngle") || cur_selection.contains("ShakuhachiEmbouchure") || cur_selection.contains("FluteEmbouchure") || cur_selection == "EmbouchureAngleView" || cur_selection == "EmbouchureAngle" || cur_selection.contains("MeriKari") || cur_selection.contains("Utaguchi");
                    let is_friction_orbit = cur_selection.contains("FrictionOrbit") || cur_selection.contains("StickSlip") || cur_selection.contains("BowedFriction") || cur_selection == "FrictionOrbitView" || cur_selection == "FrictionOrbit" || cur_selection.contains("RosinHysteresis") || cur_selection.contains("HelmholtzOrbit");
                    let is_rank_voicing = cur_selection.contains("RankVoicing") || cur_selection.contains("PipeOrganFlute") || cur_selection.contains("OrganRank") || cur_selection == "RankVoicingView" || cur_selection == "RankVoicing" || cur_selection.contains("FlueCutup") || cur_selection.contains("PneumaticPipeOrganFluteRank");
                    let is_tine_resonator = cur_selection.contains("TineResonator") || cur_selection.contains("TinePickup") || cur_selection.contains("TineResonatorPickup") || cur_selection == "TineResonatorView" || cur_selection == "TineResonator" || cur_selection.contains("EpArticulation") || cur_selection.contains("ElectricPiano");
                    let is_spring_reverb = cur_selection.contains("SpringReverb") || cur_selection.contains("SpringTank") || cur_selection.contains("AnalogSpring") || cur_selection.contains("SpringMass") || cur_selection == "SpringReverbView" || cur_selection == "AnalogSpringReverbTank" || cur_selection == "NonlinearSpringReverbTank";
                    let is_comb_resonator = cur_selection.contains("CombResonator") || cur_selection.contains("CombFilter") || cur_selection.contains("ResonantComb") || cur_selection == "CombResonatorView" || cur_selection == "CombResonatorNode" || cur_selection == "ResonantPhasedCombArray";
                    let is_wavefront_reflection = cur_selection.contains("WavefrontReflection") || cur_selection.contains("Wavefront") || cur_selection.contains("RaytracedEarlyReflection") || cur_selection == "WavefrontReflectionView" || cur_selection == "WavefrontReflectionNode";
                    let is_diffractive_propagation = cur_selection.contains("DiffractivePropagation") || cur_selection.contains("Diffraction") || cur_selection == "DiffractivePropagationView" || cur_selection == "DiffractivePropagationNode";
                    let is_idiophone = cur_selection.contains("Idiophone") || cur_selection.contains("Marimba") || cur_selection.contains("Xylophone") || cur_selection.contains("Vibraphone") || cur_selection == "IdiophoneSpectrum" || cur_selection == "StruckIdiophoneResonatorNode" || cur_selection == "IdiophoneResonator" || cur_selection == "ModalIdiophoneResonator" || cur_selection == "ModalIdiophoneChimeBank";
                    let is_sonar_hydrophone = cur_selection.contains("Sonar") || cur_selection.contains("Hydrophone") || cur_selection == "SonarHydrophoneNode" || cur_selection == "SonarHydrophoneView";
                    let is_hoa5_binaural = cur_selection.contains("Hoa5") || cur_selection.contains("Ambisonic5") || cur_selection.contains("HOA5") || cur_selection == "Hoa5BinauralView" || cur_selection == "Hoa5BinauralNode" || cur_selection == "Ambisonics5thOrderDecoder" || cur_selection.contains("HigherOrderAmbisonics") || cur_selection.contains("Hoa5thOrder");
                    let is_mpegh_3d = cur_selection.contains("Mpegh") || cur_selection.contains("MPEG-H") || cur_selection.contains("MpegH") || cur_selection == "Mpegh3DSpatializerView" || cur_selection == "Mpegh3DSpatializerNode" || cur_selection == "Mpegh3DObjectSpatializer" || cur_selection.contains("MpeghSpatializer");
                    let is_atmos_916 = cur_selection.contains("Atmos916") || cur_selection.contains("DolbyAtmos") || cur_selection == "Atmos916Spatializer" || cur_selection == "Atmos916SpatializerView" || cur_selection.contains("AtmosBed") || cur_selection.contains("AtmosSpatializer");
                    let is_auro3d = cur_selection.contains("Auro3d") || cur_selection.contains("Auro-3D") || cur_selection.contains("Auro131") || cur_selection == "Auro3dSpatializer" || cur_selection == "Auro3dSpatializerView" || cur_selection.contains("AuroSpatializer");
                    let is_nhk222 = cur_selection.contains("Nhk222") || cur_selection.contains("NHK222") || cur_selection == "Nhk222Spatializer" || cur_selection == "Nhk222SpatializerView" || cur_selection.contains("NhkSpatializer");
                    let is_wfs_array = cur_selection.contains("WfsArray") || cur_selection.contains("WfsAcoustic") || cur_selection.contains("WfsPlanar") || cur_selection == "WfsArraySpatializerNode" || cur_selection == "WfsArraySpatializerView" || cur_selection.contains("WaveFieldSynthesis");
                    let is_binaural_panner = cur_selection.contains("BinauralSpatialPanner") || cur_selection.contains("BinauralPanner") || cur_selection == "BinauralPannerView" || cur_selection == "BinauralSpatialPannerNode" || cur_selection.contains("BinauralHrtf") || cur_selection.contains("HrtfPanner");
                    let is_binaural_brir = cur_selection.contains("BinauralBrir") || cur_selection.contains("PartitionedBinauralHrtfConvolver") || cur_selection == "BinauralBrirView" || cur_selection.contains("BrirSpatializer") || cur_selection.contains("RoomImpulseResponse") || cur_selection.contains("BinauralRoom");
                    if is_crystal {
                        if ui.button(RichText::new("🔮 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Physical Modeling Crystal Resonator HUD").clicked() {
                            state.requested_open_crystal_hud = true;
                        }
                    } else if is_armonica {
                        if ui.button(RichText::new("🍷 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Physical Modeling Glass Armonica HUD").clicked() {
                            state.requested_open_armonica_hud = true;
                        }
                    } else if is_hurdy {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Physical Modeling Hurdy-Gurdy HUD").clicked() {
                            state.requested_open_hurdy_gurdy_hud = true;
                        }
                    } else if is_trompette {
                        if ui.button(RichText::new("🐕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(239, 68, 68))).on_hover_text("Open Trompette Chien Buzzing Bridge HUD").clicked() {
                            state.requested_open_trompette_hud = true;
                        }
                    } else if is_hoa {
                        if ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open 5th-Order Ambisonics (HOA5) 3D Radar HUD").clicked() {
                            state.requested_open_hoa5_radar_hud = true;
                        }
                    } else if is_bowed {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Bowed String Acoustic Friction HUD").clicked() {
                            state.requested_open_bowed_string_hud = true;
                        }
                    } else if is_shakuhachi {
                        if ui.button(RichText::new("🎍 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Shakuhachi Bamboo Flute HUD").clicked() {
                            state.requested_open_shakuhachi_hud = true;
                        }
                    } else if is_sitar {
                        if ui.button(RichText::new("🪕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Sitar Curved Jawari Bridge HUD").clicked() {
                            state.requested_open_sitar_hud = true;
                        }
                    } else if is_turkish_ney {
                        if ui.button(RichText::new("🪈 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(45, 212, 191))).on_hover_text("Open Turkish Ney Flute Embouchure HUD").clicked() {
                            state.requested_open_turkish_ney_hud = true;
                        }
                    } else if is_steelpan {
                        if ui.button(RichText::new("🛢 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Caribbean Steelpan Annular Resonance HUD").clicked() {
                            state.requested_open_steelpan_hud = true;
                        }
                    } else if is_mbira {
                        if ui.button(RichText::new("🎵 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Lamellophone Mbira & Kalimba Tine HUD").clicked() {
                            state.requested_open_mbira_hud = true;
                        }
                    } else if is_koto {
                        if ui.button(RichText::new("箏 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Japanese 13-String Koto Zither HUD").clicked() {
                            state.requested_open_koto_hud = true;
                        }
                    } else if is_gamelan {
                        if ui.button(RichText::new("🔔 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Balinese Gamelan Gender Metallophone HUD").clicked() {
                            state.requested_open_gamelan_hud = true;
                        }
                    } else if is_dulcimer {
                        if ui.button(RichText::new("🎼 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Hammered Dulcimer & Cimbalom HUD").clicked() {
                            state.requested_open_dulcimer_hud = true;
                        }
                    } else if is_clavinet {
                        if ui.button(RichText::new("⚡ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(249, 115, 22))).on_hover_text("Open Electromechanical Clavinet D6 HUD").clicked() {
                            state.requested_open_clavinet_hud = true;
                        }
                    } else if is_electric_piano {
                        if ui.button(RichText::new("⚡ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(251, 191, 36))).on_hover_text("Open Electromechanical Tine & Reed Electric Piano HUD").clicked() {
                            state.requested_open_electric_piano_hud = true;
                        }
                    } else if is_tonewheel_organ {
                        if ui.button(RichText::new("🎹 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 9-Drawbar Tonewheel Organ HUD").clicked() {
                            state.requested_open_tonewheel_organ_hud = true;
                        }
                    } else if is_piano {
                        if ui.button(RichText::new("🎹 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(229, 169, 60))).on_hover_text("Open Concert Grand Piano HUD").clicked() {
                            state.requested_open_grand_piano_hud = true;
                        }
                    } else if is_pipe_organ {
                        if ui.button(RichText::new("⛪ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 50))).on_hover_text("Open Pipe Organ Windchest HUD").clicked() {
                            state.requested_open_pipe_organ_hud = true;
                        }
                    } else if is_waveguide_brass {
                        if ui.button(RichText::new("🎺 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Waveguide Brass Acoustic Lip-Reed HUD").clicked() {
                            state.requested_open_waveguide_brass_hud = true;
                        }
                    } else if is_vocal_tract {
                        if ui.button(RichText::new("🗣 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Vocal Tract 44-Cylinder Area Function HUD").clicked() {
                            state.requested_open_vocal_tract_hud = true;
                        }
                    } else if is_rotary_speaker {
                        if ui.button(RichText::new("🌪 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Vintage Rotary Speaker Cabinet & Doppler HUD").clicked() {
                            state.requested_open_rotary_speaker_hud = true;
                        }
                    } else if is_free_reed {
                        if ui.button(RichText::new("🪗 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Free-Reed Aeroelastic Phase Portrait HUD").clicked() {
                            state.requested_open_free_reed_hud = true;
                        }
                    } else if is_granular {
                        if ui.button(RichText::new("☁ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Granular Cloud Synthesis HUD").clicked() {
                            state.requested_open_granular_cloud_hud = true;
                        }
                    } else if is_spring_lattice {
                        if ui.button(RichText::new("🌀 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Spring-Mass Lattice Deformation HUD").clicked() {
                            state.requested_open_spring_lattice_hud = true;
                        }
                    } else if is_waveguide_mesh {
                        if ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 2D Physical Waveguide Resonator Mesh HUD").clicked() {
                            state.requested_open_waveguide_mesh_hud = true;
                        }
                    } else if is_plucked_string {
                        if ui.button(RichText::new("🎸 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Plucked & Struck Waveguide String HUD").clicked() {
                            state.requested_open_plucked_string_hud = true;
                        }
                    } else if is_bellows {
                        if ui.button(RichText::new("🪗 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(217, 119, 6))).on_hover_text("Open Pneumatic Bellows Dynamics HUD").clicked() {
                            state.requested_open_bellows_hud = true;
                        }
                    } else if is_jawari_bridge {
                        if ui.button(RichText::new("🪕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Sitar Curved Jawari Bridge HUD").clicked() {
                            state.requested_open_jawari_bridge_hud = true;
                        }
                    } else if is_soundboard {
                        if ui.button(RichText::new("🪵 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Spruce Soundboard & Bridge HUD").clicked() {
                            state.requested_open_soundboard_hud = true;
                        }
                    } else if is_sympathetic {
                        if ui.button(RichText::new("✨ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Sympathetic Resonance Coupling HUD").clicked() {
                            state.requested_open_sympathetic_hud = true;
                        }
                    } else if is_woodwind_jet {
                        if ui.button(RichText::new("🌬 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Woodwind Air-Jet Embouchure HUD").clicked() {
                            state.requested_open_woodwind_jet_hud = true;
                        }
                    } else if is_tonehole_matrix {
                        if ui.button(RichText::new("🕳 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Woodwind 6-Tonehole Radiation HUD").clicked() {
                            state.requested_open_tonehole_matrix_hud = true;
                        }
                    } else if is_plate_dispersion {
                        if ui.button(RichText::new("🛸 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Plate Dispersion & APDN Reverb HUD").clicked() {
                            state.requested_open_plate_dispersion_hud = true;
                        }
                    } else if is_membrane_cavity {
                        if ui.button(RichText::new("🥁 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Membrane Cavity & Drum Displacement HUD").clicked() {
                            state.requested_open_membrane_cavity_hud = true;
                        }
                    } else if is_embouchure_angle {
                        if ui.button(RichText::new("🎋 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Shakuhachi Embouchure & Meri/Kari HUD").clicked() {
                            state.requested_open_embouchure_angle_hud = true;
                        }
                    } else if is_friction_orbit {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Bowed String Stick-Slip Friction & Orbit HUD").clicked() {
                            state.requested_open_friction_orbit_hud = true;
                        }
                    } else if is_rank_voicing {
                        if ui.button(RichText::new("⛪ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 50))).on_hover_text("Open Pipe Organ Rank Voicing HUD").clicked() {
                            state.requested_open_rank_voicing_hud = true;
                        }
                    } else if is_tine_resonator
                        && ui.button(RichText::new("🎹 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 183, 3))).on_hover_text("Open Electric Piano Tine Resonator HUD").clicked()
                    {
                        state.requested_open_tine_resonator_hud = true;
                    } else if is_spring_reverb
                        && ui.button(RichText::new("🌀 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Spring Reverb Tank Simulator & Dispersion HUD").clicked()
                    {
                        state.requested_open_spring_reverb_hud = true;
                    } else if is_comb_resonator
                        && ui.button(RichText::new("🪮 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Spectral Comb Resonator & Matrix HUD").clicked()
                    {
                        state.requested_open_comb_resonator_hud = true;
                    } else if is_wavefront_reflection
                        && ui.button(RichText::new("🌊 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Wavefront Reflection Raytracing HUD").clicked()
                    {
                        state.requested_open_wavefront_reflection_hud = true;
                    } else if is_diffractive_propagation
                        && ui.button(RichText::new("📐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 0))).on_hover_text("Open Diffractive Wave Propagation HUD").clicked()
                    {
                        state.requested_open_diffractive_propagation_hud = true;
                    } else if is_membrane_resonator
                        && ui.button(RichText::new("🥁 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Elastic Drumhead Membrane Resonator HUD").clicked()
                    {
                        state.requested_open_membrane_resonator_hud = true;
                    } else if is_membrane_plate
                        && ui.button(RichText::new("🍽 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Coupled 2D Membrane & Plate Resonator HUD").clicked()
                    {
                        state.requested_open_membrane_plate_hud = true;
                    } else if is_idiophone
                        && ui.button(RichText::new("🎵 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 190, 40))).on_hover_text("Open Struck Idiophone Modal Resonator HUD").clicked()
                    {
                        state.requested_open_idiophone_spectrum_hud = true;
                    } else if is_sonar_hydrophone
                        && ui.button(RichText::new("🌊 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 210, 255))).on_hover_text("Open Underwater Sonar & Hydrophone Cavitation HUD").clicked()
                    {
                        state.requested_open_sonar_hydrophone_hud = true;
                    } else if is_hoa5_binaural
                        && ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(157, 78, 221))).on_hover_text("Open 5th-Order Ambisonics HOA5 Binaural HUD").clicked()
                    {
                        state.requested_open_hoa5_binaural_hud = true;
                    } else if is_mpegh_3d
                        && ui.button(RichText::new("🎧 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open MPEG-H 3D Immersive Spatializer HUD").clicked()
                    {
                        state.requested_open_mpegh_3d_hud = true;
                    } else if is_atmos_916
                        && ui.button(RichText::new("🎬 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Dolby Atmos 9.1.6 3D Room Raytracer HUD").clicked()
                    {
                        state.requested_open_atmos_916_hud = true;
                    } else if is_auro3d
                        && ui.button(RichText::new("🏔 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 171, 0))).on_hover_text("Open Auro-3D 13.1 Tri-Level Spatializer HUD").clicked()
                    {
                        state.requested_open_auro3d_hud = true;
                    } else if is_nhk222
                        && ui.button(RichText::new("🛰 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open NHK 22.2 Super Hi-Vision Hemispherical Spatializer HUD").clicked()
                    {
                        state.requested_open_nhk222_hud = true;
                    } else if is_wfs_array
                        && ui.button(RichText::new("🌊 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Wave Field Synthesis (WFS) Holographic Linear Array HUD").clicked()
                    {
                        state.requested_open_wfs_array_hud = true;
                    } else if is_binaural_panner
                        && ui.button(RichText::new("🎧 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Spatial Binaural HRTF 3D Orbit Spatializer HUD").clicked()
                    {
                        state.requested_open_binaural_panner_hud = true;
                    } else if is_binaural_brir
                        && ui.button(RichText::new("🏛 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Multi-Source Binaural BRIR Room Impulse Response HUD").clicked()
                    {
                        state.requested_open_binaural_brir_hud = true;
                    }
                    ui.add_space(4.0);
                    let pwr_col = if state.is_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
                    if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).clicked() {
                        state.toggle_device_bypass(state.selected_chain_idx);
                    }
                    ui.add_space(4.0);

                    // Compact Device Chain Pills in Collapsed Header
                    let mut pill_to_select = None;
                    for (d_i, dev) in state.chain_devices.iter().enumerate() {
                        let is_d_sel = d_i == state.selected_chain_idx;
                        let card_col = if is_d_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) };
                        let bg_col = if is_d_sel { Color32::from_rgb(24, 38, 58) } else { Color32::from_rgb(18, 24, 36) };
                        egui::Frame::none()
                            .fill(bg_col)
                            .stroke(Stroke::new(1.0_f32, if is_d_sel { card_col } else { Color32::from_rgb(36, 50, 74) }))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                            .show(ui, |ui| {
                                if ui.selectable_label(is_d_sel, RichText::new(format!("{}. {}", d_i + 1, dev.display_name)).font(FontId::proportional(9.0)).color(card_col)).clicked() {
                                    pill_to_select = Some(d_i);
                                }
                            });
                        ui.add_space(2.0);
                    }
                    if let Some(idx) = pill_to_select {
                        state.select_device(idx);
                    }

                    ui.add_space(4.0);

                    let cur_display = opt_desc.map(|d| format!("{} {}", d.category.icon(), d.display_name)).unwrap_or_else(|| state.device_name.clone());
                    egui::ComboBox::from_id_source("minimized_device_rack_module_selector")
                        .selected_text(RichText::new(cur_display).font(FontId::proportional(11.0)).color(cat_accent))
                        .show_ui(ui, |ui| {
                            for desc in registry.list_all() {
                                let is_sel = state.selected_node_kind.as_deref() == Some(&desc.kind_id);
                                let label = format!("{} {} ({})", desc.category.icon(), desc.display_name, desc.category.name());
                                if ui.selectable_label(is_sel, label).clicked() {
                                    state.selected_node_kind = Some(desc.kind_id.clone());
                                    state.device_name = desc.display_name.clone();
                                    if let Some(dev) = state.chain_devices.get_mut(state.selected_chain_idx) {
                                        dev.kind = desc.kind_id.clone();
                                        dev.display_name = desc.display_name.clone();
                                    }
                                    state.node_param_values.clear();
                                    for (p_i, schema) in desc.params.iter().enumerate() {
                                        let def = match &schema.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                            _ => 0.5,
                                        };
                                        state.node_param_values.insert(schema.id.clone(), def);
                                        if let Some(bus) = param_bus {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                            if bus.get(pid).is_some() {
                                                bus.set(pid, def);
                                            }
                                        }
                                    }
                                }
                            }
                        });

                    if let Some(desc) = opt_desc {
                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(r, g, b, 25))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(r, g, b, 80)))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(desc.category.name()).font(FontId::proportional(9.0)).color(cat_accent));
                            });
                    }

                    ui.add_space(8.0);
                    let pills = [
                        ("Tone", state.cutoff),
                        ("Space", state.decay),
                        ("Punch", state.drive),
                        ("Vol", state.volume),
                    ];
                    for (name, val) in pills {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(20, 28, 44))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("{}: {:.0}%", name, val * 100.0)).font(FontId::proportional(9.0)).color(Color32::from_rgb(200, 215, 235)));
                            });
                        ui.add_space(2.0);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Modular Rack Drawer (Collapsed)").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                    });
                });
            });
        return;
    }

    let rack_height = if state.is_expanded_params { 260.0 } else { 210.0 };
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 20, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_height(rack_height);

            // 1. Device Box Header (Rule 2 from GUI_RULES.md)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Device Rack").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
                ui.add_space(8.0);
                let pwr_col = if state.is_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
                if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).on_hover_text("Bypass / Enable Active Device").clicked() {
                    state.toggle_device_bypass(state.selected_chain_idx);
                }

                // DSP Module Selection Dropdown
                let cur_display = opt_desc.map(|d| format!("{} {}", d.category.icon(), d.display_name)).unwrap_or_else(|| state.device_name.clone());

                egui::ComboBox::from_id_source("device_rack_module_selector")
                    .selected_text(RichText::new(cur_display).font(FontId::proportional(11.0)).color(cat_accent))
                    .show_ui(ui, |ui| {
                        for desc in registry.list_all() {
                            let is_sel = state.selected_node_kind.as_deref() == Some(&desc.kind_id);
                            let label = format!("{} {} ({})", desc.category.icon(), desc.display_name, desc.category.name());
                            if ui.selectable_label(is_sel, label).clicked() {
                                state.selected_node_kind = Some(desc.kind_id.clone());
                                state.device_name = desc.display_name.clone();
                                if let Some(dev) = state.chain_devices.get_mut(state.selected_chain_idx) {
                                    dev.kind = desc.kind_id.clone();
                                    dev.display_name = desc.display_name.clone();
                                }
                                state.node_param_values.clear();
                                for (p_i, schema) in desc.params.iter().enumerate() {
                                    let def = match &schema.widget {
                                        crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                        | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                        _ => 0.5,
                                    };
                                    state.node_param_values.insert(schema.id.clone(), def);
                                    if let Some(bus) = param_bus {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                        if bus.get(pid).is_some() {
                                            bus.set(pid, def);
                                        }
                                    }
                                }
                            }
                        }
                    });

                if let Some(desc) = opt_desc {
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(r, g, b, 25))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(r, g, b, 80)))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(desc.category.name()).font(FontId::proportional(9.0)).color(cat_accent));
                        });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("▼ COLLAPSE").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184))).clicked() {
                        state.is_minimized = true;
                    }
                    let pro_text = if state.is_expanded_params { "⊟ OVERVIEW" } else { "⊞ PRO PARAMS" };
                    let pro_col = if state.is_expanded_params { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(200, 215, 235) };
                    if ui.button(RichText::new(pro_text).font(FontId::proportional(10.0)).strong().color(pro_col)).clicked() {
                        state.is_expanded_params = !state.is_expanded_params;
                    }
                    if ui.button(RichText::new("📈 Auto").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8))).on_hover_text("Open Live Bézier Automation Lane for primary parameter").clicked() {
                        let primary = opt_desc.and_then(|d| d.params.first()).map(|p| p.id.clone()).unwrap_or_else(|| "cutoff".to_string());
                        state.requested_automation_param = Some(primary);
                    }
                    let is_crystal = cur_selection == "CrystalResonator" || cur_selection.contains("Crystal") || cur_selection.contains("Bowl");
                    let is_armonica = cur_selection == "GlassArmonica" || cur_selection == "FranklinGlassArmonica" || cur_selection == "ArmonicaChassisResonator" || cur_selection.contains("Armonica");
                    let is_hurdy = cur_selection == "FrenchHurdyGurdy" || cur_selection.contains("HurdyGurdy") || cur_selection.contains("Vielle");
                    let is_trompette = cur_selection == "TrompetteChienBridge" || cur_selection == "TrompetteBridge" || cur_selection.contains("Trompette") || cur_selection.contains("Chien");
                    let is_hoa = cur_selection == "AmbisonicRadarSpatializer" || cur_selection == "HoaSpatializer" || cur_selection == "Hoa5BinauralSpatializer" || cur_selection == "Hoa5RadarView" || cur_selection.contains("Ambisonic") || cur_selection.contains("Hoa");
                    let is_neural = cur_selection.contains("Neural") || cur_selection.contains("Timbre") || cur_selection.contains("Latent") || cur_selection.contains("Resynthesizer") || cur_selection == "NeuralMorphOrbView" || cur_selection == "NeuralTimbreMorph" || cur_selection == "NeuralWavetable";
                    let is_export = cur_selection.contains("Export") || cur_selection.contains("Stem") || cur_selection.contains("Batch") || cur_selection == "ExportPreset" || cur_selection == "StemExportFormat";
                    let is_bowed = cur_selection.contains("Bowed") || cur_selection.contains("Violin") || cur_selection.contains("Cello") || cur_selection == "BowedStringNode" || cur_selection == "BowedString";
                    let is_shakuhachi = cur_selection.contains("Shakuhachi") || cur_selection.contains("BambooFlute") || cur_selection == "ShakuhachiNode" || cur_selection == "Shakuhachi";
                    let is_sitar = cur_selection.contains("Sitar") || cur_selection == "SitarNode" || cur_selection == "SitarModel" || cur_selection.contains("Jawari");
                    let is_turkish_ney = cur_selection.contains("TurkishNey") || cur_selection.contains("Ney") || cur_selection == "TurkishNeyNode";
                    let is_steelpan = cur_selection.contains("Steelpan") || cur_selection.contains("SteelDrum") || cur_selection == "SteelpanNode" || cur_selection == "SteelpanModel";
                    let is_mbira = cur_selection.contains("Mbira") || cur_selection.contains("Kalimba") || cur_selection == "MbiraNode" || cur_selection == "KalimbaNode" || cur_selection == "MbiraKalimbaModel" || cur_selection == "MbiraModel";
                    let is_koto = cur_selection.contains("Koto") || cur_selection.contains("Guzheng") || cur_selection == "KotoNode" || cur_selection == "KotoSynthesizer" || cur_selection == "KotoStringVoice" || cur_selection == "KotoSoundboard";
                    let is_gamelan = cur_selection.contains("Gamelan") || cur_selection.contains("Gender") || cur_selection.contains("Jegogan") || cur_selection.contains("Metallophone") || cur_selection == "GamelanGender" || cur_selection == "AcousticBalineseGamelanJegoganBronzeBar";
                    let is_dulcimer = cur_selection.contains("Dulcimer") || cur_selection.contains("Cimbalom") || cur_selection.contains("Santur") || cur_selection.contains("Yangqin") || cur_selection.contains("Psaltery") || cur_selection == "DulcimerCimbalomModel";
                    let is_clavinet = cur_selection.contains("Clavinet") || cur_selection.contains("Clav") || cur_selection == "ClavinetNode" || cur_selection == "ClavinetD6";
                    let is_electric_piano = cur_selection.contains("ElectricPiano") || cur_selection == "ElectricPianoModel" || cur_selection == "ElectricPianoProfile" || cur_selection.contains("Rhodes") || cur_selection.contains("Wurlitzer");
                    let is_tonewheel_organ = cur_selection.contains("Tonewheel") || cur_selection == "TonewheelOrgan" || cur_selection == "TonewheelOrganModel" || cur_selection == "TonewheelOrganNode" || cur_selection == "TonewheelOrganProfile" || cur_selection.contains("Hammond") || cur_selection.contains("Drawbar");
                    let is_piano = !is_electric_piano && (cur_selection.contains("Piano") || cur_selection.contains("GrandPiano") || cur_selection == "GrandPianoModel" || cur_selection == "ConcertGrandPianoNode");
                    let is_pipe_organ = !is_tonewheel_organ && (cur_selection.contains("PipeOrgan") || cur_selection.contains("Organ") || cur_selection.contains("Windchest") || cur_selection == "PipeOrganModel");
                    let is_waveguide_brass = cur_selection.contains("WaveguideBrass") || cur_selection.contains("BrassInstrument") || cur_selection.contains("AcousticBrass") || cur_selection == "WaveguideBrassModel" || cur_selection == "WaveguideBrassNode" || cur_selection == "BrassInstrumentProfile";
                    let is_vocal_tract = cur_selection.contains("VocalTract") || cur_selection.contains("VocalTractModel") || cur_selection.contains("VowelSpace") || cur_selection.contains("VocalFormant") || cur_selection == "VocalTractNode" || cur_selection == "VocalTractProfile";
                    let is_rotary_speaker = cur_selection.contains("Rotary") || cur_selection.contains("Leslie") || cur_selection.contains("DopplerSpeaker") || cur_selection == "RotarySpeakerNode" || cur_selection == "DualRotorLeslieDopplerSpeaker" || cur_selection == "RotaryCabinetModel";
                    let is_free_reed = cur_selection.contains("FreeReed") || cur_selection.contains("Harmonium") || cur_selection.contains("Accordion") || cur_selection.contains("Bandoneon") || cur_selection.contains("Bayan") || cur_selection.contains("Cassotto") || cur_selection == "FreeReedNode" || cur_selection == "FreeReedVoice" || cur_selection == "FreeReedOscillator";
                    let is_granular = cur_selection.contains("Granular") || cur_selection.contains("Grain") || cur_selection == "GranularSynthNode" || cur_selection == "GranularFreezeNode" || cur_selection == "GranularPitchShifter" || cur_selection == "GranularFreezeView" || cur_selection.contains("Freeze") || cur_selection.contains("CloudFreeze");
                    let is_spring_lattice = cur_selection.contains("Spring") || cur_selection.contains("Lattice") || cur_selection == "SpringLatticeNode" || cur_selection == "SpringReverb" || cur_selection == "SpringTankModel" || cur_selection == "SpringMassLattice";
                    let is_waveguide_mesh = cur_selection.contains("WaveguideMesh") || cur_selection.contains("MeshPlate") || cur_selection.contains("MeshMembrane") || cur_selection == "WaveguideMeshNode" || cur_selection == "WaveguideMesh2D" || cur_selection == "RectilinearWaveguideMesh" || cur_selection == "WaveguideMeshBoundary" || cur_selection == "WaveguideMesh";
                    let is_plucked_string = cur_selection.contains("PluckedString") || cur_selection.contains("CommutedPlucked") || cur_selection == "PluckedStringNode" || cur_selection == "CommutedPluckedStringNode" || cur_selection == "CommutedPluckedString" || cur_selection == "PluckedStringWaveguide" || cur_selection == "KarplusStrong";
                    let is_bellows = cur_selection.contains("Bellows") || cur_selection.contains("Cassotto") || cur_selection == "PneumaticBellows" || cur_selection == "CassottoChamber" || cur_selection == "AccordionBellows" || cur_selection == "BellowsDynamics";
                    let is_jawari_bridge = cur_selection.contains("Jawari") || cur_selection.contains("Tarab") || cur_selection == "JawariBridge" || cur_selection == "TarabResonatorBank" || cur_selection == "TarabStringResonator" || cur_selection == "SitarGourdBody" || cur_selection == "SitarBridge";
                    let is_soundboard = cur_selection.contains("Soundboard") || cur_selection.contains("SoundboardBridge") || cur_selection == "SoundboardBridgeModel" || cur_selection == "PianoSoundboard" || cur_selection == "SpruceSoundboard";
                    let is_sympathetic = cur_selection.contains("Sympathetic") || cur_selection.contains("SympatheticCoupling") || cur_selection == "SympatheticCouplingModel" || cur_selection == "SympatheticStringMatrix" || cur_selection == "DroneStrings";
                    let is_woodwind_jet = cur_selection.contains("WoodwindJet") || cur_selection.contains("FluteJet") || cur_selection.contains("AirJet") || cur_selection.contains("WoodwindModel") || cur_selection == "WoodwindJetView" || cur_selection == "AeroacousticFluteJetExciter" || cur_selection == "AcousticConcertFluteAirJetModel" || cur_selection == "WoodwindJetNode";
                    let is_tonehole_matrix = cur_selection.contains("Tonehole") || cur_selection.contains("BoreModel") || cur_selection == "ToneholeMatrixView" || cur_selection == "ToneholeMatrix" || cur_selection == "ToneholeGrid" || cur_selection == "WoodwindToneholeGrid" || cur_selection == "ToneholeLattice";
                    let is_plate_dispersion = cur_selection.contains("PlateTank") || cur_selection.contains("PlateReverb") || cur_selection.contains("PlateDispersion") || cur_selection == "PlateTankNode" || cur_selection == "PlateDispersionView" || cur_selection == "PlateReverbNode" || cur_selection.contains("EMT140");
                    let is_membrane_resonator = cur_selection.contains("MembraneResonator") || cur_selection == "MembraneResonatorView" || cur_selection == "MembraneResonatorNode";
                    let is_membrane_plate = cur_selection.contains("MembranePlate") || cur_selection == "MembranePlateView" || cur_selection == "MembranePlateModel";
                    let is_membrane_cavity = (cur_selection.contains("PercussionMembrane") || cur_selection.contains("MembranePercussion") || cur_selection.contains("MembraneCavity") || cur_selection == "PercussionMembrane" || cur_selection == "MembraneCavityView" || cur_selection == "MembranePercussionModel") && !is_membrane_resonator && !is_membrane_plate;
                    let is_embouchure_angle = cur_selection.contains("EmbouchureAngle") || cur_selection.contains("ShakuhachiEmbouchure") || cur_selection.contains("FluteEmbouchure") || cur_selection == "EmbouchureAngleView" || cur_selection == "EmbouchureAngle" || cur_selection.contains("MeriKari") || cur_selection.contains("Utaguchi");
                    let is_friction_orbit = cur_selection.contains("FrictionOrbit") || cur_selection.contains("StickSlip") || cur_selection.contains("BowedFriction") || cur_selection == "FrictionOrbitView" || cur_selection == "FrictionOrbit" || cur_selection.contains("RosinHysteresis") || cur_selection.contains("HelmholtzOrbit");
                    let is_rank_voicing = cur_selection.contains("RankVoicing") || cur_selection.contains("PipeOrganFlute") || cur_selection.contains("OrganRank") || cur_selection == "RankVoicingView" || cur_selection == "RankVoicing" || cur_selection.contains("FlueCutup") || cur_selection.contains("PneumaticPipeOrganFluteRank");
                    let is_tine_resonator = cur_selection.contains("TineResonator") || cur_selection.contains("TinePickup") || cur_selection.contains("TineResonatorPickup") || cur_selection == "TineResonatorView" || cur_selection == "TineResonator" || cur_selection.contains("EpArticulation") || cur_selection.contains("ElectricPiano");
                    let is_spring_reverb = cur_selection.contains("SpringReverb") || cur_selection.contains("SpringTank") || cur_selection.contains("AnalogSpring") || cur_selection.contains("SpringMass") || cur_selection == "SpringReverbView" || cur_selection == "AnalogSpringReverbTank" || cur_selection == "NonlinearSpringReverbTank";
                    let is_comb_resonator = cur_selection.contains("CombResonator") || cur_selection.contains("CombFilter") || cur_selection.contains("ResonantComb") || cur_selection == "CombResonatorView" || cur_selection == "CombResonatorNode" || cur_selection == "ResonantPhasedCombArray";
                    let is_wavefront_reflection = cur_selection.contains("WavefrontReflection") || cur_selection.contains("Wavefront") || cur_selection.contains("RaytracedEarlyReflection") || cur_selection == "WavefrontReflectionView" || cur_selection == "WavefrontReflectionNode";
                    let is_diffractive_propagation = cur_selection.contains("DiffractivePropagation") || cur_selection.contains("Diffraction") || cur_selection == "DiffractivePropagationView" || cur_selection == "DiffractivePropagationNode";
                    let is_idiophone = cur_selection.contains("Idiophone") || cur_selection.contains("Marimba") || cur_selection.contains("Xylophone") || cur_selection.contains("Vibraphone") || cur_selection == "IdiophoneSpectrum" || cur_selection == "StruckIdiophoneResonatorNode" || cur_selection == "IdiophoneResonator" || cur_selection == "ModalIdiophoneResonator" || cur_selection == "ModalIdiophoneChimeBank";
                    let is_sonar_hydrophone = cur_selection.contains("Sonar") || cur_selection.contains("Hydrophone") || cur_selection == "SonarHydrophoneNode" || cur_selection == "SonarHydrophoneView";
                    let is_hoa5_binaural = cur_selection.contains("Hoa5") || cur_selection.contains("Ambisonic5") || cur_selection.contains("HOA5") || cur_selection == "Hoa5BinauralView" || cur_selection == "Hoa5BinauralNode" || cur_selection == "Ambisonics5thOrderDecoder" || cur_selection.contains("HigherOrderAmbisonics") || cur_selection.contains("Hoa5thOrder");
                    let is_mpegh_3d = cur_selection.contains("Mpegh") || cur_selection.contains("MPEG-H") || cur_selection.contains("MpegH") || cur_selection == "Mpegh3DSpatializerView" || cur_selection == "Mpegh3DSpatializerNode" || cur_selection == "Mpegh3DObjectSpatializer" || cur_selection.contains("MpeghSpatializer");
                    let is_atmos_916 = cur_selection.contains("Atmos916") || cur_selection.contains("DolbyAtmos") || cur_selection == "Atmos916Spatializer" || cur_selection == "Atmos916SpatializerView" || cur_selection.contains("AtmosBed") || cur_selection.contains("AtmosSpatializer");
                    let is_auro3d = cur_selection.contains("Auro3d") || cur_selection.contains("Auro-3D") || cur_selection.contains("Auro131") || cur_selection == "Auro3dSpatializer" || cur_selection == "Auro3dSpatializerView" || cur_selection.contains("AuroSpatializer");
                    let is_nhk222 = cur_selection.contains("Nhk222") || cur_selection.contains("NHK222") || cur_selection == "Nhk222Spatializer" || cur_selection == "Nhk222SpatializerView" || cur_selection.contains("NhkSpatializer");
                    let is_wfs_array = cur_selection.contains("WfsArray") || cur_selection.contains("WfsAcoustic") || cur_selection.contains("WfsPlanar") || cur_selection == "WfsArraySpatializerNode" || cur_selection == "WfsArraySpatializerView" || cur_selection.contains("WaveFieldSynthesis");
                    let is_binaural_panner = cur_selection.contains("BinauralSpatialPanner") || cur_selection.contains("BinauralPanner") || cur_selection == "BinauralPannerView" || cur_selection == "BinauralSpatialPannerNode" || cur_selection.contains("BinauralHrtf") || cur_selection.contains("HrtfPanner");
                    let is_binaural_brir = cur_selection.contains("BinauralBrir") || cur_selection.contains("PartitionedBinauralHrtfConvolver") || cur_selection == "BinauralBrirView" || cur_selection.contains("BrirSpatializer") || cur_selection.contains("RoomImpulseResponse") || cur_selection.contains("BinauralRoom");
                    let is_continuous_doppler = cur_selection.contains("ContinuousSpatialPanner") || cur_selection.contains("DistanceDoppler") || cur_selection.contains("DopplerSpatializer") || cur_selection == "ContinuousSpatialPannerDoppler3D" || cur_selection == "ContinuousDopplerView" || cur_selection.contains("BinauralDoppler") || cur_selection.contains("AmbisonicBFormatDoppler");
                    let is_spatial_room = cur_selection.contains("SpatialRoom") || cur_selection.contains("ImageSourceRoom") || cur_selection == "SpatialRoomNode" || cur_selection == "SpatialRoomView" || cur_selection.contains("RoomAcousticModel") || cur_selection.contains("ShoeboxRoom");
                    let is_rotary_doppler = cur_selection.contains("RotaryDualHorn") || cur_selection.contains("RotaryDualDrum") || cur_selection.contains("RotarySpeakerAcousticDeflector") || cur_selection.contains("RotaryDoppler") || cur_selection.contains("RotaryCabinet") || cur_selection.contains("DualRotorLeslie") || cur_selection == "RotaryDopplerView";
                    let is_raytraced_reverb = cur_selection.contains("VolumetricRaytracedRoomAcoustics") || cur_selection.contains("BinauralRaytracedEarlyReflection") || cur_selection.contains("RaytracedReverb") || cur_selection == "RaytracedReverbView" || cur_selection.contains("RaytracedEarlyReflection") || cur_selection.contains("RaytracedRoom");
                    let is_atmos_surround = cur_selection.contains("AtmosSurround") || cur_selection.contains("SurroundStemSplitterBedObject") || cur_selection.contains("SurroundDownmixer") || cur_selection == "AtmosSurroundNode" || cur_selection == "AtmosSurroundView";
                    let is_atmos_proximity = cur_selection.contains("AtmosProximity") || cur_selection == "AtmosProximityNode" || cur_selection == "AtmosProximityView";
                    let is_mpegh_trajectory = cur_selection.contains("MpeghTrajectory") || cur_selection.contains("MpegHTrajectory") || cur_selection == "MpeghTrajectorySpatializer" || cur_selection == "MpeghTrajectoryView" || cur_selection == "MpeghTrajectoryNode";
                    let is_nhk222_immersion = cur_selection.contains("Nhk222Immersion") || cur_selection.contains("NHK222Immersion") || cur_selection == "Nhk222ImmersionView" || cur_selection == "Nhk222ImmersionNode";
                    let is_hoa4_spatializer = cur_selection.contains("Hoa4") || cur_selection.contains("HOA4") || cur_selection == "Hoa4SpatializerView" || cur_selection == "Hoa4Spatializer" || cur_selection.contains("Ambisonics4thOrder") || cur_selection.contains("Hoa4Sphere");
                    let is_mpegh_spatializer = cur_selection.contains("MpeghSpatializer") || cur_selection.contains("MPEGHSpatializer") || cur_selection == "MpeghSpatializerView" || cur_selection == "MpeghSpatializerNode" || cur_selection.contains("MpeghHrtf") || cur_selection == "MpeghSpatialAudio";
                    let is_spatial_trajectory = cur_selection.contains("SpatialTrajectory") || cur_selection == "SpatialTrajectoryView" || cur_selection.contains("TrajectorySpline") || cur_selection == "SpatialTrajectoryNode";
                    let is_ambisonic_radar = cur_selection.contains("AmbisonicRadar") || cur_selection == "AmbisonicRadarView" || cur_selection.contains("AmbisonicPanner") || cur_selection == "AmbisonicRadarNode" || cur_selection.contains("SurroundPanner");
                    let is_hoa_spatializer = cur_selection.contains("HoaSpatializer") || cur_selection == "HoaSpatializerView" || cur_selection == "HoaSpatializerNode" || cur_selection.contains("DolbyAtmos714");
                    let is_neural_timbre_morph = cur_selection.contains("NeuralTimbreMorph") || cur_selection.contains("TimbreMorph") || cur_selection == "NeuralTimbreMorphView" || cur_selection == "NeuralTimbreMorphNode" || cur_selection == "NeuralTimbreMorphSynthesizer" || cur_selection.contains("DiffusionVocoder");
                    let is_neural_radiance = cur_selection.contains("NeuralRadiance") || cur_selection.contains("NeRFAcoustic") || cur_selection == "NeuralRadianceView" || cur_selection == "NeuralRadianceNode" || cur_selection.contains("ContinuousRadiance");
                    let is_neural_vocal_stylizer = cur_selection.contains("NeuralVocalStylizer") || cur_selection.contains("VocalStylizer") || cur_selection == "NeuralVocalStylizerView" || cur_selection == "NeuralVocalStylizerNode" || cur_selection.contains("VocalExpression") || cur_selection.contains("MelismaResynthesizer");
                    let is_neural_speech_to_singing = cur_selection.contains("NeuralSpeechToSinging") || cur_selection.contains("SpeechToSinging") || cur_selection == "NeuralSpeechToSingingView" || cur_selection == "NeuralSpeechToSingingNode" || cur_selection.contains("SingingVoice") || cur_selection.contains("ProsodyIntonation");
                    let is_neural_choir_formant = cur_selection.contains("NeuralChoirFormant") || cur_selection.contains("ChoirFormant") || cur_selection == "NeuralChoirFormantView" || cur_selection == "NeuralChoirFormantNode" || cur_selection.contains("ChoirEnsemble") || cur_selection.contains("VowelTrajectory");
                    let is_neural_vocoder_morph = cur_selection.contains("NeuralVocoderMorph") || cur_selection.contains("VocoderMorph") || cur_selection == "NeuralVocoderMorphView" || cur_selection == "NeuralVocoderMorphNode";
                    let is_neural_wavetable = cur_selection.contains("NeuralWavetable") || cur_selection.contains("DifferentialWavetable") || cur_selection == "NeuralWavetableView" || cur_selection == "NeuralDifferentialWavetableLatentOscillator" || cur_selection.contains("LatentWavetable") || cur_selection == "NeuralDifferentiableLatentWavetableSynthesizer";
                    let is_neural_phoneme = cur_selection.contains("NeuralPhoneme") || cur_selection.contains("PhonemeMorph") || cur_selection == "NeuralPhonemeView" || cur_selection == "NeuralPhonemeNode" || cur_selection.contains("Phoneme");
                    let is_neural_dereverb = cur_selection.contains("NeuralDereverb") || cur_selection.contains("Dereverb") || cur_selection == "NeuralDereverbView" || cur_selection == "NeuralDereverbNode" || cur_selection.contains("Deconvolution");
                    let is_neural_super_res = cur_selection.contains("NeuralSuperResolution") || cur_selection.contains("SuperResolution") || cur_selection == "NeuralSuperResolutionView" || cur_selection == "NeuralSuperResolutionNode" || cur_selection.contains("SuperRes");
                    let is_nam_amp = cur_selection.contains("NamAmp") || cur_selection.contains("NamModel") || cur_selection == "NamAmpView" || cur_selection == "NamAmpNode" || cur_selection.contains("NeuralAmp") || cur_selection.contains("WaveNet");
                    let is_neural_inpaint = cur_selection.contains("NeuralInpaint") || cur_selection.contains("Inpaint") || cur_selection == "NeuralInpaintView" || cur_selection == "NeuralInpaintNode" || cur_selection.contains("SpectralRepair");
                    let is_neural_timbre = cur_selection.contains("NeuralTimbre") || cur_selection.contains("TimbreTransfer") || cur_selection == "NeuralTimbreView" || cur_selection == "NeuralTimbreNode" || cur_selection.contains("LatentFlow");
                    let is_bitcrusher = cur_selection.contains("Bitcrusher") || cur_selection.contains("BitCrush") || cur_selection == "BitcrusherMorphView" || cur_selection == "BitcrusherNode" || cur_selection.contains("SampleRateReduction");
                    let is_bbd_chorus = cur_selection.contains("BbdChorus") || cur_selection.contains("Chorus") || cur_selection == "BbdChorusView" || cur_selection == "EffectChorus" || cur_selection.contains("BucketBrigade") || cur_selection.contains("LissajousChorus");
                    let is_tube = cur_selection.contains("Tube") || cur_selection.contains("Vacuum") || cur_selection.contains("Triode") || cur_selection == "TubeBiasView" || cur_selection == "TubeSaturation" || cur_selection.contains("TubeAmp");
                    let is_tape = cur_selection.contains("Tape") || cur_selection.contains("Hysteresis") || cur_selection == "TapeSaturationView" || cur_selection == "TapeSaturation" || cur_selection.contains("TapeWarmth") || cur_selection.contains("ReelToReel");
                    let is_ladder = cur_selection.contains("Ladder") || cur_selection.contains("Moog") || cur_selection.contains("Tb303") || cur_selection == "LadderFilterView" || cur_selection == "LadderFilter" || cur_selection.contains("FilterLadder");
                    let is_granular_freeze = cur_selection.contains("Freeze") || cur_selection == "GranularFreezeView" || cur_selection == "GranularFreeze" || cur_selection.contains("CloudFreeze") || cur_selection == "GranularFreezeNode";
                    let is_auto_wah = cur_selection.contains("AutoWah") || cur_selection.contains("Wah") || cur_selection == "AutoWahView" || cur_selection == "AutoWahNode" || cur_selection.contains("WahFilter");
                    let is_formant = cur_selection.contains("Formant") || cur_selection.contains("Vowel") || cur_selection == "FormantFilterView" || cur_selection == "FormantFilterNode" || cur_selection.contains("FormantFilter");
                    let is_limiter = cur_selection.contains("Limiter") || cur_selection.contains("TruePeak") || cur_selection == "OversampledLimiterNode" || cur_selection == "TruePeakOversampledLimiter" || cur_selection.contains("PeakLimiter");
                    let is_vari_mu = cur_selection.contains("VariMu") || cur_selection.contains("VariableMu") || cur_selection == "VariMuMasterNode" || cur_selection == "VariMuMasterLimiter" || cur_selection == "MasterStereoVariMuLimiter" || cur_selection.contains("ValveLimiter");
                    if is_crystal {
                        if ui.button(RichText::new("🔮 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Physical Modeling Crystal Resonator HUD").clicked() {
                            state.requested_open_crystal_hud = true;
                        }
                    } else if is_armonica {
                        if ui.button(RichText::new("🍷 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Physical Modeling Glass Armonica HUD").clicked() {
                            state.requested_open_armonica_hud = true;
                        }
                    } else if is_hurdy {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Physical Modeling Hurdy-Gurdy HUD").clicked() {
                            state.requested_open_hurdy_gurdy_hud = true;
                        }
                    } else if is_trompette {
                        if ui.button(RichText::new("🐕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(239, 68, 68))).on_hover_text("Open Trompette Chien Buzzing Bridge HUD").clicked() {
                            state.requested_open_trompette_hud = true;
                        }
                    } else if is_hoa {
                        if ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open 5th-Order Ambisonics (HOA5) 3D Radar HUD").clicked() {
                            state.requested_open_hoa5_radar_hud = true;
                        }
                    } else if is_neural {
                        if ui.button(RichText::new("🧬 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(244, 63, 94))).on_hover_text("Open 2D Neural Timbre Morphing Orb HUD").clicked() {
                            state.requested_open_neural_morph_hud = true;
                        }
                    } else if is_export {
                        if ui.button(RichText::new("📦 Stems").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(14, 165, 233))).on_hover_text("Open Multi-Track Stems Batch Exporter").clicked() {
                            state.requested_open_stems_export_modal = true;
                        }
                    } else if is_bowed {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Bowed String Acoustic Friction HUD").clicked() {
                            state.requested_open_bowed_string_hud = true;
                        }
                    } else if is_shakuhachi {
                        if ui.button(RichText::new("🎍 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Shakuhachi Bamboo Flute HUD").clicked() {
                            state.requested_open_shakuhachi_hud = true;
                        }
                    } else if is_sitar {
                        if ui.button(RichText::new("🪕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Sitar Curved Jawari Bridge HUD").clicked() {
                            state.requested_open_sitar_hud = true;
                        }
                    } else if is_turkish_ney {
                        if ui.button(RichText::new("🪈 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(45, 212, 191))).on_hover_text("Open Turkish Ney Flute Embouchure HUD").clicked() {
                            state.requested_open_turkish_ney_hud = true;
                        }
                    } else if is_steelpan {
                        if ui.button(RichText::new("🛢 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Caribbean Steelpan Annular Resonance HUD").clicked() {
                            state.requested_open_steelpan_hud = true;
                        }
                    } else if is_mbira {
                        if ui.button(RichText::new("🎵 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Lamellophone Mbira & Kalimba Tine HUD").clicked() {
                            state.requested_open_mbira_hud = true;
                        }
                    } else if is_koto {
                        if ui.button(RichText::new("箏 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Japanese 13-String Koto Zither HUD").clicked() {
                            state.requested_open_koto_hud = true;
                        }
                    } else if is_gamelan {
                        if ui.button(RichText::new("🔔 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Balinese Gamelan Gender Metallophone HUD").clicked() {
                            state.requested_open_gamelan_hud = true;
                        }
                    } else if is_dulcimer {
                        if ui.button(RichText::new("🎼 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Hammered Dulcimer & Cimbalom HUD").clicked() {
                            state.requested_open_dulcimer_hud = true;
                        }
                    } else if is_clavinet {
                        if ui.button(RichText::new("⚡ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(249, 115, 22))).on_hover_text("Open Electromechanical Clavinet D6 HUD").clicked() {
                            state.requested_open_clavinet_hud = true;
                        }
                    } else if is_electric_piano {
                        if ui.button(RichText::new("⚡ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(251, 191, 36))).on_hover_text("Open Electromechanical Tine & Reed Electric Piano HUD").clicked() {
                            state.requested_open_electric_piano_hud = true;
                        }
                    } else if is_tonewheel_organ {
                        if ui.button(RichText::new("🎹 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 9-Drawbar Tonewheel Organ HUD").clicked() {
                            state.requested_open_tonewheel_organ_hud = true;
                        }
                    } else if is_piano {
                        if ui.button(RichText::new("🎹 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(229, 169, 60))).on_hover_text("Open Concert Grand Piano HUD").clicked() {
                            state.requested_open_grand_piano_hud = true;
                        }
                    } else if is_pipe_organ {
                        if ui.button(RichText::new("⛪ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 50))).on_hover_text("Open Pipe Organ Windchest HUD").clicked() {
                            state.requested_open_pipe_organ_hud = true;
                        }
                    } else if is_waveguide_brass {
                        if ui.button(RichText::new("🎺 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Waveguide Brass Acoustic Lip-Reed HUD").clicked() {
                            state.requested_open_waveguide_brass_hud = true;
                        }
                    } else if is_vocal_tract {
                        if ui.button(RichText::new("🗣 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(52, 211, 153))).on_hover_text("Open Vocal Tract 44-Cylinder Area Function HUD").clicked() {
                            state.requested_open_vocal_tract_hud = true;
                        }
                    } else if is_rotary_speaker {
                        if ui.button(RichText::new("🌪 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Vintage Rotary Speaker Cabinet & Doppler HUD").clicked() {
                            state.requested_open_rotary_speaker_hud = true;
                        }
                    } else if is_free_reed {
                        if ui.button(RichText::new("🪗 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Free-Reed Aeroelastic Phase Portrait HUD").clicked() {
                            state.requested_open_free_reed_hud = true;
                        }
                    } else if is_granular {
                        if ui.button(RichText::new("☁ HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Granular Cloud Synthesis HUD").clicked() {
                            state.requested_open_granular_cloud_hud = true;
                        }
                    } else if is_spring_lattice {
                        if ui.button(RichText::new("🌀 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Spring-Mass Lattice Deformation HUD").clicked() {
                            state.requested_open_spring_lattice_hud = true;
                        }
                    } else if is_waveguide_mesh {
                        if ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 2D Physical Waveguide Resonator Mesh HUD").clicked() {
                            state.requested_open_waveguide_mesh_hud = true;
                        }
                    } else if is_plucked_string {
                        if ui.button(RichText::new("🎸 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Plucked & Struck Waveguide String HUD").clicked() {
                            state.requested_open_plucked_string_hud = true;
                        }
                    } else if is_bellows {
                        if ui.button(RichText::new("🪗 Bellows HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(217, 119, 6))).on_hover_text("Open Pneumatic Bellows Dynamics HUD").clicked() {
                            state.requested_open_bellows_hud = true;
                        }
                    } else if is_jawari_bridge {
                        if ui.button(RichText::new("🪕 Jawari HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Sitar Curved Jawari Bridge HUD").clicked() {
                            state.requested_open_jawari_bridge_hud = true;
                        }
                    } else if is_soundboard {
                        if ui.button(RichText::new("🪵 Soundboard HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Spruce Soundboard & Bridge HUD").clicked() {
                            state.requested_open_soundboard_hud = true;
                        }
                    } else if is_sympathetic {
                        if ui.button(RichText::new("✨ Sympathetic HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Sympathetic Resonance Coupling HUD").clicked() {
                            state.requested_open_sympathetic_hud = true;
                        }
                    } else if is_woodwind_jet {
                        if ui.button(RichText::new("🌬 Woodwind Jet HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Woodwind Air-Jet Embouchure HUD").clicked() {
                            state.requested_open_woodwind_jet_hud = true;
                        }
                    } else if is_tonehole_matrix {
                        if ui.button(RichText::new("🕳 Tonehole HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Woodwind 6-Tonehole Radiation HUD").clicked() {
                            state.requested_open_tonehole_matrix_hud = true;
                        }
                    } else if is_plate_dispersion {
                        if ui.button(RichText::new("🛸 Plate HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Plate Dispersion & APDN Reverb HUD").clicked() {
                            state.requested_open_plate_dispersion_hud = true;
                        }
                    } else if is_membrane_cavity {
                        if ui.button(RichText::new("🥁 Membrane HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Membrane Cavity & Drum Displacement HUD").clicked() {
                            state.requested_open_membrane_cavity_hud = true;
                        }
                    } else if is_embouchure_angle {
                        if ui.button(RichText::new("🎋 Embouchure HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Shakuhachi Embouchure & Meri/Kari HUD").clicked() {
                            state.requested_open_embouchure_angle_hud = true;
                        }
                    } else if is_friction_orbit {
                        if ui.button(RichText::new("🎻 Friction HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Bowed String Stick-Slip Friction & Orbit HUD").clicked() {
                            state.requested_open_friction_orbit_hud = true;
                        }
                    } else if is_rank_voicing {
                        if ui.button(RichText::new("⛪ Voicing HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 50))).on_hover_text("Open Pipe Organ Rank Voicing HUD").clicked() {
                            state.requested_open_rank_voicing_hud = true;
                        }
                    } else if is_tine_resonator
                        && ui.button(RichText::new("🎹 Tine HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 183, 3))).on_hover_text("Open Electric Piano Tine Resonator HUD").clicked()
                    {
                        state.requested_open_tine_resonator_hud = true;
                    } else if is_spring_reverb
                        && ui.button(RichText::new("🌀 Spring HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Spring Reverb Tank Simulator & Dispersion HUD").clicked()
                    {
                        state.requested_open_spring_reverb_hud = true;
                    } else if is_comb_resonator
                        && ui.button(RichText::new("🪮 Comb HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Spectral Comb Resonator & Matrix HUD").clicked()
                    {
                        state.requested_open_comb_resonator_hud = true;
                    } else if is_wavefront_reflection
                        && ui.button(RichText::new("🌊 Wavefront HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Wavefront Reflection Raytracing HUD").clicked()
                    {
                        state.requested_open_wavefront_reflection_hud = true;
                    } else if is_diffractive_propagation
                        && ui.button(RichText::new("📐 Diffract HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 0))).on_hover_text("Open Diffractive Wave Propagation HUD").clicked()
                    {
                        state.requested_open_diffractive_propagation_hud = true;
                    } else if is_membrane_resonator
                        && ui.button(RichText::new("🥁 Drum HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 0))).on_hover_text("Open Elastic Drumhead Membrane Resonator HUD").clicked()
                    {
                        state.requested_open_membrane_resonator_hud = true;
                    } else if is_membrane_plate
                        && ui.button(RichText::new("🍽 Plate HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Coupled 2D Membrane & Plate Resonator HUD").clicked()
                    {
                        state.requested_open_membrane_plate_hud = true;
                    } else if is_idiophone
                        && ui.button(RichText::new("🎵 Idiophone HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 190, 40))).on_hover_text("Open Struck Idiophone Modal Resonator HUD").clicked()
                    {
                        state.requested_open_idiophone_spectrum_hud = true;
                    } else if is_sonar_hydrophone
                        && ui.button(RichText::new("🌊 Sonar HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 210, 255))).on_hover_text("Open Underwater Sonar & Hydrophone Cavitation HUD").clicked()
                    {
                        state.requested_open_sonar_hydrophone_hud = true;
                    } else if is_hoa5_binaural
                        && ui.button(RichText::new("🌐 HOA5 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(157, 78, 221))).on_hover_text("Open 5th-Order Ambisonics HOA5 Binaural HUD").clicked()
                    {
                        state.requested_open_hoa5_binaural_hud = true;
                    } else if is_mpegh_3d
                        && ui.button(RichText::new("🎧 MPEG-H HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open MPEG-H 3D Immersive Spatializer HUD").clicked()
                    {
                        state.requested_open_mpegh_3d_hud = true;
                    } else if is_atmos_916
                        && ui.button(RichText::new("🎬 Atmos HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Dolby Atmos 9.1.6 3D Room Raytracer HUD").clicked()
                    {
                        state.requested_open_atmos_916_hud = true;
                    } else if is_auro3d
                        && ui.button(RichText::new("🏔 Auro-3D HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 171, 0))).on_hover_text("Open Auro-3D 13.1 Tri-Level Spatializer HUD").clicked()
                    {
                        state.requested_open_auro3d_hud = true;
                    } else if is_nhk222
                        && ui.button(RichText::new("🛰 NHK 22.2 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open NHK 22.2 Super Hi-Vision Hemispherical Spatializer HUD").clicked()
                    {
                        state.requested_open_nhk222_hud = true;
                    } else if is_wfs_array
                        && ui.button(RichText::new("🌊 WFS HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Wave Field Synthesis (WFS) Holographic Linear Array HUD").clicked()
                    {
                        state.requested_open_wfs_array_hud = true;
                    } else if is_binaural_panner
                        && ui.button(RichText::new("🎧 Binaural HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Spatial Binaural HRTF 3D Orbit Spatializer HUD").clicked()
                    {
                        state.requested_open_binaural_panner_hud = true;
                    } else if is_binaural_brir
                        && ui.button(RichText::new("🏛 BRIR HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Multi-Source Binaural BRIR Room Impulse Response HUD").clicked()
                    {
                        state.requested_open_binaural_brir_hud = true;
                    } else if is_continuous_doppler
                        && ui.button(RichText::new("🚀 Doppler HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Continuous 3D Kinematic Spatializer & Doppler Shift Engine HUD").clicked()
                    {
                        state.requested_open_continuous_doppler_hud = true;
                    } else if is_spatial_room
                        && ui.button(RichText::new("🏛️ Room HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open 3D Shoebox Room Acoustics & 6-Wall Early Reflection / Diffuse FDN Reverb HUD").clicked()
                    {
                        state.requested_open_spatial_room_hud = true;
                    } else if is_rotary_doppler
                        && ui.button(RichText::new("🌪️ Rotary HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Stereo Rotary Speaker Doppler & Acoustic Cabinet HUD").clicked()
                    {
                        state.requested_open_rotary_doppler_hud = true;
                    } else if is_raytraced_reverb
                        && ui.button(RichText::new("⚡ Raytrace HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Acoustic Early Reflections Raytracer HUD").clicked()
                    {
                        state.requested_open_raytraced_reverb_hud = true;
                    } else if is_atmos_surround
                        && ui.button(RichText::new("🌌 Atmos Bed HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 7.1.4 Dolby Atmos 3D Immersive Bed & Object Radar HUD").clicked()
                    {
                        state.requested_open_atmos_surround_hud = true;
                    } else if is_atmos_proximity
                        && ui.button(RichText::new("📡 Atmos Prox HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 180, 0))).on_hover_text("Open 7.1.4 Dolby Atmos Spherical Proximity & Distance HUD").clicked()
                    {
                        state.requested_open_atmos_proximity_hud = true;
                    } else if is_mpegh_trajectory
                        && ui.button(RichText::new("💫 Traj HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Broadcast Mastering Immersive MPEG-H 3D Dynamic Object Trajectory HUD").clicked()
                    {
                        state.requested_open_mpegh_trajectory_hud = true;
                    } else if is_nhk222_immersion
                        && ui.button(RichText::new("🌐 22.2 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Broadcast Mastering Immersive 22.2 Multichannel Audio (NHK 22.2) 3-Layer Spherical Immersion HUD").clicked()
                    {
                        state.requested_open_nhk222_immersion_hud = true;
                    } else if is_hoa4_spatializer
                        && ui.button(RichText::new("🌐 HOA4 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Broadcast Mastering Higher-Order Ambisonics 4th-Order 25-ch HUD").clicked()
                    {
                        state.requested_open_hoa4_spatializer_hud = true;
                    } else if is_mpegh_spatializer
                        && ui.button(RichText::new("📡 MPEG-H HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Broadcast Mastering MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD").clicked()
                    {
                        state.requested_open_mpegh_spatializer_hud = true;
                    } else if is_spatial_trajectory
                        && ui.button(RichText::new("🚀 Traj HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open 3D Spatial Trajectory Polar Canvas & Granular Grain Cloud Density HUD").clicked()
                    {
                        state.requested_open_spatial_trajectory_hud = true;
                    } else if is_ambisonic_radar
                        && ui.button(RichText::new("📡 Ambisonic HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open 3D Ambisonic Trajectory Radar & Multi-Channel Surround Sound Panner HUD").clicked()
                    {
                        state.requested_open_ambisonic_radar_hud = true;
                    } else if is_hoa_spatializer
                        && ui.button(RichText::new("🌐 HOA 7.1.4").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Dolby Atmos HOA 7.1.4 3D Spatializer & Binaural Head-Tracking HUD").clicked()
                    {
                        state.requested_open_hoa_spatializer_hud = true;
                    } else if is_neural_timbre_morph
                        && ui.button(RichText::new("🧬 Timbre HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(180, 90, 255))).on_hover_text("Open Neural Timbre Morphing Diffusion Vocoder & Resynthesizer HUD").clicked()
                    {
                        state.requested_open_neural_timbre_morph_hud = true;
                    } else if is_neural_radiance
                        && ui.button(RichText::new("🌐 Radiance HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(157, 78, 221))).on_hover_text("Open Neural Acoustic HRTF/BRIR Continuous Radiance Field HUD").clicked()
                    {
                        state.requested_open_neural_radiance_hud = true;
                    } else if is_neural_vocal_stylizer
                        && ui.button(RichText::new("🎤 Stylizer HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 93, 143))).on_hover_text("Open Neural Polyphonic Vocal Stylizer & Microtonal Ornament Resynthesizer HUD").clicked()
                    {
                        state.requested_open_neural_vocal_stylizer_hud = true;
                    } else if is_neural_speech_to_singing
                        && ui.button(RichText::new("🎙 Speech2Sing HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Neural Speech-to-Singing Synthesis HUD").clicked()
                    {
                        state.requested_open_neural_speech_to_singing_hud = true;
                    } else if is_neural_choir_formant
                        && ui.button(RichText::new("👥 Choir HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Neural Choir Formant Morpher HUD").clicked()
                    {
                        state.requested_open_neural_choir_formant_hud = true;
                    } else if is_neural_vocoder_morph
                        && ui.button(RichText::new("🎙 Vocoder HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Multi-Stage Neural Vocoder Formant Morpher HUD").clicked()
                    {
                        state.requested_open_neural_vocoder_morph_hud = true;
                    } else if is_neural_wavetable
                        && ui.button(RichText::new("🌊 Wavetable HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(180, 90, 255))).on_hover_text("Open Neural Wavetable Morphing Synthesizer HUD").clicked()
                    {
                        state.requested_open_neural_wavetable_hud = true;
                    } else if is_neural_phoneme
                        && ui.button(RichText::new("🗣 Phoneme HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(180, 90, 255))).on_hover_text("Open Neural Audio Latent Style Transfer & Phoneme Morphing Vocoder HUD").clicked()
                    {
                        state.requested_open_neural_phoneme_hud = true;
                    } else if is_neural_dereverb
                        && ui.button(RichText::new("🏛 Dereverb HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 245, 212))).on_hover_text("Open Neural Acoustic Blind Dereverberation & Deconvolution HUD").clicked()
                    {
                        state.requested_open_neural_dereverb_hud = true;
                    } else if is_neural_super_res
                        && ui.button(RichText::new("✨ Super-Res HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Neural Audio Super-Resolution & Bandwidth Extension HUD").clicked()
                    {
                        state.requested_open_neural_super_resolution_hud = true;
                    } else if is_nam_amp
                        && ui.button(RichText::new("🎸 NAM Amp HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier HUD").clicked()
                    {
                        state.requested_open_nam_amp_hud = true;
                    } else if is_neural_inpaint
                        && ui.button(RichText::new("🩹 Inpaint HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Neural Diffusion Audio Inpainter & Generative Spectral Repair HUD").clicked()
                    {
                        state.requested_open_neural_inpaint_hud = true;
                    } else if is_neural_timbre
                        && ui.button(RichText::new("🧬 Timbre HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(244, 63, 94))).on_hover_text("Open Neural Timbre Transfer Morphing Resynthesizer HUD").clicked()
                    {
                        state.requested_open_neural_timbre_hud = true;
                    } else if is_bitcrusher
                        && ui.button(RichText::new("🎛 Bitcrush HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 107, 43))).on_hover_text("Open Tactile Bitcrusher & Quantization Morphology HUD").clicked()
                    {
                        state.requested_open_bitcrusher_morph_hud = true;
                    } else if is_bbd_chorus
                        && ui.button(RichText::new("📻 BBD Chorus HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Multi-Voice Analog Bucket-Brigade (BBD) Chorus Matrix HUD").clicked()
                    {
                        state.requested_open_bbd_chorus_hud = true;
                    } else if is_tube
                        && ui.button(RichText::new("⚡ Tube HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open Physical Vacuum Tube Triode Bias & Harmonic Distortion HUD").clicked()
                    {
                        state.requested_open_tube_bias_hud = true;
                    } else if is_tape
                        && ui.button(RichText::new("📼 Tape HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 179, 0))).on_hover_text("Open Analog Reel-to-Reel Tape Saturation & Magnetic Hysteresis HUD").clicked()
                    {
                        state.requested_open_tape_saturation_hud = true;
                    } else if is_ladder
                        && ui.button(RichText::new("🎛 Ladder HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Analog Ladder 24dB/oct Filter & Self-Oscillation HUD").clicked()
                    {
                        state.requested_open_ladder_filter_hud = true;
                    } else if is_granular_freeze
                        && ui.button(RichText::new("❄ Freeze HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 255, 180))).on_hover_text("Open Granular Spectral Cloud Freeze & Trajectory HUD").clicked()
                    {
                        state.requested_open_granular_freeze_hud = true;
                    } else if is_auto_wah
                        && ui.button(RichText::new("🎛 Auto-Wah HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(6, 214, 160))).on_hover_text("Open Dynamic Resonant Auto-Wah Filter HUD").clicked()
                    {
                        state.requested_open_auto_wah_hud = true;
                    } else if is_formant
                        && ui.button(RichText::new("🗣 Formant HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 215, 0))).on_hover_text("Open 5-Vowel Phonetic Formant Filter Resonator HUD").clicked()
                    {
                        state.requested_open_formant_filter_hud = true;
                    } else if is_limiter
                        && ui.button(RichText::new("🛡️ Limiter HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(0, 229, 255))).on_hover_text("Open Mastering True-Peak 8x Oversampled Limiter HUD").clicked()
                    {
                        state.requested_open_oversampled_limiter_hud = true;
                    } else if is_vari_mu
                        && ui.button(RichText::new("📻 Vari-Mu HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(255, 170, 50))).on_hover_text("Open Mastering Dual-Mono Variable-Mu Tube Optical Compressor HUD").clicked()
                    {
                        state.requested_open_vari_mu_master_hud = true;
                    }
                    let _ = ui.small_button("✕");
                    egui::Frame::none()
                        .fill(Color32::from_rgb(24, 34, 52))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Zero CLI Left Behind").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                        });
                });
            });

            ui.add_space(4.0);

            // 1b. Track DSP Device Chain Bar (Multi-node processing chain)
            ui.horizontal(|ui| {
                ui.label(RichText::new("CHAIN:").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let mut dev_to_select = None;
                let mut dev_to_toggle_bypass = None;
                let mut dev_to_remove = None;
                let mut dev_move_left = None;
                let mut dev_move_right = None;

                for (d_i, dev) in state.chain_devices.iter().enumerate() {
                    let is_sel = d_i == state.selected_chain_idx;
                    let d_desc = registry.get(&dev.kind);
                    let (dr, dg, db) = d_desc.map(|d| d.category.color_rgb()).unwrap_or((56, 189, 248));
                    let d_accent = Color32::from_rgb(dr, dg, db);
                    let d_icon = d_desc.map(|d| d.category.icon()).unwrap_or("🎛️");

                    let border_col = if is_sel { d_accent } else { Color32::from_rgb(36, 50, 74) };
                    let bg_col = if is_sel { Color32::from_rgba_unmultiplied(dr, dg, db, 35) } else { Color32::from_rgb(18, 24, 36) };

                    egui::Frame::none()
                        .fill(bg_col)
                        .stroke(Stroke::new(if is_sel { 1.5_f32 } else { 1.0_f32 }, border_col))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let byp_col = if !dev.is_bypassed { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(100, 116, 139) };
                                if ui.button(RichText::new("⏻").font(FontId::proportional(9.5)).color(byp_col)).on_hover_text(if dev.is_bypassed { "Device Bypassed (Click to Enable)" } else { "Device Active (Click to Bypass)" }).clicked() {
                                    dev_to_toggle_bypass = Some(d_i);
                                }
                                let badge_text = format!("{} {}. {}", d_icon, d_i + 1, dev.display_name);
                                let badge_resp = ui.selectable_label(is_sel, RichText::new(badge_text).font(FontId::proportional(9.5)).strong().color(if is_sel { Color32::WHITE } else { Color32::from_rgb(200, 215, 235) }));
                                if badge_resp.clicked() {
                                    dev_to_select = Some(d_i);
                                }
                            });
                        });
                    ui.add_space(2.0);
                }

                if let Some(idx) = dev_to_select {
                    state.select_device(idx);
                }
                if let Some(idx) = dev_to_toggle_bypass {
                    state.toggle_device_bypass(idx);
                }

                // Move / Remove controls for active device
                if state.chain_devices.len() > 1 {
                    if state.selected_chain_idx > 0
                        && ui.small_button(RichText::new("◀").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184))).on_hover_text("Move device earlier in processing chain").clicked()
                    {
                        dev_move_left = Some(state.selected_chain_idx);
                    }
                    if state.selected_chain_idx + 1 < state.chain_devices.len()
                        && ui.small_button(RichText::new("▶").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184))).on_hover_text("Move device later in processing chain").clicked()
                    {
                        dev_move_right = Some(state.selected_chain_idx);
                    }
                    if ui.small_button(RichText::new("✕").font(FontId::proportional(9.0)).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Remove selected device from chain").clicked() {
                        dev_to_remove = Some(state.selected_chain_idx);
                    }
                }

                if let Some(idx) = dev_move_left {
                    state.move_device(idx, idx - 1);
                }
                if let Some(idx) = dev_move_right {
                    state.move_device(idx, idx + 1);
                }
                if let Some(idx) = dev_to_remove {
                    state.remove_device(idx);
                }

                ui.add_space(4.0);

                // [➕ Add Device] dropdown menu across all DSP categories
                let mut module_to_add: Option<String> = None;
                egui::ComboBox::from_id_source("add_device_to_chain_combo")
                    .selected_text(RichText::new("➕ Add Device ▾").font(FontId::proportional(9.5)).color(Color32::from_rgb(56, 189, 248)))
                    .show_ui(ui, |ui| {
                        for cat in [
                            crate::dsp_node_ui::DspCategory::Oscillator,
                            crate::dsp_node_ui::DspCategory::FilterEq,
                            crate::dsp_node_ui::DspCategory::Modulation,
                            crate::dsp_node_ui::DspCategory::DynamicsMaster,
                            crate::dsp_node_ui::DspCategory::DistortionSaturation,
                            crate::dsp_node_ui::DspCategory::TimeSpace,
                            crate::dsp_node_ui::DspCategory::SpatialSurround,
                            crate::dsp_node_ui::DspCategory::AcousticPhysicalModel,
                            crate::dsp_node_ui::DspCategory::SpectralResynthesis,
                            crate::dsp_node_ui::DspCategory::NeuralAi,
                            crate::dsp_node_ui::DspCategory::SamplerSlicer,
                        ] {
                            let cat_nodes = registry.list_by_category(cat);
                            if !cat_nodes.is_empty() {
                                ui.label(RichText::new(format!("{} {}", cat.icon(), cat.name())).font(FontId::proportional(9.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                                for desc in cat_nodes.iter().take(8) {
                                    if ui.selectable_label(false, format!("  {} {}", desc.category.icon(), desc.display_name)).clicked() {
                                        module_to_add = Some(desc.kind_id.clone());
                                    }
                                }
                                ui.separator();
                            }
                        }
                    });

                if let Some(kind_id) = module_to_add {
                    if let Some(desc) = registry.get(&kind_id) {
                        state.add_device(&desc.kind_id, &desc.display_name);
                        state.node_param_values.clear();
                        for (p_i, schema) in desc.params.iter().enumerate() {
                            let def = match &schema.widget {
                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                _ => 0.5,
                            };
                            state.node_param_values.insert(schema.id.clone(), def);
                            if let Some(bus) = param_bus {
                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                if bus.get(pid).is_some() {
                                    bus.set(pid, def);
                                }
                            }
                        }
                    }
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if !state.is_enabled {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("DEVICE BYPASSED — Click ⏻ to Enable").font(FontId::proportional(13.0)).color(Color32::from_rgb(148, 163, 184)));
                });
                return;
            }

            // Check if expanded Pro Parameters drawer is active
            if state.is_expanded_params {
                show_pro_parameter_drawer(ui, state, opt_desc, oscilloscope_data, cat_accent, param_bus, track_id);
                return;
            }

            // 2. Chassis Main Body (5 distinct modular sections)
            ui.horizontal(|ui| {
                // Section 1: Rotary Knobs Grid (2 rows x 3 cols)
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if let Some(desc) = opt_desc {
                                let knob_params: Vec<&crate::dsp_node_ui::DspParamSchema> = desc
                                    .params
                                    .iter()
                                    .filter(|p| matches!(p.widget, crate::dsp_node_ui::DspWidgetKind::RotaryKnob { .. } | crate::dsp_node_ui::DspWidgetKind::VerticalFader { .. }))
                                    .collect();

                                // Top Row: first 3 parameters
                                ui.horizontal(|ui| {
                                    for i in 0..3 {
                                        if let Some(&p) = knob_params.get(i) {
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 0 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            let resp = draw_rotary_dial(ui, &p.name, &mut val, color, default_norm);
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                state.node_param_values.insert(p.id.clone(), val);
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + i as u32);
                                                    if bus.get(pid).is_some() {
                                                        bus.set(pid, val);
                                                    }
                                                    let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + i as u32);
                                                    if bus.get(slot_pid).is_some() {
                                                        bus.set(slot_pid, val);
                                                    }
                                                }
                                            }
                                            if resp.secondary_clicked() {
                                                state.request_automation(&p.id);
                                            }
                                        } else {
                                            // Blank indicator for unoccupied slot
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75), 0.5);
                                        }
                                        if i < 2 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                // Bottom Row: next 3 parameters (indices 3..6)
                                ui.horizontal(|ui| {
                                    for i in 3..6 {
                                        if let Some(&p) = knob_params.get(i) {
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 3 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            let resp = draw_rotary_dial(ui, &p.name, &mut val, color, default_norm);
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                state.node_param_values.insert(p.id.clone(), val);
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + i as u32);
                                                    if bus.get(pid).is_some() {
                                                        bus.set(pid, val);
                                                    }
                                                    let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + i as u32);
                                                    if bus.get(slot_pid).is_some() {
                                                        bus.set(slot_pid, val);
                                                    }
                                                }
                                            }
                                            if resp.secondary_clicked() {
                                                state.request_automation(&p.id);
                                            }
                                        } else {
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75), 0.5);
                                        }
                                        if i < 5 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                            } else {
                                // Default / Fallback knobs
                                ui.horizontal(|ui| {
                                    let r1 = draw_rotary_dial(ui, "Cutoff", &mut state.cutoff, Color32::from_rgb(56, 189, 248), 0.65);
                                    if r1.secondary_clicked() { state.requested_automation_param = Some("cutoff".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r1.changed() || r1.dragged() || r1.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                            if bus.get(pid).is_some() { bus.set(pid, state.cutoff); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r2 = draw_rotary_dial(ui, "Reso", &mut state.resonance, Color32::from_rgb(245, 158, 11), 0.45);
                                    if r2.secondary_clicked() { state.requested_automation_param = Some("resonance".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r2.changed() || r2.dragged() || r2.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                            if bus.get(pid).is_some() { bus.set(pid, state.resonance); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r3 = draw_rotary_dial(ui, "Decay", &mut state.decay, Color32::from_rgb(245, 158, 11), 0.50);
                                    if r3.secondary_clicked() { state.requested_automation_param = Some("decay".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r3.changed() || r3.dragged() || r3.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 2);
                                            if bus.get(pid).is_some() { bus.set(pid, state.decay); }
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let r4 = draw_rotary_dial(ui, "Amt", &mut state.env_decay, Color32::from_rgb(56, 189, 248), 0.35);
                                    if r4.secondary_clicked() { state.requested_automation_param = Some("env_decay".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r4.changed() || r4.dragged() || r4.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 3);
                                            if bus.get(pid).is_some() { bus.set(pid, state.env_decay); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r5 = draw_rotary_dial(ui, "Mod", &mut state.mod_amt, Color32::from_rgb(245, 158, 11), 0.60);
                                    if r5.secondary_clicked() { state.requested_automation_param = Some("mod_amt".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r5.changed() || r5.dragged() || r5.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                            if bus.get(pid).is_some() { bus.set(pid, state.mod_amt); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r6 = draw_rotary_dial(ui, "Drive", &mut state.drive, Color32::from_rgb(245, 158, 11), 0.40);
                                    if r6.secondary_clicked() { state.requested_automation_param = Some("drive".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r6.changed() || r6.dragged() || r6.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 5);
                                            if bus.get(pid).is_some() { bus.set(pid, state.drive); }
                                        }
                                    }
                                });
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 2: Vertical Sliders (Osc Mix, Shape, Vol or dynamic reflection params 6..9)
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        let fader_params: Vec<&crate::dsp_node_ui::DspParamSchema> = if let Some(desc) = opt_desc {
                            if desc.params.len() > 6 {
                                desc.params[6..].iter().take(3).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        if !fader_params.is_empty() {
                            ui.horizontal(|ui| {
                                for (i, p) in fader_params.iter().enumerate() {
                                    let default_norm = match &p.widget {
                                        crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                        | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                            ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                        }
                                        _ => 0.5,
                                    };
                                    let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                    let resp = draw_vertical_fader(ui, &p.name, &mut val, default_norm);
                                    if resp.changed() || resp.dragged() || resp.double_clicked() {
                                        state.node_param_values.insert(p.id.clone(), val);
                                        if let Some(bus) = param_bus {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (6 + i) as u32);
                                            if bus.get(pid).is_some() {
                                                bus.set(pid, val);
                                            }
                                            let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (6 + i) as u32);
                                            if bus.get(slot_pid).is_some() {
                                                bus.set(slot_pid, val);
                                            }
                                        }
                                    }
                                    if resp.secondary_clicked() {
                                        state.request_automation(&p.id);
                                    }
                                    if i + 1 < fader_params.len() {
                                        ui.add_space(4.0);
                                    }
                                }
                                for i in fader_params.len()..3 {
                                    ui.add_space(4.0);
                                    if i == 1 {
                                        let resp = draw_vertical_fader(ui, "Shape", &mut state.shape, 0.50);
                                        if resp.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                        if let Some(bus) = param_bus {
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                                if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                            }
                                        }
                                    } else {
                                        let resp = draw_vertical_fader(ui, "Vol", &mut state.volume, 0.85);
                                        if resp.secondary_clicked() { state.requested_automation_param = Some("volume".to_string()); }
                                        if let Some(bus) = param_bus {
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                                if bus.get(pid).is_some() { bus.set(pid, state.volume); }
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                let r1 = draw_vertical_fader(ui, "Osc Mix", &mut state.osc_mix, 0.75);
                                if r1.secondary_clicked() { state.requested_automation_param = Some("osc_mix".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r1.changed() || r1.dragged() || r1.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 6);
                                        if bus.get(pid).is_some() { bus.set(pid, state.osc_mix); }
                                    }
                                }
                                ui.add_space(4.0);
                                let r2 = draw_vertical_fader(ui, "Shape", &mut state.shape, 0.50);
                                if r2.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r2.changed() || r2.dragged() || r2.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                        if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                    }
                                }
                                ui.add_space(4.0);
                                let r3 = draw_vertical_fader(ui, "Vol", &mut state.volume, 0.85);
                                if r3.secondary_clicked() { state.requested_automation_param = Some("volume".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r3.changed() || r3.dragged() || r3.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                        if bus.get(pid).is_some() { bus.set(pid, state.volume); }
                                    }
                                }
                            });
                        }
                    });

                ui.add_space(6.0);

                // Section 3: Real-Time CRT Oscilloscope Screen
                egui::Frame::none()
                    .fill(Color32::from_rgb(8, 12, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_width(170.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Oscilloscope").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(16, 185, 129)));
                                });
                            });
                            ui.add_space(2.0);

                            let (osc_resp, osc_painter) = ui.allocate_painter(Vec2::new(158.0, 96.0), egui::Sense::hover());
                            let osc_rect = osc_resp.rect;
                            osc_painter.rect_filled(osc_rect, 2.0, Color32::from_rgb(4, 7, 12));

                            // Draw subtle CRT grid
                            for g_y in 1..4 {
                                let gy = osc_rect.top() + (osc_rect.height() * (g_y as f32 / 4.0));
                                osc_painter.line_segment([egui::pos2(osc_rect.left(), gy), egui::pos2(osc_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgb(15, 25, 35)));
                            }

                            // Draw Neon Phosphor Green Wave
                            let points_count = 60;
                            let mut prev_pt = None;
                            for i in 0..points_count {
                                let norm_x = i as f32 / (points_count - 1) as f32;
                                let px = osc_rect.left() + norm_x * osc_rect.width();

                                let sample = if let Some(buf) = oscilloscope_data {
                                    let idx = (norm_x * (buf.len() as f32 - 1.0)) as usize;
                                    buf.get(idx).copied().unwrap_or(0.0)
                                } else {
                                    // Synthesize nice aesthetic wave
                                    (norm_x * 8.0 * std::f32::consts::PI).sin() * 0.45
                                        + (norm_x * 16.0 * std::f32::consts::PI).sin() * 0.25
                                };

                                let py = osc_rect.center().y - (sample * (osc_rect.height() * 0.42));
                                let current_pt = egui::pos2(px, py);

                                if let Some(last) = prev_pt {
                                    // Glow shadow
                                    osc_painter.line_segment([last, current_pt], Stroke::new(2.5_f32, Color32::from_rgba_unmultiplied(16, 185, 129, 60)));
                                    // Crisp center
                                    osc_painter.line_segment([last, current_pt], Stroke::new(1.2_f32, Color32::from_rgb(16, 185, 129)));
                                }
                                prev_pt = Some(current_pt);
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 4: Filter Frequency Curve Visualizer
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_width(170.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Filter Frequency").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(2.0);

                            let (filt_resp, filt_painter) = ui.allocate_painter(Vec2::new(158.0, 96.0), egui::Sense::click_and_drag());
                            let filt_rect = filt_resp.rect;
                            filt_painter.rect_filled(filt_rect, 2.0, Color32::from_rgb(8, 12, 20));

                            // Interactive drag handling for filter node 1 (cutoff)
                            if filt_resp.dragged() {
                                if let Some(pos) = filt_resp.interact_pointer_pos() {
                                    let nx = ((pos.x - filt_rect.left()) / filt_rect.width()).clamp(0.05, 0.95);
                                    let ny = (1.0 - ((pos.y - filt_rect.top()) / filt_rect.height())).clamp(0.05, 0.95);
                                    state.filter_nodes[1] = (nx, ny);
                                    state.cutoff = nx;
                                    state.resonance = ny;
                                    state.node_param_values.insert("cutoff".to_string(), nx);
                                    state.node_param_values.insert("resonance".to_string(), ny);
                                    if let Some(bus) = param_bus {
                                        let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                        if bus.get(pid_cut).is_some() { bus.set(pid_cut, nx); }
                                        let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                        if bus.get(pid_res).is_some() { bus.set(pid_res, ny); }
                                    }
                                }
                            }

                            // Draw shaded filter transfer curve
                            let steps = 40;
                            let mut curve_pts = Vec::with_capacity(steps);
                            for i in 0..steps {
                                let t = i as f32 / (steps - 1) as f32;
                                let px = filt_rect.left() + t * filt_rect.width();

                                // 4-pole lowpass roll-off curve shape
                                let cutoff_norm = state.filter_nodes[1].0;
                                let q = state.filter_nodes[1].1;
                                let gain = if t <= cutoff_norm {
                                    1.0 + (t / cutoff_norm) * (q - 0.5) * 0.6
                                } else {
                                    let roll = (t - cutoff_norm) / (1.0 - cutoff_norm).max(0.01);
                                    (1.0 + (q - 0.5) * 0.6) * (-roll * 3.5).exp()
                                };

                                let py = filt_rect.bottom() - (gain * filt_rect.height() * 0.70).clamp(2.0, filt_rect.height() - 2.0);
                                curve_pts.push(egui::pos2(px, py));
                            }

                            for w in curve_pts.windows(2) {
                                filt_painter.line_segment([w[0], w[1]], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));
                            }

                            // Draw draggable filter node pucks (44x44pt hit bounds)
                            for &(nx, ny) in &state.filter_nodes {
                                let px = filt_rect.left() + nx * filt_rect.width();
                                let py = filt_rect.top() + (1.0 - ny) * filt_rect.height();
                                filt_painter.circle_filled(egui::pos2(px, py), 4.5, Color32::from_rgb(56, 189, 248));
                                filt_painter.circle_stroke(egui::pos2(px, py), 5.5, Stroke::new(1.0_f32, Color32::WHITE));
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 5: LFO & Modulators / Schema Params 9..13
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        let mod_params: Vec<&crate::dsp_node_ui::DspParamSchema> = if let Some(desc) = opt_desc {
                            if desc.params.len() > 9 {
                                desc.params[9..].iter().take(4).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        if !mod_params.is_empty() {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Aux Params").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                                    ui.label(RichText::new("⚡").font(FontId::proportional(10.0)).color(cat_accent));
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    for (i, p) in mod_params.iter().take(2).enumerate() {
                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let resp = draw_rotary_dial(ui, &p.name, &mut val, cat_accent, default_norm);
                                        if resp.changed() || resp.dragged() || resp.double_clicked() {
                                            state.node_param_values.insert(p.id.clone(), val);
                                            if let Some(bus) = param_bus {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (9 + i) as u32);
                                                if bus.get(pid).is_some() {
                                                    bus.set(pid, val);
                                                }
                                                let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (9 + i) as u32);
                                                if bus.get(slot_pid).is_some() {
                                                    bus.set(slot_pid, val);
                                                }
                                            }
                                        }
                                        if resp.secondary_clicked() {
                                            state.request_automation(&p.id);
                                        }
                                        if i == 0 && mod_params.len() > 1 {
                                            ui.add_space(2.0);
                                        }
                                    }
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    for (i, p) in mod_params.iter().skip(2).take(2).enumerate() {
                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let resp = draw_rotary_dial(ui, &p.name, &mut val, cat_accent, default_norm);
                                        if resp.changed() || resp.dragged() || resp.double_clicked() {
                                            state.node_param_values.insert(p.id.clone(), val);
                                            if let Some(bus) = param_bus {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (11 + i) as u32);
                                                if bus.get(pid).is_some() {
                                                    bus.set(pid, val);
                                                }
                                                let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (11 + i) as u32);
                                                if bus.get(slot_pid).is_some() {
                                                    bus.set(slot_pid, val);
                                                }
                                            }
                                        }
                                        if resp.secondary_clicked() {
                                            state.request_automation(&p.id);
                                        }
                                        if i == 0 {
                                            ui.add_space(2.0);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("LFO").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                                    ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    let r1 = draw_rotary_dial(ui, "Rate", &mut state.lfo_speed, Color32::from_rgb(56, 189, 248), 0.40);
                                    if r1.secondary_clicked() { state.requested_automation_param = Some("lfo_speed".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r1.changed() || r1.dragged() || r1.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 8);
                                            if bus.get(pid).is_some() { bus.set(pid, state.lfo_speed); }
                                        }
                                    }
                                    let r2 = draw_rotary_dial(ui, "Depth", &mut state.lfo_depth, Color32::from_rgb(148, 163, 184), 0.60);
                                    if r2.secondary_clicked() { state.requested_automation_param = Some("lfo_depth".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r2.changed() || r2.dragged() || r2.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 9);
                                            if bus.get(pid).is_some() { bus.set(pid, state.lfo_depth); }
                                        }
                                    }
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    let r3 = draw_rotary_dial(ui, "Shape", &mut state.shape, Color32::from_rgb(56, 189, 248), 0.50);
                                    if r3.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r3.changed() || r3.dragged() || r3.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                            if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                        }
                                    }
                                    let r4 = draw_rotary_dial(ui, "Mod", &mut state.mod_amt, Color32::from_rgb(148, 163, 184), 0.60);
                                    if r4.secondary_clicked() { state.requested_automation_param = Some("mod_amt".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r4.changed() || r4.dragged() || r4.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                            if bus.get(pid).is_some() { bus.set(pid, state.mod_amt); }
                                        }
                                    }
                                });
                            });
                        }
                    });
            });
        });
}

#[cfg(feature = "gui")]
fn show_pro_parameter_drawer(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    opt_desc: Option<&crate::dsp_node_ui::DspNodeDescriptor>,
    oscilloscope_data: Option<&[f32]>,
    cat_accent: Color32,
    param_bus: Option<&summoner_core::param_bus::ParamBus>,
    track_id: u64,
) {
    ui.horizontal(|ui| {
        // Left Visualizer Column: CRT Mini Oscilloscope + Filter Curve
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(6.0, 6.0))
            .show(ui, |ui| {
                ui.set_width(170.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Scope & Curve").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(16, 185, 129)));
                        });
                    });
                    ui.add_space(2.0);

                    // CRT Mini Scope
                    let (osc_resp, osc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::hover());
                    let osc_rect = osc_resp.rect;
                    osc_painter.rect_filled(osc_rect, 2.0, Color32::from_rgb(4, 7, 12));
                    for g_y in 1..3 {
                        let gy = osc_rect.top() + (osc_rect.height() * (g_y as f32 / 3.0));
                        osc_painter.line_segment([egui::pos2(osc_rect.left(), gy), egui::pos2(osc_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgb(15, 25, 35)));
                    }
                    let points_count = 50;
                    let mut prev_pt = None;
                    for i in 0..points_count {
                        let norm_x = i as f32 / (points_count - 1) as f32;
                        let px = osc_rect.left() + norm_x * osc_rect.width();
                        let sample = if let Some(buf) = oscilloscope_data {
                            let idx = (norm_x * (buf.len() as f32 - 1.0)) as usize;
                            buf.get(idx).copied().unwrap_or(0.0)
                        } else {
                            (norm_x * 8.0 * std::f32::consts::PI).sin() * 0.45
                        };
                        let py = osc_rect.center().y - (sample * (osc_rect.height() * 0.42));
                        let current_pt = egui::pos2(px, py);
                        if let Some(last) = prev_pt {
                            osc_painter.line_segment([last, current_pt], Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));
                        }
                        prev_pt = Some(current_pt);
                    }

                    ui.add_space(4.0);

                    let dev_kind = state.chain_devices.get(state.selected_chain_idx).map(|d| d.kind.as_str())
                        .or(state.selected_node_kind.as_deref())
                        .unwrap_or("AetherSynth");
                    let is_crystal = dev_kind == "CrystalResonator" || dev_kind.contains("Crystal") || dev_kind.contains("Bowl");
                    let is_armonica = dev_kind == "GlassArmonica" || dev_kind == "FranklinGlassArmonica" || dev_kind == "ArmonicaChassisResonator" || dev_kind.contains("Armonica");
                    let is_hurdy = dev_kind == "FrenchHurdyGurdy" || dev_kind.contains("HurdyGurdy") || dev_kind.contains("Vielle");
                    let is_trompette = dev_kind == "TrompetteChienBridge" || dev_kind == "TrompetteBridge" || dev_kind.contains("Trompette") || dev_kind.contains("Chien");
                    let is_hoa = dev_kind == "AmbisonicRadarSpatializer" || dev_kind == "HoaSpatializer" || dev_kind == "Hoa5BinauralSpatializer" || dev_kind == "Hoa5RadarView" || dev_kind.contains("Ambisonic") || dev_kind.contains("Hoa");
                    let is_neural = dev_kind.contains("Neural") || dev_kind.contains("Timbre") || dev_kind.contains("Latent") || dev_kind.contains("Resynthesizer") || dev_kind == "NeuralMorphOrbView" || dev_kind == "NeuralTimbreMorph" || dev_kind == "NeuralWavetable";
                    let is_bowed = dev_kind.contains("Bowed") || dev_kind.contains("Violin") || dev_kind.contains("Cello") || dev_kind == "BowedStringNode" || dev_kind == "BowedString";
                    let is_shakuhachi = dev_kind.contains("Shakuhachi") || dev_kind.contains("BambooFlute") || dev_kind == "ShakuhachiNode" || dev_kind == "Shakuhachi";
                    let is_sitar = dev_kind.contains("Sitar") || dev_kind == "SitarNode" || dev_kind == "SitarModel" || dev_kind.contains("Jawari");
                    let is_turkish_ney = dev_kind.contains("TurkishNey") || dev_kind.contains("Ney") || dev_kind == "TurkishNeyNode";
                    let is_steelpan = dev_kind.contains("Steelpan") || dev_kind.contains("SteelDrum") || dev_kind == "SteelpanNode" || dev_kind == "SteelpanModel";
                    let is_mbira = dev_kind.contains("Mbira") || dev_kind.contains("Kalimba") || dev_kind == "MbiraNode" || dev_kind == "KalimbaNode" || dev_kind == "MbiraKalimbaModel" || dev_kind == "MbiraModel";
                    let is_koto = dev_kind.contains("Koto") || dev_kind.contains("Guzheng") || dev_kind == "KotoNode" || dev_kind == "KotoSynthesizer" || dev_kind == "KotoStringVoice" || dev_kind == "KotoSoundboard";
                    let is_gamelan = dev_kind.contains("Gamelan") || dev_kind.contains("Gender") || dev_kind.contains("Jegogan") || dev_kind.contains("Metallophone") || dev_kind == "GamelanGender" || dev_kind == "AcousticBalineseGamelanJegoganBronzeBar";
                    let is_dulcimer = dev_kind.contains("Dulcimer") || dev_kind.contains("Cimbalom") || dev_kind.contains("Santur") || dev_kind.contains("Yangqin") || dev_kind.contains("Psaltery") || dev_kind == "DulcimerCimbalomModel";
                    let is_clavinet = dev_kind.contains("Clavinet") || dev_kind.contains("Clav") || dev_kind == "ClavinetNode" || dev_kind == "ClavinetD6";
                    let is_electric_piano = dev_kind.contains("ElectricPiano") || dev_kind == "ElectricPianoModel" || dev_kind == "ElectricPianoProfile" || dev_kind.contains("Rhodes") || dev_kind.contains("Wurlitzer");
                    let is_tonewheel_organ = dev_kind.contains("Tonewheel") || dev_kind == "TonewheelOrgan" || dev_kind == "TonewheelOrganModel" || dev_kind == "TonewheelOrganNode" || dev_kind == "TonewheelOrganProfile" || dev_kind.contains("Hammond") || dev_kind.contains("Drawbar");
                    let is_piano = !is_electric_piano && (dev_kind.contains("Piano") || dev_kind.contains("GrandPiano") || dev_kind == "GrandPianoModel" || dev_kind == "ConcertGrandPianoNode");
                    let is_pipe_organ = !is_tonewheel_organ && (dev_kind.contains("PipeOrgan") || dev_kind.contains("Organ") || dev_kind.contains("Windchest") || dev_kind == "PipeOrganModel");
                    let is_waveguide_brass = dev_kind.contains("WaveguideBrass") || dev_kind.contains("BrassInstrument") || dev_kind.contains("AcousticBrass") || dev_kind == "WaveguideBrassModel" || dev_kind == "WaveguideBrassNode" || dev_kind == "BrassInstrumentProfile";
                    let is_vocal_tract = dev_kind.contains("VocalTract") || dev_kind.contains("VocalTractModel") || dev_kind.contains("VowelSpace") || dev_kind.contains("VocalFormant") || dev_kind == "VocalTractNode" || dev_kind == "VocalTractProfile";
                    let is_rotary_speaker = dev_kind.contains("Rotary") || dev_kind.contains("Leslie") || dev_kind.contains("DopplerSpeaker") || dev_kind == "RotarySpeakerNode" || dev_kind == "DualRotorLeslieDopplerSpeaker" || dev_kind == "RotaryCabinetModel";
                    let is_free_reed = dev_kind.contains("FreeReed") || dev_kind.contains("Harmonium") || dev_kind.contains("Accordion") || dev_kind.contains("Bandoneon") || dev_kind.contains("Bayan") || dev_kind.contains("Cassotto") || dev_kind == "FreeReedNode" || dev_kind == "FreeReedVoice" || dev_kind == "FreeReedOscillator";
                    let is_granular = dev_kind.contains("Granular") || dev_kind.contains("Grain") || dev_kind == "GranularSynthNode" || dev_kind == "GranularFreezeNode" || dev_kind == "GranularPitchShifter" || dev_kind == "GranularFreezeView" || dev_kind.contains("Freeze") || dev_kind.contains("CloudFreeze");
                    let is_spring_lattice = dev_kind.contains("Spring") || dev_kind.contains("Lattice") || dev_kind == "SpringLatticeNode" || dev_kind == "SpringReverb" || dev_kind == "SpringTankModel" || dev_kind == "SpringMassLattice";
                    let is_waveguide_mesh = dev_kind.contains("WaveguideMesh") || dev_kind.contains("MeshPlate") || dev_kind.contains("MeshMembrane") || dev_kind == "WaveguideMeshNode" || dev_kind == "WaveguideMesh2D" || dev_kind == "RectilinearWaveguideMesh" || dev_kind == "WaveguideMeshBoundary" || dev_kind == "WaveguideMesh";
                    let is_plucked_string = dev_kind.contains("PluckedString") || dev_kind.contains("CommutedPlucked") || dev_kind == "PluckedStringNode" || dev_kind == "CommutedPluckedStringNode" || dev_kind == "CommutedPluckedString" || dev_kind == "PluckedStringWaveguide" || dev_kind == "KarplusStrong";
                    let is_bellows = dev_kind.contains("Bellows") || dev_kind.contains("Cassotto") || dev_kind == "PneumaticBellows" || dev_kind == "CassottoChamber" || dev_kind == "AccordionBellows" || dev_kind == "BellowsDynamics";
                    let is_jawari_bridge = dev_kind.contains("Jawari") || dev_kind.contains("Tarab") || dev_kind == "JawariBridge" || dev_kind == "TarabResonatorBank" || dev_kind == "TarabStringResonator" || dev_kind == "SitarGourdBody" || dev_kind == "SitarBridge";
                    let is_soundboard = dev_kind.contains("Soundboard") || dev_kind.contains("SoundboardBridge") || dev_kind == "SoundboardBridgeModel" || dev_kind == "PianoSoundboard" || dev_kind == "SpruceSoundboard";
                    let is_sympathetic = dev_kind.contains("Sympathetic") || dev_kind.contains("SympatheticCoupling") || dev_kind == "SympatheticCouplingModel" || dev_kind == "SympatheticStringMatrix" || dev_kind == "DroneStrings";
                    let is_woodwind_jet = dev_kind.contains("WoodwindJet") || dev_kind.contains("FluteJet") || dev_kind.contains("AirJet") || dev_kind.contains("WoodwindModel") || dev_kind == "WoodwindJetView" || dev_kind == "AeroacousticFluteJetExciter" || dev_kind == "AcousticConcertFluteAirJetModel" || dev_kind == "WoodwindJetNode";
                    let is_tonehole_matrix = dev_kind.contains("Tonehole") || dev_kind.contains("BoreModel") || dev_kind == "ToneholeMatrixView" || dev_kind == "ToneholeMatrix" || dev_kind == "ToneholeGrid" || dev_kind == "WoodwindToneholeGrid" || dev_kind == "ToneholeLattice";
                    let is_plate_dispersion = dev_kind.contains("PlateTank") || dev_kind.contains("PlateReverb") || dev_kind.contains("PlateDispersion") || dev_kind == "PlateTankNode" || dev_kind == "PlateDispersionView" || dev_kind == "PlateReverbNode" || dev_kind.contains("EMT140");
                    let is_membrane_resonator = dev_kind.contains("MembraneResonator") || dev_kind == "MembraneResonatorView" || dev_kind == "MembraneResonatorNode";
                    let is_membrane_plate = dev_kind.contains("MembranePlate") || dev_kind == "MembranePlateView" || dev_kind == "MembranePlateModel";
                    let is_membrane_cavity = (dev_kind.contains("PercussionMembrane") || dev_kind.contains("MembranePercussion") || dev_kind.contains("MembraneCavity") || dev_kind == "PercussionMembrane" || dev_kind == "MembraneCavityView" || dev_kind == "MembranePercussionModel") && !is_membrane_resonator && !is_membrane_plate;
                    let is_embouchure_angle = dev_kind.contains("EmbouchureAngle") || dev_kind.contains("ShakuhachiEmbouchure") || dev_kind.contains("FluteEmbouchure") || dev_kind == "EmbouchureAngleView" || dev_kind == "EmbouchureAngle" || dev_kind.contains("MeriKari") || dev_kind.contains("Utaguchi");
                    let is_friction_orbit = dev_kind.contains("FrictionOrbit") || dev_kind.contains("StickSlip") || dev_kind.contains("BowedFriction") || dev_kind == "FrictionOrbitView" || dev_kind == "FrictionOrbit" || dev_kind.contains("RosinHysteresis") || dev_kind.contains("HelmholtzOrbit");
                    let is_rank_voicing = dev_kind.contains("RankVoicing") || dev_kind.contains("PipeOrganFlute") || dev_kind.contains("OrganRank") || dev_kind == "RankVoicingView" || dev_kind == "RankVoicing" || dev_kind.contains("FlueCutup") || dev_kind.contains("PneumaticPipeOrganFluteRank");
                    let is_tine_resonator = dev_kind.contains("TineResonator") || dev_kind.contains("TinePickup") || dev_kind.contains("TineResonatorPickup") || dev_kind == "TineResonatorView" || dev_kind == "TineResonator" || dev_kind.contains("EpArticulation") || dev_kind.contains("ElectricPiano");
                    let is_spring_reverb = dev_kind.contains("SpringReverb") || dev_kind.contains("SpringTank") || dev_kind.contains("AnalogSpring") || dev_kind.contains("SpringMass") || dev_kind == "SpringReverbView" || dev_kind == "AnalogSpringReverbTank" || dev_kind == "NonlinearSpringReverbTank";
                    let is_comb_resonator = dev_kind.contains("CombResonator") || dev_kind.contains("CombFilter") || dev_kind.contains("ResonantComb") || dev_kind == "CombResonatorView" || dev_kind == "CombResonatorNode" || dev_kind == "ResonantPhasedCombArray";
                    let is_wavefront_reflection = dev_kind.contains("WavefrontReflection") || dev_kind.contains("Wavefront") || dev_kind.contains("RaytracedEarlyReflection") || dev_kind == "WavefrontReflectionView" || dev_kind == "WavefrontReflectionNode";
                    let is_diffractive_propagation = dev_kind.contains("DiffractivePropagation") || dev_kind.contains("Diffraction") || dev_kind == "DiffractivePropagationView" || dev_kind == "DiffractivePropagationNode";
                    let is_idiophone = dev_kind.contains("Idiophone") || dev_kind.contains("Marimba") || dev_kind.contains("Xylophone") || dev_kind.contains("Vibraphone") || dev_kind == "IdiophoneSpectrum" || dev_kind == "StruckIdiophoneResonatorNode" || dev_kind == "IdiophoneResonator" || dev_kind == "ModalIdiophoneResonator" || dev_kind == "ModalIdiophoneChimeBank";
                    let is_sonar_hydrophone = dev_kind.contains("Sonar") || dev_kind.contains("Hydrophone") || dev_kind == "SonarHydrophoneNode" || dev_kind == "SonarHydrophoneView";
                    let is_hoa5_binaural = dev_kind.contains("Hoa5") || dev_kind.contains("Ambisonic5") || dev_kind.contains("HOA5") || dev_kind == "Hoa5BinauralView" || dev_kind == "Hoa5BinauralNode" || dev_kind == "Ambisonics5thOrderDecoder" || dev_kind.contains("HigherOrderAmbisonics") || dev_kind.contains("Hoa5thOrder");
                    let is_mpegh_3d = dev_kind.contains("Mpegh") || dev_kind.contains("MPEG-H") || dev_kind.contains("MpegH") || dev_kind == "Mpegh3DSpatializerView" || dev_kind == "Mpegh3DSpatializerNode" || dev_kind == "Mpegh3DObjectSpatializer" || dev_kind.contains("MpeghSpatializer");
                    let is_atmos_916 = dev_kind.contains("Atmos916") || dev_kind.contains("DolbyAtmos") || dev_kind == "Atmos916Spatializer" || dev_kind == "Atmos916SpatializerView" || dev_kind.contains("AtmosBed") || dev_kind.contains("AtmosSpatializer");
                    let is_auro3d = dev_kind.contains("Auro3d") || dev_kind.contains("Auro-3D") || dev_kind.contains("Auro131") || dev_kind == "Auro3dSpatializer" || dev_kind == "Auro3dSpatializerView" || dev_kind.contains("AuroSpatializer");
                    let is_nhk222 = dev_kind.contains("Nhk222") || dev_kind.contains("NHK222") || dev_kind == "Nhk222Spatializer" || dev_kind == "Nhk222SpatializerView" || dev_kind.contains("NhkSpatializer");
                    let is_wfs_array = dev_kind.contains("WfsArray") || dev_kind.contains("WfsAcoustic") || dev_kind.contains("WfsPlanar") || dev_kind == "WfsArraySpatializerNode" || dev_kind == "WfsArraySpatializerView" || dev_kind.contains("WaveFieldSynthesis");
                    let is_binaural_panner = dev_kind.contains("BinauralSpatialPanner") || dev_kind.contains("BinauralPanner") || dev_kind == "BinauralPannerView" || dev_kind == "BinauralSpatialPannerNode" || dev_kind.contains("BinauralHrtf") || dev_kind.contains("HrtfPanner");
                    let is_binaural_brir = dev_kind.contains("BinauralBrir") || dev_kind.contains("PartitionedBinauralHrtfConvolver") || dev_kind == "BinauralBrirView" || dev_kind.contains("BrirSpatializer") || dev_kind.contains("RoomImpulseResponse") || dev_kind.contains("BinauralRoom");
                    let is_continuous_doppler = dev_kind.contains("ContinuousSpatialPanner") || dev_kind.contains("DistanceDoppler") || dev_kind.contains("DopplerSpatializer") || dev_kind == "ContinuousSpatialPannerDoppler3D" || dev_kind == "ContinuousDopplerView" || dev_kind.contains("BinauralDoppler") || dev_kind.contains("AmbisonicBFormatDoppler");
                    let is_spatial_room = dev_kind.contains("SpatialRoom") || dev_kind.contains("ImageSourceRoom") || dev_kind == "SpatialRoomNode" || dev_kind == "SpatialRoomView" || dev_kind.contains("RoomAcousticModel") || dev_kind.contains("ShoeboxRoom");
                    let is_rotary_doppler = dev_kind.contains("RotaryDualHorn") || dev_kind.contains("RotaryDualDrum") || dev_kind.contains("RotarySpeakerAcousticDeflector") || dev_kind.contains("RotaryDoppler") || dev_kind.contains("RotaryCabinet") || dev_kind.contains("DualRotorLeslie") || dev_kind == "RotaryDopplerView";
                    let is_raytraced_reverb = dev_kind.contains("VolumetricRaytracedRoomAcoustics") || dev_kind.contains("BinauralRaytracedEarlyReflection") || dev_kind.contains("RaytracedReverb") || dev_kind == "RaytracedReverbView" || dev_kind.contains("RaytracedEarlyReflection") || dev_kind.contains("RaytracedRoom");
                    let is_atmos_surround = dev_kind.contains("AtmosSurround") || dev_kind.contains("SurroundStemSplitterBedObject") || dev_kind.contains("SurroundDownmixer") || dev_kind == "AtmosSurroundNode" || dev_kind == "AtmosSurroundView";
                    let is_atmos_proximity = dev_kind.contains("AtmosProximity") || dev_kind == "AtmosProximityNode" || dev_kind == "AtmosProximityView";
                    let is_mpegh_trajectory = dev_kind.contains("MpeghTrajectory") || dev_kind.contains("MpegHTrajectory") || dev_kind == "MpeghTrajectorySpatializer" || dev_kind == "MpeghTrajectoryView" || dev_kind == "MpeghTrajectoryNode";
                    let is_nhk222_immersion = dev_kind.contains("Nhk222Immersion") || dev_kind.contains("NHK222Immersion") || dev_kind == "Nhk222ImmersionView" || dev_kind == "Nhk222ImmersionNode";
                    let is_hoa4_spatializer = dev_kind.contains("Hoa4") || dev_kind.contains("HOA4") || dev_kind == "Hoa4SpatializerView" || dev_kind == "Hoa4Spatializer" || dev_kind.contains("Ambisonics4thOrder") || dev_kind.contains("Hoa4Sphere");
                    let is_mpegh_spatializer = dev_kind.contains("MpeghSpatializer") || dev_kind.contains("MPEGHSpatializer") || dev_kind == "MpeghSpatializerView" || dev_kind == "MpeghSpatializerNode" || dev_kind.contains("MpeghHrtf") || dev_kind == "MpeghSpatialAudio";
                    let is_spatial_trajectory = dev_kind.contains("SpatialTrajectory") || dev_kind == "SpatialTrajectoryView" || dev_kind.contains("TrajectorySpline") || dev_kind == "SpatialTrajectoryNode";
                    let is_ambisonic_radar = dev_kind.contains("AmbisonicRadar") || dev_kind == "AmbisonicRadarView" || dev_kind.contains("AmbisonicPanner") || dev_kind == "AmbisonicRadarNode" || dev_kind.contains("SurroundPanner");
                    let is_hoa_spatializer = dev_kind.contains("HoaSpatializer") || dev_kind == "HoaSpatializerView" || dev_kind == "HoaSpatializerNode" || dev_kind.contains("DolbyAtmos714");
                    let is_neural_timbre_morph = dev_kind.contains("NeuralTimbreMorph") || dev_kind.contains("TimbreMorph") || dev_kind == "NeuralTimbreMorphView" || dev_kind == "NeuralTimbreMorphNode" || dev_kind == "NeuralTimbreMorphSynthesizer" || dev_kind.contains("DiffusionVocoder");
                    let is_neural_radiance = dev_kind.contains("NeuralRadiance") || dev_kind.contains("NeRFAcoustic") || dev_kind == "NeuralRadianceView" || dev_kind == "NeuralRadianceNode" || dev_kind.contains("ContinuousRadiance");
                    let is_neural_vocal_stylizer = dev_kind.contains("NeuralVocalStylizer") || dev_kind.contains("VocalStylizer") || dev_kind == "NeuralVocalStylizerView" || dev_kind == "NeuralVocalStylizerNode" || dev_kind.contains("VocalExpression") || dev_kind.contains("MelismaResynthesizer");
                    let is_neural_speech_to_singing = dev_kind.contains("NeuralSpeechToSinging") || dev_kind.contains("SpeechToSinging") || dev_kind == "NeuralSpeechToSingingView" || dev_kind == "NeuralSpeechToSingingNode" || dev_kind.contains("SingingVoice") || dev_kind.contains("ProsodyIntonation");
                    let is_neural_choir_formant = dev_kind.contains("NeuralChoirFormant") || dev_kind.contains("ChoirFormant") || dev_kind == "NeuralChoirFormantView" || dev_kind == "NeuralChoirFormantNode" || dev_kind.contains("ChoirEnsemble") || dev_kind.contains("VowelTrajectory");
                    let is_neural_vocoder_morph = dev_kind.contains("NeuralVocoderMorph") || dev_kind.contains("VocoderMorph") || dev_kind == "NeuralVocoderMorphView" || dev_kind == "NeuralVocoderMorphNode";
                    let is_neural_wavetable = dev_kind.contains("NeuralWavetable") || dev_kind.contains("DifferentialWavetable") || dev_kind == "NeuralWavetableView" || dev_kind == "NeuralDifferentialWavetableLatentOscillator" || dev_kind.contains("LatentWavetable") || dev_kind == "NeuralDifferentiableLatentWavetableSynthesizer";
                    let is_neural_phoneme = dev_kind.contains("NeuralPhoneme") || dev_kind.contains("PhonemeMorph") || dev_kind == "NeuralPhonemeView" || dev_kind == "NeuralPhonemeNode" || dev_kind.contains("Phoneme");
                    let is_neural_dereverb = dev_kind.contains("NeuralDereverb") || dev_kind.contains("Dereverb") || dev_kind == "NeuralDereverbView" || dev_kind == "NeuralDereverbNode" || dev_kind.contains("Deconvolution");
                    let is_neural_super_res = dev_kind.contains("NeuralSuperResolution") || dev_kind.contains("SuperResolution") || dev_kind == "NeuralSuperResolutionView" || dev_kind == "NeuralSuperResolutionNode" || dev_kind.contains("SuperRes");
                    let is_nam_amp = dev_kind.contains("NamAmp") || dev_kind.contains("NamModel") || dev_kind == "NamAmpView" || dev_kind == "NamAmpNode" || dev_kind.contains("NeuralAmp") || dev_kind.contains("WaveNet");
                    let is_neural_inpaint = dev_kind.contains("NeuralInpaint") || dev_kind.contains("Inpaint") || dev_kind == "NeuralInpaintView" || dev_kind == "NeuralInpaintNode" || dev_kind.contains("SpectralRepair");
                    let is_neural_timbre = dev_kind.contains("NeuralTimbre") || dev_kind.contains("TimbreTransfer") || dev_kind == "NeuralTimbreView" || dev_kind == "NeuralTimbreNode" || dev_kind.contains("LatentFlow");
                    let is_bitcrusher = dev_kind.contains("Bitcrusher") || dev_kind.contains("BitCrush") || dev_kind == "BitcrusherMorphView" || dev_kind == "BitcrusherNode" || dev_kind.contains("SampleRateReduction");
                    let is_bbd_chorus = dev_kind.contains("BbdChorus") || dev_kind.contains("Chorus") || dev_kind == "BbdChorusView" || dev_kind == "EffectChorus" || dev_kind.contains("BucketBrigade") || dev_kind.contains("LissajousChorus");
                    let is_tube = dev_kind.contains("Tube") || dev_kind.contains("Vacuum") || dev_kind.contains("Triode") || dev_kind == "TubeBiasView" || dev_kind == "TubeSaturation" || dev_kind.contains("TubeAmp");
                    let is_tape = dev_kind.contains("Tape") || dev_kind.contains("Hysteresis") || dev_kind == "TapeSaturationView" || dev_kind == "TapeSaturation" || dev_kind.contains("TapeWarmth") || dev_kind.contains("ReelToReel");
                    let is_ladder = dev_kind.contains("Ladder") || dev_kind.contains("Moog") || dev_kind.contains("Tb303") || dev_kind == "LadderFilterView" || dev_kind == "LadderFilter" || dev_kind.contains("FilterLadder");
                    let is_granular_freeze = dev_kind.contains("Freeze") || dev_kind == "GranularFreezeView" || dev_kind == "GranularFreeze" || dev_kind.contains("CloudFreeze") || dev_kind == "GranularFreezeNode";
                    let is_auto_wah = dev_kind.contains("AutoWah") || dev_kind.contains("Wah") || dev_kind == "AutoWahView" || dev_kind == "AutoWahNode" || dev_kind.contains("WahFilter");
                    let is_formant = dev_kind.contains("Formant") || dev_kind.contains("Vowel") || dev_kind == "FormantFilterView" || dev_kind == "FormantFilterNode" || dev_kind.contains("FormantFilter");
                    let is_limiter = dev_kind.contains("Limiter") || dev_kind.contains("TruePeak") || dev_kind == "OversampledLimiterNode" || dev_kind == "TruePeakOversampledLimiter" || dev_kind.contains("PeakLimiter");
                    let is_vari_mu = dev_kind.contains("VariMu") || dev_kind.contains("VariableMu") || dev_kind == "VariMuMasterNode" || dev_kind == "VariMuMasterLimiter" || dev_kind == "MasterStereoVariMuLimiter" || dev_kind.contains("ValveLimiter");

                    if is_crystal {
                        // Crystal Resonator 2D Acoustic Rim & Hydro-Acoustic Water Level Visualizer
                        let (c_resp, c_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let c_rect = c_resp.rect;
                        c_painter.rect_filled(c_rect, 2.0, Color32::from_rgb(6, 12, 22));

                        let mut water_fill = state.node_param_values.get("water_fill_level").copied().unwrap_or(0.25);
                        let mut friction = state.node_param_values.get("friction_velocity").copied().unwrap_or(0.50);

                        if c_resp.dragged() {
                            if let Some(pos) = c_resp.interact_pointer_pos() {
                                friction = ((pos.x - c_rect.left()) / c_rect.width()).clamp(0.0, 1.0);
                                water_fill = (1.0 - ((pos.y - c_rect.top()) / c_rect.height())).clamp(0.0, 1.0);
                                state.node_param_values.insert("water_fill_level".to_string(), water_fill);
                                state.node_param_values.insert("friction_velocity".to_string(), friction);
                                if let Some(bus) = param_bus {
                                    let pid_water = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_water).is_some() { bus.set(pid_water, water_fill); }
                                    let pid_fric = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_fric).is_some() { bus.set(pid_fric, friction); }
                                }
                            }
                        }

                        // Draw singing bowl outer rim & inner cavity
                        let center = c_rect.center();
                        let radius = 26.0_f32;
                        c_painter.circle_stroke(center, radius, Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                        c_painter.circle_filled(center, radius * 0.88, Color32::from_rgb(10, 20, 36));

                        // Hydro-acoustic water fill meniscus line
                        let water_y = center.y + (1.0 - 2.0 * water_fill) * radius * 0.75;
                        let half_w = (radius * radius - (water_y - center.y).powi(2)).max(0.0).sqrt() * 0.85;
                        c_painter.line_segment(
                            [egui::pos2(center.x - half_w, water_y), egui::pos2(center.x + half_w, water_y)],
                            Stroke::new(1.5_f32, Color32::from_rgb(34, 211, 238)),
                        );

                        // Rotating wand friction puck on rim
                        let angle = friction * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                        let puck_pos = egui::pos2(center.x + angle.cos() * radius, center.y + angle.sin() * radius);
                        c_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(245, 158, 11));

                        // Text labels: Water fill & Friction
                        c_painter.text(
                            egui::pos2(c_rect.left() + 4.0, c_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("💧 {:.0}%", water_fill * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(34, 211, 238),
                        );
                        c_painter.text(
                            egui::pos2(c_rect.right() - 4.0, c_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("⚡ {:.0}%", friction * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if c_resp.hovered() {
                            let _ = c_resp.on_hover_text("Crystal Singing Bowl Rim & Hydro-Acoustic Loading\n[Drag horizontally: Stick-slip friction | Drag vertically: Water fill level]");
                        }
                    } else if is_armonica {
                        // Glass Armonica Spindle Axis & Concentric Bowl Visualizer
                        let (a_resp, a_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let a_rect = a_resp.rect;
                        a_painter.rect_filled(a_rect, 2.0, Color32::from_rgb(18, 12, 8));

                        let mut speed = state.node_param_values.get("rotation_speed_rad_s").copied().unwrap_or(2.5);
                        let mut force = state.node_param_values.get("normal_force_n").copied().unwrap_or(0.45);

                        if a_resp.dragged() {
                            if let Some(pos) = a_resp.interact_pointer_pos() {
                                speed = 0.1 + ((pos.x - a_rect.left()) / a_rect.width()).clamp(0.0, 1.0) * 9.9;
                                force = 0.05 + (1.0 - ((pos.y - a_rect.top()) / a_rect.height())).clamp(0.0, 1.0) * 0.95;
                                state.node_param_values.insert("rotation_speed_rad_s".to_string(), speed);
                                state.node_param_values.insert("normal_force_n".to_string(), force);
                                if let Some(bus) = param_bus {
                                    let pid_spd = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_spd).is_some() { bus.set(pid_spd, speed); }
                                    let pid_frc = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_frc).is_some() { bus.set(pid_frc, force); }
                                }
                            }
                        }

                        // Central horizontal spindle axis
                        let cy = a_rect.center().y;
                        a_painter.line_segment(
                            [egui::pos2(a_rect.left() + 10.0, cy), egui::pos2(a_rect.right() - 10.0, cy)],
                            Stroke::new(2.0_f32, Color32::from_rgb(148, 163, 184)),
                        );

                        // 5 Nested concentric glass cup profiles
                        for (i, cup_r) in [12.0_f32, 16.0, 20.0, 24.0, 28.0].iter().enumerate() {
                            let cx = a_rect.left() + 25.0 + i as f32 * 26.0;
                            a_painter.line_segment(
                                [egui::pos2(cx, cy - cup_r), egui::pos2(cx, cy + cup_r)],
                                Stroke::new(1.5_f32, Color32::from_rgb(245, 158, 11)),
                            );
                        }

                        // Wet finger contact indicator
                        let contact_x = a_rect.left() + 25.0 + 3.0 * 26.0;
                        a_painter.circle_filled(egui::pos2(contact_x, cy - 24.0), 3.5, Color32::from_rgb(56, 189, 248));

                        // Readout
                        a_painter.text(
                            egui::pos2(a_rect.left() + 4.0, a_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("ω: {:.1} rad/s", speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );
                        a_painter.text(
                            egui::pos2(a_rect.right() - 4.0, a_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("F: {:.2} N", force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if a_resp.hovered() {
                            let _ = a_resp.on_hover_text("Glass Armonica Spindle & Wet Friction Contact\n[Drag horizontally: Spindle angular velocity | Drag vertically: Normal force]");
                        }
                    } else if is_hurdy {
                        // Hurdy-Gurdy 2D Rosined Crank Wheel & Coup de Poignet Acceleration Visualizer
                        let (h_resp, h_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let h_rect = h_resp.rect;
                        h_painter.rect_filled(h_rect, 2.0, Color32::from_rgb(14, 18, 26));

                        let mut crank_speed = state.node_param_values.get("crank_speed_rad_s").copied().unwrap_or(2.0 * std::f32::consts::PI);
                        let mut wrist_accel = state.node_param_values.get("wrist_acceleration_pulse").copied().unwrap_or(1.2);

                        if h_resp.dragged() {
                            if let Some(pos) = h_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - h_rect.left()) / h_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - h_rect.top()) / h_rect.height())).clamp(0.0, 1.0);
                                wrist_accel = norm_x * 5.0;
                                crank_speed = norm_y * 4.0 * std::f32::consts::PI;
                                state.node_param_values.insert("crank_speed_rad_s".to_string(), crank_speed);
                                state.node_param_values.insert("wrist_acceleration_pulse".to_string(), wrist_accel);
                                if let Some(bus) = param_bus {
                                    let pid_speed = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_speed).is_some() { bus.set(pid_speed, crank_speed); }
                                    let pid_accel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_accel).is_some() { bus.set(pid_accel, wrist_accel); }
                                }
                            }
                        }

                        // Background grid
                        for gx in 1..4 {
                            let px = h_rect.left() + h_rect.width() * (gx as f32 / 4.0);
                            h_painter.line_segment([egui::pos2(px, h_rect.top()), egui::pos2(px, h_rect.bottom())], Stroke::new(0.5_f32, Color32::from_rgb(26, 36, 48)));
                        }

                        // Wooden Rosined Wheel Circle Arc
                        let wheel_center = egui::pos2(h_rect.left() + 45.0, h_rect.center().y);
                        let wheel_r = 24.0;
                        h_painter.circle_stroke(wheel_center, wheel_r, Stroke::new(2.5_f32, Color32::from_rgb(180, 83, 9)));
                        h_painter.circle_stroke(wheel_center, wheel_r - 3.0, Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)));

                        // Rotating crank indicator angle based on crank_speed
                        let crank_angle = (crank_speed * 1.5) % (2.0 * std::f32::consts::PI);
                        let crank_handle = egui::pos2(wheel_center.x + crank_angle.cos() * (wheel_r - 6.0), wheel_center.y + crank_angle.sin() * (wheel_r - 6.0));
                        h_painter.line_segment([wheel_center, crank_handle], Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)));
                        h_painter.circle_filled(crank_handle, 3.5, Color32::from_rgb(234, 88, 12));

                        // Strings across wheel: Chanterelles & Bourdon Drone
                        let str_y1 = h_rect.center().y - 10.0;
                        let str_y2 = h_rect.center().y;
                        let str_y3 = h_rect.center().y + 10.0;
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y1), egui::pos2(h_rect.right() - 10.0, str_y1)], Stroke::new(1.2_f32, Color32::from_rgb(56, 189, 248)));
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y2), egui::pos2(h_rect.right() - 10.0, str_y2)], Stroke::new(1.2_f32, Color32::from_rgb(56, 189, 248)));
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y3), egui::pos2(h_rect.right() - 10.0, str_y3)], Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));

                        // Coup de Poignet Wrist Impulse Puck
                        let puck_x = h_rect.left() + (wrist_accel / 5.0).clamp(0.0, 1.0) * h_rect.width();
                        let puck_y = h_rect.bottom() - (crank_speed / (4.0 * std::f32::consts::PI)).clamp(0.0, 1.0) * h_rect.height();
                        h_painter.circle_filled(egui::pos2(puck_x, puck_y), 5.0, Color32::from_rgb(239, 68, 68));
                        h_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 202, 202)));

                        // Readout
                        h_painter.text(
                            egui::pos2(h_rect.left() + 4.0, h_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("ω: {:.1} rad/s", crank_speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(251, 191, 36),
                        );
                        h_painter.text(
                            egui::pos2(h_rect.right() - 4.0, h_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Coup: {:.1}", wrist_accel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(239, 68, 68),
                        );

                        if h_resp.hovered() {
                            let _ = h_resp.on_hover_text("Hurdy-Gurdy Rosined Crank Wheel & Coup de Poignet\n[Drag horizontally: Wrist impulse | Drag vertically: Crank speed]");
                        }
                    } else if is_trompette {
                        // Trompette Chien Buzzing Bridge 2D Collision Gap Visualizer
                        let (t_resp, t_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let t_rect = t_resp.rect;
                        t_painter.rect_filled(t_rect, 2.0, Color32::from_rgb(24, 12, 14));

                        let mut clearance_gap = state.node_param_values.get("chien_gap_clearance_mm").copied().unwrap_or(0.35);
                        let mut strike_force = state.node_param_values.get("strike_force").copied().unwrap_or(4.0);

                        if t_resp.dragged() {
                            if let Some(pos) = t_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - t_rect.left()) / t_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - t_rect.top()) / t_rect.height())).clamp(0.0, 1.0);
                                clearance_gap = 0.05 + norm_x * (1.20 - 0.05);
                                strike_force = norm_y * 10.0;
                                state.node_param_values.insert("chien_gap_clearance_mm".to_string(), clearance_gap);
                                state.node_param_values.insert("strike_force".to_string(), strike_force);
                                if let Some(bus) = param_bus {
                                    let pid_gap = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_gap).is_some() { bus.set(pid_gap, clearance_gap); }
                                    let pid_force = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_force).is_some() { bus.set(pid_force, strike_force); }
                                }
                            }
                        }

                        // Soundboard Bone Striking Plate (rigid obstacle barrier)
                        let plate_y = t_rect.bottom() - 14.0;
                        t_painter.line_segment([egui::pos2(t_rect.left() + 8.0, plate_y), egui::pos2(t_rect.right() - 8.0, plate_y)], Stroke::new(3.0_f32, Color32::from_rgb(226, 232, 240)));

                        // Resting Dog Foot Footprint with clearance gap
                        let gap_px = (clearance_gap / 1.20) * 16.0;
                        let dog_foot_y = plate_y - gap_px - 2.0;

                        // Rocking Chien (dog bridge) contour (asymmetric triangular bridge)
                        let pivot_pt = egui::pos2(t_rect.left() + 30.0, plate_y - 2.0);
                        let dog_foot_pt = egui::pos2(t_rect.right() - 40.0, dog_foot_y);
                        let string_notch_pt = egui::pos2(t_rect.right() - 48.0, dog_foot_y - 22.0);

                        t_painter.line_segment([pivot_pt, dog_foot_pt], Stroke::new(2.5_f32, Color32::from_rgb(239, 68, 68)));
                        t_painter.line_segment([dog_foot_pt, string_notch_pt], Stroke::new(2.5_f32, Color32::from_rgb(245, 158, 11)));
                        t_painter.line_segment([string_notch_pt, pivot_pt], Stroke::new(2.0_f32, Color32::from_rgb(239, 68, 68)));

                        // Trompette vibrating buzzing string passing over notch
                        t_painter.line_segment([egui::pos2(t_rect.left() + 5.0, string_notch_pt.y - 2.0), egui::pos2(t_rect.right() - 5.0, string_notch_pt.y + 4.0)], Stroke::new(1.8_f32, Color32::from_rgb(251, 191, 36)));

                        // Collision Impact Sparkles if striking force is high
                        if strike_force > 2.0 {
                            t_painter.circle_filled(egui::pos2(dog_foot_pt.x, plate_y), 2.5, Color32::from_rgb(254, 240, 138));
                        }

                        // Readout
                        t_painter.text(
                            egui::pos2(t_rect.left() + 4.0, t_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Gap: {:.2} mm", clearance_gap),
                            FontId::proportional(8.5),
                            Color32::from_rgb(239, 68, 68),
                        );
                        t_painter.text(
                            egui::pos2(t_rect.right() - 4.0, t_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Force: {:.1} N", strike_force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if t_resp.hovered() {
                            let _ = t_resp.on_hover_text("Trompette Chien Buzzing Bridge Collision Gap\n[Drag horizontally: Clearance gap | Drag vertically: Coup strike force]");
                        }
                    } else if is_hoa {
                        // 5th-Order Ambisonics 2D Polar Radar Disk Visualizer
                        let (h_resp, h_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let h_rect = h_resp.rect;
                        h_painter.rect_filled(h_rect, 2.0, Color32::from_rgb(6, 10, 18));

                        let mut azimuth = state.node_param_values.get("azimuth_deg").or_else(|| state.node_param_values.get("yaw")).or_else(|| state.node_param_values.get("spherical_yaw")).copied().unwrap_or(45.0);
                        let mut elevation = state.node_param_values.get("elevation_deg").or_else(|| state.node_param_values.get("pitch")).or_else(|| state.node_param_values.get("spherical_pitch")).copied().unwrap_or(20.0);

                        if h_resp.dragged() {
                            if let Some(pos) = h_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - h_rect.left()) / h_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - h_rect.top()) / h_rect.height())).clamp(0.0, 1.0);
                                azimuth = -180.0 + norm_x * 360.0;
                                elevation = -90.0 + norm_y * 180.0;
                                state.node_param_values.insert("azimuth_deg".to_string(), azimuth);
                                state.node_param_values.insert("elevation_deg".to_string(), elevation);
                                state.node_param_values.insert("yaw".to_string(), azimuth);
                                state.node_param_values.insert("pitch".to_string(), elevation);
                                state.node_param_values.insert("spherical_yaw".to_string(), azimuth);
                                state.node_param_values.insert("spherical_pitch".to_string(), elevation);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, azimuth); }
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, elevation); }
                                }
                            }
                        }

                        // Draw Radar Circle & Concentric Rings
                        let center = h_rect.center();
                        let radius = 28.0_f32;
                        h_painter.circle_filled(center, radius, Color32::from_rgb(8, 14, 26));
                        h_painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgb(14, 165, 233)));
                        h_painter.circle_stroke(center, radius * 0.6, Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Compass crosshairs
                        h_painter.line_segment([egui::pos2(center.x - radius, center.y), egui::pos2(center.x + radius, center.y)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));
                        h_painter.line_segment([egui::pos2(center.x, center.y - radius), egui::pos2(center.x, center.y + radius)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Puck position on radar disk
                        let az_rad = azimuth.to_radians();
                        let el_rad = elevation.to_radians();
                        let r_norm = (el_rad.cos() * 0.85).clamp(0.1, 1.0);
                        let puck_pos = egui::pos2(center.x + az_rad.sin() * radius * r_norm, center.y - az_rad.cos() * radius * r_norm);

                        // Wavefront ripple ring
                        h_painter.circle_stroke(puck_pos, 7.0, Stroke::new(1.0_f32, Color32::from_rgb(34, 211, 238)));
                        h_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(244, 63, 94));

                        // Readout
                        h_painter.text(
                            egui::pos2(h_rect.left() + 4.0, h_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Az: {:+.0}°", azimuth),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );
                        h_painter.text(
                            egui::pos2(h_rect.right() - 4.0, h_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("El: {:+.0}°", elevation),
                            FontId::proportional(8.5),
                            Color32::from_rgb(34, 211, 238),
                        );

                        if h_resp.hovered() {
                            let _ = h_resp.on_hover_text("5th-Order Ambisonics (HOA5) 3D Radar Disk\n[Drag horizontally: Azimuth [-180°..+180°] | Drag vertically: Elevation [-90°..+90°]]");
                        }
                    } else if is_neural {
                        // 2D Neural Timbre Morphing Orb Mini Visualizer
                        let (n_resp, n_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let n_rect = n_resp.rect;
                        n_painter.rect_filled(n_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let mut morph_x = state.node_param_values.get("morph_x").or_else(|| state.node_param_values.get("timbre_x")).copied().unwrap_or(-0.35);
                        let mut morph_y = state.node_param_values.get("morph_y").or_else(|| state.node_param_values.get("timbre_y")).copied().unwrap_or(0.70);

                        if n_resp.dragged() {
                            if let Some(pos) = n_resp.interact_pointer_pos() {
                                let nx = ((pos.x - n_rect.left()) / n_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - n_rect.top()) / n_rect.height())).clamp(0.0, 1.0);
                                morph_x = -1.0 + nx * 2.0;
                                morph_y = -1.0 + ny * 2.0;
                                state.node_param_values.insert("morph_x".to_string(), morph_x);
                                state.node_param_values.insert("morph_y".to_string(), morph_y);
                                state.node_param_values.insert("timbre_x".to_string(), morph_x);
                                state.node_param_values.insert("timbre_y".to_string(), morph_y);
                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, morph_x); }
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, morph_y); }
                                }
                            }
                        }

                        // Background Crosshairs
                        let center = n_rect.center();
                        n_painter.line_segment([egui::pos2(n_rect.left() + 10.0, center.y), egui::pos2(n_rect.right() - 10.0, center.y)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));
                        n_painter.line_segment([egui::pos2(center.x, n_rect.top() + 6.0), egui::pos2(center.x, n_rect.bottom() - 6.0)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Corner Anchors
                        let anchors = [
                            (n_rect.left() + 18.0, n_rect.top() + 14.0, Color32::from_rgb(245, 158, 11)), // Acoustic (Amber)
                            (n_rect.right() - 18.0, n_rect.top() + 14.0, Color32::from_rgb(34, 211, 238)), // Metallic (Cyan)
                            (n_rect.right() - 18.0, n_rect.bottom() - 14.0, Color32::from_rgb(244, 63, 94)), // Cyber (Rose)
                            (n_rect.left() + 18.0, n_rect.bottom() - 14.0, Color32::from_rgb(168, 85, 247)), // Sub (Purple)
                        ];
                        for (ax, ay, col) in anchors {
                            n_painter.circle_filled(egui::pos2(ax, ay), 3.0, col);
                        }

                        // Puck Position
                        let half_w = n_rect.width() * 0.42;
                        let half_h = n_rect.height() * 0.38;
                        let puck_pos = egui::pos2(center.x + morph_x * half_w, center.y - morph_y * half_h);

                        // Lines from center to puck
                        n_painter.line_segment([center, puck_pos], Stroke::new(1.0_f32, Color32::from_rgb(244, 63, 94).gamma_multiply(0.5)));
                        n_painter.circle_stroke(puck_pos, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(244, 63, 94).gamma_multiply(0.6)));
                        n_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(244, 63, 94));
                        n_painter.circle_filled(puck_pos, 1.5, Color32::WHITE);

                        // Readout
                        n_painter.text(
                            egui::pos2(n_rect.left() + 4.0, n_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("X: {:+.2}", morph_x),
                            FontId::proportional(8.5),
                            Color32::from_rgb(244, 63, 94),
                        );
                        n_painter.text(
                            egui::pos2(n_rect.right() - 4.0, n_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Y: {:+.2}", morph_y),
                            FontId::proportional(8.5),
                            Color32::from_rgb(168, 85, 247),
                        );

                        if n_resp.hovered() {
                            let _ = n_resp.on_hover_text("2D Neural Timbre Morphing Orb\n[Drag horizontally: Latent X [-1.0..+1.0] | Drag vertically: Latent Y [-1.0..+1.0]]");
                        }
                    } else if is_bowed {
                        // Bowed String Acoustic Friction & Helmholtz Displacement Mini Visualizer
                        let (b_resp, b_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let b_rect = b_resp.rect;
                        b_painter.rect_filled(b_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let mut speed = state.node_param_values.get("bow_speed_mps").copied().unwrap_or(0.45);
                        let mut force = state.node_param_values.get("bow_force_n").copied().unwrap_or(1.25);

                        if b_resp.dragged() {
                            if let Some(pos) = b_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - b_rect.left()) / b_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - b_rect.top()) / b_rect.height())).clamp(0.0, 1.0);
                                speed = 0.01 + norm_x * (2.00 - 0.01);
                                force = 0.05 + norm_y * (5.00 - 0.05);
                                state.node_param_values.insert("bow_speed_mps".to_string(), speed);
                                state.node_param_values.insert("bow_force_n".to_string(), force);
                                if let Some(bus) = param_bus {
                                    let pid_spd = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_spd).is_some() { bus.set(pid_spd, speed); }
                                    let pid_frc = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_frc).is_some() { bus.set(pid_frc, force); }
                                }
                            }
                        }

                        // Background Schelleng limit boundary shade
                        b_painter.rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(b_rect.left() + 20.0, b_rect.top() + 10.0),
                                egui::pos2(b_rect.right() - 20.0, b_rect.bottom() - 10.0),
                            ),
                            2.0,
                            Color32::from_rgb(18, 26, 42),
                        );

                        // Bridge obstacle marker line
                        let bridge_x = b_rect.right() - 16.0;
                        b_painter.line_segment(
                            [egui::pos2(bridge_x, b_rect.top() + 6.0), egui::pos2(bridge_x, b_rect.bottom() - 6.0)],
                            Stroke::new(2.5_f32, Color32::from_rgb(245, 158, 11)),
                        );

                        // Vibrating Helmholtz string contour
                        let center_y = b_rect.center().y;
                        let steps = 24;
                        let mut prev_str_pt = None;
                        for i in 0..steps {
                            let nx = i as f32 / (steps - 1) as f32;
                            let sx = b_rect.left() + 10.0 + nx * (bridge_x - b_rect.left() - 10.0);
                            let disp = (nx * std::f32::consts::PI).sin() * 8.0 * (force / 3.0).clamp(0.2, 1.5);
                            let sy = center_y - disp;
                            let pt = egui::pos2(sx, sy);
                            if let Some(last) = prev_str_pt {
                                b_painter.line_segment([last, pt], Stroke::new(1.8_f32, Color32::from_rgb(56, 189, 248)));
                            }
                            prev_str_pt = Some(pt);
                        }

                        // Bow contact puck
                        let norm_spd = ((speed - 0.01) / 1.99).clamp(0.0, 1.0);
                        let norm_frc = ((force - 0.05) / 4.95).clamp(0.0, 1.0);
                        let puck_x = b_rect.left() + norm_spd * b_rect.width();
                        let puck_y = b_rect.bottom() - norm_frc * b_rect.height();
                        b_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(239, 68, 68));
                        b_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 202, 202)));

                        // Readout
                        b_painter.text(
                            egui::pos2(b_rect.left() + 4.0, b_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("v: {:.2} m/s", speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );
                        b_painter.text(
                            egui::pos2(b_rect.right() - 4.0, b_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Fn: {:.2} N", force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(239, 68, 68),
                        );

                        if b_resp.hovered() {
                            let _ = b_resp.on_hover_text("Bowed String Acoustic Friction & Helmholtz Waveguide\n[Drag horizontally: Bow speed [0.01..2.00 m/s] | Drag vertically: Normal bow force [0.05..5.00 N]]");
                        }
                    } else if is_shakuhachi {
                        // Shakuhachi Bamboo Flute Utaguchi Blowing Edge Mini Visualizer
                        let (s_resp, s_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let s_rect = s_resp.rect;
                        s_painter.rect_filled(s_rect, 2.0, Color32::from_rgb(8, 20, 16));

                        let mut jet_vel = state.node_param_values.get("jet_velocity_mps").copied().unwrap_or(19.0);
                        let mut utaguchi_ang = state.node_param_values.get("utaguchi_angle_deg").copied().unwrap_or(38.0);

                        if s_resp.dragged() {
                            if let Some(pos) = s_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - s_rect.left()) / s_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - s_rect.top()) / s_rect.height())).clamp(0.0, 1.0);
                                jet_vel = 4.0 + norm_x * (42.0 - 4.0);
                                utaguchi_ang = 10.0 + norm_y * (60.0 - 10.0);
                                state.node_param_values.insert("jet_velocity_mps".to_string(), jet_vel);
                                state.node_param_values.insert("utaguchi_angle_deg".to_string(), utaguchi_ang);
                                if let Some(bus) = param_bus {
                                    let pid_jet = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_jet).is_some() { bus.set(pid_jet, jet_vel); }
                                    let pid_ang = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_ang).is_some() { bus.set(pid_ang, utaguchi_ang); }
                                }
                            }
                        }

                        // Cylindrical Bamboo Flute Bore
                        let bore_top = s_rect.top() + 16.0;
                        let bore_bot = s_rect.bottom() - 16.0;
                        s_painter.line_segment([egui::pos2(s_rect.left() + 35.0, bore_top), egui::pos2(s_rect.right() - 8.0, bore_top)], Stroke::new(2.0_f32, Color32::from_rgb(180, 83, 9)));
                        s_painter.line_segment([egui::pos2(s_rect.left() + 35.0, bore_bot), egui::pos2(s_rect.right() - 8.0, bore_bot)], Stroke::new(2.0_f32, Color32::from_rgb(180, 83, 9)));

                        // Sharp Utaguchi Blowing Wedge Edge
                        let wedge_tip = egui::pos2(s_rect.left() + 35.0, bore_top + 4.0);
                        let wedge_back = egui::pos2(s_rect.left() + 48.0, bore_top - 6.0);
                        s_painter.line_segment([wedge_tip, wedge_back], Stroke::new(2.5_f32, Color32::from_rgb(52, 211, 153)));

                        // Air jet stream vortex
                        let jet_rad = utaguchi_ang.to_radians();
                        let jet_origin = egui::pos2(s_rect.left() + 12.0, bore_top - 8.0);
                        let jet_end = egui::pos2(jet_origin.x + jet_rad.cos() * 26.0, jet_origin.y + jet_rad.sin() * 26.0);
                        s_painter.line_segment([jet_origin, jet_end], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));

                        // Touch puck
                        let norm_v = ((jet_vel - 4.0) / 38.0).clamp(0.0, 1.0);
                        let norm_a = ((utaguchi_ang - 10.0) / 50.0).clamp(0.0, 1.0);
                        let puck_x = s_rect.left() + norm_v * s_rect.width();
                        let puck_y = s_rect.bottom() - norm_a * s_rect.height();
                        s_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(52, 211, 153));
                        s_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)));

                        // Readout
                        s_painter.text(
                            egui::pos2(s_rect.left() + 4.0, s_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Jet: {:.1} m/s", jet_vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );
                        s_painter.text(
                            egui::pos2(s_rect.right() - 4.0, s_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Angle: {:.0}°", utaguchi_ang),
                            FontId::proportional(8.5),
                            Color32::from_rgb(52, 211, 153),
                        );

                        if s_resp.hovered() {
                            let _ = s_resp.on_hover_text("Shakuhachi Bamboo Flute Utaguchi Chiff\n[Drag horizontally: Air jet velocity [4..42 m/s] | Drag vertically: Blowing angle [10..60°]]");
                        }
                    } else if is_sitar {
                        // Sitar Curved Jawari Bridge & Meend Deflection Mini Visualizer
                        let (sit_resp, sit_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sit_rect = sit_resp.rect;
                        sit_painter.rect_filled(sit_rect, 2.0, Color32::from_rgb(20, 16, 12));

                        let mut meend = state.node_param_values.get("meend_pull_semitones").copied().unwrap_or(1.5);
                        let mut strike_vel = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.75);
                        let jawari_gap = state.node_param_values.get("jawari_gap_mm").copied().unwrap_or(0.18);

                        if sit_resp.dragged() {
                            if let Some(pos) = sit_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - sit_rect.left()) / sit_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - sit_rect.top()) / sit_rect.height())).clamp(0.0, 1.0);
                                meend = norm_x * 5.0;
                                strike_vel = 0.05 + norm_y * 0.95;
                                state.node_param_values.insert("meend_pull_semitones".to_string(), meend);
                                state.node_param_values.insert("strike_velocity".to_string(), strike_vel);
                                if let Some(bus) = param_bus {
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, strike_vel); }
                                    let pid_meend = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_meend).is_some() { bus.set(pid_meend, meend); }
                                }
                            }
                        }

                        // Parabolic Jawari bridge boundary line
                        let obstacle_bound = -0.3 + (jawari_gap - 0.18) * 0.5;
                        let obst_y = sit_rect.bottom() - (obstacle_bound + 1.0) * 0.5 * sit_rect.height();
                        sit_painter.line_segment(
                            [egui::pos2(sit_rect.left() + 4.0, obst_y), egui::pos2(sit_rect.right() - 4.0, obst_y)],
                            Stroke::new(1.2_f32, Color32::from_rgb(239, 68, 68)),
                        );

                        // Vibrating clipped string waveform
                        let steps = 24;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let col_t = i as f32 / steps as f32;
                            let fundamental = (col_t * 6.0 * std::f32::consts::PI * (1.0 + meend * 0.15)).sin();
                            let harmonic2 = 0.5 * (col_t * 12.0 * std::f32::consts::PI).sin();
                            let raw_wave = (fundamental * 0.7 + harmonic2 * 0.3) * strike_vel;
                            let clipped_wave = raw_wave.max(obstacle_bound);
                            let norm_wave = (clipped_wave + 1.0) * 0.5;

                            let px = sit_rect.left() + 4.0 + col_t * (sit_rect.width() - 8.0);
                            let py = sit_rect.bottom() - 4.0 - norm_wave * (sit_rect.height() - 14.0);
                            let pt = egui::pos2(px, py);
                            if let Some(last) = prev_pt {
                                sit_painter.line_segment([last, pt], Stroke::new(1.8_f32, Color32::from_rgb(6, 182, 212)));
                            }
                            prev_pt = Some(pt);
                        }

                        // Touch puck
                        let norm_m = (meend / 5.0).clamp(0.0, 1.0);
                        let norm_v = ((strike_vel - 0.05) / 0.95).clamp(0.0, 1.0);
                        let puck_x = sit_rect.left() + norm_m * sit_rect.width();
                        let puck_y = sit_rect.bottom() - norm_v * sit_rect.height();
                        sit_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(245, 158, 11));
                        sit_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        sit_painter.text(
                            egui::pos2(sit_rect.left() + 4.0, sit_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Meend: +{:.1}st", meend),
                            FontId::proportional(8.5),
                            Color32::from_rgb(6, 182, 212),
                        );
                        sit_painter.text(
                            egui::pos2(sit_rect.right() - 4.0, sit_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Vel: {:.2}", strike_vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if sit_resp.hovered() {
                            let _ = sit_resp.on_hover_text("Sitar Curved Jawari Bridge & Meend Deflection\n[Drag horizontally: Meend bend [0..5 st] | Drag vertically: Mizrab strike [0.05..1.00]]");
                        }
                    } else if is_turkish_ney {
                        // Turkish Ney Reed Flute & Baspare Horn Mini Visualizer
                        let (ney_resp, ney_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let ney_rect = ney_resp.rect;
                        ney_painter.rect_filled(ney_rect, 2.0, Color32::from_rgb(10, 22, 24));

                        let mut jet_vel = state.node_param_values.get("jet_velocity_mps").copied().unwrap_or(18.5);
                        let mut embouchure_ang = state.node_param_values.get("embouchure_angle_deg").copied().unwrap_or(42.0);

                        if ney_resp.dragged() {
                            if let Some(pos) = ney_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - ney_rect.left()) / ney_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - ney_rect.top()) / ney_rect.height())).clamp(0.0, 1.0);
                                embouchure_ang = 15.0 + norm_x * 50.0;
                                jet_vel = 5.0 + norm_y * 40.0;
                                state.node_param_values.insert("embouchure_angle_deg".to_string(), embouchure_ang);
                                state.node_param_values.insert("jet_velocity_mps".to_string(), jet_vel);
                                if let Some(bus) = param_bus {
                                    let pid_jet = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_jet).is_some() { bus.set(pid_jet, jet_vel); }
                                    let pid_ang = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_ang).is_some() { bus.set(pid_ang, embouchure_ang); }
                                }
                            }
                        }

                        // Cylindrical Bamboo Flute Bore
                        let bore_top = ney_rect.top() + 16.0;
                        let bore_bot = ney_rect.bottom() - 16.0;
                        ney_painter.line_segment([egui::pos2(ney_rect.left() + 32.0, bore_top), egui::pos2(ney_rect.right() - 8.0, bore_top)], Stroke::new(1.8_f32, Color32::from_rgb(20, 184, 166)));
                        ney_painter.line_segment([egui::pos2(ney_rect.left() + 32.0, bore_bot), egui::pos2(ney_rect.right() - 8.0, bore_bot)], Stroke::new(1.8_f32, Color32::from_rgb(20, 184, 166)));

                        // Baspare Horn Mouthpiece Head
                        let horn_top = egui::pos2(ney_rect.left() + 32.0, bore_top - 4.0);
                        let horn_bot = egui::pos2(ney_rect.left() + 32.0, bore_bot + 4.0);
                        let horn_rim = egui::pos2(ney_rect.left() + 16.0, ney_rect.center().y);
                        ney_painter.line_segment([horn_top, horn_rim], Stroke::new(2.2_f32, Color32::from_rgb(251, 191, 36)));
                        ney_painter.line_segment([horn_bot, horn_rim], Stroke::new(2.2_f32, Color32::from_rgb(251, 191, 36)));

                        // Air jet stream vortex
                        let jet_rad = embouchure_ang.to_radians();
                        let jet_origin = egui::pos2(ney_rect.left() + 12.0, bore_top - 6.0);
                        let jet_end = egui::pos2(jet_origin.x + jet_rad.cos() * 24.0, jet_origin.y + jet_rad.sin() * 24.0);
                        ney_painter.line_segment([jet_origin, jet_end], Stroke::new(2.0_f32, Color32::from_rgb(45, 212, 191)));

                        // Touch puck
                        let norm_a = ((embouchure_ang - 15.0) / 50.0).clamp(0.0, 1.0);
                        let norm_v = ((jet_vel - 5.0) / 40.0).clamp(0.0, 1.0);
                        let puck_x = ney_rect.left() + norm_a * ney_rect.width();
                        let puck_y = ney_rect.bottom() - norm_v * ney_rect.height();
                        ney_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(45, 212, 191));
                        ney_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(153, 246, 228)));

                        // Readout
                        ney_painter.text(
                            egui::pos2(ney_rect.left() + 4.0, ney_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Jet: {:.1} m/s", jet_vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(45, 212, 191),
                        );
                        ney_painter.text(
                            egui::pos2(ney_rect.right() - 4.0, ney_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Angle: {:.0}°", embouchure_ang),
                            FontId::proportional(8.5),
                            Color32::from_rgb(251, 191, 36),
                        );

                        if ney_resp.hovered() {
                            let _ = ney_resp.on_hover_text("Turkish Ney Reed Flute & Baspare Horn Embouchure\n[Drag horizontally: Embouchure angle [15..65°] | Drag vertically: Airjet velocity [5..45 m/s]]");
                        }
                    } else if is_steelpan {
                        // Caribbean Steelpan Annular Ring Resonance & Modal Strike Mini Visualizer
                        let (pan_resp, pan_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let pan_rect = pan_resp.rect;
                        pan_painter.rect_filled(pan_rect, 2.0, Color32::from_rgb(12, 18, 28));

                        let mut strike_rad = state.node_param_values.get("strike_radial_pos").copied().unwrap_or(0.45);
                        let mut strike_vel = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.75);

                        if pan_resp.dragged() {
                            if let Some(pos) = pan_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - pan_rect.left()) / pan_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - pan_rect.top()) / pan_rect.height())).clamp(0.0, 1.0);
                                strike_rad = 0.05 + norm_x * 0.90;
                                strike_vel = 0.10 + norm_y * 0.90;
                                state.node_param_values.insert("strike_radial_pos".to_string(), strike_rad);
                                state.node_param_values.insert("strike_velocity".to_string(), strike_vel);
                                state.node_param_values.insert("strike_vel".to_string(), strike_vel);
                                if let Some(bus) = param_bus {
                                    let pid_rad = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_rad).is_some() { bus.set(pid_rad, strike_rad); }
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, strike_vel); }
                                }
                            }
                        }

                        // Left 45%: Concave Oil Barrel Bowl & Concentric Annular Rings
                        let bowl_center = egui::pos2(pan_rect.left() + 36.0, pan_rect.center().y);
                        let bowl_radius = 26.0;
                        pan_painter.circle_filled(bowl_center, bowl_radius, Color32::from_rgb(18, 26, 40));
                        pan_painter.circle_stroke(bowl_center, bowl_radius, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));

                        // Annular concentric rings
                        for r_idx in 1..=3 {
                            let ring_r = bowl_radius * (r_idx as f32 / 3.5);
                            pan_painter.circle_stroke(bowl_center, ring_r, Stroke::new(0.8_f32, Color32::from_rgb(38, 56, 82)));
                        }

                        // Strike puck on the annular bowl
                        let puck_dist = (strike_rad / 0.95).clamp(0.0, 1.0) * (bowl_radius - 4.0);
                        let puck_pos = egui::pos2(bowl_center.x + puck_dist * 0.707, bowl_center.y - puck_dist * 0.707);
                        pan_painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 107, 43));
                        pan_painter.circle_stroke(puck_pos, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(254, 215, 170)));

                        // Right 55%: Modal Harmonic Resonance Spectrum Bars
                        let bars_start_x = pan_rect.left() + 76.0;
                        let bars_w = pan_rect.right() - bars_start_x - 8.0;
                        let num_bars = 6;
                        let bar_slot = bars_w / num_bars as f32;
                        let fund_w = (1.0 - strike_rad * 0.7).clamp(0.2, 1.0);
                        let oct_w = (std::f32::consts::PI * strike_rad).sin().abs().clamp(0.1, 1.0);
                        let harm_weights = [
                            fund_w * 0.9,
                            oct_w * strike_vel,
                            strike_rad * strike_vel * 0.8,
                            (1.0 - strike_rad * 0.5) * 0.6,
                            strike_rad * 0.7,
                            strike_vel * 0.5,
                        ];

                        for (b_i, &w) in harm_weights.iter().enumerate() {
                            let bx = bars_start_x + b_i as f32 * bar_slot + 2.0;
                            let bar_h = (w.clamp(0.05, 1.0) * (pan_rect.height() - 24.0)).max(2.0);
                            let bar_rect = egui::Rect::from_min_size(
                                egui::pos2(bx, pan_rect.bottom() - 6.0 - bar_h),
                                egui::vec2(bar_slot - 4.0, bar_h),
                            );
                            let col = if b_i == 0 {
                                Color32::from_rgb(0, 229, 255)
                            } else if b_i == 1 {
                                Color32::from_rgb(56, 189, 248)
                            } else {
                                Color32::from_rgb(14, 165, 233)
                            };
                            pan_painter.rect_filled(bar_rect, 1.0, col);
                        }

                        // Readout
                        pan_painter.text(
                            egui::pos2(pan_rect.left() + 4.0, pan_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Rad: {:.2}R", strike_rad),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        pan_painter.text(
                            egui::pos2(pan_rect.right() - 4.0, pan_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Vel: {:.0}%", strike_vel * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if pan_resp.hovered() {
                            let _ = pan_resp.on_hover_text("Caribbean Steelpan Annular Ring Resonance\n[Drag horizontally: Radial strike position [0.05..0.95 R] | Drag vertically: Mallet velocity [10..100%]]");
                        }
                    } else if is_mbira {
                        // Lamellophone Mbira & Kalimba Tine Dispersion Mini Visualizer
                        let (mbi_resp, mbi_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mbi_rect = mbi_resp.rect;
                        mbi_painter.rect_filled(mbi_rect, 2.0, Color32::from_rgb(22, 14, 10));

                        let mut pluck_force = state.node_param_values.get("pluck_force_n").copied().unwrap_or(2.4);
                        let mut buzz_intensity = state.node_param_values.get("buzz_intensity_pct").copied().unwrap_or(0.85);

                        if mbi_resp.dragged() {
                            if let Some(pos) = mbi_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - mbi_rect.left()) / mbi_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - mbi_rect.top()) / mbi_rect.height())).clamp(0.0, 1.0);
                                pluck_force = 0.1 + norm_x * 4.9;
                                buzz_intensity = norm_y;
                                state.node_param_values.insert("pluck_force_n".to_string(), pluck_force);
                                state.node_param_values.insert("buzz_intensity_pct".to_string(), buzz_intensity);
                                state.node_param_values.insert("pluck_velocity".to_string(), (pluck_force / 5.0).clamp(0.1, 1.0));
                                state.node_param_values.insert("bottlecap_buzz".to_string(), buzz_intensity);
                                if let Some(bus) = param_bus {
                                    let pid_force = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_force).is_some() { bus.set(pid_force, pluck_force); }
                                    let pid_buzz = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_buzz).is_some() { bus.set(pid_buzz, buzz_intensity); }
                                }
                            }
                        }

                        // Wooden soundboard bridge bar
                        let bridge_y = mbi_rect.top() + 24.0;
                        mbi_painter.line_segment(
                            [egui::pos2(mbi_rect.left() + 6.0, bridge_y), egui::pos2(mbi_rect.right() - 6.0, bridge_y)],
                            Stroke::new(3.0_f32, Color32::from_rgb(146, 64, 14)),
                        );

                        // 11 staggered steel tines descending in V-formation
                        let tine_count: i32 = 11;
                        let tine_step = (mbi_rect.width() - 20.0) / (tine_count - 1) as f32;
                        let center_tine = tine_count / 2;
                        for t_idx in 0..tine_count {
                            let dist_from_center = (t_idx - center_tine).abs() as f32;
                            let tine_len = 36.0 - dist_from_center * 3.0;
                            let tx = mbi_rect.left() + 10.0 + t_idx as f32 * tine_step;
                            let tine_bot = bridge_y + tine_len;
                            let tine_col = if t_idx == center_tine {
                                Color32::from_rgb(245, 158, 11)
                            } else {
                                Color32::from_rgb(217, 119, 6)
                            };
                            mbi_painter.line_segment(
                                [egui::pos2(tx, bridge_y), egui::pos2(tx, tine_bot)],
                                Stroke::new(1.8_f32, tine_col),
                            );

                            // Buzz jingler rings / bottlecap beads rattling at the bridge
                            if buzz_intensity > 0.05 && t_idx % 2 == 0 {
                                let bead_y = bridge_y - 4.0 - (buzz_intensity * 3.0);
                                mbi_painter.circle_stroke(
                                    egui::pos2(tx, bead_y),
                                    2.0 + buzz_intensity * 1.5,
                                    Stroke::new(0.8_f32, Color32::from_rgb(255, 107, 43)),
                                );
                            }
                        }

                        // Touch puck indicating pluck force and buzz intensity
                        let norm_fx = ((pluck_force - 0.1) / 4.9).clamp(0.0, 1.0);
                        let norm_by = buzz_intensity.clamp(0.0, 1.0);
                        let puck_x = mbi_rect.left() + norm_fx * mbi_rect.width();
                        let puck_y = mbi_rect.bottom() - norm_by * mbi_rect.height();
                        mbi_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(255, 107, 43));
                        mbi_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 215, 170)));

                        // Readout
                        mbi_painter.text(
                            egui::pos2(mbi_rect.left() + 4.0, mbi_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Force: {:.1} N", pluck_force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );
                        mbi_painter.text(
                            egui::pos2(mbi_rect.right() - 4.0, mbi_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Buzz: {:.0}%", buzz_intensity * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if mbi_resp.hovered() {
                            let _ = mbi_resp.on_hover_text("Lamellophone Mbira & Kalimba Tines\n[Drag horizontally: Thumb pluck force [0.1..5.0 N] | Drag vertically: Bottlecap buzz intensity [0..100%]]");
                        }
                    } else if is_koto {
                        // Japanese 13-String Koto Paulownia Zither & Oshi-Ite Tension Mini Visualizer
                        let (koto_resp, koto_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let koto_rect = koto_resp.rect;
                        koto_painter.rect_filled(koto_rect, 2.0, Color32::from_rgb(18, 12, 8));

                        let mut oshi_ite_force = state.node_param_values.get("oshi_ite_force_n").copied().unwrap_or(2.0);
                        let mut tsume_vel = state.node_param_values.get("tsume_velocity").copied().unwrap_or(0.75);

                        if koto_resp.dragged() {
                            if let Some(pos) = koto_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - koto_rect.left()) / koto_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - koto_rect.top()) / koto_rect.height())).clamp(0.0, 1.0);
                                oshi_ite_force = norm_x * 30.0;
                                tsume_vel = 0.05 + norm_y * 0.95;
                                state.node_param_values.insert("oshi_ite_force_n".to_string(), oshi_ite_force);
                                state.node_param_values.insert("tsume_velocity".to_string(), tsume_vel);
                                state.node_param_values.insert("string_tension".to_string(), oshi_ite_force);
                                state.node_param_values.insert("pluck_vel".to_string(), tsume_vel);
                                if let Some(bus) = param_bus {
                                    let pid_tsume = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_tsume).is_some() { bus.set(pid_tsume, tsume_vel); }
                                    let pid_force = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_force).is_some() { bus.set(pid_force, oshi_ite_force); }
                                }
                            }
                        }

                        // Paulownia wooden body arch
                        let body_top = koto_rect.top() + 6.0;
                        let body_bot = koto_rect.bottom() - 6.0;
                        koto_painter.rect_stroke(
                            egui::Rect::from_min_max(egui::pos2(koto_rect.left() + 4.0, body_top), egui::pos2(koto_rect.right() - 4.0, body_bot)),
                            4.0,
                            Stroke::new(1.0_f32, Color32::from_rgb(120, 53, 15)),
                        );

                        // 13 horizontal strings with movable Ji bridge triangular wedges
                        let num_strings = 7; // simplified for compact mini preview
                        let string_step = (body_bot - body_top - 8.0) / (num_strings - 1) as f32;
                        for s_idx in 0..num_strings {
                            let sy = body_top + 4.0 + s_idx as f32 * string_step;
                            koto_painter.line_segment(
                                [egui::pos2(koto_rect.left() + 8.0, sy), egui::pos2(koto_rect.right() - 8.0, sy)],
                                Stroke::new(0.8_f32, Color32::from_rgb(217, 119, 6)),
                            );

                            // Movable Ji bridge marker (triangular ivory bridge)
                            let bridge_x = koto_rect.left() + 24.0 + (s_idx as f32 * 14.0) % (koto_rect.width() - 50.0);
                            koto_painter.line_segment(
                                [egui::pos2(bridge_x - 2.5, sy + 3.0), egui::pos2(bridge_x, sy - 3.0)],
                                Stroke::new(1.2_f32, Color32::from_rgb(254, 243, 199)),
                            );
                            koto_painter.line_segment(
                                [egui::pos2(bridge_x, sy - 3.0), egui::pos2(bridge_x + 2.5, sy + 3.0)],
                                Stroke::new(1.2_f32, Color32::from_rgb(254, 243, 199)),
                            );
                        }

                        // Touch puck indicating Oshi-Ite press force & Tsume strike velocity
                        let norm_fx = (oshi_ite_force / 30.0).clamp(0.0, 1.0);
                        let norm_vy = ((tsume_vel - 0.05) / 0.95).clamp(0.0, 1.0);
                        let puck_x = koto_rect.left() + norm_fx * koto_rect.width();
                        let puck_y = koto_rect.bottom() - norm_vy * koto_rect.height();
                        koto_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(245, 158, 11));
                        koto_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        let cents_bend = (oshi_ite_force / 30.0) * 400.0;
                        koto_painter.text(
                            egui::pos2(koto_rect.left() + 4.0, koto_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("+{:3.0}¢", cents_bend),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );
                        koto_painter.text(
                            egui::pos2(koto_rect.right() - 4.0, koto_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Tsume: {:.0}%", tsume_vel * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(217, 119, 6),
                        );

                        if koto_resp.hovered() {
                            let _ = koto_resp.on_hover_text("Japanese 13-String Koto Paulownia Zither\n[Drag horizontally: Oshi-Ite string press force [0..30 N] | Drag vertically: Tsume pluck velocity [5..100%]]");
                        }
                    } else if is_gamelan {
                        // Balinese Gamelan Gender Bronze Bar & Bamboo Resonator Mini Visualizer
                        let (gam_resp, gam_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let gam_rect = gam_resp.rect;
                        gam_painter.rect_filled(gam_rect, 2.0, Color32::from_rgb(8, 18, 14));

                        let mut hardness = state.node_param_values.get("mallet_hardness").copied().unwrap_or(0.45);
                        let mut ombak = state.node_param_values.get("ombak_rate_hz").copied().unwrap_or(7.2);

                        if gam_resp.dragged() {
                            if let Some(pos) = gam_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - gam_rect.left()) / gam_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - gam_rect.top()) / gam_rect.height())).clamp(0.0, 1.0);
                                hardness = 0.05 + norm_x * 0.95;
                                ombak = 2.0 + norm_y * 10.0;
                                state.node_param_values.insert("mallet_hardness".to_string(), hardness);
                                state.node_param_values.insert("ombak_rate_hz".to_string(), ombak);
                                state.node_param_values.insert("mallet_hard".to_string(), hardness);
                                state.node_param_values.insert("ombak_rate".to_string(), ombak);
                                if let Some(bus) = param_bus {
                                    let pid_hard = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_hard).is_some() { bus.set(pid_hard, hardness); }
                                    let pid_ombak = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_ombak).is_some() { bus.set(pid_ombak, ombak); }
                                }
                            }
                        }

                        // Suspended bronze metallophone keys
                        let num_keys: i32 = 8;
                        let key_w = (gam_rect.width() - 24.0) / num_keys as f32;
                        let bar_top = gam_rect.top() + 18.0;
                        for k_idx in 0..num_keys {
                            let kx = gam_rect.left() + 12.0 + k_idx as f32 * key_w;
                            let key_h = 16.0 + (num_keys - 1 - k_idx) as f32 * 2.2;
                            // Tuned bamboo resonator cylinder underneath
                            let tube_bot = gam_rect.bottom() - 6.0;
                            gam_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(kx + 2.0, bar_top + key_h), egui::pos2(kx + key_w - 2.0, tube_bot)),
                                1.0,
                                Color32::from_rgb(20, 83, 45),
                            );
                            // Bronze bar key
                            gam_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(kx + 1.0, bar_top), egui::pos2(kx + key_w - 1.0, bar_top + key_h)),
                                2.0,
                                Color32::from_rgb(217, 119, 6),
                            );
                            gam_painter.rect_stroke(
                                egui::Rect::from_min_max(egui::pos2(kx + 1.0, bar_top), egui::pos2(kx + key_w - 1.0, bar_top + key_h)),
                                2.0,
                                Stroke::new(0.8_f32, Color32::from_rgb(254, 240, 138)),
                            );
                        }

                        // Touch puck indicating Mallet Hardness vs Ombak Rate
                        let norm_hx = ((hardness - 0.05) / 0.95).clamp(0.0, 1.0);
                        let norm_oy = ((ombak - 2.0) / 10.0).clamp(0.0, 1.0);
                        let puck_x = gam_rect.left() + norm_hx * gam_rect.width();
                        let puck_y = gam_rect.bottom() - norm_oy * gam_rect.height();
                        gam_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(52, 211, 153));
                        gam_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)));

                        // Readout
                        gam_painter.text(
                            egui::pos2(gam_rect.left() + 4.0, gam_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Hard: {:.2}", hardness),
                            FontId::proportional(8.5),
                            Color32::from_rgb(52, 211, 153),
                        );
                        gam_painter.text(
                            egui::pos2(gam_rect.right() - 4.0, gam_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Ombak: {:.1} Hz", ombak),
                            FontId::proportional(8.5),
                            Color32::from_rgb(16, 185, 129),
                        );

                        if gam_resp.hovered() {
                            let _ = gam_resp.on_hover_text("Balinese Gamelan Metallophone & Gender\n[Drag horizontally: Mallet hardness [soft..hard] | Drag vertically: Ombak acoustic beating rate [2..12 Hz]]");
                        }
                    } else if is_dulcimer {
                        // Hammered Dulcimer / Cimbalom Multi-Bridge & Course Strike Mini Visualizer
                        let (dulc_resp, dulc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let dulc_rect = dulc_resp.rect;
                        dulc_painter.rect_filled(dulc_rect, 2.0, Color32::from_rgb(18, 12, 8));

                        let mut strike_pos = state.node_param_values.get("strike_pos_ratio").copied().unwrap_or(0.14);
                        let mut hardness = state.node_param_values.get("hammer_hardness").copied().unwrap_or(0.65);

                        if dulc_resp.dragged() {
                            if let Some(pos) = dulc_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - dulc_rect.left()) / dulc_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - dulc_rect.top()) / dulc_rect.height())).clamp(0.0, 1.0);
                                strike_pos = 0.05 + norm_x * 0.45;
                                hardness = 0.10 + norm_y * 0.90;
                                state.node_param_values.insert("strike_pos_ratio".to_string(), strike_pos);
                                state.node_param_values.insert("hammer_hardness".to_string(), hardness);
                                state.node_param_values.insert("strike_pos".to_string(), strike_pos);
                                state.node_param_values.insert("hammer_hard".to_string(), hardness);
                                if let Some(bus) = param_bus {
                                    let pid_pos = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_pos).is_some() { bus.set(pid_pos, strike_pos); }
                                    let pid_hard = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_hard).is_some() { bus.set(pid_hard, hardness); }
                                }
                            }
                        }

                        // Trapezoidal Dulcimer Soundboard outline
                        let top_w = dulc_rect.width() * 0.60;
                        let bot_w = dulc_rect.width() * 0.92;
                        let top_x0 = dulc_rect.center().x - top_w * 0.5;
                        let top_x1 = dulc_rect.center().x + top_w * 0.5;
                        let bot_x0 = dulc_rect.center().x - bot_w * 0.5;
                        let bot_x1 = dulc_rect.center().x + bot_w * 0.5;
                        let t_y = dulc_rect.top() + 6.0;
                        let b_y = dulc_rect.bottom() - 6.0;

                        dulc_painter.line_segment([egui::pos2(top_x0, t_y), egui::pos2(top_x1, t_y)], Stroke::new(1.0_f32, Color32::from_rgb(180, 83, 9)));
                        dulc_painter.line_segment([egui::pos2(top_x1, t_y), egui::pos2(bot_x1, b_y)], Stroke::new(1.0_f32, Color32::from_rgb(180, 83, 9)));
                        dulc_painter.line_segment([egui::pos2(bot_x1, b_y), egui::pos2(bot_x0, b_y)], Stroke::new(1.0_f32, Color32::from_rgb(180, 83, 9)));
                        dulc_painter.line_segment([egui::pos2(bot_x0, b_y), egui::pos2(top_x0, t_y)], Stroke::new(1.0_f32, Color32::from_rgb(180, 83, 9)));

                        // Dual bridge vertical lines (Treble & Bass)
                        let bridge_t_x = dulc_rect.left() + dulc_rect.width() * 0.40;
                        let bridge_b_x = dulc_rect.left() + dulc_rect.width() * 0.65;
                        dulc_painter.line_segment([egui::pos2(bridge_t_x, t_y + 2.0), egui::pos2(bridge_t_x, b_y - 2.0)], Stroke::new(1.2_f32, Color32::from_rgb(255, 107, 43)));
                        dulc_painter.line_segment([egui::pos2(bridge_b_x, t_y + 2.0), egui::pos2(bridge_b_x, b_y - 2.0)], Stroke::new(1.2_f32, Color32::from_rgb(255, 215, 0)));

                        // String courses
                        for c_idx in 0..5 {
                            let cy = t_y + 8.0 + c_idx as f32 * 8.5;
                            dulc_painter.line_segment([egui::pos2(bot_x0 + 4.0, cy), egui::pos2(bot_x1 - 4.0, cy)], Stroke::new(0.6_f32, Color32::from_rgb(254, 243, 199)));
                        }

                        // Draggable Puck indicating Strike Pos vs Hammer Hardness
                        let norm_px = ((strike_pos - 0.05) / 0.45).clamp(0.0, 1.0);
                        let norm_hy = ((hardness - 0.10) / 0.90).clamp(0.0, 1.0);
                        let puck_x = dulc_rect.left() + norm_px * dulc_rect.width();
                        let puck_y = dulc_rect.bottom() - norm_hy * dulc_rect.height();
                        dulc_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(255, 107, 43));
                        dulc_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(255, 237, 213)));

                        // Readout
                        dulc_painter.text(
                            egui::pos2(dulc_rect.left() + 4.0, dulc_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Pos: {:.2}L", strike_pos),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );
                        dulc_painter.text(
                            egui::pos2(dulc_rect.right() - 4.0, dulc_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Hard: {:.0}%", hardness * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if dulc_resp.hovered() {
                            let _ = dulc_resp.on_hover_text("Hammered Dulcimer & Cimbalom String Dispersion\n[Drag horizontally: Strike node position [0.05..0.50 L] | Drag vertically: Mallet hammer hardness [10..100%]]");
                        }
                    } else if is_clavinet {
                        // Electromechanical Clavinet D6 String-Anvil Collision Mini Visualizer
                        let (clav_resp, clav_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let clav_rect = clav_resp.rect;
                        clav_painter.rect_filled(clav_rect, 2.0, Color32::from_rgb(18, 14, 8));

                        let mut anvil_hard = state.node_param_values.get("anvil_hardness").copied().unwrap_or(0.85);
                        let mut phase_deg = state.node_param_values.get("pickup_phase_deg").copied().unwrap_or(0.0);

                        if clav_resp.dragged() {
                            if let Some(pos) = clav_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - clav_rect.left()) / clav_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - clav_rect.top()) / clav_rect.height())).clamp(0.0, 1.0);
                                anvil_hard = norm_x;
                                phase_deg = norm_y * 180.0;
                                state.node_param_values.insert("anvil_hardness".to_string(), anvil_hard);
                                state.node_param_values.insert("pickup_phase_deg".to_string(), phase_deg);
                                state.node_param_values.insert("anvil_hard".to_string(), anvil_hard);
                                state.node_param_values.insert("phase_deg".to_string(), phase_deg);
                                if let Some(bus) = param_bus {
                                    let pid_anvil = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_anvil).is_some() { bus.set(pid_anvil, anvil_hard); }
                                    let pid_phase = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_phase).is_some() { bus.set(pid_phase, phase_deg); }
                                }
                            }
                        }

                        // Clavinet string line
                        let str_y = clav_rect.center().y;
                        clav_painter.line_segment([egui::pos2(clav_rect.left() + 6.0, str_y), egui::pos2(clav_rect.right() - 6.0, str_y)], Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Rubber anvil tangent contact block
                        let anvil_x = clav_rect.left() + 45.0;
                        let anvil_rect = egui::Rect::from_min_max(egui::pos2(anvil_x - 8.0, str_y - 12.0), egui::pos2(anvil_x + 8.0, str_y));
                        clav_painter.rect_filled(anvil_rect, 2.0, Color32::from_rgb(249, 115, 22));
                        clav_painter.rect_stroke(anvil_rect, 2.0, Stroke::new(0.8_f32, Color32::from_rgb(254, 215, 170)));

                        // Dual magnetic pickups (Neck & Bridge)
                        let neck_x = clav_rect.left() + 85.0;
                        let bridge_x = clav_rect.left() + 125.0;
                        let p_h = 10.0;
                        clav_painter.rect_filled(egui::Rect::from_min_max(egui::pos2(neck_x - 6.0, str_y + 2.0), egui::pos2(neck_x + 6.0, str_y + 2.0 + p_h)), 1.5, Color32::from_rgb(30, 41, 59));
                        clav_painter.rect_stroke(egui::Rect::from_min_max(egui::pos2(neck_x - 6.0, str_y + 2.0), egui::pos2(neck_x + 6.0, str_y + 2.0 + p_h)), 1.5, Stroke::new(0.8_f32, Color32::from_rgb(148, 163, 184)));

                        clav_painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bridge_x - 6.0, str_y + 2.0), egui::pos2(bridge_x + 6.0, str_y + 2.0 + p_h)), 1.5, Color32::from_rgb(30, 41, 59));
                        clav_painter.rect_stroke(egui::Rect::from_min_max(egui::pos2(bridge_x - 6.0, str_y + 2.0), egui::pos2(bridge_x + 6.0, str_y + 2.0 + p_h)), 1.5, Stroke::new(0.8_f32, Color32::from_rgb(148, 163, 184)));

                        // Draggable Puck indicating Anvil Hardness vs Pickup Phase
                        let norm_ax = anvil_hard.clamp(0.0, 1.0);
                        let norm_py = (phase_deg / 180.0).clamp(0.0, 1.0);
                        let puck_x = clav_rect.left() + norm_ax * clav_rect.width();
                        let puck_y = clav_rect.bottom() - norm_py * clav_rect.height();
                        clav_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(249, 115, 22));
                        clav_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 215, 170)));

                        // Readout
                        clav_painter.text(
                            egui::pos2(clav_rect.left() + 4.0, clav_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Anvil: {:.0}%", anvil_hard * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(249, 115, 22),
                        );
                        clav_painter.text(
                            egui::pos2(clav_rect.right() - 4.0, clav_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Phase: {:.0}°", phase_deg),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 209, 102),
                        );

                        if clav_resp.hovered() {
                            let _ = clav_resp.on_hover_text("Electromechanical Clavinet D6 String-Anvil Collision\n[Drag horizontally: Anvil strike hardness [0..100%] | Drag vertically: Pickup phase angle [0..180°]]");
                        }
                    } else if is_electric_piano {
                        // Electromechanical Tine & Reed Electric Piano Mini Visualizer
                        let (ep_resp, ep_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let ep_rect = ep_resp.rect;
                        ep_painter.rect_filled(ep_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut air_gap = state.node_param_values.get("air_gap_mm").copied().unwrap_or(1.8);
                        let mut offset = state.node_param_values.get("alignment_offset_mm").copied().unwrap_or(0.45);

                        if ep_resp.dragged() {
                            if let Some(pos) = ep_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - ep_rect.left()) / ep_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - ep_rect.top()) / ep_rect.height())).clamp(0.0, 1.0);
                                air_gap = 0.5 + norm_x * 4.5;
                                offset = -2.0 + norm_y * 4.0;
                                state.node_param_values.insert("air_gap_mm".to_string(), air_gap);
                                state.node_param_values.insert("alignment_offset_mm".to_string(), offset);
                                state.node_param_values.insert("air_gap".to_string(), air_gap);
                                state.node_param_values.insert("offset".to_string(), offset);
                                if let Some(bus) = param_bus {
                                    let pid_gap = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_gap).is_some() { bus.set(pid_gap, air_gap); }
                                    let pid_off = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_off).is_some() { bus.set(pid_off, offset); }
                                }
                            }
                        }

                        // Draw Tine bar and pickup coil schematic
                        let tine_y = ep_rect.center().y;
                        ep_painter.line_segment([egui::pos2(ep_rect.left() + 6.0, tine_y), egui::pos2(ep_rect.right() - 24.0, tine_y)], Stroke::new(2.0_f32, Color32::from_rgb(148, 163, 184)));
                        // Pickup coil pole piece on right
                        ep_painter.rect_filled(
                            egui::Rect::from_min_size(egui::pos2(ep_rect.right() - 20.0, tine_y - 8.0), Vec2::new(12.0, 16.0)),
                            2.0,
                            Color32::from_rgb(251, 191, 36),
                        );

                        // Draw non-linear pickup waveform
                        let steps = 24;
                        let mut prev_pt = None;
                        for i in 0..steps {
                            let t = (i as f32 / (steps - 1) as f32) * std::f32::consts::TAU;
                            let px = ep_rect.left() + 8.0 + (i as f32 / (steps - 1) as f32) * (ep_rect.width() - 36.0);
                            let v = t.sin();
                            let sat = (v * (1.0 + (5.0 - air_gap) * 0.4)).tanh();
                            let py = tine_y - sat * (ep_rect.height() * 0.32);
                            let pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                ep_painter.line_segment([prev, pt], Stroke::new(1.4_f32, Color32::from_rgb(0, 229, 255)));
                            }
                            prev_pt = Some(pt);
                        }

                        // Draggable Puck: Air-Gap vs Offset
                        let norm_x = ((air_gap - 0.5) / 4.5).clamp(0.0, 1.0);
                        let norm_y = ((offset + 2.0) / 4.0).clamp(0.0, 1.0);
                        let puck_x = ep_rect.left() + norm_x * ep_rect.width();
                        let puck_y = ep_rect.bottom() - norm_y * ep_rect.height();
                        ep_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(251, 191, 36));
                        ep_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        ep_painter.text(
                            egui::pos2(ep_rect.left() + 4.0, ep_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Gap: {:.2}mm", air_gap),
                            FontId::proportional(8.5),
                            Color32::from_rgb(251, 191, 36),
                        );
                        ep_painter.text(
                            egui::pos2(ep_rect.right() - 4.0, ep_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Off: {:+.2}mm", offset),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if ep_resp.hovered() {
                            let _ = ep_resp.on_hover_text("Electromechanical Tine & Reed Electric Piano\n[Drag horizontally: Pickup air-gap [0.5..5.0 mm] | Drag vertically: Vertical alignment offset [-2.0..2.0 mm]]");
                        }
                    } else if is_tonewheel_organ {
                        // 9-Drawbar Tonewheel Organ Registration Mini Visualizer
                        let (organ_resp, organ_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let o_rect = organ_resp.rect;
                        organ_painter.rect_filled(o_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let col_step = (o_rect.width() - 8.0) / 9.0;
                        let bar_top = o_rect.top() + 18.0;
                        let bar_h = o_rect.height() - 24.0;

                        if organ_resp.dragged() || organ_resp.clicked() {
                            if let Some(pos) = organ_resp.interact_pointer_pos() {
                                let col_idx = (((pos.x - o_rect.left() - 4.0) / col_step).floor() as usize).min(8);
                                let norm_y = ((pos.y - bar_top) / bar_h).clamp(0.0, 1.0);
                                let val = norm_y * 8.0;
                                let param_keys = ["drawbar_16", "drawbar_5_1_3", "drawbar_8", "drawbar_4", "drawbar_2_2_3", "drawbar_2", "drawbar_1_3_5", "drawbar_1_1_3", "drawbar_1"];
                                state.node_param_values.insert(param_keys[col_idx].to_string(), val);
                                if let Some(bus) = param_bus {
                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + col_idx as u32);
                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                }
                            }
                        }

                        let drawbar_colors = [
                            Color32::from_rgb(145, 65, 30),  // 16' Brown
                            Color32::from_rgb(145, 65, 30),  // 5 1/3' Brown
                            Color32::from_rgb(235, 240, 250), // 8' White
                            Color32::from_rgb(235, 240, 250), // 4' White
                            Color32::from_rgb(50, 60, 80),    // 2 2/3' Black
                            Color32::from_rgb(235, 240, 250), // 2' White
                            Color32::from_rgb(50, 60, 80),    // 1 3/5' Black
                            Color32::from_rgb(50, 60, 80),    // 1 1/3' Black
                            Color32::from_rgb(235, 240, 250), // 1' White
                        ];

                        let param_keys = ["drawbar_16", "drawbar_5_1_3", "drawbar_8", "drawbar_4", "drawbar_2_2_3", "drawbar_2", "drawbar_1_3_5", "drawbar_1_1_3", "drawbar_1"];
                        for d in 0..9 {
                            let cx = o_rect.left() + 4.0 + (d as f32 + 0.5) * col_step;
                            let val = state.node_param_values.get(param_keys[d]).copied().unwrap_or(if d < 3 || d == 8 { 8.0 } else { 0.0 });
                            let norm = (val / 8.0).clamp(0.0, 1.0);
                            let hy = bar_top + norm * bar_h;

                            // Shaft
                            organ_painter.line_segment([egui::pos2(cx, bar_top), egui::pos2(cx, bar_top + bar_h)], Stroke::new(1.5_f32, Color32::from_rgb(30, 42, 60)));
                            // Handle
                            organ_painter.rect_filled(
                                egui::Rect::from_center_size(egui::pos2(cx, hy), Vec2::new(col_step - 2.0, 7.0)),
                                1.5,
                                drawbar_colors[d],
                            );
                        }

                        // Readout
                        organ_painter.text(
                            egui::pos2(o_rect.left() + 4.0, o_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            "9-Drawbar Registration",
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if organ_resp.hovered() {
                            let _ = organ_resp.on_hover_text("9-Drawbar Tonewheel Organ Registration\n[Click/Drag: Adjust drawbar pull level [0..8] | Authentic B3 Brown/White/Black color grouping]");
                        }
                    } else if is_piano {
                        // Concert Grand Piano 3-String Unison & Hammer Dynamics Mini Visualizer
                        let (piano_resp, piano_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let p_rect = piano_resp.rect;
                        piano_painter.rect_filled(p_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut strike_vel = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.75);
                        let mut hardness = state.node_param_values.get("hammer_hardness").copied().unwrap_or(0.65);

                        if piano_resp.dragged() {
                            if let Some(pos) = piano_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - p_rect.left()) / p_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - p_rect.top()) / p_rect.height())).clamp(0.0, 1.0);
                                hardness = 0.10 + norm_x * 0.90;
                                strike_vel = 0.05 + norm_y * 0.95;
                                state.node_param_values.insert("hammer_hardness".to_string(), hardness);
                                state.node_param_values.insert("strike_velocity".to_string(), strike_vel);
                                state.node_param_values.insert("hardness".to_string(), hardness);
                                state.node_param_values.insert("velocity".to_string(), strike_vel);
                                if let Some(bus) = param_bus {
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, strike_vel); }
                                    let pid_hard = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_hard).is_some() { bus.set(pid_hard, hardness); }
                                }
                            }
                        }

                        // Draw 3 unison strings
                        for s_idx in 0..3 {
                            let sy = p_rect.top() + 18.0 + s_idx as f32 * 6.0;
                            piano_painter.line_segment([egui::pos2(p_rect.left() + 6.0, sy), egui::pos2(p_rect.right() - 6.0, sy)], Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184)));
                        }

                        // Draw prompt/aftersound decay envelope
                        let steps = 24;
                        let mut prev_pt = None;
                        for i in 0..steps {
                            let t = i as f32 / (steps - 1) as f32;
                            let px = p_rect.left() + 8.0 + t * (p_rect.width() - 16.0);
                            let env = (-t * 4.0).exp() * 0.7 + 0.3 * (-t * 0.8).exp() * (1.0 + 0.3 * (t * 8.0 * std::f32::consts::PI).cos());
                            let py = p_rect.bottom() - 6.0 - env.clamp(0.0, 1.0) * (p_rect.height() - 28.0);
                            let pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                piano_painter.line_segment([prev, pt], Stroke::new(1.5_f32, Color32::from_rgb(68, 217, 232)));
                            }
                            prev_pt = Some(pt);
                        }

                        // Draggable Puck: Hardness vs Velocity
                        let norm_x = ((hardness - 0.10) / 0.90).clamp(0.0, 1.0);
                        let norm_y = ((strike_vel - 0.05) / 0.95).clamp(0.0, 1.0);
                        let puck_x = p_rect.left() + norm_x * p_rect.width();
                        let puck_y = p_rect.bottom() - norm_y * p_rect.height();
                        piano_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(229, 169, 60));
                        piano_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        piano_painter.text(
                            egui::pos2(p_rect.left() + 4.0, p_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Vel: {:.2}", strike_vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(229, 169, 60),
                        );
                        piano_painter.text(
                            egui::pos2(p_rect.right() - 4.0, p_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Hard: {:.2}", hardness),
                            FontId::proportional(8.5),
                            Color32::from_rgb(68, 217, 232),
                        );

                        if piano_resp.hovered() {
                            let _ = piano_resp.on_hover_text("Concert Grand Piano 3-String Unison & Felt Dynamics\n[Drag horizontally: Felt hardness [0.1..1.0] | Drag vertically: Strike velocity [0.05..1.0]]");
                        }
                    } else if is_pipe_organ {
                        // Pipe Organ Windchest Fluid Dynamics & Flue Turbulence Mini Visualizer
                        let (organ_resp, organ_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let o_rect = organ_resp.rect;
                        organ_painter.rect_filled(o_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let mut pressure = state.node_param_values.get("wind_pressure_mmh2o").copied().unwrap_or(75.0);
                        let mut cutup = state.node_param_values.get("cutup_ratio").copied().unwrap_or(0.25);

                        if organ_resp.dragged() {
                            if let Some(pos) = organ_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - o_rect.left()) / o_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - o_rect.top()) / o_rect.height())).clamp(0.0, 1.0);
                                pressure = 40.0 + norm_x * 120.0;
                                cutup = 0.15 + norm_y * 0.35;
                                state.node_param_values.insert("wind_pressure_mmh2o".to_string(), pressure);
                                state.node_param_values.insert("cutup_ratio".to_string(), cutup);
                                state.node_param_values.insert("wind_pressure".to_string(), pressure);
                                state.node_param_values.insert("cutup".to_string(), cutup);
                                if let Some(bus) = param_bus {
                                    let pid_press = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_press).is_some() { bus.set(pid_press, pressure); }
                                    let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_cut).is_some() { bus.set(pid_cut, cutup); }
                                }
                            }
                        }

                        // Air jet stream vector
                        let jet_w = (pressure / 160.0) * (o_rect.width() * 0.45);
                        let flue_y = o_rect.center().y;
                        organ_painter.rect_filled(egui::Rect::from_min_size(egui::pos2(o_rect.left() + 8.0, flue_y - 3.0), Vec2::new(12.0, 6.0)), 1.0, Color32::from_rgb(255, 180, 50));
                        organ_painter.line_segment([egui::pos2(o_rect.left() + 20.0, flue_y), egui::pos2(o_rect.left() + 20.0 + jet_w, flue_y)], Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)));

                        // 8 Harmonic Overtones Spectrum Bars
                        let bar_w = (o_rect.width() * 0.45) / 8.0;
                        let bar_x_start = o_rect.center().x + 8.0;
                        for h in 0..8 {
                            let bx = bar_x_start + h as f32 * bar_w;
                            let harm_h = ((1.0 / (h + 1) as f32).powf(0.85)) * (o_rect.height() - 24.0);
                            let col = if h == 0 { Color32::from_rgb(255, 180, 50) } else { Color32::from_rgb(0, 229, 255) };
                            organ_painter.rect_filled(egui::Rect::from_min_size(egui::pos2(bx, o_rect.bottom() - 6.0 - harm_h), Vec2::new(bar_w - 2.0, harm_h)), 1.0, col);
                        }

                        // Draggable Puck: Pressure vs Cutup
                        let norm_x = ((pressure - 40.0) / 120.0).clamp(0.0, 1.0);
                        let norm_y = ((cutup - 0.15) / 0.35).clamp(0.0, 1.0);
                        let puck_x = o_rect.left() + norm_x * o_rect.width();
                        let puck_y = o_rect.bottom() - norm_y * o_rect.height();
                        organ_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(255, 180, 50));
                        organ_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        organ_painter.text(
                            egui::pos2(o_rect.left() + 4.0, o_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("P: {:.0}mm", pressure),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 180, 50),
                        );
                        organ_painter.text(
                            egui::pos2(o_rect.right() - 4.0, o_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Cut: {:.2}", cutup),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if organ_resp.hovered() {
                            let _ = organ_resp.on_hover_text("Pipe Organ Windchest Fluid Jet Dynamics\n[Drag horizontally: Wind pressure [40..160 mmH2O] | Drag vertically: Mouth cutup ratio [0.15..0.50]]");
                        }
                    } else if is_waveguide_brass {
                        // Waveguide Brass Acoustic Lip-Reed & Bell Impedance Mini Visualizer
                        let (brass_resp, brass_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let b_rect = brass_resp.rect;
                        brass_painter.rect_filled(b_rect, 2.0, Color32::from_rgb(14, 18, 28));

                        let mut tension = state.node_param_values.get("lip_tension_hz").copied().unwrap_or(233.08);
                        let mut pressure = state.node_param_values.get("blowing_pressure_kpa").copied().unwrap_or(3.85);

                        if brass_resp.dragged() {
                            if let Some(pos) = brass_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - b_rect.left()) / b_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - b_rect.top()) / b_rect.height())).clamp(0.0, 1.0);
                                tension = 50.0 + norm_x * 1150.0;
                                pressure = 0.20 + norm_y * 7.80;
                                state.node_param_values.insert("lip_tension_hz".to_string(), tension);
                                state.node_param_values.insert("blowing_pressure_kpa".to_string(), pressure);
                                state.node_param_values.insert("lip_tension".to_string(), tension);
                                state.node_param_values.insert("blowing_pressure".to_string(), pressure);
                                if let Some(bus) = param_bus {
                                    let pid_ten = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_ten).is_some() { bus.set(pid_ten, tension); }
                                    let pid_press = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_press).is_some() { bus.set(pid_press, pressure); }
                                }
                            }
                        }

                        // Background harmonic guide lines
                        for h in 1..=4 {
                            let hx = b_rect.left() + (h as f32 / 4.5) * b_rect.width();
                            brass_painter.line_segment([egui::pos2(hx, b_rect.top() + 4.0), egui::pos2(hx, b_rect.bottom() - 4.0)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 40)));
                        }

                        // Horn flare contour on right side
                        let flare_start_x = b_rect.left() + b_rect.width() * 0.45;
                        let cy = b_rect.center().y;
                        let steps = 16;
                        let mut prev_t = None;
                        let mut prev_b = None;
                        for s in 0..=steps {
                            let frac = s as f32 / steps as f32;
                            let px = flare_start_x + frac * (b_rect.right() - flare_start_x - 6.0);
                            let r_flare = 3.0 + frac.powf(2.0) * 20.0;
                            let pt_t = egui::pos2(px, cy - r_flare);
                            let pt_b = egui::pos2(px, cy + r_flare);
                            if let (Some(pt0), Some(pb0)) = (prev_t, prev_b) {
                                brass_painter.line_segment([pt0, pt_t], Stroke::new(1.2_f32, Color32::from_rgb(255, 215, 0)));
                                brass_painter.line_segment([pb0, pt_b], Stroke::new(1.2_f32, Color32::from_rgb(255, 215, 0)));
                            }
                            prev_t = Some(pt_t);
                            prev_b = Some(pt_b);
                        }

                        // Draggable Embouchure Puck: Tension vs Pressure
                        let norm_x = ((tension - 50.0) / 1150.0).clamp(0.0, 1.0);
                        let norm_y = ((pressure - 0.20) / 7.80).clamp(0.0, 1.0);
                        let puck_x = b_rect.left() + norm_x * b_rect.width();
                        let puck_y = b_rect.bottom() - norm_y * b_rect.height();
                        brass_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(255, 215, 0));
                        brass_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)));

                        // Readout
                        brass_painter.text(
                            egui::pos2(b_rect.left() + 4.0, b_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Lip: {:.0}Hz", tension),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );
                        brass_painter.text(
                            egui::pos2(b_rect.right() - 4.0, b_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("P: {:.1}kPa", pressure),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if brass_resp.hovered() {
                            let _ = brass_resp.on_hover_text("Waveguide Brass Acoustic Lip-Reed HUD\n[Drag horizontally: Lip tension [50..1200 Hz] | Drag vertically: Blowing pressure [0.20..8.00 kPa]]");
                        }
                    } else if is_vocal_tract {
                        // Vocal Tract 44-Cylinder Area Function Mini Visualizer
                        let (vocal_resp, vocal_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let v_rect = vocal_resp.rect;
                        vocal_painter.rect_filled(v_rect, 2.0, Color32::from_rgb(10, 18, 24));

                        let mut tongue_pos = state.node_param_values.get("tongue_position").copied().unwrap_or(0.75);
                        let mut tongue_h = state.node_param_values.get("tongue_height").copied().unwrap_or(0.85);

                        if vocal_resp.dragged() {
                            if let Some(pos) = vocal_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - v_rect.left()) / v_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - v_rect.top()) / v_rect.height())).clamp(0.0, 1.0);
                                tongue_pos = norm_x;
                                tongue_h = norm_y;
                                state.node_param_values.insert("tongue_position".to_string(), tongue_pos);
                                state.node_param_values.insert("tongue_height".to_string(), tongue_h);
                                state.node_param_values.insert("tongue_pos".to_string(), tongue_pos);
                                state.node_param_values.insert("tongue_ht".to_string(), tongue_h);
                                if let Some(bus) = param_bus {
                                    let pid_pos = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_pos).is_some() { bus.set(pid_pos, tongue_pos); }
                                    let pid_h = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_h).is_some() { bus.set(pid_h, tongue_h); }
                                }
                            }
                        }

                        // Background grid
                        for g in 1..4 {
                            let gx = v_rect.left() + (g as f32 / 4.0) * v_rect.width();
                            vocal_painter.line_segment([egui::pos2(gx, v_rect.top() + 4.0), egui::pos2(gx, v_rect.bottom() - 4.0)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(52, 211, 153, 40)));
                        }

                        // 44-Cylinder acoustic tube area profile contour
                        let center_idx = 12.0 + tongue_pos * 24.0;
                        let cy = v_rect.center().y;
                        let cyl_steps = 22;
                        for i in 0..cyl_steps {
                            let frac = i as f32 / (cyl_steps - 1) as f32;
                            let px = v_rect.left() + 6.0 + frac * (v_rect.width() - 12.0);
                            let dist = (i as f32 * 2.0 - center_idx).abs();
                            let constriction = if dist < 7.0 { (1.0 - dist / 7.0).powi(2) } else { 0.0 };
                            let half_h = 4.0 + (1.0 - constriction * tongue_h * 0.75) * 16.0;
                            vocal_painter.line_segment(
                                [egui::pos2(px, cy - half_h), egui::pos2(px, cy + half_h)],
                                Stroke::new(1.5_f32, Color32::from_rgb(52, 211, 153)),
                            );
                        }

                        // Draggable Tongue Puck
                        let puck_x = v_rect.left() + tongue_pos.clamp(0.0, 1.0) * v_rect.width();
                        let puck_y = v_rect.bottom() - tongue_h.clamp(0.0, 1.0) * v_rect.height();
                        vocal_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.5, Color32::from_rgb(52, 211, 153));
                        vocal_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)));

                        // Readout
                        vocal_painter.text(
                            egui::pos2(v_rect.left() + 4.0, v_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Pos: {:.2}", tongue_pos),
                            FontId::proportional(8.5),
                            Color32::from_rgb(52, 211, 153),
                        );
                        vocal_painter.text(
                            egui::pos2(v_rect.right() - 4.0, v_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Ht: {:.2}", tongue_h),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if vocal_resp.hovered() {
                            let _ = vocal_resp.on_hover_text("Vocal Tract 44-Cylinder Area Function HUD\n[Drag horizontally: Tongue position [0..1] | Drag vertically: Tongue constriction height [0..1]]");
                        }
                    } else if is_rotary_speaker {
                        // Rotary Speaker Dual-Rotor Leslie Cabinet & Doppler Mini Visualizer
                        let (rot_resp, rot_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let r_rect = rot_resp.rect;
                        rot_painter.rect_filled(r_rect, 2.0, Color32::from_rgb(10, 14, 22));

                        let mut speed_mode = state.node_param_values.get("rotary_speed").copied().unwrap_or(2.0); // 0=Stop, 1=Chorale, 2=Tremolo, 3=Brake
                        let mut horn_rpm = state.node_param_values.get("horn_rpm").copied().unwrap_or(395.0);

                        if rot_resp.clicked() {
                            speed_mode = (speed_mode + 1.0) % 4.0;
                            horn_rpm = match speed_mode as i32 {
                                1 => 40.0,
                                2 => 400.0,
                                _ => 0.0,
                            };
                            state.node_param_values.insert("rotary_speed".to_string(), speed_mode);
                            state.node_param_values.insert("speed_state".to_string(), speed_mode);
                            state.node_param_values.insert("horn_rpm".to_string(), horn_rpm);
                            if let Some(bus) = param_bus {
                                let pid_spd = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                if bus.get(pid_spd).is_some() { bus.set(pid_spd, speed_mode); }
                                let pid_rpm = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                if bus.get(pid_rpm).is_some() { bus.set(pid_rpm, horn_rpm); }
                            }
                        }

                        let cx = r_rect.center().x;
                        let cy = r_rect.center().y;

                        // Outer cabinet circle
                        rot_painter.circle_stroke(egui::pos2(cx, cy), 24.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)));

                        // Rotating drum rotor (Blue-cyan)
                        let drum_angle = horn_rpm * 0.02;
                        let d_p1 = egui::pos2(cx + drum_angle.cos() * 20.0, cy + drum_angle.sin() * 20.0);
                        let d_p2 = egui::pos2(cx - drum_angle.cos() * 20.0, cy - drum_angle.sin() * 20.0);
                        rot_painter.line_segment([d_p1, d_p2], Stroke::new(4.0_f32, Color32::from_rgba_unmultiplied(0, 150, 255, 140)));

                        // Rotating treble horn rotor (Orange)
                        let horn_angle = -horn_rpm * 0.035;
                        let h_tip = egui::pos2(cx + horn_angle.cos() * 16.0, cy + horn_angle.sin() * 16.0);
                        let h_tail = egui::pos2(cx - horn_angle.cos() * 16.0, cy - horn_angle.sin() * 16.0);
                        rot_painter.line_segment([h_tip, h_tail], Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)));
                        rot_painter.circle_filled(h_tip, 3.5, Color32::from_rgb(255, 107, 43));
                        rot_painter.circle_filled(egui::pos2(cx, cy), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readout
                        let (speed_lbl, speed_col) = match speed_mode as i32 {
                            0 => ("STOP", Color32::from_rgb(255, 75, 75)),
                            1 => ("CHORALE", Color32::from_rgb(0, 229, 255)),
                            2 => ("TREMOLO", Color32::from_rgb(0, 255, 180)),
                            _ => ("BRAKE", Color32::from_rgb(255, 215, 0)),
                        };
                        rot_painter.text(
                            egui::pos2(r_rect.left() + 4.0, r_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Leslie: {}", speed_lbl),
                            FontId::proportional(8.5),
                            speed_col,
                        );
                        rot_painter.text(
                            egui::pos2(r_rect.right() - 4.0, r_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.0}RPM", horn_rpm),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if rot_resp.hovered() {
                            let _ = rot_resp.on_hover_text("Vintage Rotary Speaker Dual-Rotor Leslie Cabinet HUD\n[Click to cycle motor speed: STOP -> CHORALE -> TREMOLO -> BRAKE]");
                        }
                    } else if is_free_reed {
                        // Free-Reed Aeroelastic Phase Portrait & Musette Spectrum Mini Visualizer
                        let (reed_resp, reed_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let r_rect = reed_resp.rect;
                        reed_painter.rect_filled(r_rect, 2.0, Color32::from_rgb(11, 17, 32));

                        let mut stiffness = state.node_param_values.get("reed_stiffness").copied().unwrap_or(1.25);
                        let mut aperture = state.node_param_values.get("cassotto_aperture").copied().unwrap_or(0.70);

                        if reed_resp.dragged() {
                            if let Some(pos) = reed_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - r_rect.left()) / r_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - r_rect.top()) / r_rect.height())).clamp(0.0, 1.0);
                                aperture = norm_x;
                                stiffness = 0.5 + norm_y * 1.5;
                                state.node_param_values.insert("cassotto_aperture".to_string(), aperture);
                                state.node_param_values.insert("reed_stiffness".to_string(), stiffness);
                                if let Some(bus) = param_bus {
                                    let pid_stiff = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_stiff).is_some() { bus.set(pid_stiff, stiffness); }
                                    let pid_apert = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_apert).is_some() { bus.set(pid_apert, aperture); }
                                }
                            }
                        }

                        // Aeroelastic limit cycle orbit on the left
                        let orbit_cx = r_rect.left() + 38.0;
                        let orbit_cy = r_rect.center().y + 4.0;
                        let rx = 24.0_f32;
                        let ry = 16.0_f32;
                        for step in 0..16 {
                            let th1 = (step as f32 / 16.0) * std::f32::consts::TAU;
                            let th2 = ((step + 1) as f32 / 16.0) * std::f32::consts::TAU;
                            let p1 = egui::pos2(orbit_cx + th1.cos() * rx, orbit_cy + th1.sin() * ry);
                            let p2 = egui::pos2(orbit_cx + th2.cos() * rx, orbit_cy + th2.sin() * ry);
                            reed_painter.line_segment([p1, p2], Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)));
                        }

                        // 5-rank musette bars on the right
                        let bar_base_x = r_rect.left() + 82.0;
                        let bar_w = 9.0;
                        let bar_gap = 4.0;
                        let bottom_y = r_rect.bottom() - 8.0;
                        let energies = [0.85, 0.90, 0.65, 0.40, 0.70];
                        let bar_cols = [
                            Color32::from_rgb(59, 130, 246), // 16'
                            Color32::from_rgb(16, 185, 129), // 8'
                            Color32::from_rgb(245, 158, 11), // 8'+
                            Color32::from_rgb(236, 72, 153), // 8'-
                            Color32::from_rgb(139, 92, 246), // 4'
                        ];
                        for (idx, &eng) in energies.iter().enumerate() {
                            let bx = bar_base_x + idx as f32 * (bar_w + bar_gap);
                            let bh = eng * 28.0;
                            let b_rect = egui::Rect::from_min_max(egui::pos2(bx, bottom_y - bh), egui::pos2(bx + bar_w, bottom_y));
                            reed_painter.rect_filled(b_rect, 1.0, bar_cols[idx]);
                        }

                        // Interactive drag puck
                        let norm_stiff = ((stiffness - 0.5) / 1.5).clamp(0.0, 1.0);
                        let puck_x = r_rect.left() + aperture.clamp(0.0, 1.0) * r_rect.width();
                        let puck_y = r_rect.bottom() - norm_stiff * r_rect.height();
                        reed_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(16, 185, 129));
                        reed_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.0, Stroke::new(1.0_f32, Color32::from_rgb(110, 231, 183)));

                        // Readout
                        reed_painter.text(
                            egui::pos2(r_rect.left() + 4.0, r_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Reed k: {:.2}", stiffness),
                            FontId::proportional(8.5),
                            Color32::from_rgb(16, 185, 129),
                        );
                        reed_painter.text(
                            egui::pos2(r_rect.right() - 4.0, r_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Cassotto: {:.2}", aperture),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if reed_resp.hovered() {
                            let _ = reed_resp.on_hover_text("Free-Reed Aeroelastic Phase Portrait & Musette Spectrum HUD\n[Drag horizontally: Cassotto aperture [0..1] | Drag vertically: Reed stiffness [0.5..2.0]]");
                        }
                    } else if is_granular {
                        // Granular Cloud Synthesis Dispersion Mini Visualizer
                        let (gran_resp, gran_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let g_rect = gran_resp.rect;
                        gran_painter.rect_filled(g_rect, 2.0, Color32::from_rgb(8, 14, 24));

                        let mut pos_norm = state.node_param_values.get("emitter_pos").copied().unwrap_or(0.45);
                        let mut pitch_st = state.node_param_values.get("emitter_pitch").copied().unwrap_or(0.0);

                        if gran_resp.dragged() {
                            if let Some(mouse_pos) = gran_resp.interact_pointer_pos() {
                                pos_norm = ((mouse_pos.x - g_rect.left()) / g_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (mouse_pos.y - g_rect.top()) / g_rect.height()).clamp(0.0, 1.0);
                                pitch_st = -24.0 + norm_y * 48.0;

                                state.node_param_values.insert("emitter_pos".to_string(), pos_norm);
                                state.node_param_values.insert("emitter_pitch".to_string(), pitch_st);

                                if let Some(bus) = param_bus {
                                    let pid_pos = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_pch = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_pos).is_some() { bus.set(pid_pos, pos_norm); }
                                    if bus.get(pid_pch).is_some() { bus.set(pid_pch, pitch_st); }
                                }
                            }
                        }

                        // Spray ellipse
                        let cx = g_rect.left() + pos_norm * g_rect.width();
                        let cy = g_rect.bottom() - ((pitch_st + 24.0) / 48.0).clamp(0.0, 1.0) * g_rect.height();
                        gran_painter.rect_filled(
                            egui::Rect::from_center_size(egui::pos2(cx, cy), Vec2::new(36.0, 18.0)),
                            6.0,
                            Color32::from_rgba_unmultiplied(0, 229, 255, 30),
                        );

                        // Simulated grain particles
                        for i in 0..6 {
                            let gx = cx + ((i as f32 * 1.7).sin() * 14.0).clamp(-16.0, 16.0);
                            let gy = cy + ((i as f32 * 2.3).cos() * 7.0).clamp(-8.0, 8.0);
                            let col = if i % 2 == 0 { Color32::from_rgb(0, 255, 180) } else { Color32::from_rgb(255, 170, 40) };
                            gran_painter.circle_filled(egui::pos2(gx, gy), 2.0, col);
                        }

                        // Emitter puck
                        gran_painter.circle_filled(egui::pos2(cx, cy), 4.5, Color32::from_rgb(255, 255, 255));
                        gran_painter.circle_stroke(egui::pos2(cx, cy), 6.5, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));

                        // Readout
                        gran_painter.text(
                            egui::pos2(g_rect.left() + 4.0, g_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Pos: {:.0}%", pos_norm * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        gran_painter.text(
                            egui::pos2(g_rect.right() - 4.0, g_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:+.1}st", pitch_st),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 170, 40),
                        );

                        if gran_resp.hovered() {
                            let _ = gran_resp.on_hover_text("Granular Cloud Synthesis HUD\n[Drag horizontally: Emitter position [0..1] | Drag vertically: Pitch [-24..+24 st]]");
                        }
                    } else if is_spring_lattice {
                        // Spring-Mass Lattice Deformation & Dispersion Mini Visualizer
                        let (spring_resp, spring_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let s_rect = spring_resp.rect;
                        spring_painter.rect_filled(s_rect, 2.0, Color32::from_rgb(12, 10, 20));

                        let mut nonlin = state.node_param_values.get("spring_nonlinearity").copied().unwrap_or(0.45);
                        let mut drive = state.node_param_values.get("drive_force").copied().unwrap_or(0.80);

                        if spring_resp.dragged() {
                            if let Some(mouse_pos) = spring_resp.interact_pointer_pos() {
                                nonlin = ((mouse_pos.x - s_rect.left()) / s_rect.width()).clamp(0.0, 1.0);
                                drive = (1.0 - (mouse_pos.y - s_rect.top()) / s_rect.height()).clamp(0.05, 1.0);

                                state.node_param_values.insert("spring_nonlinearity".to_string(), nonlin);
                                state.node_param_values.insert("drive_force".to_string(), drive);

                                if let Some(bus) = param_bus {
                                    let pid_nl = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dr = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_nl).is_some() { bus.set(pid_nl, nonlin); }
                                    if bus.get(pid_dr).is_some() { bus.set(pid_dr, drive); }
                                }
                            }
                        }

                        // Wireframe spring grid lines (Left half)
                        let grid_x = s_rect.left() + 6.0;
                        let grid_y = s_rect.top() + 16.0;
                        let grid_w = 64.0;
                        let grid_h = 44.0;
                        let cols = 4;
                        let rows = 4;
                        for r in 0..rows {
                            let y0 = grid_y + (r as f32 / (rows - 1) as f32) * grid_h;
                            spring_painter.line_segment(
                                [egui::pos2(grid_x, y0), egui::pos2(grid_x + grid_w, y0)],
                                Stroke::new(1.0_f32, Color32::from_rgb(255, 170, 0)),
                            );
                        }
                        for c in 0..cols {
                            let x0 = grid_x + (c as f32 / (cols - 1) as f32) * grid_w;
                            spring_painter.line_segment(
                                [egui::pos2(x0, grid_y), egui::pos2(x0, grid_y + grid_h)],
                                Stroke::new(1.0_f32, Color32::from_rgb(255, 170, 0)),
                            );
                        }
                        // Driver and pickup nodes
                        spring_painter.circle_filled(egui::pos2(grid_x + grid_w * 0.33, grid_y + grid_h * 0.33), 3.0, Color32::from_rgb(0, 229, 255));
                        spring_painter.circle_filled(egui::pos2(grid_x + grid_w * 0.66, grid_y + grid_h * 0.66), 3.0, Color32::from_rgb(255, 230, 80));

                        // Duffing non-linear puck (Right half)
                        let puck_x = s_rect.left() + 85.0 + nonlin * 60.0;
                        let puck_y = s_rect.bottom() - drive * 50.0;
                        spring_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(255, 170, 0));
                        spring_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.0, Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 80)));

                        // Readout
                        spring_painter.text(
                            egui::pos2(s_rect.left() + 4.0, s_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Drive: {:.2}", drive),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 170, 0),
                        );
                        spring_painter.text(
                            egui::pos2(s_rect.right() - 4.0, s_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Duffing: {:.2}", nonlin),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if spring_resp.hovered() {
                            let _ = spring_resp.on_hover_text("Spring-Mass Lattice Deformation HUD\n[Drag horizontally: Duffing non-linearity [0..1] | Drag vertically: Drive force [0.05..1.0]]");
                        }
                    } else if is_waveguide_mesh {
                        // 2D Physical Waveguide Resonator Mesh Mini Visualizer
                        let (mesh_resp, mesh_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let m_rect = mesh_resp.rect;
                        mesh_painter.rect_filled(m_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut strike_x = state.node_param_values.get("strike_pos_x").copied().unwrap_or(0.50);
                        let mut strike_y = state.node_param_values.get("strike_pos_y").copied().unwrap_or(0.50);

                        if mesh_resp.dragged() {
                            if let Some(pos) = mesh_resp.interact_pointer_pos() {
                                strike_x = ((pos.x - m_rect.left()) / m_rect.width()).clamp(0.05, 0.95);
                                strike_y = ((pos.y - m_rect.top()) / m_rect.height()).clamp(0.05, 0.95);

                                state.node_param_values.insert("strike_pos_x".to_string(), strike_x);
                                state.node_param_values.insert("strike_pos_y".to_string(), strike_y);

                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, strike_x); }
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, strike_y); }
                                }
                            }
                        }

                        // Mini 4x4 Wireframe Mesh Projection
                        let mw = 72.0;
                        let mh = 42.0;
                        let mx0 = m_rect.left() + 8.0;
                        let my0 = m_rect.top() + 18.0;
                        let dim = 4;
                        for r in 0..dim {
                            let y = my0 + (r as f32 / (dim - 1) as f32) * mh;
                            mesh_painter.line_segment([egui::pos2(mx0, y), egui::pos2(mx0 + mw, y)], Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)));
                        }
                        for c in 0..dim {
                            let x = mx0 + (c as f32 / (dim - 1) as f32) * mw;
                            mesh_painter.line_segment([egui::pos2(x, my0), egui::pos2(x, my0 + mh)], Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)));
                        }

                        // Strike puck on mini mesh (Right half)
                        let px = m_rect.left() + 90.0 + strike_x * 55.0;
                        let py = m_rect.top() + 14.0 + strike_y * 42.0;
                        mesh_painter.circle_stroke(egui::pos2(px, py), 6.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 215, 0)));
                        mesh_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(255, 215, 0));

                        // Readout
                        mesh_painter.text(
                            egui::pos2(m_rect.left() + 4.0, m_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("X: {:.2}", strike_x),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        mesh_painter.text(
                            egui::pos2(m_rect.right() - 4.0, m_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Y: {:.2}", strike_y),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if mesh_resp.hovered() {
                            let _ = mesh_resp.on_hover_text("2D Physical Waveguide Resonator Mesh HUD\n[Drag: Surface strike coordinates (X/Y) | Modulates 2D wave propagation & boundary reflection]");
                        }
                    } else if is_plucked_string {
                        // Karplus-Strong Plucked & Struck String Mini Visualizer
                        let (pluck_resp, pluck_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let p_rect = pluck_resp.rect;
                        pluck_painter.rect_filled(p_rect, 2.0, Color32::from_rgb(14, 18, 28));

                        let mut beta = state.node_param_values.get("pluck_position_beta").copied().unwrap_or(0.15);
                        let mut vel = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.80);

                        if pluck_resp.dragged() {
                            if let Some(pos) = pluck_resp.interact_pointer_pos() {
                                beta = 0.05 + ((pos.x - p_rect.left()) / p_rect.width()).clamp(0.0, 1.0) * 0.90;
                                vel = 0.05 + (1.0 - ((pos.y - p_rect.top()) / p_rect.height())).clamp(0.0, 1.0) * 0.95;

                                state.node_param_values.insert("pluck_position_beta".to_string(), beta);
                                state.node_param_values.insert("strike_velocity".to_string(), vel);

                                if let Some(bus) = param_bus {
                                    let pid_beta = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_beta).is_some() { bus.set(pid_beta, beta); }
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, vel); }
                                }
                            }
                        }

                        // Triangular string pluck deflection line (Left half)
                        let s_y0 = p_rect.top() + 38.0;
                        let s_x0 = p_rect.left() + 8.0;
                        let s_x1 = p_rect.left() + 76.0;
                        let peak_x = s_x0 + beta * (s_x1 - s_x0);
                        let peak_y = s_y0 - vel * 18.0;

                        pluck_painter.line_segment([egui::pos2(s_x0, s_y0), egui::pos2(peak_x, peak_y)], Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)));
                        pluck_painter.line_segment([egui::pos2(peak_x, peak_y), egui::pos2(s_x1, s_y0)], Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)));
                        pluck_painter.circle_filled(egui::pos2(peak_x, peak_y), 3.5, Color32::from_rgb(255, 215, 0));

                        // Dual-polarization vibration ellipse (Right half)
                        let ec = egui::pos2(p_rect.left() + 118.0, p_rect.top() + 38.0);
                        let rx = 18.0 * (vel * 0.8 + 0.2);
                        let ry = 10.0 * (vel * 0.6 + 0.2);
                        pluck_painter.circle_stroke(ec, rx.max(ry), Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 100)));
                        pluck_painter.line_segment([egui::pos2(ec.x - rx, ec.y), egui::pos2(ec.x + rx, ec.y)], Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        pluck_painter.line_segment([egui::pos2(ec.x, ec.y - ry), egui::pos2(ec.x, ec.y + ry)], Stroke::new(1.2_f32, Color32::from_rgb(255, 170, 40)));

                        // Readout
                        pluck_painter.text(
                            egui::pos2(p_rect.left() + 4.0, p_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("β: {:.2}", beta),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );
                        pluck_painter.text(
                            egui::pos2(p_rect.right() - 4.0, p_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Vel: {:.2}", vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if pluck_resp.hovered() {
                            let _ = pluck_resp.on_hover_text("Plucked String Karplus-Strong Waveguide HUD\n[Drag horizontally: Pluck position β [0.05..0.95] | Drag vertically: Strike velocity [0.05..1.0]]");
                        }
                    } else if is_bellows {
                        // Pneumatic Bellows Pressure & Pallet Valve Velocity Mini Visualizer
                        let (b_resp, b_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let b_rect = b_resp.rect;
                        b_painter.rect_filled(b_rect, 2.0, Color32::from_rgb(12, 18, 28));

                        let mut pres = state.node_param_values.get("bellows_pressure_pa").copied().unwrap_or(420.0);
                        let mut vel = state.node_param_values.get("valve_velocity").copied().unwrap_or(0.80);

                        if b_resp.dragged() {
                            if let Some(pos) = b_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - b_rect.left()) / b_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - b_rect.top()) / b_rect.height())).clamp(0.0, 1.0);
                                pres = -1200.0 + norm_x * 2400.0;
                                vel = 0.05 + norm_y * 0.95;

                                state.node_param_values.insert("bellows_pressure_pa".to_string(), pres);
                                state.node_param_values.insert("valve_velocity".to_string(), vel);

                                if let Some(bus) = param_bus {
                                    let pid_p = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_v = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_p).is_some() { bus.set(pid_p, pres); }
                                    if bus.get(pid_v).is_some() { bus.set(pid_v, vel); }
                                }
                            }
                        }

                        // Zero-pressure dividing line
                        let mid_x = b_rect.center().x;
                        b_painter.line_segment([egui::pos2(mid_x, b_rect.top() + 14.0), egui::pos2(mid_x, b_rect.bottom() - 6.0)], Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                        // Pressure bar indicator
                        let is_push = pres >= 0.0;
                        let bar_color = if is_push { Color32::from_rgb(217, 119, 6) } else { Color32::from_rgb(56, 189, 248) };
                        let norm_p = (pres / 1200.0).clamp(-1.0, 1.0);
                        let bar_w = (norm_p.abs() * (b_rect.width() * 0.45)).max(2.0);
                        let bar_rect = if is_push {
                            egui::Rect::from_min_size(egui::pos2(mid_x, b_rect.top() + 24.0), Vec2::new(bar_w, 14.0))
                        } else {
                            egui::Rect::from_min_size(egui::pos2(mid_x - bar_w, b_rect.top() + 24.0), Vec2::new(bar_w, 14.0))
                        };
                        b_painter.rect_filled(bar_rect, 2.0, bar_color);

                        // Valve velocity vertical needle on right side
                        let vy = b_rect.bottom() - 8.0 - (vel / 1.0).clamp(0.0, 1.0) * 36.0;
                        let vx = b_rect.right() - 14.0;
                        b_painter.circle_filled(egui::pos2(vx, vy), 3.0, Color32::from_rgb(250, 204, 21));

                        // Readout
                        b_painter.text(
                            egui::pos2(b_rect.left() + 4.0, b_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("P: {:+.0} Pa", pres),
                            FontId::proportional(8.5),
                            bar_color,
                        );
                        b_painter.text(
                            egui::pos2(b_rect.right() - 4.0, b_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Vel: {:.2}", vel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(250, 204, 21),
                        );

                        if b_resp.hovered() {
                            let _ = b_resp.on_hover_text("Pneumatic Bellows Compression Dynamics HUD\n[Drag horizontally: Bellows pressure [-1200..+1200 Pa] | Drag vertically: Valve velocity [0.05..1.0]]");
                        }
                    } else if is_jawari_bridge {
                        // Sitar Curved Jawari Buzz Bridge & 13-String Tarab Mini Visualizer
                        let (j_resp, j_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let j_rect = j_resp.rect;
                        j_painter.rect_filled(j_rect, 2.0, Color32::from_rgb(18, 16, 26));

                        let mut gap = state.node_param_values.get("clearance_gap_mm").copied().unwrap_or(0.18);
                        let mut jiva = state.node_param_values.get("jiva_thread_pos").copied().unwrap_or(0.45);

                        if j_resp.dragged() {
                            if let Some(pos) = j_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - j_rect.left()) / j_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - j_rect.top()) / j_rect.height())).clamp(0.0, 1.0);
                                jiva = norm_x;
                                gap = 0.01 + norm_y * 1.49;

                                state.node_param_values.insert("clearance_gap_mm".to_string(), gap);
                                state.node_param_values.insert("jiva_thread_pos".to_string(), jiva);

                                if let Some(bus) = param_bus {
                                    let pid_g = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_j = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_g).is_some() { bus.set(pid_g, gap); }
                                    if bus.get(pid_j).is_some() { bus.set(pid_j, jiva); }
                                }
                            }
                        }

                        // Curved Jawari Bridge Profile (Left 65% area)
                        let bw = j_rect.width() * 0.65;
                        let bx0 = j_rect.left() + 4.0;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for i in 0..=16 {
                            let t = i as f32 / 16.0;
                            let x = bx0 + t * bw;
                            let offset = (t - 0.5) * (t - 0.5) * 4.0;
                            let y = j_rect.bottom() - 10.0 - (1.0 - offset * 0.35) * 24.0;
                            let pt = egui::pos2(x, y);
                            if let Some(prev) = prev_pt {
                                j_painter.line_segment([prev, pt], Stroke::new(1.5_f32, Color32::from_rgb(180, 130, 70)));
                            }
                            prev_pt = Some(pt);
                        }

                        // Jiva thread line
                        let jx = bx0 + jiva * bw;
                        j_painter.line_segment([egui::pos2(jx, j_rect.top() + 16.0), egui::pos2(jx, j_rect.bottom() - 8.0)], Stroke::new(1.2_f32, Color32::from_rgb(250, 250, 240)));

                        // Clearance gap puck
                        let gy = j_rect.bottom() - 10.0 - (gap / 1.5).clamp(0.0, 1.0) * 36.0;
                        j_painter.circle_stroke(egui::pos2(jx, gy), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(245, 158, 11)));
                        j_painter.circle_filled(egui::pos2(jx, gy), 2.5, Color32::from_rgb(255, 215, 0));

                        // Mini 13 Tarab bars (Right 30% area)
                        let tx0 = j_rect.left() + bw + 8.0;
                        let tw = j_rect.right() - tx0 - 4.0;
                        for ti in 0..13 {
                            let tx = tx0 + (ti as f32 / 12.0) * tw;
                            let th = 6.0 + (ti as f32 * 1.7).sin().abs() * 22.0;
                            j_painter.line_segment(
                                [egui::pos2(tx, j_rect.bottom() - 8.0), egui::pos2(tx, j_rect.bottom() - 8.0 - th)],
                                Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)),
                            );
                        }

                        // Readout
                        j_painter.text(
                            egui::pos2(j_rect.left() + 4.0, j_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("h₀: {:.2}mm", gap),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );
                        j_painter.text(
                            egui::pos2(j_rect.right() - 4.0, j_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Jiva: {:.2}", jiva),
                            FontId::proportional(8.5),
                            Color32::from_rgb(250, 250, 240),
                        );

                        if j_resp.hovered() {
                            let _ = j_resp.on_hover_text("Sitar Curved Jawari Bridge & Tarab HUD\n[Drag horizontally: Jiva position [0..1] | Drag vertically: Clearance gap h₀ [0.01..1.5 mm]]");
                        }
                    } else if is_soundboard {
                        // Spruce Soundboard & Bridge Mini Visualizer
                        let (sb_resp, sb_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sb_rect = sb_resp.rect;
                        sb_painter.rect_filled(sb_rect, 2.0, Color32::from_rgb(12, 22, 18));

                        let mut bleed = state.node_param_values.get("bridge_bleed").copied().unwrap_or(0.35);
                        let mut decay = state.node_param_values.get("soundboard_decay_scale").copied().unwrap_or(1.0);

                        if sb_resp.dragged() {
                            if let Some(pos) = sb_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - sb_rect.left()) / sb_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - sb_rect.top()) / sb_rect.height())).clamp(0.0, 1.0);
                                bleed = 0.05 + norm_x * 0.90;
                                decay = 0.2 + norm_y * 2.8;

                                state.node_param_values.insert("bridge_bleed".to_string(), bleed);
                                state.node_param_values.insert("soundboard_decay_scale".to_string(), decay);

                                if let Some(bus) = param_bus {
                                    let pid_b = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_d = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_b).is_some() { bus.set(pid_b, bleed); }
                                    if bus.get(pid_d).is_some() { bus.set(pid_d, decay); }
                                }
                            }
                        }

                        // 2D Spruce plate standing wave simulation cells
                        let grid_cols = 8;
                        let grid_rows = 4;
                        let cell_w = (sb_rect.width() - 8.0) / grid_cols as f32;
                        let cell_h = (sb_rect.height() - 8.0) / grid_rows as f32;
                        for gy in 0..grid_rows {
                            let ny = (gy as f32 + 0.5) / grid_rows as f32;
                            for gx in 0..grid_cols {
                                let nx = (gx as f32 + 0.5) / grid_cols as f32;
                                let m11 = (nx * std::f32::consts::PI).sin() * (ny * std::f32::consts::PI).sin();
                                let m12 = (nx * std::f32::consts::PI).sin() * (2.0 * ny * std::f32::consts::PI).sin();
                                let disp = (m11 * 0.6 + m12 * 0.4).abs() * (decay / 1.5).clamp(0.4, 2.0);
                                let alpha = (disp * 180.0 * (bleed / 0.5)).clamp(15.0, 220.0) as u8;
                                let col = Color32::from_rgba_premultiplied(16, 185, 129, alpha);
                                let c_rect = egui::Rect::from_min_size(
                                    egui::pos2(sb_rect.left() + 4.0 + gx as f32 * cell_w, sb_rect.top() + 4.0 + gy as f32 * cell_h),
                                    Vec2::new(cell_w - 1.0, cell_h - 1.0),
                                );
                                sb_painter.rect_filled(c_rect, 1.0, col);
                            }
                        }

                        // Maple bridge line
                        let by = sb_rect.top() + 4.0 + (1.0 - (bleed - 0.05) / 0.90) * (sb_rect.height() - 8.0);
                        sb_painter.line_segment(
                            [egui::pos2(sb_rect.left() + 4.0, by), egui::pos2(sb_rect.right() - 4.0, by)],
                            Stroke::new(1.5_f32, Color32::from_rgb(245, 158, 11)),
                        );

                        // Draggable puck
                        let px = sb_rect.left() + 4.0 + ((bleed - 0.05) / 0.90).clamp(0.0, 1.0) * (sb_rect.width() - 8.0);
                        let py = sb_rect.top() + 4.0 + (1.0 - ((decay - 0.2) / 2.8).clamp(0.0, 1.0)) * (sb_rect.height() - 8.0);
                        sb_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(16, 185, 129)));
                        sb_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readout
                        sb_painter.text(
                            egui::pos2(sb_rect.left() + 4.0, sb_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.0}% Bleed", bleed * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(16, 185, 129),
                        );
                        sb_painter.text(
                            egui::pos2(sb_rect.right() - 4.0, sb_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.2}x T60", decay),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if sb_resp.hovered() {
                            let _ = sb_resp.on_hover_text(format!("Spruce Soundboard & Bridge HUD\nBridge Bleed: {:.1}% | Decay Scale: {:.2}x\n[Drag horizontally: Bleed | Drag vertically: Decay scale]", bleed * 100.0, decay));
                        }
                    } else if is_sympathetic {
                        // Sympathetic String Matrix Energy Coupling Mini Visualizer
                        let (sym_resp, sym_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sym_rect = sym_resp.rect;
                        sym_painter.rect_filled(sym_rect, 2.0, Color32::from_rgb(14, 20, 32));

                        let mut coupling = state.node_param_values.get("coupling_strength").copied().unwrap_or(0.15);
                        let mut loss = state.node_param_values.get("bridge_loss").copied().unwrap_or(0.995);

                        if sym_resp.dragged() {
                            if let Some(pos) = sym_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - sym_rect.left()) / sym_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - sym_rect.top()) / sym_rect.height())).clamp(0.0, 1.0);
                                coupling = norm_x * 0.50;
                                loss = 0.90 + norm_y * 0.099;

                                state.node_param_values.insert("coupling_strength".to_string(), coupling);
                                state.node_param_values.insert("bridge_loss".to_string(), loss);

                                if let Some(bus) = param_bus {
                                    let pid_c = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_l = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_c).is_some() { bus.set(pid_c, coupling); }
                                    if bus.get(pid_l).is_some() { bus.set(pid_l, loss); }
                                }
                            }
                        }

                        // 6-String sympathetic energy bars
                        let num_bars = 6;
                        let bar_h = (sym_rect.height() - 8.0) / num_bars as f32;
                        for i in 0..num_bars {
                            let by = sym_rect.top() + 4.0 + i as f32 * bar_h;
                            let frac = if i == 0 {
                                0.85
                            } else {
                                (0.85 * coupling * 2.0 / (1.0 + i as f32 * 0.6)).clamp(0.05, 0.95)
                            };
                            let bar_w = (sym_rect.width() * 0.55 * frac).clamp(2.0, sym_rect.width() * 0.55);
                            let b_color = if i == 0 {
                                Color32::from_rgb(0, 229, 255)
                            } else {
                                Color32::from_rgb(255, 215, 0)
                            };
                            sym_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(sym_rect.left() + 4.0, by + 1.0), Vec2::new(bar_w, bar_h - 2.0)),
                                1.0,
                                b_color,
                            );
                        }

                        // Right side: coupling matrix puck
                        let matrix_x0 = sym_rect.left() + sym_rect.width() * 0.62;
                        let matrix_w = sym_rect.width() * 0.34;
                        let px = matrix_x0 + (coupling / 0.50).clamp(0.0, 1.0) * matrix_w;
                        let py = sym_rect.bottom() - 6.0 - ((loss - 0.90) / 0.099).clamp(0.0, 1.0) * (sym_rect.height() - 12.0);

                        sym_painter.rect_stroke(
                            egui::Rect::from_min_size(egui::pos2(matrix_x0, sym_rect.top() + 4.0), Vec2::new(matrix_w, sym_rect.height() - 8.0)),
                            2.0,
                            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
                        );
                        sym_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        sym_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readout
                        sym_painter.text(
                            egui::pos2(sym_rect.left() + 4.0, sym_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("C: {:.1}%", coupling * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        sym_painter.text(
                            egui::pos2(sym_rect.right() - 4.0, sym_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("L: {:.3}", loss),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if sym_resp.hovered() {
                            let _ = sym_resp.on_hover_text(format!("Sympathetic Resonance Coupling HUD\nCoupling Bleed: {:.1}% | Bridge Loss: {:.4}\n[Drag horizontally: Coupling bleed | Drag vertically: Loss factor]", coupling * 100.0, loss));
                        }
                    } else if is_woodwind_jet {
                        // Woodwind Air-Jet Embouchure Mini Visualizer
                        let (jet_resp, jet_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let jet_rect = jet_resp.rect;
                        jet_painter.rect_filled(jet_rect, 2.0, Color32::from_rgb(10, 20, 32));

                        let mut pressure = state.node_param_values.get("jet_pressure_kpa").copied().unwrap_or(1.25);
                        let mut offset = state.node_param_values.get("jet_offset_mm").copied().unwrap_or(7.0);

                        if jet_resp.dragged() {
                            if let Some(pos) = jet_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - jet_rect.left()) / jet_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - jet_rect.top()) / jet_rect.height())).clamp(0.0, 1.0);
                                pressure = 0.10 + norm_x * 3.90;
                                offset = 2.0 + norm_y * 13.0;

                                let p_pa = pressure * 1000.0;
                                let vel = (2.0 * p_pa / 1.204).sqrt().clamp(5.0, 120.0);
                                let d_m = offset * 1e-3;
                                let v_profile = (0.4 * vel).max(1.0);
                                let delay = (d_m / v_profile) * 1000.0;
                                let score = (1.0 / (1.0 + (delay * 0.1).powi(2))).clamp(0.1, 1.0);

                                state.node_param_values.insert("jet_pressure_kpa".to_string(), pressure);
                                state.node_param_values.insert("jet_offset_mm".to_string(), offset);
                                state.node_param_values.insert("jet_velocity_ms".to_string(), vel);
                                state.node_param_values.insert("acoustic_coupling_score".to_string(), score);

                                if let Some(bus) = param_bus {
                                    let pid_p = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_o = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_v = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_s = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_p).is_some() { bus.set(pid_p, pressure); }
                                    if bus.get(pid_o).is_some() { bus.set(pid_o, offset); }
                                    if bus.get(pid_v).is_some() { bus.set(pid_v, vel); }
                                    if bus.get(pid_s).is_some() { bus.set(pid_s, score); }
                                }
                            }
                        }

                        // Air-Jet stream curve and labium splitting wedge
                        let lip_x = jet_rect.left() + 10.0;
                        let lip_y = jet_rect.top() + jet_rect.height() * 0.45;
                        let wedge_x = jet_rect.right() - 20.0;
                        let wedge_y = jet_rect.top() + jet_rect.height() * 0.45;

                        // Labium splitting wedge triangle
                        jet_painter.line_segment(
                            [egui::pos2(wedge_x, wedge_y - 12.0), egui::pos2(wedge_x, wedge_y + 12.0)],
                            Stroke::new(2.0_f32, Color32::from_rgb(255, 107, 43)),
                        );
                        jet_painter.line_segment(
                            [egui::pos2(wedge_x, wedge_y), egui::pos2(jet_rect.right() - 6.0, wedge_y - 8.0)],
                            Stroke::new(1.5_f32, Color32::from_rgb(255, 107, 43)),
                        );

                        // Modulated air jet streamline
                        let steps = 16;
                        for s in 0..steps {
                            let frac1 = s as f32 / steps as f32;
                            let frac2 = (s + 1) as f32 / steps as f32;
                            let x1 = lip_x + frac1 * (wedge_x - lip_x);
                            let x2 = lip_x + frac2 * (wedge_x - lip_x);
                            let wave1 = (frac1 * std::f32::consts::PI * 3.0).sin() * (offset / 15.0) * 8.0;
                            let wave2 = (frac2 * std::f32::consts::PI * 3.0).sin() * (offset / 15.0) * 8.0;
                            jet_painter.line_segment(
                                [egui::pos2(x1, lip_y + wave1), egui::pos2(x2, lip_y + wave2)],
                                Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
                            );
                        }

                        // Draggable puck
                        let px = jet_rect.left() + 4.0 + ((pressure - 0.10) / 3.90).clamp(0.0, 1.0) * (jet_rect.width() - 8.0);
                        let py = jet_rect.top() + 4.0 + (1.0 - ((offset - 2.0) / 13.0).clamp(0.0, 1.0)) * (jet_rect.height() - 8.0);
                        jet_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        jet_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readout
                        jet_painter.text(
                            egui::pos2(jet_rect.left() + 4.0, jet_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.2} kPa", pressure),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        jet_painter.text(
                            egui::pos2(jet_rect.right() - 4.0, jet_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.1} mm", offset),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if jet_resp.hovered() {
                            let _ = jet_resp.on_hover_text(format!("Woodwind Air-Jet Embouchure HUD\nPressure: {:.2} kPa | Offset: {:.1} mm\n[Drag horizontally: Blow pressure | Drag vertically: Jet offset]", pressure, offset));
                        }
                    } else if is_tonehole_matrix {
                        // Woodwind 6-Tonehole Radiation Mini Visualizer
                        let (th_resp, th_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let th_rect = th_resp.rect;
                        th_painter.rect_filled(th_rect, 2.0, Color32::from_rgb(18, 14, 28));

                        let mut bore_len = state.node_param_values.get("bore_length_m").copied().unwrap_or(0.60);
                        let mut cutoff = state.node_param_values.get("lattice_cutoff_hz").copied().unwrap_or(2200.0);

                        if th_resp.dragged() {
                            if let Some(pos) = th_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - th_rect.left()) / th_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - th_rect.top()) / th_rect.height())).clamp(0.0, 1.0);
                                bore_len = 0.20 + norm_x * 1.00;
                                cutoff = 500.0 + norm_y * 5500.0;

                                let fund = (343.2 / (2.0 * bore_len)).clamp(40.0, 4000.0);
                                let power = 14.5 + (fund / 100.0);

                                state.node_param_values.insert("bore_length_m".to_string(), bore_len);
                                state.node_param_values.insert("lattice_cutoff_hz".to_string(), cutoff);
                                state.node_param_values.insert("fundamental_hz".to_string(), fund);
                                state.node_param_values.insert("radiated_power_mw".to_string(), power);

                                if let Some(bus) = param_bus {
                                    let pid_l = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_c = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_f = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_p = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_l).is_some() { bus.set(pid_l, bore_len); }
                                    if bus.get(pid_c).is_some() { bus.set(pid_c, cutoff); }
                                    if bus.get(pid_f).is_some() { bus.set(pid_f, fund); }
                                    if bus.get(pid_p).is_some() { bus.set(pid_p, power); }
                                }
                            }
                        }

                        // Cylindrical bore tube contour & standing wave
                        let bore_y = th_rect.top() + th_rect.height() * 0.50;
                        th_painter.line_segment(
                            [egui::pos2(th_rect.left() + 6.0, bore_y - 10.0), egui::pos2(th_rect.right() - 6.0, bore_y - 10.0)],
                            Stroke::new(1.0_f32, Color32::from_rgb(60, 45, 80)),
                        );
                        th_painter.line_segment(
                            [egui::pos2(th_rect.left() + 6.0, bore_y + 10.0), egui::pos2(th_rect.right() - 6.0, bore_y + 10.0)],
                            Stroke::new(1.0_f32, Color32::from_rgb(60, 45, 80)),
                        );

                        // 6 Tonehole indicator dots
                        let num_holes = 6;
                        let hole_spacing = (th_rect.width() - 32.0) / (num_holes - 1) as f32;
                        for h in 0..num_holes {
                            let hx = th_rect.left() + 16.0 + h as f32 * hole_spacing;
                            let h_col = if h < 3 { Color32::from_rgb(0, 229, 255) } else { Color32::from_rgb(255, 107, 43) };
                            th_painter.circle_filled(egui::pos2(hx, bore_y - 10.0), 3.0, h_col);
                            th_painter.circle_filled(egui::pos2(hx, bore_y + 10.0), 3.0, h_col);
                        }

                        // Standing wave sinusoidal profile
                        let wave_steps = 20;
                        for s in 0..wave_steps {
                            let f1 = s as f32 / wave_steps as f32;
                            let f2 = (s + 1) as f32 / wave_steps as f32;
                            let x1 = th_rect.left() + 8.0 + f1 * (th_rect.width() - 16.0);
                            let x2 = th_rect.left() + 8.0 + f2 * (th_rect.width() - 16.0);
                            let y1 = bore_y - (f1 * std::f32::consts::PI).sin() * 8.0;
                            let y2 = bore_y - (f2 * std::f32::consts::PI).sin() * 8.0;
                            th_painter.line_segment(
                                [egui::pos2(x1, y1), egui::pos2(x2, y2)],
                                Stroke::new(1.5_f32, Color32::from_rgb(255, 107, 43)),
                            );
                        }

                        // Draggable puck
                        let px = th_rect.left() + 4.0 + ((bore_len - 0.20) / 1.00).clamp(0.0, 1.0) * (th_rect.width() - 8.0);
                        let py = th_rect.top() + 4.0 + (1.0 - ((cutoff - 500.0) / 5500.0).clamp(0.0, 1.0)) * (th_rect.height() - 8.0);
                        th_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(255, 107, 43)));
                        th_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readout
                        th_painter.text(
                            egui::pos2(th_rect.left() + 4.0, th_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.2} m", bore_len),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );
                        th_painter.text(
                            egui::pos2(th_rect.right() - 4.0, th_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.0} Hz", cutoff),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if th_resp.hovered() {
                            let _ = th_resp.on_hover_text(format!("Woodwind 6-Tonehole Radiation HUD\nBore Length: {:.2} m | Cutoff: {:.0} Hz\n[Drag horizontally: Bore length | Drag vertically: Radiation cutoff]", bore_len, cutoff));
                        }
                    } else if is_plate_dispersion {
                        // Plate Dispersion & APDN Reverb Mini Visualizer
                        let (pl_resp, pl_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let pl_rect = pl_resp.rect;
                        pl_painter.rect_filled(pl_rect, 2.0, Color32::from_rgb(14, 20, 32));

                        let mut disp = state.node_param_values.get("dispersion_factor").copied().unwrap_or(0.65);
                        let mut damper = state.node_param_values.get("damper_position").copied().unwrap_or(0.30);

                        if pl_resp.dragged() {
                            if let Some(pos) = pl_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - pl_rect.left()) / pl_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - pl_rect.top()) / pl_rect.height())).clamp(0.0, 1.0);
                                disp = norm_x;
                                damper = (1.0 - norm_y).clamp(0.0, 1.0);
                                let t60 = (3.5 * (1.0 - 0.85 * damper)).clamp(0.4, 8.0);
                                let high_damp = 0.45;

                                state.node_param_values.insert("dispersion_factor".to_string(), disp);
                                state.node_param_values.insert("damper_position".to_string(), damper);
                                state.node_param_values.insert("decay_t60_sec".to_string(), t60);
                                state.node_param_values.insert("high_damping".to_string(), high_damp);

                                if let Some(bus) = param_bus {
                                    let pid_d = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_p = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_t = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_h = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_d).is_some() { bus.set(pid_d, disp); }
                                    if bus.get(pid_p).is_some() { bus.set(pid_p, damper); }
                                    if bus.get(pid_t).is_some() { bus.set(pid_t, t60); }
                                    if bus.get(pid_h).is_some() { bus.set(pid_h, high_damp); }
                                }
                            }
                        }

                        // Flexural dispersion curve v_p ~ f^(0.25 + 0.35 * disp)
                        let curve_steps = 16;
                        for s in 0..curve_steps {
                            let f1 = (s + 1) as f32 / (curve_steps + 1) as f32;
                            let f2 = (s + 2) as f32 / (curve_steps + 1) as f32;
                            let exp = 0.25 + 0.35 * disp;
                            let v1 = f1.powf(exp).clamp(0.05, 1.0);
                            let v2 = f2.powf(exp).clamp(0.05, 1.0);

                            let x1 = pl_rect.left() + 8.0 + (s as f32 / curve_steps as f32) * (pl_rect.width() * 0.50);
                            let x2 = pl_rect.left() + 8.0 + ((s + 1) as f32 / curve_steps as f32) * (pl_rect.width() * 0.50);
                            let y1 = pl_rect.bottom() - 10.0 - v1 * 36.0;
                            let y2 = pl_rect.bottom() - 10.0 - v2 * 36.0;
                            pl_painter.line_segment([egui::pos2(x1, y1), egui::pos2(x2, y2)], Stroke::new(1.8_f32, Color32::from_rgb(255, 170, 0)));
                        }

                        // Damper absorption pad line (right side)
                        let pad_x = pl_rect.left() + pl_rect.width() * 0.65;
                        let pad_w = pl_rect.width() * 0.30;
                        let pad_h = 40.0;
                        let pad_y = pl_rect.top() + 14.0;
                        pl_painter.rect_stroke(
                            egui::Rect::from_min_size(egui::pos2(pad_x, pad_y), egui::vec2(pad_w, pad_h)),
                            2.0_f32,
                            Stroke::new(1.0_f32, Color32::from_rgb(60, 75, 100)),
                        );

                        // Draggable puck
                        let px = pad_x + disp * pad_w;
                        let py = pad_y + damper * pad_h;
                        pl_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        pl_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Readouts
                        let t60_val = (3.5 * (1.0 - 0.85 * damper)).clamp(0.4, 8.0);
                        pl_painter.text(
                            egui::pos2(pl_rect.left() + 4.0, pl_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.0}% disp", disp * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 170, 0),
                        );
                        pl_painter.text(
                            egui::pos2(pl_rect.right() - 4.0, pl_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.1}s T60", t60_val),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if pl_resp.hovered() {
                            let _ = pl_resp.on_hover_text(format!("Plate Dispersion & APDN Reverb HUD\nDispersion: {:.1}% | Damper Pad: {:.1}%\n[Drag horizontally: Dispersion factor | Drag vertically: Damper position]", disp * 100.0, damper * 100.0));
                        }
                    } else if is_membrane_cavity {
                        // Membrane Cavity Phase & Drum Displacement Mini Visualizer
                        let (mem_resp, mem_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mem_rect = mem_resp.rect;
                        mem_painter.rect_filled(mem_rect, 2.0, Color32::from_rgb(24, 12, 18));

                        let mut strike_r = state.node_param_values.get("radial_strike_pos").copied().unwrap_or(0.70);
                        let mut strike_v = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.80);

                        if mem_resp.dragged() {
                            if let Some(pos) = mem_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - mem_rect.left()) / mem_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - mem_rect.top()) / mem_rect.height())).clamp(0.0, 1.0);
                                strike_r = norm_x;
                                strike_v = 0.05 + norm_y * 0.95;
                                let air = 0.85;
                                let rim = 0.05;

                                state.node_param_values.insert("radial_strike_pos".to_string(), strike_r);
                                state.node_param_values.insert("strike_velocity".to_string(), strike_v);
                                state.node_param_values.insert("air_cavity_depth".to_string(), air);
                                state.node_param_values.insert("rimshot_damping".to_string(), rim);

                                if let Some(bus) = param_bus {
                                    let pid_r = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_v = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_a = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_m = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_r).is_some() { bus.set(pid_r, strike_r); }
                                    if bus.get(pid_v).is_some() { bus.set(pid_v, strike_v); }
                                    if bus.get(pid_a).is_some() { bus.set(pid_a, air); }
                                    if bus.get(pid_m).is_some() { bus.set(pid_m, rim); }
                                }
                            }
                        }

                        // Circular drumhead rim hoop & concentric Bessel nodal rings
                        let cx = mem_rect.left() + 38.0;
                        let cy = mem_rect.top() + 34.0;
                        let drum_radius = 24.0;
                        mem_painter.circle_stroke(egui::pos2(cx, cy), drum_radius, Stroke::new(2.5_f32, Color32::from_rgb(255, 170, 0)));
                        mem_painter.circle_stroke(egui::pos2(cx, cy), drum_radius * 0.50, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 70, 90, 100)));

                        // Radial strike puck
                        let puck_dist = drum_radius * strike_r.clamp(0.0, 1.0);
                        let px = cx + puck_dist;
                        let py = cy;
                        mem_painter.circle_stroke(egui::pos2(px, py), 5.0, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        mem_painter.circle_filled(egui::pos2(px, py), 2.5, Color32::from_rgb(255, 255, 255));

                        // Strike velocity & air cavity bar (right side)
                        let bar_x = mem_rect.left() + 82.0;
                        let bar_w = 66.0;
                        let bar_h = 10.0;
                        let bar_y1 = mem_rect.top() + 20.0;
                        let bar_y2 = mem_rect.top() + 38.0;
                        mem_painter.rect_filled(
                            egui::Rect::from_min_size(egui::pos2(bar_x, bar_y1), egui::vec2(bar_w * strike_v, bar_h)),
                            2.0_f32,
                            Color32::from_rgb(255, 107, 43),
                        );
                        mem_painter.rect_stroke(
                            egui::Rect::from_min_size(egui::pos2(bar_x, bar_y1), egui::vec2(bar_w, bar_h)),
                            2.0_f32,
                            Stroke::new(1.0_f32, Color32::from_rgb(80, 50, 60)),
                        );

                        // Readouts
                        mem_painter.text(
                            egui::pos2(bar_x, bar_y2),
                            egui::Align2::LEFT_TOP,
                            format!("r: {:.2}  vel: {:.2}", strike_r, strike_v),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 190, 120),
                        );

                        if mem_resp.hovered() {
                            let _ = mem_resp.on_hover_text(format!("Membrane Cavity Phase & Drum Displacement HUD\nRadial Strike: {:.1}% | Velocity: {:.2}\n[Drag horizontally: Radial strike position | Drag vertically: Strike velocity]", strike_r * 100.0, strike_v));
                        }
                    } else if is_embouchure_angle {
                        // Shakuhachi Embouchure & Meri/Kari Radiation Mini Visualizer
                        let (emb_resp, emb_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let emb_rect = emb_resp.rect;
                        emb_painter.rect_filled(emb_rect, 2.0, Color32::from_rgb(10, 18, 26));

                        let mut angle = state.node_param_values.get("embouchure_angle_deg").copied().unwrap_or(38.0);
                        let mut press = state.node_param_values.get("blowing_pressure_pa").copied().unwrap_or(850.0);

                        if emb_resp.dragged() {
                            if let Some(pos) = emb_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - emb_rect.left()) / emb_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - emb_rect.top()) / emb_rect.height())).clamp(0.0, 1.0);
                                angle = 10.0 + norm_x * 50.0;
                                press = 100.0 + norm_y * 2900.0;
                                let jet_v = (2.0 * press / 1.204).sqrt().clamp(10.0, 80.0);
                                let meri = (state.node_param_values.get("meri_kari_cents").copied().unwrap_or(0.0)).clamp(-300.0, 200.0);

                                state.node_param_values.insert("embouchure_angle_deg".to_string(), angle);
                                state.node_param_values.insert("blowing_pressure_pa".to_string(), press);
                                state.node_param_values.insert("jet_velocity_mps".to_string(), jet_v);
                                state.node_param_values.insert("meri_kari_cents".to_string(), meri);

                                if let Some(bus) = param_bus {
                                    let pid_ang = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_prs = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_mer = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_ang).is_some() { bus.set(pid_ang, angle); }
                                    if bus.get(pid_prs).is_some() { bus.set(pid_prs, press); }
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, jet_v); }
                                    if bus.get(pid_mer).is_some() { bus.set(pid_mer, meri); }
                                }
                            }
                        }

                        // Utaguchi splitting wedge & jet stream
                        let wedge_x = emb_rect.left() + 48.0;
                        let wedge_y = emb_rect.top() + 34.0;
                        let rad = angle.to_radians();
                        let wx = wedge_x - 16.0 * rad.sin();
                        let wy = wedge_y + 16.0 * rad.cos();
                        emb_painter.line_segment([egui::pos2(wedge_x, wedge_y), egui::pos2(wx, wy)], Stroke::new(2.5_f32, Color32::from_rgb(255, 180, 50)));
                        emb_painter.line_segment([egui::pos2(emb_rect.left() + 10.0, wedge_y - 2.0), egui::pos2(wedge_x, wedge_y)], Stroke::new(1.8_f32, Color32::from_rgb(0, 229, 255)));

                        // Puck
                        let norm_x = ((angle - 10.0) / 50.0).clamp(0.0, 1.0);
                        let norm_y = ((press - 100.0) / 2900.0).clamp(0.0, 1.0);
                        let px = emb_rect.left() + 12.0 + norm_x * 70.0;
                        let py = emb_rect.bottom() - 10.0 - norm_y * 45.0;
                        emb_painter.circle_stroke(egui::pos2(px, py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        emb_painter.circle_filled(egui::pos2(px, py), 2.0, Color32::from_rgb(255, 255, 255));

                        // Harmonic overtone mini-bars on right side
                        let bar_base_x = emb_rect.left() + 96.0;
                        for h in 0..4 {
                            let bx = bar_base_x + h as f32 * 14.0;
                            let h_amp = match h {
                                0 => 0.90,
                                1 => 0.60 * (angle / 60.0),
                                2 => 0.40,
                                _ => 0.25 * (press / 3000.0),
                            };
                            let bh = (h_amp * 30.0).clamp(3.0, 30.0);
                            let col = if h == 0 { Color32::from_rgb(255, 180, 50) } else { Color32::from_rgb(0, 255, 180) };
                            emb_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, emb_rect.bottom() - 14.0 - bh), egui::pos2(bx + 10.0, emb_rect.bottom() - 14.0)),
                                1.5_f32,
                                col,
                            );
                        }

                        // Readouts
                        emb_painter.text(
                            egui::pos2(emb_rect.left() + 4.0, emb_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.1}°", angle),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        emb_painter.text(
                            egui::pos2(emb_rect.right() - 4.0, emb_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.0}Pa", press),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 180, 50),
                        );

                        if emb_resp.hovered() {
                            let _ = emb_resp.on_hover_text(format!("Shakuhachi Embouchure & Meri/Kari HUD\nAngle: {:.1}° | Pressure: {:.0} Pa\n[Drag horizontally: Utaguchi angle | Drag vertically: Blowing pressure]", angle, press));
                        }
                    } else if is_friction_orbit {
                        // Bowed String Stick-Slip Friction & Orbit Mini Visualizer
                        let (fric_resp, fric_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let fric_rect = fric_resp.rect;
                        fric_painter.rect_filled(fric_rect, 2.0, Color32::from_rgb(20, 14, 24));

                        let mut bow_v = state.node_param_values.get("bow_velocity_mps").copied().unwrap_or(0.45);
                        let mut bow_f = state.node_param_values.get("bow_force_n").copied().unwrap_or(1.25);

                        if fric_resp.dragged() {
                            if let Some(pos) = fric_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - fric_rect.left()) / fric_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - fric_rect.top()) / fric_rect.height())).clamp(0.0, 1.0);
                                bow_v = 0.01 + norm_x * 1.99;
                                bow_f = 0.05 + norm_y * 4.95;
                                let adhesion = (82.0 + (bow_f / 5.0) * 15.0).clamp(10.0, 100.0);
                                let score = (0.95 - (bow_v / 2.0) * 0.15).clamp(0.5, 1.0);

                                state.node_param_values.insert("bow_velocity_mps".to_string(), bow_v);
                                state.node_param_values.insert("bow_force_n".to_string(), bow_f);
                                state.node_param_values.insert("rosin_adhesion_pct".to_string(), adhesion);
                                state.node_param_values.insert("helmholtz_coherence_score".to_string(), score);

                                if let Some(bus) = param_bus {
                                    let pid_v = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_f = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_a = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_s = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_v).is_some() { bus.set(pid_v, bow_v); }
                                    if bus.get(pid_f).is_some() { bus.set(pid_f, bow_f); }
                                    if bus.get(pid_a).is_some() { bus.set(pid_a, adhesion); }
                                    if bus.get(pid_s).is_some() { bus.set(pid_s, score); }
                                }
                            }
                        }

                        // Hyperbolic friction characteristic line
                        let left_w = 64.0;
                        let mid_y = fric_rect.center().y;
                        let mut prev_pt = None;
                        for s in 0..=12 {
                            let frac = s as f32 / 12.0;
                            let v_rel = frac * 4.0 - 2.0;
                            let mu = v_rel.signum() * (0.35 + 0.60 / (1.0 + 3.0 * v_rel.abs()));
                            let px = fric_rect.left() + 6.0 + frac * left_w;
                            let py = mid_y - mu * 18.0;
                            let pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                fric_painter.line_segment([prev, pt], Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)));
                            }
                            prev_pt = Some(pt);
                        }

                        // Helmholtz limit cycle loop on right side
                        let cx = fric_rect.left() + 115.0;
                        let cy = fric_rect.top() + 34.0;
                        let rx = 24.0 * (bow_v / 2.0).clamp(0.3, 1.0);
                        let ry = 18.0 * (bow_f / 5.0).clamp(0.3, 1.0);
                        let mut prev_loop = None;
                        for s in 0..=16 {
                            let ph = s as f32 / 16.0 * std::f32::consts::TAU;
                            let lx = cx + rx * ph.cos();
                            let ly = cy + ry * ph.sin();
                            let pt = egui::pos2(lx, ly);
                            if let Some(prev) = prev_loop {
                                fric_painter.line_segment([prev, pt], Stroke::new(1.8_f32, Color32::from_rgb(255, 215, 0)));
                            }
                            prev_loop = Some(pt);
                        }

                        // Interactive Puck
                        let norm_x = ((bow_v - 0.01) / 1.99).clamp(0.0, 1.0);
                        let norm_y = ((bow_f - 0.05) / 4.95).clamp(0.0, 1.0);
                        let px = cx + (norm_x * 2.0 - 1.0) * rx;
                        let py = cy - (norm_y * 2.0 - 1.0) * ry;
                        fric_painter.circle_stroke(egui::pos2(px, py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 107, 43)));
                        fric_painter.circle_filled(egui::pos2(px, py), 2.0, Color32::from_rgb(255, 255, 255));

                        // Readouts
                        fric_painter.text(
                            egui::pos2(fric_rect.left() + 4.0, fric_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.2}m/s", bow_v),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        fric_painter.text(
                            egui::pos2(fric_rect.right() - 4.0, fric_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.2}N", bow_f),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if fric_resp.hovered() {
                            let _ = fric_resp.on_hover_text(format!("Bowed String Stick-Slip Friction & Orbit HUD\nVelocity: {:.2} m/s | Force: {:.2} N\n[Drag horizontally: Bow velocity | Drag vertically: Bow normal force]", bow_v, bow_f));
                        }
                    } else if is_rank_voicing {
                        // Pipe Organ Rank Voicing & Flue Cutup Mini Visualizer
                        let (rank_resp, rank_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let rank_rect = rank_resp.rect;
                        rank_painter.rect_filled(rank_rect, 2.0, Color32::from_rgb(14, 18, 28));

                        let mut cutup = state.node_param_values.get("cutup_ratio").copied().unwrap_or(0.25);
                        let mut toe = state.node_param_values.get("toe_hole_aperture").copied().unwrap_or(0.85);

                        if rank_resp.dragged() {
                            if let Some(pos) = rank_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - rank_rect.left()) / rank_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - rank_rect.top()) / rank_rect.height())).clamp(0.0, 1.0);
                                cutup = 0.15 + norm_x * 0.35;
                                toe = 0.20 + norm_y * 0.80;
                                let air_v = (2.0 * (80.0 * 9.80665) / 1.204f32).sqrt() * toe;

                                state.node_param_values.insert("cutup_ratio".to_string(), cutup);
                                state.node_param_values.insert("toe_hole_aperture".to_string(), toe);
                                state.node_param_values.insert("air_velocity_mps".to_string(), air_v);

                                if let Some(bus) = param_bus {
                                    let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_toe = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_cut).is_some() { bus.set(pid_cut, cutup); }
                                    if bus.get(pid_toe).is_some() { bus.set(pid_toe, toe); }
                                }
                            }
                        }

                        // Pipe body and flue mouth outline
                        let pipe_x0 = rank_rect.left() + 20.0;
                        let pipe_w = 40.0;
                        let pipe_bot = rank_rect.bottom() - 6.0;
                        let pipe_top = rank_rect.top() + 6.0;
                        rank_painter.rect_stroke(
                            egui::Rect::from_min_max(egui::pos2(pipe_x0, pipe_top), egui::pos2(pipe_x0 + pipe_w, pipe_bot)),
                            2.0,
                            Stroke::new(1.2_f32, Color32::from_rgb(120, 140, 180)),
                        );

                        // Flue mouth opening cutup
                        let mouth_h = (cutup * 50.0).clamp(6.0, 30.0);
                        let mouth_y = pipe_bot - 14.0 - mouth_h;
                        rank_painter.rect_filled(
                            egui::Rect::from_min_max(egui::pos2(pipe_x0 + 4.0, mouth_y), egui::pos2(pipe_x0 + pipe_w - 4.0, mouth_y + mouth_h)),
                            1.0,
                            Color32::from_rgb(255, 180, 50),
                        );

                        // Air jet stream line
                        let jet_x = pipe_x0 + pipe_w * 0.5;
                        rank_painter.line_segment(
                            [egui::pos2(jet_x, pipe_bot), egui::pos2(jet_x, mouth_y)],
                            Stroke::new(1.8_f32 * toe, Color32::from_rgb(56, 189, 248)),
                        );

                        // Radiation bars on right
                        let bars_x0 = rank_rect.left() + 75.0;
                        let bars_w = rank_rect.right() - bars_x0 - 8.0;
                        for b in 0..6 {
                            let bx = bars_x0 + (b as f32 / 5.0) * (bars_w - 8.0);
                            let bh = (28.0 / (b as f32 + 1.0).sqrt() * toe).clamp(4.0, 38.0);
                            let by = rank_rect.bottom() - 10.0 - bh;
                            rank_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, by), egui::pos2(bx + 6.0, rank_rect.bottom() - 10.0)),
                                1.0,
                                Color32::from_rgb(255, 200, 80),
                            );
                        }

                        // Readouts
                        rank_painter.text(
                            egui::pos2(rank_rect.left() + 4.0, rank_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Cutup: {:.2}", cutup),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 180, 50),
                        );
                        rank_painter.text(
                            egui::pos2(rank_rect.right() - 4.0, rank_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Toe: {:.2}", toe),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if rank_resp.hovered() {
                            let _ = rank_resp.on_hover_text(format!("Pipe Organ Rank Voicing HUD\nCutup: {:.2} | Toe Hole: {:.2}\n[Drag horizontally: Mouth cutup ratio | Drag vertically: Toe hole aperture]", cutup, toe));
                        }
                    } else if is_tine_resonator {
                        // Electromechanical Tine Resonator & Tremolo Mini Visualizer
                        let (tine_resp, tine_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let tine_rect = tine_resp.rect;
                        tine_painter.rect_filled(tine_rect, 2.0, Color32::from_rgb(18, 16, 26));

                        let mut coupling = state.node_param_values.get("tonebar_coupling").copied().unwrap_or(0.70);
                        let mut stiff = state.node_param_values.get("beam_stiffness").copied().unwrap_or(0.42);

                        if tine_resp.dragged() {
                            if let Some(pos) = tine_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - tine_rect.left()) / tine_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - tine_rect.top()) / tine_rect.height())).clamp(0.0, 1.0);
                                coupling = norm_x;
                                stiff = norm_y;

                                state.node_param_values.insert("tonebar_coupling".to_string(), coupling);
                                state.node_param_values.insert("beam_stiffness".to_string(), stiff);

                                if let Some(bus) = param_bus {
                                    let pid_cpl = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_stf = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_cpl).is_some() { bus.set(pid_cpl, coupling); }
                                    if bus.get(pid_stf).is_some() { bus.set(pid_stf, stiff); }
                                }
                            }
                        }

                        // Cantilever tine clamped beam curve
                        let beam_x0 = tine_rect.left() + 8.0;
                        let beam_w = 60.0;
                        let beam_y = tine_rect.center().y;
                        // Clamping block
                        tine_painter.rect_filled(
                            egui::Rect::from_min_max(egui::pos2(beam_x0, beam_y - 8.0), egui::pos2(beam_x0 + 6.0, beam_y + 8.0)),
                            1.0,
                            Color32::from_rgb(100, 116, 139),
                        );
                        let mut prev_beam = None;
                        for s in 0..=12 {
                            let frac = s as f32 / 12.0;
                            let px = beam_x0 + 6.0 + frac * beam_w;
                            let deflect = (frac * frac * frac) * 16.0 * (1.0 - 0.4 * stiff);
                            let py = beam_y - deflect;
                            let pt = egui::pos2(px, py);
                            if let Some(prev) = prev_beam {
                                tine_painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(255, 183, 3)));
                            }
                            prev_beam = Some(pt);
                        }

                        // Tonebar mass
                        tine_painter.rect_filled(
                            egui::Rect::from_min_max(egui::pos2(beam_x0 + 10.0, beam_y + 4.0), egui::pos2(beam_x0 + 50.0, beam_y + 12.0)),
                            2.0,
                            Color32::from_rgb(217, 119, 6),
                        );

                        // Tremolo Lissajous mini ellipse on right
                        let cx = tine_rect.left() + 118.0;
                        let cy = tine_rect.top() + 34.0;
                        let rx = 24.0 * coupling.clamp(0.2, 1.0);
                        let ry = 18.0;
                        let mut prev_ell = None;
                        for s in 0..=16 {
                            let ph = s as f32 / 16.0 * std::f32::consts::TAU;
                            let lx = cx + rx * ph.cos();
                            let ly = cy + ry * ph.sin();
                            let pt = egui::pos2(lx, ly);
                            if let Some(prev) = prev_ell {
                                tine_painter.line_segment([prev, pt], Stroke::new(1.4_f32, Color32::from_rgb(0, 245, 212)));
                            }
                            prev_ell = Some(pt);
                        }

                        // Drag Puck
                        let puck_px = beam_x0 + 6.0 + coupling * beam_w;
                        let puck_py = beam_y - (coupling * coupling * coupling) * 16.0 * (1.0 - 0.4 * stiff);
                        tine_painter.circle_stroke(egui::pos2(puck_px, puck_py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 183, 3)));
                        tine_painter.circle_filled(egui::pos2(puck_px, puck_py), 2.0, Color32::from_rgb(255, 255, 255));

                        // Readouts
                        tine_painter.text(
                            egui::pos2(tine_rect.left() + 4.0, tine_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Cpl: {:.0}%", coupling * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 183, 3),
                        );
                        tine_painter.text(
                            egui::pos2(tine_rect.right() - 4.0, tine_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Stiff: {:.2}", stiff),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 245, 212),
                        );

                        if tine_resp.hovered() {
                            let _ = tine_resp.on_hover_text(format!("Electric Piano Tine Resonator HUD\nCoupling: {:.0}% | Stiffness: {:.2}\n[Drag horizontally: Tonebar coupling | Drag vertically: Beam stiffness]", coupling * 100.0, stiff));
                        }
                    } else if is_spring_reverb {
                        // Spring Reverb Tank Coils & Boing Dispersion Mini Visualizer
                        let (sp_resp, sp_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sp_rect = sp_resp.rect;
                        sp_painter.rect_filled(sp_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut tension = state.node_param_values.get("tension_pct").copied().unwrap_or(60.0);
                        let mut chirp = state.node_param_values.get("dispersion_chirp_pct").copied().unwrap_or(65.0);

                        if sp_resp.dragged() {
                            if let Some(pos) = sp_resp.interact_pointer_pos() {
                                let nx = ((pos.x - sp_rect.left()) / sp_rect.width()).clamp(0.05, 0.95);
                                let ny = (1.0 - ((pos.y - sp_rect.top()) / sp_rect.height())).clamp(0.05, 0.95);
                                tension = nx * 100.0;
                                chirp = ny * 100.0;
                                state.node_param_values.insert("tension_pct".to_string(), tension);
                                state.node_param_values.insert("dispersion_chirp_pct".to_string(), chirp);
                                if let Some(bus) = param_bus {
                                    let pid_tns = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_tns).is_some() { bus.set(pid_tns, tension); }
                                    let pid_chp = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_chp).is_some() { bus.set(pid_chp, chirp); }
                                }
                            }
                        }

                        // Draw mini spring coils
                        let num_coils = 2;
                        let coil_colors = [Color32::from_rgb(0, 229, 255), Color32::from_rgb(255, 107, 43)];
                        for c in 0..num_coils {
                            let y_base = sp_rect.top() + 18.0 + c as f32 * 24.0;
                            let mut prev_pt = None;
                            for s in 0..24 {
                                let t = s as f32 / 23.0;
                                let px = sp_rect.left() + 8.0 + t * (sp_rect.width() - 16.0);
                                let py = y_base + (t * std::f32::consts::PI * 8.0 * (0.5 + tension * 0.005)).sin() * 5.0;
                                let pt = egui::pos2(px, py);
                                if let Some(prev) = prev_pt {
                                    sp_painter.line_segment([prev, pt], Stroke::new(1.4_f32, coil_colors[c % 2]));
                                }
                                prev_pt = Some(pt);
                            }
                        }

                        // Drag Puck
                        let puck_px = sp_rect.left() + (tension / 100.0) * sp_rect.width();
                        let puck_py = sp_rect.top() + (1.0 - chirp / 100.0) * sp_rect.height();
                        sp_painter.circle_stroke(egui::pos2(puck_px, puck_py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        sp_painter.circle_filled(egui::pos2(puck_px, puck_py), 2.0, Color32::from_rgb(255, 255, 255));

                        // Readouts
                        sp_painter.text(
                            egui::pos2(sp_rect.left() + 4.0, sp_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Tens: {:.0}%", tension),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        sp_painter.text(
                            egui::pos2(sp_rect.right() - 4.0, sp_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Boing: {:.0}%", chirp),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if sp_resp.hovered() {
                            let _ = sp_resp.on_hover_text(format!("Spring Reverb Tank HUD\nTension: {:.0}% | Boing: {:.0}%\n[Drag horizontally: Tension | Drag vertically: Dispersion Chirp]", tension, chirp));
                        }
                    } else if is_comb_resonator {
                        // Spectral Comb Resonator Harmonic Teeth Mini Visualizer
                        let (cb_resp, cb_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let cb_rect = cb_resp.rect;
                        cb_painter.rect_filled(cb_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let mut base_f = state.node_param_values.get("base_frequency_hz").copied().unwrap_or(440.0);
                        let mut feedback = state.node_param_values.get("feedback_pct").copied().unwrap_or(85.0);

                        if cb_resp.dragged() {
                            if let Some(pos) = cb_resp.interact_pointer_pos() {
                                let nx = ((pos.x - cb_rect.left()) / cb_rect.width()).clamp(0.02, 0.98);
                                let ny = (1.0 - ((pos.y - cb_rect.top()) / cb_rect.height())).clamp(0.05, 0.98);
                                base_f = 20.0 * 10.0_f32.powf(nx * (20000.0_f32 / 20.0).log10());
                                feedback = ny * 99.0;
                                state.node_param_values.insert("base_frequency_hz".to_string(), base_f);
                                state.node_param_values.insert("feedback_pct".to_string(), feedback);
                                if let Some(bus) = param_bus {
                                    let pid_f = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_f).is_some() { bus.set(pid_f, base_f); }
                                    let pid_fb = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_fb).is_some() { bus.set(pid_fb, feedback); }
                                }
                            }
                        }

                        // Draw mini comb response teeth curve
                        let norm_fx = ((base_f / 20.0).log10() / (20000.0_f32 / 20.0).log10()).clamp(0.0, 1.0);
                        let steps = 30;
                        let mut prev_curve = None;
                        for s in 0..=steps {
                            let t = s as f32 / steps as f32;
                            let px = cb_rect.left() + 6.0 + t * (cb_rect.width() - 12.0);
                            let tooth = (t * std::f32::consts::PI * 8.0 * (1.0 + norm_fx * 2.0)).sin().abs();
                            let py = cb_rect.bottom() - 8.0 - tooth * (feedback / 100.0) * 36.0;
                            let pt = egui::pos2(px, py);
                            if let Some(prev) = prev_curve {
                                cb_painter.line_segment([prev, pt], Stroke::new(1.4_f32, Color32::from_rgb(255, 215, 0)));
                            }
                            prev_curve = Some(pt);
                        }

                        // Drag Puck
                        let puck_px = cb_rect.left() + norm_fx * cb_rect.width();
                        let puck_py = cb_rect.top() + (1.0 - feedback / 100.0) * cb_rect.height();
                        cb_painter.circle_stroke(egui::pos2(puck_px, puck_py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 215, 0)));
                        cb_painter.circle_filled(egui::pos2(puck_px, puck_py), 2.0, Color32::from_rgb(255, 255, 255));

                        // Readouts
                        let f_label = if base_f >= 1000.0 { format!("{:.1}k", base_f / 1000.0) } else { format!("{:.0}Hz", base_f) };
                        cb_painter.text(
                            egui::pos2(cb_rect.left() + 4.0, cb_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("F0: {}", f_label),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );
                        cb_painter.text(
                            egui::pos2(cb_rect.right() - 4.0, cb_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("FB: {:.0}%", feedback),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if cb_resp.hovered() {
                            let _ = cb_resp.on_hover_text(format!("Spectral Comb Resonator HUD\nBase: {:.1} Hz | FB: {:.0}%\n[Drag horizontally: F0 pitch | Drag vertically: Feedback resonance]", base_f, feedback));
                        }
                    } else if is_wavefront_reflection {
                        // Neural Acoustic Wavefront Boundary Reflection & Impulse Decay Mini Visualizer
                        let (wf_resp, wf_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let wf_rect = wf_resp.rect;
                        wf_painter.rect_filled(wf_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut scattering = state.node_param_values.get("surface_scattering_coeff").copied().unwrap_or(0.45);
                        let mut absorption = state.node_param_values.get("boundary_absorption_alpha").copied().unwrap_or(0.20);

                        if wf_resp.dragged() {
                            if let Some(pos) = wf_resp.interact_pointer_pos() {
                                scattering = ((pos.x - wf_rect.left()) / wf_rect.width()).clamp(0.0, 1.0);
                                absorption = (1.0 - ((pos.y - wf_rect.top()) / wf_rect.height())).clamp(0.0, 1.0);
                                state.node_param_values.insert("surface_scattering_coeff".to_string(), scattering);
                                state.node_param_values.insert("boundary_absorption_alpha".to_string(), absorption);
                                if let Some(bus) = param_bus {
                                    let pid_scat = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_scat).is_some() { bus.set(pid_scat, scattering); }
                                    let pid_abs = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_abs).is_some() { bus.set(pid_abs, absorption); }
                                }
                            }
                        }

                        // Boundary Line & Ray Reflection Display
                        let origin_x = wf_rect.left() + 12.0;
                        let origin_y = wf_rect.bottom() - 10.0;
                        let boundary_x = wf_rect.left() + 70.0;

                        // Wall boundary line
                        wf_painter.line_segment(
                            [egui::pos2(boundary_x, wf_rect.top() + 6.0), egui::pos2(boundary_x, wf_rect.bottom() - 6.0)],
                            Stroke::new(1.5_f32, Color32::from_rgb(100, 130, 170)),
                        );

                        // Incident ray
                        wf_painter.line_segment(
                            [egui::pos2(origin_x, origin_y), egui::pos2(boundary_x, wf_rect.center().y)],
                            Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)),
                        );

                        // Reflected rays (specular + diffuse)
                        let spec_alpha = ((1.0 - absorption) * 200.0).clamp(30.0, 255.0) as u8;
                        wf_painter.line_segment(
                            [egui::pos2(boundary_x, wf_rect.center().y), egui::pos2(origin_x + 10.0, wf_rect.top() + 10.0)],
                            Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(0, 255, 180, spec_alpha)),
                        );

                        // Mini 6-tap impulse histogram on right side
                        let hist_left = wf_rect.left() + 82.0;
                        let hist_w = wf_rect.right() - hist_left - 6.0;
                        let tap_w = hist_w / 6.0;
                        for i in 0..6 {
                            let t = i as f32;
                            let decay = ((-0.25 * t * (1.0 + absorption)).exp() * (1.0 - absorption * 0.5)).clamp(0.05, 1.0);
                            let bar_h = decay * 36.0;
                            let bx = hist_left + i as f32 * tap_w + 1.0;
                            let by = wf_rect.bottom() - 8.0 - bar_h;
                            let col = if i < 2 {
                                Color32::from_rgb(0, 229, 255)
                            } else if i < 4 {
                                Color32::from_rgb(0, 255, 180)
                            } else {
                                Color32::from_rgb(255, 180, 0)
                            };
                            wf_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2((tap_w - 2.0).max(2.0), bar_h)),
                                1.0,
                                col,
                            );
                        }

                        // Drag Puck
                        let puck_px = wf_rect.left() + scattering * (boundary_x - wf_rect.left() - 8.0) + 4.0;
                        let puck_py = wf_rect.top() + (1.0 - absorption) * (wf_rect.height() - 16.0) + 8.0;
                        wf_painter.circle_stroke(egui::pos2(puck_px, puck_py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        wf_painter.circle_filled(egui::pos2(puck_px, puck_py), 2.0, Color32::WHITE);

                        // Readouts
                        wf_painter.text(
                            egui::pos2(wf_rect.left() + 4.0, wf_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Scat: {:.0}%", scattering * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        wf_painter.text(
                            egui::pos2(wf_rect.right() - 4.0, wf_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Abs: {:.0}%", absorption * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if wf_resp.hovered() {
                            let _ = wf_resp.on_hover_text(format!(
                                "Wavefront Reflection HUD\nScattering: {:.0}% | Absorption: {:.0}%\n[Drag horizontally: Surface scattering | Drag vertically: Absorption alpha]",
                                scattering * 100.0, absorption * 100.0
                            ));
                        }
                    } else if is_diffractive_propagation {
                        // Neural Acoustic Continuous Latent Diffractive Wave Propagation Mini Visualizer
                        let (df_resp, df_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let df_rect = df_resp.rect;
                        df_painter.rect_filled(df_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let mut angle = state.node_param_values.get("diffraction_angle_deg").copied().unwrap_or(75.0);
                        let mut dist = state.node_param_values.get("source_distance_m").copied().unwrap_or(4.5);

                        if df_resp.dragged() {
                            if let Some(pos) = df_resp.interact_pointer_pos() {
                                let nx = ((pos.x - df_rect.left()) / df_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - df_rect.top()) / df_rect.height())).clamp(0.0, 1.0);
                                angle = nx * 180.0;
                                dist = 0.5 + ny * (25.0 - 0.5);
                                state.node_param_values.insert("diffraction_angle_deg".to_string(), angle);
                                state.node_param_values.insert("source_distance_m".to_string(), dist);
                                if let Some(bus) = param_bus {
                                    let pid_ang = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_ang).is_some() { bus.set(pid_ang, angle); }
                                    let pid_dst = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_dst).is_some() { bus.set(pid_dst, dist); }
                                }
                            }
                        }

                        // Obstacle Wedge & Diffracted Wavefront Arcs
                        let wedge_tip = egui::pos2(df_rect.left() + 50.0, df_rect.center().y);
                        df_painter.line_segment(
                            [wedge_tip, egui::pos2(df_rect.left() + 75.0, df_rect.bottom() - 4.0)],
                            Stroke::new(2.5_f32, Color32::from_rgb(120, 140, 170)),
                        );
                        df_painter.line_segment(
                            [wedge_tip, egui::pos2(df_rect.left() + 25.0, df_rect.bottom() - 4.0)],
                            Stroke::new(2.5_f32, Color32::from_rgb(120, 140, 170)),
                        );

                        // Arcs around wedge
                        for r in [12.0, 24.0, 36.0] {
                            df_painter.circle_stroke(
                                wedge_tip,
                                r,
                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 70)),
                            );
                        }

                        // Right side: 6-band attenuation spectrum bars
                        let spec_left = df_rect.left() + 85.0;
                        let spec_w = df_rect.right() - spec_left - 6.0;
                        let bar_w = spec_w / 6.0;
                        let bend = (angle / 180.0).clamp(0.0, 1.0);
                        for i in 0..6 {
                            let f_norm = i as f32 / 5.0;
                            let att = (bend * (1.0 + f_norm * 2.0) * 0.7).clamp(0.05, 0.95);
                            let bar_h = (1.0 - att) * 36.0;
                            let bx = spec_left + i as f32 * bar_w + 1.0;
                            let by = df_rect.bottom() - 8.0 - bar_h;
                            let col = if i < 2 {
                                Color32::from_rgb(0, 255, 180)
                            } else if i < 4 {
                                Color32::from_rgb(0, 229, 255)
                            } else {
                                Color32::from_rgb(255, 180, 0)
                            };
                            df_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2((bar_w - 2.0).max(2.0), bar_h)),
                                1.0,
                                col,
                            );
                        }

                        // Source Puck
                        let norm_ang = (angle / 180.0).clamp(0.0, 1.0);
                        let norm_dist = ((dist - 0.5) / 24.5).clamp(0.0, 1.0);
                        let puck_px = df_rect.left() + norm_ang * 45.0 + 6.0;
                        let puck_py = df_rect.top() + (1.0 - norm_dist) * (df_rect.height() - 16.0) + 8.0;
                        df_painter.circle_stroke(egui::pos2(puck_px, puck_py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 180, 0)));
                        df_painter.circle_filled(egui::pos2(puck_px, puck_py), 2.0, Color32::WHITE);

                        // Readouts
                        df_painter.text(
                            egui::pos2(df_rect.left() + 4.0, df_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.0}°", angle),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 180, 0),
                        );
                        df_painter.text(
                            egui::pos2(df_rect.right() - 4.0, df_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.1}m", dist),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if df_resp.hovered() {
                            let _ = df_resp.on_hover_text(format!(
                                "Diffractive Wave Propagation HUD\nAngle: {:.1}° | Distance: {:.1}m\n[Drag horizontally: Diffraction angle | Drag vertically: Distance]",
                                angle, dist
                            ));
                        }
                    } else if is_membrane_resonator {
                        // Physical Modeling Acoustic Membrane Resonator Mini Visualizer
                        let (mr_resp, mr_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mr_rect = mr_resp.rect;
                        mr_painter.rect_filled(mr_rect, 2.0, Color32::from_rgb(14, 18, 28));

                        let mut strike_pos = state.node_param_values.get("radial_strike_pos").copied().unwrap_or(0.35);
                        let mut tension = state.node_param_values.get("membrane_tension_n").copied().unwrap_or(450.0);

                        if mr_resp.dragged() {
                            if let Some(pos) = mr_resp.interact_pointer_pos() {
                                let nx = ((pos.x - mr_rect.left()) / mr_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - mr_rect.top()) / mr_rect.height())).clamp(0.0, 1.0);
                                strike_pos = nx;
                                tension = 50.0 + ny * (2000.0 - 50.0);
                                state.node_param_values.insert("radial_strike_pos".to_string(), strike_pos);
                                state.node_param_values.insert("membrane_tension_n".to_string(), tension);
                                if let Some(bus) = param_bus {
                                    let pid_pos = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_pos).is_some() { bus.set(pid_pos, strike_pos); }
                                    let pid_ten = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_ten).is_some() { bus.set(pid_ten, tension); }
                                }
                            }
                        }

                        // Circular Drumhead Rim & Concentric Modal Rings
                        let center = egui::pos2(mr_rect.left() + 45.0, mr_rect.center().y);
                        mr_painter.circle_stroke(center, 26.0, Stroke::new(2.0_f32, Color32::from_rgb(180, 140, 80)));
                        mr_painter.circle_stroke(center, 18.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 170, 0, 70)));
                        mr_painter.circle_stroke(center, 10.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 170, 0, 120)));

                        // Strike Puck
                        let puck_x = center.x + strike_pos * 24.0 * 0.707;
                        let puck_y = center.y + strike_pos * 24.0 * 0.707;
                        mr_painter.circle_stroke(egui::pos2(puck_x, puck_y), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 170, 0)));
                        mr_painter.circle_filled(egui::pos2(puck_x, puck_y), 2.0, Color32::WHITE);

                        // Right side: Tension meter & frequency readout
                        let meter_x = mr_rect.left() + 85.0;
                        let meter_w = mr_rect.right() - meter_x - 6.0;
                        let norm_t = ((tension - 50.0) / 1950.0).clamp(0.0, 1.0);
                        mr_painter.rect_filled(
                            egui::Rect::from_min_max(egui::pos2(meter_x, mr_rect.top() + 24.0), egui::pos2(meter_x + meter_w, mr_rect.top() + 32.0)),
                            2.0,
                            Color32::from_rgb(24, 30, 44),
                        );
                        mr_painter.rect_filled(
                            egui::Rect::from_min_max(egui::pos2(meter_x, mr_rect.top() + 24.0), egui::pos2(meter_x + meter_w * norm_t, mr_rect.top() + 32.0)),
                            2.0,
                            Color32::from_rgb(255, 170, 0),
                        );

                        // Readouts
                        mr_painter.text(
                            egui::pos2(mr_rect.left() + 4.0, mr_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("r: {:.2}", strike_pos),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 170, 0),
                        );
                        mr_painter.text(
                            egui::pos2(mr_rect.right() - 4.0, mr_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("{:.0} N/m", tension),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 215, 0),
                        );
                        let approx_f0 = (tension / 0.26).sqrt() * 2.4048 / (2.0 * std::f32::consts::PI * 0.178);
                        mr_painter.text(
                            egui::pos2(meter_x, mr_rect.bottom() - 14.0),
                            egui::Align2::LEFT_TOP,
                            format!("f0: {:.0} Hz", approx_f0),
                            FontId::proportional(9.0),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if mr_resp.hovered() {
                            let _ = mr_resp.on_hover_text(format!(
                                "Membrane Resonator HUD\nStrike Radius: {:.2} r/R | Tension: {:.0} N/m (f0: {:.0} Hz)\n[Drag horizontally: Strike radius | Drag vertically: Skin tension]",
                                strike_pos, tension, approx_f0
                            ));
                        }
                    } else if is_membrane_plate {
                        // Physical Modeling Coupled Membrane & Plate Resonator Mini Visualizer
                        let (mp_resp, mp_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mp_rect = mp_resp.rect;
                        mp_painter.rect_filled(mp_rect, 2.0, Color32::from_rgb(14, 20, 30));

                        let mut thickness = state.node_param_values.get("plate_elastic_modulus").copied().unwrap_or(1.8);
                        let mut membrane_tension = state.node_param_values.get("membrane_tension").copied().unwrap_or(0.75);

                        if mp_resp.dragged() {
                            if let Some(pos) = mp_resp.interact_pointer_pos() {
                                let nx = ((pos.x - mp_rect.left()) / mp_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - mp_rect.top()) / mp_rect.height())).clamp(0.0, 1.0);
                                thickness = 0.1 + nx * (5.0 - 0.1);
                                membrane_tension = 0.1 + ny * (1.0 - 0.1);
                                state.node_param_values.insert("plate_elastic_modulus".to_string(), thickness);
                                state.node_param_values.insert("membrane_tension".to_string(), membrane_tension);
                                if let Some(bus) = param_bus {
                                    let pid_mod = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_mod).is_some() { bus.set(pid_mod, thickness); }
                                    let pid_ten = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_ten).is_some() { bus.set(pid_ten, membrane_tension); }
                                }
                            }
                        }

                        // Plate & Membrane 2D Graphic
                        let plate_box = egui::Rect::from_min_max(
                            egui::pos2(mp_rect.left() + 10.0, mp_rect.top() + 16.0),
                            egui::pos2(mp_rect.left() + 70.0, mp_rect.bottom() - 10.0),
                        );
                        mp_painter.rect_filled(plate_box, 2.0, Color32::from_rgb(28, 38, 54));
                        mp_painter.rect_stroke(plate_box, 2.0, Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)));

                        // Modal nodal lines
                        mp_painter.line_segment(
                            [egui::pos2(plate_box.center().x, plate_box.top()), egui::pos2(plate_box.center().x, plate_box.bottom())],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
                        );
                        mp_painter.line_segment(
                            [egui::pos2(plate_box.left(), plate_box.center().y), egui::pos2(plate_box.right(), plate_box.center().y)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
                        );

                        // 5 Modal Energy Bars on right
                        let spec_left = mp_rect.left() + 82.0;
                        let spec_w = mp_rect.right() - spec_left - 6.0;
                        let bar_w = spec_w / 5.0;
                        let ratios: [f32; 5] = [1.0, 1.6, 2.4, 3.1, 3.9];
                        for (i, &r) in ratios.iter().enumerate() {
                            let h = ((1.0 / r.sqrt()) * (thickness / 5.0 * 24.0 + membrane_tension * 12.0)).clamp(4.0, 36.0);
                            let bx = spec_left + i as f32 * bar_w + 1.0;
                            let by = mp_rect.bottom() - 8.0 - h;
                            let col = if i == 0 { Color32::from_rgb(0, 255, 180) } else if i < 3 { Color32::from_rgb(0, 229, 255) } else { Color32::from_rgb(255, 180, 0) };
                            mp_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2((bar_w - 2.0).max(2.0), h)),
                                1.0,
                                col,
                            );
                        }

                        // Readouts
                        mp_painter.text(
                            egui::pos2(mp_rect.left() + 4.0, mp_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("E: {:.1} GPa", thickness),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 229, 255),
                        );
                        mp_painter.text(
                            egui::pos2(mp_rect.right() - 4.0, mp_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("T: {:.0}%", membrane_tension * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 180, 0),
                        );

                        if mp_resp.hovered() {
                            let _ = mp_resp.on_hover_text(format!(
                                "Membrane & Plate Resonator HUD\nPlate Stiffness: {:.1} GPa | Tension: {:.0}%\n[Drag horizontally: Plate stiffness | Drag vertically: Membrane tension]",
                                thickness, membrane_tension * 100.0
                            ));
                        }
                    } else if is_idiophone {
                        // Physical Modeling Struck Idiophone Resonator Bank Mini Visualizer
                        let (id_resp, id_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let id_rect = id_resp.rect;
                        id_painter.rect_filled(id_rect, 2.0, Color32::from_rgb(18, 16, 26));

                        let mut hardness = state.node_param_values.get("mallet_hardness").copied().unwrap_or(0.35);
                        let mut velocity = state.node_param_values.get("strike_velocity").copied().unwrap_or(0.80);

                        if id_resp.dragged() {
                            if let Some(pos) = id_resp.interact_pointer_pos() {
                                let nx = ((pos.x - id_rect.left()) / id_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - id_rect.top()) / id_rect.height())).clamp(0.0, 1.0);
                                hardness = nx;
                                velocity = 0.05 + ny * 0.95;
                                state.node_param_values.insert("mallet_hardness".to_string(), hardness);
                                state.node_param_values.insert("strike_velocity".to_string(), velocity);
                                if let Some(bus) = param_bus {
                                    let pid_hard = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_hard).is_some() { bus.set(pid_hard, hardness); }
                                    let pid_vel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_vel).is_some() { bus.set(pid_vel, velocity); }
                                }
                            }
                        }

                        // Modal Spectrum Bars (8 modes)
                        let bar_w = 8.0;
                        let base_y = id_rect.bottom() - 14.0;
                        for i in 0..8 {
                            let bx = id_rect.left() + 6.0 + i as f32 * 11.0;
                            let mode_mult = 1.0 / (1.0 + 0.45 * (i as f32) * (1.0 - 0.75 * hardness));
                            let bar_h = (32.0 * mode_mult * velocity).clamp(3.0, 38.0);
                            let bar_col = if i == 0 {
                                Color32::from_rgb(255, 190, 40)
                            } else if i == 1 || i == 2 {
                                Color32::from_rgb(245, 158, 11)
                            } else {
                                Color32::from_rgb(180, 110, 30)
                            };
                            id_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, base_y - bar_h), egui::pos2(bx + bar_w, base_y)),
                                1.5,
                                bar_col,
                            );
                        }

                        // Strike Mallet Puck Indicator on right side
                        let puck_box = egui::Rect::from_min_max(
                            egui::pos2(id_rect.left() + 100.0, id_rect.top() + 18.0),
                            egui::pos2(id_rect.right() - 8.0, id_rect.bottom() - 14.0),
                        );
                        id_painter.rect_filled(puck_box, 2.0, Color32::from_rgb(26, 22, 38));
                        id_painter.rect_stroke(puck_box, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 50, 80)));
                        let px = puck_box.left() + hardness * puck_box.width();
                        let py = puck_box.bottom() - ((velocity - 0.05) / 0.95) * puck_box.height();
                        id_painter.circle_stroke(egui::pos2(px, py), 4.5, Stroke::new(1.2_f32, Color32::from_rgb(255, 190, 40)));
                        id_painter.circle_filled(egui::pos2(px, py), 2.0, Color32::WHITE);

                        // Readout
                        id_painter.text(
                            egui::pos2(id_rect.left() + 4.0, id_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Hard: {:.2} | Vel: {:.2}", hardness, velocity),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 190, 40),
                        );

                        if id_resp.hovered() {
                            let _ = id_resp.on_hover_text(format!(
                                "Idiophone Spectrum HUD\nMallet Hardness: {:.2} | Velocity: {:.2}\n[Drag: Hardness (X) vs Strike Velocity (Y)]",
                                hardness, velocity
                            ));
                        }
                    } else if is_sonar_hydrophone {
                        // Physical Modeling Underwater Sonar & Ocean Hydrophone Mini Visualizer
                        let (so_resp, so_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let so_rect = so_resp.rect;
                        so_painter.rect_filled(so_rect, 2.0, Color32::from_rgb(10, 22, 32));

                        let mut depth = state.node_param_values.get("depth_m").copied().unwrap_or(250.0);
                        let mut cav = state.node_param_values.get("cavitation_index").copied().unwrap_or(0.85);

                        if so_resp.dragged() {
                            if let Some(pos) = so_resp.interact_pointer_pos() {
                                let nx = ((pos.x - so_rect.left()) / so_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - so_rect.top()) / so_rect.height())).clamp(0.0, 1.0);
                                cav = 0.05 + nx * (5.0 - 0.05);
                                depth = 1.0 + ny * (5000.0 - 1.0);
                                state.node_param_values.insert("cavitation_index".to_string(), cav);
                                state.node_param_values.insert("depth_m".to_string(), depth);
                                if let Some(bus) = param_bus {
                                    let pid_depth = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_depth).is_some() { bus.set(pid_depth, depth); }
                                    let pid_cav = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_cav).is_some() { bus.set(pid_cav, cav); }
                                }
                            }
                        }

                        // Concentric Sonar Ping Waves & Thermocline Layer
                        let ping_center = egui::pos2(so_rect.left() + 30.0, so_rect.center().y);
                        so_painter.circle_stroke(ping_center, 8.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 210, 255, 120)));
                        so_painter.circle_stroke(ping_center, 16.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 210, 255, 70)));
                        so_painter.circle_stroke(ping_center, 24.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 210, 255, 30)));

                        // Thermocline depth line
                        let thermocline_y = so_rect.top() + 20.0 + (depth / 5000.0) * 36.0;
                        so_painter.line_segment(
                            [egui::pos2(so_rect.left() + 60.0, thermocline_y), egui::pos2(so_rect.right() - 8.0, thermocline_y)],
                            Stroke::new(1.2_f32, Color32::from_rgb(0, 255, 180)),
                        );

                        // Readout
                        let sound_speed = 1448.96 + 4.591 * 12.5 + 0.0163 * depth;
                        so_painter.text(
                            egui::pos2(so_rect.left() + 4.0, so_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("z: {:.0}m | c: {:.0}m/s", depth, sound_speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(0, 210, 255),
                        );
                        so_painter.text(
                            egui::pos2(so_rect.right() - 4.0, so_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("σ: {:.2}", cav),
                            FontId::proportional(8.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if so_resp.hovered() {
                            let _ = so_resp.on_hover_text(format!(
                                "Underwater Sonar & Hydrophone HUD\nDepth: {:.0} m | Sound Speed: {:.0} m/s | Cavitation σ: {:.2}\n[Drag: Cavitation index (X) vs Depth (Y)]",
                                depth, sound_speed, cav
                            ));
                        }
                    } else if is_hoa5_binaural {
                        // Broadcast Mastering 5th-Order Ambisonics HOA5 Spherical Virtualizer Mini Visualizer
                        let (hoa_resp, hoa_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let hoa_rect = hoa_resp.rect;
                        hoa_painter.rect_filled(hoa_rect, 2.0, Color32::from_rgb(16, 12, 28));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(20.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.5);
                        let focus = state.node_param_values.get("energy_focus").copied().unwrap_or(0.65);

                        if hoa_resp.dragged() {
                            if let Some(pos) = hoa_resp.interact_pointer_pos() {
                                let nx = ((pos.x - hoa_rect.left()) / hoa_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - hoa_rect.top()) / hoa_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                el = -90.0 + ny * 180.0;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, el); }
                                }
                            }
                        }

                        // Spherical harmonic concentric order rings (N=1..5)
                        let center = egui::pos2(hoa_rect.left() + 38.0, hoa_rect.center().y);
                        for order in 1..=5 {
                            let r = order as f32 * 5.5;
                            let alpha = 30 + order as u8 * 20;
                            hoa_painter.circle_stroke(
                                center,
                                r,
                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(157, 78, 221, alpha)),
                            );
                        }

                        // Crosshair axis lines
                        hoa_painter.line_segment(
                            [egui::pos2(center.x - 30.0, center.y), egui::pos2(center.x + 30.0, center.y)],
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 60)),
                        );
                        hoa_painter.line_segment(
                            [egui::pos2(center.x, center.y - 30.0), egui::pos2(center.x, center.y + 30.0)],
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 60)),
                        );

                        // Sound Object trajectory vector & puck
                        let az_rad = az.to_radians();
                        let el_rad = el.to_radians();
                        let puck_r = (el_rad.cos() * 26.0).clamp(3.0, 28.0);
                        let puck_pos = egui::pos2(
                            center.x + az_rad.sin() * puck_r,
                            center.y - az_rad.cos() * puck_r * 0.7,
                        );
                        hoa_painter.line_segment([center, puck_pos], Stroke::new(1.2_f32, Color32::from_rgb(200, 120, 255)));
                        hoa_painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(220, 160, 255));
                        hoa_painter.circle_stroke(puck_pos, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(157, 78, 221)));

                        // Readouts
                        hoa_painter.text(
                            egui::pos2(hoa_rect.right() - 6.0, hoa_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("HOA5 (36-CH)\naz: {:.0}° el: {:.0}°\nd: {:.1}m | rE: {:.2}", az, el, dist, focus),
                            FontId::proportional(8.0),
                            Color32::from_rgb(180, 140, 240),
                        );

                        if hoa_resp.hovered() {
                            let _ = hoa_resp.on_hover_text(format!(
                                "5th-Order Ambisonics HOA5 HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.1}m\nEnergy Vector Focus: {:.2}\n[Drag: Azimuth (X) vs Elevation (Y)]",
                                az, el, dist, focus
                            ));
                        }
                    } else if is_mpegh_3d {
                        // Broadcast Mastering Immersive MPEG-H 3D Audio Object Spatializer Mini Visualizer
                        let (mp_resp, mp_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mp_rect = mp_resp.rect;
                        mp_painter.rect_filled(mp_rect, 2.0, Color32::from_rgb(8, 24, 20));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(35.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(20.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.8);
                        let spread = state.node_param_values.get("spread_divergence_pct").copied().unwrap_or(30.0);

                        if mp_resp.dragged() {
                            if let Some(pos) = mp_resp.interact_pointer_pos() {
                                let nx = ((pos.x - mp_rect.left()) / mp_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - mp_rect.top()) / mp_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                el = -90.0 + ny * 180.0;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, el); }
                                }
                            }
                        }

                        // 7.1.4 Bed speaker perimeter dots
                        let center = egui::pos2(mp_rect.left() + 38.0, mp_rect.center().y);
                        mp_painter.circle_stroke(
                            center,
                            26.0,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 50)),
                        );
                        // Speaker positions (L, R, C, Ls, Rs, Lb, Rb)
                        let spk_angles = [-30.0_f32, 30.0, 0.0, -110.0, 110.0, -150.0, 150.0];
                        for ang in spk_angles {
                            let rad = ang.to_radians();
                            let spk_p = egui::pos2(center.x + rad.sin() * 26.0, center.y - rad.cos() * 26.0 * 0.7);
                            mp_painter.circle_filled(spk_p, 1.8, Color32::from_rgb(0, 255, 180));
                        }

                        // Object position puck & spread aura
                        let az_rad = az.to_radians();
                        let el_rad = el.to_radians();
                        let obj_r = (el_rad.cos() * 24.0).clamp(2.0, 26.0);
                        let obj_pos = egui::pos2(
                            center.x + az_rad.sin() * obj_r,
                            center.y - az_rad.cos() * obj_r * 0.7,
                        );
                        let spread_r = 4.0 + (spread / 100.0) * 8.0;
                        mp_painter.circle_stroke(obj_pos, spread_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 80)));
                        mp_painter.line_segment([center, obj_pos], Stroke::new(1.2_f32, Color32::from_rgb(0, 255, 180)));
                        mp_painter.circle_filled(obj_pos, 3.8, Color32::from_rgb(180, 255, 220));

                        // Readouts
                        mp_painter.text(
                            egui::pos2(mp_rect.right() - 6.0, mp_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("MPEG-H 3D\naz: {:.0}° el: {:.0}°\nd: {:.1}m | div: {:.0}%", az, el, dist, spread),
                            FontId::proportional(8.0),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if mp_resp.hovered() {
                            let _ = mp_resp.on_hover_text(format!(
                                "MPEG-H 3D Audio Spatializer HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.1}m\nSpread Divergence: {:.0}%\n[Drag: Azimuth (X) vs Elevation (Y)]",
                                az, el, dist, spread
                            ));
                        }
                    } else if is_atmos_916 {
                        // Broadcast Mastering Dolby Atmos 9.1.6 3D Room Raytracing Mini Visualizer
                        let (at_resp, at_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let at_rect = at_resp.rect;
                        at_painter.rect_filled(at_rect, 2.0, Color32::from_rgb(10, 16, 26));

                        let mut sx = state.node_param_values.get("source_x_m").copied().unwrap_or(1.5);
                        let mut sy = state.node_param_values.get("source_y_m").copied().unwrap_or(2.0);
                        let sz = state.node_param_values.get("source_z_m").copied().unwrap_or(1.2);
                        let rt60 = state.node_param_values.get("rt60_decay_s").copied().unwrap_or(0.35);

                        if at_resp.dragged() {
                            if let Some(pos) = at_resp.interact_pointer_pos() {
                                let nx = ((pos.x - at_rect.left()) / at_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - at_rect.top()) / at_rect.height())).clamp(0.0, 1.0);
                                sx = -8.0 + nx * 16.0;
                                sy = -8.0 + ny * 16.0;
                                let dist = (sx * sx + sy * sy + sz * sz).sqrt().max(0.01);
                                let az = sx.atan2(sy).to_degrees();
                                let el = (sz / dist).clamp(-1.0, 1.0).asin().to_degrees();

                                state.node_param_values.insert("source_x_m".to_string(), sx);
                                state.node_param_values.insert("source_y_m".to_string(), sy);
                                state.node_param_values.insert("azimuth".to_string(), az);
                                state.node_param_values.insert("elevation".to_string(), el);
                                state.node_param_values.insert("distance".to_string(), dist);

                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, sx); }
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, sy); }
                                }
                            }
                        }

                        // Room Stage Boundary Box (Stage footprint)
                        let room_min = egui::pos2(at_rect.left() + 10.0, at_rect.top() + 8.0);
                        let room_max = egui::pos2(at_rect.left() + 74.0, at_rect.bottom() - 8.0);
                        let room_rect = egui::Rect::from_min_max(room_min, room_max);
                        at_painter.rect_stroke(room_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 65, 95)));

                        // 9.1.6 Bed speaker dots on stage perimeter
                        let spk_pts = [
                            room_rect.left_top(),
                            room_rect.center_top(),
                            room_rect.right_top(),
                            room_rect.left_center(),
                            room_rect.right_center(),
                            room_rect.left_bottom(),
                            room_rect.center_bottom(),
                            room_rect.right_bottom(),
                        ];
                        for &sp in &spk_pts {
                            at_painter.circle_filled(sp, 1.6, Color32::from_rgb(0, 229, 255));
                        }

                        // Object source puck inside stage
                        let p_norm_x = ((sx + 8.0) / 16.0).clamp(0.0, 1.0);
                        let p_norm_y = (1.0 - (sy + 8.0) / 16.0).clamp(0.0, 1.0);
                        let puck_pos = egui::pos2(
                            room_rect.left() + p_norm_x * room_rect.width(),
                            room_rect.top() + p_norm_y * room_rect.height(),
                        );

                        // Raytrace rays radiating to room boundaries
                        for sp in [&room_rect.left_top(), &room_rect.right_top(), &room_rect.right_bottom(), &room_rect.left_bottom()] {
                            at_painter.line_segment([puck_pos, *sp], Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 50)));
                        }
                        at_painter.circle_filled(puck_pos, 3.8, Color32::from_rgb(0, 255, 180));

                        // Readouts
                        at_painter.text(
                            egui::pos2(at_rect.right() - 6.0, at_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("ATMOS (9.1.6)\nx: {:.1}m y: {:.1}m\nRT60: {:.2}s | 128 rays", sx, sy, rt60),
                            FontId::proportional(8.0),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if at_resp.hovered() {
                            let _ = at_resp.on_hover_text(format!(
                                "Dolby Atmos 9.1.6 3D Room Raytracer HUD\nX: {:.2}m | Y: {:.2}m | Z: {:.2}m\nRT60: {:.2}s\n[Drag: Stage Position (X vs Y)]",
                                sx, sy, sz, rt60
                            ));
                        }
                    } else if is_auro3d {
                        // Broadcast Mastering Immersive Auro-3D 13.1 Tri-Level Height Spatializer Mini Visualizer
                        let (au_resp, au_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let au_rect = au_resp.rect;
                        au_painter.rect_filled(au_rect, 2.0, Color32::from_rgb(18, 16, 26));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(35.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(28.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.8);
                        let delay = state.node_param_values.get("height_layer_delay_ms").copied().unwrap_or(12.5);

                        if au_resp.dragged() {
                            if let Some(pos) = au_resp.interact_pointer_pos() {
                                let nx = ((pos.x - au_rect.left()) / au_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - au_rect.top()) / au_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                el = -30.0 + ny * 120.0;

                                let bed = if el <= 0.0 { 1.0 } else if el <= 30.0 { (1.0 - el / 30.0).powi(2) } else { 0.0 };
                                let hgt = if el <= 0.0 { 0.0 } else if el <= 30.0 { (el / 30.0 * (2.0 - el / 30.0)).clamp(0.0, 1.0) } else { (1.0 - (el - 30.0) / 60.0).powi(2) };
                                let top = if el <= 30.0 { 0.0 } else { ((el - 30.0) / 60.0 * (2.0 - (el - 30.0) / 60.0)).clamp(0.0, 1.0) };

                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                state.node_param_values.insert("lower_layer".to_string(), bed);
                                state.node_param_values.insert("height_layer".to_string(), hgt);
                                state.node_param_values.insert("top_voice".to_string(), top);

                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, el); }
                                }
                            }
                        }

                        // Tri-Level Concentric Rings (Center on left half)
                        let center = egui::pos2(au_rect.left() + 38.0, au_rect.center().y);
                        // Layer 1: Lower Bed (outer ring)
                        au_painter.circle_stroke(center, 26.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(101, 84, 171, 70)));
                        // Layer 2: Height Layer (middle ring)
                        au_painter.circle_stroke(center, 17.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 171, 0, 90)));
                        // Layer 3: Top Voice of God (center dot / small ring)
                        au_painter.circle_stroke(center, 7.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 110)));

                        // Azimuth / Elevation Vector Puck
                        let az_rad = az.to_radians();
                        let r_proj = ((1.0 - (el + 30.0) / 120.0) * 24.0).clamp(3.0, 26.0);
                        let puck_pos = egui::pos2(
                            center.x + az_rad.sin() * r_proj,
                            center.y - az_rad.cos() * r_proj * 0.75,
                        );
                        au_painter.line_segment([center, puck_pos], Stroke::new(1.2_f32, Color32::from_rgb(255, 171, 0)));
                        au_painter.circle_filled(puck_pos, 3.8, Color32::from_rgb(255, 215, 0));

                        // Readouts
                        au_painter.text(
                            egui::pos2(au_rect.right() - 6.0, au_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("AURO-3D 13.1\naz: {:.0}° el: {:.0}°\nd: {:.1}m | del: {:.1}ms", az, el, dist, delay),
                            FontId::proportional(8.0),
                            Color32::from_rgb(255, 171, 0),
                        );

                        if au_resp.hovered() {
                            let _ = au_resp.on_hover_text(format!(
                                "Auro-3D 13.1 Tri-Level Spatializer HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.1}m\nHeight Delay: {:.1}ms\n[Drag: Azimuth (X) vs Elevation (Y)]",
                                az, el, dist, delay
                            ));
                        }
                    } else if is_nhk222 {
                        // Broadcast Mastering Immersive NHK 22.2 Hemispherical Loudspeaker Dome Mini Visualizer
                        let (nhk_resp, nhk_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let nhk_rect = nhk_resp.rect;
                        nhk_painter.rect_filled(nhk_rect, 2.0, Color32::from_rgb(10, 18, 28));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(25.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(3.5);
                        let spread = state.node_param_values.get("spread_pct").copied().unwrap_or(35.0);

                        if nhk_resp.dragged() {
                            if let Some(pos) = nhk_resp.interact_pointer_pos() {
                                let nx = ((pos.x - nhk_rect.left()) / nhk_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - nhk_rect.top()) / nhk_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                el = -30.0 + ny * 120.0;

                                let top = if el <= 0.0 { 0.0 } else { (el / 90.0).clamp(0.0, 1.0) };
                                let mid = if el < 0.0 { ((el + 30.0) / 30.0).clamp(0.0, 1.0) } else { (1.0 - el / 60.0).clamp(0.0, 1.0) };
                                let bot = if el < 0.0 { (-el / 30.0).clamp(0.0, 1.0) } else { 0.0 };
                                let lfe = 0.20_f32;

                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                state.node_param_values.insert("top_gain".to_string(), top);
                                state.node_param_values.insert("mid_gain".to_string(), mid);
                                state.node_param_values.insert("bottom_gain".to_string(), bot);
                                state.node_param_values.insert("lfe_sub".to_string(), lfe);

                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, el); }
                                }
                            }
                        }

                        // NHK 22.2 Dome Concentric Rings (Center on left half)
                        let center = egui::pos2(nhk_rect.left() + 38.0, nhk_rect.center().y);
                        // Middle Layer (10ch, outer ear-level ring)
                        nhk_painter.circle_stroke(center, 26.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 70)));
                        // Top Layer (9ch, middle elevation ring)
                        nhk_painter.circle_stroke(center, 16.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(101, 84, 171, 90)));
                        // Top Voice of God center ceiling dot
                        nhk_painter.circle_filled(center, 2.0, Color32::from_rgba_unmultiplied(0, 255, 180, 140));

                        // 10 Middle-layer speaker dots around perimeter
                        for spk_i in 0..10 {
                            let spk_ang = spk_i as f32 * std::f32::consts::TAU / 10.0;
                            let spk_pos = egui::pos2(center.x + spk_ang.sin() * 26.0, center.y - spk_ang.cos() * 26.0 * 0.75);
                            nhk_painter.circle_filled(spk_pos, 1.4, Color32::from_rgb(0, 229, 255));
                        }

                        // Azimuth / Elevation Vector Puck
                        let az_rad = az.to_radians();
                        let r_proj = ((1.0 - (el + 30.0) / 120.0) * 24.0).clamp(2.0, 26.0);
                        let puck_pos = egui::pos2(
                            center.x + az_rad.sin() * r_proj,
                            center.y - az_rad.cos() * r_proj * 0.75,
                        );
                        nhk_painter.line_segment([center, puck_pos], Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                        nhk_painter.circle_filled(puck_pos, 3.8, Color32::from_rgb(0, 255, 200));

                        // Readouts
                        nhk_painter.text(
                            egui::pos2(nhk_rect.right() - 6.0, nhk_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NHK 22.2 DOME\naz: {:.0}° el: {:.0}°\nd: {:.1}m | spr: {:.0}%", az, el, dist, spread),
                            FontId::proportional(8.0),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if nhk_resp.hovered() {
                            let _ = nhk_resp.on_hover_text(format!(
                                "NHK 22.2 Super Hi-Vision Hemispherical Spatializer HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.1}m\nSpread: {:.0}%\n[Drag: Azimuth (X) vs Elevation (Y)]",
                                az, el, dist, spread
                            ));
                        }
                    } else if is_wfs_array {
                        // Broadcast Mastering Wave Field Synthesis (WFS) Holographic Linear Acoustic Array Mini Visualizer
                        let (wfs_resp, wfs_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let wfs_rect = wfs_resp.rect;
                        wfs_painter.rect_filled(wfs_rect, 2.0, Color32::from_rgb(10, 22, 18));

                        let mut sx = state.node_param_values.get("source_x_m").copied().unwrap_or(0.0);
                        let mut sy = state.node_param_values.get("source_y_m").copied().unwrap_or(3.5);
                        let ch_count = state.node_param_values.get("array_elements_count").copied().unwrap_or(64.0);
                        let alias_hz = state.node_param_values.get("spatial_aliasing_cut").copied().unwrap_or(2200.0);

                        if wfs_resp.dragged() {
                            if let Some(pos) = wfs_resp.interact_pointer_pos() {
                                let nx = ((pos.x - wfs_rect.left()) / wfs_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - wfs_rect.top()) / wfs_rect.height())).clamp(0.0, 1.0);
                                sx = -8.0 + nx * 16.0;
                                sy = -5.0 + ny * 15.0;

                                state.node_param_values.insert("source_x_m".to_string(), sx);
                                state.node_param_values.insert("source_y_m".to_string(), sy);
                                state.node_param_values.insert("focused_source_depth_m".to_string(), sy);

                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, sx); }
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, sy); }
                                }
                            }
                        }

                        // Linear Transducer Array Bar (drawn across left half)
                        let array_y = wfs_rect.top() + 38.0;
                        let array_x_min = wfs_rect.left() + 10.0;
                        let array_x_max = wfs_rect.left() + 72.0;
                        wfs_painter.line_segment(
                            [egui::pos2(array_x_min, array_y), egui::pos2(array_x_max, array_y)],
                            Stroke::new(1.8_f32, Color32::from_rgb(0, 255, 180)),
                        );

                        // 8 Sampled Transducer Dots along array line
                        for i in 0..8 {
                            let frac = i as f32 / 7.0;
                            let dot_x = array_x_min + frac * (array_x_max - array_x_min);
                            wfs_painter.circle_filled(egui::pos2(dot_x, array_y), 1.5, Color32::from_rgb(0, 255, 220));
                        }

                        // Virtual Source Puck
                        let p_norm_x = ((sx + 8.0) / 16.0).clamp(0.0, 1.0);
                        let p_norm_y = ((sy + 5.0) / 15.0).clamp(0.0, 1.0);
                        let puck_pos = egui::pos2(
                            array_x_min + p_norm_x * (array_x_max - array_x_min),
                            wfs_rect.top() + 8.0 + (1.0 - p_norm_y) * (wfs_rect.height() - 16.0),
                        );

                        // Wavefront Curved Propagation Arc
                        let is_focused = sy < 0.0;
                        let wave_col = if is_focused {
                            Color32::from_rgba_unmultiplied(255, 171, 0, 70)
                        } else {
                            Color32::from_rgba_unmultiplied(0, 255, 180, 60)
                        };
                        wfs_painter.circle_stroke(puck_pos, 14.0, Stroke::new(1.0_f32, wave_col));
                        wfs_painter.circle_stroke(puck_pos, 22.0, Stroke::new(0.8_f32, wave_col));

                        // Virtual Source Indicator
                        let puck_col = if is_focused { Color32::from_rgb(255, 171, 0) } else { Color32::from_rgb(0, 255, 180) };
                        wfs_painter.circle_filled(puck_pos, 3.8, puck_col);

                        // Readouts
                        wfs_painter.text(
                            egui::pos2(wfs_rect.right() - 6.0, wfs_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("WFS ARRAY ({:.0}-CH)\nx: {:.1}m y: {:.1}m\ncut: {:.0}Hz | {}", ch_count, sx, sy, alias_hz, if is_focused { "FOCUSED" } else { "SYNTH" }),
                            FontId::proportional(8.0),
                            puck_col,
                        );

                        if wfs_resp.hovered() {
                            let _ = wfs_resp.on_hover_text(format!(
                                "Wave Field Synthesis (WFS) Acoustic Array HUD\nVirtual X: {:.2}m | Depth Y: {:.2}m\nChannels: {:.0} | Cutoff: {:.0}Hz\nFocus: {}\n[Drag: Virtual Source (X vs Depth Y)]",
                                sx, sy, ch_count, alias_hz, if is_focused { "FOCUSED (In front)" } else { "BEHIND ARRAY" }
                            ));
                        }
                    } else if is_binaural_panner {
                        // Spatial Binaural HRTF 3D Orbit Mini Visualizer
                        let (bin_resp, bin_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let bin_rect = bin_resp.rect;
                        bin_painter.rect_filled(bin_rect, 2.0, Color32::from_rgb(10, 16, 26));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(1.5);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(10.0);
                        let crossfeed = state.node_param_values.get("crossfeed_pct").copied().unwrap_or(65.0);

                        if bin_resp.dragged() {
                            if let Some(pos) = bin_resp.interact_pointer_pos() {
                                let nx = ((pos.x - bin_rect.left()) / bin_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - bin_rect.top()) / bin_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                dist = 0.1 + ny * 9.9;

                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("distance_m".to_string(), dist);

                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    if bus.get(pid_dist).is_some() { bus.set(pid_dist, dist); }
                                }
                            }
                        }

                        // Head model in center of left region
                        let center = egui::pos2(bin_rect.left() + 38.0, bin_rect.center().y);
                        bin_painter.circle_filled(center, 9.0, Color32::from_rgb(28, 42, 65));
                        bin_painter.circle_stroke(center, 9.0, Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)));
                        // Left & Right ears
                        bin_painter.circle_filled(egui::pos2(center.x - 10.5, center.y), 2.2, Color32::from_rgb(0, 229, 255));
                        bin_painter.circle_filled(egui::pos2(center.x + 10.5, center.y), 2.2, Color32::from_rgb(0, 229, 255));
                        // Distance orbit circle
                        bin_painter.circle_stroke(center, 24.0, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)));

                        // Sound source puck
                        let az_rad = (az - 90.0).to_radians();
                        let d_norm = ((dist - 0.1) / 9.9).clamp(0.0, 1.0);
                        let r_px = 12.0 + d_norm * 14.0;
                        let puck_pos = egui::pos2(center.x + az_rad.cos() * r_px, center.y + az_rad.sin() * r_px);
                        bin_painter.line_segment([center, puck_pos], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)));
                        bin_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(0, 255, 200));

                        // Calculate Woodworth ITD & ILD cues
                        let (itd, ild) = crate::views::binaural_panner_view::BinauralPannerView::calculate_interaural_cues(az, el, dist);

                        // Readouts
                        bin_painter.text(
                            egui::pos2(bin_rect.right() - 6.0, bin_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("HRTF 3D ORBIT\naz: {:.0}° el: {:.0}°\nd: {:.1}m | crs: {:.0}%\nITD: {:.0}µs | ILD: {:.1}dB", az, el, dist, crossfeed, itd, ild),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if bin_resp.hovered() {
                            let _ = bin_resp.on_hover_text(format!(
                                "Spatial Binaural HRTF 3D Orbit Spatializer HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.2}m\nITD: {:.0} µs | ILD: {:.1} dB\nCrossfeed: {:.0}%\n[Drag: Azimuth (X) vs Distance (Y)]",
                                az, el, dist, itd, ild, crossfeed
                            ));
                        }
                    } else if is_binaural_brir {
                        // Multi-Source Binaural BRIR Impulse Reflectogram Mini Visualizer
                        let (brir_resp, brir_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let brir_rect = brir_resp.rect;
                        brir_painter.rect_filled(brir_rect, 2.0, Color32::from_rgb(10, 24, 20));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(0.0);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.5);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(0.0);
                        let notch_hz = state.node_param_values.get("pinna_notch_hz").copied().unwrap_or(7200.0);

                        if brir_resp.dragged() {
                            if let Some(pos) = brir_resp.interact_pointer_pos() {
                                let nx = ((pos.x - brir_rect.left()) / (brir_rect.width() * 0.5)).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - brir_rect.top()) / brir_rect.height())).clamp(0.0, 1.0);
                                az = -180.0 + nx * 360.0;
                                dist = 0.2 + ny * 9.8;

                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("distance_m".to_string(), dist);

                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, az); }
                                    if bus.get(pid_dist).is_some() { bus.set(pid_dist, dist); }
                                }
                            }
                        }

                        // Left region: Polar Sound Source Radar
                        let center = egui::pos2(brir_rect.left() + 28.0, brir_rect.center().y);
                        brir_painter.circle_stroke(center, 22.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 70)));
                        brir_painter.circle_filled(center, 2.5, Color32::from_rgb(0, 255, 200));

                        let az_rad = (az - 90.0).to_radians();
                        let d_norm = ((dist - 0.2) / 9.8).clamp(0.0, 1.0);
                        let r_px = 6.0 + d_norm * 14.0;
                        let puck_pos = egui::pos2(center.x + az_rad.cos() * r_px, center.y + az_rad.sin() * r_px);
                        brir_painter.circle_filled(puck_pos, 3.2, Color32::from_rgb(0, 255, 180));

                        // Right region: 6 BRIR Impulse Reflection Stem Bars
                        let bar_x_start = brir_rect.left() + 64.0;
                        let bar_w = 4.0;
                        let bar_gap = 4.5;
                        let base_y = brir_rect.bottom() - 10.0;
                        for i in 0..6 {
                            let delay_factor = (i as f32 * 10.0 + 8.0) * (dist / 3.0);
                            let amp = (-0.03 * delay_factor).exp() * (0.85 / ((i as f32 + 1.0).sqrt()));
                            let bar_h = (amp * 40.0).clamp(3.0, 44.0);
                            let bx = bar_x_start + i as f32 * (bar_w + bar_gap);
                            let bar_rect = egui::Rect::from_min_size(egui::pos2(bx, base_y - bar_h), egui::vec2(bar_w, bar_h));
                            brir_painter.rect_filled(bar_rect, 1.0, Color32::from_rgba_unmultiplied(0, 255, 180, 180));
                        }

                        // Readouts
                        brir_painter.text(
                            egui::pos2(brir_rect.right() - 4.0, brir_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("BINAURAL BRIR\naz: {:.0}° el: {:.0}°\nd: {:.1}m | notch: {:.0}Hz\n6 Early Reflections", az, el, dist, notch_hz),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if brir_resp.hovered() {
                            let _ = brir_resp.on_hover_text(format!(
                                "Multi-Source Binaural Room Impulse Response (BRIR) Spatializer HUD\nAzimuth: {:.1}° | Elevation: {:.1}° | Distance: {:.2}m\nPinna Notch: {:.0} Hz | Early Reflections: 6 Rays\n[Drag: Polar Azimuth (X) vs Distance (Y)]",
                                az, el, dist, notch_hz
                            ));
                        }
                    } else if is_continuous_doppler {
                        // Continuous 3D Doppler Kinematic Radar Mini Visualizer
                        let (dop_resp, dop_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let dop_rect = dop_resp.rect;
                        dop_painter.rect_filled(dop_rect, 2.0, Color32::from_rgb(6, 12, 22));

                        let mut pos_x = state.node_param_values.get("pos_x").copied().unwrap_or(0.0);
                        let mut pos_y = state.node_param_values.get("pos_y").copied().unwrap_or(3.5);
                        let pos_z = state.node_param_values.get("pos_z").copied().unwrap_or(1.0);
                        let speed_of_sound = state.node_param_values.get("speed_of_sound").copied().unwrap_or(343.0);

                        if dop_resp.dragged() {
                            if let Some(pos) = dop_resp.interact_pointer_pos() {
                                let nx = ((pos.x - dop_rect.left() - 34.0) / 26.0).clamp(-1.0, 1.0);
                                let ny = ((dop_rect.center().y - pos.y) / 26.0).clamp(-1.0, 1.0);
                                pos_x = nx * 25.0;
                                pos_y = ny * 25.0;
                                state.node_param_values.insert("pos_x".to_string(), pos_x);
                                state.node_param_values.insert("pos_y".to_string(), pos_y);
                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_x, pos_x);
                                    bus.set(pid_y, pos_y);
                                }
                            }
                        }

                        // Concentric range circles
                        let dcx = dop_rect.left() + 34.0;
                        let dcy = dop_rect.center().y;
                        dop_painter.circle_stroke(egui::pos2(dcx, dcy), 10.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 50)));
                        dop_painter.circle_stroke(egui::pos2(dcx, dcy), 22.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 50)));
                        dop_painter.circle_filled(egui::pos2(dcx, dcy), 2.5, Color32::from_rgb(56, 189, 248));

                        // Sound source position
                        let px = dcx + (pos_x / 25.0).clamp(-1.0, 1.0) * 24.0;
                        let py = dcy - (pos_y / 25.0).clamp(-1.0, 1.0) * 24.0;
                        dop_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 255, 200));

                        // Calculate Doppler kinematics
                        let (v_rad, _ratio, cents) = crate::views::continuous_doppler_view::ContinuousDopplerView::calculate_doppler_ratio(
                            pos_x, pos_y, pos_z, 12.0, -8.0, 0.0, speed_of_sound
                        );
                        let dist = (pos_x * pos_x + pos_y * pos_y + pos_z * pos_z).sqrt().max(0.1);

                        // Readout text
                        dop_painter.text(
                            egui::pos2(dop_rect.right() - 4.0, dop_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("DOPPLER 3D\nd: {:.1}m | c: {:.0}m/s\nVrad: {:+.1} m/s\nShift: {:+.1} cents", dist, speed_of_sound, v_rad, cents),
                            FontId::proportional(7.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if dop_resp.hovered() {
                            let _ = dop_resp.on_hover_text(format!(
                                "Continuous 3D Kinematic Spatializer & Doppler Engine HUD\nPosition: ({:.1}, {:.1}, {:.1}) m | Distance: {:.2} m\nRadial Velocity: {:+.1} m/s | Doppler Shift: {:+.1} cents\n[Drag: Orbit Position X/Y]",
                                pos_x, pos_y, pos_z, dist, v_rad, cents
                            ));
                        }
                    } else if is_spatial_room {
                        // 3D Shoebox Room Acoustics & Image-Source Reverb Mini Visualizer
                        let (rm_resp, rm_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let rm_rect = rm_resp.rect;
                        rm_painter.rect_filled(rm_rect, 2.0, Color32::from_rgb(6, 12, 22));

                        let room_w = state.node_param_values.get("room_width").copied().unwrap_or(8.0);
                        let room_d = state.node_param_values.get("room_depth").copied().unwrap_or(12.0);
                        let room_h = state.node_param_values.get("room_height").copied().unwrap_or(3.5);
                        let mut src_x = state.node_param_values.get("source_x").copied().unwrap_or(3.0);
                        let mut src_y = state.node_param_values.get("source_y").copied().unwrap_or(4.0);
                        let absorption = state.node_param_values.get("wall_absorption").copied().unwrap_or(0.35);

                        // Shoebox mini box
                        let box_rect = egui::Rect::from_min_size(egui::pos2(rm_rect.left() + 6.0, rm_rect.top() + 6.0), egui::vec2(56.0, 56.0));
                        rm_painter.rect_filled(box_rect, 1.0, Color32::from_rgb(10, 18, 30));
                        rm_painter.rect_stroke(box_rect, 1.0, Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)));

                        if rm_resp.dragged() {
                            if let Some(pos) = rm_resp.interact_pointer_pos() {
                                let nx = ((pos.x - box_rect.left()) / box_rect.width()).clamp(0.05, 0.95);
                                let ny = ((box_rect.bottom() - pos.y) / box_rect.height()).clamp(0.05, 0.95);
                                src_x = nx * room_w;
                                src_y = ny * room_d;
                                state.node_param_values.insert("source_x".to_string(), src_x);
                                state.node_param_values.insert("source_y".to_string(), src_y);
                                if let Some(bus) = param_bus {
                                    let pid_sx = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    let pid_sy = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 4);
                                    bus.set(pid_sx, src_x);
                                    bus.set(pid_sy, src_y);
                                }
                            }
                        }

                        // Source (S) and Listener (L)
                        let spx = box_rect.left() + (src_x / room_w.max(1.0)).clamp(0.05, 0.95) * box_rect.width();
                        let spy = box_rect.bottom() - (src_y / room_d.max(1.0)).clamp(0.05, 0.95) * box_rect.height();
                        let lpx = box_rect.left() + 0.5 * box_rect.width();
                        let lpy = box_rect.top() + 0.3 * box_rect.height();

                        // Direct ray line
                        rm_painter.line_segment([egui::pos2(spx, spy), egui::pos2(lpx, lpy)], Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255)));
                        // Source point
                        rm_painter.circle_filled(egui::pos2(spx, spy), 3.0, Color32::from_rgb(239, 68, 68));
                        // Listener point
                        rm_painter.circle_filled(egui::pos2(lpx, lpy), 3.0, Color32::from_rgb(56, 189, 248));

                        let rt60 = crate::views::spatial_room_view::SpatialRoomView::calculate_sabine_rt60(room_w, room_d, room_h, absorption);
                        let dist = ((src_x - 4.0).powi(2) + (src_y - 8.0).powi(2)).sqrt().max(0.1);

                        // Readout
                        rm_painter.text(
                            egui::pos2(rm_rect.right() - 4.0, rm_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("SHOEBOX ROOM\n{}x{}x{}m\nRT60: {:.2}s\nSrc: ({:.1},{:.1})m", room_w as i32, room_d as i32, room_h as i32, rt60, src_x, src_y),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if rm_resp.hovered() {
                            let _ = rm_resp.on_hover_text(format!(
                                "3D Shoebox Room Acoustics & 6-Wall Early Reflection Reverb HUD\nDimensions: {:.1}m x {:.1}m x {:.1}m | Sabine RT60: {:.2} s\nSource: ({:.1}, {:.1}) m | Direct Dist: {:.2} m\n[Drag: Room Floor Source X/Y]",
                                room_w, room_d, room_h, rt60, src_x, src_y, dist
                            ));
                        }
                    } else if is_rotary_doppler {
                        // Stereo Rotary Speaker Doppler & Mic Placement Mini Visualizer
                        let (rot_resp, rot_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let rot_rect = rot_resp.rect;
                        rot_painter.rect_filled(rot_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let horn_spd = state.node_param_values.get("horn_speed_rpm").copied().unwrap_or(380.0);
                        let drum_spd = state.node_param_values.get("drum_speed_rpm").copied().unwrap_or(340.0);
                        let mut mic_ang = state.node_param_values.get("mic_angle_deg").copied().unwrap_or(90.0);
                        let mut mic_dist = state.node_param_values.get("mic_distance_m").copied().unwrap_or(0.6);

                        let rcx = rot_rect.left() + 34.0;
                        let rcy = rot_rect.center().y;

                        if rot_resp.dragged() {
                            if let Some(pos) = rot_resp.interact_pointer_pos() {
                                let dx = pos.x - rcx;
                                let dy = rcy - pos.y;
                                let ang = (dy.atan2(dx).to_degrees() * 2.0).clamp(0.0, 180.0);
                                let dist = ((dx * dx + dy * dy).sqrt() / 24.0 * 1.9 + 0.1).clamp(0.1, 2.0);
                                mic_ang = ang;
                                mic_dist = dist;
                                state.node_param_values.insert("mic_angle_deg".to_string(), mic_ang);
                                state.node_param_values.insert("mic_distance_m".to_string(), mic_dist);
                                if let Some(bus) = param_bus {
                                    let pid_ang = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    bus.set(pid_ang, mic_ang);
                                    bus.set(pid_dist, mic_dist);
                                }
                            }
                        }

                        // Horn orbit & drum orbit
                        rot_painter.circle_stroke(egui::pos2(rcx, rcy), 18.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)));
                        rot_painter.circle_stroke(egui::pos2(rcx, rcy), 11.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 60)));
                        rot_painter.circle_filled(egui::pos2(rcx, rcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // Mic positions
                        let mic_rad = (mic_ang * 0.5).to_radians();
                        let mr = 18.0 + mic_dist * 6.0;
                        let mlx = rcx - mic_rad.sin() * mr;
                        let mly = rcy - mic_rad.cos() * mr;
                        let mrx = rcx + mic_rad.sin() * mr;
                        let mry = rcy - mic_rad.cos() * mr;
                        rot_painter.circle_filled(egui::pos2(mlx, mly), 2.5, Color32::from_rgb(0, 255, 180));
                        rot_painter.circle_filled(egui::pos2(mrx, mry), 2.5, Color32::from_rgb(0, 255, 180));

                        // Doppler cues
                        let speed_factor = horn_spd / 400.0;
                        let dist_factor = (1.0 / mic_dist.max(0.1)).min(3.0);
                        let dop_c = 25.0 * speed_factor * dist_factor;

                        // Readout
                        rot_painter.text(
                            egui::pos2(rot_rect.right() - 4.0, rot_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("ROTARY DOPPLER\nHorn: {:.0} RPM\nDrum: {:.0} RPM\nShift: ±{:.1} c\nDist: {:.2}m", horn_spd, drum_spd, dop_c, mic_dist),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if rot_resp.hovered() {
                            let _ = rot_resp.on_hover_text(format!(
                                "Stereo Rotary Speaker Doppler Acceleration & Mic Placement HUD\nHorn: {:.0} RPM | Drum: {:.0} RPM\nMic Angle: {:.0}° | Mic Distance: {:.2}m | Shift: ±{:.1} cents\n[Drag: Mic Angle / Distance]",
                                horn_spd, drum_spd, mic_ang, mic_dist, dop_c
                            ));
                        }
                    } else if is_raytraced_reverb {
                        // Acoustic Early Reflections Raytracer Mini Visualizer
                        let (ray_resp, ray_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let ray_rect = ray_resp.rect;
                        ray_painter.rect_filled(ray_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let r_len = state.node_param_values.get("room_length").copied().unwrap_or(12.0);
                        let r_wid = state.node_param_values.get("room_width").copied().unwrap_or(8.0);
                        let r_hgt = state.node_param_values.get("room_height").copied().unwrap_or(3.5);
                        let mut s_x = state.node_param_values.get("source_x").copied().unwrap_or(0.3);
                        let mut s_y = state.node_param_values.get("source_y").copied().unwrap_or(0.3);
                        let l_x = state.node_param_values.get("listener_x").copied().unwrap_or(0.65);
                        let l_y = state.node_param_values.get("listener_y").copied().unwrap_or(0.5);

                        let box_r = egui::Rect::from_min_size(egui::pos2(ray_rect.left() + 6.0, ray_rect.top() + 6.0), egui::vec2(56.0, 56.0));
                        ray_painter.rect_filled(box_r, 1.0, Color32::from_rgb(12, 16, 26));
                        ray_painter.rect_stroke(box_r, 1.0, Stroke::new(1.0_f32, Color32::from_rgb(70, 95, 135)));

                        if ray_resp.dragged() {
                            if let Some(pos) = ray_resp.interact_pointer_pos() {
                                let nx = ((pos.x - box_r.left()) / box_r.width()).clamp(0.05, 0.95);
                                let ny = ((box_r.bottom() - pos.y) / box_r.height()).clamp(0.05, 0.95);
                                s_x = nx;
                                s_y = ny;
                                state.node_param_values.insert("source_x".to_string(), s_x);
                                state.node_param_values.insert("source_y".to_string(), s_y);
                                if let Some(bus) = param_bus {
                                    let pid_sx = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    let pid_sy = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 4);
                                    bus.set(pid_sx, s_x);
                                    bus.set(pid_sy, s_y);
                                }
                            }
                        }

                        let spx = box_r.left() + s_x.clamp(0.05, 0.95) * box_r.width();
                        let spy = box_r.bottom() - s_y.clamp(0.05, 0.95) * box_r.height();
                        let lpx = box_r.left() + l_x.clamp(0.05, 0.95) * box_r.width();
                        let lpy = box_r.bottom() - l_y.clamp(0.05, 0.95) * box_r.height();

                        // Direct path
                        ray_painter.line_segment([egui::pos2(spx, spy), egui::pos2(lpx, lpy)], Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)));
                        // Top bounce ray
                        let top_y = box_r.top();
                        let bounce_x = (spx + lpx) * 0.5;
                        ray_painter.line_segment([egui::pos2(spx, spy), egui::pos2(bounce_x, top_y)], Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 0)));
                        ray_painter.line_segment([egui::pos2(bounce_x, top_y), egui::pos2(lpx, lpy)], Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 0)));

                        // Source (red) & Listener (cyan)
                        ray_painter.circle_filled(egui::pos2(spx, spy), 3.0, Color32::from_rgb(239, 68, 68));
                        ray_painter.circle_filled(egui::pos2(lpx, lpy), 3.0, Color32::from_rgb(56, 189, 248));

                        let vol = r_len * r_wid * r_hgt;
                        let s_area = 2.0 * (r_len * r_wid + r_len * r_hgt + r_wid * r_hgt);
                        let rt60 = (0.161 * vol) / (s_area * 0.15).max(0.01);

                        // Readout
                        ray_painter.text(
                            egui::pos2(ray_rect.right() - 4.0, ray_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("RAYTRACER\nRT60: {:.2} s\nDim: {:.0}x{:.0}m\nEarly Rays: 4\nSource: S/L", rt60, r_len, r_wid),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if ray_resp.hovered() {
                            let _ = ray_resp.on_hover_text(format!(
                                "Acoustic Early Reflections Raytracer & Binaural Distance HUD\nRoom: {:.1}m x {:.1}m x {:.1}m | RT60: {:.2}s\nSource ({:.2}, {:.2}) | Listener ({:.2}, {:.2})\n[Drag: Source Position]",
                                r_len, r_wid, r_hgt, rt60, s_x, s_y, l_x, l_y
                            ));
                        }
                    } else if is_atmos_surround {
                        // 7.1.4 Dolby Atmos Immersive Bed & Object Radar Mini Visualizer
                        let (atm_resp, atm_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let atm_rect = atm_resp.rect;
                        atm_painter.rect_filled(atm_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let mut obj_x = state.node_param_values.get("surround_azimuth").map(|v| v / 180.0).or_else(|| state.node_param_values.get("object_x").copied()).unwrap_or(0.35);
                        let mut obj_y = state.node_param_values.get("surround_elevation").map(|v| v / 90.0).or_else(|| state.node_param_values.get("object_y").copied()).unwrap_or(0.55);
                        let obj_z = state.node_param_values.get("height_elevation").map(|v| v / 90.0).or_else(|| state.node_param_values.get("object_z_height").copied()).unwrap_or(0.40);
                        let spread = state.node_param_values.get("divergence_spread").map(|v| v * 100.0).or_else(|| state.node_param_values.get("object_size_spread").copied()).unwrap_or(25.0);
                        let lfe_gain = state.node_param_values.get("lfe_send_level").copied().or_else(|| state.node_param_values.get("lfe_send_gain_db").copied()).unwrap_or(-12.0);

                        let radar_cx = atm_rect.left() + 34.0;
                        let radar_cy = atm_rect.center().y;
                        let radar_r = 24.0_f32;

                        if atm_resp.dragged() {
                            if let Some(pos) = atm_resp.interact_pointer_pos() {
                                let nx = ((pos.x - radar_cx) / radar_r).clamp(-1.0, 1.0);
                                let ny = ((radar_cy - pos.y) / radar_r).clamp(-1.0, 1.0);
                                obj_x = nx;
                                obj_y = ny;
                                state.node_param_values.insert("object_x".to_string(), obj_x);
                                state.node_param_values.insert("object_y".to_string(), obj_y);
                                state.node_param_values.insert("surround_azimuth".to_string(), obj_x * 180.0);
                                state.node_param_values.insert("surround_elevation".to_string(), obj_y * 90.0);
                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_x, obj_x);
                                    bus.set(pid_y, obj_y);
                                }
                            }
                        }

                        // Boundary ring & listener center
                        atm_painter.circle_stroke(egui::pos2(radar_cx, radar_cy), radar_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)));
                        atm_painter.circle_stroke(egui::pos2(radar_cx, radar_cy), radar_r * 0.5, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 30)));
                        atm_painter.circle_filled(egui::pos2(radar_cx, radar_cy), 2.5, Color32::from_rgb(0, 229, 255));

                        // 7.1.4 Speaker markers (L, C, R, Ls, Rs, Lr, Rr)
                        let spk_angles = [-30.0_f32, 0.0, 30.0, -90.0, 90.0, -135.0, 135.0];
                        for deg in spk_angles {
                            let rad = (deg - 90.0).to_radians();
                            let sx = radar_cx + rad.cos() * (radar_r + 2.0);
                            let sy = radar_cy + rad.sin() * (radar_r + 2.0);
                            atm_painter.circle_filled(egui::pos2(sx, sy), 1.5, Color32::from_rgb(56, 189, 248));
                        }

                        // Object puck and spread diffusion circle
                        let opx = radar_cx + obj_x.clamp(-1.0, 1.0) * radar_r;
                        let opy = radar_cy - obj_y.clamp(-1.0, 1.0) * radar_r;
                        let spr_r = (spread * 0.01 * 8.0).max(2.0);
                        atm_painter.circle_stroke(egui::pos2(opx, opy), spr_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 180, 0, 120)));
                        atm_painter.circle_filled(egui::pos2(opx, opy), 3.0, Color32::from_rgb(255, 180, 0));

                        // Readout
                        atm_painter.text(
                            egui::pos2(atm_rect.right() - 4.0, atm_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("ATMOS 7.1.4\nX: {:.2} Y: {:.2}\nZ: {:.2} Spr: {:.0}%\nLFE: {:.1} dB", obj_x, obj_y, obj_z, spread, lfe_gain),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if atm_resp.hovered() {
                            let _ = atm_resp.on_hover_text(format!(
                                "7.1.4 Dolby Atmos Immersive Bed & Object Radar HUD\nPan X: {:.2} | Depth Y: {:.2} | Height Z: {:.2}\nSpread: {:.0}% | LFE: {:.1} dB\n[Drag: Pan / Depth Position]",
                                obj_x, obj_y, obj_z, spread, lfe_gain
                            ));
                        }
                    } else if is_atmos_proximity {
                        // 7.1.4 Dolby Atmos Spherical Proximity & Distance Mini Visualizer
                        let (prx_resp, prx_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let prx_rect = prx_resp.rect;
                        prx_painter.rect_filled(prx_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.5);
                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(30.0);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(15.0);
                        let bass = state.node_param_values.get("proximity_bass_boost_db").copied().unwrap_or(3.5);
                        let air = state.node_param_values.get("air_absorption_loss_db").copied().unwrap_or(2.0);

                        let pcx = prx_rect.left() + 34.0;
                        let pcy = prx_rect.center().y;
                        let max_r = 24.0_f32;

                        if prx_resp.dragged() {
                            if let Some(pos) = prx_resp.interact_pointer_pos() {
                                let dx = pos.x - pcx;
                                let dy = pcy - pos.y;
                                let d = ((dx * dx + dy * dy).sqrt() / max_r * 49.9 + 0.1).clamp(0.1, 50.0);
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                dist = d;
                                az = a;
                                state.node_param_values.insert("distance_m".to_string(), dist);
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                if let Some(bus) = param_bus {
                                    let pid_d = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_a = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_d, dist);
                                    bus.set(pid_a, az);
                                }
                            }
                        }

                        // Polar concentric distance rings
                        for r_factor in [0.33_f32, 0.66, 1.0] {
                            prx_painter.circle_stroke(
                                egui::pos2(pcx, pcy),
                                max_r * r_factor,
                                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 180, 0, 40)),
                            );
                        }
                        // Listener head dot
                        prx_painter.circle_filled(egui::pos2(pcx, pcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // Sound object position on polar grid
                        let rad = az.to_radians();
                        let d_norm = (dist / 50.0).clamp(0.1, 1.0);
                        let spx = pcx + rad.sin() * d_norm * max_r;
                        let spy = pcy - rad.cos() * d_norm * max_r;

                        // Ray connecting listener to source
                        prx_painter.line_segment([egui::pos2(pcx, pcy), egui::pos2(spx, spy)], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 180, 0, 100)));
                        prx_painter.circle_filled(egui::pos2(spx, spy), 3.0, Color32::from_rgb(255, 180, 0));

                        // Readout
                        prx_painter.text(
                            egui::pos2(prx_rect.right() - 4.0, prx_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("ATMOS PROX\nDist: {:.1}m\nAz: {:.0}° El: {:.0}°\nBass: +{:.1}dB\nAir: -{:.1}dB", dist, az, el, bass, air),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 180, 0),
                        );

                        if prx_resp.hovered() {
                            let _ = prx_resp.on_hover_text(format!(
                                "7.1.4 Dolby Atmos Spherical Proximity HUD\nDistance: {:.1}m | Azimuth: {:.0}° | Elevation: {:.0}°\nProximity Bass Boost: +{:.1} dB | Air Absorption: -{:.1} dB\n[Drag: Distance / Azimuth Position]",
                                dist, az, el, bass, air
                            ));
                        }
                    } else if is_mpegh_trajectory {
                        // MPEG-H 3D Dynamic Object Trajectory Mini Visualizer
                        let (traj_resp, traj_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let traj_rect = traj_resp.rect;
                        traj_painter.rect_filled(traj_rect, 2.0, Color32::from_rgb(8, 14, 24));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(35.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(4.5);
                        let spd = state.node_param_values.get("trajectory_speed_hz").copied().unwrap_or(0.25);
                        let spread = state.node_param_values.get("object_spread").copied().unwrap_or(0.30);

                        let tcx = traj_rect.left() + 34.0;
                        let tcy = traj_rect.center().y;
                        let max_r = 24.0_f32;

                        if traj_resp.dragged() {
                            if let Some(pos) = traj_resp.interact_pointer_pos() {
                                let dx = pos.x - tcx;
                                let dy = tcy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let e = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 180.0 - 90.0).clamp(-90.0, 90.0);
                                az = a;
                                el = e;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_az, az);
                                    bus.set(pid_el, el);
                                }
                            }
                        }

                        // Orbit bounding rings & crosshair
                        traj_painter.circle_stroke(egui::pos2(tcx, tcy), max_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 50)));
                        traj_painter.circle_stroke(egui::pos2(tcx, tcy), max_r * 0.5, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 25)));
                        traj_painter.circle_filled(egui::pos2(tcx, tcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // 8 speaker dots
                        for i in 0..8 {
                            let spk_ang = (-180.0 + i as f32 * 45.0).to_radians();
                            let sx = tcx + spk_ang.sin() * (max_r + 2.0);
                            let sy = tcy - spk_ang.cos() * (max_r + 2.0);
                            traj_painter.circle_filled(egui::pos2(sx, sy), 1.5, Color32::from_rgb(56, 189, 248));
                        }

                        // Sound object puck position
                        let az_rad = az.to_radians();
                        let el_norm = ((el + 90.0) / 180.0).clamp(0.1, 1.0);
                        let opx = tcx + az_rad.sin() * el_norm * max_r;
                        let opy = tcy - az_rad.cos() * el_norm * max_r;
                        let spr_r = (spread * 6.0).max(2.0);
                        traj_painter.circle_stroke(egui::pos2(opx, opy), spr_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 180, 0, 100)));
                        traj_painter.circle_filled(egui::pos2(opx, opy), 3.0, Color32::from_rgb(255, 180, 0));

                        // Readout
                        traj_painter.text(
                            egui::pos2(traj_rect.right() - 4.0, traj_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("MPEG-H 3D\nAz: {:.0}° El: {:.0}°\nDist: {:.1}m Spr: {:.0}%\nSpd: {:.2} Hz", az, el, dist, spread * 100.0, spd),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if traj_resp.hovered() {
                            let _ = traj_resp.on_hover_text(format!(
                                "MPEG-H 3D Dynamic Object Trajectory HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nSpread: {:.0}% | Trajectory Speed: {:.2} Hz\n[Drag: Azimuth / Elevation Trajectory]",
                                az, el, dist, spread * 100.0, spd
                            ));
                        }
                    } else if is_nhk222_immersion {
                        // NHK 22.2 3-Layer Spherical Immersion Mini Visualizer
                        let (nhk_resp, nhk_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let nhk_rect = nhk_resp.rect;
                        nhk_painter.rect_filled(nhk_rect, 2.0, Color32::from_rgb(8, 20, 18));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(0.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(35.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(8.0);
                        let top = state.node_param_values.get("top_layer_energy").copied().unwrap_or(0.85);
                        let mid = state.node_param_values.get("middle_layer_energy").copied().unwrap_or(0.90);
                        let bot = state.node_param_values.get("bottom_layer_energy").copied().unwrap_or(0.40);
                        let lfe = state.node_param_values.get("lfe_energy").copied().unwrap_or(0.60);

                        let ncx = nhk_rect.left() + 34.0;
                        let ncy = nhk_rect.center().y;
                        let max_r = 24.0_f32;

                        if nhk_resp.dragged() {
                            if let Some(pos) = nhk_resp.interact_pointer_pos() {
                                let dx = pos.x - ncx;
                                let dy = ncy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let e = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 180.0 - 90.0).clamp(-90.0, 90.0);
                                az = a;
                                el = e;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_az, az);
                                    bus.set(pid_el, el);
                                }
                            }
                        }

                        // 3 Concentric Hemisphere layer arcs (Top, Mid, Bottom)
                        nhk_painter.circle_stroke(egui::pos2(ncx, ncy), max_r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 60)));
                        nhk_painter.circle_stroke(egui::pos2(ncx, ncy), max_r * 0.66, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 35)));
                        nhk_painter.circle_stroke(egui::pos2(ncx, ncy), max_r * 0.33, Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 20)));
                        nhk_painter.circle_filled(egui::pos2(ncx, ncy), 2.5, Color32::from_rgb(0, 255, 180));

                        // 22.2 speaker dots around perimeter
                        for i in 0..12 {
                            let spk_ang = (-180.0 + i as f32 * 30.0).to_radians();
                            let sx = ncx + spk_ang.sin() * (max_r + 2.0);
                            let sy = ncy - spk_ang.cos() * (max_r + 2.0);
                            nhk_painter.circle_filled(egui::pos2(sx, sy), 1.2, Color32::from_rgb(52, 211, 153));
                        }

                        // Listener / Object puck position
                        let az_rad = az.to_radians();
                        let el_norm = ((el + 90.0) / 180.0).clamp(0.1, 1.0);
                        let npx = ncx + az_rad.sin() * el_norm * max_r;
                        let npy = ncy - az_rad.cos() * el_norm * max_r;
                        nhk_painter.circle_filled(egui::pos2(npx, npy), 3.0, Color32::from_rgb(255, 180, 0));

                        // Readout
                        nhk_painter.text(
                            egui::pos2(nhk_rect.right() - 4.0, nhk_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NHK 22.2\nTop: {:.0}% Mid: {:.0}%\nBot: {:.0}% LFE: {:.0}%\nDist: {:.1}m", top * 100.0, mid * 100.0, bot * 100.0, lfe * 100.0, dist),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if nhk_resp.hovered() {
                            let _ = nhk_resp.on_hover_text(format!(
                                "NHK 22.2 3-Layer Spherical Immersion HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nTop Energy: {:.0}% | Mid Energy: {:.0}% | Bot: {:.0}% | LFE: {:.0}%\n[Drag: Azimuth / Elevation Immersion]",
                                az, el, dist, top * 100.0, mid * 100.0, bot * 100.0, lfe * 100.0
                            ));
                        }
                    } else if is_hoa4_spatializer {
                        // HOA 4th-Order (25-ch) 3D Ambisonic Energy Density Mini Visualizer
                        let (hoa_resp, hoa_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let hoa_rect = hoa_resp.rect;
                        hoa_painter.rect_filled(hoa_rect, 2.0, Color32::from_rgb(8, 16, 26));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(25.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(3.0);
                        let order = state.node_param_values.get("energy_focus_order").copied().unwrap_or(4.0);
                        let oct0 = state.node_param_values.get("octant_energy_0").copied().unwrap_or(0.85);
                        let oct1 = state.node_param_values.get("octant_energy_1").copied().unwrap_or(0.65);

                        let hcx = hoa_rect.left() + 34.0;
                        let hcy = hoa_rect.center().y;
                        let max_r = 24.0_f32;

                        if hoa_resp.dragged() {
                            if let Some(pos) = hoa_resp.interact_pointer_pos() {
                                let dx = pos.x - hcx;
                                let dy = hcy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let e = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 180.0 - 90.0).clamp(-90.0, 90.0);
                                az = a;
                                el = e;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_az, az);
                                    bus.set(pid_el, el);
                                }
                            }
                        }

                        // 4 Concentric Ambisonic order spheres (N=1, 2, 3, 4)
                        for n in 1..=4 {
                            let r_frac = n as f32 / 4.0;
                            hoa_painter.circle_stroke(
                                egui::pos2(hcx, hcy),
                                max_r * r_frac,
                                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 229, 255, (25 + n * 20) as u8)),
                            );
                        }
                        hoa_painter.circle_filled(egui::pos2(hcx, hcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // 25 spherical harmonic representation dots around perimeter
                        for i in 0..16 {
                            let ang = (i as f32 * 22.5).to_radians();
                            let sx = hcx + ang.sin() * (max_r + 1.5);
                            let sy = hcy - ang.cos() * (max_r + 1.5);
                            hoa_painter.circle_filled(egui::pos2(sx, sy), 1.0, Color32::from_rgb(56, 189, 248));
                        }

                        // Ambisonic 4th-Order Energy Puck position
                        let az_rad = az.to_radians();
                        let el_norm = ((el + 90.0) / 180.0).clamp(0.1, 1.0);
                        let hpx = hcx + az_rad.sin() * el_norm * max_r;
                        let hpy = hcy - az_rad.cos() * el_norm * max_r;
                        hoa_painter.circle_filled(egui::pos2(hpx, hpy), 3.2, Color32::from_rgb(255, 200, 50));

                        // Readout
                        hoa_painter.text(
                            egui::pos2(hoa_rect.right() - 4.0, hoa_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("HOA4 25-CH\nOrder: {:.1} | Dist: {:.1}m\nOct0: {:.0}% Oct1: {:.0}%\nAz: {:.0}° El: {:.0}°", order, dist, oct0 * 100.0, oct1 * 100.0, az, el),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if hoa_resp.hovered() {
                            let _ = hoa_resp.on_hover_text(format!(
                                "HOA 4th-Order (25-ch) 3D Ambisonic Energy Density HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nEnergy Focus Order: {:.1} | Octant0: {:.0}% | Octant1: {:.0}%\n[Drag: Azimuth / Elevation Ambisonic Energy]",
                                az, el, dist, order, oct0 * 100.0, oct1 * 100.0
                            ));
                        }
                    } else if is_mpegh_spatializer {
                        // MPEG-H 3D Spatial Audio & Personalized HRTF Profile Mini Visualizer
                        let (mpg_resp, mpg_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let mpg_rect = mpg_resp.rect;
                        mpg_painter.rect_filled(mpg_rect, 2.0, Color32::from_rgb(8, 22, 20));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(0.0);
                        let mut el = state.node_param_values.get("elevation_deg").copied().unwrap_or(0.0);
                        let dist = state.node_param_values.get("distance_m").copied().unwrap_or(1.5);
                        let itd = state.node_param_values.get("itd_microseconds").copied().unwrap_or(0.0);
                        let ild = state.node_param_values.get("ild_db").copied().unwrap_or(0.0);
                        let lufs = state.node_param_values.get("integrated_lufs").copied().unwrap_or(-24.0);

                        let mcx = mpg_rect.left() + 34.0;
                        let mcy = mpg_rect.center().y;
                        let max_r = 24.0_f32;

                        if mpg_resp.dragged() {
                            if let Some(pos) = mpg_resp.interact_pointer_pos() {
                                let dx = pos.x - mcx;
                                let dy = mcy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let e = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 180.0 - 90.0).clamp(-90.0, 90.0);
                                az = a;
                                el = e;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("elevation_deg".to_string(), el);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_az, az);
                                    bus.set(pid_el, el);
                                }
                            }
                        }

                        // Spatial Stage Perimeter & Listener Ears
                        mpg_painter.circle_stroke(
                            egui::pos2(mcx, mcy),
                            max_r,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 50)),
                        );
                        mpg_painter.circle_stroke(
                            egui::pos2(mcx, mcy),
                            max_r * 0.5,
                            Stroke::new(0.7_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 30)),
                        );
                        // Listener head & ears
                        mpg_painter.circle_filled(egui::pos2(mcx, mcy), 3.0, Color32::from_rgb(0, 255, 180));
                        mpg_painter.circle_filled(egui::pos2(mcx - 4.5, mcy), 1.5, Color32::from_rgb(52, 211, 153)); // L ear
                        mpg_painter.circle_filled(egui::pos2(mcx + 4.5, mcy), 1.5, Color32::from_rgb(52, 211, 153)); // R ear

                        // Speaker bed perimeter dots (7.1.4 layout positions)
                        for spk_a in [-30.0, 30.0, 0.0, -90.0, 90.0, -135.0, 135.0_f32] {
                            let spk_rad = spk_a.to_radians();
                            let sx = mcx + spk_rad.sin() * max_r;
                            let sy = mcy - spk_rad.cos() * max_r;
                            mpg_painter.circle_filled(egui::pos2(sx, sy), 1.3, Color32::from_rgb(52, 211, 153));
                        }

                        // Object puck
                        let az_rad = az.to_radians();
                        let dist_norm = (dist / 10.0).clamp(0.15, 1.0);
                        let opx = mcx + az_rad.sin() * dist_norm * max_r;
                        let opy = mcy - az_rad.cos() * dist_norm * max_r;
                        mpg_painter.circle_filled(egui::pos2(opx, opy), 3.0, Color32::from_rgb(255, 180, 0));

                        // Readout
                        mpg_painter.text(
                            egui::pos2(mpg_rect.right() - 4.0, mpg_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("MPEG-H 3D\nITD: {:.0}μs | ILD: {:.1}dB\nLUFS: {:.1} | Dist: {:.1}m\nAz: {:.0}° El: {:.0}°", itd, ild, lufs, dist, az, el),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if mpg_resp.hovered() {
                            let _ = mpg_resp.on_hover_text(format!(
                                "MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nITD: {:.0} μs | ILD: {:.1} dB | Loudness: {:.1} LUFS\n[Drag: Azimuth / Distance Object Spatialization]",
                                az, el, dist, itd, ild, lufs
                            ));
                        }
                    } else if is_spatial_trajectory {
                        // 3D Spatial Trajectory & Granular Cloud Density Mini Visualizer
                        let (spat_resp, spat_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let spat_rect = spat_resp.rect;
                        spat_painter.rect_filled(spat_rect, 2.0, Color32::from_rgb(8, 16, 26));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(0.0);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(0.0);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(4.0);
                        let spread = state.node_param_values.get("spread").copied().unwrap_or(0.25);
                        let density = state.node_param_values.get("grain_density").copied().unwrap_or(45.0);
                        let duration = state.node_param_values.get("grain_duration_ms").copied().unwrap_or(60.0);

                        let scx = spat_rect.left() + 34.0;
                        let scy = spat_rect.center().y;
                        let max_r = 24.0_f32;

                        if spat_resp.dragged() {
                            if let Some(pos) = spat_resp.interact_pointer_pos() {
                                let dx = pos.x - scx;
                                let dy = scy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let d = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 10.0).max(0.5);
                                az = a;
                                dist = d;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("distance_m".to_string(), dist);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    bus.set(pid_az, az);
                                    bus.set(pid_dist, dist);
                                }
                            }
                        }

                        // Polar rings
                        for r_step in [0.33, 0.66, 1.0] {
                            spat_painter.circle_stroke(
                                egui::pos2(scx, scy),
                                max_r * r_step,
                                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(40, 55, 80, 140)),
                            );
                        }
                        // Center listener
                        spat_painter.circle_filled(egui::pos2(scx, scy), 2.5, Color32::from_rgb(0, 229, 255));

                        // Trajectory orbit preview ribbon
                        for step in 0..12 {
                            let ang = (step as f32 / 12.0) * std::f32::consts::TAU;
                            let rx = scx + ang.sin() * max_r * 0.7;
                            let ry = scy - ang.cos() * max_r * 0.7;
                            spat_painter.circle_filled(egui::pos2(rx, ry), 0.8, Color32::from_rgb(0, 229, 255));
                        }

                        // Simulated granular particle dots around puck
                        for i in 0..8 {
                            let t = i as f32 / 8.0;
                            let jx = (t * 19.3).sin() * 5.0;
                            let jy = (t * 31.7).cos() * 5.0;
                            let norm_r = (dist / 10.0).clamp(0.05, 1.0) * max_r;
                            let px = scx + az.to_radians().sin() * norm_r + jx;
                            let py = scy - az.to_radians().cos() * norm_r + jy;
                            spat_painter.circle_filled(egui::pos2(px, py), 1.2, Color32::from_rgba_unmultiplied(0, 255, 180, 180));
                        }

                        // Source Puck
                        let norm_r = (dist / 10.0).clamp(0.05, 1.0) * max_r;
                        let spx = scx + az.to_radians().sin() * norm_r;
                        let spy = scy - az.to_radians().cos() * norm_r;
                        spat_painter.circle_filled(egui::pos2(spx, spy), 3.5, Color32::from_rgb(255, 215, 0));

                        // Readout
                        spat_painter.text(
                            egui::pos2(spat_rect.right() - 4.0, spat_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("TRAJECTORY 3D\nDist: {:.1}m | Az: {:.0}°\nEl: {:.0}° | Sprd: {:.0}%\nGrains: {:.0} | Dur: {:.0}ms", dist, az, el, spread * 100.0, density, duration),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if spat_resp.hovered() {
                            let _ = spat_resp.on_hover_text(format!(
                                "3D Spatial Trajectory & Granular Cloud HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nSpread: {:.0}% | Grain Density: {:.0} | Duration: {:.0}ms\n[Drag: Azimuth / Distance Position Puck]",
                                az, el, dist, spread * 100.0, density, duration
                            ));
                        }
                    } else if is_ambisonic_radar {
                        // 3D Ambisonic Trajectory Radar Mini Visualizer
                        let (rad_resp, rad_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let rad_rect = rad_resp.rect;
                        rad_painter.rect_filled(rad_rect, 2.0, Color32::from_rgb(8, 24, 20));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(-30.0);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(15.0);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.5);
                        let sprd = state.node_param_values.get("spread_pct").copied().unwrap_or(20.0);
                        let gain = state.node_param_values.get("gain_db").copied().unwrap_or(0.0);
                        let elev_off = state.node_param_values.get("master_elevation_offset_deg").copied().unwrap_or(0.0);

                        let rcx = rad_rect.left() + 34.0;
                        let rcy = rad_rect.center().y;
                        let max_r = 24.0_f32;

                        if rad_resp.dragged() {
                            if let Some(pos) = rad_resp.interact_pointer_pos() {
                                let dx = pos.x - rcx;
                                let dy = rcy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let d = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 5.0).max(0.1);
                                az = a;
                                dist = d;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("distance_m".to_string(), dist);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    bus.set(pid_az, az);
                                    bus.set(pid_dist, dist);
                                }
                            }
                        }

                        // Radar outer ring & crosshairs
                        rad_painter.circle_stroke(
                            egui::pos2(rcx, rcy),
                            max_r,
                            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
                        );
                        rad_painter.circle_stroke(
                            egui::pos2(rcx, rcy),
                            max_r * 0.5,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 100)),
                        );
                        rad_painter.line_segment(
                            [egui::pos2(rcx - max_r, rcy), egui::pos2(rcx + max_r, rcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 70)),
                        );
                        rad_painter.line_segment(
                            [egui::pos2(rcx, rcy - max_r), egui::pos2(rcx, rcy + max_r)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 70)),
                        );

                        // Speaker perimeter markers (8 speakers around radar)
                        for spk in 0..8 {
                            let spk_ang = (spk as f32 * 45.0).to_radians();
                            let sx = rcx + spk_ang.sin() * max_r;
                            let sy = rcy - spk_ang.cos() * max_r;
                            rad_painter.circle_filled(egui::pos2(sx, sy), 1.2, Color32::from_rgb(140, 165, 195));
                        }

                        // Center listener
                        rad_painter.circle_filled(egui::pos2(rcx, rcy), 2.5, Color32::from_rgb(0, 255, 180));

                        // Sound Object Puck
                        let norm_r = (dist / 5.0).clamp(0.05, 1.0) * max_r;
                        let rpx = rcx + az.to_radians().sin() * norm_r;
                        let rpy = rcy - az.to_radians().cos() * norm_r;
                        rad_painter.circle_filled(egui::pos2(rpx, rpy), 3.2, Color32::from_rgb(255, 215, 0));
                        rad_painter.circle_stroke(
                            egui::pos2(rpx, rpy),
                            5.0,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 120)),
                        );

                        // Readout
                        rad_painter.text(
                            egui::pos2(rad_rect.right() - 4.0, rad_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("AMBISONIC RADAR\nDist: {:.1}m | Az: {:.0}°\nEl: {:.0}° | Sprd: {:.0}%\nGain: {:+.1}dB | Off: {:+.0}°", dist, az, el, sprd, gain, elev_off),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if rad_resp.hovered() {
                            let _ = rad_resp.on_hover_text(format!(
                                "3D Ambisonic Trajectory Radar HUD\nAzimuth: {:.0}° | Elevation: {:.0}° | Distance: {:.1}m\nSpread: {:.0}% | Gain: {:+.1}dB | Master Offset: {:+.0}°\n[Drag: Azimuth / Distance Object Puck]",
                                az, el, dist, sprd, gain, elev_off
                            ));
                        }
                    } else if is_hoa_spatializer {
                        // Dolby Atmos HOA 7.1.4 3D Spatializer Mini Visualizer
                        let (hoa_resp, hoa_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let hoa_rect = hoa_resp.rect;
                        hoa_painter.rect_filled(hoa_rect, 2.0, Color32::from_rgb(10, 16, 26));

                        let mut az = state.node_param_values.get("azimuth_deg").copied().unwrap_or(45.0);
                        let el = state.node_param_values.get("elevation_deg").copied().unwrap_or(18.5);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(2.4);
                        let yaw = state.node_param_values.get("head_yaw_deg").copied().unwrap_or(-12.4);

                        let hcx = hoa_rect.left() + 34.0;
                        let hcy = hoa_rect.center().y;
                        let max_r = 24.0_f32;

                        if hoa_resp.dragged() {
                            if let Some(pos) = hoa_resp.interact_pointer_pos() {
                                let dx = pos.x - hcx;
                                let dy = hcy - pos.y;
                                let a = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let d = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 10.0).max(0.1);
                                az = a;
                                dist = d;
                                state.node_param_values.insert("azimuth_deg".to_string(), az);
                                state.node_param_values.insert("distance_m".to_string(), dist);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    bus.set(pid_az, az);
                                    bus.set(pid_dist, dist);
                                }
                            }
                        }

                        // Concentric rings & crosshairs
                        for r_step in [0.33, 0.66, 1.0] {
                            hoa_painter.circle_stroke(
                                egui::pos2(hcx, hcy),
                                max_r * r_step,
                                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
                            );
                        }
                        hoa_painter.line_segment(
                            [egui::pos2(hcx - max_r, hcy), egui::pos2(hcx + max_r, hcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 50)),
                        );
                        hoa_painter.line_segment(
                            [egui::pos2(hcx, hcy - max_r), egui::pos2(hcx, hcy + max_r)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 50)),
                        );

                        // Atmos 7.1.4 Speakers
                        for spk_ang_deg in [-30.0_f32, 0.0, 30.0, -90.0, 90.0, -140.0, 140.0] {
                            let spk_ang = spk_ang_deg.to_radians();
                            let sx = hcx + spk_ang.sin() * max_r;
                            let sy = hcy - spk_ang.cos() * max_r;
                            hoa_painter.circle_filled(egui::pos2(sx, sy), 1.2, Color32::from_rgb(160, 185, 215));
                        }

                        // Center listener puck
                        hoa_painter.circle_filled(egui::pos2(hcx, hcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // Spatial Sound Puck
                        let norm_r = (dist / 10.0).clamp(0.05, 1.0) * max_r;
                        let spx = hcx + az.to_radians().sin() * norm_r;
                        let spy = hcy - az.to_radians().cos() * norm_r;
                        hoa_painter.circle_filled(egui::pos2(spx, spy), 3.2, Color32::from_rgb(255, 215, 0));

                        // Harmonic bars preview on right side
                        let bar_x_start = hoa_rect.left() + 72.0;
                        let bar_bottom = hoa_rect.bottom() - 10.0;
                        for b in 0..8 {
                            let h_norm = ((b as f32 * 1.7 + az.abs() * 0.1).sin().abs() * 0.8 + 0.2).clamp(0.1, 1.0);
                            let bh = h_norm * 24.0;
                            let bx = bar_x_start + b as f32 * 4.0;
                            let b_col = if b < 2 { Color32::from_rgb(0, 255, 180) } else if b < 5 { Color32::from_rgb(255, 215, 0) } else { Color32::from_rgb(0, 229, 255) };
                            hoa_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 2.5, bar_bottom)),
                                0.5,
                                b_col,
                            );
                        }

                        // Readout
                        hoa_painter.text(
                            egui::pos2(hoa_rect.right() - 4.0, hoa_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("HOA 7.1.4\nAz: {:+.0}° | El: {:+.0}°\nDist: {:.1}m\nYaw: {:+.0}°", az, el, dist, yaw),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if hoa_resp.hovered() {
                            let _ = hoa_resp.on_hover_text(format!(
                                "Dolby Atmos HOA 7.1.4 3D Spatializer HUD\nAzimuth: {:+.1}° | Elevation: {:+.1}° | Distance: {:.2}m | Head Yaw: {:+.1}°\n[Drag: Azimuth / Distance Position Puck]",
                                az, el, dist, yaw
                            ));
                        }
                    } else if is_neural_timbre_morph {
                        // Neural Timbre Morphing Diffusion Vocoder Mini Visualizer
                        let (timb_resp, timb_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let timb_rect = timb_resp.rect;
                        timb_painter.rect_filled(timb_rect, 2.0, Color32::from_rgb(18, 12, 28));

                        let mut z1 = state.node_param_values.get("latent_z1").copied().unwrap_or(0.45);
                        let mut z2 = state.node_param_values.get("latent_z2").copied().unwrap_or(-0.60);
                        let f0 = state.node_param_values.get("tracked_f0_hz").copied().unwrap_or(220.0);
                        let morph = state.node_param_values.get("morph_weight_pct").copied().unwrap_or(65.0);

                        let tcx = timb_rect.left() + 34.0;
                        let tcy = timb_rect.center().y;
                        let max_box = 24.0_f32;

                        if timb_resp.dragged() {
                            if let Some(pos) = timb_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (tcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (tcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                z1 = -2.0 + norm_x * 4.0;
                                z2 = -2.0 + norm_y * 4.0;
                                state.node_param_values.insert("latent_z1".to_string(), z1);
                                state.node_param_values.insert("latent_z2".to_string(), z2);
                                if let Some(bus) = param_bus {
                                    let pid_z1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_z2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_z1, z1);
                                    bus.set(pid_z2, z2);
                                }
                            }
                        }

                        // Latent grid boundary & axes
                        let latent_rect = egui::Rect::from_center_size(egui::pos2(tcx, tcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        timb_painter.rect_stroke(latent_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(180, 90, 255)));
                        timb_painter.line_segment(
                            [egui::pos2(latent_rect.left(), tcy), egui::pos2(latent_rect.right(), tcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );
                        timb_painter.line_segment(
                            [egui::pos2(tcx, latent_rect.top()), egui::pos2(tcx, latent_rect.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );

                        // Timbre Puck
                        let px = tcx + (z1 / 2.0).clamp(-1.0, 1.0) * max_box;
                        let py = tcy - (z2 / 2.0).clamp(-1.0, 1.0) * max_box;
                        timb_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(255, 105, 180));
                        timb_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 105, 180, 140)),
                        );

                        // 6-bar spectral envelope preview
                        let bar_x_start = timb_rect.left() + 74.0;
                        let bar_bottom = timb_rect.bottom() - 10.0;
                        let env_heights = [0.90_f32, 0.75, 0.60, 0.45, 0.65, 0.25];
                        for (b, base_h) in env_heights.iter().enumerate() {
                            let bh = (*base_h * (morph / 100.0).clamp(0.2, 1.0)) * 26.0;
                            let bx = bar_x_start + b as f32 * 4.5;
                            let col = Color32::from_rgb(180, 90, 255);
                            timb_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 3.0, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        timb_painter.text(
                            egui::pos2(timb_rect.right() - 4.0, timb_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NEURAL TIMBRE\nZ1: {:+.2} | Z2: {:+.2}\nF0: {:.1} Hz\nMorph: {:.0}%", z1, z2, f0, morph),
                            FontId::proportional(7.5),
                            Color32::from_rgb(180, 90, 255),
                        );

                        if timb_resp.hovered() {
                            let _ = timb_resp.on_hover_text(format!(
                                "Neural Timbre Morphing Diffusion Vocoder HUD\nLatent Z1: {:+.2} | Z2: {:+.2} | Tracked F0: {:.1} Hz | Morph: {:.0}%\n[Drag: Latent Timbre Vector Puck]",
                                z1, z2, f0, morph
                            ));
                        }
                    } else if is_neural_radiance {
                        // Neural Continuous Spatial Radiance Field Mini Visualizer
                        let (rad_resp, rad_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let rad_rect = rad_resp.rect;
                        rad_painter.rect_filled(rad_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut head = state.node_param_values.get("heading_deg").copied().unwrap_or(30.0);
                        let mut dist = state.node_param_values.get("distance_m").copied().unwrap_or(3.5);
                        let latent = state.node_param_values.get("latent_dim").copied().unwrap_or(256.0) as usize;
                        let clarity = state.node_param_values.get("directivity_clarity_db").copied().unwrap_or(14.5);

                        let rcx = rad_rect.left() + 34.0;
                        let rcy = rad_rect.center().y;
                        let max_r = 24.0_f32;

                        if rad_resp.dragged() {
                            if let Some(pos) = rad_resp.interact_pointer_pos() {
                                let dx = pos.x - rcx;
                                let dy = rcy - pos.y;
                                let h = (dx.atan2(dy).to_degrees()).clamp(-180.0, 180.0);
                                let d = (((dx * dx + dy * dy).sqrt() / max_r).clamp(0.0, 1.0) * 11.5 + 0.5).clamp(0.5, 12.0);
                                head = h;
                                dist = d;
                                state.node_param_values.insert("heading_deg".to_string(), head);
                                state.node_param_values.insert("distance_m".to_string(), dist);
                                if let Some(bus) = param_bus {
                                    let pid_head = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_dist = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_head, head);
                                    bus.set(pid_dist, dist);
                                }
                            }
                        }

                        // Concentric field rings & crosshairs
                        for r_step in [0.4, 0.7, 1.0] {
                            rad_painter.circle_stroke(
                                egui::pos2(rcx, rcy),
                                max_r * r_step,
                                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 60)),
                            );
                        }
                        rad_painter.line_segment(
                            [egui::pos2(rcx - max_r, rcy), egui::pos2(rcx + max_r, rcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 50)),
                        );
                        rad_painter.line_segment(
                            [egui::pos2(rcx, rcy - max_r), egui::pos2(rcx, rcy + max_r)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 50)),
                        );

                        // 8-direction acoustic radiance rays preview
                        let h_rad = head.to_radians();
                        for i in 0..8 {
                            let ray_angle = i as f32 * (std::f32::consts::TAU / 8.0);
                            let angle_diff = (ray_angle - h_rad).cos();
                            let len_norm = ((1.0 + angle_diff) * 0.5).powf(1.5).clamp(0.15, 1.0);
                            let rx = rcx + ray_angle.sin() * (max_r * len_norm);
                            let ry = rcy - ray_angle.cos() * (max_r * len_norm);
                            rad_painter.line_segment(
                                [egui::pos2(rcx, rcy), egui::pos2(rx, ry)],
                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 120)),
                            );
                        }

                        // Center origin listener
                        rad_painter.circle_filled(egui::pos2(rcx, rcy), 2.5, Color32::from_rgb(0, 229, 255));

                        // Draggable Radiance Puck
                        let norm_r = ((dist - 0.5) / 11.5).clamp(0.05, 1.0) * max_r;
                        let rpx = rcx + head.to_radians().sin() * norm_r;
                        let rpy = rcy - head.to_radians().cos() * norm_r;
                        rad_painter.circle_filled(egui::pos2(rpx, rpy), 3.5, Color32::from_rgb(157, 78, 221));
                        rad_painter.circle_stroke(
                            egui::pos2(rpx, rpy),
                            5.5,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
                        );

                        // Readout
                        rad_painter.text(
                            egui::pos2(rad_rect.right() - 4.0, rad_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NEURAL RADIANCE\nHead: {:+.0}° | Dist: {:.1}m\nLatent: {} | Clar: {:.1}dB", head, dist, latent, clarity),
                            FontId::proportional(7.5),
                            Color32::from_rgb(157, 78, 221),
                        );

                        if rad_resp.hovered() {
                            let _ = rad_resp.on_hover_text(format!(
                                "Neural Acoustic Continuous Spatial Radiance Field HUD\nHeading: {:+.1}° | Distance: {:.2}m | Latent Dim: {}\nDirectivity Clarity: {:.1} dB\n[Drag: Heading / Distance Radiance Puck]",
                                head, dist, latent, clarity
                            ));
                        }
                    } else if is_neural_vocal_stylizer {
                        // Neural Polyphonic Vocal Stylizer Mini Visualizer
                        let (voc_resp, voc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let voc_rect = voc_resp.rect;
                        voc_painter.rect_filled(voc_rect, 2.0, Color32::from_rgb(18, 12, 22));

                        let mut blend = state.node_param_values.get("style_blend_pct").copied().unwrap_or(0.85);
                        let mut ornm = state.node_param_values.get("ornament_depth_pct").copied().unwrap_or(0.45);
                        let v_rate = state.node_param_values.get("vibrato_rate_hz").copied().unwrap_or(5.8);
                        let v_depth = state.node_param_values.get("vibrato_depth_cents").copied().unwrap_or(95.0);
                        let snap = state.node_param_values.get("microtonal_snap_cents").copied().unwrap_or(25.0);

                        let vcx = voc_rect.left() + 34.0;
                        let vcy = voc_rect.center().y;
                        let max_box = 24.0_f32;

                        if voc_resp.dragged() {
                            if let Some(pos) = voc_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (vcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (vcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                blend = norm_x;
                                ornm = norm_y;
                                state.node_param_values.insert("style_blend_pct".to_string(), blend);
                                state.node_param_values.insert("ornament_depth_pct".to_string(), ornm);
                                if let Some(bus) = param_bus {
                                    let pid_blend = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_ornm = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_blend, blend);
                                    bus.set(pid_ornm, ornm);
                                }
                            }
                        }

                        // Expression grid boundary & axes
                        let expr_rect = egui::Rect::from_center_size(egui::pos2(vcx, vcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        voc_painter.rect_stroke(expr_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(255, 93, 143)));
                        voc_painter.line_segment(
                            [egui::pos2(expr_rect.left(), vcy), egui::pos2(expr_rect.right(), vcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 93, 143, 80)),
                        );
                        voc_painter.line_segment(
                            [egui::pos2(vcx, expr_rect.top()), egui::pos2(vcx, expr_rect.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 93, 143, 80)),
                        );

                        // Stylizer Puck
                        let px = expr_rect.left() + blend.clamp(0.0, 1.0) * (max_box * 2.0);
                        let py = expr_rect.bottom() - ornm.clamp(0.0, 1.0) * (max_box * 2.0);
                        voc_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(255, 93, 143));
                        voc_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // 6-bar expression radar preview
                        let bar_x_start = voc_rect.left() + 74.0;
                        let bar_bottom = voc_rect.bottom() - 10.0;
                        let env_heights = [ornm * 0.9, blend * 0.95, blend * 0.85, 0.25, 0.35, 0.88];
                        for (b, base_h) in env_heights.iter().enumerate() {
                            let bh = (base_h.clamp(0.1, 1.0)) * 26.0;
                            let bx = bar_x_start + b as f32 * 4.5;
                            let col = Color32::from_rgb(255, 93, 143);
                            voc_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 3.0, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        voc_painter.text(
                            egui::pos2(voc_rect.right() - 4.0, voc_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("VOCAL STYLIZER\nBlend: {:.0}% | Ornm: {:.0}%\nVib: {:.1}Hz (±{:.0}c)\nSnap: {:.0}c", blend * 100.0, ornm * 100.0, v_rate, v_depth, snap),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 93, 143),
                        );

                        if voc_resp.hovered() {
                            let _ = voc_resp.on_hover_text(format!(
                                "Neural Polyphonic Vocal Stylizer & Microtonal Ornament HUD\nStyle Blend: {:.0}% | Ornament: {:.0}%\nVibrato: {:.1} Hz (±{:.0} cents) | Microtonal Snap: {:.0} cents\n[Drag: Style Blend / Ornament Depth Puck]",
                                blend * 100.0, ornm * 100.0, v_rate, v_depth, snap
                            ));
                        }
                    } else if is_neural_speech_to_singing {
                        // Neural Speech-to-Singing Synthesis Mini Visualizer
                        let (sng_resp, sng_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sng_rect = sng_resp.rect;
                        sng_painter.rect_filled(sng_rect, 2.0, Color32::from_rgb(10, 16, 26));

                        let mut pitch_f0 = state.node_param_values.get("target_pitch_hz").copied().unwrap_or(220.0);
                        let mut vib_cents = state.node_param_values.get("vibrato_depth_cents").copied().unwrap_or(45.0);
                        let v_rate = state.node_param_values.get("vibrato_rate_hz").copied().unwrap_or(5.8);
                        let f_shift = state.node_param_values.get("formant_shift_semitones").copied().unwrap_or(0.0);
                        let conf = state.node_param_values.get("f0_confidence").copied().unwrap_or(0.985);

                        let scx = sng_rect.left() + 34.0;
                        let scy = sng_rect.center().y;
                        let max_box = 24.0_f32;

                        if sng_resp.dragged() {
                            if let Some(pos) = sng_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (scx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (scy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                pitch_f0 = crate::views::neural_speech_to_singing_view::NeuralSpeechToSingingView::normalized_to_pitch(norm_x);
                                vib_cents = crate::views::neural_speech_to_singing_view::NeuralSpeechToSingingView::normalized_to_vibrato(norm_y);
                                state.node_param_values.insert("target_pitch_hz".to_string(), pitch_f0);
                                state.node_param_values.insert("vibrato_depth_cents".to_string(), vib_cents);
                                if let Some(bus) = param_bus {
                                    let pid_pitch = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_vib = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_pitch, pitch_f0);
                                    bus.set(pid_vib, vib_cents);
                                }
                            }
                        }

                        // Retuning coordinate bounding box & axes
                        let pitch_rect = egui::Rect::from_center_size(egui::pos2(scx, scy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        sng_painter.rect_stroke(pitch_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)));
                        sng_painter.line_segment(
                            [egui::pos2(pitch_rect.left(), scy), egui::pos2(pitch_rect.right(), scy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );
                        sng_painter.line_segment(
                            [egui::pos2(scx, pitch_rect.top()), egui::pos2(scx, pitch_rect.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        // Singing Puck
                        let norm_p = crate::views::neural_speech_to_singing_view::NeuralSpeechToSingingView::pitch_to_normalized(pitch_f0);
                        let norm_v = crate::views::neural_speech_to_singing_view::NeuralSpeechToSingingView::vibrato_to_normalized(vib_cents);
                        let px = pitch_rect.left() + norm_p * (max_box * 2.0);
                        let py = pitch_rect.bottom() - norm_v * (max_box * 2.0);
                        sng_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 229, 255));
                        sng_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // 6-bar formant envelope preview
                        let bar_x_start = sng_rect.left() + 74.0;
                        let bar_bottom = sng_rect.bottom() - 10.0;
                        let env_heights: [f32; 6] = [0.95, 0.80, 0.60, 0.90, 0.45, 0.25];
                        for (b, &base_h) in env_heights.iter().enumerate() {
                            let bh = base_h.clamp(0.1_f32, 1.0_f32) * 26.0;
                            let bx = bar_x_start + b as f32 * 4.5;
                            let col = Color32::from_rgb(0, 229, 255);
                            sng_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 3.0, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        sng_painter.text(
                            egui::pos2(sng_rect.right() - 4.0, sng_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("SPEECH2SING\nF0: {:.1}Hz | Vib: {:.0}c\nRate: {:.1}Hz | Shift: {:+.1}st\nConf: {:.0}%", pitch_f0, vib_cents, v_rate, f_shift, conf * 100.0),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if sng_resp.hovered() {
                            let _ = sng_resp.on_hover_text(format!(
                                "Neural Speech-to-Singing Prosody & Intonation Transformer HUD\nTarget F0: {:.1} Hz | Vibrato Depth: {:.0} cents\nVibrato Rate: {:.1} Hz | Formant Shift: {:+.1} semitones\n[Drag: F0 Pitch / Vibrato Depth Singing Puck]",
                                pitch_f0, vib_cents, v_rate, f_shift
                            ));
                        }
                    } else if is_neural_choir_formant {
                        // Neural Polyphonic Choir Formant Morpher Mini Visualizer
                        let (choir_resp, choir_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let choir_rect = choir_resp.rect;
                        choir_painter.rect_filled(choir_rect, 2.0, Color32::from_rgb(18, 16, 10));

                        let mut f1 = state.node_param_values.get("formant_f1_hz").copied().unwrap_or(650.0);
                        let mut f2 = state.node_param_values.get("formant_f2_hz").copied().unwrap_or(1400.0);
                        let v_count = state.node_param_values.get("voice_count").copied().unwrap_or(32.0) as usize;
                        let morph = state.node_param_values.get("neural_morph_blend").copied().unwrap_or(0.88);

                        let ccx = choir_rect.left() + 34.0;
                        let ccy = choir_rect.center().y;
                        let max_box = 24.0_f32;

                        if choir_resp.dragged() {
                            if let Some(pos) = choir_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (ccx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (ccy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                f2 = crate::views::neural_choir_formant_view::NeuralChoirFormantView::normalized_to_f2(norm_x);
                                f1 = crate::views::neural_choir_formant_view::NeuralChoirFormantView::normalized_to_f1(norm_y);
                                state.node_param_values.insert("formant_f1_hz".to_string(), f1);
                                state.node_param_values.insert("formant_f2_hz".to_string(), f2);
                                if let Some(bus) = param_bus {
                                    let pid_f1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_f2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_f1, f1);
                                    bus.set(pid_f2, f2);
                                }
                            }
                        }

                        // Vowel space bounding box & crosshairs
                        let vowel_rect = egui::Rect::from_center_size(egui::pos2(ccx, ccy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        choir_painter.rect_stroke(vowel_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 0)));
                        choir_painter.line_segment(
                            [egui::pos2(vowel_rect.left(), ccy), egui::pos2(vowel_rect.right(), ccy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 80)),
                        );
                        choir_painter.line_segment(
                            [egui::pos2(ccx, vowel_rect.top()), egui::pos2(ccx, vowel_rect.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 80)),
                        );

                        // Choir Puck
                        let norm_f2 = crate::views::neural_choir_formant_view::NeuralChoirFormantView::f2_to_normalized(f2);
                        let norm_f1 = crate::views::neural_choir_formant_view::NeuralChoirFormantView::f1_to_normalized(f1);
                        let px = vowel_rect.left() + norm_f2 * (max_box * 2.0);
                        let py = vowel_rect.bottom() - norm_f1 * (max_box * 2.0);
                        choir_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(255, 215, 0));
                        choir_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // 5-bar voice formant peaks preview
                        let bar_x_start = choir_rect.left() + 74.0;
                        let bar_bottom = choir_rect.bottom() - 10.0;
                        let env_heights: [f32; 5] = [0.90, 0.75, 0.55, 0.40, 0.22];
                        for (b, &base_h) in env_heights.iter().enumerate() {
                            let bh = base_h.clamp(0.1_f32, 1.0_f32) * 26.0;
                            let bx = bar_x_start + b as f32 * 4.5;
                            let col = Color32::from_rgb(255, 215, 0);
                            choir_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 3.0, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        choir_painter.text(
                            egui::pos2(choir_rect.right() - 4.0, choir_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("CHOIR FORMANT\nF1: {:.0}Hz | F2: {:.0}Hz\nVoices: {} | Morph: {:.0}%", f1, f2, v_count, morph * 100.0),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if choir_resp.hovered() {
                            let _ = choir_resp.on_hover_text(format!(
                                "Neural Polyphonic Choir Formant Morpher HUD\nFormant F1: {:.0} Hz | Formant F2: {:.0} Hz\nVoice Count: {} | Morph Blend: {:.0}%\n[Drag: F2 (Frontness) / F1 (Height) Choir Puck]",
                                f1, f2, v_count, morph * 100.0
                            ));
                        }
                    } else if is_neural_vocoder_morph {
                        // Multi-Stage Neural Vocoder Formant Morpher Mini Visualizer
                        let (voc_resp, voc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let voc_rect = voc_resp.rect;
                        voc_painter.rect_filled(voc_rect, 2.0, Color32::from_rgb(10, 18, 28));

                        let mut f1 = state.node_param_values.get("f1_freq_hz").copied().unwrap_or(500.0);
                        let mut f2 = state.node_param_values.get("f2_freq_hz").copied().unwrap_or(1500.0);
                        let artic = state.node_param_values.get("articulation_depth_pct").copied().unwrap_or(82.5);
                        let carrier = state.node_param_values.get("carrier_harmonics_mix").copied().unwrap_or(65.0);

                        let vcx = voc_rect.left() + 34.0;
                        let vcy = voc_rect.center().y;
                        let max_box = 24.0_f32;

                        if voc_resp.dragged() {
                            if let Some(pos) = voc_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (vcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (vcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                f1 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::normalized_to_f1_freq(norm_x);
                                f2 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::normalized_to_f2_freq(norm_y);
                                state.node_param_values.insert("f1_freq_hz".to_string(), f1);
                                state.node_param_values.insert("f2_freq_hz".to_string(), f2);
                                if let Some(bus) = param_bus {
                                    let pid_f1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_f2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_f1, f1);
                                    bus.set(pid_f2, f2);
                                }
                            }
                        }

                        // Vowel formant box & crosshairs
                        let formant_box = egui::Rect::from_center_size(egui::pos2(vcx, vcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        voc_painter.rect_stroke(formant_box, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)));
                        voc_painter.line_segment(
                            [egui::pos2(formant_box.left(), vcy), egui::pos2(formant_box.right(), vcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );
                        voc_painter.line_segment(
                            [egui::pos2(vcx, formant_box.top()), egui::pos2(vcx, formant_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        // Formant Puck
                        let norm_f1 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::f1_freq_to_normalized(f1);
                        let norm_f2 = crate::views::neural_vocoder_morph_view::NeuralVocoderMorphView::f2_freq_to_normalized(f2);
                        let px = formant_box.left() + norm_f1 * (max_box * 2.0);
                        let py = formant_box.bottom() - norm_f2 * (max_box * 2.0);
                        voc_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 229, 255));
                        voc_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // 4-bar formant peaks preview
                        let bar_x_start = voc_rect.left() + 74.0;
                        let bar_bottom = voc_rect.bottom() - 10.0;
                        let env_heights: [f32; 4] = [0.92, 0.78, 0.55, 0.35];
                        for (b, &base_h) in env_heights.iter().enumerate() {
                            let bh = base_h.clamp(0.1_f32, 1.0_f32) * 26.0;
                            let bx = bar_x_start + b as f32 * 5.5;
                            let col = Color32::from_rgb(0, 229, 255);
                            voc_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 3.5, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        voc_painter.text(
                            egui::pos2(voc_rect.right() - 4.0, voc_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("VOCODER MORPH\nF1: {:.0}Hz | F2: {:.0}Hz\nArt: {:.0}% | Carr: {:.0}%", f1, f2, artic, carrier),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if voc_resp.hovered() {
                            let _ = voc_resp.on_hover_text(format!(
                                "Multi-Stage Neural Vocoder Formant Morpher HUD\nFormant F1: {:.0} Hz | Formant F2: {:.0} Hz\nArticulation Depth: {:.1}% | Carrier Harmonics: {:.1}%\n[Drag: F1 (X) / F2 (Y) Formant Puck]",
                                f1, f2, artic, carrier
                            ));
                        }
                    } else if is_neural_wavetable {
                        // Neural Wavetable Morphing Synthesizer Mini Visualizer
                        let (wt_resp, wt_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let wt_rect = wt_resp.rect;
                        wt_painter.rect_filled(wt_rect, 2.0, Color32::from_rgb(18, 12, 28));

                        let mut z1 = state.node_param_values.get("latent_z1").copied().unwrap_or(0.62);
                        let mut z2 = state.node_param_values.get("latent_z2").copied().unwrap_or(-0.45);
                        let speed = state.node_param_values.get("morph_speed_hz").copied().unwrap_or(0.85);
                        let radius = state.node_param_values.get("orbit_radius").copied().unwrap_or(0.45);

                        let wcx = wt_rect.left() + 34.0;
                        let wcy = wt_rect.center().y;
                        let max_box = 24.0_f32;

                        if wt_resp.dragged() {
                            if let Some(pos) = wt_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (wcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (wcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                z1 = crate::views::neural_wavetable_view::NeuralWavetableView::normalized_to_coord(norm_x);
                                z2 = crate::views::neural_wavetable_view::NeuralWavetableView::normalized_to_coord(norm_y);
                                state.node_param_values.insert("latent_z1".to_string(), z1);
                                state.node_param_values.insert("latent_z2".to_string(), z2);
                                if let Some(bus) = param_bus {
                                    let pid_z1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_z2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_z1, z1);
                                    bus.set(pid_z2, z2);
                                }
                            }
                        }

                        // Latent manifold box & crosshairs
                        let latent_box = egui::Rect::from_center_size(egui::pos2(wcx, wcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        wt_painter.rect_stroke(latent_box, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(180, 90, 255)));
                        wt_painter.line_segment(
                            [egui::pos2(latent_box.left(), wcy), egui::pos2(latent_box.right(), wcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );
                        wt_painter.line_segment(
                            [egui::pos2(wcx, latent_box.top()), egui::pos2(wcx, latent_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );

                        // Orbital circle indicator
                        let orbit_r = (radius / 2.0 * max_box).clamp(2.0, max_box - 2.0);
                        wt_painter.circle_stroke(
                            egui::pos2(wcx, wcy),
                            orbit_r,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 100)),
                        );

                        // Latent Puck
                        let norm_z1 = crate::views::neural_wavetable_view::NeuralWavetableView::coord_to_normalized(z1);
                        let norm_z2 = crate::views::neural_wavetable_view::NeuralWavetableView::coord_to_normalized(z2);
                        let px = latent_box.left() + norm_z1 * (max_box * 2.0);
                        let py = latent_box.bottom() - norm_z2 * (max_box * 2.0);
                        wt_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(180, 90, 255));
                        wt_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Wavetable single-cycle wave curve preview
                        let wave_left = wt_rect.left() + 72.0;
                        let wave_w = wt_rect.right() - wave_left - 8.0;
                        let wave_mid_y = wt_rect.center().y + 8.0;
                        let steps = 24;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let phase = s as f32 / steps as f32;
                            let sample = ((phase * std::f32::consts::PI * 2.0).sin() * (0.8 + z1 * 0.1)
                                + (phase * std::f32::consts::PI * 4.0).sin() * (0.4 + z2.abs() * 0.2)).tanh() * 0.85;
                            let cur_pt = egui::pos2(wave_left + phase * wave_w, wave_mid_y - sample * 14.0);
                            if let Some(p) = prev_pt {
                                wt_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(180, 90, 255)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        wt_painter.text(
                            egui::pos2(wt_rect.right() - 4.0, wt_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NEURAL WAVETABLE\nz1: {:.2} | z2: {:.2}\nSpeed: {:.2}Hz | Rad: {:.2}", z1, z2, speed, radius),
                            FontId::proportional(7.5),
                            Color32::from_rgb(180, 90, 255),
                        );

                        if wt_resp.hovered() {
                            let _ = wt_resp.on_hover_text(format!(
                                "Neural Wavetable Morphing Synthesizer HUD\nLatent Coord: ({:.2}, {:.2})\nMorph Speed: {:.2} Hz | Orbit Radius: {:.2}\n[Drag: z1 (X) / z2 (Y) Latent Puck]",
                                z1, z2, speed, radius
                            ));
                        }
                    } else if is_neural_phoneme {
                        // Neural Phoneme Morphing Vocoder Mini Visualizer
                        let (phon_resp, phon_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let phon_rect = phon_resp.rect;
                        phon_painter.rect_filled(phon_rect, 2.0, Color32::from_rgb(16, 12, 24));

                        let mut f1 = state.node_param_values.get("formant_f1_hz").copied().unwrap_or(500.0);
                        let mut f2 = state.node_param_values.get("formant_f2_hz").copied().unwrap_or(1800.0);
                        let tract = state.node_param_values.get("vocal_tract_cm").copied().unwrap_or(17.0);
                        let style_weight = state.node_param_values.get("latent_style_weight").copied().unwrap_or(0.80);

                        let pcx = phon_rect.left() + 34.0;
                        let pcy = phon_rect.center().y;
                        let max_box = 24.0_f32;

                        if phon_resp.dragged() {
                            if let Some(pos) = phon_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (pcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (pcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                f2 = crate::views::neural_phoneme_view::NeuralPhonemeView::normalized_to_f2(norm_x);
                                f1 = crate::views::neural_phoneme_view::NeuralPhonemeView::normalized_to_f1(norm_y);
                                state.node_param_values.insert("formant_f1_hz".to_string(), f1);
                                state.node_param_values.insert("formant_f2_hz".to_string(), f2);
                                if let Some(bus) = param_bus {
                                    let pid_f1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_f2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_f1, f1);
                                    bus.set(pid_f2, f2);
                                }
                            }
                        }

                        // IPA Vowel Quadrilateral box & corner guides
                        let vowel_box = egui::Rect::from_center_size(egui::pos2(pcx, pcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        phon_painter.rect_stroke(vowel_box, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(180, 90, 255)));
                        phon_painter.line_segment(
                            [egui::pos2(vowel_box.left(), pcy), egui::pos2(vowel_box.right(), pcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );
                        phon_painter.line_segment(
                            [egui::pos2(pcx, vowel_box.top()), egui::pos2(pcx, vowel_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 80)),
                        );

                        // Phoneme Puck
                        let norm_f2 = crate::views::neural_phoneme_view::NeuralPhonemeView::f2_to_normalized(f2);
                        let norm_f1 = crate::views::neural_phoneme_view::NeuralPhonemeView::f1_to_normalized(f1);
                        let px = vowel_box.left() + norm_f2 * (max_box * 2.0);
                        let py = vowel_box.bottom() - norm_f1 * (max_box * 2.0);
                        phon_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(180, 90, 255));
                        phon_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // 4 Latent Style Bars preview
                        let bar_left = phon_rect.left() + 72.0;
                        let bar_bottom = phon_rect.center().y + 18.0;
                        for i in 0..4 {
                            let bx = bar_left + i as f32 * 7.0;
                            let bh = ((i as f32 * 0.2 + 0.35) * style_weight * 26.0).clamp(4.0, 26.0);
                            let col = Color32::from_rgb(180, 90, 255);
                            phon_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, bar_bottom - bh), egui::pos2(bx + 4.5, bar_bottom)),
                                0.5,
                                col,
                            );
                        }

                        // Readout
                        phon_painter.text(
                            egui::pos2(phon_rect.right() - 4.0, phon_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("PHONEME MORPH\nF1: {:.0}Hz | F2: {:.0}Hz\nTract: {:.1}cm | Style: {:.0}%", f1, f2, tract, style_weight * 100.0),
                            FontId::proportional(7.5),
                            Color32::from_rgb(180, 90, 255),
                        );

                        if phon_resp.hovered() {
                            let _ = phon_resp.on_hover_text(format!(
                                "Neural Phoneme Morphing Vocoder HUD\nFormant F1: {:.0} Hz | Formant F2: {:.0} Hz\nVocal Tract: {:.1} cm | Style Weight: {:.0}%\n[Drag: F2 (X) / F1 (Y) Phoneme Puck]",
                                f1, f2, tract, style_weight * 100.0
                            ));
                        }
                    } else if is_neural_dereverb {
                        // Neural Dereverberation & Deconvolution Mini Visualizer
                        let (dr_resp, dr_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let dr_rect = dr_resp.rect;
                        dr_painter.rect_filled(dr_rect, 2.0, Color32::from_rgb(8, 18, 20));

                        let mut depth = state.node_param_values.get("suppression_depth_db").copied().unwrap_or(18.0);
                        let mut drr = state.node_param_values.get("direct_to_reverberant_ratio_db").copied().unwrap_or(6.0);
                        let rt60 = state.node_param_values.get("room_rt60_s").copied().unwrap_or(1.5);
                        let blend = state.node_param_values.get("neural_blend_pct").copied().unwrap_or(85.0);

                        let dcx = dr_rect.left() + 34.0;
                        let dcy = dr_rect.center().y;
                        let max_box = 24.0_f32;

                        if dr_resp.dragged() {
                            if let Some(pos) = dr_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - (dcx - max_box)) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - (dcy - max_box)) / (max_box * 2.0))).clamp(0.0, 1.0);
                                depth = crate::views::neural_dereverb_view::NeuralDereverbView::normalized_to_depth(norm_x);
                                drr = crate::views::neural_dereverb_view::NeuralDereverbView::normalized_to_drr(norm_y);
                                state.node_param_values.insert("suppression_depth_db".to_string(), depth);
                                state.node_param_values.insert("direct_to_reverberant_ratio_db".to_string(), drr);
                                if let Some(bus) = param_bus {
                                    let pid_depth = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_drr = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    bus.set(pid_depth, depth);
                                    bus.set(pid_drr, drr);
                                }
                            }
                        }

                        // DRR vs Depth coordinate box & crosshairs
                        let drr_box = egui::Rect::from_center_size(egui::pos2(dcx, dcy), egui::vec2(max_box * 2.0, max_box * 2.0));
                        dr_painter.rect_stroke(drr_box, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(0, 245, 212)));
                        dr_painter.line_segment(
                            [egui::pos2(drr_box.left(), dcy), egui::pos2(drr_box.right(), dcy)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 245, 212, 80)),
                        );
                        dr_painter.line_segment(
                            [egui::pos2(dcx, drr_box.top()), egui::pos2(dcx, drr_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 245, 212, 80)),
                        );

                        // Dereverb Puck
                        let norm_depth = crate::views::neural_dereverb_view::NeuralDereverbView::depth_to_normalized(depth);
                        let norm_drr = crate::views::neural_dereverb_view::NeuralDereverbView::drr_to_normalized(drr);
                        let px = drr_box.left() + norm_depth * (max_box * 2.0);
                        let py = drr_box.bottom() - norm_drr * (max_box * 2.0);
                        dr_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 245, 212));
                        dr_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Impulse energy decay preview curve
                        let curve_left = dr_rect.left() + 72.0;
                        let curve_w = dr_rect.right() - curve_left - 8.0;
                        let curve_bottom = dr_rect.center().y + 18.0;
                        let steps = 16;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let t = s as f32 / steps as f32;
                            let raw_decay = (-3.0 * t / (rt60 * 0.5 + 0.1)).exp();
                            let cleaned = raw_decay * (1.0 - (depth / 36.0) * (1.0 - (-8.0 * t).exp()));
                            let cur_pt = egui::pos2(curve_left + t * curve_w, curve_bottom - cleaned.clamp(0.0, 1.0) * 28.0);
                            if let Some(p) = prev_pt {
                                dr_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(0, 245, 212)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        dr_painter.text(
                            egui::pos2(dr_rect.right() - 4.0, dr_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NEURAL DEREVERB\nDepth: {:.1}dB | DRR: {:+.1}dB\nRT60: {:.2}s | Blend: {:.0}%", depth, drr, rt60, blend),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 245, 212),
                        );

                        if dr_resp.hovered() {
                            let _ = dr_resp.on_hover_text(format!(
                                "Neural Acoustic Blind Dereverberation HUD\nSuppression Depth: {:.1} dB | DRR: {:+.1} dB\nRoom RT60: {:.2} s | Blend: {:.0}%\n[Drag: Depth (X) / DRR (Y) Dereverb Puck]",
                                depth, drr, rt60, blend
                            ));
                        }
                    } else if is_neural_super_res {
                        // Neural Audio Super-Resolution & High-Frequency Bandwidth Extension Visualizer
                        let (sr_resp, sr_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let sr_rect = sr_resp.rect;
                        sr_painter.rect_filled(sr_rect, 2.0, Color32::from_rgb(6, 12, 22));

                        let mut cont = state.node_param_values.get("spectral_continuity").copied().unwrap_or(0.75);
                        let mut boost = state.node_param_values.get("hf_boost_db").copied().unwrap_or(4.5);
                        let density = state.node_param_values.get("harmonic_density").copied().unwrap_or(0.60);
                        let cutoff = state.node_param_values.get("cutoff_frequency_hz").copied().unwrap_or(12000.0);

                        let center = sr_rect.center();
                        let max_box = 24.0;
                        let sr_box = egui::Rect::from_center_size(
                            egui::pos2(sr_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        sr_painter.rect_filled(sr_box, 1.0, Color32::from_rgb(10, 18, 30));
                        sr_painter.rect_stroke(
                            sr_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 80)),
                        );

                        if sr_resp.dragged() {
                            if let Some(pos) = sr_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - sr_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - sr_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                cont = crate::views::neural_super_resolution_view::NeuralSuperResolutionView::normalized_to_continuity(norm_x);
                                boost = crate::views::neural_super_resolution_view::NeuralSuperResolutionView::normalized_to_boost(norm_y);

                                state.node_param_values.insert("spectral_continuity".to_string(), cont);
                                state.node_param_values.insert("hf_boost_db".to_string(), boost);

                                if let Some(bus) = param_bus {
                                    let pid_boost = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_cont = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_boost).is_some() { bus.set(pid_boost, boost); }
                                    if bus.get(pid_cont).is_some() { bus.set(pid_cont, cont); }
                                }
                            }
                        }

                        // Grid crosshairs
                        sr_painter.line_segment(
                            [egui::pos2(sr_box.left(), sr_box.center().y), egui::pos2(sr_box.right(), sr_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 80)),
                        );
                        sr_painter.line_segment(
                            [egui::pos2(sr_box.center().x, sr_box.top()), egui::pos2(sr_box.center().x, sr_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 80)),
                        );

                        // Super-Res Puck
                        let norm_cont = crate::views::neural_super_resolution_view::NeuralSuperResolutionView::continuity_to_normalized(cont);
                        let norm_boost = crate::views::neural_super_resolution_view::NeuralSuperResolutionView::boost_to_normalized(boost);
                        let px = sr_box.left() + norm_cont * (max_box * 2.0);
                        let py = sr_box.bottom() - norm_boost * (max_box * 2.0);
                        sr_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(56, 189, 248));
                        sr_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Spectrum analyzer preview bars (8 baseband + 8 reconstructed HF)
                        let spec_left = sr_rect.left() + 72.0;
                        let spec_w = sr_rect.right() - spec_left - 8.0;
                        let bar_w = (spec_w / 16.0 - 1.0).max(1.5);
                        let spec_bottom = sr_rect.center().y + 18.0;

                        for i in 0..16 {
                            let bx = spec_left + i as f32 * (bar_w + 1.0);
                            let val = if i < 8 {
                                (1.0 - 0.45 * (i as f32 / 8.0)).clamp(0.1, 1.0)
                            } else {
                                let decay = (-0.18 * (i - 8) as f32).exp();
                                (decay * (1.0 + boost / 12.0) * (0.5 + 0.5 * cont) * (0.6 + 0.4 * density)).clamp(0.05, 1.25)
                            };
                            let bh = (val * 24.0).clamp(2.0, 28.0);
                            let color = if i < 8 {
                                Color32::from_rgb(71, 85, 105)
                            } else {
                                Color32::from_rgb(56, 189, 248)
                            };
                            sr_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(bx, spec_bottom - bh), egui::vec2(bar_w, bh)),
                                1.0,
                                color,
                            );
                        }

                        // Readout
                        sr_painter.text(
                            egui::pos2(sr_rect.right() - 4.0, sr_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("SUPER-RESOLUTION\nBoost: +{:.1}dB | Cont: {:.0}%\nCutoff: {:.0}Hz", boost, cont * 100.0, cutoff),
                            FontId::proportional(7.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if sr_resp.hovered() {
                            let _ = sr_resp.on_hover_text(format!(
                                "Neural Audio Super-Resolution HUD\nHF Boost: +{:.1} dB | Continuity: {:.0}%\nHarmonic Density: {:.0}% | Cutoff: {:.0} Hz\n[Drag: Continuity (X) / Boost (Y) Puck]",
                                boost, cont * 100.0, density * 100.0, cutoff
                            ));
                        }
                    } else if is_nam_amp {
                        // Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier Visualizer
                        let (na_resp, na_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let na_rect = na_resp.rect;
                        na_painter.rect_filled(na_rect, 2.0, Color32::from_rgb(18, 12, 6));

                        let mut gate = state.node_param_values.get("gate_threshold_db").copied().unwrap_or(-60.0);
                        let mut drive = state.node_param_values.get("drive").copied().unwrap_or(2.5);
                        let gain = state.node_param_values.get("output_gain").copied().unwrap_or(0.95);
                        let sag = state.node_param_values.get("tube_sag").copied().unwrap_or(0.35);

                        let center = na_rect.center();
                        let max_box = 24.0;
                        let na_box = egui::Rect::from_center_size(
                            egui::pos2(na_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        na_painter.rect_filled(na_box, 1.0, Color32::from_rgb(26, 18, 10));
                        na_painter.rect_stroke(
                            na_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 80)),
                        );

                        if na_resp.dragged() {
                            if let Some(pos) = na_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - na_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - na_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                gate = crate::views::nam_amp_view::NamAmpView::normalized_to_gate(norm_x);
                                drive = crate::views::nam_amp_view::NamAmpView::normalized_to_drive(norm_y);

                                state.node_param_values.insert("gate_threshold_db".to_string(), gate);
                                state.node_param_values.insert("drive".to_string(), drive);

                                if let Some(bus) = param_bus {
                                    let pid_drive = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_gain = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    let pid_gate = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    if bus.get(pid_drive).is_some() { bus.set(pid_drive, drive); }
                                    if bus.get(pid_gain).is_some() { bus.set(pid_gain, gain); }
                                    if bus.get(pid_gate).is_some() { bus.set(pid_gate, gate); }
                                }
                            }
                        }

                        // Grid crosshairs
                        na_painter.line_segment(
                            [egui::pos2(na_box.left(), na_box.center().y), egui::pos2(na_box.right(), na_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 80)),
                        );
                        na_painter.line_segment(
                            [egui::pos2(na_box.center().x, na_box.top()), egui::pos2(na_box.center().x, na_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 80)),
                        );

                        // NAM Amp Puck
                        let norm_gate = crate::views::nam_amp_view::NamAmpView::gate_to_normalized(gate);
                        let norm_drive = crate::views::nam_amp_view::NamAmpView::drive_to_normalized(drive);
                        let px = na_box.left() + norm_gate * (max_box * 2.0);
                        let py = na_box.bottom() - norm_drive * (max_box * 2.0);
                        na_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(245, 158, 11));
                        na_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Non-linear transfer curve preview
                        let curve_left = na_rect.left() + 72.0;
                        let curve_w = na_rect.right() - curve_left - 8.0;
                        let curve_center_y = na_rect.center().y + 8.0;
                        let steps = 16;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let t = -1.0 + (s as f32 / steps as f32) * 2.0;
                            let y_sat = (t * drive * (1.0 + sag * 0.4)).tanh() * gain;
                            let cur_pt = egui::pos2(curve_left + (s as f32 / steps as f32) * curve_w, curve_center_y - y_sat * 22.0);
                            if let Some(p) = prev_pt {
                                na_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(245, 158, 11)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        na_painter.text(
                            egui::pos2(na_rect.right() - 4.0, na_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("NAM WAVENET AMP\nDrive: {:.2}x | Gate: {:.1}dB\nGain: {:.2}x | Sag: {:.0}%", drive, gate, gain, sag * 100.0),
                            FontId::proportional(7.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if na_resp.hovered() {
                            let _ = na_resp.on_hover_text(format!(
                                "Neural Amp Modeler (NAM) WaveNet HUD\nPre-Amp Drive: {:.2} x | Gate Threshold: {:.1} dB\nOutput Gain: {:.2} x | Tube Sag: {:.0}%\n[Drag: Gate (X) / Drive (Y) Amp Puck]",
                                drive, gate, gain, sag * 100.0
                            ));
                        }
                    } else if is_neural_inpaint {
                        // Neural Diffusion Audio Inpainter & Spectral Repair Mini Visualizer
                        let (nip_resp, nip_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let nip_rect = nip_resp.rect;
                        nip_painter.rect_filled(nip_rect, 2.0, Color32::from_rgb(10, 16, 24));

                        let mut time_ms = state.node_param_values.get("mask_center_time_ms").copied().unwrap_or(250.0);
                        let mut freq_hz = state.node_param_values.get("mask_center_freq_hz").copied().unwrap_or(1000.0);
                        let duration_ms = state.node_param_values.get("mask_duration_ms").copied().unwrap_or(45.0);
                        let steps = state.node_param_values.get("diffusion_steps").copied().unwrap_or(25.0) as usize;
                        let guidance = state.node_param_values.get("guidance_scale").copied().unwrap_or(4.5);

                        let center = nip_rect.center();
                        let max_box = 24.0;
                        let nip_box = egui::Rect::from_center_size(
                            egui::pos2(nip_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        nip_painter.rect_filled(nip_box, 1.0, Color32::from_rgb(14, 22, 34));
                        nip_painter.rect_stroke(
                            nip_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        if nip_resp.dragged() {
                            if let Some(pos) = nip_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - nip_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - nip_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                time_ms = crate::views::neural_inpaint_view::NeuralInpaintView::normalized_to_time(norm_x);
                                freq_hz = crate::views::neural_inpaint_view::NeuralInpaintView::normalized_to_freq(norm_y);

                                state.node_param_values.insert("mask_center_time_ms".to_string(), time_ms);
                                state.node_param_values.insert("mask_center_freq_hz".to_string(), freq_hz);

                                if let Some(bus) = param_bus {
                                    let pid_time = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_freq = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_time).is_some() { bus.set(pid_time, time_ms); }
                                    if bus.get(pid_freq).is_some() { bus.set(pid_freq, freq_hz); }
                                }
                            }
                        }

                        // Grid crosshairs
                        nip_painter.line_segment(
                            [egui::pos2(nip_box.left(), nip_box.center().y), egui::pos2(nip_box.right(), nip_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );
                        nip_painter.line_segment(
                            [egui::pos2(nip_box.center().x, nip_box.top()), egui::pos2(nip_box.center().x, nip_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        // Inpaint Mask Puck
                        let norm_t = crate::views::neural_inpaint_view::NeuralInpaintView::time_to_normalized(time_ms);
                        let norm_f = crate::views::neural_inpaint_view::NeuralInpaintView::freq_to_normalized(freq_hz);
                        let px = nip_box.left() + norm_t * (max_box * 2.0);
                        let py = nip_box.bottom() - norm_f * (max_box * 2.0);
                        nip_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 229, 255));
                        nip_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 180)),
                        );

                        // Mask rectangle preview
                        let mask_w = (duration_ms / 500.0) * (max_box * 2.0) * 1.5;
                        let mask_rect = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(mask_w.clamp(6.0, 36.0), 12.0));
                        nip_painter.rect_stroke(
                            mask_rect,
                            1.0,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
                        );

                        // Diffusion step progress bars preview
                        let bars_left = nip_rect.left() + 72.0;
                        let bars_w = nip_rect.right() - bars_left - 8.0;
                        let num_steps = steps.clamp(5, 30);
                        let step_w = (bars_w / num_steps as f32 - 1.0).max(1.5);
                        let step_bottom = nip_rect.center().y + 18.0;
                        for i in 0..num_steps {
                            let bx = bars_left + i as f32 * (step_w + 1.0);
                            let bh = ((i + 1) as f32 / num_steps as f32 * 22.0).clamp(3.0, 24.0);
                            let col = if i < num_steps / 2 {
                                Color32::from_rgb(0, 229, 255)
                            } else {
                                Color32::from_rgb(255, 215, 0)
                            };
                            nip_painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(bx, step_bottom - bh), egui::vec2(step_w, bh)),
                                1.0,
                                col,
                            );
                        }

                        // Readout
                        nip_painter.text(
                            egui::pos2(nip_rect.right() - 4.0, nip_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("DIFFUSION INPAINT\n{:.1}ms @ {:.0}Hz\nSteps: {} | Guide: {:.1}x", time_ms, freq_hz, steps, guidance),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if nip_resp.hovered() {
                            let _ = nip_resp.on_hover_text(format!(
                                "Neural Diffusion Audio Inpainter HUD\nCenter: {:.1} ms @ {:.0} Hz | Duration: {:.1} ms\nDiffusion Steps: {} | Guidance: {:.1}x\n[Drag: Time (X) / Frequency (Y) Puck]",
                                time_ms, freq_hz, duration_ms, steps, guidance
                            ));
                        }
                    } else if is_neural_timbre {
                        // Neural Timbre Transfer Morphing Mini Visualizer
                        let (nt_resp, nt_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let nt_rect = nt_resp.rect;
                        nt_painter.rect_filled(nt_rect, 2.0, Color32::from_rgb(20, 10, 16));

                        let mut z1 = state.node_param_values.get("latent_z1").copied().unwrap_or(0.45);
                        let mut z2 = state.node_param_values.get("latent_z2").copied().unwrap_or(-0.30);
                        let flow = state.node_param_values.get("flow_rate_hz").copied().unwrap_or(1.20);
                        let blend = state.node_param_values.get("residual_blend").copied().unwrap_or(0.20);
                        let conv = state.node_param_values.get("timbre_convergence_pct").copied().unwrap_or(99.4);

                        let center = nt_rect.center();
                        let max_box = 24.0;
                        let nt_box = egui::Rect::from_center_size(
                            egui::pos2(nt_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        nt_painter.rect_filled(nt_box, 1.0, Color32::from_rgb(28, 14, 22));
                        nt_painter.rect_stroke(
                            nt_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(244, 63, 94, 80)),
                        );

                        if nt_resp.dragged() {
                            if let Some(pos) = nt_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - nt_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - nt_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                z1 = crate::views::neural_timbre_view::NeuralTimbreView::normalized_to_coord(norm_x);
                                z2 = crate::views::neural_timbre_view::NeuralTimbreView::normalized_to_coord(norm_y);

                                state.node_param_values.insert("latent_z1".to_string(), z1);
                                state.node_param_values.insert("latent_z2".to_string(), z2);

                                if let Some(bus) = param_bus {
                                    let pid_z1 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_z2 = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_z1).is_some() { bus.set(pid_z1, z1); }
                                    if bus.get(pid_z2).is_some() { bus.set(pid_z2, z2); }
                                }
                            }
                        }

                        // Grid crosshairs
                        nt_painter.line_segment(
                            [egui::pos2(nt_box.left(), nt_box.center().y), egui::pos2(nt_box.right(), nt_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(244, 63, 94, 80)),
                        );
                        nt_painter.line_segment(
                            [egui::pos2(nt_box.center().x, nt_box.top()), egui::pos2(nt_box.center().x, nt_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(244, 63, 94, 80)),
                        );

                        // Timbre Puck
                        let norm_z1 = crate::views::neural_timbre_view::NeuralTimbreView::coord_to_normalized(z1);
                        let norm_z2 = crate::views::neural_timbre_view::NeuralTimbreView::coord_to_normalized(z2);
                        let px = nt_box.left() + norm_z1 * (max_box * 2.0);
                        let py = nt_box.bottom() - norm_z2 * (max_box * 2.0);
                        nt_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(244, 63, 94));
                        nt_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Spectral envelope trajectory preview curve
                        let curve_left = nt_rect.left() + 72.0;
                        let curve_w = nt_rect.right() - curve_left - 8.0;
                        let curve_center_y = nt_rect.center().y + 8.0;
                        let steps = 16;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let frac = s as f32 / steps as f32;
                            let y_env = ((frac * 3.14159 * 2.0 + z1).sin() * 0.5 + (frac * 6.28 + z2).cos() * 0.3) * (0.6 + 0.4 * blend);
                            let cur_pt = egui::pos2(curve_left + frac * curve_w, curve_center_y - y_env * 20.0);
                            if let Some(p) = prev_pt {
                                nt_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(244, 63, 94)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        nt_painter.text(
                            egui::pos2(nt_rect.right() - 4.0, nt_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("TIMBRE FLOW\nZ: ({:.2}, {:.2})\nFlow: {:.2}Hz | {:.0}%", z1, z2, flow, blend * 100.0),
                            FontId::proportional(7.5),
                            Color32::from_rgb(244, 63, 94),
                        );

                        if nt_resp.hovered() {
                            let _ = nt_resp.on_hover_text(format!(
                                "Neural Timbre Transfer Morphing Resynthesizer HUD\nLatent Coord: ({:.2}, {:.2}) | Flow Rate: {:.2} Hz\nResidual Blend: {:.0}% | Convergence: {:.1}%\n[Drag: Latent Z1 (X) / Z2 (Y) Puck]",
                                z1, z2, flow, blend * 100.0, conv
                            ));
                        }
                    } else if is_bitcrusher {
                        // Tactile Bitcrusher & Quantization Mini Visualizer
                        let (bc_resp, bc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let bc_rect = bc_resp.rect;
                        bc_painter.rect_filled(bc_rect, 2.0, Color32::from_rgb(14, 10, 8));

                        let mut bits = state.node_param_values.get("bit_depth").copied().unwrap_or(6.5);
                        let mut downsample = state.node_param_values.get("downsample_ratio").copied().unwrap_or(8.0);
                        let jitter = state.node_param_values.get("jitter_pct").copied().unwrap_or(12.0);
                        let drive = state.node_param_values.get("pre_drive_db").copied().unwrap_or(4.5);
                        let mix = state.node_param_values.get("mix_pct").copied().unwrap_or(100.0);

                        let center = bc_rect.center();
                        let max_box = 24.0;
                        let bc_box = egui::Rect::from_center_size(
                            egui::pos2(bc_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        bc_painter.rect_filled(bc_box, 1.0, Color32::from_rgb(24, 16, 12));
                        bc_painter.rect_stroke(
                            bc_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 80)),
                        );

                        if bc_resp.dragged() {
                            if let Some(pos) = bc_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - bc_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - bc_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                bits = crate::views::bitcrusher_morph_view::BitcrusherMorphView::normalized_to_bit(norm_x);
                                downsample = crate::views::bitcrusher_morph_view::BitcrusherMorphView::normalized_to_downsample(1.0 - norm_y);

                                state.node_param_values.insert("bit_depth".to_string(), bits);
                                state.node_param_values.insert("downsample_ratio".to_string(), downsample);

                                if let Some(bus) = param_bus {
                                    let pid_bits = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_down = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_bits).is_some() { bus.set(pid_bits, bits); }
                                    if bus.get(pid_down).is_some() { bus.set(pid_down, downsample); }
                                }
                            }
                        }

                        // Grid crosshairs
                        bc_painter.line_segment(
                            [egui::pos2(bc_box.left(), bc_box.center().y), egui::pos2(bc_box.right(), bc_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 80)),
                        );
                        bc_painter.line_segment(
                            [egui::pos2(bc_box.center().x, bc_box.top()), egui::pos2(bc_box.center().x, bc_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 80)),
                        );

                        // Bitcrusher Puck
                        let norm_bits = crate::views::bitcrusher_morph_view::BitcrusherMorphView::bit_to_normalized(bits);
                        let norm_down = 1.0 - crate::views::bitcrusher_morph_view::BitcrusherMorphView::downsample_to_normalized(downsample);
                        let px = bc_box.left() + norm_bits * (max_box * 2.0);
                        let py = bc_box.bottom() - norm_down * (max_box * 2.0);
                        bc_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(255, 107, 43));
                        bc_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Quantization staircase curve preview
                        let curve_left = bc_rect.left() + 72.0;
                        let curve_w = bc_rect.right() - curve_left - 8.0;
                        let curve_center_y = bc_rect.center().y + 8.0;
                        let steps = 16;
                        let levels = 2.0_f32.powf(bits.clamp(1.0, 16.0));
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let t = -1.0 + (s as f32 / steps as f32) * 2.0;
                            let q = (t * levels * 0.5).round() / (levels * 0.5);
                            let cur_pt = egui::pos2(curve_left + (s as f32 / steps as f32) * curve_w, curve_center_y - q.clamp(-1.0, 1.0) * 20.0);
                            if let Some(p) = prev_pt {
                                bc_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(255, 107, 43)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        bc_painter.text(
                            egui::pos2(bc_rect.right() - 4.0, bc_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("BITCRUSHER\nBits: {:.1} | Down: {:.1}x\nJitter: {:.0}% | Mix: {:.0}%", bits, downsample, jitter, mix),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 107, 43),
                        );

                        if bc_resp.hovered() {
                            let _ = bc_resp.on_hover_text(format!(
                                "Tactile Bitcrusher & Morphology HUD\nBit Depth: {:.1} bits | Downsample: {:.1}x\nJitter: {:.0}% | Pre-Drive: {:+.1} dB\n[Drag: Bits (X) / Downsample (Y) Puck]",
                                bits, downsample, jitter, drive
                            ));
                        }
                    } else if is_bbd_chorus {
                        // Multi-Voice Analog Bucket-Brigade (BBD) Chorus Mini Visualizer
                        let (bbd_resp, bbd_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let bbd_rect = bbd_resp.rect;
                        bbd_painter.rect_filled(bbd_rect, 2.0, Color32::from_rgb(8, 16, 24));

                        let mut spread = state.node_param_values.get("stereo_spread_percent").copied().unwrap_or(85.0);
                        let mut feedback = state.node_param_values.get("feedback_percent").copied().unwrap_or(30.0);
                        let mix = state.node_param_values.get("mix_percent").copied().unwrap_or(50.0);
                        let clock = state.node_param_values.get("bbd_clock_rate_khz").copied().unwrap_or(44.1);

                        let center = bbd_rect.center();
                        let max_box = 24.0;
                        let bbd_box = egui::Rect::from_center_size(
                            egui::pos2(bbd_rect.left() + 32.0, center.y),
                            egui::vec2(max_box * 2.0, max_box * 2.0),
                        );
                        bbd_painter.rect_filled(bbd_box, 1.0, Color32::from_rgb(12, 24, 34));
                        bbd_painter.rect_stroke(
                            bbd_box,
                            1.0,
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        if bbd_resp.dragged() {
                            if let Some(pos) = bbd_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - bbd_box.left()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                let norm_y = (1.0 - (pos.y - bbd_box.top()) / (max_box * 2.0)).clamp(0.0, 1.0);
                                spread = crate::views::bbd_chorus_view::BbdChorusView::normalized_to_spread(norm_x);
                                feedback = crate::views::bbd_chorus_view::BbdChorusView::normalized_to_feedback(norm_y);

                                state.node_param_values.insert("stereo_spread_percent".to_string(), spread);
                                state.node_param_values.insert("feedback_percent".to_string(), feedback);

                                if let Some(bus) = param_bus {
                                    let pid_spr = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    let pid_fb = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_spr).is_some() { bus.set(pid_spr, spread); }
                                    if bus.get(pid_fb).is_some() { bus.set(pid_fb, feedback); }
                                }
                            }
                        }

                        // Grid crosshairs
                        bbd_painter.line_segment(
                            [egui::pos2(bbd_box.left(), bbd_box.center().y), egui::pos2(bbd_box.right(), bbd_box.center().y)],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );
                        bbd_painter.line_segment(
                            [egui::pos2(bbd_box.center().x, bbd_box.top()), egui::pos2(bbd_box.center().x, bbd_box.bottom())],
                            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                        );

                        // BBD Spatial Puck
                        let norm_spr = crate::views::bbd_chorus_view::BbdChorusView::spread_to_normalized(spread);
                        let norm_fb = crate::views::bbd_chorus_view::BbdChorusView::feedback_to_normalized(feedback);
                        let px = bbd_box.left() + norm_spr * (max_box * 2.0);
                        let py = bbd_box.bottom() - norm_fb * (max_box * 2.0);
                        bbd_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(0, 229, 255));
                        bbd_painter.circle_stroke(
                            egui::pos2(px, py),
                            5.5,
                            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                        );

                        // Lissajous stereo phase ellipse preview
                        let curve_left = bbd_rect.left() + 72.0;
                        let curve_w = bbd_rect.right() - curve_left - 8.0;
                        let curve_center_y = bbd_rect.center().y + 8.0;
                        let steps = 24;
                        let mut prev_pt: Option<egui::Pos2> = None;
                        for s in 0..=steps {
                            let theta = (s as f32 / steps as f32) * std::f32::consts::TAU;
                            let phase_diff = (feedback / 100.0) * 1.5;
                            let lx = theta.sin() * (spread / 100.0).clamp(0.2, 1.0);
                            let ry = (theta + phase_diff).sin();
                            let cur_pt = egui::pos2(curve_left + (lx * 0.5 + 0.5) * curve_w, curve_center_y - ry * 18.0);
                            if let Some(p) = prev_pt {
                                bbd_painter.line_segment([p, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(0, 229, 255)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Readout
                        bbd_painter.text(
                            egui::pos2(bbd_rect.right() - 4.0, bbd_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("BBD CHORUS\nSpread: {:.0}% | FB: {:+.0}%\nMix: {:.0}% | {:.1}kHz", spread, feedback, mix, clock),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if bbd_resp.hovered() {
                            let _ = bbd_resp.on_hover_text(format!(
                                "Multi-Voice BBD Chorus Matrix HUD\nStereo Spread: {:.0}% | Feedback: {:+.0}%\nDry/Wet Mix: {:.0}% | Clock Rate: {:.1} kHz\n[Drag: Spread (X) / Feedback (Y) Puck]",
                                spread, feedback, mix, clock
                            ));
                        }
                    } else if is_tube {
                        // Vacuum Tube Triode Bias & Harmonic Distortion Mini Visualizer
                        let (tb_resp, tb_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let tb_rect = tb_resp.rect;
                        tb_painter.rect_filled(tb_rect, 2.0, Color32::from_rgb(14, 12, 10));

                        let mut bias_v = state.node_param_values.get("bias_voltage_v").copied().unwrap_or(-1.85);
                        let mut drive_db = state.node_param_values.get("drive_warmth_db").copied().unwrap_or(8.5);
                        let plate_v = state.node_param_values.get("plate_voltage_v").copied().unwrap_or(250.0);

                        if tb_resp.dragged() {
                            if let Some(pos) = tb_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - tb_rect.left()) / tb_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - tb_rect.top()) / tb_rect.height())).clamp(0.0, 1.0);
                                bias_v = -4.0 + norm_x * 4.0;
                                drive_db = norm_y * 24.0;
                                state.node_param_values.insert("bias_voltage_v".to_string(), bias_v);
                                state.node_param_values.insert("drive_warmth_db".to_string(), drive_db);
                                if let Some(bus) = param_bus {
                                    let pid_bias = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_bias).is_some() { bus.set(pid_bias, bias_v); }
                                    let pid_drive = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    if bus.get(pid_drive).is_some() { bus.set(pid_drive, drive_db); }
                                }
                            }
                        }

                        // Load-line representation
                        tb_painter.line_segment(
                            [egui::pos2(tb_rect.left() + 8.0, tb_rect.top() + 14.0), egui::pos2(tb_rect.left() + 76.0, tb_rect.bottom() - 10.0)],
                            Stroke::new(1.2_f32, Color32::from_rgb(255, 107, 43)),
                        );

                        // Q-point puck
                        let norm_b = ((bias_v - (-4.0)) / 4.0).clamp(0.0, 1.0);
                        let norm_d = (drive_db / 24.0).clamp(0.0, 1.0);
                        let q_x = tb_rect.left() + 8.0 + norm_b * 68.0;
                        let q_y = tb_rect.bottom() - 10.0 - norm_d * 44.0;
                        tb_painter.circle_filled(egui::pos2(q_x, q_y), 4.0, Color32::from_rgb(255, 215, 0));
                        tb_painter.circle_stroke(egui::pos2(q_x, q_y), 6.0, Stroke::new(1.0_f32, Color32::from_rgb(255, 240, 150)));

                        // Harmonic bars preview (f0..5f0)
                        let bar_base_x = tb_rect.left() + 90.0;
                        for i in 0..5 {
                            let bx = bar_base_x + (i as f32) * 8.0;
                            let bh = match i {
                                0 => 32.0,
                                1 => (drive_db / 24.0 * 24.0).max(6.0),
                                2 => (drive_db / 24.0 * 18.0).max(4.0),
                                3 => (drive_db / 24.0 * 12.0).max(2.0),
                                _ => (drive_db / 24.0 * 8.0).max(1.0),
                            };
                            let b_col = if i == 0 {
                                Color32::from_rgb(0, 229, 255)
                            } else if i % 2 == 1 {
                                Color32::from_rgb(255, 215, 0)
                            } else {
                                Color32::from_rgb(255, 107, 43)
                            };
                            tb_painter.rect_filled(
                                egui::Rect::from_min_max(egui::pos2(bx, tb_rect.bottom() - 10.0 - bh), egui::pos2(bx + 6.0, tb_rect.bottom() - 10.0)),
                                1.0,
                                b_col,
                            );
                        }

                        // Readout
                        tb_painter.text(
                            egui::pos2(tb_rect.right() - 4.0, tb_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("TUBE BIAS\n{:.2}V | +{:.1}dB\n{:.0}V DC", bias_v, drive_db, plate_v),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if tb_resp.hovered() {
                            let _ = tb_resp.on_hover_text(format!(
                                "Vacuum Tube Triode Bias & Harmonic Distortion HUD\nGrid Bias: {:.2} V DC | Drive Warmth: +{:.1} dB\nPlate Voltage: {:.0} V DC\n[Drag: Grid Bias (X) / Drive (Y) Puck]",
                                bias_v, drive_db, plate_v
                            ));
                        }
                    } else if is_tape {
                        // Analog Tape Saturation & Magnetic Hysteresis Mini Visualizer
                        let (tp_resp, tp_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let tp_rect = tp_resp.rect;
                        tp_painter.rect_filled(tp_rect, 2.0, Color32::from_rgb(12, 14, 20));

                        let mut drive = state.node_param_values.get("drive").copied().unwrap_or(2.0);
                        let mut sat = state.node_param_values.get("saturation").copied().unwrap_or(0.6);
                        let bias = state.node_param_values.get("bias").copied().unwrap_or(0.0);

                        if tp_resp.dragged() {
                            if let Some(pos) = tp_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - tp_rect.left()) / tp_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - tp_rect.top()) / tp_rect.height())).clamp(0.0, 1.0);
                                drive = 0.1 + norm_x * (10.0 - 0.1);
                                sat = norm_y;
                                state.node_param_values.insert("drive".to_string(), drive);
                                state.node_param_values.insert("saturation".to_string(), sat);
                                if let Some(bus) = param_bus {
                                    let pid_drv = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_drv).is_some() { bus.set(pid_drv, drive); }
                                    let pid_sat = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_sat).is_some() { bus.set(pid_sat, sat); }
                                }
                            }
                        }

                        // Center crosshair
                        let center_x = tp_rect.left() + 48.0;
                        let center_y = tp_rect.center().y;
                        tp_painter.line_segment([egui::pos2(center_x, tp_rect.top() + 8.0), egui::pos2(center_x, tp_rect.bottom() - 8.0)], Stroke::new(0.6_f32, Color32::from_rgb(35, 48, 68)));
                        tp_painter.line_segment([egui::pos2(center_x - 38.0, center_y), egui::pos2(center_x + 38.0, center_y)], Stroke::new(0.6_f32, Color32::from_rgb(35, 48, 68)));

                        // Hysteresis B-H loop preview (16 points)
                        let steps = 16;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let t = (i as f32 / steps as f32) * std::f32::consts::TAU;
                            let h = t.sin();
                            let b = ((h + bias * 0.2).tanh() * sat + 0.25 * (t - 0.4).cos()).clamp(-1.0, 1.0);
                            let px = center_x + h * 34.0;
                            let py = center_y - b * 24.0;
                            let cur_pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                tp_painter.line_segment([prev, cur_pt], Stroke::new(1.2_f32, Color32::from_rgb(255, 179, 0)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Puck
                        let norm_drv = ((drive - 0.1) / (10.0 - 0.1)).clamp(0.0, 1.0);
                        let puck_x = center_x - 34.0 + norm_drv * 68.0;
                        let puck_y = center_y + 24.0 - sat * 48.0;
                        tp_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(255, 179, 0));
                        tp_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.0, Stroke::new(1.0_f32, Color32::from_rgb(255, 220, 120)));

                        // Readout
                        tp_painter.text(
                            egui::pos2(tp_rect.right() - 4.0, tp_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("TAPE SAT\nDrv: {:.1}x | Sat: {:.0}%\nBias: {:+.1}", drive, sat * 100.0, bias),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 179, 0),
                        );

                        if tp_resp.hovered() {
                            let _ = tp_resp.on_hover_text(format!(
                                "Analog Tape Saturation & Magnetic Hysteresis HUD\nDrive: {:.1}x | Saturation: {:.0}%\nAC Bias: {:+.2}\n[Drag: Drive (X) / Saturation (Y) Puck]",
                                drive, sat * 100.0, bias
                            ));
                        }
                    } else if is_ladder {
                        // Analog Ladder 24dB/oct Filter Mini Visualizer
                        let (lf_resp, lf_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let lf_rect = lf_resp.rect;
                        lf_painter.rect_filled(lf_rect, 2.0, Color32::from_rgb(10, 14, 22));

                        let mut cutoff = state.node_param_values.get("cutoff_freq_hz").copied().unwrap_or(1450.0);
                        let mut res = state.node_param_values.get("resonance_q").copied().unwrap_or(6.5);
                        let drive = state.node_param_values.get("drive_saturation").copied().unwrap_or(35.0);

                        if lf_resp.dragged() {
                            if let Some(pos) = lf_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - lf_rect.left()) / lf_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - lf_rect.top()) / lf_rect.height())).clamp(0.0, 1.0);
                                cutoff = crate::views::ladder_filter_view::LadderFilterView::normalized_to_freq(norm_x);
                                res = crate::views::ladder_filter_view::LadderFilterView::normalized_to_resonance(norm_y);
                                state.node_param_values.insert("cutoff_freq_hz".to_string(), cutoff);
                                state.node_param_values.insert("resonance_q".to_string(), res);
                                if let Some(bus) = param_bus {
                                    let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_cut).is_some() { bus.set(pid_cut, cutoff); }
                                    let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_res).is_some() { bus.set(pid_res, res); }
                                }
                            }
                        }

                        // Background grid lines
                        for step in 1..3 {
                            let gy = lf_rect.top() + lf_rect.height() * (step as f32 * 0.33);
                            lf_painter.line_segment([egui::pos2(lf_rect.left(), gy), egui::pos2(lf_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(40, 55, 80, 80)));
                        }

                        // Draw 24dB/oct response curve preview (16 points)
                        let norm_cut = crate::views::ladder_filter_view::LadderFilterView::freq_to_normalized(cutoff);
                        let norm_res = crate::views::ladder_filter_view::LadderFilterView::resonance_to_normalized(res);
                        let steps = 16;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let nx = i as f32 / steps as f32;
                            let slope = if nx <= norm_cut { 1.0 + (norm_res * 0.8) * (1.0 - (nx - norm_cut).abs() * 3.0).max(0.0) } else { (1.0 - (nx - norm_cut) * 2.2).max(0.05) };
                            let px = lf_rect.left() + nx * lf_rect.width();
                            let py = lf_rect.bottom() - (slope.clamp(0.0, 1.8) / 1.8) * (lf_rect.height() - 10.0) - 5.0;
                            let cur_pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                lf_painter.line_segment([prev, cur_pt], Stroke::new(1.3_f32, Color32::from_rgb(0, 229, 255)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Cutoff / Resonance Puck
                        let puck_x = lf_rect.left() + norm_cut * lf_rect.width();
                        let puck_y = lf_rect.bottom() - norm_res * (lf_rect.height() - 12.0) - 6.0;
                        lf_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(0, 229, 255));
                        lf_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(140, 240, 255)));

                        // Readout
                        lf_painter.text(
                            egui::pos2(lf_rect.right() - 4.0, lf_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("LADDER 24dB\n{:.0}Hz | Q:{:.1}\nDrv:{:.0}%", cutoff, res, drive),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if lf_resp.hovered() {
                            let _ = lf_resp.on_hover_text(format!(
                                "Analog Ladder 24dB/oct Filter & Self-Oscillation HUD\nCutoff: {:.0} Hz | Res Q: {:.1}\nDrive: {:.0}%\n[Drag: Cutoff (X) / Resonance (Y) Puck]",
                                cutoff, res, drive
                            ));
                        }
                    } else if is_granular_freeze {
                        // Granular Spectral Cloud Freeze Mini Visualizer
                        let (gf_resp, gf_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let gf_rect = gf_resp.rect;
                        gf_painter.rect_filled(gf_rect, 2.0, Color32::from_rgb(12, 16, 26));

                        let mut playhead = state.node_param_values.get("position_norm").copied().unwrap_or(0.45);
                        let mut spray = state.node_param_values.get("pitch_spray_st").copied().unwrap_or(7.0);
                        let size = state.node_param_values.get("grain_size_ms").copied().unwrap_or(120.0);
                        let density = state.node_param_values.get("grain_density_hz").copied().unwrap_or(35.0);

                        if gf_resp.dragged() {
                            if let Some(pos) = gf_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - gf_rect.left()) / gf_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - gf_rect.top()) / gf_rect.height())).clamp(0.0, 1.0);
                                playhead = norm_x;
                                spray = crate::views::granular_freeze_view::GranularFreezeView::normalized_to_pitch(norm_y);
                                state.node_param_values.insert("position_norm".to_string(), playhead);
                                state.node_param_values.insert("pitch_spray_st".to_string(), spray);
                                if let Some(bus) = param_bus {
                                    let pid_spray = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 2);
                                    if bus.get(pid_spray).is_some() { bus.set(pid_spray, spray); }
                                }
                            }
                        }

                        // Simulated grain particles cloud
                        for i in 0..8 {
                            let gx = gf_rect.left() + 10.0 + ((i as f32 * 17.3 + playhead * 40.0) % (gf_rect.width() - 20.0));
                            let gy = gf_rect.top() + 10.0 + ((i as f32 * 23.7) % (gf_rect.height() - 20.0));
                            gf_painter.circle_filled(egui::pos2(gx, gy), 2.5, Color32::from_rgba_premultiplied(0, 255, 180, 140));
                        }

                        // Vertical Playhead Line
                        let pk_x = gf_rect.left() + playhead * gf_rect.width();
                        gf_painter.line_segment([egui::pos2(pk_x, gf_rect.top() + 4.0), egui::pos2(pk_x, gf_rect.bottom() - 4.0)], Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)));

                        // Playhead Puck
                        let norm_p = crate::views::granular_freeze_view::GranularFreezeView::pitch_to_normalized(spray);
                        let pk_y = gf_rect.bottom() - norm_p * (gf_rect.height() - 12.0) - 6.0;
                        gf_painter.circle_filled(egui::pos2(pk_x, pk_y), 4.0, Color32::from_rgb(0, 255, 180));
                        gf_painter.circle_stroke(egui::pos2(pk_x, pk_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(160, 255, 220)));

                        // Readout
                        gf_painter.text(
                            egui::pos2(gf_rect.right() - 4.0, gf_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("GRANULAR\n{:.0}ms | {:+.1}st\n{:.0} gr/s", size, spray, density),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 255, 180),
                        );

                        if gf_resp.hovered() {
                            let _ = gf_resp.on_hover_text(format!(
                                "Granular Spectral Cloud Freeze & Trajectory HUD\nPitch Spray: {:+.1} st | Size: {:.0} ms\nDensity: {:.0} gr/s\n[Drag: Playhead (X) / Pitch Spray (Y) Puck]",
                                spray, size, density
                            ));
                        }
                    } else if is_auto_wah {
                        // Dynamic Resonant Auto-Wah Filter Mini Visualizer
                        let (aw_resp, aw_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let aw_rect = aw_resp.rect;
                        aw_painter.rect_filled(aw_rect, 2.0, Color32::from_rgb(10, 16, 24));

                        let mut sens = state.node_param_values.get("sensitivity").copied().unwrap_or(0.75);
                        let mut res = state.node_param_values.get("resonance_q").copied().unwrap_or(6.5);
                        let base_freq = state.node_param_values.get("base_freq_hz").copied().unwrap_or(350.0);

                        if aw_resp.dragged() {
                            if let Some(pos) = aw_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - aw_rect.left()) / aw_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - aw_rect.top()) / aw_rect.height())).clamp(0.0, 1.0);
                                sens = crate::views::auto_wah_view::AutoWahView::normalized_to_sensitivity(norm_x);
                                res = crate::views::auto_wah_view::AutoWahView::normalized_to_resonance(norm_y);
                                state.node_param_values.insert("sensitivity".to_string(), sens);
                                state.node_param_values.insert("resonance_q".to_string(), res);
                                if let Some(bus) = param_bus {
                                    let pid_sens = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_sens).is_some() { bus.set(pid_sens, sens); }
                                    let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_res).is_some() { bus.set(pid_res, res); }
                                }
                            }
                        }

                        // Background grid lines
                        for step in 1..3 {
                            let gy = aw_rect.top() + aw_rect.height() * (step as f32 * 0.33);
                            aw_painter.line_segment([egui::pos2(aw_rect.left(), gy), egui::pos2(aw_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(40, 55, 80, 80)));
                        }

                        // Draw bandpass response curve preview (16 points)
                        let norm_s = crate::views::auto_wah_view::AutoWahView::sensitivity_to_normalized(sens);
                        let norm_q = crate::views::auto_wah_view::AutoWahView::resonance_to_normalized(res);
                        let steps = 16;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let nx = i as f32 / steps as f32;
                            let bell = (1.0 - ((nx - norm_s) * 3.5).powi(2)).max(0.0) * (0.4 + norm_q * 0.6);
                            let px = aw_rect.left() + nx * aw_rect.width();
                            let py = aw_rect.bottom() - (bell.clamp(0.0, 1.0)) * (aw_rect.height() - 10.0) - 5.0;
                            let cur_pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                aw_painter.line_segment([prev, cur_pt], Stroke::new(1.3_f32, Color32::from_rgb(6, 214, 160)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Puck handle
                        let puck_x = aw_rect.left() + norm_s * aw_rect.width();
                        let puck_y = aw_rect.bottom() - norm_q * (aw_rect.height() - 12.0) - 6.0;
                        aw_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(6, 214, 160));
                        aw_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(140, 255, 220)));

                        // Readout
                        aw_painter.text(
                            egui::pos2(aw_rect.right() - 4.0, aw_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("AUTO-WAH\nSens:{:.2} | Q:{:.1}\n{:.0}Hz", sens, res, base_freq),
                            FontId::proportional(7.5),
                            Color32::from_rgb(6, 214, 160),
                        );

                        if aw_resp.hovered() {
                            let _ = aw_resp.on_hover_text(format!(
                                "Dynamic Resonant Auto-Wah Filter HUD\nSensitivity: {:.2} | Res Q: {:.1}\nBase Freq: {:.0} Hz\n[Drag: Sens (X) / Resonance (Y) Puck]",
                                sens, res, base_freq
                            ));
                        }
                    } else if is_formant {
                        // 5-Vowel Formant Filter Mini Visualizer
                        let (fm_resp, fm_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let fm_rect = fm_resp.rect;
                        fm_painter.rect_filled(fm_rect, 2.0, Color32::from_rgb(12, 14, 24));

                        let mut morph_x = state.node_param_values.get("morph_pos_x").copied().unwrap_or(0.40);
                        let mut morph_y = state.node_param_values.get("morph_pos_y").copied().unwrap_or(0.85);
                        let res = state.node_param_values.get("resonance_q").copied().unwrap_or(8.0);
                        let tract = state.node_param_values.get("vocal_tract_scale").copied().unwrap_or(1.0);

                        if fm_resp.dragged() {
                            if let Some(pos) = fm_resp.interact_pointer_pos() {
                                morph_x = ((pos.x - fm_rect.left()) / fm_rect.width()).clamp(0.0, 1.0);
                                morph_y = (1.0 - ((pos.y - fm_rect.top()) / fm_rect.height())).clamp(0.0, 1.0);
                                state.node_param_values.insert("morph_pos_x".to_string(), morph_x);
                                state.node_param_values.insert("morph_pos_y".to_string(), morph_y);
                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 5);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, morph_x); }
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 6);
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, morph_y); }
                                }
                            }
                        }

                        // Draw vowel landmark dots
                        for (vx, vy, lbl) in [(0.40, 0.85, "A"), (0.70, 0.50, "E"), (0.90, 0.15, "I"), (0.20, 0.45, "O"), (0.10, 0.15, "U")] {
                            let px = fm_rect.left() + vx * fm_rect.width();
                            let py = fm_rect.bottom() - vy * (fm_rect.height() - 10.0) - 5.0;
                            fm_painter.circle_filled(egui::pos2(px, py), 2.0, Color32::from_rgb(255, 215, 0));
                            fm_painter.text(egui::pos2(px + 4.0, py - 4.0), egui::Align2::LEFT_TOP, lbl, FontId::monospace(6.5), Color32::from_rgb(180, 200, 230));
                        }

                        // Puck handle
                        let puck_x = fm_rect.left() + morph_x * fm_rect.width();
                        let puck_y = fm_rect.bottom() - morph_y * (fm_rect.height() - 12.0) - 6.0;
                        fm_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(255, 215, 0));
                        fm_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(255, 235, 120)));

                        // Readout
                        fm_painter.text(
                            egui::pos2(fm_rect.right() - 4.0, fm_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("FORMANT\nF1/F2 Morph\nQ:{:.1} | {:.2}x", res, tract),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 215, 0),
                        );

                        if fm_resp.hovered() {
                            let _ = fm_resp.on_hover_text(format!(
                                "5-Vowel Phonetic Formant Filter Resonator HUD\nMorph: ({:.2}, {:.2})\nRes Q: {:.1} | Vocal Tract: {:.2}x\n[Drag: F2 (X) / F1 (Y) Vowel Morph Puck]",
                                morph_x, morph_y, res, tract
                            ));
                        }
                    } else if is_limiter {
                        // Mastering True-Peak 8x Oversampled Limiter Mini Visualizer
                        let (lm_resp, lm_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let lm_rect = lm_resp.rect;
                        lm_painter.rect_filled(lm_rect, 2.0, Color32::from_rgb(10, 14, 24));

                        let mut thresh = state.node_param_values.get("threshold_db").copied().unwrap_or(-6.0);
                        let mut ceil = state.node_param_values.get("ceiling_dbtp").copied().unwrap_or(-0.3);
                        let gr = state.node_param_values.get("gain_reduction_db").copied().unwrap_or(4.2);

                        if lm_resp.dragged() {
                            if let Some(pos) = lm_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - lm_rect.left()) / lm_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - lm_rect.top()) / lm_rect.height())).clamp(0.0, 1.0);
                                thresh = crate::views::oversampled_limiter_view::OversampledLimiterView::normalized_to_thresh(norm_x);
                                ceil = crate::views::oversampled_limiter_view::OversampledLimiterView::normalized_to_ceiling(norm_y);
                                state.node_param_values.insert("threshold_db".to_string(), thresh);
                                state.node_param_values.insert("ceiling_dbtp".to_string(), ceil);
                                if let Some(bus) = param_bus {
                                    let pid_ceil = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_ceil).is_some() { bus.set(pid_ceil, ceil); }
                                    let pid_thresh = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_thresh).is_some() { bus.set(pid_thresh, thresh); }
                                }
                            }
                        }

                        // Background grid lines
                        for step in 1..3 {
                            let gy = lm_rect.top() + lm_rect.height() * (step as f32 * 0.33);
                            lm_painter.line_segment([egui::pos2(lm_rect.left(), gy), egui::pos2(lm_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(35, 50, 75, 80)));
                        }

                        // 8x Polyphase Sinc ISP waveform preview (16 points)
                        let norm_t = crate::views::oversampled_limiter_view::OversampledLimiterView::thresh_to_normalized(thresh);
                        let norm_c = crate::views::oversampled_limiter_view::OversampledLimiterView::ceiling_to_normalized(ceil);
                        let steps = 16;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let nx = i as f32 / steps as f32;
                            let sinc = if (nx - 0.5).abs() < 1e-3 {
                                1.0
                            } else {
                                let pi_t = std::f32::consts::PI * (nx - 0.5) * 4.0;
                                (pi_t.sin() / pi_t).abs()
                            };
                            let px = lm_rect.left() + nx * lm_rect.width();
                            let py = lm_rect.bottom() - (sinc * 0.75 * norm_c).clamp(0.0, 1.0) * (lm_rect.height() - 10.0) - 5.0;
                            let cur_pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                lm_painter.line_segment([prev, cur_pt], Stroke::new(1.3_f32, Color32::from_rgb(0, 229, 255)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Puck handle
                        let puck_x = lm_rect.left() + norm_t * lm_rect.width();
                        let puck_y = lm_rect.bottom() - norm_c * (lm_rect.height() - 12.0) - 6.0;
                        lm_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(0, 229, 255));
                        lm_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(180, 245, 255)));

                        // Readout
                        lm_painter.text(
                            egui::pos2(lm_rect.right() - 4.0, lm_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("LIMITER 8x ISP\nCeil:{:.1} | Th:{:.1}\nGR: -{:.1}dB", ceil, thresh, gr),
                            FontId::proportional(7.5),
                            Color32::from_rgb(0, 229, 255),
                        );

                        if lm_resp.hovered() {
                            let _ = lm_resp.on_hover_text(format!(
                                "Mastering True-Peak 8x Oversampled Limiter HUD\nCeiling: {:.1} dBTP | Threshold: {:.1} dB\nGain Reduction: -{:.1} dB\n[Drag: Thresh (X) / Ceiling (Y) Puck]",
                                ceil, thresh, gr
                            ));
                        }
                    } else if is_vari_mu {
                        // Mastering Dual-Mono Variable-Mu Tube Optical Compressor Mini Visualizer
                        let (vm_resp, vm_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let vm_rect = vm_resp.rect;
                        vm_painter.rect_filled(vm_rect, 2.0, Color32::from_rgb(18, 14, 22));

                        let mut thresh = state.node_param_values.get("threshold_db").copied().unwrap_or(-14.0);
                        let mut drive = state.node_param_values.get("input_drive_db").copied().unwrap_or(4.5);
                        let ratio = state.node_param_values.get("dynamic_ratio").copied().unwrap_or(2.4);
                        let thd = state.node_param_values.get("thd_distortion_pct").copied().unwrap_or(0.35);

                        if vm_resp.dragged() {
                            if let Some(pos) = vm_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - vm_rect.left()) / vm_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - vm_rect.top()) / vm_rect.height())).clamp(0.0, 1.0);
                                thresh = crate::views::vari_mu_master_view::VariMuMasterView::normalized_to_threshold(norm_x);
                                drive = crate::views::vari_mu_master_view::VariMuMasterView::normalized_to_drive(norm_y);
                                state.node_param_values.insert("threshold_db".to_string(), thresh);
                                state.node_param_values.insert("input_drive_db".to_string(), drive);
                                if let Some(bus) = param_bus {
                                    let pid_th = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_th).is_some() { bus.set(pid_th, thresh); }
                                    let pid_drv = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_drv).is_some() { bus.set(pid_drv, drive); }
                                }
                            }
                        }

                        // Background grid lines & 1:1 diagonal guide
                        for step in 1..3 {
                            let gy = vm_rect.top() + vm_rect.height() * (step as f32 * 0.33);
                            vm_painter.line_segment([egui::pos2(vm_rect.left(), gy), egui::pos2(vm_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(60, 45, 35, 80)));
                        }
                        vm_painter.line_segment([vm_rect.left_bottom(), vm_rect.right_top()], Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(100, 80, 60, 60)));

                        // Soft-knee dynamic compression curve preview (16 points)
                        let norm_th = crate::views::vari_mu_master_view::VariMuMasterView::threshold_to_normalized(thresh);
                        let norm_drv = crate::views::vari_mu_master_view::VariMuMasterView::drive_to_normalized(drive);
                        let steps = 16;
                        let mut prev_pt = None;
                        for i in 0..=steps {
                            let nx = i as f32 / steps as f32;
                            let ny = if nx <= norm_th {
                                nx * (0.8 + 0.2 * norm_drv)
                            } else {
                                norm_th * (0.8 + 0.2 * norm_drv) + (nx - norm_th) / ratio
                            };
                            let px = vm_rect.left() + nx * vm_rect.width();
                            let py = vm_rect.bottom() - ny.clamp(0.0, 1.0) * (vm_rect.height() - 10.0) - 5.0;
                            let cur_pt = egui::pos2(px, py);
                            if let Some(prev) = prev_pt {
                                vm_painter.line_segment([prev, cur_pt], Stroke::new(1.3_f32, Color32::from_rgb(255, 170, 50)));
                            }
                            prev_pt = Some(cur_pt);
                        }

                        // Puck handle
                        let puck_x = vm_rect.left() + norm_th * vm_rect.width();
                        let puck_y = vm_rect.bottom() - norm_drv * (vm_rect.height() - 12.0) - 6.0;
                        vm_painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::from_rgb(255, 170, 50));
                        vm_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(255, 220, 150)));

                        // Readout
                        vm_painter.text(
                            egui::pos2(vm_rect.right() - 4.0, vm_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("VARI-MU TUBE\nRatio:{:.1}:1\nTh:{:.0} | Drv:{:.0}\nTHD:{:.2}%", ratio, thresh, drive, thd),
                            FontId::proportional(7.5),
                            Color32::from_rgb(255, 170, 50),
                        );

                        if vm_resp.hovered() {
                            let _ = vm_resp.on_hover_text(format!(
                                "Mastering Dual-Mono Variable-Mu Vacuum Tube Optical Compressor HUD\nThreshold: {:.1} dB | Drive: {:+.1} dB\nRatio: {:.1}:1 | THD: {:.2}%\n[Drag: Thresh (X) / Drive (Y) Puck]",
                                thresh, drive, ratio, thd
                            ));
                        }
                    } else {
                        // Filter Curve Mini View (slot-addressed)
                        let (filt_resp, filt_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let filt_rect = filt_resp.rect;
                        filt_painter.rect_filled(filt_rect, 2.0, Color32::from_rgb(8, 12, 20));
                        if filt_resp.dragged() {
                            if let Some(pos) = filt_resp.interact_pointer_pos() {
                                let nx = ((pos.x - filt_rect.left()) / filt_rect.width()).clamp(0.05, 0.95);
                                let ny = (1.0 - ((pos.y - filt_rect.top()) / filt_rect.height())).clamp(0.05, 0.95);
                                state.filter_nodes[1] = (nx, ny);
                                state.cutoff = nx;
                                state.resonance = ny;
                                state.node_param_values.insert("cutoff".to_string(), nx);
                                state.node_param_values.insert("resonance".to_string(), ny);
                                if let Some(bus) = param_bus {
                                    let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_cut).is_some() { bus.set(pid_cut, nx); }
                                    let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_res).is_some() { bus.set(pid_res, ny); }
                                }
                            }
                        }
                        let steps = 30;
                        let mut curve_pts = Vec::with_capacity(steps);
                        for i in 0..steps {
                            let t = i as f32 / (steps - 1) as f32;
                            let px = filt_rect.left() + t * filt_rect.width();
                            let cutoff_norm = state.filter_nodes[1].0;
                            let q = state.filter_nodes[1].1;
                            let gain = if t <= cutoff_norm {
                                1.0 + (t / cutoff_norm) * (q - 0.5) * 0.6
                            } else {
                                let roll = (t - cutoff_norm) / (1.0 - cutoff_norm).max(0.01);
                                (1.0 + (q - 0.5) * 0.6) * (-roll * 3.5).exp()
                            };
                            let py = filt_rect.bottom() - (gain * filt_rect.height() * 0.70).clamp(2.0, filt_rect.height() - 2.0);
                            curve_pts.push(egui::pos2(px, py));
                        }
                        for w in curve_pts.windows(2) {
                            filt_painter.line_segment([w[0], w[1]], Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                        }
                        for &(nx, ny) in &state.filter_nodes {
                            let px = filt_rect.left() + nx * filt_rect.width();
                            let py = filt_rect.top() + (1.0 - ny) * filt_rect.height();
                            filt_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(56, 189, 248));
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // Main Parameters Drawer (Scrollable Grid of Cards)
        egui::ScrollArea::horizontal()
            .id_source("pro_device_rack_scroll_area")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(desc) = opt_desc {
                    let chunks: Vec<&[crate::dsp_node_ui::DspParamSchema]> = desc.params.chunks(2).collect();
                    for (chunk_idx, chunk) in chunks.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.set_width(128.0);
                            for (item_idx, p) in chunk.iter().enumerate() {
                                let p_i = chunk_idx * 2 + item_idx;
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(18, 24, 36))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                                    .rounding(Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let name_lbl = ui.label(RichText::new(&p.name).font(FontId::proportional(9.5)).strong().color(Color32::from_rgb(226, 232, 240)));
                                            if name_lbl.secondary_clicked() {
                                                state.requested_automation_param = Some(p.id.clone());
                                            }
                                            if ui.button(RichText::new("📈").font(FontId::proportional(8.5)).color(Color32::from_rgb(234, 179, 8)))
                                                .on_hover_text(format!("Open Live Bézier Automation Lane for {}", p.name))
                                                .clicked()
                                            {
                                                state.requested_automation_param = Some(p.id.clone());
                                            }
                                            if p.macro_role != crate::dsp_node_ui::MacroRole::None {
                                                let (role_label, role_col) = match p.macro_role {
                                                    crate::dsp_node_ui::MacroRole::Tone => ("T", Color32::from_rgb(56, 189, 248)),
                                                    crate::dsp_node_ui::MacroRole::Space => ("S", Color32::from_rgb(99, 102, 241)),
                                                    crate::dsp_node_ui::MacroRole::Punch => ("P", Color32::from_rgb(239, 68, 68)),
                                                    crate::dsp_node_ui::MacroRole::Character => ("C", Color32::from_rgb(245, 158, 11)),
                                                    _ => ("", Color32::WHITE),
                                                };
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    egui::Frame::none()
                                                        .fill(Color32::from_rgba_unmultiplied(role_col.r(), role_col.g(), role_col.b(), 40))
                                                        .rounding(Rounding::same(2.0))
                                                        .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                                                        .show(ui, |ui| {
                                                            ui.label(RichText::new(role_label).font(FontId::proportional(8.0)).strong().color(role_col));
                                                        });
                                                });
                                            }
                                        });

                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::Toggle { default } => {
                                                if *default { 1.0 } else { 0.0 }
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::EnumChoice { default_idx, .. } => {
                                                *default_idx as f32
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let mut req_automation = false;

                                        match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, unit, is_logarithmic, .. } => {
                                                let actual_val = *min + val * (*max - *min);
                                                ui.horizontal(|ui| {
                                                    let dial_resp = draw_rotary_dial(ui, "", &mut val, cat_accent, default_norm);
                                                    if dial_resp.secondary_clicked() {
                                                        req_automation = true;
                                                    }
                                                    ui.vertical(|ui| {
                                                        ui.label(RichText::new(format!("{:.1} {}", actual_val, unit)).font(FontId::proportional(8.5)).color(Color32::from_rgb(56, 189, 248)));
                                                        let slider = if *is_logarithmic {
                                                            egui::Slider::new(&mut val, 0.0..=1.0).logarithmic(true).show_value(false)
                                                        } else {
                                                            egui::Slider::new(&mut val, 0.0..=1.0).show_value(false)
                                                        };
                                                        let slider_resp = ui.add(slider);
                                                        if slider_resp.secondary_clicked() {
                                                            req_automation = true;
                                                        }
                                                        if slider_resp.double_clicked() {
                                                            val = default_norm;
                                                        }
                                                    });
                                                });
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, unit, .. } => {
                                                let actual_val = *min + val * (*max - *min);
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(format!("{:.1} {}", actual_val, unit)).font(FontId::proportional(8.5)).color(Color32::from_rgb(56, 189, 248)));
                                                    let slider_resp = ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(false));
                                                    if slider_resp.secondary_clicked() {
                                                        req_automation = true;
                                                    }
                                                    if slider_resp.double_clicked() {
                                                        val = default_norm;
                                                    }
                                                });
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::Toggle { .. } => {
                                                let mut b = val >= 0.5;
                                                let tog_text = if b { "ON" } else { "OFF" };
                                                let tog_col = if b { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(148, 163, 184) };
                                                let tog_btn = ui.button(RichText::new(tog_text).font(FontId::proportional(9.0)).color(tog_col));
                                                if tog_btn.clicked() {
                                                    b = !b;
                                                    val = if b { 1.0 } else { 0.0 };
                                                }
                                                if tog_btn.secondary_clicked() {
                                                    req_automation = true;
                                                }
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::EnumChoice { choices, .. } => {
                                                let idx = (val as usize).min(choices.len().saturating_sub(1));
                                                let cur_choice = choices.get(idx).cloned().unwrap_or_default();
                                                egui::ComboBox::from_id_source(format!("pro_rack_combo_{}_{}", desc.kind_id, p.id))
                                                    .selected_text(RichText::new(&cur_choice).font(FontId::proportional(9.0)).color(cat_accent))
                                                    .show_ui(ui, |ui| {
                                                        for (c_i, choice) in choices.iter().enumerate() {
                                                            ui.selectable_value(&mut val, c_i as f32, choice);
                                                        }
                                                    });
                                            }
                                            _ => {
                                                let slider_resp = ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(false));
                                                if slider_resp.secondary_clicked() {
                                                    req_automation = true;
                                                }
                                                if slider_resp.double_clicked() {
                                                    val = default_norm;
                                                }
                                            }
                                        }

                                        state.node_param_values.insert(p.id.clone(), val);
                                        if req_automation {
                                            state.request_automation(&p.id);
                                        }

                                        if let Some(bus) = param_bus {
                                            let pid_pro = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                            if bus.get(pid_pro).is_some() {
                                                bus.set(pid_pro, val);
                                            }
                                            let pid_multi = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + p_i as u32);
                                            if bus.get(pid_multi).is_some() {
                                                bus.set(pid_multi, val);
                                            }
                                            let pid_compat = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + p_i as u32);
                                            if bus.get(pid_compat).is_some() {
                                                bus.set(pid_compat, val);
                                            }
                                        }

                                        // Synchronize standard dial values if modified
                                        match p.id.as_str() {
                                            "cutoff" => {
                                                state.cutoff = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "resonance" => {
                                                state.resonance = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "decay" => {
                                                state.decay = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 2);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "env_decay" => {
                                                state.env_decay = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 3);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "mod_amt" => {
                                                state.mod_amt = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "drive" => {
                                                state.drive = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 5);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "osc_mix" => {
                                                state.osc_mix = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 6);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "shape" => {
                                                state.shape = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "volume" => {
                                                state.volume = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "lfo_speed" => {
                                                state.lfo_speed = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 8);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "lfo_depth" => {
                                                state.lfo_depth = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 9);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            _ => {}
                                        }
                                    });
                                ui.add_space(4.0);
                            }
                        });
                        ui.add_space(4.0);
                    }
                } else {
                    ui.label(RichText::new("No descriptor for selected module").font(FontId::proportional(11.0)).color(Color32::from_rgb(148, 163, 184)));
                }
            });
    });
}

#[cfg(feature = "gui")]
fn draw_rotary_dial(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    ring_color: Color32,
    default_val: f32,
) -> egui::Response {
    let size = Vec2::new(38.0, 52.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
    }
    if resp.double_clicked() {
        *value = default_val.clamp(0.0, 1.0);
    }
    if resp.hovered() && !label.is_empty() && label != "---" {
        resp = resp.on_hover_text(format!(
            "{}: {:.0}%\n[Drag to adjust | Double-click to reset | Right-click to automate]",
            label,
            *value * 100.0
        ));
    }

    let center = egui::pos2(rect.center().x, rect.top() + 18.0);
    let radius = 14.0;

    // Background circle
    painter.circle_filled(center, radius, Color32::from_rgb(12, 16, 26));
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));

    // Active arc ring (from -135 deg to +135 deg)
    let start_angle = -std::f32::consts::PI * 0.75;
    let end_angle = start_angle + (*value * std::f32::consts::PI * 1.5);

    let arc_steps = 16;
    let mut prev_arc = None;
    for i in 0..=arc_steps {
        let a = start_angle + (i as f32 / arc_steps as f32) * (end_angle - start_angle);
        let pt = egui::pos2(center.x + a.cos() * (radius - 1.5), center.y + a.sin() * (radius - 1.5));
        if let Some(last) = prev_arc {
            painter.line_segment([last, pt], Stroke::new(2.5_f32, ring_color));
        }
        prev_arc = Some(pt);
    }

    // Pointer notch indicator
    let notch_pt = egui::pos2(center.x + end_angle.cos() * (radius - 3.0), center.y + end_angle.sin() * (radius - 3.0));
    painter.line_segment([center, notch_pt], Stroke::new(1.5_f32, Color32::WHITE));

    // Label
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 38.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        Color32::from_rgb(148, 163, 184),
    );

    resp
}

#[cfg(feature = "gui")]
fn draw_vertical_fader(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    default_val: f32,
) -> egui::Response {
    let size = Vec2::new(28.0, 110.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
    }
    if resp.double_clicked() {
        *value = default_val.clamp(0.0, 1.0);
    }
    if resp.hovered() && !label.is_empty() {
        resp = resp.on_hover_text(format!(
            "{}: {:.1}\n[Drag to adjust | Double-click to reset | Right-click to automate]",
            label,
            *value * 10.0
        ));
    }

    let track_x = rect.center().x;
    let track_top = rect.top() + 18.0;
    let track_bottom = rect.bottom() - 16.0;

    // Fader slot track
    painter.line_segment(
        [egui::pos2(track_x, track_top), egui::pos2(track_x, track_bottom)],
        Stroke::new(2.0_f32, Color32::from_rgb(10, 14, 22)),
    );

    // Fader thumb handle (amber cap)
    let thumb_y = track_bottom - (*value * (track_bottom - track_top));
    let thumb_rect = Rect::from_center_size(egui::pos2(track_x, thumb_y), Vec2::new(18.0, 8.0));
    painter.rect_filled(thumb_rect, 2.0, Color32::from_rgb(245, 158, 11));
    painter.rect_stroke(thumb_rect, 2.0, Stroke::new(1.0_f32, Color32::WHITE));

    // Top Label
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 6.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        Color32::from_rgb(148, 163, 184),
    );

    // Bottom Value
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 6.0),
        egui::Align2::CENTER_CENTER,
        format!("{:.1}", *value * 10.0),
        FontId::proportional(8.0),
        Color32::from_rgb(200, 215, 235),
    );

    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_device_rack_defaults() {
        let state = ModernDeviceRackState::default();
        assert!(state.is_enabled);
        assert_eq!(state.selected_node_kind.as_deref(), Some("AetherSynth"));
        assert_eq!(state.device_name, "Synth 1");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_headless_rendering_dynamic_modules() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        let modules_to_test = [
            "AetherSynth",
            "DemucsV4Separator",
            "SingingSynthesisNode",
            "AiAutonomousMasteringEngine",
            "PluckedStringNode",
            "BowedString",
            "FilterSVF",
            "NavierStokesFluidNode",
        ];

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for mod_kind in modules_to_test {
                    state.selected_node_kind = Some(mod_kind.to_string());
                    show_modern_device_rack(ui, &mut state, None);
                }
            });
        });

        assert!(!state.node_param_values.is_empty());
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_drawer_minimize_expand_toggle() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        // 1. Initial expanded state
        assert!(!state.is_minimized);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        // 2. Collapse to drawer mode
        state.is_minimized = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });
        assert!(state.is_minimized);

        // 3. Expand back to full modular rack
        state.is_minimized = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });
        assert!(!state.is_minimized);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_dynamic_faders_and_aux_reflection() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        // Test with AI Mastering Engine (contains multiple parameters spanning faders & aux)
        state.selected_node_kind = Some("AiAutonomousMasteringEngine".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        assert!(state.node_param_values.contains_key("target_lufs"));
        assert!(state.node_param_values.contains_key("ceiling_db"));
        assert!(state.node_param_values.contains_key("punch"));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_expanded_pro_params_drawer() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        state.selected_node_kind = Some("AetherSynth".to_string());
        state.is_expanded_params = true;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        assert!(state.is_expanded_params);
        assert!(!state.node_param_values.is_empty());
        assert!(state.node_param_values.contains_key("cutoff"));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_with_context_and_live_parambus() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();
        let mut bus = summoner_core::param_bus::ParamBus::new();
        bus.register(summoner_core::param_bus::ParamId(1000), 0.5);
        bus.register(summoner_core::param_bus::ParamId(1001), 0.5);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), 1);
            });
        });

        assert_eq!(state.device_name, "Synth 1");
    }
}
