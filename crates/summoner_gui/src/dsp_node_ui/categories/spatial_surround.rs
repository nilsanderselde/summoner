// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `SpatialSurround`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("Atmos916Spatializer".to_string(), DspNodeDescriptor::new("Atmos916Spatializer", "Dolby Atmos 9.1.6 Object Panner with HRTF", DspNodeCategory::SpatialSurround, "Dolby Atmos 9.1.6 Object Panner with HRTF")
        .with_param(DspParamSchema::knob("azimuth", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Horizontal Azimuth parameter"))
        .with_param(DspParamSchema::knob("elevation", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Vertical Elevation parameter"))
        .with_param(DspParamSchema::knob("distance", "Virtual Distance", 0.5, 20.0, 2.0, "m", MacroRole::Tone, "Virtual Distance parameter"))
        .with_param(DspParamSchema::knob("spread", "Object Size Spread", 0.0, 100.0, 15.0, "%", MacroRole::Tone, "Object Size Spread parameter"))
    );
    descriptors.insert("AmbisonicRadarSpatializer".to_string(), DspNodeDescriptor::new("AmbisonicRadarSpatializer", "Higher-Order Ambisonics HOA-5 Spherical Radar", DspNodeCategory::SpatialSurround, "Higher-Order Ambisonics HOA-5 Spherical Radar")
        .with_param(DspParamSchema::knob("yaw", "Radar Yaw", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Radar Yaw parameter"))
        .with_param(DspParamSchema::knob("pitch", "Radar Pitch", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Radar Pitch parameter"))
        .with_param(DspParamSchema::knob("roll", "Radar Roll", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Radar Roll parameter"))
        .with_param(DspParamSchema::knob("hoa_order", "Ambisonics Order", 1.0, 5.0, 3.0, "order", MacroRole::Character, "Ambisonics Order integer control"))
    );
    descriptors.insert("Auro3dSpatializer".to_string(), DspNodeDescriptor::new("Auro3dSpatializer", "Auro-3D 13.1 Tri-Layer Immersive Panner", DspNodeCategory::SpatialSurround, "Auro-3D 13.1 Tri-Layer Immersive Panner")
        .with_param(DspParamSchema::knob("lower_layer", "Lower Ear-Level Gain", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Lower Ear-Level Gain parameter"))
        .with_param(DspParamSchema::knob("height_layer", "Height Layer Gain", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Height Layer Gain parameter"))
        .with_param(DspParamSchema::knob("top_voice", "Top Voice of God", -12.0, 6.0, -3.0, "dB", MacroRole::Tone, "Top Voice of God parameter"))
        .with_param(DspParamSchema::knob("spread", "Surround Width Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Surround Width Spread parameter"))
    );
    descriptors.insert("Nhk222Spatializer".to_string(), DspNodeDescriptor::new("Nhk222Spatializer", "NHK 22.2 Super Hi-Vision Multi-Channel Matrix", DspNodeCategory::SpatialSurround, "NHK 22.2 Super Hi-Vision Multi-Channel Matrix")
        .with_param(DspParamSchema::knob("top_gain", "Top Layer (9ch)", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Top Layer (9ch) parameter"))
        .with_param(DspParamSchema::knob("mid_gain", "Middle Layer (10ch)", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Middle Layer (10ch) parameter"))
        .with_param(DspParamSchema::knob("bottom_gain", "Bottom Layer (3ch)", -12.0, 6.0, -2.0, "dB", MacroRole::Tone, "Bottom Layer (3ch) parameter"))
        .with_param(DspParamSchema::knob("lfe_sub", "Dual LFE Subwoofers", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Dual LFE Subwoofers parameter"))
    );
    descriptors.insert("BinauralBrirSpatializer".to_string(), DspNodeDescriptor::new("BinauralBrirSpatializer", "Binaural Room Impulse Response Convolver", DspNodeCategory::SpatialSurround, "Binaural Room Impulse Response Convolver")
        .with_param(DspParamSchema::knob("room_size", "Room Dimensions", 5.0, 50.0, 15.0, "m", MacroRole::Tone, "Room Dimensions parameter"))
        .with_param(DspParamSchema::choice("hrtf", "HRTF Profile", &["KEMAR Dummy Head", "CIPIC High-Res", "Listen Individual", "Sphaero HRIR"], 0, "HRTF Profile mode selector"))
        .with_param(DspParamSchema::knob("early_gain", "Early Reflections Gain", -24.0, 6.0, 0.0, "dB", MacroRole::Tone, "Early Reflections Gain parameter"))
        .with_param(DspParamSchema::knob("late_diffuse", "Late Diffuse Field", -24.0, 6.0, -6.0, "dB", MacroRole::Tone, "Late Diffuse Field parameter"))
    );
    descriptors.insert("BinauralSpatializerNode".to_string(), DspNodeDescriptor::new("BinauralSpatializerNode", "3D Binaural HRTF Spatializer with ITD & ILD", DspNodeCategory::SpatialSurround, "3D Binaural HRTF Spatializer with ITD & ILD")
        .with_param(DspParamSchema::knob("azim_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Azimuth Angle parameter"))
        .with_param(DspParamSchema::knob("elev_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Elevation Angle parameter"))
        .with_param(DspParamSchema::knob("dist_m", "Source Distance", 0.2, 20.0, 1.5, "m", MacroRole::Tone, "Source Distance parameter"))
        .with_param(DspParamSchema::toggle("itd_ild_mode", "Interaural Delay & Level On", true, "Interaural Delay & Level On toggle switch"))
    );
    descriptors.insert("HyperDimensionalTensorSpatializer".to_string(), DspNodeDescriptor::new("HyperDimensionalTensorSpatializer", "11-Dimensional Calabi-Yau Spatial Tensor Panner", DspNodeCategory::SpatialSurround, "11-Dimensional Calabi-Yau Spatial Tensor Panner")
        .with_param(DspParamSchema::knob("calabi_yau_dim", "Calabi-Yau Manifold Dims", 4.0, 11.0, 11.0, "dims", MacroRole::Character, "Calabi-Yau Manifold Dims integer control"))
        .with_param(DspParamSchema::knob("manifold_curvature", "Ricci Curvature Scaling", 0.1, 5.0, 1.8, "k", MacroRole::Tone, "Ricci Curvature Scaling parameter"))
        .with_param(DspParamSchema::knob("subspace_rotation_deg", "Subspace Rotation Angle", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Subspace Rotation Angle parameter"))
        .with_param(DspParamSchema::knob("dimensional_spread", "High-Dim Sound Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "High-Dim Sound Spread parameter"))
    );
    descriptors.insert("Holographic3dSpatialSoundfield".to_string(), DspNodeDescriptor::new("Holographic3dSpatialSoundfield", "Holographic 3D Spherical Wavefield Synthesizer", DspNodeCategory::SpatialSurround, "Holographic 3D Spherical Wavefield Synthesizer")
        .with_param(DspParamSchema::knob("spherical_radius_m", "Wavefield Sphere Radius", 0.5, 25.0, 3.0, "m", MacroRole::Tone, "Wavefield Sphere Radius parameter"))
        .with_param(DspParamSchema::knob("azimuth_rad", "Source Azimuth Angle", -3.14, 3.14, 0.0, "rad", MacroRole::Tone, "Source Azimuth Angle parameter"))
        .with_param(DspParamSchema::knob("elevation_rad", "Source Elevation Angle", -1.57, 1.57, 0.0, "rad", MacroRole::Tone, "Source Elevation Angle parameter"))
        .with_param(DspParamSchema::knob("wavefront_curvature", "Virtual Wave Curvature", 0.1, 5.0, 1.2, "", MacroRole::Tone, "Virtual Wave Curvature parameter"))
    );
    descriptors.insert("AcousticCloakingSpatializer".to_string(), DspNodeDescriptor::new("AcousticCloakingSpatializer", "Acoustic Cloaking Phase Cancellation Spatializer", DspNodeCategory::SpatialSurround, "Acoustic Cloaking Phase Cancellation Spatializer")
        .with_param(DspParamSchema::knob("cloaking_radius_m", "Cloaking Shell Radius", 0.5, 10.0, 2.0, "m", MacroRole::Tone, "Cloaking Shell Radius parameter"))
        .with_param(DspParamSchema::knob("scattering_cancellation", "Phase Scattering Cancel", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Phase Scattering Cancel parameter"))
        .with_param(DspParamSchema::knob("shadow_attenuation_db", "Acoustic Shadow Depth", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Acoustic Shadow Depth parameter"))
        .with_param(DspParamSchema::knob("phase_invert_blend", "Metasurface Anti-Phase Blend", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Metasurface Anti-Phase Blend parameter"))
    );
    descriptors.insert("AtmosProximityNode".to_string(), DspNodeDescriptor::new("AtmosProximityNode", "Dolby Atmos Proximity Effect & Distance Attenuator", DspNodeCategory::SpatialSurround, "Dolby Atmos Proximity Effect & Distance Attenuator")
        .with_param(DspParamSchema::knob("distance_m", "Acoustic Source Distance", 0.1, 50.0, 2.5, "m", MacroRole::Tone, "Acoustic Source Distance parameter"))
        .with_param(DspParamSchema::knob("air_absorption_hf_loss_db", "Air Absorption HF Loss", 0.0, 24.0, 4.5, "dB", MacroRole::Tone, "Air Absorption HF Loss parameter"))
        .with_param(DspParamSchema::knob("proximity_bass_boost_db", "Proximity Bass Boost", 0.0, 18.0, 3.0, "dB", MacroRole::Tone, "Proximity Bass Boost parameter"))
        .with_param(DspParamSchema::knob("itd_spread", "Interaural Distance Spread", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Interaural Distance Spread parameter"))
    );
    descriptors.insert("AtmosSurroundNode".to_string(), DspNodeDescriptor::new("AtmosSurroundNode", "7.1.4 Immersive Atmos Surround Bed Renderer", DspNodeCategory::SpatialSurround, "7.1.4 Immersive Atmos Surround Bed Renderer")
        .with_param(DspParamSchema::knob("surround_azimuth", "Bed Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Bed Horizontal Azimuth parameter"))
        .with_param(DspParamSchema::knob("height_elevation", "Height Overhead Angle", 0.0, 90.0, 35.0, "deg", MacroRole::Tone, "Height Overhead Angle parameter"))
        .with_param(DspParamSchema::knob("lfe_send_level", "LFE Subwoofer Send", -40.0, 6.0, 0.0, "dB", MacroRole::Tone, "LFE Subwoofer Send parameter"))
        .with_param(DspParamSchema::knob("divergence_spread", "Center-Surround Divergence", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Center-Surround Divergence parameter"))
    );
    descriptors.insert("HoaSpatializer".to_string(), DspNodeDescriptor::new("HoaSpatializer", "4th/5th-Order High Order Ambisonic Spherical Panner", DspNodeCategory::SpatialSurround, "4th/5th-Order High Order Ambisonic Spherical Panner")
        .with_param(DspParamSchema::knob("spherical_yaw", "Ambisonic Yaw Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Ambisonic Yaw Angle parameter"))
        .with_param(DspParamSchema::knob("spherical_pitch", "Ambisonic Pitch Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Ambisonic Pitch Angle parameter"))
        .with_param(DspParamSchema::knob("ambisonic_order", "HOA Spatial Order", 1.0, 5.0, 4.0, "order", MacroRole::Character, "HOA Spatial Order integer control"))
        .with_param(DspParamSchema::knob("in_phase_weighting", "In-Phase Max-rE Weighting", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "In-Phase Max-rE Weighting parameter"))
    );
    descriptors.insert("Hoa5BinauralSpatializer".to_string(), DspNodeDescriptor::new("Hoa5BinauralSpatializer", "5th-Order Ambisonic Binaural Decoder with SOFA HRTF", DspNodeCategory::SpatialSurround, "5th-Order Ambisonic Binaural Decoder with SOFA HRTF")
        .with_param(DspParamSchema::choice("sofa_profile", "SOFA HRIR Profile", &["KEMAR High-Precision", "Genelec Aural ID", "SADIE II Dummy", "IRCAM Listen Subject 1002"], 0, "SOFA HRIR Profile mode selector"))
        .with_param(DspParamSchema::choice("binaural_rendering_mode", "Binaural Decoder Kernel", &["Dual-Parabolic SVD", "Spherical Harmonics Least-Squares", "Diffuse Field Equalized"], 1, "Binaural Decoder Kernel mode selector"))
        .with_param(DspParamSchema::knob("nearfield_compensation", "Nearfield Bass Compensation", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Nearfield Bass Compensation parameter"))
        .with_param(DspParamSchema::knob("itd_emphasis", "Psychoacoustic ITD Emphasis", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Psychoacoustic ITD Emphasis parameter"))
    );
    descriptors.insert("Mpegh3DSpatializer".to_string(), DspNodeDescriptor::new("Mpegh3DSpatializer", "MPEG-H 3D Immersive Audio Channel Bed Panner", DspNodeCategory::SpatialSurround, "MPEG-H 3D Immersive Audio Channel Bed Panner")
        .with_param(DspParamSchema::choice("bed_layout", "MPEG-H Target Bed", &["5.1.4 Surround", "7.1.4 Immersive", "9.1.6 3D Master", "22.2 Hi-Vision"], 1, "MPEG-H Target Bed mode selector"))
        .with_param(DspParamSchema::knob("elevation_gain", "Overhead Layer Trim", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Overhead Layer Trim parameter"))
        .with_param(DspParamSchema::knob("object_spread", "Object Spherical Divergence", 0.0, 100.0, 20.0, "%", MacroRole::Tone, "Object Spherical Divergence parameter"))
        .with_param(DspParamSchema::knob("lfe_gain", "LFE Subwoofer Level", -24.0, 6.0, 0.0, "dB", MacroRole::Tone, "LFE Subwoofer Level parameter"))
    );
    descriptors.insert("MpeghTrajectorySpatializer".to_string(), DspNodeDescriptor::new("MpeghTrajectorySpatializer", "3D Trajectory Bezier Spline Audio Spatializer", DspNodeCategory::SpatialSurround, "3D Trajectory Bezier Spline Audio Spatializer")
        .with_param(DspParamSchema::knob("trajectory_speed_bpm", "Spline Trajectory Speed", 1.0, 240.0, 60.0, "BPM", MacroRole::Tone, "Spline Trajectory Speed parameter"))
        .with_param(DspParamSchema::knob("spline_tension", "Bezier Spline Tension", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Bezier Spline Tension parameter"))
        .with_param(DspParamSchema::knob("path_radius_m", "Orbital Path Radius", 0.5, 20.0, 3.5, "m", MacroRole::Tone, "Orbital Path Radius parameter"))
        .with_param(DspParamSchema::knob("elevation_range_m", "Vertical Elevation Sweep", 0.0, 10.0, 2.0, "m", MacroRole::Tone, "Vertical Elevation Sweep parameter"))
    );
    descriptors.insert("WfsArraySpatializerNode".to_string(), DspNodeDescriptor::new("WfsArraySpatializerNode", "Wave Field Synthesis Multi-Transducer Array Engine", DspNodeCategory::SpatialSurround, "Wave Field Synthesis Multi-Transducer Array Engine")
        .with_param(DspParamSchema::knob("array_elements_count", "WFS Transducer Count", 16.0, 128.0, 64.0, "speakers", MacroRole::Character, "WFS Transducer Count integer control"))
        .with_param(DspParamSchema::knob("speaker_spacing_cm", "Transducer Element Pitch", 2.0, 30.0, 8.0, "cm", MacroRole::Tone, "Transducer Element Pitch parameter"))
        .with_param(DspParamSchema::knob("focused_source_depth_m", "Focused Virtual Depth", -5.0, 15.0, 2.5, "m", MacroRole::Tone, "Focused Virtual Depth parameter"))
        .with_param(DspParamSchema::knob("spatial_aliasing_cut", "Spatial Aliasing Lowpass", 1000.0, 15000.0, 4500.0, "Hz", MacroRole::Tone, "Spatial Aliasing Lowpass parameter"))
    );
    descriptors.insert("DiffractivePropagationNode".to_string(), DspNodeDescriptor::new("DiffractivePropagationNode", "Acoustic Obstacle Diffraction & Shadow Zone Simulator", DspNodeCategory::SpatialSurround, "Acoustic Obstacle Diffraction & Shadow Zone Simulator")
        .with_param(DspParamSchema::knob("obstacle_width_m", "Obstacle Barrier Width", 0.2, 20.0, 3.0, "m", MacroRole::Tone, "Obstacle Barrier Width parameter"))
        .with_param(DspParamSchema::knob("diffraction_angle_deg", "Bending Diffraction Angle", 0.0, 180.0, 45.0, "deg", MacroRole::Tone, "Bending Diffraction Angle parameter"))
        .with_param(DspParamSchema::log_knob("shadow_zone_lowpass_hz", "Shadow Zone High Cut", 200.0, 10000.0, 1200.0, "Hz", MacroRole::Tone, "Shadow Zone High Cut logarithmic parameter"))
        .with_param(DspParamSchema::knob("edge_scattering_gain", "Edge Scattering Amplitude", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Edge Scattering Amplitude parameter"))
    );
    descriptors.insert("BinauralPannerNode".to_string(), DspNodeDescriptor::new("BinauralPannerNode", "Precision Interaural Time & Level Difference 3D Panner", DspNodeCategory::SpatialSurround, "Precision Interaural Time & Level Difference 3D Panner")
        .with_param(DspParamSchema::knob("azimuth_deg", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Source Azimuth Angle parameter"))
        .with_param(DspParamSchema::knob("elevation_deg", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Source Elevation Angle parameter"))
        .with_param(DspParamSchema::knob("itd_us", "Interaural Time Difference", -700.0, 700.0, 0.0, "us", MacroRole::Tone, "Interaural Time Difference parameter"))
        .with_param(DspParamSchema::knob("ild_db", "Interaural Level Difference", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Interaural Level Difference parameter"))
    );
    descriptors.insert("WavefrontReflectionNode".to_string(), DspNodeDescriptor::new("WavefrontReflectionNode", "Early Specular & Diffuse Wavefront Room Reflection Engine", DspNodeCategory::SpatialSurround, "Early Specular & Diffuse Wavefront Room Reflection Engine")
        .with_param(DspParamSchema::knob("room_dimensions_xyz", "Room Scale Multiplier", 0.5, 5.0, 1.0, "x", MacroRole::Tone, "Room Scale Multiplier parameter"))
        .with_param(DspParamSchema::knob("wall_absorption_coeff", "Boundary Absorption", 0.05, 0.95, 0.2, "", MacroRole::Tone, "Boundary Absorption parameter"))
        .with_param(DspParamSchema::knob("specular_order", "Ray Image Source Order", 1.0, 8.0, 4.0, "orders", MacroRole::Character, "Ray Image Source Order integer control"))
        .with_param(DspParamSchema::knob("diffuse_scatter_mix", "Diffuse Lambertian Scatter", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Diffuse Lambertian Scatter parameter"))
    );
    descriptors.insert("MultibandSpatialNode".to_string(), DspNodeDescriptor::new("MultibandSpatialNode", "Tri-Band Frequency-Dependent Stereo & Spatial Panner", DspNodeCategory::SpatialSurround, "Tri-Band Frequency-Dependent Stereo & Spatial Panner")
        .with_param(DspParamSchema::knob("low_band_pan", "Low Band Panning (<200Hz)", -1.0, 1.0, 0.0, "pan", MacroRole::Tone, "Low Band Panning (<200Hz) parameter"))
        .with_param(DspParamSchema::knob("mid_band_pan", "Mid Band Panning", -1.0, 1.0, 0.25, "pan", MacroRole::Tone, "Mid Band Panning parameter"))
        .with_param(DspParamSchema::knob("high_band_pan", "High Band Panning (>4kHz)", -1.0, 1.0, -0.3, "pan", MacroRole::Tone, "High Band Panning (>4kHz) parameter"))
        .with_param(DspParamSchema::log_knob("crossover_freq_hz", "Mid-High Crossover Point", 500.0, 8000.0, 2500.0, "Hz", MacroRole::Tone, "Mid-High Crossover Point logarithmic parameter"))
    );
    descriptors.insert("AtmosphericDensityNode".to_string(), DspNodeDescriptor::new("AtmosphericDensityNode", "Atmospheric Gas Composition & Speed of Sound Simulator", DspNodeCategory::SpatialSurround, "Atmospheric Gas Composition & Speed of Sound Simulator")
        .with_param(DspParamSchema::knob("ambient_pressure_kpa", "Atmospheric Pressure", 1.0, 1000.0, 101.3, "kPa", MacroRole::Tone, "Atmospheric Pressure parameter"))
        .with_param(DspParamSchema::knob("temperature_celsius", "Ambient Temperature", -100.0, 200.0, 20.0, "degC", MacroRole::Tone, "Ambient Temperature parameter"))
        .with_param(DspParamSchema::knob("helium_fraction_ratio", "Helium Gas Ratio (Speed Boost)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Helium Gas Ratio (Speed Boost) parameter"))
        .with_param(DspParamSchema::knob("co2_fraction_ratio", "CO2 Gas Ratio (Speed Drop)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "CO2 Gas Ratio (Speed Drop) parameter"))
    );
    descriptors.insert("SpatialRoomNode".to_string(), DspNodeDescriptor::new("SpatialRoomNode", "Image-Source Acoustic 3D Room Simulator", DspNodeCategory::SpatialSurround, "Geometric 6-wall image-source early reflections generator with wall absorption, air absorption, and listener orientation")
        .with_param(DspParamSchema::knob("room_width", "Room Width", 1.0, 50.0, 10.0, "m", MacroRole::Space, "Room lateral dimension parameter"))
        .with_param(DspParamSchema::knob("room_depth", "Room Depth", 1.0, 50.0, 14.0, "m", MacroRole::Space, "Room longitudinal dimension parameter"))
        .with_param(DspParamSchema::knob("room_height", "Room Height", 2.0, 20.0, 4.5, "m", MacroRole::Space, "Room vertical ceiling height parameter"))
        .with_param(DspParamSchema::knob("source_x", "Source X Position", -25.0, 25.0, 0.0, "m", MacroRole::Space, "Source lateral offset in room coordinate space parameter"))
        .with_param(DspParamSchema::knob("source_y", "Source Y Distance", -25.0, 25.0, 3.0, "m", MacroRole::Space, "Source depth distance from listener parameter"))
        .with_param(DspParamSchema::knob("source_z", "Source Z Height", 0.0, 10.0, 1.5, "m", MacroRole::Space, "Source elevation above floor parameter"))
        .with_param(DspParamSchema::knob("wall_absorption", "Wall Acoustic Absorption", 0.05, 0.95, 0.30, "%", MacroRole::Tone, "Boundary material acoustic reflection damping parameter"))
    );
    descriptors.insert("ContinuousSpatialPannerDoppler3D".to_string(), DspNodeDescriptor::new("ContinuousSpatialPannerDoppler3D", "Continuous 3D Dynamic Spatializer & Doppler Engine", DspNodeCategory::SpatialSurround, "Dynamic 3D audio trajectories with continuous time-varying Doppler pitch shift, ITD delay lines, and ILD head-shadowing")
        .with_param(DspParamSchema::knob("pos_x", "Azimuth Position X", -10.0, 10.0, 0.0, "m", MacroRole::Space, "3D sound source X coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("pos_y", "Distance Depth Y", -10.0, 10.0, 2.0, "m", MacroRole::Space, "3D sound source Y coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("pos_z", "Elevation Height Z", -5.0, 5.0, 0.0, "m", MacroRole::Space, "3D sound source Z coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("speed_of_sound", "Speed of Sound", 300.0, 400.0, 343.0, "m/s", MacroRole::Tone, "Atmospheric sound propagation velocity parameter"))
        .with_param(DspParamSchema::knob("air_absorption", "High-Frequency Air Loss", 0.0, 1.0, 0.35, "%", MacroRole::Character, "Distance-dependent air humidity absorption parameter"))
    );
    descriptors.insert("AmbisonicsBinauralConvolver3D".to_string(), DspNodeDescriptor::new("AmbisonicsBinauralConvolver3D", "Higher-Order Ambisonics (HOA) Binaural Convolver", DspNodeCategory::SpatialSurround, "Convolves 3D spherical harmonics Ambisonic soundfield streams into binaural headphone audio using HRTF filter sets")
        .with_param(DspParamSchema::choice("ambisonic_order", "Ambisonic HOA Order", &["1st Order (4 ch)", "2nd Order (9 ch)", "3rd Order (16 ch)"], 0, "Spherical harmonic spatial resolution order selector"))
        .with_param(DspParamSchema::choice("hrtf_dataset", "HRTF Dataset Profile", &["KEMAR", "CIPIC", "SADIE II", "Generic Sphere"], 0, "Head-related transfer function impulse response profile selector"))
        .with_param(DspParamSchema::knob("head_rotation_yaw", "Head Yaw Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Listener head horizontal panning rotation parameter"))
        .with_param(DspParamSchema::knob("head_rotation_pitch", "Head Pitch Tilt", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Listener head vertical tilt elevation parameter"))
    );
    descriptors.insert("BinauralSpatialPannerNode".to_string(), DspNodeDescriptor::new("BinauralSpatialPannerNode", "Binaural 3D Headphone Spatial Panner", DspNodeCategory::SpatialSurround, "Point-source 3D HRTF spatial panner with interaural time difference (ITD) and interaural level difference (ILD)")
        .with_param(DspParamSchema::knob("azimuth", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Polar horizontal azimuth angle parameter"))
        .with_param(DspParamSchema::knob("elevation", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Polar vertical elevation angle parameter"))
        .with_param(DspParamSchema::knob("distance", "Radial Distance", 0.1, 20.0, 1.5, "m", MacroRole::Tone, "Radial distance from listener head parameter"))
    );
    descriptors.insert("HolographicSpatializer".to_string(), DspNodeDescriptor::new("HolographicSpatializer", "Acoustic Lightfield Holographic Spatializer", DspNodeCategory::SpatialSurround, "Acoustic soundfield hologram projector recreating volumetric 3D wavefronts across multiple planar layers")
        .with_param(DspParamSchema::knob("pos_x", "Hologram Position X", -10.0, 10.0, 0.0, "m", MacroRole::Space, "Volumetric sound source X coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("pos_y", "Hologram Position Y", -10.0, 10.0, 2.0, "m", MacroRole::Space, "Volumetric sound source Y coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("pos_z", "Hologram Position Z", -5.0, 5.0, 1.0, "m", MacroRole::Space, "Volumetric sound source Z coordinate in meters parameter"))
        .with_param(DspParamSchema::knob("planes", "Acoustic Wavefront Planes", 2.0, 16.0, 4.0, "planes", MacroRole::Character, "Discrete holographic planar wave reconstruction resolution parameter"))
    );
    descriptors.insert("DistanceDopplerNode".to_string(), DspNodeDescriptor::new("DistanceDopplerNode", "3D Distance Attenuation, Air Absorption & Doppler Shift", DspNodeCategory::SpatialSurround, "Physically accurate 3D acoustic propagation with inverse-square falloff, frequency-dependent air absorption, and continuous Doppler shift")
        .with_param(DspParamSchema::knob("pos_x", "Source X Position", -50.0, 50.0, 0.0, "m", MacroRole::Space, "Lateral 3D spatial coordinate parameter"))
        .with_param(DspParamSchema::knob("pos_y", "Source Y Position", -50.0, 50.0, 1.0, "m", MacroRole::Space, "Front-back 3D spatial coordinate parameter"))
        .with_param(DspParamSchema::knob("pos_z", "Source Z Position", -50.0, 50.0, 0.0, "m", MacroRole::Space, "Vertical elevation 3D spatial coordinate parameter"))
        .with_param(DspParamSchema::knob("speed_of_sound", "Speed of Sound", 300.0, 400.0, 343.0, "m/s", MacroRole::Character, "Medium acoustic velocity for Doppler pitch calculations parameter"))
        .with_param(DspParamSchema::knob("air_absorption", "Air Absorption", 0.0, 1.0, 0.30, "%", MacroRole::Tone, "Distance-dependent high frequency dampening parameter"))
    );
    descriptors.insert("BrirConvolutionNode".to_string(), DspNodeDescriptor::new("BrirConvolutionNode", "Binaural Room Impulse Response (BRIR) Convolver", DspNodeCategory::SpatialSurround, "Zero-latency dual-ear BRIR convolution engine placing virtual sound sources into hyper-realistic acoustic rooms")
        .with_param(DspParamSchema::knob("room_decay", "Impulse Decay Time", 0.05, 5.0, 0.8, "s", MacroRole::Space, "Binaural room reverberation tail duration parameter"))
        .with_param(DspParamSchema::knob("reflection_density", "Early Reflections", 0.0, 1.0, 0.75, "%", MacroRole::Space, "Early boundary reflection density parameter"))
        .with_param(DspParamSchema::knob("head_shadowing", "Head Shadowing ILD", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Interaural level difference filtering intensity parameter"))
        .with_param(DspParamSchema::knob("wet_dry", "Wet / Dry Mix", 0.0, 1.0, 0.50, "%", MacroRole::Space, "Balance between direct dry sound and binaural room convolution parameter"))
    );
    descriptors.insert("SpatialReverb3D".to_string(), DspNodeDescriptor::new("SpatialReverb3D", "3D Volumetric Spatial Geometric Room Reverb", DspNodeCategory::SpatialSurround, "Full 3D spatial reverberation engine supporting immersive speaker layouts (Stereo up to 22.2) with geometric room boundary reflections")
        .with_param(DspParamSchema::knob("room_size_m", "Room Dimension", 1.0, 50.0, 12.0, "m", MacroRole::Space, "Length of virtual acoustic room cube parameter"))
        .with_param(DspParamSchema::knob("rt60_decay", "Reverb Decay RT60", 0.1, 10.0, 1.8, "s", MacroRole::Space, "Time required for late diffuse reflections to decay by 60dB parameter"))
        .with_param(DspParamSchema::knob("diffusion", "Wall Diffusion", 0.0, 1.0, 0.70, "%", MacroRole::Space, "Scattering coefficient of virtual acoustic walls parameter"))
        .with_param(DspParamSchema::knob("predelay_ms", "Pre-Delay Time", 0.0, 200.0, 20.0, "ms", MacroRole::Space, "Gap before first early reflections arrive parameter"))
        .with_param(DspParamSchema::knob("high_freq_damping", "Air Damping Cutoff", 500.0, 20000.0, 6000.0, "Hz", MacroRole::Tone, "High frequency absorption corner frequency parameter"))
    );
    descriptors.insert("SpeakerCalibrationMatrix".to_string(), DspNodeDescriptor::new("SpeakerCalibrationMatrix", "Studio Monitor Layout Calibration Matrix", DspNodeCategory::SpatialSurround, "Multi-channel surround and immersive monitor calibration matrix with per-channel gain trimming, delay alignment, and target curve compensation")
        .with_param(DspParamSchema::knob("num_channels", "Speaker Channel Count", 2.0, 16.0, 8.0, "ch", MacroRole::Character, "Total monitored surround/immersive channels parameter"))
        .with_param(DspParamSchema::knob("master_trim_db", "Master Alignment Trim", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Global output calibration trim level parameter"))
        .with_param(DspParamSchema::knob("delay_compensation_ms", "Inter-Channel Delay Align", 0.0, 50.0, 0.0, "ms", MacroRole::Space, "Distance-compensation inter-channel delay parameter"))
        .with_param(DspParamSchema::choice("target_curve", "Target EQ Curve", &["Flat Studio", "Harman Target", "B&K 1974 House", "Diffuse Field"], 0, "Target frequency response profile"))
    );
    descriptors.insert("AmbisonicsDecoder3D".to_string(), DspNodeDescriptor::new("AmbisonicsDecoder3D", "Higher-Order Ambisonics (HOA) Spherical Decoder", DspNodeCategory::SpatialSurround, "Higher-Order Ambisonics (HOA) spherical harmonic soundfield decoder supporting 1st through 5th order decoding to binaural HRTF or multichannel loudspeaker arrays")
        .with_param(DspParamSchema::knob("ambisonic_order", "Ambisonic Decoding Order", 1.0, 5.0, 3.0, "ord", MacroRole::Space, "Maximum spherical harmonic decoding order parameter"))
        .with_param(DspParamSchema::choice("target_layout", "Target Speaker Layout", &["Binaural HRTF", "5.1 Surround", "7.1.4 Dolby Atmos", "22.2 NHK Immersive"], 0, "Target reproduction channel configuration"))
        .with_param(DspParamSchema::choice("decode_mode", "Energy Optimization Mode", &["In-Phase", "Max-rE", "Basic Energy Preserving"], 1, "Ambisonic energy preservation decode mode"))
        .with_param(DspParamSchema::log_knob("lfe_sub_cutoff", "LFE Sub Crossover", 40.0, 160.0, 80.0, "Hz", MacroRole::Punch, "Low frequency bass management cutoff parameter"))
    );
    descriptors.insert("AmbisonicsEncoder3D".to_string(), DspNodeDescriptor::new("AmbisonicsEncoder3D", "Higher-Order Ambisonics (HOA) Spherical Panner", DspNodeCategory::SpatialSurround, "Real-time spherical coordinate encoder converting monophonic or stereophonic point sources into 3D Higher-Order Ambisonics B-format spherical harmonic representations")
        .with_param(DspParamSchema::knob("azimuth", "Spherical Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth pan angle parameter"))
        .with_param(DspParamSchema::knob("elevation", "Spherical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation angle parameter"))
        .with_param(DspParamSchema::knob("distance", "Source Radial Distance", 0.1, 50.0, 1.5, "m", MacroRole::Space, "Radial source distance parameter"))
        .with_param(DspParamSchema::knob("encoding_order", "HOA Spherical Harmonic Order", 1.0, 5.0, 3.0, "ord", MacroRole::Character, "Spherical harmonic order resolution parameter"))
    );
    descriptors.insert("PartitionedBinauralHrtfConvolver".to_string(), DspNodeDescriptor::new("PartitionedBinauralHrtfConvolver", "Zero-Latency Partitioned FFT Binaural Convolver", DspNodeCategory::SpatialSurround, "Uniformly partitioned zero-latency fast Fourier transform (FFT) binaural HRIR convolver delivering pin-point 3D spatial localization with zero audible delay")
        .with_param(DspParamSchema::knob("azimuth", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth panning coordinate parameter"))
        .with_param(DspParamSchema::knob("elevation", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation coordinate parameter"))
        .with_param(DspParamSchema::knob("hrtf_distance", "Virtual Source Distance", 0.2, 20.0, 1.2, "m", MacroRole::Space, "Distance-dependent attenuation and parallax parameter"))
        .with_param(DspParamSchema::choice("partition_size", "FFT Partition Block Size", &["32 Samples (Sub-1ms)", "64 Samples (Ultra-low)", "128 Samples (Balanced)", "256 Samples (Economy)"], 1, "Partitioned convolution sub-block size"))
    );
    descriptors.insert("AcousticHologramFilter".to_string(), DspNodeDescriptor::new("AcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Wavefield Filter", DspNodeCategory::SpatialSurround, "Wavefield synthesis (WFS) loudspeaker array acoustic hologram reconstruction filter calculating cylindrical wavefronts")
        .with_param(DspParamSchema::knob("num_speakers", "Loudspeaker Count", 4.0, 128.0, 16.0, "spk", MacroRole::Space, "Active physical transducers in linear or curved wavefield array parameter"))
        .with_param(DspParamSchema::knob("array_curvature", "Array Curvature Radius", 1.0, 20.0, 5.0, "m", MacroRole::Tone, "Physical geometry radius of holographic speaker array parameter"))
        .with_param(DspParamSchema::knob("distance_attenuation", "Distance Falloff Exponent", 0.1, 2.0, 0.5, "p", MacroRole::Punch, "Acoustic pressure 1/sqrt(r) cylindrical distance attenuation factor parameter"))
        .with_param(DspParamSchema::choice("focus_mode", "Wavefront Curvature Mode", &["Convergent Focal Point", "Divergent Plane Wave", "Focused Point Source", "Curved Acoustic Lens"], 0, "Spatial holographic wavefront geometry parameter"))
    );
    descriptors.insert("HyperDimensionalHrtfLoader".to_string(), DspNodeDescriptor::new("HyperDimensionalHrtfLoader", "Hyper-Dimensional Spatial HRTF Dataset Interpolator", DspNodeCategory::SpatialSurround, "Continuous SOFA spherical spline interpolator rendering head-related transfer functions across 5 spatial dimensions")
        .with_param(DspParamSchema::knob("azimuth_deg", "Spherical Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular position around listener head parameter"))
        .with_param(DspParamSchema::knob("elevation_deg", "Spherical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Vertical angular height above horizon parameter"))
        .with_param(DspParamSchema::knob("distance_m", "Source Radial Distance", 0.2, 10.0, 1.5, "m", MacroRole::Punch, "Near-field to far-field distance acoustic attenuation parameter"))
        .with_param(DspParamSchema::knob("pinna_morph", "Anthropometric Pinna Scale", 0.5, 2.0, 1.0, "x", MacroRole::Character, "Ear notch frequency scaling for personalized HRTF tuning parameter"))
    );
    descriptors.insert("AcousticCloakingSpatializerNode".to_string(), DspNodeDescriptor::new("AcousticCloakingSpatializerNode", "Acoustic Cloaking Spatializer & Soundfield Invisibility Zone", DspNodeCategory::SpatialSurround, "Transformation acoustics spatializer bending sound waves around virtual obstacles to create acoustic shadow invisibility zones")
        .with_param(DspParamSchema::knob("inner_radius_m", "Inner Cloak Boundary", 0.1, 10.0, 1.0, "m", MacroRole::Space, "Radius of soundless obstacle zone inside acoustic metamaterial parameter"))
        .with_param(DspParamSchema::knob("outer_radius_m", "Outer Cloak Shell", 0.2, 20.0, 2.5, "m", MacroRole::Tone, "Outer boundary where acoustic path returns to free-field parameter"))
        .with_param(DspParamSchema::knob("cloaking_factor", "Cloaking Attenuation Factor", 0.0, 1.0, 0.95, "%", MacroRole::Punch, "Efficiency of sound wave redirection around inner core parameter"))
        .with_param(DspParamSchema::choice("cloak_profile", "Metamaterial Gradient Profile", &["Linear Anisotropic", "Exponential Conformal", "Layered Acoustic Pentamode", "Lattice Phononic Crystal"], 0, "Transformation acoustics shell design parameter"))
    );
    descriptors.insert("SurroundStemSplitterBedObject".to_string(), DspNodeDescriptor::new("SurroundStemSplitterBedObject", "7.1.4 Dolby Atmos & MPEG-H Bed & Dynamic Object Stem Splitter", DspNodeCategory::SpatialSurround, "Splits immersive multi-channel stems into stationary bed channels and moving 3D point audio objects with metadata")
        .with_param(DspParamSchema::knob("object_cluster_radius", "Object Cluster Radius", 0.1, 10.0, 1.5, "m", MacroRole::Space, "Spatial proximity threshold for dynamic object grouping parameter"))
        .with_param(DspParamSchema::knob("spatial_divergence", "Spatial Divergence", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Acoustic energy bleed between point objects and bed speakers parameter"))
        .with_param(DspParamSchema::knob("height_gain_trim_db", "Height Bed Gain Trim", -12.0, 6.0, 0.0, "dB", MacroRole::Character, "Level adjustment for overhead top ceiling speakers parameter"))
        .with_param(DspParamSchema::toggle("binaural_bed_render", "Binaural Headphone Monitor", true, "Render HRTF binaural preview downmix for stereo headphones parameter"))
    );
    descriptors.insert("HeadTrackerReceiver".to_string(), DspNodeDescriptor::new("HeadTrackerReceiver", "6-DOF OSC / BLE IMU Binaural Head-Tracking Receiver", DspNodeCategory::SpatialSurround, "Ultra-low-latency 6-degrees-of-freedom inertial head tracking receiver updating 3D binaural HRTF orientation")
        .with_param(DspParamSchema::knob("yaw_deg", "Head Yaw Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth head angle rotation parameter"))
        .with_param(DspParamSchema::knob("pitch_deg", "Head Pitch Tilt", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Vertical elevation head angle tilt parameter"))
        .with_param(DspParamSchema::knob("roll_deg", "Head Roll Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Character, "Ear-to-shoulder roll angle parameter"))
        .with_param(DspParamSchema::knob("drift_correction_tau", "Gyro Drift Time Constant", 0.1, 10.0, 2.5, "s", MacroRole::Punch, "Highpass filter time constant eliminating slow IMU sensor drift parameter"))
    );
    descriptors.insert("SpatialAutomationEngine".to_string(), DspNodeDescriptor::new("SpatialAutomationEngine", "Spatial Trajectory Path Interpolator & Dynamic 3D Automation Engine", DspNodeCategory::SpatialSurround, "3D Cartesian and spherical coordinate automation with parametric geometric paths (Orbit, Lissajous3D, Spiral, Flyby) and cubic Catmull-Rom splines")
        .with_param(DspParamSchema::log_knob("orbit_radius_m", "Trajectory Orbit Radius", 0.2, 25.0, 3.5, "m", MacroRole::Space, "Orbital radius distance from listener center parameter"))
        .with_param(DspParamSchema::knob("orbit_speed_cycles", "Orbit Speed Per Beat", 0.05, 10.0, 0.5, "cyc/b", MacroRole::Tone, "Orbital rotational speed in cycles per musical beat parameter"))
        .with_param(DspParamSchema::knob("orbit_tilt_deg", "Orbit Plane Inclination", -90.0, 90.0, 15.0, "deg", MacroRole::Character, "Inclination tilt angle of 3D orbital trajectory plane parameter"))
        .with_param(DspParamSchema::toggle("spline_smooth_easing", "Catmull-Rom Spline Easing", true, "Catmull-Rom cubic spline waypoint velocity smoothing parameter"))
    );
    descriptors.insert("SpatialBus".to_string(), DspNodeDescriptor::new("SpatialBus", "Real-Time 64-Channel 3D Atomic Coordinate Coordination Bus", DspNodeCategory::SpatialSurround, "64-slot lock-free atomic spatial bus synchronizing Cartesian X,Y,Z positions, azimuth, elevation, and source diffusion width")
        .with_param(DspParamSchema::knob("cartesian_x", "Source Cartesian X Position", -20.0, 20.0, 0.0, "m", MacroRole::Space, "Left-Right Cartesian coordinate along X-axis parameter"))
        .with_param(DspParamSchema::knob("cartesian_y", "Source Cartesian Y Position", -20.0, 20.0, 2.0, "m", MacroRole::Tone, "Front-Back Cartesian coordinate along Y-axis parameter"))
        .with_param(DspParamSchema::knob("cartesian_z", "Source Cartesian Z Position", -10.0, 10.0, 0.0, "m", MacroRole::Punch, "Height Cartesian coordinate along Z-axis parameter"))
        .with_param(DspParamSchema::toggle("atomic_slot_lockless", "Lockless Double-Buffer Sync", true, "Atomic double-buffered coordinate update without mutexes parameter"))
    );
    descriptors.insert("SpatialObjectAutomation".to_string(), DspNodeDescriptor::new("SpatialObjectAutomation", "3D Spatial Audio Vector Trajectory Engine", DspNodeCategory::SpatialSurround, "Real-time 3D object automation engine with spline interpolation, azimuth, elevation, and distance curves")
        .with_param(DspParamSchema::knob("azimuth_deg", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Horizontal spatial azimuth angle around listener parameter"))
        .with_param(DspParamSchema::knob("elevation_deg", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation angle relative to listener plane parameter"))
        .with_param(DspParamSchema::log_knob("distance_m", "Radial Distance", 0.1, 100.0, 2.5, "m", MacroRole::Punch, "Distance from listener center for inverse-square attenuation parameter"))
        .with_param(DspParamSchema::knob("orbit_speed_hz", "Orbit Angular Velocity", 0.01, 10.0, 0.25, "Hz", MacroRole::Character, "Rotational trajectory angular velocity parameter"))
        .with_param(DspParamSchema::toggle("doppler_tracking", "Acoustic Doppler Shift", true, "Enable relativistic Doppler pitch shift during rapid movement parameter"))
    );
    descriptors.insert("ImageReflection".to_string(), DspNodeDescriptor::new("ImageReflection", "Shoebox Room Geometric Acoustic Image-Source Engine", DspNodeCategory::SpatialSurround, "First and second order specular acoustic reflection generator for shoebox geometry listening rooms")
        .with_param(DspParamSchema::knob("room_length_m", "Room Length", 2.0, 50.0, 10.0, "m", MacroRole::Space, "Shoebox room physical length coordinate parameter"))
        .with_param(DspParamSchema::knob("room_width_m", "Room Width", 2.0, 40.0, 7.5, "m", MacroRole::Space, "Shoebox room physical width coordinate parameter"))
        .with_param(DspParamSchema::knob("room_height_m", "Room Ceiling Height", 2.0, 20.0, 3.8, "m", MacroRole::Space, "Shoebox room ceiling height coordinate parameter"))
        .with_param(DspParamSchema::knob("wall_absorption_coeff", "Wall Absorption Alpha", 0.05, 0.95, 0.25, "alpha", MacroRole::Tone, "Frequency-dependent acoustic boundary wall absorption coefficient parameter"))
        .with_param(DspParamSchema::knob("specular_diffusion_mix", "Diffusion Blend", 0.0, 1.0, 0.30, "diff", MacroRole::Character, "Ratio between mirror-like specular reflections and scattered diffuse tail parameter"))
    );
    descriptors.insert("StereoPanner".to_string(), DspNodeDescriptor::new("StereoPanner", "Equal-Power Stereo Coordinate Panner & Balance Trimmer", DspNodeCategory::SpatialSurround, "Sine-cosine 3dB and 4.5dB equal-power stereo panner with width expansion, center channel trim, and polarity invert")
        .with_param(DspParamSchema::knob("pan_position", "Pan Coordinate", -1.0, 1.0, 0.0, "pan", MacroRole::Tone, "Horizontal stereo position: -1.0=Hard Left, 0.0=Center, 1.0=Hard Right parameter"))
        .with_param(DspParamSchema::knob("stereo_width", "Stereo Image Width", 0.0, 2.0, 1.0, "width", MacroRole::Space, "Stereo width scaling: 0.0=Mono, 1.0=Normal Stereo, 2.0=Super-Wide M/S expansion parameter"))
        .with_param(DspParamSchema::knob("center_attenuation_db", "Center Attenuation", -6.0, 0.0, -3.0, "dB", MacroRole::Punch, "Pan law center attenuation compensation curve parameter"))
        .with_param(DspParamSchema::knob("balance_trim_db", "Fine Balance Trim", -12.0, 12.0, 0.0, "dB", MacroRole::Character, "Fine balance gain trim between Left and Right channels parameter"))
        .with_param(DspParamSchema::toggle("phase_invert_right", "Invert Right Polarity", false, "Invert phase polarity of right audio channel parameter"))
    );
    descriptors.insert("StereoWidth".to_string(), DspNodeDescriptor::new("StereoWidth", "Psychoacoustic Mid/Side Stereo Width & Haas Expander", DspNodeCategory::SpatialSurround, "Precision Mid/Side psychoacoustic stereo spatial width adjuster with mono correlation protection and elliptical sub-bass centering")
        .with_param(DspParamSchema::knob("width", "Stereo Field Width", 0.0, 2.0, 1.0, "%", MacroRole::Space, "Side channel level scaling from mono (0%) to ultra-wide (200%)"))
        .with_param(DspParamSchema::knob("stereo_balance", "Left / Right Balance", -1.0, 1.0, 0.0, "bal", MacroRole::Tone, "Stereo center panning balance"))
        .with_param(DspParamSchema::log_knob("side_hpf_hz", "Bass Mono Filter Cutoff", 20.0, 1000.0, 120.0, "Hz", MacroRole::Tone, "Elliptical highpass filter centering low bass below cutoff in mono"))
        .with_param(DspParamSchema::knob("mono_correlation_threshold", "Phase Safety Guard", 0.0, 1.0, 0.20, "r", MacroRole::Punch, "Limits width expansion if phase correlation drops below threshold"))
        .with_param(DspParamSchema::knob("dry_wet_mix", "Stereo Effect Blend", 0.0, 1.0, 1.0, "%", MacroRole::Space, "Wet processed stereo signal blend ratio"))
    );
    descriptors.insert("SpatialTrajectoryTrack".to_string(), DspNodeDescriptor::new("SpatialTrajectoryTrack", "Ambisonics 3D Automated Trajectory Orbit Engine", DspNodeCategory::SpatialSurround, "Autonomous 3D spatial orbit and spiral trajectory generator calculating dynamic XYZ position vectors and Doppler shifts for surround soundfields")
        .with_param(DspParamSchema::knob("orbit_speed_hz", "Orbit Revolution Rate", 0.01, 10.0, 0.25, "Hz", MacroRole::Tone, "Speed of continuous rotational orbit around listener in Hertz"))
        .with_param(DspParamSchema::knob("orbit_radius_m", "Soundfield Orbit Radius", 0.5, 20.0, 4.0, "m", MacroRole::Space, "Distance from listener origin to rotating sound source"))
        .with_param(DspParamSchema::choice("trajectory_shape", "Spatial Trajectory Path", &["Circle XY", "Vertical Spiral XYZ", "Figure-8 Lissajous", "Chaotic Lorenz"], 0, "Geometric curve equation governing 3D position vector"))
        .with_param(DspParamSchema::knob("elevation_angle_deg", "Orbit Center Elevation", -90.0, 90.0, 15.0, "deg", MacroRole::Tone, "Pitch angle in degrees tilting orbit plane relative to horizon"))
        .with_param(DspParamSchema::knob("doppler_intensity", "Doppler Pitch Warp Ratio", 0.0, 1.0, 0.35, "%", MacroRole::Character, "Physical Doppler pitch shift intensity proportional to orbital velocity"))
    );
    descriptors.insert("Hoa4Spatializer".to_string(), DspNodeDescriptor::new("Hoa4Spatializer", "Higher-Order Ambisonics 4th-Order (25-Ch) 3D Spatial Energy HUD", DspNodeCategory::SpatialSurround, "Full 4th-order (N+1)^2 = 25-channel spherical harmonics spatializer with energy-max rE decoding and 3D soundfield energy density display")
        .with_param(DspParamSchema::choice("hoa_profile", "Ambisonics HOA4 Decoding Profile", &["25-Ch HOA4 Sphere", "Binaural HOA4 HRIR", "22.2 Broadcast", "7.1.4 Dome Decode", "Energy-Max rE"], 0, "Target loudspeaker setup or binaural rendering format"))
        .with_param(DspParamSchema::knob("azimuth_deg", "Soundfield Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular position of primary 3D audio source"))
        .with_param(DspParamSchema::knob("elevation_deg", "Soundfield Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical tilt angle in degrees above/below listener horizontal plane"))
        .with_param(DspParamSchema::knob("distance_m", "3D Radial Distance Radius", 0.5, 15.0, 2.5, "m", MacroRole::Space, "Radial distance from center sweet spot in meters"))
        .with_param(DspParamSchema::knob("diffuse_energy_ratio", "Diffuse-Field Ambient Energy Ratio", 0.0, 1.0, 0.20, "%", MacroRole::Tone, "Proportion of source energy distributed uniformly into diffuse surround tail"))
    );
    descriptors.insert("MpeghSpatializer".to_string(), DspNodeDescriptor::new("MpeghSpatializer", "Broadcast MPEG-H 3D Audio Multi-Object Spatial Bus HUD", DspNodeCategory::SpatialSurround, "Interactive 3D spatial panner for MPEG-H broadcast authoring, featuring 7.1.4 bed layout, dynamic 6-DoF audio objects, and personalized SOFA HRTF profiles")
        .with_param(DspParamSchema::choice("mpegh_format", "MPEG-H Target Output Layout", &["MPEG-H 7.1.4", "5.1 Surround", "22.2 NHK Dome", "SOFA Binaural", "Dynamic 6-DoF Objects"], 0, "Standard multichannel audio format for broadcast transmission"))
        .with_param(DspParamSchema::choice("hrtf_profile", "Binaural Personalized HRTF Profile", &["KEMAR Standard", "Genelec Aural", "Analytical Spherical", "Photogrammetry 3D Mesh"], 0, "Binaural filter dataset simulating human pinna and head shadow"))
        .with_param(DspParamSchema::knob("azimuth_deg", "Spatial Object Azimuth Angle", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal azimuth panning coordinate in degrees"))
        .with_param(DspParamSchema::knob("elevation_deg", "Spatial Object Elevation Angle", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical elevation coordinate in degrees above listener horizon"))
        .with_param(DspParamSchema::knob("distance_m", "Spatial Object Distance", 0.1, 10.0, 2.0, "m", MacroRole::Space, "Apparent radial distance from listener sweet spot in meters"))
    );
    descriptors.insert("SpatialIrConfig".to_string(), DspNodeDescriptor::new("SpatialIrConfig", "3D Spatial Impulse Response Acoustic Environment HUD", DspNodeCategory::SpatialSurround, "Higher-Order Ambisonics HOA 3D spatial impulse response configuration with spherical harmonic deconvolution, distance attenuation, and air scattering")
        .with_param(DspParamSchema::knob("azimuth_rotation_deg", "Acoustic Sphere Azimuth Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Rotates the spatial impulse response soundfield horizontally around listener"))
        .with_param(DspParamSchema::knob("elevation_tilt_deg", "Acoustic Sphere Elevation Tilt Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Tilts the spatial impulse response vertically above or below listener"))
        .with_param(DspParamSchema::knob("direct_to_reverberant_ratio_db", "Direct Path to Diffuse Field Ratio", -30.0, 30.0, 6.0, "dB", MacroRole::Character, "Balance between early direct acoustic arrivals and diffuse late reverberation"))
        .with_param(DspParamSchema::knob("spatial_width_percent", "Ambisonic B-Format Spatial Spread", 0.0, 200.0, 100.0, "%", MacroRole::Space, "Expands or collapses spherical harmonic components in soundfield"))
        .with_param(DspParamSchema::knob("room_absorption_hf", "Air Viscosity & Surface High-Freq Absorption", 0.1, 1.0, 0.65, "%", MacroRole::Tone, "Frequency-dependent atmospheric high-frequency attenuation coefficient"))
    );
    descriptors.insert("PeerMeshNetwork".to_string(), DspNodeDescriptor::new("PeerMeshNetwork", "Collaborative WebRTC Peer Mesh Network HUD", DspNodeCategory::SpatialSurround, "Decentralized zero-server WebRTC peer mesh networking engine for low-latency multi-user remote collaboration and live audio streaming")
        .with_param(DspParamSchema::knob("max_mesh_peers_limit", "Maximum Connected Collaborative Peer Nodes", 2.0, 16.0, 6.0, "peers", MacroRole::Tone, "Maximum simultaneous peer nodes connected in full-mesh topology before switching to star"))
        .with_param(DspParamSchema::knob("jitter_buffer_target_ms", "Adaptive Dynamic Audio Jitter Buffer Target", 10.0, 250.0, 40.0, "ms", MacroRole::Space, "Dynamic network audio jitter buffer target balancing latency against dropped packet clicks"))
        .with_param(DspParamSchema::choice("packet_loss_concealment", "Audio Packet Loss Concealment (PLC) Mode", &["Silence on Loss", "Zero-Order Hold", "Waveform Linear Fade", "Neural Spectral Inpainting"], 3, "Algorithm used to synthesize missing audio samples during transient internet packet loss"))
        .with_param(DspParamSchema::choice("peer_audio_compression_mode", "Collaborative Stream Audio Codec Mode", &["Opus High-Fidelity 128kbps", "Opus Studio 256kbps", "FLAC Ultra-Low-Loss", "PCM 16-bit Uncompressed"], 1, "Audio compression codec applied to outgoing collaborative audio streams"))
        .with_param(DspParamSchema::toggle("end_to_end_encryption_enable", "DTLS-SRTP End-to-End Stream Encryption", true, "Enforces DTLS-SRTP end-to-end cryptographic encryption across all collaborative peer streams"))
    );
    descriptors.insert("AdmSpatialBwfExporter".to_string(), DspNodeDescriptor::new("AdmSpatialBwfExporter", "ITU-R BS.2076 ADM Dolby Atmos Spatial BWF Exporter HUD", DspNodeCategory::SpatialSurround, "Master spatial audio exporter formatting multi-channel bed tracks and dynamic 3D audio objects into ITU-R BS.2076 Audio Definition Model BWF")
        .with_param(DspParamSchema::choice("adm_channel_configuration", "Spatial Speaker Bed & Object Profile", &["Dolby Atmos 7.1.4", "Binaural Spatial BWF", "Ambisonics 3rd Order (16ch)", "NHK 22.2 Spatial"], 0, "Selects master multichannel loudspeaker geometry and positional coordinate framework"))
        .with_param(DspParamSchema::knob("speaker_layout_bed_channels", "Fixed Speaker Bed Channel Count", 2.0, 12.0, 8.0, "ch", MacroRole::Tone, "Number of fixed ear-level and height bed channels assigned in ADM header"))
        .with_param(DspParamSchema::knob("spatial_audio_objects_count", "Dynamic 3D Positional Audio Objects", 1.0, 128.0, 64.0, "objs", MacroRole::Space, "Maximum simultaneously rendered Cartesian 3D object tracks with time-varying coordinates"))
        .with_param(DspParamSchema::knob("metadata_refresh_rate_hz", "Positional Coordinate Refresh Cadence", 24.0, 120.0, 60.0, "Hz", MacroRole::Character, "Temporal frequency of dynamic azimuth, elevation, and distance metadata updates"))
        .with_param(DspParamSchema::toggle("embed_itu_r_bs2076_xml", "Embed Compliant ITU-R BS.2076 XML Chunks", true, "Encodes standardized axml and chna chunks directly into Broadcast Wave Format container"))
    );
    descriptors.insert("BinauralHrtfInterpolator".to_string(), DspNodeDescriptor::new("BinauralHrtfInterpolator", "Continuous Spherical Harmonic 3D Binaural HRTF Interpolator HUD", DspNodeCategory::SpatialSurround, "Spherical harmonic 3D spatial HRTF interpolator computing continuous binaural pinna, head-shadow, and interaural time difference (ITD) filters for realistic 360-degree headphone monitoring")
        .with_param(DspParamSchema::knob("azimuth_angle_deg", "Horizontal Virtual Sound Source Azimuth Coordinate", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Horizontal compass bearing of virtual audio source relative to listener"))
        .with_param(DspParamSchema::knob("elevation_angle_deg", "Vertical Virtual Sound Source Elevation Angle", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical angle of virtual sound source above or below horizon plane"))
        .with_param(DspParamSchema::knob("source_distance_meters", "Listener-to-Source Distance Inverse-Square Attenuation", 0.2, 25.0, 2.0, "m", MacroRole::Tone, "Simulated physical distance in meters applying 6dB per doubling inverse-square falloff"))
        .with_param(DspParamSchema::choice("hrtf_dataset_model", "Acoustic Anatomical HRTF Impulse Response Dataset", &["KEMAR Dummy Head Studio", "CIPIC High-Resolution Grid", "LISTEN Individualized Subject", "Spherical Approximation (Low CPU)"], 0, "High-resolution measured Head-Related Impulse Response library"))
        .with_param(DspParamSchema::knob("pinna_spectral_notch_depth", "Anatomical Pinna Notch Reflection Intensity", 0.0, 1.0, 0.85, "%", MacroRole::Character, "Intensity of outer-ear pinna reflection notches providing elevation perception"))
    );
    descriptors.insert("SurroundDownmixer".to_string(), DspNodeDescriptor::new("SurroundDownmixer", "ITU-R BS.775 7.1.4 to Stereo Downmixing & Fold-Down Matrix HUD", DspNodeCategory::SpatialSurround, "ITU-R BS.775 compliant multi-channel surround downmixer folding 7.1.4 Dolby Atmos or 5.1 surround tracks into phase-aligned stereo or mono with customizable center/LFE attenuation")
        .with_param(DspParamSchema::knob("center_channel_attenuation_db", "Center Dialogue Channel Fold-Down Attenuation", -6.0, 0.0, -3.0, "dB", MacroRole::Tone, "Gain attenuation applied to center channel dialogue when summing into Left/Right"))
        .with_param(DspParamSchema::knob("surround_channel_attenuation_db", "Left/Right Surround and Height Channel Fold-Down Attenuation", -6.0, 0.0, -3.0, "dB", MacroRole::Space, "Gain attenuation applied to side and top height surround speakers in fold-down"))
        .with_param(DspParamSchema::knob("lfe_sub_channel_gain_db", "Low-Frequency Effects (LFE) Subwoofer Blend Level", -24.0, 0.0, -10.0, "dB", MacroRole::Punch, "Level of dedicated .1 subwoofer channel mixed into stereo speaker channels"))
        .with_param(DspParamSchema::toggle("surround_phase_shift_90deg", "Apply 90° Hilbert Phase Shift to Surrounds for Mono Compatibility", true, "Applies wideband 90-degree phase shift to surround channels preventing mono cancel"))
        .with_param(DspParamSchema::choice("downmix_output_mode", "Output Audio Channel Matrix & Downmix Profile", &["Stereo Standard (Lo/Ro)", "Pro Logic Compatible (Lt/Rt)", "Direct Mono Sum (M)", "Binaural Virtual Surround"], 0, "Standards-based downmix matrix routing surround beds to destination monitors"))
    );
    descriptors.insert("AtmosphericDensity".to_string(), DspNodeDescriptor::new("AtmosphericDensity", "Acoustic Humidity, Barometric Air Damping & Propagation HUD", DspNodeCategory::SpatialSurround, "ISO 9613-1 acoustic atmospheric attenuation model calculating frequency-dependent air absorption, relative humidity damping, and temperature sound velocity variations")
        .with_param(DspParamSchema::knob("ambient_temperature_celsius", "Ambient Environment Air Temperature", -20.0, 50.0, 20.0, "C", MacroRole::Tone, "Air temperature altering speed of sound and molecular relaxation frequencies"))
        .with_param(DspParamSchema::knob("relative_humidity_pct", "Atmospheric Relative Humidity Percentage", 5.0, 100.0, 50.0, "%", MacroRole::Space, "Humidity determining oxygen and nitrogen vibrational relaxation damping"))
        .with_param(DspParamSchema::knob("barometric_pressure_kpa", "Atmospheric Static Barometric Air Pressure", 70.0, 115.0, 101.3, "kPa", MacroRole::Tone, "Ambient barometric pressure modulating air molecular density"))
        .with_param(DspParamSchema::knob("propagation_distance_meters", "Acoustic Path Propagation Distance in Air", 1.0, 500.0, 25.0, "m", MacroRole::Space, "Physical distance from source applying cumulative ISO 9613-1 high-frequency loss"))
        .with_param(DspParamSchema::knob("doppler_wind_speed_m_per_s", "Vector Wind Speed Along Acoustic Propagation Axis", -30.0, 30.0, 0.0, "m/s", MacroRole::Punch, "Wind velocity creating positive or negative acoustic Doppler frequency shifts"))
    );
    descriptors.insert("AcousticLevitationTrap".to_string(), DspNodeDescriptor::new("AcousticLevitationTrap", "40kHz Ultrasonic Phased-Array Acoustic Levitation Trap HUD", DspNodeCategory::SpatialSurround, "Ultrasonic phased-array acoustic levitation transducer simulating acoustic radiation pressure, standing wave focal nodes, and Gor'kov acoustic potential wells in 3D space")
        .with_param(DspParamSchema::knob("ultrasonic_carrier_freq_khz", "Ultrasonic Transducer Array Carrier Frequency", 20.0, 80.0, 40.0, "kHz", MacroRole::Tone, "Operating carrier frequency of opposed piezoelectric transducer arrays"))
        .with_param(DspParamSchema::knob("transducer_array_power_watts", "Total Electrical Acoustic Array Driving Power", 0.5, 100.0, 24.0, "W", MacroRole::Punch, "Ultrasonic sound pressure level generating acoustic radiation force"))
        .with_param(DspParamSchema::knob("trap_focal_point_x_mm", "3D Standing Wave Node Focal Point X Coordinate", -100.0, 100.0, 0.0, "mm", MacroRole::Space, "Horizontal lateral position of Gor'kov acoustic potential trap node"))
        .with_param(DspParamSchema::knob("trap_focal_point_y_mm", "3D Standing Wave Node Focal Point Y Coordinate", -100.0, 100.0, 0.0, "mm", MacroRole::Space, "Horizontal depth position of Gor'kov acoustic potential trap node"))
        .with_param(DspParamSchema::knob("trap_focal_point_z_height_mm", "3D Standing Wave Node Levitation Height Z", 0.0, 200.0, 50.0, "mm", MacroRole::Space, "Vertical levitation elevation above lower ultrasonic emitter grid"))
    );
    descriptors.insert("MultichannelAudioBuffer".to_string(), DspNodeDescriptor::new("MultichannelAudioBuffer", "N-Channel Immersive & Surround Bus Audio Buffer", DspNodeCategory::SpatialSurround, "Zero-allocation multi-channel audio buffer supporting quadraphonic, 5.1, 7.1.4 Dolby Atmos, and 64-channel higher-order ambisonic soundfields")
        .with_param(DspParamSchema::choice("channels", "Discrete Audio Channel Layout Bus", &["Stereo (2)", "Quadraphonic (4)", "5.1 Surround (6)", "7.1.4 Atmos (12)", "22.2 Immersive (24)"], 2, "Channel count and spatial loudspeaker configuration"))
        .with_param(DspParamSchema::knob("frame_capacity", "Per-Channel Frame Allocation Length", 64.0, 2048.0, 512.0, "frames", MacroRole::Space, "Allocated sample storage per independent surround channel"))
        .with_param(DspParamSchema::toggle("interleaved", "Channel Sample Interleaving Memory Layout", false, "Interleaved vs de-interleaved channel memory arrangement"))
        .with_param(DspParamSchema::knob("master_gain", "Surround Master Bus Gain Multiplier", 0.0, 2.0, 1.0, "gain", MacroRole::Punch, "Master gain applied across all multi-channel tracks uniformly"))
        .with_param(DspParamSchema::knob("lfe_sub_cutoff_hz", "Low-Frequency Effects Subwoofer Crossover", 40.0, 200.0, 120.0, "Hz", MacroRole::Tone, "Low-pass crossover frequency feeding the dedicated LFE channel"))
    );
    descriptors.insert("SpatialWaypoint".to_string(), DspNodeDescriptor::new("SpatialWaypoint", "3D Spatial Trajectory Coordinate Node HUD", DspNodeCategory::SpatialSurround, "3D spatial audio coordinate node defining Cartesian X/Y/Z position, azimuth/elevation angles, arrival timestamp, and smooth bezier spatial flight velocity")
        .with_param(DspParamSchema::knob("coord_x", "Cartesian Sound Source X Position (Left-Right)", -50.0, 50.0, 0.0, "m", MacroRole::Space, "Lateral lateral distance relative to the listener origin"))
        .with_param(DspParamSchema::knob("coord_y", "Cartesian Sound Source Y Position (Front-Back)", -50.0, 50.0, 5.0, "m", MacroRole::Space, "Forward/backward distance relative to the listener origin"))
        .with_param(DspParamSchema::knob("coord_z", "Cartesian Sound Source Z Position (Height-Elevation)", -20.0, 20.0, 0.0, "m", MacroRole::Space, "Vertical elevation relative to the listener ear plane"))
        .with_param(DspParamSchema::knob("curve_tension", "Hermite Spline Trajectory Curve Tension", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Geometric tension curving the 3D trajectory through this point"))
        .with_param(DspParamSchema::knob("arrival_time_ms", "Waypoint Trajectory Arrival Timestamp", 0.0, 60000.0, 2500.0, "ms", MacroRole::Tone, "Timeline timestamp in milliseconds when audio emitter reaches node"))
    );
    descriptors.insert("AtomicSpatialSlot".to_string(), DspNodeDescriptor::new("AtomicSpatialSlot", "Lock-Free Atomic 3D Spatial Audio Coordinate Slot", DspNodeCategory::SpatialSurround, "Lock-free atomic 3D spatial coordinate slot bus supporting wait-free position updates across real-time DSP threads, binaural spatializers, and ambisonic encoders")
        .with_param(DspParamSchema::knob("azimuth_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Azimuth angle of sound source relative to listener center"))
        .with_param(DspParamSchema::knob("elevation_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Elevation inclination angle of spatial sound source"))
        .with_param(DspParamSchema::log_knob("distance_m", "Source Distance", 0.1, 100.0, 1.5, "m", MacroRole::Space, "Source distance in meters from listener origin"))
        .with_param(DspParamSchema::knob("spread_factor", "Sound Source Directivity Spread", 0.0, 1.0, 0.2, "spread", MacroRole::Space, "Angular directivity dispersion factor for spatial soundfield"))
        .with_param(DspParamSchema::knob("doppler_factor", "Relativistic Doppler Pitch Factor", 0.0, 2.0, 1.0, "x", MacroRole::Character, "Doppler pitch shifting factor driven by source velocity"))
    );
    descriptors.insert("Position3D".to_string(), DspNodeDescriptor::new("Position3D", "Cartesian 3D Spatial Position & Distance Coordinates", DspNodeCategory::SpatialSurround, "3D spatial vector coordinate descriptor specifying Cartesian X, Y, Z emitter location, velocity vector for Doppler shift, and acoustic air absorption roll-off")
        .with_param(DspParamSchema::knob("pos_x", "Lateral Right/Left X Position", -50.0, 50.0, 0.0, "m", MacroRole::Space, "Cartesian X coordinate relative to virtual listening position"))
        .with_param(DspParamSchema::knob("pos_y", "Vertical Elevation Y Position", -50.0, 50.0, 0.0, "m", MacroRole::Space, "Cartesian Y coordinate denoting height above/below ear level"))
        .with_param(DspParamSchema::knob("pos_z", "Depth Front/Back Z Position", -50.0, 50.0, 2.0, "m", MacroRole::Space, "Cartesian Z coordinate denoting depth in front of or behind listener"))
        .with_param(DspParamSchema::knob("air_absorption_hf_db", "High-Frequency Atmospheric Damping", -18.0, 0.0, -2.5, "dB", MacroRole::Tone, "Humidity and distance-dependent high frequency lowpass attenuation"))
        .with_param(DspParamSchema::toggle("inverse_square_attenuation", "Enable Physical 1/R Distance Attenuation", true, "Calculates natural 6 dB drop per doubling of distance"))
    );
    descriptors.insert("SpatialImpulseResponse".to_string(), DspNodeDescriptor::new("SpatialImpulseResponse", "Binaural Room Acoustic Spatial Transfer Impulse", DspNodeCategory::SpatialSurround, "Stereo and ambisonic binaural impulse response modeling physical acoustic reflections, RT60 decay time constant, direct sound path delay, and spherical elevation damping")
        .with_param(DspParamSchema::knob("sample_rate", "Acoustic Sampling Rate", 22050.0, 192000.0, 48000.0, "Hz", MacroRole::Tone, "Impulse response measurement native sampling frequency"))
        .with_param(DspParamSchema::knob("rt60_sec", "RT60 Reverberation Decay Time", 0.05, 15.0, 1.8, "s", MacroRole::Space, "Reverberation decay time for sound pressure level to attenuate by 60 dB"))
        .with_param(DspParamSchema::knob("direct_delay_ms", "Direct Path Arrival Pre-Delay", 0.0, 100.0, 4.5, "ms", MacroRole::Space, "Initial acoustic direct sound path arrival propagation delay"))
        .with_param(DspParamSchema::knob("elevation_damping_db", "Spherical Elevation HRIR Damping", -18.0, 0.0, -3.0, "dB", MacroRole::Tone, "Spherical head-related transfer function elevation spectral damping"))
        .with_param(DspParamSchema::toggle("air_absorption_hf", "Air Molecular High Damping", true, "Enable atmospheric air molecular frequency-dependent high damping"))
    );
    descriptors.insert("SpatialVector3D".to_string(), DspNodeDescriptor::new("SpatialVector3D", "Spherical & Cartesian 3D Spatial Audio Vector", DspNodeCategory::SpatialSurround, "High-precision 3D spatial position vector storing azimuth angle (-PI to +PI), elevation angle (-PI/2 to +PI/2), distance radius, Doppler shift velocity, and sound cone spread")
        .with_param(DspParamSchema::knob("azimuth_rad", "Horizontal Azimuth Angle", -3.14159, 3.14159, 0.0, "rad", MacroRole::Space, "Horizontal azimuth angle in radians (0 = front, +PI/2 = right, -PI/2 = left)"))
        .with_param(DspParamSchema::knob("elevation_rad", "Vertical Elevation Angle", -1.5708, 1.5708, 0.0, "rad", MacroRole::Space, "Vertical elevation angle in radians (0 = horizon, +PI/2 = zenith, -PI/2 = nadir)"))
        .with_param(DspParamSchema::knob("distance_m", "Listener Radial Distance Radius", 0.1, 100.0, 2.0, "m", MacroRole::Space, "Radial distance in meters from listener origin"))
        .with_param(DspParamSchema::knob("doppler_factor", "Acoustic Doppler Pitch Shift Ratio", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Relative acoustic velocity Doppler pitch shift multiplier"))
        .with_param(DspParamSchema::knob("spread_deg", "Virtual Sound Source Spread Aperture", 0.0, 180.0, 30.0, "deg", MacroRole::Character, "Virtual sound source aperture and spatial radiation width"))
    );
    descriptors.insert("RoomAcousticModel".to_string(), DspNodeDescriptor::new("RoomAcousticModel", "3D Spatial Acoustic Room Geometry & Early Reflections", DspNodeCategory::SpatialSurround, "Spatial room acoustics simulator configuring Image Source Method early specular reflections, Monte Carlo ray-tracing diffuse scattering, room shoebox dimensions, and listener orientation")
        .with_param(DspParamSchema::choice("simulation_acoustic_topology", "Room Acoustics Simulation Topology", &["Image Source Method (Accurate Specular)", "Monte Carlo Ray Tracing (Volumetric Diffuse)", "3D Finite-Difference Time-Domain Mesh", "Hybrid Early Image + Late Statistical Reverb"], 3, "Computational geometry model for acoustic simulation"))
        .with_param(DspParamSchema::knob("room_length_meters", "Acoustic Chamber Length Dimension", 2.0, 50.0, 12.0, "m", MacroRole::Space, "Shoebox room length dimension"))
        .with_param(DspParamSchema::knob("room_width_meters", "Acoustic Chamber Width Dimension", 2.0, 40.0, 8.5, "m", MacroRole::Character, "Shoebox room width dimension"))
        .with_param(DspParamSchema::knob("room_height_meters", "Acoustic Chamber Ceiling Height", 2.0, 20.0, 5.0, "m", MacroRole::Tone, "Shoebox room ceiling height dimension"))
        .with_param(DspParamSchema::knob("air_absorption_humidity_pct", "Relative Ambient Air Humidity for HF Loss", 10.0, 90.0, 45.0, "%", MacroRole::Punch, "Relative humidity controlling ISO 9613-1 air absorption"))
        .with_param(DspParamSchema::toggle("enable_doppler_moving_source", "Calculate Moving Source Doppler Acoustic Shift", true, "Model Doppler pitch shift when sound sources move through chamber"))
    );
    descriptors.insert("WallAcousticMaterial".to_string(), DspNodeDescriptor::new("WallAcousticMaterial", "Acoustic Boundary Surface Material & Scattering Coefficient", DspNodeCategory::SpatialSurround, "Boundary surface acoustic properties selector modeling frequency-dependent sound absorption coefficients, Lambertian diffuse scattering, air cavity depth, and boundary impedance")
        .with_param(DspParamSchema::choice("boundary_wall_material", "Surface Construction Material & Absorption", &["Polished Poured Concrete (Hard Reflective)", "Heavy Velvet Pleated Drapery (HF Absorptive)", "1-Inch Solid Maple Wood Paneling (Warm Resonant)", "Acoustic Fiberglass Gypsum Baffles (Linear Absorb)", "Double-Glazed Thermal Windowpane Glass"], 2, "Acoustic wall lining and boundary material"))
        .with_param(DspParamSchema::knob("scattering_diffusion_coefficient", "Surface Lambertian Roughness Diffusion", 0.0, 1.0, 0.4, "coeff", MacroRole::Space, "Fraction of reflected sound scattered diffusely vs specularly"))
        .with_param(DspParamSchema::knob("low_freq_absorption_125hz", "125 Hz Sub-Bass Absorption Factor", 0.02, 0.8, 0.15, "alpha", MacroRole::Punch, "Sound absorption coefficient in 125 Hz octave band"))
        .with_param(DspParamSchema::knob("mid_freq_absorption_1khz", "1 kHz Mid-Frequency Absorption Factor", 0.05, 0.95, 0.35, "alpha", MacroRole::Tone, "Sound absorption coefficient in 1 kHz octave band"))
        .with_param(DspParamSchema::knob("high_freq_absorption_8khz", "8 kHz High-Frequency Absorption Factor", 0.1, 0.99, 0.65, "alpha", MacroRole::Character, "Sound absorption coefficient in 8 kHz octave band"))
        .with_param(DspParamSchema::toggle("model_diaphragmatic_bass_trapping", "Simulate Panel Flexure Bass Trapping", true, "Simulate panel resonance absorption in low frequencies"))
    );
    descriptors.insert("TrajectoryPathType".to_string(), DspNodeDescriptor::new("TrajectoryPathType", "3D Spatial Trajectory Path Generator", DspNodeCategory::SpatialSurround, "Parametric 3D spatial trajectory path generator producing continuous orbital, Lissajous knot, logarithmic spiral, and flyby coordinate vectors with Catmull-Rom spline interpolation")
        .with_param(DspParamSchema::choice("path_mode", "Parametric Trajectory Geometry Profile", &["Orbit (Elliptical Circular Rotation)", "Lissajous3D (Harmonic Spatial Knot)", "Spiral (Inward Outward Conical Motion)", "Flyby (Linear Coordinate Transit)", "WaypointSpline (Keyframed Catmull-Rom)"], 0, "Parametric geometric shape governing sound source movement"))
        .with_param(DspParamSchema::knob("orbit_radius_m", "Orbital Radius from Listener Center", 0.5, 50.0, 3.0, "m", MacroRole::Space, "Radial distance of the trajectory center from the listener"))
        .with_param(DspParamSchema::knob("speed_cycles_per_beat", "Trajectory Orbital Velocity per Beat", 0.01, 8.0, 0.25, "cyc/b", MacroRole::Tone, "Rotational frequency synchronized to timeline tempo beats"))
        .with_param(DspParamSchema::knob("elevation_tilt_rad", "Spatial Orbital Plane Elevation Tilt", -3.14, 3.14, 0.0, "rad", MacroRole::Character, "Inclination angle of the orbital trajectory plane"))
        .with_param(DspParamSchema::knob("orbit_eccentricity", "Elliptical Orbit Eccentricity Ratio", 0.0, 0.99, 0.0, "ecc", MacroRole::Punch, "Eccentricity flattening circular orbit into elongated ellipse"))
        .with_param(DspParamSchema::toggle("enable_catmull_rom_smoothing", "Engage Cubic Catmull-Rom Spline Easing", true, "Interpolate smooth continuous curvature through trajectory points"))
    );
    descriptors.insert("SpatialInterpolationCurve".to_string(), DspNodeDescriptor::new("SpatialInterpolationCurve", "Spatial 3D Trajectory Keyframe Interpolation Curve", DspNodeCategory::SpatialSurround, "Lock-free 3D coordinate vector keyframe interpolation curve selector supporting linear, cubic Catmull-Rom spline, smoothstep, and sample-and-hold transitions")
        .with_param(DspParamSchema::choice("curve_type", "Spatial Keyframe Curve Transition Algorithm", &["Linear (Direct Vector Lerp)", "CubicCatmullRom (Continuous Spline)", "SmoothStep (Hermite S-Curve Easing)", "Hold (Discrete Keyframe Step)"], 1, "Mathematical interpolation method connecting spatial keyframes"))
        .with_param(DspParamSchema::knob("spline_tension", "Catmull-Rom Hermite Spline Tension Factor", 0.0, 1.0, 0.5, "tension", MacroRole::Character, "Tightness of curvature through intermediate 3D waypoints"))
        .with_param(DspParamSchema::knob("smoothing_window_ms", "Position Smoothing Slew Window Duration", 0.1, 100.0, 10.0, "ms", MacroRole::Space, "Temporal lowpass filter removing coordinate jitter transients"))
        .with_param(DspParamSchema::knob("azimuth_deadzone_rad", "Azimuth Micro-Movement Noise Deadzone", 0.0, 0.2, 0.01, "rad", MacroRole::Tone, "Suppression threshold for sub-degree tracking position noise"))
        .with_param(DspParamSchema::knob("distance_damping_ratio", "Radial Distance Slew Velocity Damping", 0.1, 2.0, 1.0, "damp", MacroRole::Punch, "Inertial damping applied to rapid depth distance changes"))
        .with_param(DspParamSchema::toggle("enable_elevation_clamping", "Clamp Elevation Vector to Hemispherical Limits", true, "Prevent audio source trajectory inversion beyond zenith and nadir"))
    );
    descriptors.insert("WallMaterial".to_string(), DspNodeDescriptor::new("WallMaterial", "Acoustic Space Boundary Wall Material & Absorption Spectrum", DspNodeCategory::SpatialSurround, "Acoustic room impulse simulation boundary material calculating multi-band sound absorption, diffuse scattering, and specular reflection coefficients for concrete, wood, acoustic studio foam, carpet, glass, and cathedral stone")
        .with_param(DspParamSchema::choice("material_type", "Acoustic Boundary Surface Material", &["Concrete", "Wood Paneling", "Studio Acoustic Foam", "Carpet on Felt", "Architectural Glass", "Cathedral Stone"], 2, "Selects frequency-dependent absorption, scattering coefficients, and structural transmission loss"))
        .with_param(DspParamSchema::knob("absorption_low", "Low-Frequency Absorption Coefficient (<250 Hz)", 0.01, 0.99, 0.15, "coeff", MacroRole::Tone, "Fraction of low-frequency acoustic energy absorbed by boundary surface on each reflection"))
        .with_param(DspParamSchema::knob("absorption_mid", "Mid-Frequency Absorption Coefficient (250-2000 Hz)", 0.01, 0.99, 0.55, "coeff", MacroRole::Space, "Fraction of mid-frequency acoustic energy absorbed governing vocal clarity and warmth"))
        .with_param(DspParamSchema::knob("absorption_high", "High-Frequency Absorption Coefficient (>2000 Hz)", 0.01, 0.99, 0.85, "coeff", MacroRole::Tone, "Fraction of treble acoustic energy absorbed preventing flutter echo and harsh reflections"))
        .with_param(DspParamSchema::knob("scattering_roughness", "Diffuse Lambertian Surface Scattering Ratio", 0.0, 1.0, 0.42, "ratio", MacroRole::Punch, "Fraction of reflected sound energy dispersed diffusely rather than specularly reflected"))
        .with_param(DspParamSchema::knob("transmission_loss_db", "Through-Wall Acoustic Sound Transmission Loss", 5.0, 80.0, 45.0, "dB", MacroRole::Space, "Attenuation of sound waves passing through boundary walls into adjacent virtual acoustic rooms"))
    );
    descriptors.insert("CliAdmSpatialExporter".to_string(), DspNodeDescriptor::new("CliAdmSpatialExporter", "ITU-R BS.2076 ADM & Dolby Atmos Broadcast Audio Exporter", DspNodeCategory::SpatialSurround, "Spatial audio renderer encoding dynamic 3D object positions, XYZ coordinates, room metadata, and bed channels into standardized Broadcast WAVE ADM XML chunks")
        .with_param(DspParamSchema::choice("adm_channel_configuration", "ADM Speaker Bed & Object Configuration", &["7.1.4 Immersive Bed", "9.1.6 Dolby Atmos", "5.1.2 Surround", "22.2 Super Hi-Vision", "Binaural Preview"], 0, "Speaker layout bed channels plus dedicated dynamic spatial sound objects"))
        .with_param(DspParamSchema::knob("spatial_trajectory_fps", "Object Vector Metadata Trajectory Frame Rate", 24.0, 120.0, 60.0, "fps", MacroRole::Space, "Temporal sample rate for 3D positional coordinate automation metadata"))
        .with_param(DspParamSchema::knob("binaural_distance_attenuation_db", "Binaural Head Shadow & Distance Rolloff", 0.0, 24.0, 6.0, "dB", MacroRole::Tone, "High-frequency head shadow absorption and inverse-square distance attenuation"))
        .with_param(DspParamSchema::toggle("enforce_ebu_r128_loudness", "Strict EBU R128 (-23 LUFS) Loudness Compliance Check", true, "Verifies master mix does not exceed broadcast standard loudness limits"))
        .with_param(DspParamSchema::knob("lfe_crossover_hz", "Subwoofer LFE Channel Low-Pass Crossover", 40.0, 160.0, 80.0, "Hz", MacroRole::Punch, "Cutoff frequency filtering low frequencies into discrete LFE subwoofer channel"))
        .with_param(DspParamSchema::toggle("write_xml_metadata_chunk", "Embed Complete BS.2076 axml and chna WAV Metadata", true, "Injects standard ADM XML definition chunks directly into output BWF file"))
    );
    descriptors.insert("BinauralHeadphoneEqualizer".to_string(), DspNodeDescriptor::new("BinauralHeadphoneEqualizer", "Diffuse-Field & Harman Curve Headphone Calibration Filter", DspNodeCategory::SpatialSurround, "Headphone target response compensator with natural crossfeed matrix, reducing in-head acoustic localization fatigue")
        .with_param(DspParamSchema::choice("target_equalization_curve", "Target Acoustic EQ Target", &["Harman Target 2019", "Diffuse-Field Flat", "Free-Field Speaker Simulation", "Custom Studio Target"], 0, "Acoustic calibration target compensating for headphone ear canal resonances"))
        .with_param(DspParamSchema::knob("crossfeed_frequency_cutoff_hz", "Crossfeed Acoustic Cutoff", 300.0, 1200.0, 650.0, "Hz", MacroRole::Space, "Low-pass crossover frequency for interaural crosstalk bleed"))
        .with_param(DspParamSchema::knob("crossfeed_feed_gain_db", "Crossfeed Bleed Gain", -18.0, -3.0, -9.0, "dB", MacroRole::Space, "Amplitude level of contralateral ear acoustic leakage"))
        .with_param(DspParamSchema::knob("inter_aural_time_delay_us", "Interaural Head Delay (ITD)", 100.0, 450.0, 280.0, "us", MacroRole::Space, "Time delay simulating physical head diameter shadow (approx. 17.5 cm)"))
        .with_param(DspParamSchema::knob("bass_shelf_boost_db", "Target Bass Shelf Boost", 0.0, 12.0, 4.5, "dB", MacroRole::Tone, "Sub-bass target shelf compensating for headphone acoustic seal leakage"))
        .with_param(DspParamSchema::toggle("high_treble_air_damping", "Smooth High-Treble Air Damping", true, "Softens piercing treble peaks above 10 kHz to minimize ear fatigue"))
    );
    descriptors.insert("TransientSplayStereoWidener".to_string(), DspNodeDescriptor::new("TransientSplayStereoWidener", "Transient Decorrelation & Haas Effect Stereo Imager", DspNodeCategory::SpatialSurround, "Mono-compatible stereo widening matrix applying micro-delays and phase-decorrelation exclusively to high-frequency transient peaks")
        .with_param(DspParamSchema::knob("stereo_spread_width_percentage", "Stereo Spatial Spread Width", 0.0, 200.0, 120.0, "%", MacroRole::Space, "Width multiplier expanding stereo image (100% = neutral, 200% = ultra-wide)"))
        .with_param(DspParamSchema::knob("haas_delay_offset_ms", "Haas Micro-Delay Offset", 0.1, 25.0, 7.0, "ms", MacroRole::Space, "Interaural Haas delay time creating psychoacoustic spatial localization"))
        .with_param(DspParamSchema::knob("transient_detector_sensitivity", "Transient Trigger Sensitivity", 0.1, 1.0, 0.6, "sens", MacroRole::Punch, "Threshold sensitivity triggering stereo widening solely on percussive attack onsets"))
        .with_param(DspParamSchema::knob("low_frequency_mono_crossover_hz", "Bass Mono Crossover Frequency", 60.0, 350.0, 150.0, "Hz", MacroRole::Tone, "Retains low frequencies below crossover in strict center mono to preserve punch"))
        .with_param(DspParamSchema::choice("decorrelation_filter_stages", "Allpass Phase Decorrelation Stages", &["2 Allpass Stages", "4 Allpass Stages", "8 Complex Lattice Stages"], 1, "Number of cascaded allpass filter stages decorrelating left and right channels"))
        .with_param(DspParamSchema::toggle("strict_mono_compatibility_guard", "Strict Mono Cancellation Guard", true, "Guarantees complete phase cancellation immunity when summed to single mono channel"))
    );
    descriptors.insert("MidSideSpectralHarmonizer".to_string(), DspNodeDescriptor::new("MidSideSpectralHarmonizer", "Mid/Side Spectral Harmonic Enhancer & Spatial Stereo Widener", DspNodeCategory::SpatialSurround, "Decoupled mid/side psychoacoustic harmonic saturator injecting warmth into the center image while generating wide, uncorrelated ambient air in the side channel")
        .with_param(DspParamSchema::knob("mid_tube_saturation_drive", "Mid Channel Tube Saturation Drive", 0.0, 1.0, 0.25, "drive", MacroRole::Punch, "In-phase mono center harmonic saturation adding presence and body to lead instruments"))
        .with_param(DspParamSchema::knob("side_air_exciter_gain_db", "Side Channel Air Harmonic Exciter", -12.0, 12.0, 2.5, "dB", MacroRole::Tone, "Out-of-phase stereo difference high-frequency exciter expanding perceived space"))
        .with_param(DspParamSchema::knob("stereo_spread_width_ratio", "Global Stereo Field Width", 0.5, 2.5, 1.25, "width", MacroRole::Space, "Mid/side balance multiplier controlling overall stereo soundstage breadth"))
        .with_param(DspParamSchema::knob("crossover_bass_mono_hz", "Bass Mono Fold Crossover", 60.0, 400.0, 140.0, "Hz", MacroRole::Tone, "Folds all frequencies below cutoff into pure mono center to maximize club punch"))
        .with_param(DspParamSchema::knob("harmonic_odd_even_balance", "Harmonic Order Odd/Even Balance", -1.0, 1.0, 0.0, "bal", MacroRole::Character, "Blend between warm 2nd-order even harmonics and cutting 3rd-order odd harmonics"))
        .with_param(DspParamSchema::toggle("mono_phase_guard_enabled", "Strict Mono Cancellation Guard", true, "Protects against complete phase cancellation when stereo audio is collapsed to mono"))
    );
    descriptors.insert("BinauralAmbisonicDecoder".to_string(), DspNodeDescriptor::new("BinauralAmbisonicDecoder", "Higher-Order Ambisonics (HOA 7th-Order) to Binaural HRTF Renderer", DspNodeCategory::SpatialSurround, "Spherical harmonic 3D spatial audio decoder converting up to 7th-order (64-channel) Ambisonic soundfields into binaural headphone audio with individualized pinna filtering")
        .with_param(DspParamSchema::choice("ambisonic_order_order", "Ambisonic Spherical Order", &["1st Order (4-Ch)", "3rd Order (16-Ch)", "5th Order (36-Ch)", "7th Order (64-Ch)"], 1, "Number of spherical harmonic decomposition channels used for spatial reconstruction"))
        .with_param(DspParamSchema::choice("hrtf_profile_database", "HRTF Head Profile Database", &["KEMAR Dummy Head Standard", "Diffuse-Field Equalized", "Individualized Pinna Model", "Miniature Boundary Array"], 0, "Selected Head-Related Transfer Function acoustic impulse database"))
        .with_param(DspParamSchema::knob("head_tracking_yaw_deg", "Head-Tracking Horizontal Yaw", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth head rotation compensating for VR/AR orientation"))
        .with_param(DspParamSchema::knob("head_tracking_pitch_deg", "Head-Tracking Vertical Pitch", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation tilt angle compensating for head motion"))
        .with_param(DspParamSchema::knob("near_field_compensation_distance", "Near-Field Parallax Distance", 0.2, 10.0, 1.5, "m", MacroRole::Space, "Applies acoustic curvature compensation for sound sources closer than 1 meter"))
        .with_param(DspParamSchema::toggle("diffuse_field_sound_correction", "Diffuse-Field Equalization Correction", true, "Compensates for headphone diffuse-field response to prevent high-frequency fatigue"))
    );
    descriptors.insert("DolbyAtmosBedDownmixer".to_string(), DspNodeDescriptor::new("DolbyAtmosBedDownmixer", "Dolby Atmos 7.1.4 / 9.1.6 Bed Downmixer & ITU Fold-Down Matrix", DspNodeCategory::SpatialSurround, "Broadcast-compliant immersive audio fold-down matrix downmixing multi-channel height and surround speaker configurations to clean stereo, binaural, or 5.1 surrounds")
        .with_param(DspParamSchema::choice("input_bed_format", "Source Immersive Speaker Bed", &["7.1.4 Atmos Bed (12-Ch)", "9.1.6 Immersive Bed (16-Ch)", "5.1.2 Surround Bed (8-Ch)", "22.2 Broadcast NHK"], 0, "Input multi-channel speaker configuration to be folded down"))
        .with_param(DspParamSchema::knob("center_downmix_level_db", "Center Channel Fold-Down Gain", -6.0, 0.0, -3.0, "dB", MacroRole::Tone, "Attenuation applied when summing dialogue center channel into left/right stereo"))
        .with_param(DspParamSchema::knob("surround_downmix_level_db", "Surround Channel Fold-Down Gain", -9.0, 0.0, -3.0, "dB", MacroRole::Space, "Attenuation applied to rear surround speakers folded into stereo"))
        .with_param(DspParamSchema::knob("height_channel_gain_db", "Overhead Height Channels Gain", -12.0, 0.0, -4.5, "dB", MacroRole::Space, "Level of ceiling height speakers mixed into horizontal ear-level beds"))
        .with_param(DspParamSchema::choice("lfe_subwoofer_downmix_mode", "Subwoofer LFE Channel Routing", &["Discard LFE Channel", "Sum to Main (-10 dB)", "Route to Sub Aux"], 0, "Handling of dedicated Low Frequency Effects (.1) channel during downmixing"))
        .with_param(DspParamSchema::toggle("enable_itu_r_bs775_gain_normalizer", "ITU-R BS.775 Normalization Coefficient", true, "Applies standardized downmix coefficients preventing output clipping"))
    );
    descriptors.insert("AcousticRoomRaytracer".to_string(), DspNodeDescriptor::new("AcousticRoomRaytracer", "Acoustic Boundary Raytracing Engine & Diffuse Wavefront Simulator", DspNodeCategory::SpatialSurround, "Geometric room acoustics solver tracing thousands of audio sound rays against 3D room geometries to calculate early reflections, specular scattering, and absorption")
        .with_param(DspParamSchema::knob("sound_ray_count", "Dispatched Sound Rays Count", 500.0, 20000.0, 4000.0, "rays", MacroRole::Space, "Number of Monte Carlo acoustic rays fired into 3D geometric virtual environment"))
        .with_param(DspParamSchema::knob("maximum_reflection_bounces", "Maximum Reflection Bounces Depth", 1.0, 32.0, 8.0, "bounces", MacroRole::Space, "Maximum number of wall collisions simulated before terminating ray trajectory"))
        .with_param(DspParamSchema::knob("room_wall_absorption_coefficient", "Boundary Wall Absorption Factor", 0.01, 0.95, 0.25, "alpha", MacroRole::Tone, "Proportion of sound energy absorbed on each boundary wall collision"))
        .with_param(DspParamSchema::knob("diffuse_scattering_fraction", "Lambertian Diffuse Scattering Ratio", 0.0, 1.0, 0.35, "scat", MacroRole::Space, "Fraction of sound energy scattered randomly versus specular mirror reflection"))
        .with_param(DspParamSchema::knob("room_air_humidity_percentage", "Ambient Room Air Humidity", 10.0, 90.0, 45.0, "%", MacroRole::None, "Relative humidity governing molecular atmospheric attenuation of high treble"))
        .with_param(DspParamSchema::toggle("enable_edge_diffraction_modeling", "Simulate Obstacle Edge Diffraction", true, "Models bending of acoustic soundwaves around architectural pillars and acoustic baffles"))
    );
    descriptors.insert("SpatialBinauralPanner3D".to_string(), DspNodeDescriptor::new("SpatialBinauralPanner3D", "7th-Order Ambisonic Spherical HRTF Binaural Spatializer", DspNodeCategory::SpatialSurround, "Full 3D spherical coordinate binaural panner combining 7th-order Higher Order Ambisonics (HOA) with pinna spectral notches, interaural time difference (ITD), interaural level difference (ILD), and distance attenuation")
        .with_param(DspParamSchema::knob("azimuth_angle_degrees", "Horizontal Sound Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular position of sound source (0 = center, +/-90 = sides, +/-180 = rear)"))
        .with_param(DspParamSchema::knob("elevation_angle_degrees", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical sound elevation (-90 = below, 0 = ear level, +90 = zenith directly overhead)"))
        .with_param(DspParamSchema::knob("source_distance_meters", "Acoustic Radial Source Distance", 0.1, 50.0, 2.0, "m", MacroRole::Space, "Physical distance from listener center with inverse-square law air absorption"))
        .with_param(DspParamSchema::knob("pinna_spectral_notch_depth", "Pinna Concha Spectral Notch Depth", 0.0, 1.0, 0.7, "depth", MacroRole::Tone, "Human outer ear pinna spectral carving establishing front-vs-back discrimination"))
        .with_param(DspParamSchema::knob("head_shadow_dispersion_ratio", "Acoustic Head Shadow Dispersion", 0.1, 1.0, 0.65, "ratio", MacroRole::Character, "Head acoustic shadowing frequency roll-off on contralateral ear"))
        .with_param(DspParamSchema::toggle("doppler_motion_tracking", "Dynamic Doppler Frequency Shift", true, "Calculates acoustic pitch Doppler shift during high-speed source motion"))
    );
    descriptors.insert("AmbisonicRotatorBinaural".to_string(), DspNodeDescriptor::new("AmbisonicRotatorBinaural", "3D Ambisonic B-Format Yaw/Pitch/Roll Spherical Rotator", DspNodeCategory::SpatialSurround, "Full 3D rotation matrix for spherical Ambisonic audio signals (B-format up to 3rd order) enabling interactive yaw, pitch, and roll spatial rotations with head-tracking integration for VR and gaming")
        .with_param(DspParamSchema::knob("yaw_rotation_degrees", "Horizontal Yaw Rotation Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal head turning angle (panning soundfield around vertical Z axis)"))
        .with_param(DspParamSchema::knob("pitch_tilt_degrees", "Vertical Pitch Tilt Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical head nodding angle (tilting soundfield up or down)"))
        .with_param(DspParamSchema::knob("roll_bank_degrees", "Sideways Roll Banking Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Sideways ear-to-shoulder tilting angle"))
        .with_param(DspParamSchema::choice("ambisonic_decoding_order", "Ambisonic Spherical Order", &["1st Order (4 Channels B-Format)", "2nd Order (9 Channels HOA)", "3rd Order (16 Channels Studio HOA)"], 1, "Spatial spherical resolution and channel count"))
        .with_param(DspParamSchema::knob("head_tracker_smoothing_ms", "Head-Tracker Smoothing Constant", 0.0, 50.0, 12.0, "ms", MacroRole::Character, "Lowpass smoothing applied to raw sensor orientation data preventing jitter"))
        .with_param(DspParamSchema::toggle("near_field_effect_compensation", "Near-Field Bass Distance Compensation", true, "Compensates for low-frequency proximity boost in spherical wavefronts"))
    );
    descriptors.insert("BinauralNearfieldSimulator".to_string(), DspNodeDescriptor::new("BinauralNearfieldSimulator", "Spherical Head Parallax Nearfield HRTF Spatializer", DspNodeCategory::SpatialSurround, "Acoustic nearfield binaural renderer simulating wave curvature parallax, pinna notch elevation filtering, and dynamic head shadowing for distances within 1 meter")
        .with_param(DspParamSchema::knob("azimuth_angle_deg", "Horizontal Azimuth Direction", -180.0, 180.0, 35.0, "deg", MacroRole::Space, "Horizontal sound source angle in degrees (negative left, positive right)"))
        .with_param(DspParamSchema::knob("elevation_angle_deg", "Vertical Elevation Direction", -90.0, 90.0, 15.0, "deg", MacroRole::Tone, "Vertical elevation relative to interaural eye level"))
        .with_param(DspParamSchema::knob("radial_distance_meters", "Radial Proximity Distance", 0.05, 3.0, 0.45, "m", MacroRole::Space, "Distance between listener head center and virtual acoustic sound emitter"))
        .with_param(DspParamSchema::knob("head_circumference_cm", "Anthropometric Head Circumference", 48.0, 64.0, 56.0, "cm", MacroRole::Character, "Listener cranial circumference tuning interaural time difference (ITD)"))
        .with_param(DspParamSchema::choice("pinna_spectral_contour", "Pinna Notch Acoustic Profile", &["Standard KEMAR Model", "High-Definition Diffuse Field", "Compact Ear Canal", "Wide Interaural Contour"], 0, "Anatomical pinna reflection model sculpting high-frequency elevation cues"))
        .with_param(DspParamSchema::toggle("enable_head_tracking_compensation", "Dynamic Head-Tracking Yaw Lock", false, "Inverts spatial pan coordinates dynamically to track listener IMU orientation"))
    );
    descriptors.insert("HarmonicStereoSpreadWidener".to_string(), DspNodeDescriptor::new("HarmonicStereoSpreadWidener", "Frequency-Comb Psychoacoustic Stereo Imager & Mono Guard", DspNodeCategory::SpatialSurround, "Complementary multi-band comb Haas imager that widens stereo panoramas without introducing destructive center-channel cancellations when summed to mono")
        .with_param(DspParamSchema::knob("stereo_spread_width_percent", "Stereo Panorama Expansion", 0.0, 250.0, 135.0, "%", MacroRole::Space, "Stereo width multiplier ranging from collapsed mono (0%) to hyper-wide (250%)"))
        .with_param(DspParamSchema::knob("comb_filter_spacing_hz", "Comb Interference Frequency Spacing", 50.0, 800.0, 240.0, "Hz", MacroRole::Tone, "Bandwidth interval of complementary Left/Right spectral distribution notches"))
        .with_param(DspParamSchema::knob("center_channel_retention", "Phantom Center Anchor Retention", 0.0, 1.0, 0.85, "ret", MacroRole::Punch, "Direct Mid-channel preservation ensuring vocal presence remains firmly locked"))
        .with_param(DspParamSchema::knob("bass_mono_cutoff_frequency_hz", "Bass Mono Collapse Frequency", 40.0, 350.0, 120.0, "Hz", MacroRole::Punch, "High-pass corner frequency below which all low end is strictly centered to mono"))
        .with_param(DspParamSchema::knob("haas_micro_delay_ms", "Psychoacoustic Haas Delay Offset", 0.5, 25.0, 8.5, "ms", MacroRole::Space, "Complementary micro-delay timing separating lateral reflection wavefronts"))
        .with_param(DspParamSchema::toggle("mono_correlation_safety_guard", "Enforce Mono Correlation Safety", true, "Dynamic limiter preventing phase correlation meter from dropping below zero"))
    );
    descriptors.insert("MultichannelSurroundBedUpmixer".to_string(), DspNodeDescriptor::new("MultichannelSurroundBedUpmixer", "Stereo to 7.1.4 Immersive 3D Audio Bed Upmixer", DspNodeCategory::SpatialSurround, "Psychoacoustic immersive upmixer converting two-channel stereo signals into 5.1/7.1.4 surround beds with center-dialog extraction, height diffuse decorrelation, and sub LFE filtering")
        .with_param(DspParamSchema::knob("center_extraction_divergence", "Center Dialog Extraction Divergence", 0.0, 1.0, 0.65, "div", MacroRole::Punch, "Separates coherent correlated audio energy and routes it to discrete Center channel"))
        .with_param(DspParamSchema::knob("surround_diffuse_decorrelation", "Surround Diffuse Decorrelation", 0.0, 1.0, 0.55, "diff", MacroRole::Space, "Phase decorrelation injected into rear and surround side speaker channels"))
        .with_param(DspParamSchema::knob("height_layer_level_db", "Top / Height Ceiling Layer Gain", -24.0, 6.0, -4.0, "dB", MacroRole::Tone, "Gain trim for four top ceiling speakers (Left/Right Front/Rear Height)"))
        .with_param(DspParamSchema::knob("lfe_subwoofer_crossover_hz", "LFE Subwoofer Extraction Crossover", 40.0, 160.0, 80.0, "Hz", MacroRole::Punch, "Steep 4th-order low-pass filter routing low bass to dedicated .1 LFE channel"))
        .with_param(DspParamSchema::choice("surround_bed_target_format", "Speaker Layout Target Bed", &["5.1 Surround", "7.1 Cinema", "5.1.4 Immersive Height", "7.1.4 Atmos Bed"], 3, "Output multichannel channel layout geometry destination"))
        .with_param(DspParamSchema::knob("front_to_back_balance", "Front / Rear Surround Energy Balance", -1.0, 1.0, 0.1, "bal", MacroRole::Space, "Energy bias weighting soundstage toward front proscenium (-1.0) or rear immersion (+1.0)"))
    );
    descriptors.insert("DynamicAirAbsorptionAcoustics".to_string(), DspNodeDescriptor::new("DynamicAirAbsorptionAcoustics", "ISO 9613-1 Atmospheric Ultrasonic Damping & Humidity Filter", DspNodeCategory::SpatialSurround, "Physical acoustic medium simulation implementing ISO 9613-1 air absorption dissipation curves as a function of temperature, barometric pressure, relative humidity, and sound travel distance")
        .with_param(DspParamSchema::knob("propagation_distance_meters", "Acoustic Path Propagation Distance", 1.0, 200.0, 25.0, "m", MacroRole::Space, "Physical travel distance from virtual source to listener position"))
        .with_param(DspParamSchema::knob("ambient_temperature_celsius", "Ambient Medium Temperature", -10.0, 45.0, 20.0, "C", MacroRole::Tone, "Air temperature dictating molecular relaxation and sound speed"))
        .with_param(DspParamSchema::knob("relative_humidity_percent", "Atmospheric Relative Humidity", 10.0, 99.0, 50.0, "%", MacroRole::Tone, "Moisture percentage governing ultrasonic atmospheric absorption roll-off"))
        .with_param(DspParamSchema::knob("barometric_pressure_kpa", "Atmospheric Static Pressure", 80.0, 110.0, 101.3, "kPa", MacroRole::Character, "Static atmospheric pressure modifying molecular gas relaxation constants"))
        .with_param(DspParamSchema::choice("air_viscosity_relaxation_mode", "Molecular Relaxation Model", &["ISO 9613-1 Molecular O2/N2", "Standard Room Acoustics", "High-Altitude Rarefied Air", "Tropical Dense Medium"], 0, "Relaxation equation formulation for diatomic nitrogen and oxygen resonance"))
        .with_param(DspParamSchema::toggle("enable_high_frequency_air_roll_off", "Apply Ultrasonic Air Dissipation", true, "Engages frequency-dependent atmospheric attenuation filter bank"))
    );
    descriptors.insert("MidSideSpectralStereoImager".to_string(), DspNodeDescriptor::new("MidSideSpectralStereoImager", "3-Band Psychoacoustic Mid/Side Phase Correlation Expander", DspNodeCategory::SpatialSurround, "Multiband M/S spatial processor with frequency-dependent stereophonic width expanding, mono bass centering below adjustable cutoff, and inter-channel phase correlation metering")
        .with_param(DspParamSchema::knob("low_band_stereo_width", "Sub-Bass Stereo Field Width", 0.0, 2.0, 0.0, "x", MacroRole::Punch, "Width multiplier for low frequencies (0.0 collapses sub-bass to pure mono)"))
        .with_param(DspParamSchema::knob("mid_band_stereo_width", "Midrange Body Stereo Field Width", 0.0, 2.0, 1.15, "x", MacroRole::Space, "Width multiplier expanding vocal and instrumental stereophonic image"))
        .with_param(DspParamSchema::knob("high_band_stereo_width", "Air Band Treble Stereo Width", 0.0, 2.5, 1.35, "x", MacroRole::Space, "High-frequency side-channel expansion for ultra-wide mastering sheen"))
        .with_param(DspParamSchema::knob("bass_mono_crossover_hz", "Mono Bass Crossover Cutoff Frequency", 40.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Low-frequency boundary below which side content is smoothly attenuated"))
        .with_param(DspParamSchema::knob("side_harmonic_presence_db", "Side-Channel Harmonic Presence Boost", -12.0, 12.0, 1.5, "dB", MacroRole::Character, "High-shelf presence boost applied exclusively to side channel"))
        .with_param(DspParamSchema::toggle("enable_linear_phase_midside_crossover", "Linear Phase Crossover Filtering", true, "Zero-phase Linkwitz-Riley crossover preventing phase smear between bands"))
    );
    descriptors.insert("AmbisonicBFormatDecoder".to_string(), DspNodeDescriptor::new("AmbisonicBFormatDecoder", "Higher-Order Ambisonic (HOA) Spherical Harmonic to Speaker Decoder", DspNodeCategory::SpatialSurround, "3D ambisonic decoder matrix transforming Higher-Order Ambisonic B-format signals (W, X, Y, Z, R, S, T, U, V) into surround loudspeaker arrays with distance energy normalization")
        .with_param(DspParamSchema::choice("ambisonic_order", "Spherical Harmonic Ambisonic Order", &["1st Order (4 Channels WXYZ)", "2nd Order (9 Channels HOA)", "3rd Order (16 Channels High-Def)"], 1, "Ambisonic expansion order defining spatial directional resolution"))
        .with_param(DspParamSchema::choice("speaker_layout_configuration", "Loudspeaker Target Layout Array", &["5.1 Surround", "7.1 Surround", "7.1.4 Dolby Atmos Bed", "Quadraphonic 4.0", "Binaural Headphone Virtualizer"], 2, "Physical monitor speaker layout geometry for decoded soundstage"))
        .with_param(DspParamSchema::choice("spatial_decoding_matrix_mode", "Ambisonic Energy Vector Decoding Mode", &["In-Phase (Optimized Sweet Spot)", "Max-rE (Energy Vector Balanced)", "Basic Direct Decoder"], 1, "Optimizes directional decoding for either head movement sweet spot or energy balance"))
        .with_param(DspParamSchema::knob("distance_compensation_delay_ms", "Speaker Ring Distance Alignment Delay", 0.0, 20.0, 2.5, "ms", MacroRole::Space, "Time-alignment delay compensating for non-spherical physical speaker distances"))
        .with_param(DspParamSchema::knob("nearfield_distance_compensation_hz", "Nearfield Acoustic Bass Compensation", 20.0, 250.0, 60.0, "Hz", MacroRole::Tone, "Bass filter boosting radial wavefront proximity effect for close speakers"))
        .with_param(DspParamSchema::knob("subwoofer_lfe_crossover_hz", "LFE Bass Management Crossover", 40.0, 160.0, 80.0, "Hz", MacroRole::Tone, "Low-pass crossover frequency routing multi-channel bass energy into LFE subwoofer"))
    );
    descriptors.insert("BinauralSphericalHeadModel".to_string(), DspNodeDescriptor::new("BinauralSphericalHeadModel", "Continuous Spherical Head Shadow & Diffraction Simulator", DspNodeCategory::SpatialSurround, "Analytical spherical head acoustic model calculating frequency-dependent interaural time differences (ITD), head shadow diffraction, and pinna reflections")
        .with_param(DspParamSchema::knob("head_radius_meters", "Listener Cranial Radius Dimension", 0.06, 0.12, 0.0875, "m", MacroRole::Tone, "Physical ear-to-ear head radius setting ITD time delay scaling"))
        .with_param(DspParamSchema::knob("source_azimuth_deg", "Virtual Source Horizontal Azimuth Angle", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Horizontal angular position of sound source relative to head front axis"))
        .with_param(DspParamSchema::knob("source_elevation_deg", "Virtual Source Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation angle above or below horizontal listening plane"))
        .with_param(DspParamSchema::knob("source_distance_meters", "Virtual Sound Source Distance", 0.2, 50.0, 2.0, "m", MacroRole::Space, "Physical distance from center of head introducing spherical air attenuation"))
        .with_param(DspParamSchema::knob("woodworth_itd_weight", "Woodworth Analytical Time Delay Weight", 0.5, 1.5, 1.0, "wt", MacroRole::Tone, "Weighting coefficient applied to analytical Woodworth-Schlosberg ITD formula"))
        .with_param(DspParamSchema::knob("torso_shoulder_bounce_mix", "Shoulder & Torso Acoustic Reflection Level", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Early reflection delayed bounce from virtual listener shoulders and chest"))
    );
    descriptors.insert("BinauralDopplerMotionPanner".to_string(), DspNodeDescriptor::new("BinauralDopplerMotionPanner", "3D Trajectory Doppler-Shifted Binaural Panner", DspNodeCategory::SpatialSurround, "Continuous 3D spatial panner modeling dynamic acoustic Doppler frequency shifts, spherical air damping, and HRTF head shadow filtering along arbitrary spatial trajectories")
        .with_param(DspParamSchema::knob("azimuth_angle_deg", "Horizontal Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth angle of sound source relative to listener"))
        .with_param(DspParamSchema::knob("elevation_angle_deg", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation angle above or below horizontal listening plane"))
        .with_param(DspParamSchema::knob("source_distance_meters", "Sound Source Distance", 0.1, 100.0, 3.0, "m", MacroRole::Space, "Physical distance from listener introducing spherical 1/r amplitude attenuation"))
        .with_param(DspParamSchema::knob("trajectory_speed_m_s", "Source Trajectory Velocity", 0.0, 340.0, 15.0, "m/s", MacroRole::Character, "Source velocity inducing Doppler frequency shifts and interaural time rate changes"))
        .with_param(DspParamSchema::knob("atmospheric_absorption_db", "Atmospheric Ultrasonic Absorption", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Air moisture and molecular viscosity ultrasonic attenuation with distance"))
        .with_param(DspParamSchema::toggle("enable_doppler_pitch_shift", "Enable Real-Time Doppler Pitch Modulation", true, "Applies real-time sub-sample delay modulation for authentic Doppler effect"))
    );
    descriptors.insert("AmbisonicDiffractionEncoder".to_string(), DspNodeDescriptor::new("AmbisonicDiffractionEncoder", "Higher-Order Ambisonic Spherical Diffraction Encoder", DspNodeCategory::SpatialSurround, "4th-Order Ambisonic (HOA 25-channel) spherical harmonic spatializer modeling barrier diffraction acoustic shadows and early specular boundary bounces")
        .with_param(DspParamSchema::knob("source_azimuth_deg", "Spherical Coordinate Azimuth", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal azimuth angle on spherical ambisonic coordinate system"))
        .with_param(DspParamSchema::knob("source_elevation_deg", "Spherical Coordinate Elevation", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical elevation angle on spherical ambisonic coordinate system"))
        .with_param(DspParamSchema::choice("hoa_order_selection", "Spherical Harmonic HOA Order", &["1st Order (4 Channels)", "2nd Order (9 Channels)", "3rd Order (16 Channels)", "4th Order (25 Channels)"], 3, "Higher-order ambisonic spherical harmonic order resolution"))
        .with_param(DspParamSchema::knob("acoustic_shadow_diffraction", "Barrier Acoustic Shadow Damping", 0.0, 1.0, 0.6, "diff", MacroRole::Tone, "Obstacle boundary shadowing dampening high frequencies in acoustic shadow zone"))
        .with_param(DspParamSchema::knob("wall_boundary_reflection_db", "First-Order Specular Reflection Gain", -36.0, 0.0, -12.0, "dB", MacroRole::Space, "Level of first-order specular wall reflection encoded into spherical field"))
        .with_param(DspParamSchema::toggle("distance_nearfield_compensation", "Radial Nearfield Bass Compensation", true, "Applies radial distance compensation filters correcting low-frequency proximity bass boost"))
    );
    descriptors.insert("MasteringStereoWidthMaximizer".to_string(), DspNodeDescriptor::new("MasteringStereoWidthMaximizer", "Psychoacoustic Stereo Width Maximizer & Mono Bass Guard", DspNodeCategory::SpatialSurround, "Phase-coherent stereo field expander utilizing psychoacoustic Haas micro-delays, mid/side energy equalization, and steep elliptic low-frequency mono-summing")
        .with_param(DspParamSchema::knob("stereo_spread_width_percent", "Total Panoramic Stereo Width Expansion", 0.0, 250.0, 120.0, "%", MacroRole::Space, "Total stereo field width (0% = mono sum, 100% = original, 200% = ultra-wide)"))
        .with_param(DspParamSchema::knob("mono_bass_crossover_hz", "Mono Bass Summing Elliptic Crossover", 30.0, 250.0, 100.0, "Hz", MacroRole::Tone, "Elliptic filter crossover below which all low frequencies are collapsed to mono"))
        .with_param(DspParamSchema::knob("side_band_eq_tilt_db", "Side Channel High-Shelf Equalizer Tilt", -6.0, 6.0, 1.5, "dB", MacroRole::Tone, "High-shelf tilt boost applied exclusively to stereo side signal for sparkle"))
        .with_param(DspParamSchema::knob("haas_delay_spread_ms", "Haas Precedence Micro-Delay Time", 0.0, 25.0, 6.0, "ms", MacroRole::Space, "Microsecond interaural precedence delay enhancing perceived room dimension"))
        .with_param(DspParamSchema::knob("phase_correlation_safety_ceiling", "Minimum Phase Correlation Floor", 0.0, 1.0, 0.35, "corr", MacroRole::Punch, "Prevents stereo widening if inter-channel phase correlation drops below safety floor"))
        .with_param(DspParamSchema::toggle("audition_side_channel_solo", "Solo Isolated Stereo Side Channel (L-R)", false, "Solos isolated side signal (L-R) for monitoring stereo content and phase health"))
    );
    descriptors.insert("BinauralRoomImpulseSpatializer".to_string(), DspNodeDescriptor::new("BinauralRoomImpulseSpatializer", "Ray-Traced Binaural Room Impulse Response Spatializer", DspNodeCategory::SpatialSurround, "3D binaural acoustic renderer calculating directional HRTF filtering, early specular boundary reflections, and late diffuse reverberation energy")
        .with_param(DspParamSchema::knob("azimuth_angle_degrees", "Horizontal Azimuth Sound Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular position of acoustic sound source in 360-degree space"))
        .with_param(DspParamSchema::knob("elevation_angle_degrees", "Vertical Elevation Sound Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical angle of acoustic sound source above or below listener plane"))
        .with_param(DspParamSchema::knob("source_distance_meters", "Listener Radial Distance", 0.2, 30.0, 2.5, "m", MacroRole::Space, "Radial distance from listener head with inverse-square law law attenuation"))
        .with_param(DspParamSchema::knob("room_wall_absorption_coeff", "Acoustic Wall Boundary Absorption", 0.05, 0.95, 0.35, "abs", MacroRole::Tone, "Surface material absorption coefficient dampening specular early reflections"))
        .with_param(DspParamSchema::knob("diffuse_reverb_blend", "Late Diffuse Soundfield Reverb Blend", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Proportion of late isotropic diffuse reverberation added to direct path"))
        .with_param(DspParamSchema::toggle("enable_interaural_time_delay", "Model Head Shadow & ITD Delay", true, "Accurately renders interaural time difference (ITD) and head-shadow acoustic filtering"))
    );
    descriptors.insert("MidSidePsychoacousticImager".to_string(), DspNodeDescriptor::new("MidSidePsychoacousticImager", "Frequency-Dependent Mid/Side Psychoacoustic Imager", DspNodeCategory::SpatialSurround, "Three-band mid/side stereo panorama engine with dynamic cross-feed, high-frequency spatial widening, and elliptical subwoofer centering")
        .with_param(DspParamSchema::knob("low_band_mono_crossover_hz", "Elliptic Mono Bass Crossover Cutoff", 40.0, 300.0, 110.0, "Hz", MacroRole::Tone, "Crossover below which all side channel energy is collapsed to solid mono"))
        .with_param(DspParamSchema::knob("mid_band_stereo_width_percent", "Mid-Band Panorama Stereo Width", 0.0, 200.0, 100.0, "%", MacroRole::Space, "Stereo width ratio of body and vocal fundamental midrange"))
        .with_param(DspParamSchema::knob("high_band_stereo_width_percent", "High-Frequency Airy Stereo Spread", 0.0, 250.0, 135.0, "%", MacroRole::Space, "Expansive stereo spread applied to high frequencies for open acoustic width"))
        .with_param(DspParamSchema::knob("center_mid_channel_gain_db", "Center Mid (M) Channel Balance", -12.0, 6.0, 0.0, "dB", MacroRole::Punch, "Level adjustment for center phantom mono signal"))
        .with_param(DspParamSchema::knob("stereo_side_channel_gain_db", "Stereo Side (S) Channel Balance", -12.0, 6.0, 0.0, "dB", MacroRole::Space, "Level adjustment for isolated stereo side difference signal"))
        .with_param(DspParamSchema::toggle("enable_binaural_crossfeed_filter", "Binaural Interaural Crossfeed Filter", true, "Blends subtle low-pass cross-feed between channels preventing headphone listening fatigue"))
    );
    descriptors.insert("AmbisonicBinauralDecoderSpatializer".to_string(), DspNodeDescriptor::new("AmbisonicBinauralDecoderSpatializer", "3rd-Order Higher-Order Ambisonics (HOA) to Binaural HRIR Spherical Decoder", DspNodeCategory::SpatialSurround, "Converts 16-channel 3rd-order Ambisonic spherical soundfield representations into pristine binaural audio using minimum-phase diffuse-field equalized HRIRs")
        .with_param(DspParamSchema::knob("soundfield_yaw_rotation_deg", "Spherical Soundfield Yaw Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Rotates entire 3D soundfield horizontally around listener head"))
        .with_param(DspParamSchema::knob("soundfield_pitch_tilt_deg", "Spherical Soundfield Pitch Tilt", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Tilts soundfield upward or downward relative to horizontal ear plane"))
        .with_param(DspParamSchema::knob("soundfield_roll_rotation_deg", "Spherical Soundfield Roll Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Rolls soundfield about frontal listening axis"))
        .with_param(DspParamSchema::knob("binaural_head_radius_cm", "Acoustic Anthropomorphic Head Radius", 6.0, 12.0, 8.75, "cm", MacroRole::Space, "Radius of head model determining interaural time difference (ITD) delay"))
        .with_param(DspParamSchema::knob("diffuse_field_equalization_blend", "Diffuse-Field HRIR Equalization Blend", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Applies diffuse-field compensation curves to prevent timbral coloration"))
        .with_param(DspParamSchema::toggle("enable_nearfield_compensation", "Near-Field Wavefront Curvature Compensation", true, "Compensates for spherical wavefront curvature of close-proximity sound sources"))
    );
    descriptors.insert("DynamicStereoMicroDelayPanner".to_string(), DspNodeDescriptor::new("DynamicStereoMicroDelayPanner", "Haas-Effect Micro-Delay Spatializer & Psychoacoustic Precedence Panner", DspNodeCategory::SpatialSurround, "Sub-millisecond interaural time difference (ITD) delay line coupled with high-frequency head-shadow attenuation for hyper-realistic 3D acoustic localization")
        .with_param(DspParamSchema::knob("haas_delay_offset_ms", "Interaural Haas Time Delay Offset", 0.0, 35.0, 12.0, "ms", MacroRole::Space, "Microsecond delay skewing perceived sound source arrival angle via Haas effect"))
        .with_param(DspParamSchema::knob("interaural_intensity_diff_db", "Interaural Level Difference (ILD)", -18.0, 18.0, 0.0, "dB", MacroRole::Space, "Volume disparity between left and right ears complementing time-delay panner"))
        .with_param(DspParamSchema::knob("comb_filter_anti_phase_reduction", "Comb Filter Anti-Phase Attenuation", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Cancels destructive summing notches when mono-folded"))
        .with_param(DspParamSchema::knob("high_frequency_pinna_shadow_hz", "Pinna Ear Shadow Cutoff Filter", 2000.0, 16000.0, 6500.0, "Hz", MacroRole::Tone, "Damps high frequencies on contralateral side to emulate head shadowing"))
        .with_param(DspParamSchema::choice("precedence_effect_mode", "Psychoacoustic Spatial Delay Mode", &["Natural Haas", "Phase Inverted Left", "Phase Inverted Right", "Cross-Delayed Diffusion"], 0, "Selects phase and delay routing algorithm for stereophonic positioning"))
        .with_param(DspParamSchema::toggle("mono_sum_compatibility_guard", "Mono-Sum Phase Cancellation Guard", true, "Prevents complete signal cancellation when stereo audio is collapsed to mono playback"))
    );
    descriptors.insert("BinauralDistanceRoomRenderer".to_string(), DspNodeDescriptor::new("BinauralDistanceRoomRenderer", "Directivity & Parallax-Aware Binaural Acoustic Distance Modeler", DspNodeCategory::SpatialSurround, "Physical acoustic propagation model calculating air humidity high-frequency absorption, parallax ITD/ILD angles, and ground boundary reflection geometry for natural distance perception")
        .with_param(DspParamSchema::knob("sound_source_distance_meters", "Acoustic Source Distance to Listener", 0.2, 50.0, 3.5, "m", MacroRole::Space, "Physical distance between virtual sound source and binaural listener head"))
        .with_param(DspParamSchema::knob("atmospheric_absorption_hf_damping", "Air Humidity Absorption High Damping", 0.1, 2.0, 1.0, "coef", MacroRole::Tone, "Frequency-dependent molecular air damping attenuating highs over distance"))
        .with_param(DspParamSchema::knob("binaural_parallax_azimuth_deg", "Source Azimuth Angle Positioning", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal angular placement around listener skull"))
        .with_param(DspParamSchema::knob("initial_delay_gap_ms", "Direct-to-Early Reflection Delay Offset", 0.0, 80.0, 12.0, "ms", MacroRole::Space, "Time gap between direct arrival and first acoustic boundary reflection"))
        .with_param(DspParamSchema::knob("wall_absorption_coefficient", "Boundary Boundary Absorption Material", 0.05, 0.95, 0.35, "abs", MacroRole::Space, "Wall and floor reflection absorption emulating wood, glass, or drapery"))
        .with_param(DspParamSchema::toggle("enable_early_ground_reflection", "Specular Floor Reflection Simulation", true, "Renders ground reflection phase comb filter that humans use to judge elevation"))
    );
    descriptors.insert("DolbyAtmos714BedEncoder".to_string(), DspNodeDescriptor::new("DolbyAtmos714BedEncoder", "7.1.4 Immersive Audio Multi-Speaker Height Bus Matrix Panner", DspNodeCategory::SpatialSurround, "Surround and overhead immersive audio encoder routing audio across standard 7.1.4 loudspeaker configurations (Left, Center, Right, Sub, Sides, Rears, and 4 Top Heights)")
        .with_param(DspParamSchema::knob("bed_azimuth_panning_deg", "Horizontal Surround Azimuth Pan", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Azimuth angle panning sound continuously around surround horizon"))
        .with_param(DspParamSchema::knob("elevation_height_spread_deg", "Overhead Height Elevation Angle", 0.0, 90.0, 35.0, "deg", MacroRole::Space, "Elevates soundfield upward toward Top Front and Top Rear ceiling speakers"))
        .with_param(DspParamSchema::knob("lfe_subwoofer_crossover_send_db", "LFE Subwoofer Dedicated Send Gain", -60.0, 6.0, -12.0, "dB", MacroRole::Punch, "Low-frequency effects channel feed for subwoofer impact"))
        .with_param(DspParamSchema::knob("center_channel_divergence_ratio", "Center Speaker Phantom Divergence", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Spreads center dialogue energy outward into left and right front speakers"))
        .with_param(DspParamSchema::knob("surround_decorrelation_width", "Decorrelation Diffuse Surround Width", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Phase-decorrelates surround channels to envelop listener without localization hot-spots"))
        .with_param(DspParamSchema::toggle("enable_discrete_height_isolation", "Discrete Height Channel Isolation", false, "Strictly isolates audio to 4 overhead height speakers, silencing ground bed"))
    );
    descriptors.insert("AtmosphericDensityAbsorption".to_string(), DspNodeDescriptor::new("AtmosphericDensityAbsorption", "Barometric Pressure & Humidity Acoustic Wavefront Absorption", DspNodeCategory::SpatialSurround, "Precision ISO 9613-1 acoustic air absorption filter calculating molecular relaxation loss of nitrogen and oxygen across altitude, temperature, and relative humidity")
        .with_param(DspParamSchema::knob("altitude_elevation_meters", "Geographic Altitude Elevation", -500.0, 9000.0, 0.0, "m", MacroRole::Tone, "Altitude above sea level altering barometric air density and speed of sound"))
        .with_param(DspParamSchema::knob("ambient_temperature_celsius", "Ambient Atmospheric Temperature", -40.0, 50.0, 20.0, "deg C", MacroRole::Tone, "Air temperature shifting molecular relaxation frequencies of diatomic gas molecules"))
        .with_param(DspParamSchema::knob("relative_air_humidity_pct", "Relative Atmospheric Humidity", 5.0, 100.0, 50.0, "%", MacroRole::Space, "Moisture percentage governing water vapor absorption peaks in high treble"))
        .with_param(DspParamSchema::knob("barometric_static_pressure_kpa", "Static Barometric Pressure", 30.0, 110.0, 101.3, "kPa", MacroRole::Character, "Local atmospheric static pressure governing acoustic impedance of air"))
        .with_param(DspParamSchema::knob("distance_propagation_meters", "Wavefront Distance Traveled", 1.0, 500.0, 25.0, "m", MacroRole::Space, "Physical propagation distance over which frequency-dependent absorption accumulates"))
        .with_param(DspParamSchema::toggle("enable_molecular_nitrogen_relaxation", "Diatomic Nitrogen Molecular Relaxation", true, "Calculates rotational relaxation resonance of nitrogen molecules around 1 kHz"))
    );
    descriptors.insert("AcousticCloakingTransformationPanner".to_string(), DspNodeDescriptor::new("AcousticCloakingTransformationPanner", "Transformation Optics Acoustic Cloaking & Wavefront Curvature Panner", DspNodeCategory::SpatialSurround, "Coordinate transformation mapping sound waves smoothly around a virtual cylinder/sphere without acoustic shadow, backscattering, or wave obstruction")
        .with_param(DspParamSchema::knob("cloaking_shell_inner_radius_m", "Acoustic Cloak Inner Core Radius", 0.1, 5.0, 0.75, "m", MacroRole::Tone, "Radius of silent shielded zone inside the transformation cloaking shell"))
        .with_param(DspParamSchema::knob("cloaking_shell_outer_radius_m", "Acoustic Cloak Outer Shell Radius", 0.2, 10.0, 1.8, "m", MacroRole::Space, "Outer boundary where acoustic metamaterial transformation transitions to ambient air"))
        .with_param(DspParamSchema::knob("anisotropic_density_radial_ratio", "Radial vs Azimuthal Anisotropy", 0.1, 10.0, 2.2, "ratio", MacroRole::Character, "Directional mass density anisotropy guiding wavefronts around obstacles"))
        .with_param(DspParamSchema::knob("wavefront_deflection_angle_deg", "Curvature Deflection Steering Angle", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Spatial panning angle directing wavefront curvature across stereo speaker array"))
        .with_param(DspParamSchema::knob("shadowless_transparency_factor", "Shadowless Acoustic Transparency", 0.0, 1.0, 0.95, "%", MacroRole::Tone, "Elimination of downstream acoustic shadow preserving crisp high frequencies"))
        .with_param(DspParamSchema::toggle("enable_ideal_transformation_coordinate_map", "Exact Non-Euclidean Metric Tensor Mapping", true, "Computes mathematically rigorous non-Euclidean transformation tensor coordinates"))
    );
    descriptors.insert("DynamicLobeBeamformerSpatializer".to_string(), DspNodeDescriptor::new("DynamicLobeBeamformerSpatializer", "Spherical Harmonics Adaptive Beamforming Spatializer", DspNodeCategory::SpatialSurround, "Dynamic acoustic steering array synthesizing sharp directional lobes, side-lobe cancellation, and 3D spatial isolation")
        .with_param(DspParamSchema::knob("beam_azimuth_angle_deg", "Beam Steering Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular trajectory direction of synthesized acoustic beam lobe"))
        .with_param(DspParamSchema::knob("beam_elevation_angle_deg", "Beam Steering Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation angle tilt of acoustic directivity beam lobe"))
        .with_param(DspParamSchema::knob("lobe_focus_directivity_q", "Spatial Directivity Main Lobe Q", 1.0, 32.0, 6.0, "Q", MacroRole::Punch, "Aperture sharpness and angular concentration of primary acoustic lobe"))
        .with_param(DspParamSchema::knob("side_lobe_suppression_db", "Side Lobe Attenuation Rejection", 0.0, 60.0, 24.0, "dB", MacroRole::Tone, "Damping attenuation applied to Chebychev array side-lobes"))
        .with_param(DspParamSchema::knob("array_radius_aperture_cm", "Virtual Transducer Array Aperture", 5.0, 200.0, 35.0, "cm", MacroRole::Space, "Geometric physical dimension of virtual transducer array aperture"))
        .with_param(DspParamSchema::toggle("enable_superdirective_null_steering", "Adaptive Null Interference Steering", true, "Places sharp spatial zeroes directly in directions of masking acoustic bleed"))
    );
    descriptors.insert("PhasedArrayUltrasoundTransducer".to_string(), DspNodeDescriptor::new("PhasedArrayUltrasoundTransducer", "Self-Demodulating Parametric Ultrasound Audio Spotlight", DspNodeCategory::SpatialSurround, "Acoustic audio spotlight simulation leveraging finite-amplitude ultrasonic carrier distortion to project highly localized sound beams in air")
        .with_param(DspParamSchema::knob("ultrasonic_carrier_freq_khz", "Parametric Ultrasonic Carrier Freq", 30.0, 80.0, 40.0, "kHz", MacroRole::Tone, "Inaudible primary carrier frequency transmitted into air column by array"))
        .with_param(DspParamSchema::knob("carrier_modulation_depth", "AM Carrier Modulation Index", 0.1, 1.0, 0.85, "%", MacroRole::Punch, "Amplitude modulation depth encoding audible signal onto ultrasonic carrier"))
        .with_param(DspParamSchema::knob("acoustic_nonlinearity_param_b_a", "Air Non-Linear Acoustic Factor (B/A)", 0.1, 2.0, 0.7, "B/A", MacroRole::Character, "Nonlinear thermodynamic air parameter driving self-demodulation efficiency"))
        .with_param(DspParamSchema::knob("projection_spotlight_throw_m", "Parametric Beam Throw Distance", 0.5, 30.0, 5.0, "m", MacroRole::Space, "Target distance along acoustic beam axis where demodulated sound focuses"))
        .with_param(DspParamSchema::knob("spotlight_spot_radius_cm", "Acoustic Focus Spot Radius", 5.0, 100.0, 25.0, "cm", MacroRole::Space, "Narrow spatial radius outside of which acoustic sound level drops abruptly"))
        .with_param(DspParamSchema::toggle("enable_berktay_demodulation_filter", "Berktay Envelope Self-Demodulation Model", true, "Applies Berktay second derivative envelope demodulation transform"))
    );
    descriptors.insert("BinauralNearFieldCrosstalkCanceller".to_string(), DspNodeDescriptor::new("BinauralNearFieldCrosstalkCanceller", "Transaural Acoustic Crosstalk Cancellation & Head Model", DspNodeCategory::SpatialSurround, "Optimal recursive filter network cancelling contralateral speaker acoustic crosstalk to deliver 3D binaural sound over stereo loudspeakers")
        .with_param(DspParamSchema::knob("speaker_spread_angle_deg", "Stereo Monitor Spacing Angle", 15.0, 90.0, 60.0, "deg", MacroRole::Space, "Physical angle subtended by stereo loudspeaker monitors relative to listener"))
        .with_param(DspParamSchema::knob("listener_distance_meters", "Listener-to-Speaker Distance", 0.5, 5.0, 1.8, "m", MacroRole::Space, "Physical listening distance setting acoustic propagation delay and wavefront curvature"))
        .with_param(DspParamSchema::knob("interaural_distance_head_cm", "Interaural Head Breadth Spacing", 12.0, 25.0, 17.5, "cm", MacroRole::Character, "Head width dimension setting interaural time difference (ITD) cancel delay"))
        .with_param(DspParamSchema::knob("crosstalk_cancellation_depth_db", "Contralateral Crosstalk Attenuation", 6.0, 40.0, 22.0, "dB", MacroRole::Punch, "Depth of opposite-ear acoustic cancellation: higher depth expands 3D stage"))
        .with_param(DspParamSchema::knob("spectral_pinna_coloration_trim", "Pinna Notch Coloration Compensation", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Equalizes boost and notch coloration introduced by transaural inversion filters"))
        .with_param(DspParamSchema::toggle("enable_transaural_sweet_spot_boost", "Sweet-Spot Widening High-Frequency Guard", true, "Broadens sweet spot allowing head movement without catastrophic 3D collapse"))
    );
    descriptors.insert("SphericalHarmonicSoundfieldRotator".to_string(), DspNodeDescriptor::new("SphericalHarmonicSoundfieldRotator", "Higher-Order Ambisonic Spherical Soundfield Rotator", DspNodeCategory::SpatialSurround, "Full 3D rotation and tilting matrix for Higher-Order Ambisonic (HOA) soundfields using spherical harmonic Wigner-D transformation matrices for yaw, pitch, and roll")
        .with_param(DspParamSchema::knob("yaw_azimuth_rotation_deg", "Soundfield Yaw Azimuth Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal azimuth panning rotation around vertical Z axis"))
        .with_param(DspParamSchema::knob("pitch_elevation_tilt_deg", "Soundfield Pitch Elevation Tilt", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical elevation tilt rotation around lateral Y axis"))
        .with_param(DspParamSchema::knob("roll_barrel_rotation_deg", "Soundfield Roll Barrel Rotation", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Barrel roll axial rotation around front-back X axis"))
        .with_param(DspParamSchema::knob("spherical_order_processing_limit", "Ambisonic Rotation Order Depth", 1.0, 5.0, 3.0, "order", MacroRole::Tone, "Spherical harmonic decomposition order for transformation matrix"))
        .with_param(DspParamSchema::knob("doppler_motion_smearing_ms", "Dynamic Angular Slew Interpolation", 0.0, 100.0, 10.0, "ms", MacroRole::Character, "Angular smoothing time constant preventing zipper noise during fast rotation"))
        .with_param(DspParamSchema::toggle("enable_wigner_matrix_normalization", "Spherical Energy Conservation Norm", true, "Enforces strict total acoustic energy conservation across spherical rotation"))
    );
    descriptors.insert("HilbertTransformStereoPanner".to_string(), DspNodeDescriptor::new("HilbertTransformStereoPanner", "Hilbert Transform Analytic Stereo Spatializer", DspNodeCategory::SpatialSurround, "Generates analytic 90-degree quadrature signals via Parks-McClellan Hilbert FIR transform, enabling frequency-invariant phase rotation and binaural stereo width expansion")
        .with_param(DspParamSchema::knob("quadrature_phase_angle_deg", "Analytic Quadrature Phase Angle", -180.0, 180.0, 90.0, "deg", MacroRole::Space, "Phase rotation angle between analytic in-phase and quadrature paths"))
        .with_param(DspParamSchema::knob("stereo_expansion_width_pct", "Hilbert Decorrelation Stereo Width", 0.0, 200.0, 120.0, "%", MacroRole::Space, "Stereo field width scaling based on Hilbert decorrelation"))
        .with_param(DspParamSchema::knob("hilbert_fir_taps_count", "Hilbert FIR Filter Tap Order", 16.0, 256.0, 64.0, "taps", MacroRole::Tone, "FIR filter order determining low-frequency phase accuracy"))
        .with_param(DspParamSchema::knob("frequency_split_low_shelf_hz", "Mono Low-End Anchor Crossover", 50.0, 500.0, 180.0, "Hz", MacroRole::Punch, "Low-frequency boundary preserving mono bass center anchor"))
        .with_param(DspParamSchema::knob("correlation_guard_attenuation_db", "Anti-Phase Correlation Limiter", -12.0, 0.0, -3.0, "dB", MacroRole::Character, "Protects mono compatibility by ducking out-of-phase side energy"))
        .with_param(DspParamSchema::toggle("enable_binaural_crossfeed_filter", "Binaural Head-Shadow Crossfeed", true, "Enables subtle head-shadow HRTF crossfeed between left and right channels"))
    );
    descriptors.insert("BinauralPinnaReflectionEngine".to_string(), DspNodeDescriptor::new("BinauralPinnaReflectionEngine", "Binaural Pinna & Torso Acoustic Reflection Spatializer", DspNodeCategory::SpatialSurround, "Simulates external ear pinna notches, concha cavity resonances, and shoulder/torso reflections to provide ultra-realistic vertical elevation and front/back localization")
        .with_param(DspParamSchema::knob("sound_elevation_angle_deg", "Vertical Pinna Elevation Angle", -45.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical sound source elevation angle modulating pinna spectral notches"))
        .with_param(DspParamSchema::knob("sound_azimuth_angle_deg", "Horizontal Head Azimuth Angle", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal sound source azimuth angle relative to listener head"))
        .with_param(DspParamSchema::knob("torso_reflection_delay_ms", "Shoulder & Clavicle Reflection Delay", 0.5, 4.0, 1.2, "ms", MacroRole::Space, "Acoustic reflection time delay off chest and torso boundaries"))
        .with_param(DspParamSchema::knob("concha_cavity_notch_depth_db", "Concha Resonance Elevation Notch Depth", 0.0, 24.0, 12.0, "dB", MacroRole::Tone, "Depth of characteristic 8 kHz pinna elevation spectral notch"))
        .with_param(DspParamSchema::knob("head_shadow_damping_loss_db", "Contralateral Acoustic Head Shadow Loss", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Contralateral high-frequency shadow attenuation caused by head mass"))
        .with_param(DspParamSchema::toggle("enable_near_field_interaural_level_boost", "Proximity Low-Frequency Bass Boost", true, "Applies low-frequency interaural level boost for close-proximity sound sources"))
    );
    descriptors.insert("PsychoacousticStereoImager".to_string(), DspNodeDescriptor::new("PsychoacousticStereoImager", "Multi-Band Psychoacoustic Stereo Width Imager", DspNodeCategory::SpatialSurround, "Expands perceived soundstage width across low, mid, and high bands using Haas micro-delay offsets, center cancellation, and mono bass summing with automated correlation protection")
        .with_param(DspParamSchema::knob("stereo_spread_width_pct", "Frequency-Dependent Stereo Width Scale", 0.0, 250.0, 135.0, "%", MacroRole::Space, "Overall stereo width expansion scaling across active frequency bands"))
        .with_param(DspParamSchema::knob("haas_delay_offset_ms", "Interaural Haas Micro-Delay Depth", 0.0, 20.0, 8.5, "ms", MacroRole::Space, "Precedence Haas effect interaural micro-delay generating spatial depth"))
        .with_param(DspParamSchema::knob("low_band_mono_crossover_hz", "Sub-Bass Mono Collapse Crossover", 60.0, 400.0, 160.0, "Hz", MacroRole::Punch, "Collapses frequencies below this point to pure mono for punch and vinyl cutting"))
        .with_param(DspParamSchema::knob("high_shelf_air_widening_db", "Ultra-High Air Side Channel Boost", -6.0, 6.0, 2.0, "dB", MacroRole::Tone, "High-frequency side-channel boost adding ethereal top-end air"))
        .with_param(DspParamSchema::knob("phase_correlation_safety_ceiling", "Dynamic Phase Correlation Floor Guard", 0.0, 1.0, 0.75, "corr", MacroRole::Character, "Dynamic correlation limiter reducing width if mono cancellation is detected"))
        .with_param(DspParamSchema::toggle("enable_mid_side_listen_audition", "Audition Isolated Side Channel Audio", false, "Auditions isolated side channel signal to verify stereo spatial content"))
    );
    descriptors.insert("HigherOrderAmbisonicMicrophoneEncoder".to_string(), DspNodeDescriptor::new("HigherOrderAmbisonicMicrophoneEncoder", "5th-Order Ambisonic Spherical Microphone Encoder", DspNodeCategory::SpatialSurround, "Translates multichannel tetrahedral and spherical capsule arrays into 5th-order 36-channel Higher Order Ambisonics (HOA) with polar pattern compensation and near-field distance filtering")
        .with_param(DspParamSchema::knob("capsule_array_radius_mm", "Microphone Sphere Array Radius", 10.0, 150.0, 32.0, "mm", MacroRole::Space, "Physical radius of the spherical microphone capsule array"))
        .with_param(DspParamSchema::knob("ambisonic_hoa_order", "Ambisonic Truncation Order", 1.0, 5.0, 3.0, "order", MacroRole::Tone, "Higher Order Ambisonic spatial resolution order"))
        .with_param(DspParamSchema::knob("radial_nearfield_distance_m", "Near-Field Proximity Distance Filter", 0.1, 10.0, 1.5, "m", MacroRole::Space, "Distance parameter for spherical wave radial near-field filtering"))
        .with_param(DspParamSchema::knob("capsule_polar_directivity_ratio", "Capsule Polar Pattern (Cardioid to Omni)", 0.0, 1.0, 0.5, "dir", MacroRole::Character, "Raw microphone capsule transducer directivity response"))
        .with_param(DspParamSchema::knob("diffuse_field_equalization_db", "Diffuse Field High-Frequency Boost", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Diffuse-field high-frequency loss equalization compensation"))
        .with_param(DspParamSchema::toggle("enable_strict_phase_alignment", "Capsule Delay Skew Correction", true, "Compensates for inter-capsule time-of-flight phase delays"))
    );
    descriptors.insert("EllipticalStereoPhaseCorrelator".to_string(), DspNodeDescriptor::new("EllipticalStereoPhaseCorrelator", "Elliptical Stereo Equalizer & Phase Correlator", DspNodeCategory::SpatialSurround, "Mastering spatial processor combining an elliptical equalizer to sum bass to mono below a target crossover, mid/side stereo expansion, and an anti-phase correlation limiter")
        .with_param(DspParamSchema::knob("mono_sum_crossover_hz", "Elliptical Bass Mono Crossover Cutoff", 40.0, 400.0, 140.0, "Hz", MacroRole::Punch, "Frequency below which all stereo side channel energy is summed to mono"))
        .with_param(DspParamSchema::knob("stereo_side_channel_gain_db", "Side Channel Spatial Gain Boost", -12.0, 12.0, 1.5, "dB", MacroRole::Space, "Gain adjustment applied to side (stereo difference) channel"))
        .with_param(DspParamSchema::knob("anti_phase_clamp_threshold", "Phase Correlation Limit Threshold", -1.0, 0.5, 0.1, "corr", MacroRole::Character, "Correlation threshold triggering automatic side-channel ducking"))
        .with_param(DspParamSchema::knob("high_shelf_widening_frequency_hz", "High-Frequency Spatial Air Crossover", 2000.0, 16000.0, 8000.0, "Hz", MacroRole::Tone, "High-shelf frequency for selective stereo air expansion"))
        .with_param(DspParamSchema::knob("mid_channel_tilt_balance_db", "Mid Channel Spectral Tilt EQ", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "Linear tilt equalization on center channel balancing warm vs bright"))
        .with_param(DspParamSchema::toggle("enable_correlation_safety_attenuation", "Mono Summing Safety Gain Attenuation", true, "Attenuates mix slightly during severe out-of-phase events"))
    );
    descriptors.insert("VolumetricRaytracedRoomAcoustics".to_string(), DspNodeDescriptor::new("VolumetricRaytracedRoomAcoustics", "Volumetric Ray-Traced 3D Room Acoustics Engine", DspNodeCategory::SpatialSurround, "Real-time acoustic simulation employing stochastic Monte Carlo ray tracing to compute specular sound reflections, diffraction over obstacle edges, and absorption per room wall material")
        .with_param(DspParamSchema::knob("room_length_meters", "Room Acoustic Enclosure Length", 3.0, 80.0, 18.0, "m", MacroRole::Space, "Physical length dimension of the simulated rectangular acoustic hall"))
        .with_param(DspParamSchema::knob("room_width_meters", "Room Acoustic Enclosure Width", 3.0, 60.0, 12.0, "m", MacroRole::Space, "Physical width dimension of the simulated rectangular acoustic hall"))
        .with_param(DspParamSchema::knob("wall_absorption_coefficient_pct", "Boundary Wall Acoustic Absorption", 5.0, 95.0, 25.0, "%", MacroRole::Tone, "Sound absorption percentage of wall, ceiling, and floor boundaries"))
        .with_param(DspParamSchema::knob("air_humidity_absorption_db", "High Frequency Air Moisture Damping", 0.0, 12.0, 3.2, "dB", MacroRole::Tone, "Frequency-dependent attenuation caused by air humidity and temperature"))
        .with_param(DspParamSchema::knob("stochastic_ray_trace_count", "Simulated Monte Carlo Rays per Frame", 64.0, 2048.0, 512.0, "rays", MacroRole::Punch, "Number of acoustic ray paths calculated per audio processing block"))
        .with_param(DspParamSchema::toggle("enable_specular_edge_diffraction", "Obstacle Edge Diffraction Scattering", true, "Computes secondary sound scattering around pillars and room obstacles"))
    );
    descriptors.insert("AmbisonicDistanceCuesBinauralizer".to_string(), DspNodeDescriptor::new("AmbisonicDistanceCuesBinauralizer", "Ambisonic Distance & Air Absorption Binauralizer", DspNodeCategory::SpatialSurround, "Converts Higher Order Ambisonic soundfields into binaural headphone audio with ISO 9613-1 atmospheric absorption, distance parallax, and dynamic head tracking")
        .with_param(DspParamSchema::knob("sound_source_distance_meters", "Acoustic Virtual Source Distance", 0.2, 50.0, 2.5, "m", MacroRole::Space, "Simulated distance in meters affecting parallax and high-frequency air loss"))
        .with_param(DspParamSchema::knob("atmospheric_relative_humidity_pct", "Air Relative Humidity", 10.0, 95.0, 50.0, "%", MacroRole::Tone, "Governs frequency-dependent acoustic air absorption curves"))
        .with_param(DspParamSchema::knob("air_temperature_celsius", "Atmospheric Air Temperature", -10.0, 45.0, 20.0, "degC", MacroRole::Character, "Alters speed of sound and molecular vibrational relaxation frequencies"))
        .with_param(DspParamSchema::knob("head_rotation_yaw_degrees", "Listener Head Yaw Rotation Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal head orientation angle for real-time spatial positioning"))
        .with_param(DspParamSchema::knob("binaural_pinna_diffraction_amount", "Pinna Notch & Flange Diffraction Depth", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Scale of personalized HRTF pinna spectral notch filtering"))
        .with_param(DspParamSchema::toggle("enable_ground_reflection_specular", "Specular Floor Boundary Bounce", true, "Generates ground reflection comb filtering matching room acoustic geometry"))
    );
    descriptors.insert("BinauralConchaReflectionSpatializer".to_string(), DspNodeDescriptor::new("BinauralConchaReflectionSpatializer", "Concha & Ear Canal Anatomical Spatializer", DspNodeCategory::SpatialSurround, "High-resolution anatomical pinna spatializer modeling concha cavity acoustic resonances, ear canal quarter-wave standing waves, and head shadow diffraction")
        .with_param(DspParamSchema::knob("concha_cavity_depth_mm", "Anatomical Concha Cavity Depth", 5.0, 25.0, 14.0, "mm", MacroRole::Tone, "Physical depth of concha bowl governing vertical elevation cues"))
        .with_param(DspParamSchema::knob("sound_elevation_angle_degrees", "Source Elevation Trajectory Angle", -60.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical angle of incoming sound source relative to horizon"))
        .with_param(DspParamSchema::knob("sound_azimuth_angle_degrees", "Source Azimuth Horizontal Angle", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal angle of incoming sound source around the head"))
        .with_param(DspParamSchema::knob("ear_canal_resonant_frequency_hz", "Ear Canal Resonance Peak Tuning", 2000.0, 5000.0, 3200.0, "Hz", MacroRole::Character, "Quarter-wave ear canal resonant amplification peak"))
        .with_param(DspParamSchema::knob("interaural_time_delay_microsec", "Head Shadow ITD Delay Multiplier", 0.0, 900.0, 650.0, "us", MacroRole::Punch, "Interaural time difference scale between left and right ears"))
        .with_param(DspParamSchema::toggle("enable_interaural_spectral_diffraction", "Torso & Shoulder Acoustic Diffraction", true, "Models anatomical shoulder reflections providing tactile elevation realism"))
    );
    descriptors.insert("RelativisticDopplerLorentzTransform".to_string(), DspNodeDescriptor::new("RelativisticDopplerLorentzTransform", "Relativistic Doppler & Lorentz Contraction Spatializer", DspNodeCategory::SpatialSurround, "Super-sonic and relativistic acoustic propagation simulator applying kinematic Doppler shifts, Lorentz time dilation, and dynamic head-shadow diffraction for supersonic sound sources")
        .with_param(DspParamSchema::knob("source_velocity_mach", "Source Relative Velocity", 0.0, 5.0, 0.85, "Mach", MacroRole::Tone, "Governs relativistic velocity ratio relative to medium sound speed"))
        .with_param(DspParamSchema::knob("lorentz_dilation_factor", "Lorentz Acoustic Contraction Factor", 0.5, 1.0, 0.98, "gamma", MacroRole::Space, "Applies relativistic space-time contraction to acoustic wave trains"))
        .with_param(DspParamSchema::knob("sonic_boom_shockwave_steepness", "Mach Cone Shockwave Steepness", 0.1, 10.0, 3.5, "slope", MacroRole::Punch, "Steepness of acoustic pressure jump across supersonic Mach cone discontinuity"))
        .with_param(DspParamSchema::knob("binaural_observer_azimuth_deg", "Observer Azimuth Trajectory Angle", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Angular direction of moving supersonic sound source relative to listener"))
        .with_param(DspParamSchema::knob("head_shadow_dispersion_damping", "Atmospheric Hypersonic Dispersion", 0.01, 1.0, 0.35, "loss", MacroRole::Character, "High-frequency atmospheric damping and shockwave air absorption"))
        .with_param(DspParamSchema::toggle("enable_mach_cone_transonic_singularity", "Transonic Shock Singularity Modeling", true, "Simulates extreme pressure accumulation and sonic boom impulse at Mach 1.0 boundary"))
    );
    descriptors.insert("BinauralPinnaDiffractionSpatializer".to_string(), DspNodeDescriptor::new("BinauralPinnaDiffractionSpatializer", "3D Head-Related Anatomical Pinna Diffraction Spatializer", DspNodeCategory::SpatialSurround, "Continuous HRTF model simulating outer ear pinna micro-reflections, head shadow ITD (Interaural Time Difference), and ILD (Interaural Level Difference) in 3D sphere")
        .with_param(DspParamSchema::knob("azimuth_horizontal_angle_deg", "3D Azimuth Direction Angle", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal angular position of sound source (-180 to +180)"))
        .with_param(DspParamSchema::knob("elevation_vertical_angle_deg", "3D Elevation Height Angle", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical angle above or below listener horizon"))
        .with_param(DspParamSchema::knob("head_radius_measurement_cm", "Anatomical Head Radius Size", 6.0, 14.0, 8.75, "cm", MacroRole::Character, "Spherical head radius determining interaural acoustic propagation delay"))
        .with_param(DspParamSchema::knob("pinna_concha_notch_frequency_khz", "Upper Pinna Spectral Notch", 4.0, 12.0, 7.2, "kHz", MacroRole::Tone, "Elevation-dependent pinna notch frequency providing vertical psychoacoustic cues"))
        .with_param(DspParamSchema::knob("distance_attenuation_meters", "Virtual Sound Source Distance", 0.1, 50.0, 2.0, "m", MacroRole::Punch, "Inverse square law acoustic distance attenuation and air absorption"))
        .with_param(DspParamSchema::toggle("enable_torso_shoulder_bounce_reflection", "Torso & Shoulder Reflection Modeling", true, "Calculates anatomical early reflections off upper chest for natural realism"))
    );
    descriptors.insert("BinauralNearFieldAcousticScattering".to_string(), DspNodeDescriptor::new("BinauralNearFieldAcousticScattering", "Near-Field Head & Torso HRTF Acoustic Scatterer", DspNodeCategory::SpatialSurround, "Continuous 3D near-field acoustic propagation model computing spherical wave curvature, distance parallax low-boost, torso scattering reflections, and pinna diffraction")
        .with_param(DspParamSchema::knob("radial_source_distance_cm", "Radial Distance to Listener Head", 5.0, 200.0, 35.0, "cm", MacroRole::Space, "Proximity distance inducing acoustic wave curvature and low-frequency bass boost"))
        .with_param(DspParamSchema::knob("parallax_azimuth_angle_deg", "Horizontal Parallax Azimuth Angle", -180.0, 180.0, 25.0, "deg", MacroRole::Space, "Azimuth angle with close-range parallax correction between ears"))
        .with_param(DspParamSchema::knob("torso_acoustic_scattering_radius", "Anatomical Torso Width Radius", 15.0, 45.0, 26.0, "cm", MacroRole::Character, "Simulates early chest and shoulder sound scattering and reflection comb notches"))
        .with_param(DspParamSchema::knob("interaural_low_frequency_boost_db", "Near-Field Bass Curvature Boost", 0.0, 18.0, 5.5, "dB", MacroRole::Punch, "Proximity effect boost on ipsilateral ear caused by spherical acoustic wave curvature"))
        .with_param(DspParamSchema::knob("head_boundary_absorption_alpha", "Cranial Acoustic Absorption Coefficient", 0.01, 0.5, 0.12, "alpha", MacroRole::Tone, "High-frequency acoustic energy absorbed by facial hair and scalp tissues"))
        .with_param(DspParamSchema::toggle("enable_pinna_micro_multipath", "Pinna Micro-Multipath Reflections", true, "Renders individual micro-cavity pinna reflections for realistic height elevation"))
    );
    descriptors.insert("AmbisonicBFormatDopplerSpatialPanner".to_string(), DspNodeDescriptor::new("AmbisonicBFormatDopplerSpatialPanner", "Higher-Order Ambisonic (HOA) Spherical Kinematic Panner", DspNodeCategory::SpatialSurround, "Spherical 3D coordinate spatializer encoding sound into Higher-Order Ambisonics (HOA) with dynamic kinematic trajectory Doppler shift, distance gain falloff, and air dispersion")
        .with_param(DspParamSchema::knob("source_trajectory_radius_meters", "Spatial Distance Radius to Center", 0.2, 50.0, 3.5, "m", MacroRole::Punch, "Distance from listener origin governing inverse-square attenuation"))
        .with_param(DspParamSchema::knob("spherical_azimuth_bearing_deg", "Spherical Horizontal Azimuth Bearing", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "360-degree horizontal rotation angle around virtual listening position"))
        .with_param(DspParamSchema::knob("spherical_elevation_tilt_deg", "Spherical Vertical Elevation Angle", -90.0, 90.0, 20.0, "deg", MacroRole::Space, "Vertical tilt angle positioning sound source above or below listener horizon"))
        .with_param(DspParamSchema::knob("kinematic_doppler_velocity_mps", "Radial Trajectory Velocity (Doppler)", -150.0, 150.0, 0.0, "m/s", MacroRole::Tone, "Radial speed toward or away from listener producing physical pitch Doppler shift"))
        .with_param(DspParamSchema::knob("air_damping_high_frequency_loss", "Atmospheric Air Damping Coefficient", 0.001, 0.2, 0.035, "loss", MacroRole::Character, "Simulates high-frequency humidity absorption over physical distance"))
        .with_param(DspParamSchema::toggle("enable_third_order_hoa_coefficients", "Higher-Order Ambisonic (3rd Order) Output", true, "Calculates 16-channel HOA spherical harmonic coefficients for extreme spatial resolution"))
    );
    descriptors.insert("SphericalHarmonicEigenmikeBeamformer".to_string(), DspNodeDescriptor::new("SphericalHarmonicEigenmikeBeamformer", "Spherical Microphone Array Beamformer", DspNodeCategory::SpatialSurround, "Rigid sphere acoustic beamforming processor decomposing soundfields into spherical harmonics (Orders 1-4) with steerable look directions, null steering, and spatial diffuse decorrelation")
        .with_param(DspParamSchema::knob("beam_steering_azimuth_deg", "Beamformer Look Direction Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal aim angle of primary directional acoustic listening beam"))
        .with_param(DspParamSchema::knob("beam_steering_elevation_deg", "Beamformer Look Direction Elevation", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical tilt angle of directional listening beam above or below horizon"))
        .with_param(DspParamSchema::knob("spherical_order_directivity", "Spherical Harmonic Expansion Order", 1.0, 4.0, 3.0, "order", MacroRole::Punch, "Spatial sharpness of directional acoustic beam (higher orders yield laser focus)"))
        .with_param(DspParamSchema::knob("null_steering_suppression_db", "Spatial Interference Null Attenuation", 0.0, 40.0, 24.0, "dB", MacroRole::Tone, "Depth of steerable cancellation notch placed toward unwanted acoustic sources"))
        .with_param(DspParamSchema::knob("diffuse_soundfield_energy_blend", "Diffuse Ambient Soundfield Preservation", 0.0, 1.0, 0.25, "ratio", MacroRole::Character, "Maintains natural room reverberation around directional isolated target"))
        .with_param(DspParamSchema::toggle("enable_binaural_ear_synthesis", "Spherical Array to Binaural HRTF Synthesis", true, "Renders steerable beam output directly through calibrated binaural HRTF filters"))
    );
    descriptors.insert("BinauralRaytracedEarlyReflectionSpatializer".to_string(), DspNodeDescriptor::new("BinauralRaytracedEarlyReflectionSpatializer", "Binaural Raytraced Early Reflection Spatializer", DspNodeCategory::SpatialSurround, "Shoebox room acoustic simulator combining image-source algorithms and stochastic raytracing to calculate binaural early reflections, boundary material absorption, and HRTF parallax")
        .with_param(DspParamSchema::knob("virtual_room_length_meters", "Virtual Acoustic Room Length", 2.0, 40.0, 12.0, "m", MacroRole::Space, "Length dimension of rectangular virtual acoustic chamber"))
        .with_param(DspParamSchema::knob("virtual_room_width_meters", "Virtual Acoustic Room Width", 2.0, 30.0, 8.5, "m", MacroRole::Space, "Width dimension governing lateral reflection delays and spatial width"))
        .with_param(DspParamSchema::knob("virtual_room_height_meters", "Virtual Acoustic Room Height", 2.0, 15.0, 4.0, "m", MacroRole::Tone, "Ceiling height determining vertical reflection timing"))
        .with_param(DspParamSchema::knob("wall_surface_absorption_alpha", "Boundary Acoustic Absorption Alpha", 0.05, 0.95, 0.35, "alpha", MacroRole::Character, "Absorption coefficient of wall materials (concrete vs wood panels)"))
        .with_param(DspParamSchema::knob("direct_to_reverberant_ratio_db", "Direct Sound vs Reflection Ratio", -24.0, 24.0, 4.0, "dB", MacroRole::Punch, "Energy balance between direct dry sound and early room reflections"))
        .with_param(DspParamSchema::toggle("enable_high_order_corner_scattering", "Second-Order Corner Scattering Rays", true, "Traces complex secondary reflections bouncing off wall-floor intersections"))
    );
    descriptors.insert("AmbisonicRadialDistanceFilterBinaural".to_string(), DspNodeDescriptor::new("AmbisonicRadialDistanceFilterBinaural", "Higher-Order Ambisonic Radial Distance Filter (RDF)", DspNodeCategory::SpatialSurround, "HOA spherical wave curvature distance filter applying Bessel function near-field effect correction, parallax compensation, and frequency-dependent distance attenuation")
        .with_param(DspParamSchema::knob("sound_source_radial_distance_m", "Virtual Source Radial Distance", 0.1, 50.0, 1.8, "m", MacroRole::Space, "Apparent radial distance of virtual audio source from listener head"))
        .with_param(DspParamSchema::knob("near_field_bass_boost_boost_db", "Acoustic Proximity Bass Shelf Boost", 0.0, 24.0, 6.5, "dB", MacroRole::Punch, "Proximity effect boosting lower frequencies when source is near head"))
        .with_param(DspParamSchema::knob("air_absorption_humidity_percent", "Atmospheric Relative Humidity", 10.0, 100.0, 50.0, "%", MacroRole::Tone, "Air humidity defining high-frequency atmospheric absorption curve"))
        .with_param(DspParamSchema::knob("spherical_wave_order_truncation", "Ambisonic Radial Expansion Order", 1.0, 7.0, 4.0, "order", MacroRole::Character, "Spherical Bessel expansion order for radial wave curvature calculation"))
        .with_param(DspParamSchema::knob("parallax_elevation_compensation_deg", "Distance Parallax Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Elevation parallax shift compensation between ears and source"))
        .with_param(DspParamSchema::toggle("enable_inverse_square_law_decay", "Physical 1/R Amplitude Attenuation", true, "Enforces strict physical 1/r energy decay with distance limiting"))
    );
    descriptors.insert("AmbisonicSphericalMicrophonePanner".to_string(), DspNodeDescriptor::new("AmbisonicSphericalMicrophonePanner", "Spherical HOA 3D Spatial Panner & Doppler Engine", DspNodeCategory::SpatialSurround, "3D ambisonic spatializer with azimuth, elevation, and distance control, velocity Doppler frequency shift, spherical head shadow emulation, and binaural decoding")
        .with_param(DspParamSchema::knob("source_azimuth_angle_deg", "Spatial Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular placement around virtual listener head"))
        .with_param(DspParamSchema::knob("source_elevation_angle_deg", "Spatial Vertical Elevation", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical angle above or below the listener horizontal plane"))
        .with_param(DspParamSchema::knob("source_radial_distance_m", "Radial Spatial Distance from Center", 0.2, 30.0, 2.5, "m", MacroRole::Tone, "Source distance governing acoustic gain and high frequency roll-off"))
        .with_param(DspParamSchema::knob("doppler_motion_velocity_mps", "Source Motion Velocity (Doppler)", -50.0, 50.0, 0.0, "m/s", MacroRole::Punch, "Radial velocity vector causing real-time pitch Doppler shift"))
        .with_param(DspParamSchema::knob("acoustic_head_shadow_strength", "Pinna & Head Acoustic Shadow Filter", 0.0, 1.0, 0.7, "shadow", MacroRole::Character, "Frequency-dependent head diffraction filtering opposite-ear signals"))
        .with_param(DspParamSchema::toggle("enable_binaural_diffuse_field_eq", "Diffuse-Field Equalization Filter", true, "Applies diffuse-field equalization curve to binaural HRIR decoders"))
    );
    descriptors.insert("TransauralAcousticDipoleSpatializer".to_string(), DspNodeDescriptor::new("TransauralAcousticDipoleSpatializer", "Transaural Acoustic Dipole 3D Spatializer", DspNodeCategory::SpatialSurround, "Acoustic dipole radiation pattern spatializer with figure-8 directional polar dispersion, virtual listener head shadowing, and inverse distance air attenuation")
        .with_param(DspParamSchema::knob("dipole_axis_azimuth_angle_deg", "Dipole Acoustic Axis Azimuth Angle", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Azimuth angle orientation of the virtual acoustic dipole radiating axis"))
        .with_param(DspParamSchema::knob("dipole_figure_eight_purity", "Figure-8 Dipole Polar Dispersion Ratio", 0.0, 1.0, 0.85, "ratio", MacroRole::Character, "Crossfade between omnidirectional monopole (0) and figure-8 velocity dipole (1)"))
        .with_param(DspParamSchema::knob("distance_radiation_falloff_m", "Dipole Near-Field Distance Attenuation", 0.2, 50.0, 2.5, "m", MacroRole::Space, "Distance between dipole acoustic center and listening evaluation position"))
        .with_param(DspParamSchema::knob("contralateral_pinna_diffraction_db", "Contralateral Ear Pinna Diffraction", 0.0, 18.0, 7.5, "dB", MacroRole::Tone, "High-frequency head-shadow diffraction applied to the acoustic far ear"))
        .with_param(DspParamSchema::knob("spatial_crosstalk_cancellation_depth", "Stereo Speaker Crosstalk Cancellation", 0.0, 1.0, 0.6, "depth", MacroRole::Space, "Interaural crosstalk cancellation depth for speaker playback spatialization"))
        .with_param(DspParamSchema::toggle("enable_ground_reflection_bounce", "Virtual Floor Boundary Acoustic Bounce", true, "Calculates specular ground reflection and comb filtering from acoustic floor plane"))
    );
    descriptors.insert("AmbisonicDistanceAirAbsorptionFilter".to_string(), DspNodeDescriptor::new("AmbisonicDistanceAirAbsorptionFilter", "ISO 9613-1 Ambisonic Atmospheric Air Absorption", DspNodeCategory::SpatialSurround, "ISO 9613-1 acoustic atmospheric air absorption and spherical wavefront curvature filter for high-order ambisonic 3D soundfield environments")
        .with_param(DspParamSchema::knob("virtual_source_distance_meters", "Virtual Sound Source Distance", 0.5, 500.0, 18.0, "m", MacroRole::Space, "Physical distance between virtual sound emitter and ambisonic listening point"))
        .with_param(DspParamSchema::knob("ambient_air_temperature_celsius", "Atmospheric Air Temperature", -20.0, 50.0, 20.0, "C", MacroRole::Tone, "Ambient temperature in Celsius dictating speed of sound and molecular relaxation frequencies"))
        .with_param(DspParamSchema::knob("relative_air_humidity_percent", "Atmospheric Relative Humidity", 10.0, 99.0, 50.0, "%", MacroRole::Tone, "Relative air humidity altering water vapor molecular vibrational absorption resonance"))
        .with_param(DspParamSchema::knob("inverse_square_gain_attenuation_db", "Spherical Inverse-Square Law Falloff", 0.0, 60.0, 24.0, "dB", MacroRole::Space, "Acoustic geometric divergence attenuation over distance (6 dB per doubling of distance)"))
        .with_param(DspParamSchema::knob("ambisonic_order_proximity_bass_boost", "Near-Field Wavefront Curvature Proximity Bass", 0.0, 18.0, 4.0, "dB", MacroRole::Punch, "Spherical wavefront radial curvature causing low-frequency proximity boost on close sources"))
        .with_param(DspParamSchema::toggle("enable_doppler_pitch_shift_air_motion", "Wind Vector Atmospheric Doppler Frequency Shift", false, "Simulates pitch modulation caused by atmospheric wind drift and medium movement"))
    );
    descriptors.insert("SphericalBinauralPinnaDiffractionSpatializer".to_string(), DspNodeDescriptor::new("SphericalBinauralPinnaDiffractionSpatializer", "Spherical BEM HRTF Pinna & Torso Diffraction 3D Spatializer", DspNodeCategory::SpatialSurround, "Boundary element method (BEM) 3D binaural spatializer modeling anatomical ear pinna cavum concha notches, ear canal resonance, and upper torso acoustic diffraction")
        .with_param(DspParamSchema::knob("sound_source_azimuth_deg", "Horizontal Soundfield Azimuth Angle", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal angular position of virtual sound emitter (-180 Left to +180 Right)"))
        .with_param(DspParamSchema::knob("sound_source_elevation_deg", "Vertical Soundfield Elevation Angle", -90.0, 90.0, 15.0, "deg", MacroRole::Space, "Vertical angle of virtual sound emitter (-90 Nadir to +90 Zenith)"))
        .with_param(DspParamSchema::knob("ear_pinna_size_scaling_factor", "Anatomical Pinna Cavum Dimension Scaling", 0.7, 1.4, 1.0, "scale", MacroRole::Character, "Morphs pinna spectral notch frequencies to match listener ear dimensions"))
        .with_param(DspParamSchema::knob("torso_chest_shadowing_attenuation_db", "Torso Shadowing Acoustic Attenuation", 0.0, 18.0, 6.5, "dB", MacroRole::Tone, "High-frequency acoustic shadow and reflection from human shoulders and chest"))
        .with_param(DspParamSchema::knob("head_acoustic_diameter_cm", "Interaural Spherical Head Diameter", 12.0, 24.0, 17.5, "cm", MacroRole::Space, "Interaural distance defining Woodworth spherical head delay formula time offsets"))
        .with_param(DspParamSchema::toggle("enable_interaural_spectral_cross_talk", "Diffuse Field Equalization Compensation", true, "Equalizes headphone HRTF response to standard diffuse-field target curve"))
    );
    descriptors.insert("AcousticParabolicReflectorSpatializer".to_string(), DspNodeDescriptor::new("AcousticParabolicReflectorSpatializer", "Acoustic Parabolic Dish Concentrator & Focal Beam Spatializer", DspNodeCategory::SpatialSurround, "Physical geometric simulation of an acoustic parabolic sound mirror dish concentrating sound at focal point with narrow dispersion beam, edge diffraction, and focal gain")
        .with_param(DspParamSchema::knob("parabolic_dish_diameter_meters", "Parabolic Reflector Dish Aperture Diameter", 0.3, 5.0, 1.2, "m", MacroRole::Space, "Physical diameter of reflective dish determining low-frequency acoustic cutoff limit"))
        .with_param(DspParamSchema::knob("focal_point_distance_offset_cm", "Focal Microphone Position Axial Offset", -20.0, 20.0, 0.0, "cm", MacroRole::Tone, "Axial position of receiver relative to parabolic focus shaping phase coherence"))
        .with_param(DspParamSchema::knob("acoustic_beam_directivity_q", "Directional Acoustic Beam Sharpness (Q)", 1.0, 30.0, 8.5, "Q", MacroRole::Punch, "On-axis directional acoustic amplification sharpness rejecting lateral ambient sounds"))
        .with_param(DspParamSchema::knob("rim_edge_diffraction_cancellation", "Dish Perimeter Rim Edge Wave Diffraction", 0.0, 1.0, 0.42, "diff", MacroRole::Character, "Edge boundary wave diffraction interference causing destructive phase ripple"))
        .with_param(DspParamSchema::knob("distance_air_viscous_attenuation_db", "High-Frequency Atmospheric Distance Loss", 0.0, 30.0, 4.0, "dB", MacroRole::Tone, "ISO air molecular absorption loss across traveled path"))
        .with_param(DspParamSchema::toggle("enable_spherical_aberration_modeling", "Deep Dish Off-Axis Spherical Aberration", true, "Models phase blurring when sounds arrive at angles oblique to the parabolic axis"))
    );
    descriptors.insert("WfsAcousticCircularArrayRenderer".to_string(), DspNodeDescriptor::new("WfsAcousticCircularArrayRenderer", "Wave Field Synthesis (WFS) Circular Speaker Array Renderer", DspNodeCategory::SpatialSurround, "Holographic Wave Field Synthesis audio renderer driving circular loudspeaker arrays according to Kirchhoff-Helmholtz integral acoustics, producing real acoustic wavefront curvature")
        .with_param(DspParamSchema::knob("virtual_source_distance_meters", "Virtual Sound Source Radial Distance", -15.0, 15.0, 3.5, "m", MacroRole::Space, "Distance from array center; negative values render focused sources inside the room"))
        .with_param(DspParamSchema::knob("virtual_source_angle_degrees", "Virtual Sound Source Angular Direction", -180.0, 180.0, 45.0, "deg", MacroRole::Space, "Angular direction of virtual sound wavefront source around listening ring"))
        .with_param(DspParamSchema::knob("array_diameter_meters", "Loudspeaker Circular Array Physical Diameter", 1.5, 20.0, 5.0, "m", MacroRole::Space, "Diameter of real physical surround speaker ring in venue"))
        .with_param(DspParamSchema::knob("spatial_aliasing_cutoff_hz", "Discrete Transducer Spatial Aliasing Frequency", 800.0, 8000.0, 1650.0, "Hz", MacroRole::Tone, "Upper frequency limit determined by speaker spacing above which wavefront ripples appear"))
        .with_param(DspParamSchema::knob("attenuation_prefilter_slope_db", "Half-Derivative 3 dB/Oct Wavefront Prefilter", 0.0, 6.0, 3.0, "dB/oct", MacroRole::Tone, "Acoustic line-source to point-source frequency correction filter"))
        .with_param(DspParamSchema::toggle("enable_focused_source_room_compensation", "Focused Source Evaporation Boundary Compensation", true, "Compensates acoustic inversion when virtual sound is placed in front of speakers"))
    );
    descriptors.insert("AmbisonicBinauralEarCanalOcclusion".to_string(), DspNodeDescriptor::new("AmbisonicBinauralEarCanalOcclusion", "Ambisonic HRTF Concha & Ear Canal Occlusion Resonator", DspNodeCategory::SpatialSurround, "Binaural HRIR spatial processor incorporating detailed anatomical modeling of ear canal quarter-wave acoustic cavity resonance, in-ear earbud occlusion effect, and tragus diffraction")
        .with_param(DspParamSchema::knob("ear_canal_length_mm", "Anatomical Ear Canal Physical Depth Length", 18.0, 35.0, 26.0, "mm", MacroRole::Tone, "Physical length of the external auditory meatus setting primary 3-4 kHz quarter-wave resonance"))
        .with_param(DspParamSchema::knob("occlusion_seal_depth_factor", "Earbud Silicone Tip Ear Canal Occlusion Seal", 0.0, 1.0, 0.85, "seal", MacroRole::Punch, "Tightness of in-ear acoustic seal amplifying bone-conduction bass and trapped cavity resonances"))
        .with_param(DspParamSchema::knob("concha_cavity_anti_resonance_hz", "Pinna Concha Anti-Resonance Pin Hole Notch", 6000.0, 12000.0, 8400.0, "Hz", MacroRole::Tone, "Spectral pinna notch frequency providing critical height and vertical elevation perception"))
        .with_param(DspParamSchema::knob("interaural_time_delay_us", "Interaural Head Diffraction Time Delay (ITD)", 0.0, 800.0, 450.0, "us", MacroRole::Space, "Sub-millisecond arrival time difference between ears for horizontal localization"))
        .with_param(DspParamSchema::knob("tympanic_membrane_acoustic_impedance", "Eardrum Acoustic Terminal Compliance (Z0)", 0.1, 2.0, 0.95, "Z0", MacroRole::Character, "Tympanic membrane compliance damping ear canal reflections"))
        .with_param(DspParamSchema::toggle("enable_head_motion_yaw_compensation", "Low-Latency Head Tracker Yaw Angular Compensation", false, "Locks soundstage position in space when user rotates head wearing IMU headphones"))
    );
    descriptors.insert("AmbisonicSphericalHeadShadowDiffraction".to_string(), DspNodeDescriptor::new("AmbisonicSphericalHeadShadowDiffraction", "Higher-Order Ambisonic Spherical Head Shadow & ITD", DspNodeCategory::SpatialSurround, "Analytical spherical head diffraction processor for Higher-Order Ambisonics (HOA) applying Rayleigh-Sommerfeld creeping wave diffraction, continuous ITD phase delay, and pinna shadowing")
        .with_param(DspParamSchema::knob("head_radius_meters", "Rigid Sphere Cranial Radius (Anatomical Head Size)", 0.06, 0.12, 0.0875, "m", MacroRole::Tone, "Physical cranial radius setting exact frequency onset of head shadow attenuation"))
        .with_param(DspParamSchema::knob("creeping_wave_diffraction_damping", "Contralateral Creeping Wave Shadow Attenuation", 0.0, 24.0, 12.5, "dB", MacroRole::Tone, "High-frequency attenuation on ear shadowed by the skull"))
        .with_param(DspParamSchema::knob("interaural_time_delay_scaling", "Woodworth-Schlosberg ITD Geometric Time Delay", 0.5, 1.5, 1.0, "scale", MacroRole::Space, "Scales microsecond time delay between ears for horizontal angle localization"))
        .with_param(DspParamSchema::knob("elevation_pinna_notch_frequency_hz", "Ear Pinna Elevation Concha Notch Tuning", 5000.0, 11000.0, 7800.0, "Hz", MacroRole::Character, "Spectral notch frequency providing vertical elevation perception cues"))
        .with_param(DspParamSchema::knob("torso_shoulder_bounce_delay_ms", "Shoulder & Torso Acoustic Reflection Delay", 0.5, 4.0, 1.2, "ms", MacroRole::Space, "Early reflection off shoulders providing vital realistic externalization cues"))
        .with_param(DspParamSchema::toggle("enable_near_field_curvature_correction", "Near-Field Wavefront Curvature Correction", true, "Compensates spherical wave curvature when sound source is closer than 1 meter"))
    );
    descriptors.insert("WfsPlanarLinearArrayAcousticSynthesizer".to_string(), DspNodeDescriptor::new("WfsPlanarLinearArrayAcousticSynthesizer", "Wave Field Synthesis Linear Speaker Array Synthesizer", DspNodeCategory::SpatialSurround, "Wave Field Synthesis (WFS) linear boundary array engine driving physical or virtual loudspeaker lines, calculating Rayleigh I diffraction integrals for holographic spatial audio reproduction")
        .with_param(DspParamSchema::knob("virtual_source_x_pos_meters", "Virtual Sound Source Lateral Position (X)", -10.0, 10.0, 0.0, "m", MacroRole::Space, "Left/right position of virtual sound source relative to array center"))
        .with_param(DspParamSchema::knob("virtual_source_depth_z_meters", "Virtual Sound Source Depth Distance (Z)", -8.0, 12.0, 2.5, "m", MacroRole::Space, "Distance perpendicular to speaker line; negative values place sound inside room"))
        .with_param(DspParamSchema::knob("loudspeaker_spacing_pitch_meters", "Physical Speaker Array Element Spacing Pitch", 0.05, 0.5, 0.12, "m", MacroRole::Character, "Distance between individual speakers in line setting spatial aliasing boundary"))
        .with_param(DspParamSchema::knob("spatial_tapering_window_width", "Array Edge Cosine Window Tapering Width", 0.0, 1.0, 0.35, "win", MacroRole::Tone, "Tapers edge speakers to prevent diffraction ripples at ends of speaker line"))
        .with_param(DspParamSchema::knob("wavefront_curvature_comp_db", "Distance Attenuation 1/sqrt(r) Gain Correction", -12.0, 6.0, 0.0, "dB", MacroRole::Punch, "Compensates cylindrical-to-spherical wavefront energy conservation"))
        .with_param(DspParamSchema::toggle("enable_focused_point_source_rendering", "Focused Point Source Interior Wavefront Mode", false, "Renders virtual source between listener and speaker array with converging wavefronts"))
    );
    descriptors.insert("AmbisonicSphericalMicrophoneArrayBinauralizer".to_string(), DspNodeDescriptor::new("AmbisonicSphericalMicrophoneArrayBinauralizer", "Higher-Order Ambisonic Spherical Array Binauralizer", DspNodeCategory::SpatialSurround, "High-order spherical microphone array (32-capsule rigid sphere) decoder converting higher-order spherical harmonics (HOA) into HRIR-matched binaural reproduction with head-tracking compensation")
        .with_param(DspParamSchema::knob("hoa_spherical_order", "Spherical Harmonics Ambisonic Order (1st-5th)", 1.0, 5.0, 3.0, "order", MacroRole::Space, "Harmonic decomposition order: higher orders provide pinpoint spatial angular accuracy"))
        .with_param(DspParamSchema::knob("rigid_sphere_radius_cm", "Microphone Array Sphere Radius", 2.0, 20.0, 8.5, "cm", MacroRole::Character, "Physical radius of rigid array sphere compensating for high-frequency scattering"))
        .with_param(DspParamSchema::knob("pinna_spectral_diffraction_db", "Pinna Concha Spectral Notch Intensity", 0.0, 18.0, 9.5, "dB", MacroRole::Tone, "Depth of 6-8 kHz spectral pinna notch indicating vertical elevation"))
        .with_param(DspParamSchema::knob("interaural_cross_talk_damping_db", "Transaural Acoustic Shadow Attenuation", 0.0, 24.0, 14.0, "dB", MacroRole::Space, "Head shadow attenuation depth across the contralateral ear"))
        .with_param(DspParamSchema::knob("head_yaw_rotation_deg", "Binaural Listener Head Yaw Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Character, "Horizontal head orientation compensating for head tracker motion"))
        .with_param(DspParamSchema::toggle("enable_diffuse_field_equalization", "Diffuse-Field HRIR Equalization", true, "Flattens collective HRTF frequency response for fatigue-free headphone monitoring"))
    );
    descriptors.insert("TransauralStereoDipoleCrosstalkCanceller".to_string(), DspNodeDescriptor::new("TransauralStereoDipoleCrosstalkCanceller", "Transaural Stereo Dipole Loudspeaker Crosstalk Canceller", DspNodeCategory::SpatialSurround, "Transaural acoustic dipole speaker layout filter network providing wide 3D panoramic imaging over narrow loudspeaker setups using recursive cross-talk cancellation filters")
        .with_param(DspParamSchema::knob("loudspeaker_span_angle_deg", "Stereo Dipole Loudspeaker Span Angle", 5.0, 60.0, 12.0, "deg", MacroRole::Space, "Angular separation between narrow speakers (typical dipole: 10-15 degrees)"))
        .with_param(DspParamSchema::knob("listener_distance_meters", "Listener Distance from Loudspeakers", 0.5, 4.0, 1.8, "m", MacroRole::Tone, "Distance setting interaural path length delay compensation"))
        .with_param(DspParamSchema::knob("crosstalk_cancellation_depth_db", "Contralateral Crosstalk Cancellation Depth", 6.0, 36.0, 22.0, "dB", MacroRole::Space, "Attenuates contralateral acoustic leakage reaching the opposite ear"))
        .with_param(DspParamSchema::knob("sweet_spot_width_tolerance", "Sweet Spot Off-Axis Tolerance Window", 0.1, 1.0, 0.5, "tol", MacroRole::Character, "Broadens listening sweet spot at the cost of maximum cancellation depth"))
        .with_param(DspParamSchema::knob("low_frequency_bass_monomaker_hz", "Bass Monomaker Crossover Frequency", 40.0, 250.0, 90.0, "Hz", MacroRole::Punch, "Blends sub-bass to mono avoiding out-of-phase speaker excursion cancellation"))
        .with_param(DspParamSchema::toggle("enable_head_motion_robustness", "Head Motion Robustness Pre-Filtering", true, "Prevents harsh phase artifacts when listener shifts off acoustic center"))
    );
    descriptors.insert("HigherOrderAmbisonicBinauralNearFieldCompensator".to_string(), DspNodeDescriptor::new("HigherOrderAmbisonicBinauralNearFieldCompensator", "Higher-Order Ambisonic Near-Field Distance Compensator", DspNodeCategory::SpatialSurround, "5th-order Ambisonic near-field effect (NFE) radial filter correcting low-frequency spherical wave curvature and proximity bass boost for close-distance virtual audio sources")
        .with_param(DspParamSchema::knob("source_radial_distance_meters", "Virtual Source Radial Distance", 0.1, 10.0, 0.45, "m", MacroRole::Space, "Virtual distance from listener head center to sound source"))
        .with_param(DspParamSchema::knob("ambisonic_spherical_order_limit", "Maximum HOA Spherical Expansion Order", 1.0, 5.0, 3.0, "order", MacroRole::Character, "Maximum spherical harmonics expansion order processed by near-field filters"))
        .with_param(DspParamSchema::knob("acoustic_reference_radius_meters", "Acoustic Far-Field Normalization Radius", 0.5, 5.0, 1.5, "m", MacroRole::Space, "Far-field normalization radius for binaural decoder matching"))
        .with_param(DspParamSchema::knob("proximity_bass_boost_damping", "Spherical Proximity Bass Boost Damping", 0.0, 1.0, 0.25, "damp", MacroRole::Tone, "Attenuates excessive sub-bass accumulation on extreme close-mic positions"))
        .with_param(DspParamSchema::knob("air_absorption_humidity_percent", "Atmospheric Relative Humidity Percentage", 10.0, 95.0, 50.0, "%", MacroRole::Tone, "Atmospheric relative humidity calculating high-frequency distance attenuation"))
        .with_param(DspParamSchema::toggle("enable_spherical_wave_parallax_shift", "Interaural Spherical Wave Parallax Shift", true, "Applies interaural parallax angle shift between left and right ears"))
    );
    descriptors.insert("TransauralAcousticCrossfeedHeadphoneOptimizer".to_string(), DspNodeDescriptor::new("TransauralAcousticCrossfeedHeadphoneOptimizer", "Transaural Acoustic Crossfeed Headphone Optimizer", DspNodeCategory::SpatialSurround, "Psychoacoustic natural crossfeed filter network simulating loudspeaker acoustic head-diffraction on headphones to eliminate lateral stereo fatigue and center hole effect")
        .with_param(DspParamSchema::knob("crossfeed_blend_intensity_db", "Contralateral Crossfeed Blend Depth", -24.0, -6.0, -12.0, "dB", MacroRole::Space, "Level of contralateral signal mixed into opposite ear channel"))
        .with_param(DspParamSchema::knob("head_shadow_crossover_hz", "Acoustic Head-Shadow Baffle Crossover", 300.0, 1200.0, 650.0, "Hz", MacroRole::Tone, "Head-shadow acoustic baffle corner frequency above which sound is delayed & filtered"))
        .with_param(DspParamSchema::knob("interaural_time_delay_microsec", "Interaural Time Delay (ITD) Path Length", 100.0, 450.0, 260.0, "us", MacroRole::Space, "Interaural time difference (ITD) delay simulating typical head width (~21 cm)"))
        .with_param(DspParamSchema::knob("natural_room_reflection_ambience", "Binaural Early Ambience Blend", 0.0, 1.0, 0.22, "amb", MacroRole::Space, "Adds subtle early binaural room reflections to mimic high-end listening room"))
        .with_param(DspParamSchema::knob("low_frequency_mono_depth", "Sub-Bass Mono Anchor Convergence", 0.0, 1.0, 0.4, "mono", MacroRole::Punch, "Gradually converges sub-bass below 100 Hz to stereo center for solid punch"))
        .with_param(DspParamSchema::toggle("enable_harman_target_tilt_equalization", "Harman Headphone Target Tilt Curve", true, "Applies gentle Harman headphone target compensation curve"))
    );
    descriptors.insert("InterauralPinnaSpectralNotchFilter".to_string(), DspNodeDescriptor::new("InterauralPinnaSpectralNotchFilter", "Interaural Pinna Spectral Notch Filter", DspNodeCategory::SpatialSurround, "Psychoacoustic vertical elevation and localization filter modeling outer ear pinna concha notches and anti-resonances")
        .with_param(DspParamSchema::knob("elevation_angle", "Vertical Elevation Angle", -45.0, 90.0, 0.0, "deg", MacroRole::Space, "Apparent vertical angle of the source relative to horizontal ear axis"))
        .with_param(DspParamSchema::knob("notch_depth", "Pinna Concha Notch Depth", 0.0, 36.0, 18.0, "dB", MacroRole::Tone, "Attenuation depth of the destructive interference pinna spectral notch"))
        .with_param(DspParamSchema::knob("pinna_scale", "Ear Anthropometric Scaling", 0.8, 1.2, 1.0, "scale", MacroRole::Tone, "Anatomical ear size scale shifting notch center frequencies"))
        .with_param(DspParamSchema::knob("torso_reflection", "Shoulder & Torso Reflection", 0.0, 1.0, 0.30, "refl", MacroRole::Space, "Early torso and clavicle acoustic reflections providing lower elevation cues"))
        .with_param(DspParamSchema::knob("interaural_balance", "Interaural Asymmetry Balance", -1.0, 1.0, 0.0, "pan", MacroRole::Space, "Fine left/right pinna resonance coloration balance"))
        .with_param(DspParamSchema::knob("spectral_smoothing", "HRTF Curve Smoothing", 0.0, 1.0, 0.20, "smooth", MacroRole::Character, "Reduces steep notch phase jitter to prevent comb filter coloration"))
    );
    descriptors.insert("AmbisonicSphericalCapMicrophonePanner".to_string(), DspNodeDescriptor::new("AmbisonicSphericalCapMicrophonePanner", "Ambisonic Spherical Cap Microphone Panner", DspNodeCategory::SpatialSurround, "Encodes mono/stereo signals into higher-order spherical ambisonics (HOA) with virtual directional spherical cap capsule weighting")
        .with_param(DspParamSchema::knob("azimuth", "Horizontal Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Horizontal angular direction of the sound source"))
        .with_param(DspParamSchema::knob("elevation", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Vertical angle of source relative to the listening plane"))
        .with_param(DspParamSchema::knob("distance", "Virtual Source Radial Distance", 0.1, 20.0, 2.0, "m", MacroRole::Space, "Radial distance from center applying 1/r law and air absorption"))
        .with_param(DspParamSchema::knob("cap_aperture", "Capsule Cap Opening Angle", 10.0, 180.0, 90.0, "deg", MacroRole::Space, "Virtual spherical cap microphone opening angle / directivity pattern"))
        .with_param(DspParamSchema::knob("hoa_order", "Ambisonic Spherical Harmonic Order", 1.0, 7.0, 3.0, "order", MacroRole::Character, "HOA encoding truncation order governing spatial focus"))
        .with_param(DspParamSchema::knob("divergence", "Source Divergence Spread", 0.0, 1.0, 0.15, "spread", MacroRole::Space, "Source volume width expanding source beyond a single point"))
    );
    descriptors.insert("BinauralSpectralNotchElevationSynthesizer".to_string(), DspNodeDescriptor::new("BinauralSpectralNotchElevationSynthesizer", "Binaural Spectral Notch Elevation Synthesizer", DspNodeCategory::SpatialSurround, "Head-related acoustic pinna diffraction filter synthesizing spatial elevation perception via dynamic spectral notch tracking and shoulder reflection timing")
        .with_param(DspParamSchema::knob("elevation_angle", "Vertical Elevation Angle", -45.0, 90.0, 15.0, "deg", MacroRole::Space, "Apparent vertical source angle driving dynamic pinna spectral notch frequencies"))
        .with_param(DspParamSchema::knob("concha_notch_attenuation", "Concha Destructive Notch Depth", 0.0, 40.0, 22.0, "dB", MacroRole::Tone, "Depth of pinna notch anti-resonance delivering vertical height perception"))
        .with_param(DspParamSchema::knob("ear_canal_resonance", "Ear Canal 3kHz Acoustic Peaking", 0.0, 12.0, 4.5, "dB", MacroRole::Tone, "Resonant gain boost of the human auditory ear canal around 3 to 4 kHz"))
        .with_param(DspParamSchema::knob("shoulder_delay_time", "Shoulder Reflection Delay Offset", 0.2, 2.5, 0.85, "ms", MacroRole::Space, "Comb-filter delay time from anatomical shoulder reflection"))
        .with_param(DspParamSchema::knob("head_shadow_falloff", "Interaural Head Shadow Damping", 0.0, 1.0, 0.65, "loss", MacroRole::Character, "High-frequency acoustic shadowing from contralateral ear diffraction"))
        .with_param(DspParamSchema::knob("diffuse_field_equalization", "Diffuse-Field Calibration Curve", 0.0, 1.0, 0.85, "eq", MacroRole::Character, "Flattens unnatural coloration when listening through calibrated studio headphones"))
    );
    descriptors.insert("AmbisonicSphericalHarmonicRotatorMatrix".to_string(), DspNodeDescriptor::new("AmbisonicSphericalHarmonicRotatorMatrix", "Ambisonic Spherical Harmonic Rotator Matrix", DspNodeCategory::SpatialSurround, "Full 3D rotation and tilting matrix for 3rd-order Higher Order Ambisonics (HOA) soundfields with Yaw, Pitch, Roll coordinate transformations")
        .with_param(DspParamSchema::knob("yaw_heading_angle", "Yaw Horizontal Heading Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Azimuth rotation angle around the vertical Z axis"))
        .with_param(DspParamSchema::knob("pitch_elevation_angle", "Pitch Vertical Tilt Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Space, "Elevation rotation angle around the lateral Y axis"))
        .with_param(DspParamSchema::knob("roll_tilt_angle", "Roll Lateral Tumble Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Tumble rotation angle around the frontal X axis"))
        .with_param(DspParamSchema::knob("hoa_order_selection", "Spherical Harmonics HOA Order", 1.0, 5.0, 3.0, "order", MacroRole::Character, "Ambisonic expansion order processed by the 3D rotation transform"))
        .with_param(DspParamSchema::knob("spatial_energy_normalization", "Acoustic Energy Conservation", 0.0, 1.0, 1.0, "norm", MacroRole::Punch, "Ensures total soundfield acoustic power remains strictly invariant during rotation"))
        .with_param(DspParamSchema::knob("angular_smoothing_lag", "Gimbal Smoothing Lag Constant", 5.0, 500.0, 50.0, "ms", MacroRole::Space, "Lowpass slew-rate limiting on orientation angles preventing zipper noise"))
    );
    descriptors.insert("BinauralConchaReflectionPinnaFilter".to_string(), DspNodeDescriptor::new("BinauralConchaReflectionPinnaFilter", "Binaural Concha Reflection & Pinna Notch Filter", DspNodeCategory::SpatialSurround, "Anatomical pinna model simulating high-frequency acoustic reflections off the human concha bowl, tragus notch, and fossa triangularis for pin-point vertical elevation cues")
        .with_param(DspParamSchema::knob("elevation_angle_degrees", "Sound Source Elevation Angle", -45.0, 90.0, 15.0, "deg", MacroRole::Tone, "Sound source vertical angle above or below the horizontal ear canal horizon"))
        .with_param(DspParamSchema::knob("azimuth_angle_degrees", "Sound Source Horizontal Azimuth", -180.0, 180.0, 30.0, "deg", MacroRole::Space, "Horizontal azimuth panning angle around the listener's head"))
        .with_param(DspParamSchema::knob("concha_cavity_depth", "Anatomical Concha Cavity Depth", 5.0, 25.0, 14.0, "mm", MacroRole::Character, "Physical concha bowl acoustic dimension shaping primary elevation notch frequencies"))
        .with_param(DspParamSchema::knob("pinna_notch_frequency", "Primary Pinna Spectral Notch", 4000.0, 12000.0, 7800.0, "Hz", MacroRole::Tone, "Center frequency of the destructive pinna interference notch"))
        .with_param(DspParamSchema::knob("torso_shoulder_reflection", "Shoulder Reflection Delay Offset", 0.1, 2.5, 0.8, "ms", MacroRole::Space, "Comb-filtering delay time from upper torso and shoulder bounce reflections"))
        .with_param(DspParamSchema::knob("head_shadow_dispersion", "Spherical Head Acoustic Shadowing", 0.0, 1.0, 0.75, "damp", MacroRole::Punch, "High-frequency interaural level attenuation from contra-lateral skull mass"))
    );
    descriptors.insert("AmbisonicThirdOrderPlanarSoundfieldDecoder".to_string(), DspNodeDescriptor::new("AmbisonicThirdOrderPlanarSoundfieldDecoder", "Ambisonic 3rd-Order Planar Soundfield Decoder", DspNodeCategory::SpatialSurround, "16-channel HOA (Higher Order Ambisonics) 3rd-order spatial soundfield decoder with max-rE energy vector weighting, near-field distance compensation, and loudspeaker layout optimization")
        .with_param(DspParamSchema::knob("ambisonic_order_limit", "Active Ambisonic Decomposition Order", 1.0, 3.0, 3.0, "ord", MacroRole::Tone, "Spherical harmonic expansion order dictating spatial angular resolution"))
        .with_param(DspParamSchema::knob("max_re_energy_weighting", "Max-rE Acoustic Energy Beamforming", 0.0, 1.0, 0.85, "ratio", MacroRole::Space, "Optimizes energy vector focus for sharp localization across off-center listening areas"))
        .with_param(DspParamSchema::knob("distance_compensation_m", "Listener-to-Speaker Radius", 0.5, 10.0, 2.2, "m", MacroRole::Space, "Near-field spherical curvature wave compensation filter distance"))
        .with_param(DspParamSchema::knob("high_frequency_shelf_cutoff", "Dual-Band Shelf Crossover Freq", 200.0, 2000.0, 700.0, "Hz", MacroRole::Tone, "Crossover frequency between low-frequency velocity and high-frequency energy decoding"))
        .with_param(DspParamSchema::knob("binaural_decoding_matrix", "Decoded Loudspeaker Count", 4.0, 16.0, 8.0, "spk", MacroRole::Character, "Virtual or physical speaker array configuration (e.g. 5.1, 7.1.4, or 16-ch dome)"))
        .with_param(DspParamSchema::knob("spatial_blur_width", "Diffuse Soundfield Spread Angle", 0.0, 90.0, 12.0, "deg", MacroRole::Tone, "Controlled angular diffusion widening ambient reverberant soundfields"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "Atmos916Spatializer" => Box::new(
            GenericDspNodeUi::new("Atmos916Spatializer", "Dolby Atmos 9.1.6 Object Panner with HRTF", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("elevation", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("distance", "Virtual Distance", 0.5, 20.0, 2.0, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("spread", "Object Size Spread", 0.0, 100.0, 15.0, "%", (0, 230, 118)))
        ),
        "AmbisonicRadarSpatializer" => Box::new(
            GenericDspNodeUi::new("AmbisonicRadarSpatializer", "Higher-Order Ambisonics HOA-5 Spherical Radar", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("yaw", "Radar Yaw", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("pitch", "Radar Pitch", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("roll", "Radar Roll", -180.0, 180.0, 0.0, "deg", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("hoa_order", "Ambisonics Order", 1, 5, 3, "order", (153, 102, 255)))
        ),
        "Auro3dSpatializer" => Box::new(
            GenericDspNodeUi::new("Auro3dSpatializer", "Auro-3D 13.1 Tri-Layer Immersive Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("lower_layer", "Lower Ear-Level Gain", -12.0, 6.0, 0.0, "dB", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("height_layer", "Height Layer Gain", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("top_voice", "Top Voice of God", -12.0, 6.0, -3.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("spread", "Surround Width Spread", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
        ),
        "Nhk222Spatializer" => Box::new(
            GenericDspNodeUi::new("Nhk222Spatializer", "NHK 22.2 Super Hi-Vision Multi-Channel Matrix", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("top_gain", "Top Layer (9ch)", -12.0, 6.0, 0.0, "dB", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("mid_gain", "Middle Layer (10ch)", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("bottom_gain", "Bottom Layer (3ch)", -12.0, 6.0, -2.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("lfe_sub", "Dual LFE Subwoofers", -12.0, 6.0, 0.0, "dB", (255, 61, 113)))
        ),
        "BinauralBrirSpatializer" => Box::new(
            GenericDspNodeUi::new("BinauralBrirSpatializer", "Binaural Room Impulse Response Convolver", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("room_size", "Room Dimensions", 5.0, 50.0, 15.0, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_enum("hrtf", "HRTF Profile", vec!["KEMAR Dummy Head".into(), "CIPIC High-Res".into(), "Listen Individual".into(), "Sphaero HRIR".into()], 0, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("early_gain", "Early Reflections Gain", -24.0, 6.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("late_diffuse", "Late Diffuse Field", -24.0, 6.0, -6.0, "dB", (153, 102, 255)))
        ),
        "BinauralSpatializerNode" => Box::new(
            GenericDspNodeUi::new("BinauralSpatializerNode", "3D Binaural HRTF Spatializer with ITD & ILD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azim_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("elev_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("dist_m", "Source Distance", 0.2, 20.0, 1.5, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("itd_ild_mode", "Interaural Delay & Level On", true, (0, 230, 118)))
        ),
        "HyperDimensionalTensorSpatializer" => Box::new(
            GenericDspNodeUi::new("HyperDimensionalTensorSpatializer", "11-Dimensional Calabi-Yau Spatial Tensor Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_int("calabi_yau_dim", "Calabi-Yau Manifold Dims", 4, 11, 11, "dims", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("manifold_curvature", "Ricci Curvature Scaling", 0.1, 5.0, 1.8, "k", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("subspace_rotation_deg", "Subspace Rotation Angle", 0.0, 360.0, 45.0, "deg", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("dimensional_spread", "High-Dim Sound Spread", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
        ),
        "Holographic3dSpatialSoundfield" => Box::new(
            GenericDspNodeUi::new("Holographic3dSpatialSoundfield", "Holographic 3D Spherical Wavefield Synthesizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("spherical_radius_m", "Wavefield Sphere Radius", 0.5, 25.0, 3.0, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("azimuth_rad", "Source Azimuth Angle", -3.14, 3.14, 0.0, "rad", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("elevation_rad", "Source Elevation Angle", -1.57, 1.57, 0.0, "rad", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("wavefront_curvature", "Virtual Wave Curvature", 0.1, 5.0, 1.2, "", (0, 230, 118)))
        ),
        "AcousticCloakingSpatializer" => Box::new(
            GenericDspNodeUi::new("AcousticCloakingSpatializer", "Acoustic Cloaking Phase Cancellation Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("cloaking_radius_m", "Cloaking Shell Radius", 0.5, 10.0, 2.0, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("scattering_cancellation", "Phase Scattering Cancel", 0.0, 1.0, 0.9, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("shadow_attenuation_db", "Acoustic Shadow Depth", -40.0, 0.0, -18.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("phase_invert_blend", "Metasurface Anti-Phase Blend", 0.0, 1.0, 1.0, "%", (0, 230, 118)))
        ),
        "AtmosProximityNode" => Box::new(
            GenericDspNodeUi::new("AtmosProximityNode", "Dolby Atmos Proximity Effect & Distance Attenuator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("distance_m", "Acoustic Source Distance", 0.1, 50.0, 2.5, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("air_absorption_hf_loss_db", "Air Absorption HF Loss", 0.0, 24.0, 4.5, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("proximity_bass_boost_db", "Proximity Bass Boost", 0.0, 18.0, 3.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("itd_spread", "Interaural Distance Spread", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
        ),
        "AtmosSurroundNode" => Box::new(
            GenericDspNodeUi::new("AtmosSurroundNode", "7.1.4 Immersive Atmos Surround Bed Renderer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("surround_azimuth", "Bed Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("height_elevation", "Height Overhead Angle", 0.0, 90.0, 35.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("lfe_send_level", "LFE Subwoofer Send", -40.0, 6.0, 0.0, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("divergence_spread", "Center-Surround Divergence", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "HoaSpatializer" => Box::new(
            GenericDspNodeUi::new("HoaSpatializer", "4th/5th-Order High Order Ambisonic Spherical Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("spherical_yaw", "Ambisonic Yaw Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("spherical_pitch", "Ambisonic Pitch Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("ambisonic_order", "HOA Spatial Order", 1, 5, 4, "order", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("in_phase_weighting", "In-Phase Max-rE Weighting", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
        ),
        "Hoa5BinauralSpatializer" => Box::new(
            GenericDspNodeUi::new("Hoa5BinauralSpatializer", "5th-Order Ambisonic Binaural Decoder with SOFA HRTF", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("sofa_profile", "SOFA HRIR Profile", vec!["KEMAR High-Precision".into(), "Genelec Aural ID".into(), "SADIE II Dummy".into(), "IRCAM Listen Subject 1002".into()], 0, (101, 84, 171)))
                .with_param(DspParamDescriptor::new_enum("binaural_rendering_mode", "Binaural Decoder Kernel", vec!["Dual-Parabolic SVD".into(), "Spherical Harmonics Least-Squares".into(), "Diffuse Field Equalized".into()], 1, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("nearfield_compensation", "Nearfield Bass Compensation", 0.0, 12.0, 2.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("itd_emphasis", "Psychoacoustic ITD Emphasis", 0.5, 2.0, 1.0, "x", (0, 230, 118)))
        ),
        "Mpegh3DSpatializer" => Box::new(
            GenericDspNodeUi::new("Mpegh3DSpatializer", "MPEG-H 3D Immersive Audio Channel Bed Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("bed_layout", "MPEG-H Target Bed", vec!["5.1.4 Surround".into(), "7.1.4 Immersive".into(), "9.1.6 3D Master".into(), "22.2 Hi-Vision".into()], 1, (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("elevation_gain", "Overhead Layer Trim", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("object_spread", "Object Spherical Divergence", 0.0, 100.0, 20.0, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("lfe_gain", "LFE Subwoofer Level", -24.0, 6.0, 0.0, "dB", (255, 61, 113)))
        ),
        "MpeghTrajectorySpatializer" => Box::new(
            GenericDspNodeUi::new("MpeghTrajectorySpatializer", "3D Trajectory Bezier Spline Audio Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("trajectory_speed_bpm", "Spline Trajectory Speed", 1.0, 240.0, 60.0, "BPM", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("spline_tension", "Bezier Spline Tension", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("path_radius_m", "Orbital Path Radius", 0.5, 20.0, 3.5, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("elevation_range_m", "Vertical Elevation Sweep", 0.0, 10.0, 2.0, "m", (0, 230, 118)))
        ),
        "WfsArraySpatializerNode" => Box::new(
            GenericDspNodeUi::new("WfsArraySpatializerNode", "Wave Field Synthesis Multi-Transducer Array Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_int("array_elements_count", "WFS Transducer Count", 16, 128, 64, "speakers", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("speaker_spacing_cm", "Transducer Element Pitch", 2.0, 30.0, 8.0, "cm", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("focused_source_depth_m", "Focused Virtual Depth", -5.0, 15.0, 2.5, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("spatial_aliasing_cut", "Spatial Aliasing Lowpass", 1000.0, 15000.0, 4500.0, "Hz", (0, 230, 118)))
        ),
        "DiffractivePropagationNode" => Box::new(
            GenericDspNodeUi::new("DiffractivePropagationNode", "Acoustic Obstacle Diffraction & Shadow Zone Simulator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("obstacle_width_m", "Obstacle Barrier Width", 0.2, 20.0, 3.0, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("diffraction_angle_deg", "Bending Diffraction Angle", 0.0, 180.0, 45.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("shadow_zone_lowpass_hz", "Shadow Zone High Cut", 200.0, 10000.0, 1200.0, "Hz", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("edge_scattering_gain", "Edge Scattering Amplitude", 0.0, 1.0, 0.35, "%", (0, 230, 118)))
        ),
        "BinauralPannerNode" => Box::new(
            GenericDspNodeUi::new("BinauralPannerNode", "Precision Interaural Time & Level Difference 3D Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("itd_us", "Interaural Time Difference", -700.0, 700.0, 0.0, "us", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("ild_db", "Interaural Level Difference", -24.0, 24.0, 0.0, "dB", (0, 230, 118)))
        ),
        "WavefrontReflectionNode" => Box::new(
            GenericDspNodeUi::new("WavefrontReflectionNode", "Early Specular & Diffuse Wavefront Room Reflection Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("room_dimensions_xyz", "Room Scale Multiplier", 0.5, 5.0, 1.0, "x", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("wall_absorption_coeff", "Boundary Absorption", 0.05, 0.95, 0.2, "", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("specular_order", "Ray Image Source Order", 1, 8, 4, "orders", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("diffuse_scatter_mix", "Diffuse Lambertian Scatter", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
        ),
        "MultibandSpatialNode" => Box::new(
            GenericDspNodeUi::new("MultibandSpatialNode", "Tri-Band Frequency-Dependent Stereo & Spatial Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("low_band_pan", "Low Band Panning (<200Hz)", -1.0, 1.0, 0.0, "pan", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mid_band_pan", "Mid Band Panning", -1.0, 1.0, 0.25, "pan", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_band_pan", "High Band Panning (>4kHz)", -1.0, 1.0, -0.3, "pan", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_log("crossover_freq_hz", "Mid-High Crossover Point", 500.0, 8000.0, 2500.0, "Hz", (0, 230, 118)))
        ),
        "AtmosphericDensityNode" => Box::new(
            GenericDspNodeUi::new("AtmosphericDensityNode", "Atmospheric Gas Composition & Speed of Sound Simulator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambient_pressure_kpa", "Atmospheric Pressure", 1.0, 1000.0, 101.3, "kPa", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("temperature_celsius", "Ambient Temperature", -100.0, 200.0, 20.0, "degC", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("helium_fraction_ratio", "Helium Gas Ratio (Speed Boost)", 0.0, 1.0, 0.0, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("co2_fraction_ratio", "CO2 Gas Ratio (Speed Drop)", 0.0, 1.0, 0.0, "%", (0, 230, 118)))
        ),
        "SpeakerCalibrationMatrix" => Box::new(
            GenericDspNodeUi::new("SpeakerCalibrationMatrix", "Studio Monitor Layout Calibration Matrix", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("num_channels", "Speaker Channel Count", 2.0, 16.0, 8.0, "ch", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("master_trim_db", "Master Alignment Trim", -24.0, 12.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("delay_compensation_ms", "Inter-Channel Delay Align", 0.0, 50.0, 0.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_enum("target_curve", "Target EQ Curve", vec!["Flat Studio".into(), "Harman Target".into(), "B&K 1974 House".into(), "Diffuse Field".into()], 0, (56, 189, 248)))
        ),
        "AmbisonicsDecoder3D" => Box::new(
            GenericDspNodeUi::new("AmbisonicsDecoder3D", "Higher-Order Ambisonics (HOA) Spherical Decoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambisonic_order", "Ambisonic Decoding Order", 1.0, 5.0, 3.0, "ord", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("target_layout", "Target Speaker Layout", vec!["Binaural HRTF".into(), "5.1 Surround".into(), "7.1.4 Dolby Atmos".into(), "22.2 NHK Immersive".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("decode_mode", "Energy Optimization Mode", vec!["In-Phase".into(), "Max-rE".into(), "Basic Energy Preserving".into()], 1, (99, 102, 241)))
                .with_param(DspParamDescriptor::new_log("lfe_sub_cutoff", "LFE Sub Crossover", 40.0, 160.0, 80.0, "Hz", (239, 68, 68)))
        ),
        "AmbisonicsEncoder3D" => Box::new(
            GenericDspNodeUi::new("AmbisonicsEncoder3D", "Higher-Order Ambisonics (HOA) Spherical Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth", "Spherical Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation", "Spherical Elevation Angle", -90.0, 90.0, 0.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("distance", "Source Radial Distance", 0.1, 50.0, 1.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("encoding_order", "HOA Spherical Harmonic Order", 1.0, 5.0, 3.0, "ord", (245, 158, 11)))
        ),
        "PartitionedBinauralHrtfConvolver" => Box::new(
            GenericDspNodeUi::new("PartitionedBinauralHrtfConvolver", "Zero-Latency Partitioned FFT Binaural Convolver", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("hrtf_distance", "Virtual Source Distance", 0.2, 20.0, 1.2, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("partition_size", "FFT Partition Block Size", vec!["32 Samples (Sub-1ms)".into(), "64 Samples (Ultra-low)".into(), "128 Samples (Balanced)".into(), "256 Samples (Economy)".into()], 1, (245, 158, 11)))
        ),
        "AcousticHologramFilter" => Box::new(
            GenericDspNodeUi::new("AcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Wavefield Filter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("num_speakers", "Loudspeaker Count", 4.0, 128.0, 16.0, "spk", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("array_curvature", "Array Curvature Radius", 1.0, 20.0, 5.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("distance_attenuation", "Distance Falloff Exponent", 0.1, 2.0, 0.5, "p", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("focus_mode", "Wavefront Curvature Mode", vec!["Convergent Focal Point".into(), "Divergent Plane Wave".into(), "Focused Point Source".into(), "Curved Acoustic Lens".into()], 0, (99, 102, 241)))
        ),
        "HyperDimensionalHrtfLoader" => Box::new(
            GenericDspNodeUi::new("HyperDimensionalHrtfLoader", "Hyper-Dimensional Spatial HRTF Dataset Interpolator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Spherical Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Spherical Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("distance_m", "Source Radial Distance", 0.2, 10.0, 1.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pinna_morph", "Anthropometric Pinna Scale", 0.5, 2.0, 1.0, "x", (99, 102, 241)))
        ),
        "AcousticCloakingSpatializerNode" => Box::new(
            GenericDspNodeUi::new("AcousticCloakingSpatializerNode", "Acoustic Cloaking Spatializer & Soundfield Invisibility Zone", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("inner_radius_m", "Inner Cloak Boundary", 0.1, 10.0, 1.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("outer_radius_m", "Outer Cloak Shell", 0.2, 20.0, 2.5, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cloaking_factor", "Cloaking Attenuation Factor", 0.0, 1.0, 0.95, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("cloak_profile", "Metamaterial Gradient Profile", vec!["Linear Anisotropic".into(), "Exponential Conformal".into(), "Layered Acoustic Pentamode".into(), "Lattice Phononic Crystal".into()], 0, (99, 102, 241)))
        ),
        "SurroundStemSplitterBedObject" => Box::new(
            GenericDspNodeUi::new("SurroundStemSplitterBedObject", "7.1.4 Dolby Atmos & MPEG-H Bed & Dynamic Object Stem Splitter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("object_cluster_radius", "Object Cluster Radius", 0.1, 10.0, 1.5, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spatial_divergence", "Spatial Divergence", 0.0, 1.0, 0.35, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("height_gain_trim_db", "Height Bed Gain Trim", -12.0, 6.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("binaural_bed_render", "Binaural Headphone Monitor", true, (16, 185, 129)))
        ),
        "HeadTrackerReceiver" => Box::new(
            GenericDspNodeUi::new("HeadTrackerReceiver", "6-DOF OSC / BLE IMU Binaural Head-Tracking Receiver", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("yaw_deg", "Head Yaw Rotation", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_deg", "Head Pitch Tilt", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("roll_deg", "Head Roll Angle", -180.0, 180.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("drift_correction_tau", "Gyro Drift Time Constant", 0.1, 10.0, 2.5, "s", (16, 185, 129)))
        ),
        "SpatialAutomationEngine" => Box::new(
            GenericDspNodeUi::new("SpatialAutomationEngine", "Spatial Trajectory Path Interpolator & Dynamic 3D Automation Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_log("orbit_radius_m", "Trajectory Orbit Radius", 0.2, 25.0, 3.5, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("orbit_speed_cycles", "Orbit Speed Per Beat", 0.05, 10.0, 0.5, "cyc/b", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("orbit_tilt_deg", "Orbit Plane Inclination", -90.0, 90.0, 15.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("spline_smooth_easing", "Catmull-Rom Spline Easing", true, (16, 185, 129)))
        ),
        "SpatialBus" => Box::new(
            GenericDspNodeUi::new("SpatialBus", "Real-Time 64-Channel 3D Atomic Coordinate Coordination Bus", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("cartesian_x", "Source Cartesian X Position", -20.0, 20.0, 0.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cartesian_y", "Source Cartesian Y Position", -20.0, 20.0, 2.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cartesian_z", "Source Cartesian Z Position", -10.0, 10.0, 0.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("atomic_slot_lockless", "Lockless Double-Buffer Sync", true, (16, 185, 129)))
        ),
        "SpatialObjectAutomation" => Box::new(
            GenericDspNodeUi::new("SpatialObjectAutomation", "3D Spatial Audio Vector Trajectory Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("distance_m", "Radial Distance", 0.1, 100.0, 2.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("orbit_speed_hz", "Orbit Angular Velocity", 0.01, 10.0, 0.25, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("doppler_tracking", "Acoustic Doppler Shift", true, (16, 185, 129)))
        ),
        "ImageReflection" => Box::new(
            GenericDspNodeUi::new("ImageReflection", "Shoebox Room Geometric Acoustic Image-Source Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("room_length_m", "Room Length", 2.0, 50.0, 10.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("room_width_m", "Room Width", 2.0, 40.0, 7.5, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("room_height_m", "Room Ceiling Height", 2.0, 20.0, 3.8, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("wall_absorption_coeff", "Wall Absorption Alpha", 0.05, 0.95, 0.25, "alpha", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("specular_diffusion_mix", "Diffusion Blend", 0.0, 1.0, 0.30, "diff", (16, 185, 129)))
        ),
        "StereoPanner" => Box::new(
            GenericDspNodeUi::new("StereoPanner", "Equal-Power Stereo Coordinate Panner & Balance Trimmer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("pan_position", "Pan Coordinate", -1.0, 1.0, 0.0, "pan", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereo_width", "Stereo Image Width", 0.0, 2.0, 1.0, "width", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("center_attenuation_db", "Center Attenuation", -6.0, 0.0, -3.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("balance_trim_db", "Fine Balance Trim", -12.0, 12.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("phase_invert_right", "Invert Right Polarity", false, (16, 185, 129)))
        ),
        "StereoWidth" => Box::new(
            GenericDspNodeUi::new("StereoWidth", "Psychoacoustic Mid/Side Stereo Width & Haas Expander", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("width", "Stereo Field Width", 0.0, 2.0, 1.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereo_balance", "Left / Right Balance", -1.0, 1.0, 0.0, "bal", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("side_hpf_hz", "Bass Mono Filter Cutoff", 20.0, 1000.0, 120.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("mono_correlation_threshold", "Phase Safety Guard", 0.0, 1.0, 0.20, "r", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dry_wet_mix", "Stereo Effect Blend", 0.0, 1.0, 1.0, "%", (16, 185, 129)))
        ),
        "SpatialTrajectoryTrack" => Box::new(
            GenericDspNodeUi::new("SpatialTrajectoryTrack", "Ambisonics 3D Automated Trajectory Orbit Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("orbit_speed_hz", "Orbit Revolution Rate", 0.01, 10.0, 0.25, "Hz", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("orbit_radius_m", "Soundfield Orbit Radius", 0.5, 20.0, 4.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("trajectory_shape", "Spatial Trajectory Path", vec!["Circle XY".into(), "Vertical Spiral XYZ".into(), "Figure-8 Lissajous".into(), "Chaotic Lorenz".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_deg", "Orbit Center Elevation", -90.0, 90.0, 15.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("doppler_intensity", "Doppler Pitch Warp Ratio", 0.0, 1.0, 0.35, "%", (168, 85, 247)))
        ),
        "Hoa4Spatializer" => Box::new(
            GenericDspNodeUi::new("Hoa4Spatializer", "Higher-Order Ambisonics 4th-Order (25-Ch) 3D Spatial Energy HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("hoa_profile", "Ambisonics HOA4 Decoding Profile", vec!["25-Ch HOA4 Sphere".into(), "Binaural HOA4 HRIR".into(), "22.2 Broadcast".into(), "7.1.4 Dome Decode".into(), "Energy-Max rE".into()], 0, (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Soundfield Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Soundfield Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("distance_m", "3D Radial Distance Radius", 0.5, 15.0, 2.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diffuse_energy_ratio", "Diffuse-Field Ambient Energy Ratio", 0.0, 1.0, 0.20, "%", (168, 85, 247)))
        ),
        "MpeghSpatializer" => Box::new(
            GenericDspNodeUi::new("MpeghSpatializer", "Broadcast MPEG-H 3D Audio Multi-Object Spatial Bus HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("mpegh_format", "MPEG-H Target Output Layout", vec!["MPEG-H 7.1.4".into(), "5.1 Surround".into(), "22.2 NHK Dome".into(), "SOFA Binaural".into(), "Dynamic 6-DoF Objects".into()], 0, (14, 165, 233)))
                .with_param(DspParamDescriptor::new_enum("hrtf_profile", "Binaural Personalized HRTF Profile", vec!["KEMAR Standard".into(), "Genelec Aural".into(), "Analytical Spherical".into(), "Photogrammetry 3D Mesh".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Spatial Object Azimuth Angle", -180.0, 180.0, 30.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Spatial Object Elevation Angle", -90.0, 90.0, 15.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("distance_m", "Spatial Object Distance", 0.1, 10.0, 2.0, "m", (168, 85, 247)))
        ),
        "SpatialIrConfig" => Box::new(
            GenericDspNodeUi::new("SpatialIrConfig", "3D Spatial Impulse Response Acoustic Environment HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_rotation_deg", "Acoustic Sphere Azimuth Rotation", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_tilt_deg", "Acoustic Sphere Elevation Tilt Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("direct_to_reverberant_ratio_db", "Direct Path to Diffuse Field Ratio", -30.0, 30.0, 6.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spatial_width_percent", "Ambisonic B-Format Spatial Spread", 0.0, 200.0, 100.0, "%", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("room_absorption_hf", "Air Viscosity & Surface High-Freq Absorption", 0.1, 1.0, 0.65, "%", (34, 197, 94)))
        ),
        "PeerMeshNetwork" => Box::new(
            GenericDspNodeUi::new("PeerMeshNetwork", "Collaborative WebRTC Peer Mesh Network HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("max_mesh_peers_limit", "Maximum Connected Collaborative Peer Nodes", 2.0, 16.0, 6.0, "peers", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("jitter_buffer_target_ms", "Adaptive Dynamic Audio Jitter Buffer Target", 10.0, 250.0, 40.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("packet_loss_concealment", "Audio Packet Loss Concealment (PLC) Mode", vec!["Silence on Loss".into(), "Zero-Order Hold".into(), "Waveform Linear Fade".into(), "Neural Spectral Inpainting".into()], 3, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("peer_audio_compression_mode", "Collaborative Stream Audio Codec Mode", vec!["Opus High-Fidelity 128kbps".into(), "Opus Studio 256kbps".into(), "FLAC Ultra-Low-Loss".into(), "PCM 16-bit Uncompressed".into()], 1, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("end_to_end_encryption_enable", "DTLS-SRTP End-to-End Stream Encryption", true, (34, 197, 94)))
        ),
        "AdmSpatialBwfExporter" => Box::new(
            GenericDspNodeUi::new("AdmSpatialBwfExporter", "ITU-R BS.2076 ADM Dolby Atmos Spatial BWF Exporter HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("adm_channel_configuration", "Spatial Speaker Bed & Object Profile", vec!["Dolby Atmos 7.1.4".into(), "Binaural Spatial BWF".into(), "Ambisonics 3rd Order (16ch)".into(), "NHK 22.2 Spatial".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("speaker_layout_bed_channels", "Fixed Speaker Bed Channel Count", 2.0, 12.0, 8.0, "ch", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spatial_audio_objects_count", "Dynamic 3D Positional Audio Objects", 1.0, 128.0, 64.0, "objs", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("metadata_refresh_rate_hz", "Positional Coordinate Refresh Cadence", 24.0, 120.0, 60.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("embed_itu_r_bs2076_xml", "Embed Compliant ITU-R BS.2076 XML Chunks", true, (34, 197, 94)))
        ),
        "BinauralHrtfInterpolator" => Box::new(
            GenericDspNodeUi::new("BinauralHrtfInterpolator", "Continuous Spherical Harmonic 3D Binaural HRTF Interpolator HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_deg", "Horizontal Virtual Sound Source Azimuth Coordinate", -180.0, 180.0, 45.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_deg", "Vertical Virtual Sound Source Elevation Angle", -90.0, 90.0, 15.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_distance_meters", "Listener-to-Source Distance Inverse-Square Attenuation", 0.2, 25.0, 2.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("hrtf_dataset_model", "Acoustic Anatomical HRTF Impulse Response Dataset", vec!["KEMAR Dummy Head Studio".into(), "CIPIC High-Resolution Grid".into(), "LISTEN Individualized Subject".into(), "Spherical Approximation (Low CPU)".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pinna_spectral_notch_depth", "Anatomical Pinna Notch Reflection Intensity", 0.0, 1.0, 0.85, "%", (34, 197, 94)))
        ),
        "SurroundDownmixer" => Box::new(
            GenericDspNodeUi::new("SurroundDownmixer", "ITU-R BS.775 7.1.4 to Stereo Downmixing & Fold-Down Matrix HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("center_channel_attenuation_db", "Center Dialogue Channel Fold-Down Attenuation", -6.0, 0.0, -3.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("surround_channel_attenuation_db", "Left/Right Surround and Height Channel Fold-Down Attenuation", -6.0, 0.0, -3.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lfe_sub_channel_gain_db", "Low-Frequency Effects (LFE) Subwoofer Blend Level", -24.0, 0.0, -10.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("surround_phase_shift_90deg", "Apply 90° Hilbert Phase Shift to Surrounds for Mono Compatibility", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("downmix_output_mode", "Output Audio Channel Matrix & Downmix Profile", vec!["Stereo Standard (Lo/Ro)".into(), "Pro Logic Compatible (Lt/Rt)".into(), "Direct Mono Sum (M)".into(), "Binaural Virtual Surround".into()], 0, (34, 197, 94)))
        ),
        "AtmosphericDensity" => Box::new(
            GenericDspNodeUi::new("AtmosphericDensity", "Acoustic Humidity, Barometric Air Damping & Propagation HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambient_temperature_celsius", "Ambient Environment Air Temperature", -20.0, 50.0, 20.0, "C", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("relative_humidity_pct", "Atmospheric Relative Humidity Percentage", 5.0, 100.0, 50.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("barometric_pressure_kpa", "Atmospheric Static Barometric Air Pressure", 70.0, 115.0, 101.3, "kPa", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("propagation_distance_meters", "Acoustic Path Propagation Distance in Air", 1.0, 500.0, 25.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("doppler_wind_speed_m_per_s", "Vector Wind Speed Along Acoustic Propagation Axis", -30.0, 30.0, 0.0, "m/s", (34, 197, 94)))
        ),
        "AcousticLevitationTrap" => Box::new(
            GenericDspNodeUi::new("AcousticLevitationTrap", "40kHz Ultrasonic Phased-Array Acoustic Levitation Trap HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ultrasonic_carrier_freq_khz", "Ultrasonic Transducer Array Carrier Frequency", 20.0, 80.0, 40.0, "kHz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transducer_array_power_watts", "Total Electrical Acoustic Array Driving Power", 0.5, 100.0, 24.0, "W", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("trap_focal_point_x_mm", "3D Standing Wave Node Focal Point X Coordinate", -100.0, 100.0, 0.0, "mm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("trap_focal_point_y_mm", "3D Standing Wave Node Focal Point Y Coordinate", -100.0, 100.0, 0.0, "mm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("trap_focal_point_z_height_mm", "3D Standing Wave Node Levitation Height Z", 0.0, 200.0, 50.0, "mm", (34, 197, 94)))
        ),
        "MultichannelAudioBuffer" => Box::new(
            GenericDspNodeUi::new("MultichannelAudioBuffer", "N-Channel Immersive & Surround Bus Audio Buffer HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_enum("channels", "Discrete Audio Channel Layout Bus", vec!["Stereo (2)".into(), "Quadraphonic (4)".into(), "5.1 Surround (6)".into(), "7.1.4 Atmos (12)".into(), "22.2 Immersive (24)".into()], 2, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("frame_capacity", "Per-Channel Frame Allocation Length", 64.0, 2048.0, 512.0, "frames", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("interleaved", "Channel Sample Interleaving Memory Layout", 0.0, 1.0, 0.0, "bool", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("master_gain", "Surround Master Bus Gain Multiplier", 0.0, 2.0, 1.0, "gain", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lfe_sub_cutoff_hz", "Low-Frequency Effects Subwoofer Crossover", 40.0, 200.0, 120.0, "Hz", (34, 197, 94)))
        ),
        "SpatialWaypoint" => Box::new(
            GenericDspNodeUi::new("SpatialWaypoint", "3D Spatial Trajectory Coordinate Node HUD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("coord_x", "Cartesian Sound Source X Position (Left-Right)", -50.0, 50.0, 0.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("coord_y", "Cartesian Sound Source Y Position (Front-Back)", -50.0, 50.0, 5.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("coord_z", "Cartesian Sound Source Z Position (Height-Elevation)", -20.0, 20.0, 0.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("curve_tension", "Hermite Spline Trajectory Curve Tension", 0.0, 1.0, 0.5, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("arrival_time_ms", "Waypoint Trajectory Arrival Timestamp", 0.0, 60000.0, 2500.0, "ms", (34, 197, 94)))
        ),
        "AtomicSpatialSlot" => Box::new(
            GenericDspNodeUi::new("AtomicSpatialSlot", "Lock-Free Atomic 3D Spatial Audio Coordinate Slot", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("distance_m", "Source Distance", 0.1, 100.0, 1.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spread_factor", "Sound Source Directivity Spread", 0.0, 1.0, 0.2, "spread", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("doppler_factor", "Relativistic Doppler Pitch Factor", 0.0, 2.0, 1.0, "x", (34, 197, 94)))
        ),
        "Position3D" => Box::new(
            GenericDspNodeUi::new("Position3D", "Cartesian 3D Spatial Position & Distance Coordinates", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("pos_x", "Lateral Right/Left X Position", -50.0, 50.0, 0.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pos_y", "Vertical Elevation Y Position", -50.0, 50.0, 0.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pos_z", "Depth Front/Back Z Position", -50.0, 50.0, 2.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_absorption_hf_db", "High-Frequency Atmospheric Damping", -18.0, 0.0, -2.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("inverse_square_attenuation", "Enable Physical 1/R Distance Attenuation", true, (34, 197, 94)))
        ),
        "SpatialImpulseResponse" => Box::new(
            GenericDspNodeUi::new("SpatialImpulseResponse", "Binaural Room Acoustic Spatial Transfer Impulse", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sample_rate", "Acoustic Sampling Rate", 22050.0, 192000.0, 48000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("rt60_sec", "RT60 Reverberation Decay Time", 0.05, 15.0, 1.8, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("direct_delay_ms", "Direct Path Arrival Pre-Delay", 0.0, 100.0, 4.5, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("elevation_damping_db", "Spherical Elevation HRIR Damping", -18.0, 0.0, -3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("air_absorption_hf", "Air Molecular High Damping", true, (34, 197, 94)))
        ),
        "SpatialVector3D" => Box::new(
            GenericDspNodeUi::new("SpatialVector3D", "Spherical & Cartesian 3D Spatial Audio Vector", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_rad", "Horizontal Azimuth Angle", -3.14159, 3.14159, 0.0, "rad", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_rad", "Vertical Elevation Angle", -1.5708, 1.5708, 0.0, "rad", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("distance_m", "Listener Radial Distance Radius", 0.1, 100.0, 2.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("doppler_factor", "Acoustic Doppler Pitch Shift Ratio", 0.0, 2.0, 1.0, "x", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spread_deg", "Virtual Sound Source Spread Aperture", 0.0, 180.0, 30.0, "deg", (34, 197, 94)))
        ),
        "RoomAcousticModel" => Box::new(
            GenericDspNodeUi::new("RoomAcousticModel", "3D Spatial Acoustic Room Geometry & Early Reflections", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("simulation_acoustic_topology", "Room Acoustics Simulation Topology", 0.0, 3.0, 3.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("room_length_meters", "Acoustic Chamber Length Dimension", 2.0, 50.0, 12.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("room_width_meters", "Acoustic Chamber Width Dimension", 2.0, 40.0, 8.5, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("room_height_meters", "Acoustic Chamber Ceiling Height", 2.0, 20.0, 5.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_absorption_humidity_pct", "Relative Ambient Air Humidity for HF Loss", 10.0, 90.0, 45.0, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_doppler_moving_source", "Calculate Moving Source Doppler Acoustic Shift", true, (14, 165, 233)))
        ),
        "WallAcousticMaterial" => Box::new(
            GenericDspNodeUi::new("WallAcousticMaterial", "Acoustic Boundary Surface Material & Scattering Coefficient", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("boundary_wall_material", "Surface Construction Material & Absorption", 0.0, 4.0, 2.0, "mat", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("scattering_diffusion_coefficient", "Surface Lambertian Roughness Diffusion", 0.0, 1.0, 0.4, "coeff", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_freq_absorption_125hz", "125 Hz Sub-Bass Absorption Factor", 0.02, 0.8, 0.15, "alpha", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("mid_freq_absorption_1khz", "1 kHz Mid-Frequency Absorption Factor", 0.05, 0.95, 0.35, "alpha", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_freq_absorption_8khz", "8 kHz High-Frequency Absorption Factor", 0.1, 0.99, 0.65, "alpha", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("model_diaphragmatic_bass_trapping", "Simulate Panel Flexure Bass Trapping", true, (14, 165, 233)))
        ),
        "TrajectoryPathType" => Box::new(
            GenericDspNodeUi::new("TrajectoryPathType", "3D Spatial Trajectory Path Generator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("path_mode", "Parametric Trajectory Geometry Profile", 0.0, 4.0, 0.0, "path", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("orbit_radius_m", "Orbital Radius from Listener Center", 0.5, 50.0, 3.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("speed_cycles_per_beat", "Trajectory Orbital Velocity per Beat", 0.01, 8.0, 0.25, "cyc/b", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("elevation_tilt_rad", "Spatial Orbital Plane Elevation Tilt", -3.14, 3.14, 0.0, "rad", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("orbit_eccentricity", "Elliptical Orbit Eccentricity Ratio", 0.0, 0.99, 0.0, "ecc", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_catmull_rom_smoothing", "Engage Cubic Catmull-Rom Spline Easing", true, (14, 165, 233)))
        ),
        "SpatialInterpolationCurve" => Box::new(
            GenericDspNodeUi::new("SpatialInterpolationCurve", "Spatial 3D Trajectory Keyframe Interpolation Curve", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("curve_type", "Spatial Keyframe Curve Transition Algorithm", 0.0, 3.0, 1.0, "curve", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spline_tension", "Catmull-Rom Hermite Spline Tension Factor", 0.0, 1.0, 0.5, "tension", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("smoothing_window_ms", "Position Smoothing Slew Window Duration", 0.1, 100.0, 10.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("azimuth_deadzone_rad", "Azimuth Micro-Movement Noise Deadzone", 0.0, 0.2, 0.01, "rad", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("distance_damping_ratio", "Radial Distance Slew Velocity Damping", 0.1, 2.0, 1.0, "damp", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_elevation_clamping", "Clamp Elevation Vector to Hemispherical Limits", true, (14, 165, 233)))
        ),
        "WallMaterial" => Box::new(
            GenericDspNodeUi::new("WallMaterial", "Acoustic Space Boundary Wall Material & Absorption Spectrum", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("material_type", "Acoustic Boundary Surface Material", 0.0, 5.0, 2.0, "mat", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("absorption_low", "Low-Frequency Absorption Coefficient (<250 Hz)", 0.01, 0.99, 0.15, "coeff", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("absorption_mid", "Mid-Frequency Absorption Coefficient (250-2000 Hz)", 0.01, 0.99, 0.55, "coeff", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("absorption_high", "High-Frequency Absorption Coefficient (>2000 Hz)", 0.01, 0.99, 0.85, "coeff", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("scattering_roughness", "Diffuse Lambertian Surface Scattering Ratio", 0.0, 1.0, 0.42, "ratio", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("transmission_loss_db", "Through-Wall Acoustic Sound Transmission Loss", 5.0, 80.0, 45.0, "dB", (52, 211, 153)))
        ),
        "CliAdmSpatialExporter" => Box::new(
            GenericDspNodeUi::new("CliAdmSpatialExporter", "ITU-R BS.2076 ADM & Dolby Atmos Broadcast Audio Exporter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("adm_channel_configuration", "ADM Speaker Bed & Object Configuration", 0.0, 4.0, 0.0, "conf", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spatial_trajectory_fps", "Object Vector Metadata Trajectory Frame Rate", 24.0, 120.0, 60.0, "fps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("binaural_distance_attenuation_db", "Binaural Head Shadow & Distance Rolloff", 0.0, 24.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enforce_ebu_r128_loudness", "Strict EBU R128 (-23 LUFS) Loudness Compliance Check", true, (52, 211, 153)))
                .with_param(DspParamDescriptor::new_linear("lfe_crossover_hz", "Subwoofer LFE Channel Low-Pass Crossover", 40.0, 160.0, 80.0, "Hz", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_bool("write_xml_metadata_chunk", "Embed Complete BS.2076 axml and chna WAV Metadata", true, (239, 68, 68)))
        ),
        "BinauralHeadphoneEqualizer" => Box::new(
            GenericDspNodeUi::new("BinauralHeadphoneEqualizer", "Diffuse-Field & Harman Curve Headphone Calibration Filter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("target_equalization_curve", "Target Acoustic EQ Target", 0.0, 3.0, 0.0, "tgt", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossfeed_frequency_cutoff_hz", "Crossfeed Acoustic Cutoff", 300.0, 1200.0, 650.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossfeed_feed_gain_db", "Crossfeed Bleed Gain", -18.0, -3.0, -9.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("inter_aural_time_delay_us", "Interaural Head Delay (ITD)", 100.0, 450.0, 280.0, "us", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("bass_shelf_boost_db", "Target Bass Shelf Boost", 0.0, 12.0, 4.5, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("high_treble_air_damping", "Smooth High-Treble Air Damping", true, (14, 165, 233)))
        ),
        "TransientSplayStereoWidener" => Box::new(
            GenericDspNodeUi::new("TransientSplayStereoWidener", "Transient Decorrelation & Haas Effect Stereo Imager", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("stereo_spread_width_percentage", "Stereo Spatial Spread Width", 0.0, 200.0, 120.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("haas_delay_offset_ms", "Haas Micro-Delay Offset", 0.1, 25.0, 7.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_detector_sensitivity", "Transient Trigger Sensitivity", 0.1, 1.0, 0.6, "sens", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("low_frequency_mono_crossover_hz", "Bass Mono Crossover Frequency", 60.0, 350.0, 150.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("decorrelation_filter_stages", "Allpass Phase Decorrelation Stages", 0.0, 2.0, 1.0, "stages", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("strict_mono_compatibility_guard", "Strict Mono Cancellation Guard", true, (14, 165, 233)))
        ),
        "MidSideSpectralHarmonizer" => Box::new(
            GenericDspNodeUi::new("MidSideSpectralHarmonizer", "Mid/Side Spectral Harmonic Enhancer & Spatial Stereo Widener", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("mid_tube_saturation_drive", "Mid Channel Tube Saturation Drive", 0.0, 1.0, 0.25, "drive", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("side_air_exciter_gain_db", "Side Channel Air Harmonic Exciter", -12.0, 12.0, 2.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_spread_width_ratio", "Global Stereo Field Width", 0.5, 2.5, 1.25, "width", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crossover_bass_mono_hz", "Bass Mono Fold Crossover", 60.0, 400.0, 140.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("harmonic_odd_even_balance", "Harmonic Order Odd/Even Balance", -1.0, 1.0, 0.0, "bal", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("mono_phase_guard_enabled", "Strict Mono Cancellation Guard", true, (14, 165, 233)))
        ),
        "BinauralAmbisonicDecoder" => Box::new(
            GenericDspNodeUi::new("BinauralAmbisonicDecoder", "Higher-Order Ambisonics (HOA 7th-Order) to Binaural HRTF Renderer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambisonic_order_order", "Ambisonic Spherical Order", 0.0, 3.0, 1.0, "order", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("hrtf_profile_database", "HRTF Head Profile Database", 0.0, 3.0, 0.0, "hrtf", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("head_tracking_yaw_deg", "Head-Tracking Horizontal Yaw", -180.0, 180.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("head_tracking_pitch_deg", "Head-Tracking Vertical Pitch", -90.0, 90.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("near_field_compensation_distance", "Near-Field Parallax Distance", 0.2, 10.0, 1.5, "m", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("diffuse_field_sound_correction", "Diffuse-Field Equalization Correction", true, (14, 165, 233)))
        ),
        "DolbyAtmosBedDownmixer" => Box::new(
            GenericDspNodeUi::new("DolbyAtmosBedDownmixer", "Dolby Atmos 7.1.4 / 9.1.6 Bed Downmixer & ITU Fold-Down Matrix", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("input_bed_format", "Source Immersive Speaker Bed", 0.0, 3.0, 0.0, "bed", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("center_downmix_level_db", "Center Channel Fold-Down Gain", -6.0, 0.0, -3.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("surround_downmix_level_db", "Surround Channel Fold-Down Gain", -9.0, 0.0, -3.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("height_channel_gain_db", "Overhead Height Channels Gain", -12.0, 0.0, -4.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lfe_subwoofer_downmix_mode", "Subwoofer LFE Channel Routing", 0.0, 2.0, 0.0, "lfe", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_itu_r_bs775_gain_normalizer", "ITU-R BS.775 Normalization Coefficient", true, (14, 165, 233)))
        ),
        "AcousticRoomRaytracer" => Box::new(
            GenericDspNodeUi::new("AcousticRoomRaytracer", "Acoustic Boundary Raytracing Engine & Diffuse Wavefront Simulator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_ray_count", "Dispatched Sound Rays Count", 500.0, 20000.0, 4000.0, "rays", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("maximum_reflection_bounces", "Maximum Reflection Bounces Depth", 1.0, 32.0, 8.0, "bounces", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("room_wall_absorption_coefficient", "Boundary Wall Absorption Factor", 0.01, 0.95, 0.25, "alpha", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diffuse_scattering_fraction", "Lambertian Diffuse Scattering Ratio", 0.0, 1.0, 0.35, "scat", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("room_air_humidity_percentage", "Ambient Room Air Humidity", 10.0, 90.0, 45.0, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_edge_diffraction_modeling", "Simulate Obstacle Edge Diffraction", true, (14, 165, 233)))
        ),
        "SpatialBinauralPanner3D" => Box::new(
            GenericDspNodeUi::new("SpatialBinauralPanner3D", "7th-Order Ambisonic Spherical HRTF Binaural Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_degrees", "Horizontal Sound Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_degrees", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_distance_meters", "Acoustic Radial Source Distance", 0.1, 50.0, 2.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pinna_spectral_notch_depth", "Pinna Concha Spectral Notch Depth", 0.0, 1.0, 0.7, "depth", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_dispersion_ratio", "Acoustic Head Shadow Dispersion", 0.1, 1.0, 0.65, "ratio", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("doppler_motion_tracking", "Dynamic Doppler Frequency Shift", true, (14, 165, 233)))
        ),
        "AmbisonicRotatorBinaural" => Box::new(
            GenericDspNodeUi::new("AmbisonicRotatorBinaural", "3D Ambisonic B-Format Yaw/Pitch/Roll Spherical Rotator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("yaw_rotation_degrees", "Horizontal Yaw Rotation Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_tilt_degrees", "Vertical Pitch Tilt Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("roll_bank_degrees", "Sideways Roll Banking Angle", -90.0, 90.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ambisonic_decoding_order", "Ambisonic Spherical Order", 0.0, 2.0, 1.0, "order", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("head_tracker_smoothing_ms", "Head-Tracker Smoothing Constant", 0.0, 50.0, 12.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("near_field_effect_compensation", "Near-Field Bass Distance Compensation", true, (14, 165, 233)))
        ),
        "BinauralNearfieldSimulator" => Box::new(
            GenericDspNodeUi::new("BinauralNearfieldSimulator", "Spherical Head Parallax Nearfield HRTF Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_deg", "Horizontal Azimuth Direction", -180.0, 180.0, 35.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_deg", "Vertical Elevation Direction", -90.0, 90.0, 15.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("radial_distance_meters", "Radial Proximity Distance", 0.05, 3.0, 0.45, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("head_circumference_cm", "Anthropometric Head Circumference", 48.0, 64.0, 56.0, "cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pinna_spectral_contour", "Pinna Notch Acoustic Profile", 0.0, 3.0, 0.0, "idx", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_head_tracking_compensation", "Dynamic Head-Tracking Yaw Lock", false, (14, 165, 233)))
        ),
        "HarmonicStereoSpreadWidener" => Box::new(
            GenericDspNodeUi::new("HarmonicStereoSpreadWidener", "Frequency-Comb Psychoacoustic Stereo Imager & Mono Guard", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("stereo_spread_width_percent", "Stereo Panorama Expansion", 0.0, 250.0, 135.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("comb_filter_spacing_hz", "Comb Interference Frequency Spacing", 50.0, 800.0, 240.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("center_channel_retention", "Phantom Center Anchor Retention", 0.0, 1.0, 0.85, "ret", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("bass_mono_cutoff_frequency_hz", "Bass Mono Collapse Frequency", 40.0, 350.0, 120.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("haas_micro_delay_ms", "Psychoacoustic Haas Delay Offset", 0.5, 25.0, 8.5, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("mono_correlation_safety_guard", "Enforce Mono Correlation Safety", true, (14, 165, 233)))
        ),
        "MultichannelSurroundBedUpmixer" => Box::new(
            GenericDspNodeUi::new("MultichannelSurroundBedUpmixer", "Stereo to 7.1.4 Immersive 3D Audio Bed Upmixer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("center_extraction_divergence", "Center Dialog Extraction Divergence", 0.0, 1.0, 0.65, "div", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("surround_diffuse_decorrelation", "Surround Diffuse Decorrelation", 0.0, 1.0, 0.55, "diff", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("height_layer_level_db", "Top / Height Ceiling Layer Gain", -24.0, 6.0, -4.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lfe_subwoofer_crossover_hz", "LFE Subwoofer Extraction Crossover", 40.0, 160.0, 80.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("surround_bed_target_format", "Speaker Layout Target Bed", 0.0, 3.0, 3.0, "fmt", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("front_to_back_balance", "Front / Rear Surround Energy Balance", -1.0, 1.0, 0.1, "bal", (14, 165, 233)))
        ),
        "DynamicAirAbsorptionAcoustics" => Box::new(
            GenericDspNodeUi::new("DynamicAirAbsorptionAcoustics", "ISO 9613-1 Atmospheric Ultrasonic Damping & Humidity Filter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("propagation_distance_meters", "Acoustic Path Propagation Distance", 1.0, 200.0, 25.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ambient_temperature_celsius", "Ambient Medium Temperature", -10.0, 45.0, 20.0, "C", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("relative_humidity_percent", "Atmospheric Relative Humidity", 10.0, 99.0, 50.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("barometric_pressure_kpa", "Atmospheric Static Pressure", 80.0, 110.0, 101.3, "kPa", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("air_viscosity_relaxation_mode", "Molecular Relaxation Model", 0.0, 3.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_high_frequency_air_roll_off", "Apply Ultrasonic Air Dissipation", true, (14, 165, 233)))
        ),
        "MidSideSpectralStereoImager" => Box::new(
            GenericDspNodeUi::new("MidSideSpectralStereoImager", "3-Band Psychoacoustic Mid/Side Phase Correlation Expander", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("low_band_stereo_width", "Sub-Bass Stereo Field Width", 0.0, 2.0, 0.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_band_stereo_width", "Midrange Body Stereo Field Width", 0.0, 2.0, 1.15, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_band_stereo_width", "Air Band Treble Stereo Width", 0.0, 2.5, 1.35, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("bass_mono_crossover_hz", "Mono Bass Crossover Cutoff Frequency", 40.0, 300.0, 120.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("side_harmonic_presence_db", "Side-Channel Harmonic Presence Boost", -12.0, 12.0, 1.5, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_linear_phase_midside_crossover", "Linear Phase Crossover Filtering", true, (14, 165, 233)))
        ),
        "AmbisonicBFormatDecoder" => Box::new(
            GenericDspNodeUi::new("AmbisonicBFormatDecoder", "Higher-Order Ambisonic (HOA) Spherical Harmonic to Speaker Decoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambisonic_order", "Spherical Harmonic Ambisonic Order", 0.0, 2.0, 1.0, "ord", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("speaker_layout_configuration", "Loudspeaker Target Layout Array", 0.0, 4.0, 2.0, "layout", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spatial_decoding_matrix_mode", "Ambisonic Energy Vector Decoding Mode", 0.0, 2.0, 1.0, "mode", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("distance_compensation_delay_ms", "Speaker Ring Distance Alignment Delay", 0.0, 20.0, 2.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nearfield_distance_compensation_hz", "Nearfield Acoustic Bass Compensation", 20.0, 250.0, 60.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("subwoofer_lfe_crossover_hz", "LFE Bass Management Crossover", 40.0, 160.0, 80.0, "Hz", (14, 165, 233)))
        ),
        "BinauralSphericalHeadModel" => Box::new(
            GenericDspNodeUi::new("BinauralSphericalHeadModel", "Continuous Spherical Head Shadow & Diffraction Simulator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("head_radius_meters", "Listener Cranial Radius Dimension", 0.06, 0.12, 0.0875, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("source_azimuth_deg", "Virtual Source Horizontal Azimuth Angle", -180.0, 180.0, 45.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_elevation_deg", "Virtual Source Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("source_distance_meters", "Virtual Sound Source Distance", 0.2, 50.0, 2.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("woodworth_itd_weight", "Woodworth Analytical Time Delay Weight", 0.5, 1.5, 1.0, "wt", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("torso_shoulder_bounce_mix", "Shoulder & Torso Acoustic Reflection Level", 0.0, 1.0, 0.25, "%", (14, 165, 233)))
        ),
        "BinauralDopplerMotionPanner" => Box::new(
            GenericDspNodeUi::new("BinauralDopplerMotionPanner", "3D Trajectory Doppler-Shifted Binaural Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_deg", "Horizontal Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_deg", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_distance_meters", "Sound Source Distance", 0.1, 100.0, 3.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("trajectory_speed_m_s", "Source Trajectory Velocity", 0.0, 340.0, 15.0, "m/s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("atmospheric_absorption_db", "Atmospheric Ultrasonic Absorption", 0.0, 12.0, 2.5, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_doppler_pitch_shift", "Enable Real-Time Doppler Pitch Modulation", true, (14, 165, 233)))
        ),
        "AmbisonicDiffractionEncoder" => Box::new(
            GenericDspNodeUi::new("AmbisonicDiffractionEncoder", "Higher-Order Ambisonic Spherical Diffraction Encoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("source_azimuth_deg", "Spherical Coordinate Azimuth", -180.0, 180.0, 30.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("source_elevation_deg", "Spherical Coordinate Elevation", -90.0, 90.0, 15.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("acoustic_shadow_diffraction", "Barrier Acoustic Shadow Damping", 0.0, 1.0, 0.6, "diff", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("wall_boundary_reflection_db", "First-Order Specular Reflection Gain", -36.0, 0.0, -12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spherical_spread_radius", "Spherical Source Sound Field Radius", 0.1, 10.0, 1.0, "m", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("distance_nearfield_compensation", "Radial Nearfield Bass Compensation", true, (14, 165, 233)))
        ),
        "MasteringStereoWidthMaximizer" => Box::new(
            GenericDspNodeUi::new("MasteringStereoWidthMaximizer", "Psychoacoustic Stereo Width Maximizer & Mono Bass Guard", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("stereo_spread_width_percent", "Total Panoramic Stereo Width Expansion", 0.0, 250.0, 120.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mono_bass_crossover_hz", "Mono Bass Summing Elliptic Crossover", 30.0, 250.0, 100.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("side_band_eq_tilt_db", "Side Channel High-Shelf Equalizer Tilt", -6.0, 6.0, 1.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("haas_delay_spread_ms", "Haas Precedence Micro-Delay Time", 0.0, 25.0, 6.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_correlation_safety_ceiling", "Minimum Phase Correlation Floor", 0.0, 1.0, 0.35, "corr", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_side_channel_solo", "Solo Isolated Stereo Side Channel (L-R)", false, (14, 165, 233)))
        ),
        "BinauralRoomImpulseSpatializer" => Box::new(
            GenericDspNodeUi::new("BinauralRoomImpulseSpatializer", "Ray-Traced Binaural Room Impulse Response Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_degrees", "Horizontal Azimuth Sound Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_angle_degrees", "Vertical Elevation Sound Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_distance_meters", "Listener Radial Distance", 0.2, 30.0, 2.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("room_wall_absorption_coeff", "Acoustic Wall Boundary Absorption", 0.05, 0.95, 0.35, "abs", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diffuse_reverb_blend", "Late Diffuse Soundfield Reverb Blend", 0.0, 1.0, 0.25, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_interaural_time_delay", "Model Head Shadow & ITD Delay", true, (14, 165, 233)))
        ),
        "MidSidePsychoacousticImager" => Box::new(
            GenericDspNodeUi::new("MidSidePsychoacousticImager", "Frequency-Dependent Mid/Side Psychoacoustic Imager", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("low_band_mono_crossover_hz", "Elliptic Mono Bass Crossover Cutoff", 40.0, 300.0, 110.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_band_stereo_width_percent", "Mid-Band Panorama Stereo Width", 0.0, 200.0, 100.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_band_stereo_width_percent", "High-Frequency Airy Stereo Spread", 0.0, 250.0, 135.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("center_mid_channel_gain_db", "Center Mid (M) Channel Balance", -12.0, 6.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_side_channel_gain_db", "Stereo Side (S) Channel Balance", -12.0, 6.0, 0.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_binaural_crossfeed_filter", "Binaural Interaural Crossfeed Filter", true, (14, 165, 233)))
        ),
        "AmbisonicBinauralDecoderSpatializer" => Box::new(
            GenericDspNodeUi::new("AmbisonicBinauralDecoderSpatializer", "3rd-Order Higher-Order Ambisonics (HOA) to Binaural HRIR Spherical Decoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("soundfield_yaw_rotation_deg", "Spherical Soundfield Yaw Azimuth", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("soundfield_pitch_tilt_deg", "Spherical Soundfield Pitch Tilt", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("soundfield_roll_rotation_deg", "Spherical Soundfield Roll Rotation", -180.0, 180.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("binaural_head_radius_cm", "Acoustic Anthropomorphic Head Radius", 6.0, 12.0, 8.75, "cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diffuse_field_equalization_blend", "Diffuse-Field HRIR Equalization Blend", 0.0, 1.0, 0.5, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_nearfield_compensation", "Near-Field Wavefront Curvature Compensation", true, (14, 165, 233)))
        ),
        "DynamicStereoMicroDelayPanner" => Box::new(
            GenericDspNodeUi::new("DynamicStereoMicroDelayPanner", "Haas-Effect Micro-Delay Spatializer & Psychoacoustic Precedence Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("haas_delay_offset_ms", "Interaural Haas Time Delay Offset", 0.0, 35.0, 12.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("interaural_intensity_diff_db", "Interaural Level Difference (ILD)", -18.0, 18.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("comb_filter_anti_phase_reduction", "Comb Filter Anti-Phase Attenuation", 0.0, 1.0, 0.65, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_pinna_shadow_hz", "Pinna Ear Shadow Cutoff Filter", 2000.0, 16000.0, 6500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("precedence_effect_mode", "Psychoacoustic Spatial Delay Mode", 0.0, 3.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("mono_sum_compatibility_guard", "Mono-Sum Phase Cancellation Guard", true, (14, 165, 233)))
        ),
        "BinauralDistanceRoomRenderer" => Box::new(
            GenericDspNodeUi::new("BinauralDistanceRoomRenderer", "Directivity & Parallax-Aware Binaural Acoustic Distance Modeler", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_source_distance_meters", "Acoustic Source Distance to Listener", 0.2, 50.0, 3.5, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("atmospheric_absorption_hf_damping", "Air Humidity Absorption High Damping", 0.1, 2.0, 1.0, "coef", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("binaural_parallax_azimuth_deg", "Source Azimuth Angle Positioning", -180.0, 180.0, 30.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("initial_delay_gap_ms", "Direct-to-Early Reflection Delay Offset", 0.0, 80.0, 12.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("wall_absorption_coefficient", "Boundary Boundary Absorption Material", 0.05, 0.95, 0.35, "abs", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_early_ground_reflection", "Specular Floor Reflection Simulation", true, (14, 165, 233)))
        ),
        "DolbyAtmos714BedEncoder" => Box::new(
            GenericDspNodeUi::new("DolbyAtmos714BedEncoder", "7.1.4 Immersive Audio Multi-Speaker Height Bus Matrix Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("bed_azimuth_panning_deg", "Horizontal Surround Azimuth Pan", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation_height_spread_deg", "Overhead Height Elevation Angle", 0.0, 90.0, 35.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lfe_subwoofer_crossover_send_db", "LFE Subwoofer Dedicated Send Gain", -60.0, 6.0, -12.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("center_channel_divergence_ratio", "Center Speaker Phantom Divergence", 0.0, 1.0, 0.25, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("surround_decorrelation_width", "Decorrelation Diffuse Surround Width", 0.0, 1.0, 0.5, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_discrete_height_isolation", "Discrete Height Channel Isolation", false, (14, 165, 233)))
        ),
        "AtmosphericDensityAbsorption" => Box::new(
            GenericDspNodeUi::new("AtmosphericDensityAbsorption", "Barometric Pressure & Humidity Acoustic Wavefront Absorption", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("altitude_elevation_meters", "Geographic Altitude Elevation", -500.0, 9000.0, 0.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ambient_temperature_celsius", "Ambient Atmospheric Temperature", -40.0, 50.0, 20.0, "deg C", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("relative_air_humidity_pct", "Relative Atmospheric Humidity", 5.0, 100.0, 50.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("barometric_static_pressure_kpa", "Static Barometric Pressure", 30.0, 110.0, 101.3, "kPa", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("distance_propagation_meters", "Wavefront Distance Traveled", 1.0, 500.0, 25.0, "m", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_molecular_nitrogen_relaxation", "Diatomic Nitrogen Molecular Relaxation", true, (14, 165, 233)))
        ),
        "AcousticCloakingTransformationPanner" => Box::new(
            GenericDspNodeUi::new("AcousticCloakingTransformationPanner", "Transformation Optics Acoustic Cloaking & Wavefront Curvature Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("cloaking_shell_inner_radius_m", "Acoustic Cloak Inner Core Radius", 0.1, 5.0, 0.75, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cloaking_shell_outer_radius_m", "Acoustic Cloak Outer Shell Radius", 0.2, 10.0, 1.8, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("anisotropic_density_radial_ratio", "Radial vs Azimuthal Anisotropy", 0.1, 10.0, 2.2, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("wavefront_deflection_angle_deg", "Curvature Deflection Steering Angle", -180.0, 180.0, 45.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("shadowless_transparency_factor", "Shadowless Acoustic Transparency", 0.0, 1.0, 0.95, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_ideal_transformation_coordinate_map", "Exact Non-Euclidean Metric Tensor Mapping", true, (14, 165, 233)))
        ),
        "DynamicLobeBeamformerSpatializer" => Box::new(
            GenericDspNodeUi::new("DynamicLobeBeamformerSpatializer", "Spherical Harmonics Adaptive Beamforming Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("beam_azimuth_angle_deg", "Beam Steering Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("beam_elevation_angle_deg", "Beam Steering Elevation Angle", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lobe_focus_directivity_q", "Spatial Directivity Main Lobe Q", 1.0, 32.0, 6.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("side_lobe_suppression_db", "Side Lobe Attenuation Rejection", 0.0, 60.0, 24.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("array_radius_aperture_cm", "Virtual Transducer Array Aperture", 5.0, 200.0, 35.0, "cm", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_superdirective_null_steering", "Adaptive Null Interference Steering", true, (14, 165, 233)))
        ),
        "PhasedArrayUltrasoundTransducer" => Box::new(
            GenericDspNodeUi::new("PhasedArrayUltrasoundTransducer", "Self-Demodulating Parametric Ultrasound Audio Spotlight", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ultrasonic_carrier_freq_khz", "Parametric Ultrasonic Carrier Freq", 30.0, 80.0, 40.0, "kHz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("carrier_modulation_depth", "AM Carrier Modulation Index", 0.1, 1.0, 0.85, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("acoustic_nonlinearity_param_b_a", "Air Non-Linear Acoustic Factor (B/A)", 0.1, 2.0, 0.7, "B/A", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("projection_spotlight_throw_m", "Parametric Beam Throw Distance", 0.5, 30.0, 5.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spotlight_spot_radius_cm", "Acoustic Focus Spot Radius", 5.0, 100.0, 25.0, "cm", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_berktay_demodulation_filter", "Berktay Envelope Self-Demodulation Model", true, (14, 165, 233)))
        ),
        "BinauralNearFieldCrosstalkCanceller" => Box::new(
            GenericDspNodeUi::new("BinauralNearFieldCrosstalkCanceller", "Transaural Acoustic Crosstalk Cancellation & Head Model", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("speaker_spread_angle_deg", "Stereo Monitor Spacing Angle", 15.0, 90.0, 60.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("listener_distance_meters", "Listener-to-Speaker Distance", 0.5, 5.0, 1.8, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("interaural_distance_head_cm", "Interaural Head Breadth Spacing", 12.0, 25.0, 17.5, "cm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crosstalk_cancellation_depth_db", "Contralateral Crosstalk Attenuation", 6.0, 40.0, 22.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_pinna_coloration_trim", "Pinna Notch Coloration Compensation", 0.0, 1.0, 0.45, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_transaural_sweet_spot_boost", "Sweet-Spot Widening High-Frequency Guard", true, (14, 165, 233)))
        ),
        "SphericalHarmonicSoundfieldRotator" => Box::new(
            GenericDspNodeUi::new("SphericalHarmonicSoundfieldRotator", "Higher-Order Ambisonic Spherical Soundfield Rotator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("yaw_azimuth_rotation_deg", "Soundfield Yaw Azimuth Rotation", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_elevation_tilt_deg", "Soundfield Pitch Elevation Tilt", -90.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("roll_barrel_rotation_deg", "Soundfield Roll Barrel Rotation", -180.0, 180.0, 0.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spherical_order_processing_limit", "Ambisonic Rotation Order Depth", 1.0, 5.0, 3.0, "order", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("doppler_motion_smearing_ms", "Dynamic Angular Slew Interpolation", 0.0, 100.0, 10.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_wigner_matrix_normalization", "Spherical Energy Conservation Norm", true, (14, 165, 233)))
        ),
        "HilbertTransformStereoPanner" => Box::new(
            GenericDspNodeUi::new("HilbertTransformStereoPanner", "Hilbert Transform Analytic Stereo Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("quadrature_phase_angle_deg", "Analytic Quadrature Phase Angle", -180.0, 180.0, 90.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereo_expansion_width_pct", "Hilbert Decorrelation Stereo Width", 0.0, 200.0, 120.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hilbert_fir_taps_count", "Hilbert FIR Filter Tap Order", 16.0, 256.0, 64.0, "taps", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("frequency_split_low_shelf_hz", "Mono Low-End Anchor Crossover", 50.0, 500.0, 180.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("correlation_guard_attenuation_db", "Anti-Phase Correlation Limiter", -12.0, 0.0, -3.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_binaural_crossfeed_filter", "Binaural Head-Shadow Crossfeed", true, (14, 165, 233)))
        ),
        "BinauralPinnaReflectionEngine" => Box::new(
            GenericDspNodeUi::new("BinauralPinnaReflectionEngine", "Binaural Pinna & Torso Acoustic Reflection Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_elevation_angle_deg", "Vertical Pinna Elevation Angle", -45.0, 90.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sound_azimuth_angle_deg", "Horizontal Head Azimuth Angle", -180.0, 180.0, 30.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("torso_reflection_delay_ms", "Shoulder & Clavicle Reflection Delay", 0.5, 4.0, 1.2, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("concha_cavity_notch_depth_db", "Concha Resonance Elevation Notch Depth", 0.0, 24.0, 12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_damping_loss_db", "Contralateral Acoustic Head Shadow Loss", 0.0, 18.0, 6.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_near_field_interaural_level_boost", "Proximity Low-Frequency Bass Boost", true, (14, 165, 233)))
        ),
        "PsychoacousticStereoImager" => Box::new(
            GenericDspNodeUi::new("PsychoacousticStereoImager", "Multi-Band Psychoacoustic Stereo Width Imager", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("stereo_spread_width_pct", "Frequency-Dependent Stereo Width Scale", 0.0, 250.0, 135.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("haas_delay_offset_ms", "Interaural Haas Micro-Delay Depth", 0.0, 20.0, 8.5, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_band_mono_crossover_hz", "Sub-Bass Mono Collapse Crossover", 60.0, 400.0, 160.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_shelf_air_widening_db", "Ultra-High Air Side Channel Boost", -6.0, 6.0, 2.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_correlation_safety_ceiling", "Dynamic Phase Correlation Floor Guard", 0.0, 1.0, 0.75, "corr", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_mid_side_listen_audition", "Audition Isolated Side Channel Audio", false, (14, 165, 233)))
        ),
        "HigherOrderAmbisonicMicrophoneEncoder" => Box::new(
            GenericDspNodeUi::new("HigherOrderAmbisonicMicrophoneEncoder", "5th-Order Ambisonic Spherical Microphone Encoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("capsule_array_radius_mm", "Microphone Sphere Array Radius", 10.0, 150.0, 32.0, "mm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ambisonic_hoa_order", "Ambisonic Truncation Order", 1.0, 5.0, 3.0, "order", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("radial_nearfield_distance_m", "Near-Field Proximity Distance Filter", 0.1, 10.0, 1.5, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("capsule_polar_directivity_ratio", "Capsule Polar Pattern (Cardioid to Omni)", 0.0, 1.0, 0.5, "dir", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diffuse_field_equalization_db", "Diffuse Field High-Frequency Boost", 0.0, 12.0, 2.5, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_strict_phase_alignment", "Capsule Delay Skew Correction", true, (14, 165, 233)))
        ),
        "EllipticalStereoPhaseCorrelator" => Box::new(
            GenericDspNodeUi::new("EllipticalStereoPhaseCorrelator", "Elliptical Stereo Equalizer & Phase Correlator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("mono_sum_crossover_hz", "Elliptical Bass Mono Crossover Cutoff", 40.0, 400.0, 140.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereo_side_channel_gain_db", "Side Channel Spatial Gain Boost", -12.0, 12.0, 1.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("anti_phase_clamp_threshold", "Phase Correlation Limit Threshold", -1.0, 0.5, 0.1, "corr", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_shelf_widening_frequency_hz", "High-Frequency Spatial Air Crossover", 2000.0, 16000.0, 8000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("mid_channel_tilt_balance_db", "Mid Channel Spectral Tilt EQ", -6.0, 6.0, 0.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_correlation_safety_attenuation", "Mono Summing Safety Gain Attenuation", true, (14, 165, 233)))
        ),
        "VolumetricRaytracedRoomAcoustics" => Box::new(
            GenericDspNodeUi::new("VolumetricRaytracedRoomAcoustics", "Volumetric Ray-Traced 3D Room Acoustics Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("room_length_meters", "Room Acoustic Enclosure Length", 3.0, 80.0, 18.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("room_width_meters", "Room Acoustic Enclosure Width", 3.0, 60.0, 12.0, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("wall_absorption_coefficient_pct", "Boundary Wall Acoustic Absorption", 5.0, 95.0, 25.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_humidity_absorption_db", "High Frequency Air Moisture Damping", 0.0, 12.0, 3.2, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stochastic_ray_trace_count", "Simulated Monte Carlo Rays per Frame", 64.0, 2048.0, 512.0, "rays", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_specular_edge_diffraction", "Obstacle Edge Diffraction Scattering", true, (14, 165, 233)))
        ),
        "AmbisonicDistanceCuesBinauralizer" => Box::new(
            GenericDspNodeUi::new("AmbisonicDistanceCuesBinauralizer", "Ambisonic Distance & Air Absorption Binauralizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_source_distance_meters", "Acoustic Virtual Source Distance", 0.2, 50.0, 2.5, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("atmospheric_relative_humidity_pct", "Air Relative Humidity", 10.0, 95.0, 50.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("air_temperature_celsius", "Atmospheric Air Temperature", -10.0, 45.0, 20.0, "degC", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("head_rotation_yaw_degrees", "Listener Head Yaw Rotation Angle", -180.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("binaural_pinna_diffraction_amount", "Pinna Notch & Flange Diffraction Depth", 0.0, 1.0, 0.8, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_ground_reflection_specular", "Specular Floor Boundary Bounce", true, (14, 165, 233)))
        ),
        "BinauralConchaReflectionSpatializer" => Box::new(
            GenericDspNodeUi::new("BinauralConchaReflectionSpatializer", "Concha & Ear Canal Anatomical Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("concha_cavity_depth_mm", "Anatomical Concha Cavity Depth", 5.0, 25.0, 14.0, "mm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sound_elevation_angle_degrees", "Source Elevation Trajectory Angle", -60.0, 90.0, 0.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sound_azimuth_angle_degrees", "Source Azimuth Horizontal Angle", -180.0, 180.0, 30.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ear_canal_resonant_frequency_hz", "Ear Canal Resonance Peak Tuning", 2000.0, 5000.0, 3200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("interaural_time_delay_microsec", "Head Shadow ITD Delay Multiplier", 0.0, 900.0, 650.0, "us", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_interaural_spectral_diffraction", "Torso & Shoulder Acoustic Diffraction", true, (14, 165, 233)))
        ),
        "RelativisticDopplerLorentzTransform" => Box::new(
            GenericDspNodeUi::new("RelativisticDopplerLorentzTransform", "Relativistic Doppler & Lorentz Contraction Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("source_velocity_mach", "Source Relative Velocity", 0.0, 5.0, 0.85, "Mach", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("lorentz_dilation_factor", "Lorentz Acoustic Contraction Factor", 0.5, 1.0, 0.98, "gamma", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sonic_boom_shockwave_steepness", "Mach Cone Shockwave Steepness", 0.1, 10.0, 3.5, "slope", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("binaural_observer_azimuth_deg", "Observer Azimuth Trajectory Angle", -180.0, 180.0, 45.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_dispersion_damping", "Atmospheric Hypersonic Dispersion", 0.01, 1.0, 0.35, "loss", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_mach_cone_transonic_singularity", "Transonic Shock Singularity Modeling", true, (34, 197, 94)))
        ),
        "BinauralPinnaDiffractionSpatializer" => Box::new(
            GenericDspNodeUi::new("BinauralPinnaDiffractionSpatializer", "3D Head-Related Anatomical Pinna Diffraction Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth_horizontal_angle_deg", "3D Azimuth Direction Angle", -180.0, 180.0, 30.0, "deg", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("elevation_vertical_angle_deg", "3D Elevation Height Angle", -90.0, 90.0, 15.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("head_radius_measurement_cm", "Anatomical Head Radius Size", 6.0, 14.0, 8.75, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pinna_concha_notch_frequency_khz", "Upper Pinna Spectral Notch", 4.0, 12.0, 7.2, "kHz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("distance_attenuation_meters", "Virtual Sound Source Distance", 0.1, 50.0, 2.0, "m", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_torso_shoulder_bounce_reflection", "Torso & Shoulder Reflection Modeling", true, (34, 197, 94)))
        ),
        "BinauralNearFieldAcousticScattering" => Box::new(
            GenericDspNodeUi::new("BinauralNearFieldAcousticScattering", "Near-Field Head & Torso HRTF Acoustic Scatterer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("radial_source_distance_cm", "Radial Distance to Listener Head", 5.0, 200.0, 35.0, "cm", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("parallax_azimuth_angle_deg", "Horizontal Parallax Azimuth Angle", -180.0, 180.0, 25.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("torso_acoustic_scattering_radius", "Anatomical Torso Width Radius", 15.0, 45.0, 26.0, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("interaural_low_frequency_boost_db", "Near-Field Bass Curvature Boost", 0.0, 18.0, 5.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("head_boundary_absorption_alpha", "Cranial Acoustic Absorption Coefficient", 0.01, 0.5, 0.12, "alpha", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_pinna_micro_multipath", "Pinna Micro-Multipath Reflections", true, (34, 197, 94)))
        ),
        "AmbisonicBFormatDopplerSpatialPanner" => Box::new(
            GenericDspNodeUi::new("AmbisonicBFormatDopplerSpatialPanner", "Higher-Order Ambisonic (HOA) Spherical Kinematic Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("source_trajectory_radius_meters", "Spatial Distance Radius to Center", 0.2, 50.0, 3.5, "m", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("spherical_azimuth_bearing_deg", "Spherical Horizontal Azimuth Bearing", -180.0, 180.0, 45.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("spherical_elevation_tilt_deg", "Spherical Vertical Elevation Angle", -90.0, 90.0, 20.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("kinematic_doppler_velocity_mps", "Radial Trajectory Velocity (Doppler)", -150.0, 150.0, 0.0, "m/s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_damping_high_frequency_loss", "Atmospheric Air Damping Coefficient", 0.001, 0.2, 0.035, "loss", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_third_order_hoa_coefficients", "Higher-Order Ambisonic (3rd Order) Output", true, (34, 197, 94)))
        ),
        "SphericalHarmonicEigenmikeBeamformer" => Box::new(
            GenericDspNodeUi::new("SphericalHarmonicEigenmikeBeamformer", "Spherical Microphone Array Beamformer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("beam_steering_azimuth_deg", "Beamformer Look Direction Azimuth", -180.0, 180.0, 0.0, "deg", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("beam_steering_elevation_deg", "Beamformer Look Direction Elevation", -90.0, 90.0, 15.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("spherical_order_directivity", "Spherical Harmonic Expansion Order", 1.0, 4.0, 3.0, "order", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("null_steering_suppression_db", "Spatial Interference Null Attenuation", 0.0, 40.0, 24.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diffuse_soundfield_energy_blend", "Diffuse Ambient Soundfield Preservation", 0.0, 1.0, 0.25, "ratio", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_binaural_ear_synthesis", "Spherical Array to Binaural HRTF Synthesis", true, (34, 197, 94)))
        ),
        "BinauralRaytracedEarlyReflectionSpatializer" => Box::new(
            GenericDspNodeUi::new("BinauralRaytracedEarlyReflectionSpatializer", "Binaural Raytraced Early Reflection Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("virtual_room_length_meters", "Virtual Acoustic Room Length", 2.0, 40.0, 12.0, "m", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("virtual_room_width_meters", "Virtual Acoustic Room Width", 2.0, 30.0, 8.5, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("virtual_room_height_meters", "Virtual Acoustic Room Height", 2.0, 15.0, 4.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("wall_surface_absorption_alpha", "Boundary Acoustic Absorption Alpha", 0.05, 0.95, 0.35, "alpha", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("direct_to_reverberant_ratio_db", "Direct Sound vs Reflection Ratio", -24.0, 24.0, 4.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_high_order_corner_scattering", "Second-Order Corner Scattering Rays", true, (34, 197, 94)))
        ),
        "AmbisonicRadialDistanceFilterBinaural" => Box::new(
            GenericDspNodeUi::new("AmbisonicRadialDistanceFilterBinaural", "Higher-Order Ambisonic Radial Distance Filter (RDF)", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_source_radial_distance_m", "Virtual Source Radial Distance", 0.1, 50.0, 1.8, "m", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("near_field_bass_boost_boost_db", "Acoustic Proximity Bass Shelf Boost", 0.0, 24.0, 6.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("air_absorption_humidity_percent", "Atmospheric Relative Humidity", 10.0, 100.0, 50.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spherical_wave_order_truncation", "Ambisonic Radial Expansion Order", 1.0, 7.0, 4.0, "order", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("parallax_elevation_compensation_deg", "Distance Parallax Elevation Angle", -90.0, 90.0, 0.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_inverse_square_law_decay", "Physical 1/R Amplitude Attenuation", true, (34, 197, 94)))
        ),
        "AmbisonicSphericalMicrophonePanner" => Box::new(
            GenericDspNodeUi::new("AmbisonicSphericalMicrophonePanner", "Spherical HOA 3D Spatial Panner & Doppler Engine", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("source_azimuth_angle_deg", "Spatial Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("source_elevation_angle_deg", "Spatial Vertical Elevation", -90.0, 90.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("source_radial_distance_m", "Radial Spatial Distance from Center", 0.2, 30.0, 2.5, "m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("doppler_motion_velocity_mps", "Source Motion Velocity (Doppler)", -50.0, 50.0, 0.0, "m/s", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("acoustic_head_shadow_strength", "Pinna & Head Acoustic Shadow Filter", 0.0, 1.0, 0.7, "shadow", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_binaural_diffuse_field_eq", "Diffuse-Field Equalization Filter", true, (34, 197, 94)))
        ),
        "TransauralAcousticDipoleSpatializer" => Box::new(
            GenericDspNodeUi::new("TransauralAcousticDipoleSpatializer", "Transaural Acoustic Dipole 3D Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("dipole_axis_azimuth_angle_deg", "Dipole Acoustic Axis Azimuth Angle", -180.0, 180.0, 45.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("dipole_figure_eight_purity", "Figure-8 Dipole Polar Dispersion Ratio", 0.0, 1.0, 0.85, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("distance_radiation_falloff_m", "Dipole Near-Field Distance Attenuation", 0.2, 50.0, 2.5, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("contralateral_pinna_diffraction_db", "Contralateral Ear Pinna Diffraction", 0.0, 18.0, 7.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("spatial_crosstalk_cancellation_depth", "Stereo Speaker Crosstalk Cancellation", 0.0, 1.0, 0.6, "depth", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_ground_reflection_bounce", "Virtual Floor Boundary Acoustic Bounce", true, (34, 197, 94)))
        ),
        "AmbisonicDistanceAirAbsorptionFilter" => Box::new(
            GenericDspNodeUi::new("AmbisonicDistanceAirAbsorptionFilter", "ISO 9613-1 Ambisonic Atmospheric Air Absorption", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("virtual_source_distance_meters", "Virtual Sound Source Distance", 0.5, 500.0, 18.0, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ambient_air_temperature_celsius", "Atmospheric Air Temperature", -20.0, 50.0, 20.0, "C", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("relative_air_humidity_percent", "Atmospheric Relative Humidity", 10.0, 99.0, 50.0, "%", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("inverse_square_gain_attenuation_db", "Spherical Inverse-Square Law Falloff", 0.0, 60.0, 24.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ambisonic_order_proximity_bass_boost", "Near-Field Wavefront Curvature Proximity Bass", 0.0, 18.0, 4.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_doppler_pitch_shift_air_motion", "Wind Vector Atmospheric Doppler Frequency Shift", false, (34, 197, 94)))
        ),
        "SphericalBinauralPinnaDiffractionSpatializer" => Box::new(
            GenericDspNodeUi::new("SphericalBinauralPinnaDiffractionSpatializer", "Spherical BEM HRTF Pinna & Torso Diffraction 3D Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("sound_source_azimuth_deg", "Horizontal Soundfield Azimuth Angle", -180.0, 180.0, 30.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sound_source_elevation_deg", "Vertical Soundfield Elevation Angle", -90.0, 90.0, 15.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("ear_pinna_size_scaling_factor", "Anatomical Pinna Cavum Dimension Scaling", 0.7, 1.4, 1.0, "scale", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("torso_chest_shadowing_attenuation_db", "Torso Shadowing Acoustic Attenuation", 0.0, 18.0, 6.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("head_acoustic_diameter_cm", "Interaural Spherical Head Diameter", 12.0, 24.0, 17.5, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_interaural_spectral_cross_talk", "Diffuse Field Equalization Compensation", true, (34, 197, 94)))
        ),
        "AcousticParabolicReflectorSpatializer" => Box::new(
            GenericDspNodeUi::new("AcousticParabolicReflectorSpatializer", "Acoustic Parabolic Dish Concentrator & Focal Beam Spatializer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("parabolic_dish_diameter_meters", "Parabolic Reflector Dish Aperture Diameter", 0.3, 5.0, 1.2, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("focal_point_distance_offset_cm", "Focal Microphone Position Axial Offset", -20.0, 20.0, 0.0, "cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("acoustic_beam_directivity_q", "Directional Acoustic Beam Sharpness (Q)", 1.0, 30.0, 8.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("rim_edge_diffraction_cancellation", "Dish Perimeter Rim Edge Wave Diffraction", 0.0, 1.0, 0.42, "diff", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("distance_air_viscous_attenuation_db", "High-Frequency Atmospheric Distance Loss", 0.0, 30.0, 4.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_spherical_aberration_modeling", "Deep Dish Off-Axis Spherical Aberration", true, (34, 197, 94)))
        ),
        "WfsAcousticCircularArrayRenderer" => Box::new(
            GenericDspNodeUi::new("WfsAcousticCircularArrayRenderer", "Wave Field Synthesis (WFS) Circular Speaker Array Renderer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("virtual_source_distance_meters", "Virtual Sound Source Radial Distance", -15.0, 15.0, 3.5, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("virtual_source_angle_degrees", "Virtual Sound Source Angular Direction", -180.0, 180.0, 45.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("array_diameter_meters", "Loudspeaker Circular Array Physical Diameter", 1.5, 20.0, 5.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spatial_aliasing_cutoff_hz", "Discrete Transducer Spatial Aliasing Frequency", 800.0, 8000.0, 1650.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attenuation_prefilter_slope_db", "Half-Derivative 3 dB/Oct Wavefront Prefilter", 0.0, 6.0, 3.0, "dB/oct", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_focused_source_room_compensation", "Focused Source Evaporation Boundary Compensation", true, (34, 197, 94)))
        ),
        "AmbisonicBinauralEarCanalOcclusion" => Box::new(
            GenericDspNodeUi::new("AmbisonicBinauralEarCanalOcclusion", "Ambisonic HRTF Concha & Ear Canal Occlusion Resonator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ear_canal_length_mm", "Anatomical Ear Canal Physical Depth Length", 18.0, 35.0, 26.0, "mm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("occlusion_seal_depth_factor", "Earbud Silicone Tip Ear Canal Occlusion Seal", 0.0, 1.0, 0.85, "seal", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("concha_cavity_anti_resonance_hz", "Pinna Concha Anti-Resonance Pin Hole Notch", 6000.0, 12000.0, 8400.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("interaural_time_delay_us", "Interaural Head Diffraction Time Delay (ITD)", 0.0, 800.0, 450.0, "us", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("tympanic_membrane_acoustic_impedance", "Eardrum Acoustic Terminal Compliance (Z0)", 0.1, 2.0, 0.95, "Z0", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_head_motion_yaw_compensation", "Low-Latency Head Tracker Yaw Angular Compensation", false, (34, 197, 94)))
        ),
        "AmbisonicSphericalHeadShadowDiffraction" => Box::new(
            GenericDspNodeUi::new("AmbisonicSphericalHeadShadowDiffraction", "Higher-Order Ambisonic Spherical Head Shadow & ITD", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("head_radius_meters", "Rigid Sphere Cranial Radius (Anatomical Head Size)", 0.06, 0.12, 0.0875, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("creeping_wave_diffraction_damping", "Contralateral Creeping Wave Shadow Attenuation", 0.0, 24.0, 12.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("interaural_time_delay_scaling", "Woodworth-Schlosberg ITD Geometric Time Delay", 0.5, 1.5, 1.0, "scale", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("elevation_pinna_notch_frequency_hz", "Ear Pinna Elevation Concha Notch Tuning", 5000.0, 11000.0, 7800.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("torso_shoulder_bounce_delay_ms", "Shoulder & Torso Acoustic Reflection Delay", 0.5, 4.0, 1.2, "ms", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_near_field_curvature_correction", "Near-Field Wavefront Curvature Correction", true, (34, 197, 94)))
        ),
        "WfsPlanarLinearArrayAcousticSynthesizer" => Box::new(
            GenericDspNodeUi::new("WfsPlanarLinearArrayAcousticSynthesizer", "Wave Field Synthesis Linear Speaker Array Synthesizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("virtual_source_x_pos_meters", "Virtual Sound Source Lateral Position (X)", -10.0, 10.0, 0.0, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("virtual_source_depth_z_meters", "Virtual Sound Source Depth Distance (Z)", -8.0, 12.0, 2.5, "m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("loudspeaker_spacing_pitch_meters", "Physical Speaker Array Element Spacing Pitch", 0.05, 0.5, 0.12, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spatial_tapering_window_width", "Array Edge Cosine Window Tapering Width", 0.0, 1.0, 0.35, "win", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("wavefront_curvature_comp_db", "Distance Attenuation 1/sqrt(r) Gain Correction", -12.0, 6.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_focused_point_source_rendering", "Focused Point Source Interior Wavefront Mode", false, (34, 197, 94)))
        ),
        "AmbisonicSphericalMicrophoneArrayBinauralizer" => Box::new(
            GenericDspNodeUi::new("AmbisonicSphericalMicrophoneArrayBinauralizer", "Higher-Order Ambisonic Spherical Array Binauralizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("hoa_spherical_order", "Spherical Harmonics Ambisonic Order (1st-5th)", 1.0, 5.0, 3.0, "order", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("rigid_sphere_radius_cm", "Microphone Array Sphere Radius", 2.0, 20.0, 8.5, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pinna_spectral_diffraction_db", "Pinna Concha Spectral Notch Intensity", 0.0, 18.0, 9.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("interaural_cross_talk_damping_db", "Transaural Acoustic Shadow Attenuation", 0.0, 24.0, 14.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("head_yaw_rotation_deg", "Binaural Listener Head Yaw Angle", -180.0, 180.0, 0.0, "deg", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_diffuse_field_equalization", "Diffuse-Field HRIR Equalization", true, (34, 197, 94)))
        ),
        "TransauralStereoDipoleCrosstalkCanceller" => Box::new(
            GenericDspNodeUi::new("TransauralStereoDipoleCrosstalkCanceller", "Transaural Stereo Dipole Loudspeaker Crosstalk Canceller", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("loudspeaker_span_angle_deg", "Stereo Dipole Loudspeaker Span Angle", 5.0, 60.0, 12.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("listener_distance_meters", "Listener Distance from Loudspeakers", 0.5, 4.0, 1.8, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("crosstalk_cancellation_depth_db", "Contralateral Crosstalk Cancellation Depth", 6.0, 36.0, 22.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sweet_spot_width_tolerance", "Sweet Spot Off-Axis Tolerance Window", 0.1, 1.0, 0.5, "tol", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_frequency_bass_monomaker_hz", "Bass Monomaker Crossover Frequency", 40.0, 250.0, 90.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_head_motion_robustness", "Head Motion Robustness Pre-Filtering", true, (34, 197, 94)))
        ),
        "HigherOrderAmbisonicBinauralNearFieldCompensator" => Box::new(
            GenericDspNodeUi::new("HigherOrderAmbisonicBinauralNearFieldCompensator", "Higher-Order Ambisonic Near-Field Distance Compensator", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("source_radial_distance_meters", "Virtual Source Radial Distance", 0.1, 10.0, 0.45, "m", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("ambisonic_spherical_order_limit", "Maximum HOA Spherical Expansion Order", 1.0, 5.0, 3.0, "order", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("acoustic_reference_radius_meters", "Acoustic Far-Field Normalization Radius", 0.5, 5.0, 1.5, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("proximity_bass_boost_damping", "Spherical Proximity Bass Boost Damping", 0.0, 1.0, 0.25, "damp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_absorption_humidity_percent", "Atmospheric Relative Humidity Percentage", 10.0, 95.0, 50.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_spherical_wave_parallax_shift", "Interaural Spherical Wave Parallax Shift", true, (34, 197, 94)))
        ),
        "TransauralAcousticCrossfeedHeadphoneOptimizer" => Box::new(
            GenericDspNodeUi::new("TransauralAcousticCrossfeedHeadphoneOptimizer", "Transaural Acoustic Crossfeed Headphone Optimizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("crossfeed_blend_intensity_db", "Contralateral Crossfeed Blend Depth", -24.0, -6.0, -12.0, "dB", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_crossover_hz", "Acoustic Head-Shadow Baffle Crossover", 300.0, 1200.0, 650.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("interaural_time_delay_microsec", "Interaural Time Delay (ITD) Path Length", 100.0, 450.0, 260.0, "us", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("natural_room_reflection_ambience", "Binaural Early Ambience Blend", 0.0, 1.0, 0.22, "amb", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_frequency_mono_depth", "Sub-Bass Mono Anchor Convergence", 0.0, 1.0, 0.4, "mono", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_harman_target_tilt_equalization", "Harman Headphone Target Tilt Curve", true, (34, 197, 94)))
        ),
        "InterauralPinnaSpectralNotchFilter" => Box::new(
            GenericDspNodeUi::new("InterauralPinnaSpectralNotchFilter", "Interaural Pinna Spectral Notch Filter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("elevation_angle", "Vertical Elevation Angle", -45.0, 90.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("notch_depth", "Pinna Concha Notch Depth", 0.0, 36.0, 18.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pinna_scale", "Ear Anthropometric Scaling", 0.8, 1.2, 1.0, "scale", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("torso_reflection", "Shoulder & Torso Reflection", 0.0, 1.0, 0.30, "refl", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("interaural_balance", "Interaural Asymmetry Balance", -1.0, 1.0, 0.0, "pan", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothing", "HRTF Curve Smoothing", 0.0, 1.0, 0.20, "smooth", (34, 197, 94)))
        ),
        "AmbisonicSphericalCapMicrophonePanner" => Box::new(
            GenericDspNodeUi::new("AmbisonicSphericalCapMicrophonePanner", "Ambisonic Spherical Cap Microphone Panner", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("azimuth", "Horizontal Azimuth Angle", -180.0, 180.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("elevation", "Vertical Elevation Angle", -90.0, 90.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("distance", "Virtual Source Radial Distance", 0.1, 20.0, 2.0, "m", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("cap_aperture", "Capsule Cap Opening Angle", 10.0, 180.0, 90.0, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hoa_order", "Ambisonic Spherical Harmonic Order", 1.0, 7.0, 3.0, "order", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("divergence", "Source Divergence Spread", 0.0, 1.0, 0.15, "spread", (34, 197, 94)))
        ),
        "BinauralSpectralNotchElevationSynthesizer" => Box::new(
            GenericDspNodeUi::new("BinauralSpectralNotchElevationSynthesizer", "Binaural Spectral Notch Elevation Synthesizer", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("elevation_angle", "Vertical Elevation Angle", -45.0, 90.0, 15.0, "deg", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("concha_notch_attenuation", "Concha Destructive Notch Depth", 0.0, 40.0, 22.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ear_canal_resonance", "Ear Canal 3kHz Acoustic Peaking", 0.0, 12.0, 4.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("shoulder_delay_time", "Shoulder Reflection Delay Offset", 0.2, 2.5, 0.85, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_falloff", "Interaural Head Shadow Damping", 0.0, 1.0, 0.65, "loss", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diffuse_field_equalization", "Diffuse-Field Calibration Curve", 0.0, 1.0, 0.85, "eq", (34, 197, 94)))
        ),
        "AmbisonicSphericalHarmonicRotatorMatrix" => Box::new(
            GenericDspNodeUi::new("AmbisonicSphericalHarmonicRotatorMatrix", "Ambisonic Spherical Harmonic Rotator Matrix", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("yaw_heading_angle", "Yaw Horizontal Heading Angle", -180.0, 180.0, 0.0, "deg", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("pitch_elevation_angle", "Pitch Vertical Tilt Angle", -90.0, 90.0, 0.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("roll_tilt_angle", "Roll Lateral Tumble Angle", -180.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("hoa_order_selection", "Spherical Harmonics HOA Order", 1.0, 5.0, 3.0, "order", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spatial_energy_normalization", "Acoustic Energy Conservation", 0.0, 1.0, 1.0, "norm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("angular_smoothing_lag", "Gimbal Smoothing Lag Constant", 5.0, 500.0, 50.0, "ms", (34, 197, 94)))
        ),
        "BinauralConchaReflectionPinnaFilter" => Box::new(
            GenericDspNodeUi::new("BinauralConchaReflectionPinnaFilter", "Binaural Concha Reflection & Pinna Notch Filter", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("elevation_angle_degrees", "Sound Source Elevation Angle", -45.0, 90.0, 15.0, "deg", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("azimuth_angle_degrees", "Sound Source Horizontal Azimuth", -180.0, 180.0, 30.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("concha_cavity_depth", "Anatomical Concha Cavity Depth", 5.0, 25.0, 14.0, "mm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pinna_notch_frequency", "Primary Pinna Spectral Notch", 4000.0, 12000.0, 7800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("torso_shoulder_reflection", "Shoulder Reflection Delay Offset", 0.1, 2.5, 0.8, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("head_shadow_dispersion", "Spherical Head Acoustic Shadowing", 0.0, 1.0, 0.75, "damp", (34, 197, 94)))
        ),
        "AmbisonicThirdOrderPlanarSoundfieldDecoder" => Box::new(
            GenericDspNodeUi::new("AmbisonicThirdOrderPlanarSoundfieldDecoder", "Ambisonic 3rd-Order Planar Soundfield Decoder", DspNodeCategory::SpatialSurround)
                .with_param(DspParamDescriptor::new_linear("ambisonic_order_limit", "Active Ambisonic Decomposition Order", 1.0, 3.0, 3.0, "ord", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("max_re_energy_weighting", "Max-rE Acoustic Energy Beamforming", 0.0, 1.0, 0.85, "ratio", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("distance_compensation_m", "Listener-to-Speaker Radius", 0.5, 10.0, 2.2, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_shelf_cutoff", "Dual-Band Shelf Crossover Freq", 200.0, 2000.0, 700.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("binaural_decoding_matrix", "Decoded Loudspeaker Count", 4.0, 16.0, 8.0, "spk", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spatial_blur_width", "Diffuse Soundfield Spread Angle", 0.0, 90.0, 12.0, "deg", (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
