// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#[cfg(feature = "gui")]
use eframe::egui;
use summoner_core::node::KNOWN_NODE_TYPES;

#[derive(Debug, Clone)]
pub struct CommandAction {
    pub label: String,
    pub category: String,
    pub action_id: String,
    pub shortcut_hint: Option<String>,
}

/// Character-subsequence fuzzy matcher for command palette search.
/// Returns `Some(score)` if all characters of `query` appear in sequence within `target`.
pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.to_lowercase().chars().collect();

    let mut q_idx = 0;
    let mut score = 0;
    let mut last_match_idx = 0;

    for (t_idx, &t_char) in target_chars.iter().enumerate() {
        if q_idx < query_chars.len() && t_char == query_chars[q_idx] {
            if q_idx == 0 {
                if t_idx == 0 {
                    score += 50;
                }
            } else {
                if t_idx == last_match_idx + 1 {
                    score += 30;
                } else {
                    score -= (t_idx - last_match_idx) as i32 * 2;
                }
            }
            score += 10;
            last_match_idx = t_idx;
            q_idx += 1;
        }
    }

    if q_idx == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Command Palette for rapid keyboard-first navigation (Ctrl+K / Cmd+K).
pub struct CommandPalette {
    pub is_open: bool,
    pub search_query: String,
    pub actions: Vec<CommandAction>,
    pub selected_index: usize,
    pub recent_action_ids: Vec<String>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut actions = vec![
            CommandAction {
                label: "Switch to Arranger View".into(),
                category: "Navigation".into(),
                action_id: "nav_arranger".into(),
                shortcut_hint: Some("Ctrl+1".into()),
            },
            CommandAction {
                label: "Switch to Console Mixer".into(),
                category: "Navigation".into(),
                action_id: "nav_mixer".into(),
                shortcut_hint: Some("Ctrl+2".into()),
            },
            CommandAction {
                label: "Switch to Live Stage Performance".into(),
                category: "Navigation".into(),
                action_id: "nav_performance".into(),
                shortcut_hint: Some("Ctrl+3".into()),
            },
            CommandAction {
                label: "Open Selected Track Node Graph".into(),
                category: "Navigation".into(),
                action_id: "nav_nodegraph".into(),
                shortcut_hint: Some("Ctrl+4".into()),
            },
            CommandAction {
                label: "Toggle Play / Stop Transport".into(),
                category: "Transport".into(),
                action_id: "transport_play".into(),
                shortcut_hint: Some("Space".into()),
            },
            CommandAction {
                label: "Toggle Record All Automation".into(),
                category: "Transport".into(),
                action_id: "transport_record".into(),
                shortcut_hint: Some("R".into()),
            },
            CommandAction {
                label: "PANIC - Stop All Audio Immediately".into(),
                category: "Control".into(),
                action_id: "panic".into(),
                shortcut_hint: Some("Esc".into()),
            },
            CommandAction {
                label: "Set Project BPM".into(),
                category: "Transport".into(),
                action_id: "set_bpm".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Add Audio Track".into(),
                category: "Session".into(),
                action_id: "add_track".into(),
                shortcut_hint: Some("Ctrl+T".into()),
            },
            CommandAction {
                label: "Render WAV Audio".into(),
                category: "Session".into(),
                action_id: "render_wav".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "SFZ Instrument Convert".into(),
                category: "Session".into(),
                action_id: "sfz_convert".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Auto-Slice Sample File".into(),
                category: "Session".into(),
                action_id: "auto_slice".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Load Sampler Preset".into(),
                category: "Session".into(),
                action_id: "load_preset".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Export CLAP Plugin".into(),
                category: "Session".into(),
                action_id: "export_clap".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle SIMD Acceleration".into(),
                category: "Control".into(),
                action_id: "toggle_simd".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Patch Browser".into(),
                category: "Navigation".into(),
                action_id: "toggle_patch_browser".into(),
                shortcut_hint: Some("Ctrl+B".into()),
            },
            CommandAction {
                label: "Toggle High Contrast Mode".into(),
                category: "Accessibility".into(),
                action_id: "toggle_high_contrast".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Reduce Motion Mode".into(),
                category: "Accessibility".into(),
                action_id: "toggle_reduce_motion".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Accessibility Settings".into(),
                category: "Accessibility".into(),
                action_id: "open_accessibility_settings".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Audio Driver Settings & Native Device Selector".into(),
                category: "Settings".into(),
                action_id: "open_audio_driver_settings".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Loudness & Peak Headroom Meter".into(),
                category: "View".into(),
                action_id: "toggle_loudness_meter".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "MPE Expression Curve Editor".into(),
                category: "Tools".into(),
                action_id: "open_mpe_editor".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Project Backup Snapshots Manager".into(),
                category: "File".into(),
                action_id: "open_backup_manager".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Neural Audio Style Transfer Preview".into(),
                category: "Tools".into(),
                action_id: "open_style_transfer".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Multi-Channel Spectral Equalizer".into(),
                category: "Tools".into(),
                action_id: "open_spectral_eq".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Docking Grid Layout Manager".into(),
                category: "GUI".into(),
                action_id: "open_docking_layout".into(),
                shortcut_hint: Some("Ctrl+D".into()),
            },
            CommandAction {
                label: "Open Modular Synth Patch Matrix".into(),
                category: "GUI".into(),
                action_id: "open_patch_matrix".into(),
                shortcut_hint: Some("Ctrl+M".into()),
            },
            CommandAction {
                label: "Open High-DPI Multi-Track Oscilloscope".into(),
                category: "GUI".into(),
                action_id: "open_oscilloscope".into(),
                shortcut_hint: Some("Ctrl+O".into()),
            },
            CommandAction {
                label: "Toggle DAW Transport Header Bar".into(),
                category: "GUI".into(),
                action_id: "toggle_transport_bar".into(),
                shortcut_hint: Some("Ctrl+H".into()),
            },
            CommandAction {
                label: "Open Touch Control & Slider Rack".into(),
                category: "GUI".into(),
                action_id: "open_touch_controls".into(),
                shortcut_hint: Some("Ctrl+K".into()),
            },
            // Tier 49 Actions (Steps 1301-1305)
            CommandAction {
                label: "Open 3D Spatial Audio Panner Canvas".into(),
                category: "GUI".into(),
                action_id: "open_spatial_panner".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Sample Editor & Transient Slicer".into(),
                category: "GUI".into(),
                action_id: "open_sample_editor".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Theme Customizer & Contrast Preview".into(),
                category: "GUI".into(),
                action_id: "open_theme_customizer".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Performance HUD Telemetry Overlay".into(),
                category: "GUI".into(),
                action_id: "toggle_hud_overlay".into(),
                shortcut_hint: None,
            },
            // Tier 50 Actions (Steps 1321-1325)
            CommandAction {
                label: "Open Touch-Optimized Live Performance Macro Rack".into(),
                category: "GUI".into(),
                action_id: "open_live_macro_rack".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Real-Time 3D FFT Waterfall Spectrogram".into(),
                category: "GUI".into(),
                action_id: "open_spectrogram_3d".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Keyboard Shortcuts & Keybinding Editor".into(),
                category: "GUI".into(),
                action_id: "open_keybinding_editor".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Track Peak Metering Bridge Panel".into(),
                category: "GUI".into(),
                action_id: "open_meter_bridge".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open High-DPI Display Scaling & Calibration".into(),
                category: "GUI".into(),
                action_id: "open_dpi_scale_panel".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Quartz Crystal Singing Bowl & Glass Chalice HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_crystal_resonator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Franklin Glass Armonica & Spindle HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_glass_armonica_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Hurdy-Gurdy (Vielle à roue) Performance HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_hurdy_gurdy_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Trompette Chien Buzzing Bridge HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_trompette_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 5th-Order Ambisonics (HOA5) 3D Radar HUD".into(),
                category: "Spatial & Ambisonics".into(),
                action_id: "open_hoa5_radar_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 2D Neural Timbre Morphing Orb HUD".into(),
                category: "Neural AI & Resynthesis".into(),
                action_id: "open_neural_morph_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Advanced Stems Export Modal & Loudness Inspector".into(),
                category: "Project & Export".into(),
                action_id: "open_stems_export_modal".into(),
                shortcut_hint: Some("Ctrl+Shift+E".into()),
            },
            CommandAction {
                label: "Open Bowed String Acoustic Friction HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_bowed_string_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Shakuhachi Bamboo Flute HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_shakuhachi_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Sitar Curved Jawari Bridge HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_sitar_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Turkish Ney Flute Embouchure HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_turkish_ney_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Caribbean Steelpan Annular Ring Resonance HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_steelpan_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Lamellophone Mbira & Kalimba Tine Resonance HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_mbira_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Japanese 13-String Koto Paulownia Zither HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_koto_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Balinese Gamelan Gender Metallophone HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_gamelan_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Hammered Dulcimer & Cimbalom String Dispersion HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_dulcimer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Electromechanical Clavinet D6 String-Anvil HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_clavinet_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Concert Grand Piano 3-String Unison & Felt Dynamics HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_grand_piano_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Pipe Organ Windchest & Flue Turbulence HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_pipe_organ_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Electromechanical Tine & Reed Electric Piano HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_electric_piano_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 9-Drawbar Tonewheel Organ Console HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_tonewheel_organ_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Waveguide Brass Acoustic Lip-Reed HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_waveguide_brass_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Vocal Tract 44-Cylinder Area Function HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_vocal_tract_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Vintage Rotary Speaker Cabinet & Doppler HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_rotary_speaker_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Free-Reed Aeroelastic Phase Portrait HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_free_reed_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Granular Synthesis Cloud Grain Dispersion HUD".into(),
                category: "Synthesis & Granular".into(),
                action_id: "open_granular_cloud_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Spring-Mass Lattice Deformation & Dispersion HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_spring_lattice_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 2D Physical Waveguide Resonator Mesh HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_waveguide_mesh_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Plucked & Struck Waveguide String HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_plucked_string_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Pneumatic Bellows Dynamics HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_bellows_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Sitar Curved Jawari Bridge HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_jawari_bridge_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Spruce Soundboard & Bridge HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_soundboard_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Sympathetic Resonance Coupling HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_sympathetic_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Woodwind Air-Jet Embouchure HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_woodwind_jet_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Woodwind 6-Tonehole Radiation HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_tonehole_matrix_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mechanical Plate Dispersion & APDN Reverb HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_plate_dispersion_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Membrane Cavity Phase & Drum Displacement HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_membrane_cavity_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Shakuhachi Embouchure & Meri/Kari HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_embouchure_angle_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Bowed String Stick-Slip Friction & Orbit HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_friction_orbit_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Pipe Organ Rank Voicing & Flue Cutup HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_rank_voicing_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Electric Piano Tine Resonator & Tremolo Orbit HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_tine_resonator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Spring Reverb Tank & Mechanical Dispersion HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_spring_reverb_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Spectral Comb Resonator & Matrix HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_comb_resonator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Wavefront Reflection Raytracing HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_wavefront_reflection_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Diffractive Wave Propagation HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_diffractive_propagation_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Elastic Drumhead Membrane Resonator HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_membrane_resonator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Coupled 2D Membrane & Plate Resonator HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_membrane_plate_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Struck Idiophone Modal Resonator HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_idiophone_spectrum_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Underwater Acoustic Sonar & Hydrophone HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_sonar_hydrophone_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 5th-Order Ambisonics HOA5 Binaural HUD".into(),
                category: "Spatial".into(),
                action_id: "open_hoa5_binaural_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open MPEG-H 3D Immersive Spatializer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_mpegh_3d_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dolby Atmos 9.1.6 3D Room Raytracer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_atmos_916_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Auro-3D 13.1 Tri-Level Spatializer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_auro3d_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open NHK 22.2 Super Hi-Vision Hemispherical Spatializer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_nhk222_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Wave Field Synthesis (WFS) Holographic Linear Array HUD".into(),
                category: "Spatial".into(),
                action_id: "open_wfs_array_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Spatial Binaural HRTF 3D Orbit Spatializer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_binaural_panner_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Source Binaural BRIR Room Impulse Response Spatializer HUD"
                    .into(),
                category: "Spatial".into(),
                action_id: "open_binaural_brir_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Continuous 3D Kinematic Spatializer & Doppler Shift Engine HUD".into(),
                category: "Spatial".into(),
                action_id: "open_continuous_doppler_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 3D Shoebox Room Acoustics & Image-Source Reverb HUD".into(),
                category: "Spatial".into(),
                action_id: "open_spatial_room_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Stereo Rotary Speaker Doppler & Acoustic Cabinet HUD".into(),
                category: "Acoustic".into(),
                action_id: "open_rotary_doppler_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Acoustic Early Reflections Stochastic Raytracer HUD".into(),
                category: "Spatial".into(),
                action_id: "open_raytraced_reverb_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 7.1.4 Dolby Atmos 3D Immersive Bed & Object Radar HUD".into(),
                category: "Spatial".into(),
                action_id: "open_atmos_surround_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 7.1.4 Dolby Atmos Spherical Proximity & Distance HUD".into(),
                category: "Spatial".into(),
                action_id: "open_atmos_proximity_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open MPEG-H 3D Dynamic Object Trajectory HUD".into(),
                category: "Spatial".into(),
                action_id: "open_mpegh_trajectory_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open NHK 22.2 3-Layer Spherical Immersion HUD".into(),
                category: "Spatial".into(),
                action_id: "open_nhk222_immersion_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open HOA 4th-Order (25-ch) 3D Ambisonic Energy Density HUD".into(),
                category: "Spatial".into(),
                action_id: "open_hoa4_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD".into(),
                category: "Spatial".into(),
                action_id: "open_mpegh_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 3D Spatial Trajectory Polar Canvas & Granular Cloud HUD".into(),
                category: "Spatial".into(),
                action_id: "open_spatial_trajectory_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 3D Ambisonic Trajectory Radar & Surround Sound Panner HUD".into(),
                category: "Spatial".into(),
                action_id: "open_ambisonic_radar_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dolby Atmos HOA 7.1.4 3D Spatializer & Head-Tracking HUD".into(),
                category: "Spatial".into(),
                action_id: "open_hoa_spatializer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Timbre Morphing Diffusion Vocoder & Resynthesizer HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_timbre_morph_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Continuous Spatial Neural Radiance Field & HRTF/BRIR HUD".into(),
                category: "Spatial".into(),
                action_id: "open_neural_radiance_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Polyphonic Vocal Stylizer & Microtonal Ornament HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_vocal_stylizer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Speech-to-Singing Synthesis & Pitch Retuning HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_speech_to_singing_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Polyphonic Choir Formant Morpher HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_choir_formant_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Stage Neural Vocoder Formant Morpher HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_vocoder_morph_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Wavetable Morphing Synthesizer HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_wavetable_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Phoneme Morphing & Style Transfer HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_phoneme_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Blind Dereverberation & Deconvolution HUD".into(),
                category: "Restoration".into(),
                action_id: "open_neural_dereverb_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Audio Super-Resolution & Bandwidth Extension HUD".into(),
                category: "Restoration".into(),
                action_id: "open_neural_super_resolution_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Amp Modeler (NAM) WaveNet Guitar & Bass Amplifier HUD".into(),
                category: "Amplification".into(),
                action_id: "open_nam_amp_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Diffusion Audio Inpainter & Generative Spectral Repair HUD"
                    .into(),
                category: "Restoration".into(),
                action_id: "open_neural_inpaint_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural Timbre Transfer Morphing Resynthesizer HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_neural_timbre_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Tactile Bitcrusher & Quantization Morphology HUD".into(),
                category: "Distortion / Lo-Fi".into(),
                action_id: "open_bitcrusher_morph_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Voice Analog Bucket-Brigade (BBD) Chorus Matrix HUD".into(),
                category: "Modulation / Time".into(),
                action_id: "open_bbd_chorus_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Physical Vacuum Tube Triode Bias HUD".into(),
                category: "Distortion / Saturation".into(),
                action_id: "open_tube_bias_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Reel-to-Reel Tape Saturation & Hysteresis HUD".into(),
                category: "Distortion / Saturation".into(),
                action_id: "open_tape_saturation_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Ladder 24dB Filter & Self-Oscillation HUD".into(),
                category: "Filter / EQ".into(),
                action_id: "open_ladder_filter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Granular Spectral Cloud Freeze & Trajectory HUD".into(),
                category: "Granular / Time".into(),
                action_id: "open_granular_freeze_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dynamic Resonant Auto-Wah Filter HUD".into(),
                category: "Filter / EQ".into(),
                action_id: "open_auto_wah_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 5-Vowel Phonetic Formant Filter Resonator HUD".into(),
                category: "Filter / EQ".into(),
                action_id: "open_formant_filter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering True-Peak 8x Oversampled Limiter HUD".into(),
                category: "Dynamics & Mastering".into(),
                action_id: "open_oversampled_limiter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Dual-Mono Variable-Mu Vacuum Tube Optical Compressor HUD"
                    .into(),
                category: "Dynamics & Mastering".into(),
                action_id: "open_vari_mu_master_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Tape Flanger & Through-Zero HUD".into(),
                category: "Modulation".into(),
                action_id: "open_through_zero_flanger_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Tap Delay Matrix & Spatial Bounce HUD".into(),
                category: "Delay & Reverb".into(),
                action_id: "open_multitap_delay_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Frequency Shifter & SSB Modulator HUD".into(),
                category: "Modulation".into(),
                action_id: "open_frequency_shifter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Vintage Optical & VCA Compressor HUD".into(),
                category: "Dynamics & Mastering".into(),
                action_id: "open_optical_compressor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dynamic Harmonic Exciter & Brilliance HUD".into(),
                category: "Distortion & Color".into(),
                action_id: "open_harmonic_exciter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Psychoacoustic Stereo Widener & Vector Scope HUD".into(),
                category: "Spatial & Surround".into(),
                action_id: "open_stereo_widener_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Psychoacoustic Subharmonic Synth & Phase Alignment HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_subharmonic_synth_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Tape Wow/Flutter & Hysteresis Saturation HUD".into(),
                category: "Distortion & Color".into(),
                action_id: "open_tape_flutter_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Tactile Transient Designer & Harmonic Punch HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_transient_designer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Multiband Upward Compressor (OTT) HUD".into(),
                category: "Dynamics & Mastering".into(),
                action_id: "open_upward_compressor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Band Dynamic Resonance Suppressor HUD".into(),
                category: "Dynamics & EQ".into(),
                action_id: "open_resonance_suppressor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 6-Operator FM Matrix Synthesizer HUD".into(),
                category: "Synthesis".into(),
                action_id: "open_fm_matrix_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dynamic Spectral De-Esser HUD".into(),
                category: "Restoration & Dynamics".into(),
                action_id: "open_spectral_deesser_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 64-Band Spectral Matching Linear-Phase EQ HUD".into(),
                category: "Mastering & EQ".into(),
                action_id: "open_spectral_matching_eq_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Master Bus Dynamic Multiband Saturator HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_multiband_saturator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multiband Stereo Imager & Correlation HUD".into(),
                category: "Spatial & Surround".into(),
                action_id: "open_multiband_imager_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Dynamic Multiband Clipper HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_multiband_clipper_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 4-Band Dynamic Expander & Noise Gate HUD".into(),
                category: "Dynamics & Gate".into(),
                action_id: "open_multiband_expander_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Linear-Phase Multiband De-Compressor HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_multiband_decompressor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Dynamic Stereo Width & Bass Mono HUD".into(),
                category: "Spatial & Surround".into(),
                action_id: "open_dynamic_stereo_width_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Multi-Band Dynamic Crest Shaper HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_dynamic_crest_shaper_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Linear-Phase Crossover & Limiter HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_linear_phase_crossover_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Multi-Bus Parallel Dynamic Saturator HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_parallel_transient_saturator_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Stereo Mid/Side Dynamic Punch & Mono-Bass HUD".into(),
                category: "Spatial & Surround".into(),
                action_id: "open_midside_focuser_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Multi-Band Dynamic Transient Clipper HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_transient_clipper_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Mastering Multi-Stage Magnetic Tape Flux Saturation HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_tape_flux_master_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Linear-Phase Dynamic Transient De-Clicker & Vinyl Restoration HUD".into(),
                category: "Restoration & Dynamics".into(),
                action_id: "open_transient_declicker_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Dynamic Transient Sidechain Expander/Gate & Hysteresis HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_transient_gate_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open ITU-R BS.1770-4 Dialog Gating & Speech Normalization HUD".into(),
                category: "Mastering & Dynamics".into(),
                action_id: "open_dialog_gating_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multiband Lookahead Transient Reconstructor & Crest Expander HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_transient_reconstructor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Psychoacoustic Stereo Transient Unwrapper & Spatial Depth HUD".into(),
                category: "Spatial & Binaural".into(),
                action_id: "open_transient_unwrapper_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Master Bus Multi-Point Spectral Dynamics Unmasker HUD".into(),
                category: "Spectral & Synthesis".into(),
                action_id: "open_spectral_unmasker_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Psychoacoustic Multi-Band Spectral Transient De-Bleed HUD".into(),
                category: "Spectral & Synthesis".into(),
                action_id: "open_spectral_debleed_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Multi-Band Transient Spectral Reshaper & Formant Envelope HUD".into(),
                category: "Spectral & Synthesis".into(),
                action_id: "open_spectral_reshaper_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Physical Modeling Wooden Cassotto Tone Chamber & Soundbox HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_cassotto_chamber_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Zero-Gravity Navier-Stokes Acoustic Fluid Dynamics & Molecular Resonator HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_zero_gravity_fluid_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Acoustic Waveguide Brass Horn Flare Reflection & Webster Horn Impedance HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_horn_reflection_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Acoustic Glottal Pulse Flow & Vocal Fold Vibration Phonation HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_glottal_pulse_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Physical Modeling Non-Linear Felt Hammer Strike & Sympathetic Resonance Matrix HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_hammer_strike_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Mixing Console Desk Emulation & Transformer Character HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_console_emulation_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Japanese Satsuma Biwa Silk String & Sawari Buzzing Bridge HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_biwa_sawari_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Discrete Schottky Diode Multi-Stage Wavefolder & Overdrive HUD".into(),
                category: "Dynamics & Saturation".into(),
                action_id: "open_schottky_wavefolder_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Pipe Organ Windchest Aerodynamics & Dynamic Reservoir HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_windchest_reservoir_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Vinyl Turntable Inertial Motor Brake Tape Stop & Stutter HUD".into(),
                category: "Creative & Glitch".into(),
                action_id: "open_tape_stop_glitch_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Quantum Bloch Sphere & State Tomography HUD".into(),
                category: "Quantum & Spectral Synthesis".into(),
                action_id: "open_quantum_bloch_sphere_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Chaotic Fractal Attractor & Dynamic Systems Modulation HUD".into(),
                category: "Modulation & Dynamics".into(),
                action_id: "open_chaotic_attractor_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Visual Spectrogram Art & Sonification Engine HUD".into(),
                category: "Spectral Resynthesis".into(),
                action_id: "open_spectrogram_art_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open 10-20 BCI EEG Brainwave Decoder & Neuro-Feedback HUD".into(),
                category: "Neural & Modulation".into(),
                action_id: "open_bci_eeg_decoder_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Analog Diode-Ring Modulator & Quadrature Frequency Doubler HUD".into(),
                category: "Modulation & Dynamics".into(),
                action_id: "open_ring_mod_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Psychoacoustic Shaped Noise Dither & Bit-Depth Truncation Radar HUD".into(),
                category: "Mastering & Utility".into(),
                action_id: "open_dither_quantizer_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Air-Jet Reed & Utaguchi Flue Vortex Aerodynamics HUD".into(),
                category: "Physical Modeling".into(),
                action_id: "open_air_reed_hud".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Neural ONNX 4-Stem Spectral Separator & Multi-Track Router HUD".into(),
                category: "Neural & Spectral".into(),
                action_id: "open_stem_separator_hud".into(),
                shortcut_hint: None,
            },
        ];

        // Add entries for all KNOWN_NODE_TYPES
        for &kind in KNOWN_NODE_TYPES {
            actions.push(CommandAction {
                label: format!("Add {} Node", kind),
                category: "DSP".into(),
                action_id: format!("add_node_{}", kind.to_lowercase()),
                shortcut_hint: None,
            });
        }

        Self {
            is_open: false,
            search_query: String::new(),
            actions,
            selected_index: 0,
            recent_action_ids: Vec::new(),
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn record_recent_action(&mut self, action_id: &str) {
        self.recent_action_ids.retain(|id| id != action_id);
        self.recent_action_ids.insert(0, action_id.to_string());
        if self.recent_action_ids.len() > 5 {
            self.recent_action_ids.truncate(5);
        }
    }

    /// Filters and ranks actions based on fuzzy matching.
    pub fn get_filtered_actions(&self) -> Vec<CommandAction> {
        if self.search_query.trim().is_empty() {
            let mut result = Vec::new();

            // Recently used section
            for recent_id in &self.recent_action_ids {
                if let Some(action) = self.actions.iter().find(|a| &a.action_id == recent_id) {
                    let mut recent_action = action.clone();
                    recent_action.category = "Recently Used".into();
                    result.push(recent_action);
                }
            }

            // All remaining actions
            for action in &self.actions {
                if !self.recent_action_ids.contains(&action.action_id) {
                    result.push(action.clone());
                }
            }
            result
        } else {
            let mut scored: Vec<(i32, CommandAction)> = self
                .actions
                .iter()
                .filter_map(|action| {
                    let target_str = format!("{} {}", action.category, action.label);
                    fuzzy_score(&self.search_query, &target_str).map(|s| (s, action.clone()))
                })
                .collect();

            scored.sort_by_key(|b| std::cmp::Reverse(b.0));
            scored.into_iter().map(|(_, act)| act).collect()
        }
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut executed_action = None;

        let alpha = ctx.animate_value_with_time(
            egui::Id::new("command_palette_fade"),
            if self.is_open { 1.0 } else { 0.0 },
            0.15,
        );

        if !self.is_open && alpha <= 0.001 {
            return None;
        }

        let filtered = self.get_filtered_actions();

        // Keyboard navigation
        let (up_pressed, down_pressed, enter_pressed, esc_pressed) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
            )
        });

        if esc_pressed {
            self.is_open = false;
            return None;
        }

        if !filtered.is_empty() {
            if up_pressed {
                if self.selected_index == 0 {
                    self.selected_index = filtered.len() - 1;
                } else {
                    self.selected_index -= 1;
                }
            }
            if down_pressed {
                self.selected_index = (self.selected_index + 1) % filtered.len();
            }
            if self.selected_index >= filtered.len() {
                self.selected_index = 0;
            }

            if enter_pressed {
                let action = &filtered[self.selected_index];
                executed_action = Some(action.action_id.clone());
                self.record_recent_action(&action.action_id);
                self.is_open = false;
            }
        }

        egui::Window::new("Command Palette (Ctrl+K)")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
            .fixed_size(egui::vec2(520.0, 320.0))
            .show(ctx, |ui| {
                let text_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Type a command or node kind (e.g. 'lf', 'wav', 'sine')...")
                        .desired_width(f32::INFINITY),
                );
                text_edit.request_focus();

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut current_category = String::new();
                    for (idx, action) in filtered.iter().enumerate() {
                        if self.search_query.trim().is_empty()
                            && action.category != current_category
                        {
                            current_category = action.category.clone();
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(&current_category)
                                    .strong()
                                    .color(egui::Color32::from_rgb(140, 160, 200)),
                            );
                        }

                        let is_sel = idx == self.selected_index;
                        ui.horizontal(|ui| {
                            let text = format!("[{}] {}", action.category, action.label);
                            let response = ui.selectable_label(is_sel, text);
                            if response.clicked() {
                                executed_action = Some(action.action_id.clone());
                                self.record_recent_action(&action.action_id);
                                self.is_open = false;
                            }
                            if let Some(ref hint) = action.shortcut_hint {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(hint)
                                                .small()
                                                .color(egui::Color32::from_rgb(120, 120, 140)),
                                        );
                                    },
                                );
                            }
                        });
                    }
                });
            });

        executed_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_fuzzy_lf_matches_lfo() {
        let _palette = CommandPalette::new();
        let score_lfo = fuzzy_score("lf", "OscLFO");
        let score_sine = fuzzy_score("lf", "OscSine");
        assert!(score_lfo.is_some());
        assert!(score_sine.is_none());

        let mut pal = CommandPalette::new();
        pal.open();
        pal.search_query = "lf".into();
        let filtered = pal.get_filtered_actions();
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|a| a.label.contains("OscLFO")));
    }

    #[test]
    fn test_command_palette_arrow_navigation() {
        let mut palette = CommandPalette::new();
        palette.open();
        let actions_count = palette.get_filtered_actions().len();

        assert_eq!(palette.selected_index, 0);

        // Arrow down
        palette.selected_index = (palette.selected_index + 1) % actions_count;
        assert_eq!(palette.selected_index, 1);

        // Arrow up wrap around
        palette.selected_index = 0;
        palette.selected_index = actions_count - 1;
        assert_eq!(palette.selected_index, actions_count - 1);
    }

    #[test]
    fn test_command_palette_enter_executes() {
        let mut palette = CommandPalette::new();
        palette.open();
        palette.selected_index = 0;
        let actions = palette.get_filtered_actions();
        let first_id = actions[0].action_id.clone();

        palette.record_recent_action(&first_id);
        palette.close();

        assert!(!palette.is_open);
        assert_eq!(palette.recent_action_ids[0], first_id);
    }
}
