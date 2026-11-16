// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `DynamicsMaster`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("CompressorNode".to_string(), DspNodeDescriptor::new("CompressorNode", "VCA Studio Compressor with RMS Detection", DspNodeCategory::DynamicsMaster, "VCA Studio Compressor with RMS Detection")
        .with_param(DspParamSchema::knob("threshold", "Threshold", -48.0, 0.0, -18.0, "dB", MacroRole::Tone, "Threshold parameter"))
        .with_param(DspParamSchema::knob("ratio", "Compression Ratio", 1.0, 20.0, 4.0, ":1", MacroRole::Tone, "Compression Ratio parameter"))
        .with_param(DspParamSchema::log_knob("attack", "Attack Time", 0.1, 200.0, 15.0, "ms", MacroRole::Tone, "Attack Time logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("release", "Release Time", 10.0, 2000.0, 150.0, "ms", MacroRole::Tone, "Release Time logarithmic parameter"))
        .with_param(DspParamSchema::knob("makeup", "Makeup Gain", 0.0, 24.0, 3.0, "dB", MacroRole::Tone, "Makeup Gain parameter"))
        .with_param(DspParamSchema::knob("knee", "Soft Knee Width", 0.0, 18.0, 3.0, "dB", MacroRole::Tone, "Soft Knee Width parameter"))
    );
    descriptors.insert("MultibandCompressorNode".to_string(), DspNodeDescriptor::new("MultibandCompressorNode", "3-Band Linear-Phase Dynamics Processor", DspNodeCategory::DynamicsMaster, "3-Band Linear-Phase Dynamics Processor")
        .with_param(DspParamSchema::knob("low_thresh", "Low Band Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Low Band Threshold parameter"))
        .with_param(DspParamSchema::knob("mid_thresh", "Mid Band Threshold", -40.0, 0.0, -16.0, "dB", MacroRole::Tone, "Mid Band Threshold parameter"))
        .with_param(DspParamSchema::knob("high_thresh", "High Band Threshold", -40.0, 0.0, -14.0, "dB", MacroRole::Tone, "High Band Threshold parameter"))
        .with_param(DspParamSchema::log_knob("low_cross", "Low/Mid Crossover", 60.0, 500.0, 180.0, "Hz", MacroRole::Tone, "Low/Mid Crossover logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("high_cross", "Mid/High Crossover", 1000.0, 10000.0, 3500.0, "Hz", MacroRole::Tone, "Mid/High Crossover logarithmic parameter"))
        .with_param(DspParamSchema::knob("master_gain", "Master Output Trim", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Master Output Trim parameter"))
    );
    descriptors.insert("MultibandDynamicsProcessor".to_string(), DspNodeDescriptor::new("MultibandDynamicsProcessor", "Tri-Band Expander/Compressor Dynamics Matrix", DspNodeCategory::DynamicsMaster, "Tri-Band Expander/Compressor Dynamics Matrix")
        .with_param(DspParamSchema::knob("low_comp", "Low Band Compression", 0.0, 100.0, 35.0, "%", MacroRole::Tone, "Low Band Compression parameter"))
        .with_param(DspParamSchema::knob("mid_comp", "Mid Band Compression", 0.0, 100.0, 30.0, "%", MacroRole::Tone, "Mid Band Compression parameter"))
        .with_param(DspParamSchema::knob("high_comp", "High Band Compression", 0.0, 100.0, 25.0, "%", MacroRole::Tone, "High Band Compression parameter"))
        .with_param(DspParamSchema::log_knob("crossover_1", "Low/Mid Crossover", 80.0, 600.0, 200.0, "Hz", MacroRole::Tone, "Low/Mid Crossover logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("crossover_2", "Mid/High Crossover", 1500.0, 8000.0, 4000.0, "Hz", MacroRole::Tone, "Mid/High Crossover logarithmic parameter"))
        .with_param(DspParamSchema::knob("saturation", "Tape Saturation Drive", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Tape Saturation Drive parameter"))
    );
    descriptors.insert("OpticalCompressorNode".to_string(), DspNodeDescriptor::new("OpticalCompressorNode", "Vintage LA-2A Optical Photocell Dynamics", DspNodeCategory::DynamicsMaster, "Vintage LA-2A Optical Photocell Dynamics")
        .with_param(DspParamSchema::knob("peak_reduct", "Peak Reduction", 0.0, 100.0, 45.0, "%", MacroRole::Tone, "Peak Reduction parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 100.0, 50.0, "%", MacroRole::Tone, "Output Gain parameter"))
        .with_param(DspParamSchema::toggle("limit_mode", "Compress / Limit Mode", false, "Compress / Limit Mode toggle switch"))
    );
    descriptors.insert("NoiseGateNode".to_string(), DspNodeDescriptor::new("NoiseGateNode", "Fast Transient Expander & Noise Gate", DspNodeCategory::DynamicsMaster, "Fast Transient Expander & Noise Gate")
        .with_param(DspParamSchema::knob("threshold", "Gate Threshold", -80.0, 0.0, -45.0, "dB", MacroRole::Tone, "Gate Threshold parameter"))
        .with_param(DspParamSchema::log_knob("attack", "Fast Attack", 0.05, 50.0, 1.0, "ms", MacroRole::Tone, "Fast Attack logarithmic parameter"))
        .with_param(DspParamSchema::knob("hold", "Hold Duration", 0.0, 500.0, 20.0, "ms", MacroRole::Tone, "Hold Duration parameter"))
        .with_param(DspParamSchema::log_knob("release", "Smooth Release", 5.0, 1000.0, 80.0, "ms", MacroRole::Tone, "Smooth Release logarithmic parameter"))
        .with_param(DspParamSchema::knob("range", "Floor Attenuation Range", -80.0, 0.0, -60.0, "dB", MacroRole::Tone, "Floor Attenuation Range parameter"))
    );
    descriptors.insert("DeesserNode".to_string(), DspNodeDescriptor::new("DeesserNode", "Dynamic High-Frequency Sibilance Suppressor", DspNodeCategory::DynamicsMaster, "Dynamic High-Frequency Sibilance Suppressor")
        .with_param(DspParamSchema::knob("threshold", "De-Esser Threshold", -40.0, 0.0, -20.0, "dB", MacroRole::Tone, "De-Esser Threshold parameter"))
        .with_param(DspParamSchema::log_knob("freq", "Sibilance Center Freq", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Sibilance Center Freq logarithmic parameter"))
        .with_param(DspParamSchema::toggle("listen", "Delta Audition Listen", false, "Delta Audition Listen toggle switch"))
        .with_param(DspParamSchema::knob("reduction", "Max Reduction Cap", -24.0, 0.0, -12.0, "dB", MacroRole::Tone, "Max Reduction Cap parameter"))
    );
    descriptors.insert("MultibandClipperNode".to_string(), DspNodeDescriptor::new("MultibandClipperNode", "Tri-Band Mastering Clipper & Saturation Stage", DspNodeCategory::DynamicsMaster, "Tri-Band Mastering Clipper & Saturation Stage")
        .with_param(DspParamSchema::knob("low_clip_ceiling_db", "Low Band Ceiling", -12.0, 0.0, -0.5, "dBFS", MacroRole::Tone, "Low Band Ceiling parameter"))
        .with_param(DspParamSchema::knob("mid_clip_ceiling_db", "Mid Band Ceiling", -12.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "Mid Band Ceiling parameter"))
        .with_param(DspParamSchema::knob("high_clip_ceiling_db", "High Band Ceiling", -12.0, 0.0, -0.1, "dBFS", MacroRole::Tone, "High Band Ceiling parameter"))
        .with_param(DspParamSchema::knob("softness_knee", "Soft Clipping Knee", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Soft Clipping Knee parameter"))
    );
    descriptors.insert("MultibandDecompressorNode".to_string(), DspNodeDescriptor::new("MultibandDecompressorNode", "Multiband Upward Dynamic Range Decompressor", DspNodeCategory::DynamicsMaster, "Multiband Upward Dynamic Range Decompressor")
        .with_param(DspParamSchema::knob("expansion_ratio", "Upward Expansion Ratio", 1.0, 4.0, 1.5, ":1", MacroRole::Tone, "Upward Expansion Ratio parameter"))
        .with_param(DspParamSchema::knob("upward_threshold_db", "Decompression Threshold", -40.0, -10.0, -24.0, "dB", MacroRole::Tone, "Decompression Threshold parameter"))
        .with_param(DspParamSchema::log_knob("crossover_low_hz", "Low/Mid Split Frequency", 60.0, 500.0, 160.0, "Hz", MacroRole::Tone, "Low/Mid Split Frequency logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("crossover_high_hz", "Mid/High Split Frequency", 1500.0, 10000.0, 4000.0, "Hz", MacroRole::Tone, "Mid/High Split Frequency logarithmic parameter"))
    );
    descriptors.insert("MultibandExpanderNode".to_string(), DspNodeDescriptor::new("MultibandExpanderNode", "Tri-Band Downward Transient Expander", DspNodeCategory::DynamicsMaster, "Tri-Band Downward Transient Expander")
        .with_param(DspParamSchema::knob("downward_ratio", "Expansion Ratio", 1.0, 8.0, 2.0, ":1", MacroRole::Tone, "Expansion Ratio parameter"))
        .with_param(DspParamSchema::knob("gate_threshold_db", "Expansion Floor Gate", -60.0, -10.0, -32.0, "dB", MacroRole::Tone, "Expansion Floor Gate parameter"))
        .with_param(DspParamSchema::knob("range_cap_db", "Max Dynamic Range Cap", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Max Dynamic Range Cap parameter"))
        .with_param(DspParamSchema::knob("release_time_ms", "Expander Recovery Time", 10.0, 500.0, 85.0, "ms", MacroRole::Tone, "Expander Recovery Time parameter"))
    );
    descriptors.insert("MultibandSaturatorNode".to_string(), DspNodeDescriptor::new("MultibandSaturatorNode", "3-Band Frequency-Split Saturation Warmer", DspNodeCategory::DynamicsMaster, "3-Band Frequency-Split Saturation Warmer")
        .with_param(DspParamSchema::knob("low_drive_x", "Low Band Tape Warmth", 1.0, 10.0, 2.0, "x", MacroRole::Tone, "Low Band Tape Warmth parameter"))
        .with_param(DspParamSchema::knob("mid_drive_x", "Mid Band Tube Drive", 1.0, 10.0, 3.2, "x", MacroRole::Tone, "Mid Band Tube Drive parameter"))
        .with_param(DspParamSchema::knob("high_drive_x", "High Band Diode Sheen", 1.0, 10.0, 1.8, "x", MacroRole::Tone, "High Band Diode Sheen parameter"))
        .with_param(DspParamSchema::knob("tape_tube_blend", "Tape vs Tube Color Blend", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tape vs Tube Color Blend parameter"))
    );
    descriptors.insert("UpwardCompressorNode".to_string(), DspNodeDescriptor::new("UpwardCompressorNode", "OTT-Style Upward Audio Compressor", DspNodeCategory::DynamicsMaster, "OTT-Style Upward Audio Compressor")
        .with_param(DspParamSchema::knob("upward_depth", "Upward Compression Depth", 0.0, 100.0, 65.0, "%", MacroRole::Tone, "Upward Compression Depth parameter"))
        .with_param(DspParamSchema::knob("downward_depth", "Downward Compression Depth", 0.0, 100.0, 40.0, "%", MacroRole::Tone, "Downward Compression Depth parameter"))
        .with_param(DspParamSchema::knob("dynamics_speed", "Dynamics Ballistics Speed", 0.1, 10.0, 2.5, "x", MacroRole::Tone, "Dynamics Ballistics Speed parameter"))
        .with_param(DspParamSchema::knob("high_lift_db", "High Air Presence Lift", 0.0, 18.0, 4.0, "dB", MacroRole::Tone, "High Air Presence Lift parameter"))
    );
    descriptors.insert("VariMuMasterNode".to_string(), DspNodeDescriptor::new("VariMuMasterNode", "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor", DspNodeCategory::DynamicsMaster, "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor")
        .with_param(DspParamSchema::choice("time_constant", "Fairchild Time Constant", &["Pos 1 (Fast 0.2s/0.3s)", "Pos 2 (0.2s/0.8s)", "Pos 3 (0.4s/2.0s)", "Pos 4 (0.8s/5.0s)", "Pos 5 (Auto Fast)", "Pos 6 (Auto Program)"], 4, "Fairchild Time Constant mode selector"))
        .with_param(DspParamSchema::knob("dc_threshold", "Vari-Mu Tube Threshold", -30.0, 0.0, -12.0, "dB", MacroRole::Tone, "Vari-Mu Tube Threshold parameter"))
        .with_param(DspParamSchema::choice("tube_bias_mode", "Tube Bias Configuration", &["Standard Push-Pull", "Aggressive Mid Push", "Clean Master Linear"], 0, "Tube Bias Configuration mode selector"))
        .with_param(DspParamSchema::toggle("link_channels", "Stereo Linked Sidechain", true, "Stereo Linked Sidechain toggle switch"))
    );
    descriptors.insert("DynamicCrestShaperNode".to_string(), DspNodeDescriptor::new("DynamicCrestShaperNode", "Intelligent Peak-to-RMS Crest Factor Shaper", DspNodeCategory::DynamicsMaster, "Intelligent Peak-to-RMS Crest Factor Shaper")
        .with_param(DspParamSchema::knob("target_crest_db", "Target Crest Factor", 6.0, 20.0, 12.0, "dB", MacroRole::Tone, "Target Crest Factor parameter"))
        .with_param(DspParamSchema::knob("transient_priority", "Transient vs Body Weight", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Transient vs Body Weight parameter"))
        .with_param(DspParamSchema::knob("smoothing_window_ms", "RMS Detection Window", 10.0, 500.0, 100.0, "ms", MacroRole::Tone, "RMS Detection Window parameter"))
        .with_param(DspParamSchema::toggle("auto_makeup", "Auto Crest Makeup Gain", true, "Auto Crest Makeup Gain toggle switch"))
    );
    descriptors.insert("ParallelTransientSaturatorNode".to_string(), DspNodeDescriptor::new("ParallelTransientSaturatorNode", "Parallel Wet/Dry Transient Driver & Saturator", DspNodeCategory::DynamicsMaster, "Parallel Wet/Dry Transient Driver & Saturator")
        .with_param(DspParamSchema::knob("transient_drive_db", "Transient Punch Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Tone, "Transient Punch Drive parameter"))
        .with_param(DspParamSchema::knob("sustain_warmth_db", "Sustain Body Saturation", 0.0, 24.0, 4.5, "dB", MacroRole::Tone, "Sustain Body Saturation parameter"))
        .with_param(DspParamSchema::knob("parallel_mix", "Parallel Wet Blend", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Parallel Wet Blend parameter"))
        .with_param(DspParamSchema::choice("clipping_mode", "Clipping Topology", &["Soft Germanium Diode", "Hard Silicon Diode", "Asymmetric Tube Triode"], 0, "Clipping Topology mode selector"))
    );
    descriptors.insert("TransientClipperNode".to_string(), DspNodeDescriptor::new("TransientClipperNode", "Sub-Millisecond True Transient Soft Clipper", DspNodeCategory::DynamicsMaster, "Sub-Millisecond True Transient Soft Clipper")
        .with_param(DspParamSchema::knob("threshold_db", "Clipping Knee Threshold", -18.0, 0.0, -3.0, "dBFS", MacroRole::Tone, "Clipping Knee Threshold parameter"))
        .with_param(DspParamSchema::knob("clip_hardness", "Clipper Hardness Exponent", 0.1, 5.0, 1.2, "", MacroRole::Tone, "Clipper Hardness Exponent parameter"))
        .with_param(DspParamSchema::knob("oversampling_order", "Anti-Aliasing Oversampling", 1.0, 16.0, 4.0, "x", MacroRole::Character, "Anti-Aliasing Oversampling integer control"))
        .with_param(DspParamSchema::knob("ceiling_db", "True Peak Limit Ceiling", -6.0, 0.0, -0.1, "dBTP", MacroRole::Tone, "True Peak Limit Ceiling parameter"))
    );
    descriptors.insert("TransientDeclickerNode".to_string(), DspNodeDescriptor::new("TransientDeclickerNode", "Surgical Sample De-Clicker & Pop Filter", DspNodeCategory::DynamicsMaster, "Surgical Sample De-Clicker & Pop Filter")
        .with_param(DspParamSchema::knob("sensitivity", "Detection Threshold", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Detection Threshold parameter"))
        .with_param(DspParamSchema::knob("click_width_samples", "Max Click Width", 2.0, 128.0, 16.0, "samples", MacroRole::Character, "Max Click Width integer control"))
        .with_param(DspParamSchema::knob("spectral_inpaint_depth", "Inpainting Reconstruction", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Inpainting Reconstruction parameter"))
        .with_param(DspParamSchema::toggle("listen_delta", "Audition Isolated Clicks", false, "Audition Isolated Clicks toggle switch"))
    );
    descriptors.insert("TransientDesignerNode".to_string(), DspNodeDescriptor::new("TransientDesignerNode", "Dual-Band Attack & Sustain Transient Designer", DspNodeCategory::DynamicsMaster, "Dual-Band Attack & Sustain Transient Designer")
        .with_param(DspParamSchema::knob("attack_gain_db", "Attack Transient Gain", -15.0, 15.0, 3.5, "dB", MacroRole::Tone, "Attack Transient Gain parameter"))
        .with_param(DspParamSchema::knob("sustain_gain_db", "Sustain Tail Gain", -15.0, 15.0, -2.0, "dB", MacroRole::Tone, "Sustain Tail Gain parameter"))
        .with_param(DspParamSchema::log_knob("split_frequency_hz", "Band Split Frequency", 100.0, 5000.0, 1200.0, "Hz", MacroRole::Tone, "Band Split Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("output_gain_db", "Master Output Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Master Output Level parameter"))
    );
    descriptors.insert("TransientGateNode".to_string(), DspNodeDescriptor::new("TransientGateNode", "Envelope-Triggered Transient Noise Gate", DspNodeCategory::DynamicsMaster, "Envelope-Triggered Transient Noise Gate")
        .with_param(DspParamSchema::knob("threshold_db", "Trigger Threshold", -70.0, 0.0, -40.0, "dB", MacroRole::Tone, "Trigger Threshold parameter"))
        .with_param(DspParamSchema::knob("attack_fast_ms", "Fast Attack Speed", 0.01, 10.0, 0.5, "ms", MacroRole::Tone, "Fast Attack Speed parameter"))
        .with_param(DspParamSchema::knob("hold_time_ms", "Hold Gate Duration", 0.0, 250.0, 15.0, "ms", MacroRole::Tone, "Hold Gate Duration parameter"))
        .with_param(DspParamSchema::knob("floor_attenuation_db", "Gate Attenuation Floor", -80.0, 0.0, -50.0, "dB", MacroRole::Tone, "Gate Attenuation Floor parameter"))
    );
    descriptors.insert("TransientReconstructorNode".to_string(), DspNodeDescriptor::new("TransientReconstructorNode", "Spectral Attack Transient Reconstructor", DspNodeCategory::DynamicsMaster, "Spectral Attack Transient Reconstructor")
        .with_param(DspParamSchema::knob("attack_restoration_db", "Attack Restoration Boost", 0.0, 18.0, 5.0, "dB", MacroRole::Tone, "Attack Restoration Boost parameter"))
        .with_param(DspParamSchema::knob("harmonic_synthesis_q", "Harmonic Phase Match Q", 1.0, 20.0, 6.0, "Q", MacroRole::Tone, "Harmonic Phase Match Q parameter"))
        .with_param(DspParamSchema::knob("onset_sensitivity", "Onset Trigger Sensitivity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Onset Trigger Sensitivity parameter"))
        .with_param(DspParamSchema::knob("wet_dry", "Reconstructed Blend", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Reconstructed Blend parameter"))
    );
    descriptors.insert("TransientShaperNode".to_string(), DspNodeDescriptor::new("TransientShaperNode", "Zero-Latency Transient Punch & Sustain Shaper", DspNodeCategory::DynamicsMaster, "Zero-Latency Transient Punch & Sustain Shaper")
        .with_param(DspParamSchema::knob("punch_level_db", "Transient Punch Level", -12.0, 12.0, 4.0, "dB", MacroRole::Tone, "Transient Punch Level parameter"))
        .with_param(DspParamSchema::knob("body_level_db", "Body / Sustain Tail Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Body / Sustain Tail Level parameter"))
        .with_param(DspParamSchema::knob("recovery_speed_ms", "Transient Recovery Speed", 5.0, 200.0, 45.0, "ms", MacroRole::Tone, "Transient Recovery Speed parameter"))
        .with_param(DspParamSchema::toggle("clip_protection", "Soft Clip Brickwall Guard", true, "Soft Clip Brickwall Guard toggle switch"))
    );
    descriptors.insert("TransientUnwrapperNode".to_string(), DspNodeDescriptor::new("TransientUnwrapperNode", "De-Compression & Dynamic Unwrapper", DspNodeCategory::DynamicsMaster, "De-Compression & Dynamic Unwrapper")
        .with_param(DspParamSchema::knob("crest_expansion_ratio", "Dynamic Unwrapping Ratio", 1.0, 3.5, 1.6, "x", MacroRole::Tone, "Dynamic Unwrapping Ratio parameter"))
        .with_param(DspParamSchema::knob("unwrapper_threshold_db", "Unwrapping Floor Level", -36.0, -6.0, -18.0, "dB", MacroRole::Tone, "Unwrapping Floor Level parameter"))
        .with_param(DspParamSchema::toggle("transient_preserve", "Preserve Fast Transients", true, "Preserve Fast Transients toggle switch"))
        .with_param(DspParamSchema::knob("gain_trim_db", "Post-Unwrap Output Trim", -12.0, 12.0, -1.0, "dB", MacroRole::Tone, "Post-Unwrap Output Trim parameter"))
    );
    descriptors.insert("LimiterNode".to_string(), DspNodeDescriptor::new("LimiterNode", "True-Peak Lookahead Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster, "True-Peak Lookahead Mastering Brickwall Limiter")
        .with_param(DspParamSchema::knob("ceiling", "True Peak Ceiling", -12.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "True Peak Ceiling parameter"))
        .with_param(DspParamSchema::knob("threshold", "Limiter Threshold", -24.0, 0.0, -3.0, "dB", MacroRole::Tone, "Limiter Threshold parameter"))
        .with_param(DspParamSchema::knob("release_ms", "Lookahead Release", 5.0, 500.0, 50.0, "ms", MacroRole::Tone, "Lookahead Release parameter"))
        .with_param(DspParamSchema::toggle("oversampling", "4x ISP Oversampling", true, "4x ISP Oversampling toggle switch"))
        .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer", 0.5, 10.0, 3.0, "ms", MacroRole::Tone, "Lookahead Buffer parameter"))
    );
    descriptors.insert("EbuLoudnessRadar".to_string(), DspNodeDescriptor::new("EbuLoudnessRadar", "ITU-R BS.1770 / EBU R128 Loudness Radar Meter", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 / EBU R128 Loudness Radar Meter")
        .with_param(DspParamSchema::knob("target_lufs", "Target Integrated Loudness", -24.0, -9.0, -14.0, "LUFS", MacroRole::Tone, "Target Integrated Loudness parameter"))
        .with_param(DspParamSchema::knob("integrated_gate", "Relative Silence Gate", -70.0, -10.0, -23.0, "LUFS", MacroRole::Tone, "Relative Silence Gate parameter"))
        .with_param(DspParamSchema::knob("max_momentary", "Max Momentary True Peak", -18.0, 0.0, -1.0, "dBTP", MacroRole::Tone, "Max Momentary True Peak parameter"))
    );
    descriptors.insert("LufsMeterNode".to_string(), DspNodeDescriptor::new("LufsMeterNode", "Multi-Scale Momentary & Integrated LUFS Meter", DspNodeCategory::DynamicsMaster, "Multi-Scale Momentary & Integrated LUFS Meter")
        .with_param(DspParamSchema::knob("momentary_window", "Momentary Window", 100.0, 1000.0, 400.0, "ms", MacroRole::Tone, "Momentary Window parameter"))
        .with_param(DspParamSchema::knob("short_term_window", "Short-Term Window", 1000.0, 5000.0, 3000.0, "ms", MacroRole::Tone, "Short-Term Window parameter"))
        .with_param(DspParamSchema::knob("relative_threshold", "Relative Gate Threshold", -20.0, -5.0, -10.0, "LU", MacroRole::Tone, "Relative Gate Threshold parameter"))
    );
    descriptors.insert("MidSideNode".to_string(), DspNodeDescriptor::new("MidSideNode", "Mid/Side Matrix Encoder & Stereo Width Imager", DspNodeCategory::DynamicsMaster, "Mid/Side Matrix Encoder & Stereo Width Imager")
        .with_param(DspParamSchema::knob("width", "Stereo Field Width", 0.0, 3.0, 1.0, "x", MacroRole::Tone, "Stereo Field Width parameter"))
        .with_param(DspParamSchema::knob("mid_gain", "Mid Channel Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Mid Channel Gain parameter"))
        .with_param(DspParamSchema::knob("side_gain", "Side Channel Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Side Channel Gain parameter"))
        .with_param(DspParamSchema::log_knob("mono_sub", "Mono Bass Crossover", 20.0, 250.0, 90.0, "Hz", MacroRole::Tone, "Mono Bass Crossover logarithmic parameter"))
    );
    descriptors.insert("StereoImager".to_string(), DspNodeDescriptor::new("StereoImager", "Phase-Safe Psychoacoustic Stereo Imager", DspNodeCategory::DynamicsMaster, "Phase-Safe Psychoacoustic Stereo Imager")
        .with_param(DspParamSchema::knob("width_factor", "Stereo Width Expansion", 0.0, 2.5, 1.2, "x", MacroRole::Tone, "Stereo Width Expansion parameter"))
        .with_param(DspParamSchema::log_knob("bass_mono_cutoff", "Bass Mono Filter Corner", 30.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Bass Mono Filter Corner logarithmic parameter"))
        .with_param(DspParamSchema::toggle("phase_correlation_protect", "Phase Correlation Protection", true, "Phase Correlation Protection toggle switch"))
    );
    descriptors.insert("TpdfDitherNode".to_string(), DspNodeDescriptor::new("TpdfDitherNode", "Triangular PDF Dither & Noise Shaping Quantizer", DspNodeCategory::DynamicsMaster, "Triangular PDF Dither & Noise Shaping Quantizer")
        .with_param(DspParamSchema::choice("bit_depth", "Target Bit Depth", &["24-bit HD", "16-bit Red Book CD", "12-bit Vintage", "8-bit Lo-Fi"], 1, "Target Bit Depth mode selector"))
        .with_param(DspParamSchema::choice("noise_shaping", "Noise Shaping Curve", &["None (Flat TPDF)", "Lipshitz Moderate", "High-Pass Ultra", "Psychoacoustic F-Weight"], 1, "Noise Shaping Curve mode selector"))
        .with_param(DspParamSchema::toggle("auto_blank", "Auto Blank On Digital Silence", true, "Auto Blank On Digital Silence toggle switch"))
    );
    descriptors.insert("AdaptivePsychoacousticLoudness".to_string(), DspNodeDescriptor::new("AdaptivePsychoacousticLoudness", "ISO 226 Equal Loudness Contour Balancing Engine", DspNodeCategory::DynamicsMaster, "ISO 226 Equal Loudness Contour Balancing Engine")
        .with_param(DspParamSchema::knob("target_phon", "Target Phon Loudness", 40.0, 100.0, 75.0, "phon", MacroRole::Tone, "Target Phon Loudness parameter"))
        .with_param(DspParamSchema::knob("phon_compensation_intensity", "Equal Loudness Compensation", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Equal Loudness Compensation parameter"))
        .with_param(DspParamSchema::knob("low_contour_boost_db", "Fletcher-Munson Sub Boost", 0.0, 12.0, 3.5, "dB", MacroRole::Tone, "Fletcher-Munson Sub Boost parameter"))
        .with_param(DspParamSchema::knob("high_contour_presence_db", "Treble Contour Presence", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Treble Contour Presence parameter"))
    );
    descriptors.insert("DynamicStereoWidthNode".to_string(), DspNodeDescriptor::new("DynamicStereoWidthNode", "Frequency-Dependent Mid/Side Dynamic Stereo Widener", DspNodeCategory::DynamicsMaster, "Frequency-Dependent Mid/Side Dynamic Stereo Widener")
        .with_param(DspParamSchema::knob("stereo_expansion_factor", "Side Band Expansion", 0.0, 3.0, 1.35, "x", MacroRole::Tone, "Side Band Expansion parameter"))
        .with_param(DspParamSchema::log_knob("bass_mono_crossover_hz", "Mono Bass Frequency Cut", 30.0, 250.0, 100.0, "Hz", MacroRole::Tone, "Mono Bass Frequency Cut logarithmic parameter"))
        .with_param(DspParamSchema::knob("high_side_air_gain_db", "Side Channel Air Boost", 0.0, 6.0, 1.5, "dB", MacroRole::Tone, "Side Channel Air Boost parameter"))
        .with_param(DspParamSchema::toggle("phase_correlation_protect", "Auto Anti-Phase Limiter", true, "Auto Anti-Phase Limiter toggle switch"))
    );
    descriptors.insert("KSystemMeterNode".to_string(), DspNodeDescriptor::new("KSystemMeterNode", "Bob Katz K-12 / K-14 / K-20 Metering Bridge", DspNodeCategory::DynamicsMaster, "Bob Katz K-12 / K-14 / K-20 Metering Bridge")
        .with_param(DspParamSchema::choice("meter_scale", "K-Scale Target Reference", &["K-12 (Broadcast / Pop)", "K-14 (Standard Music)", "K-20 (Audiophile / Film Dynamic)"], 1, "K-Scale Target Reference mode selector"))
        .with_param(DspParamSchema::knob("headroom_target_db", "0 VU Calibration Offset", 0.0, 20.0, 14.0, "dB", MacroRole::Tone, "0 VU Calibration Offset parameter"))
        .with_param(DspParamSchema::knob("peak_hold_time_ms", "Peak Hold Persistence", 100.0, 3000.0, 1500.0, "ms", MacroRole::Tone, "Peak Hold Persistence parameter"))
        .with_param(DspParamSchema::knob("monitor_calibration_db", "SPL Reference Calibration", 60.0, 90.0, 83.0, "dBSPL", MacroRole::Tone, "SPL Reference Calibration parameter"))
    );
    descriptors.insert("EqualLoudnessContourNode".to_string(), DspNodeDescriptor::new("EqualLoudnessContourNode", "Fletcher-Munson Psychoacoustic Equal Loudness Compensator", DspNodeCategory::DynamicsMaster, "Fletcher-Munson Psychoacoustic Equal Loudness Compensator")
        .with_param(DspParamSchema::knob("phon_level", "Listening Volume (Phon)", 30.0, 100.0, 80.0, "phon", MacroRole::Tone, "Listening Volume (Phon) parameter"))
        .with_param(DspParamSchema::choice("curve_version", "Contour Standard", &["ISO 226:2003 Standard", "Fletcher-Munson (1933)", "Robinson-Dadson (1956)"], 0, "Contour Standard mode selector"))
        .with_param(DspParamSchema::knob("bass_contour_trim", "Low Contour Trim", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "Low Contour Trim parameter"))
        .with_param(DspParamSchema::knob("treble_contour_trim", "High Contour Trim", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "High Contour Trim parameter"))
    );
    descriptors.insert("MasterLimiterRadarNode".to_string(), DspNodeDescriptor::new("MasterLimiterRadarNode", "True-Peak Limiter with Integrated Loudness Radar", DspNodeCategory::DynamicsMaster, "True-Peak Limiter with Integrated Loudness Radar")
        .with_param(DspParamSchema::knob("true_peak_ceiling_dbtp", "True-Peak Ceiling", -6.0, 0.0, -0.3, "dBTP", MacroRole::Tone, "True-Peak Ceiling parameter"))
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Integrated LUFS", -24.0, -8.0, -14.0, "LUFS", MacroRole::Tone, "Target Integrated LUFS parameter"))
        .with_param(DspParamSchema::knob("lookahead_time_ms", "True-Peak Lookahead", 1.0, 10.0, 4.0, "ms", MacroRole::Tone, "True-Peak Lookahead parameter"))
        .with_param(DspParamSchema::toggle("release_curve_adaptive", "Adaptive Program Release", true, "Adaptive Program Release toggle switch"))
    );
    descriptors.insert("MeterBridgeNode".to_string(), DspNodeDescriptor::new("MeterBridgeNode", "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge", DspNodeCategory::DynamicsMaster, "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge")
        .with_param(DspParamSchema::choice("meter_type", "Ballistics Standard", &["True-Peak ISP (ITU-R BS.1770)", "VU Meter (ANSI C16.5)", "Nordic PPM (IEC 60268-10)", "BBC Type II PPM"], 0, "Ballistics Standard mode selector"))
        .with_param(DspParamSchema::knob("integration_window_ms", "RMS Integration Window", 50.0, 1000.0, 300.0, "ms", MacroRole::Tone, "RMS Integration Window parameter"))
        .with_param(DspParamSchema::knob("vu_reference_db", "VU Zero Reference", -24.0, -10.0, -18.0, "dBFS", MacroRole::Tone, "VU Zero Reference parameter"))
        .with_param(DspParamSchema::knob("peak_decay_speed", "Decay Ballistics Rate", 5.0, 60.0, 20.0, "dB/s", MacroRole::Tone, "Decay Ballistics Rate parameter"))
    );
    descriptors.insert("MidSideFocuserNode".to_string(), DspNodeDescriptor::new("MidSideFocuserNode", "Surgical Mid/Side Bass Mono & High-Side Air Focuser", DspNodeCategory::DynamicsMaster, "Surgical Mid/Side Bass Mono & High-Side Air Focuser")
        .with_param(DspParamSchema::log_knob("bass_mono_freq_hz", "Mono Bass Frequency Cut", 30.0, 300.0, 110.0, "Hz", MacroRole::Tone, "Mono Bass Frequency Cut logarithmic parameter"))
        .with_param(DspParamSchema::knob("high_side_boost_db", "Side Channel Air Sheen", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Side Channel Air Sheen parameter"))
        .with_param(DspParamSchema::knob("side_dynamic_compression", "Side Band Dynamic Tame", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Side Band Dynamic Tame parameter"))
        .with_param(DspParamSchema::knob("mid_presence_gain_db", "Mid Channel Vocal Presence", -6.0, 6.0, 0.8, "dB", MacroRole::Tone, "Mid Channel Vocal Presence parameter"))
    );
    descriptors.insert("MultibandImagerNode".to_string(), DspNodeDescriptor::new("MultibandImagerNode", "4-Band Mastering Stereo Width & Phase Correlator", DspNodeCategory::DynamicsMaster, "4-Band Mastering Stereo Width & Phase Correlator")
        .with_param(DspParamSchema::knob("band1_sub_width", "Sub Width (<120Hz)", 0.0, 2.0, 0.0, "x", MacroRole::Tone, "Sub Width (<120Hz) parameter"))
        .with_param(DspParamSchema::knob("band2_low_width", "Low-Mid Width (120-1kHz)", 0.0, 2.0, 0.9, "x", MacroRole::Tone, "Low-Mid Width (120-1kHz) parameter"))
        .with_param(DspParamSchema::knob("band3_mid_width", "High-Mid Width (1k-6kHz)", 0.0, 2.0, 1.25, "x", MacroRole::Tone, "High-Mid Width (1k-6kHz) parameter"))
        .with_param(DspParamSchema::knob("band4_high_width", "Air Width (>6kHz)", 0.0, 2.0, 1.4, "x", MacroRole::Tone, "Air Width (>6kHz) parameter"))
    );
    descriptors.insert("OversampledLimiterNode".to_string(), DspNodeDescriptor::new("OversampledLimiterNode", "16x Polyphase Oversampled Intersample Peak Limiter", DspNodeCategory::DynamicsMaster, "16x Polyphase Oversampled Intersample Peak Limiter")
        .with_param(DspParamSchema::choice("oversampling_rate", "Polyphase Oversampling", &["2x Polyphase", "4x High Quality", "8x Mastering Grade", "16x Ultra Fidelity"], 2, "Polyphase Oversampling mode selector"))
        .with_param(DspParamSchema::knob("brickwall_ceiling_dbfs", "Ceiling Level", -6.0, 0.0, -0.2, "dBFS", MacroRole::Tone, "Ceiling Level parameter"))
        .with_param(DspParamSchema::knob("release_time_ms", "Lookahead Release Speed", 5.0, 400.0, 45.0, "ms", MacroRole::Tone, "Lookahead Release Speed parameter"))
        .with_param(DspParamSchema::toggle("isp_guard_active", "Intersample Peak Guard", true, "Intersample Peak Guard toggle switch"))
    );
    descriptors.insert("PolarPhaseCorrelatorNode".to_string(), DspNodeDescriptor::new("PolarPhaseCorrelatorNode", "Polar Goniometer Lissajous Phase Correlation Display", DspNodeCategory::DynamicsMaster, "Polar Goniometer Lissajous Phase Correlation Display")
        .with_param(DspParamSchema::knob("phosphor_persistence_ms", "Goniometer Phosphor Decay", 50.0, 1000.0, 300.0, "ms", MacroRole::Tone, "Goniometer Phosphor Decay parameter"))
        .with_param(DspParamSchema::knob("polar_scaling", "Polar Radius Scale", 0.5, 3.0, 1.0, "x", MacroRole::Tone, "Polar Radius Scale parameter"))
        .with_param(DspParamSchema::knob("lissajous_resolution", "Display Point Resolution", 128.0, 2048.0, 512.0, "pts", MacroRole::Character, "Display Point Resolution integer control"))
        .with_param(DspParamSchema::toggle("phase_cancel_alert", "Phase Inversion Warning", true, "Phase Inversion Warning toggle switch"))
    );
    descriptors.insert("ResonanceSuppressorNode".to_string(), DspNodeDescriptor::new("ResonanceSuppressorNode", "Automatic Dynamic Harmonic Notch Resonance Suppressor", DspNodeCategory::DynamicsMaster, "Automatic Dynamic Harmonic Notch Resonance Suppressor")
        .with_param(DspParamSchema::knob("suppression_depth_db", "Dynamic Notch Depth", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Dynamic Notch Depth parameter"))
        .with_param(DspParamSchema::knob("notch_q_sharpness", "Dynamic Q Sharpness", 2.0, 50.0, 15.0, "Q", MacroRole::Tone, "Dynamic Q Sharpness parameter"))
        .with_param(DspParamSchema::knob("attack_speed_ms", "Resonance Lock Attack", 1.0, 100.0, 12.0, "ms", MacroRole::Tone, "Resonance Lock Attack parameter"))
        .with_param(DspParamSchema::knob("max_active_notches", "Concurrent Notch Filters", 1.0, 16.0, 6.0, "filters", MacroRole::Character, "Concurrent Notch Filters integer control"))
    );
    descriptors.insert("StereoWidenerNode".to_string(), DspNodeDescriptor::new("StereoWidenerNode", "Haas Effect & Phase-Coherent Stereo Field Widener", DspNodeCategory::DynamicsMaster, "Haas Effect & Phase-Coherent Stereo Field Widener")
        .with_param(DspParamSchema::knob("haas_delay_ms", "Haas Delay Offset", 0.1, 30.0, 12.0, "ms", MacroRole::Tone, "Haas Delay Offset parameter"))
        .with_param(DspParamSchema::knob("mid_side_ratio", "Side vs Mid Ratio", 0.0, 3.0, 1.25, "x", MacroRole::Tone, "Side vs Mid Ratio parameter"))
        .with_param(DspParamSchema::knob("center_channel_retention", "Center Lead Vocal Retain", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Center Lead Vocal Retain parameter"))
        .with_param(DspParamSchema::knob("phase_filter_q", "Allpass Phase Filter Q", 0.5, 5.0, 1.2, "Q", MacroRole::Tone, "Allpass Phase Filter Q parameter"))
    );
    descriptors.insert("TapeFluxMasterNode".to_string(), DspNodeDescriptor::new("TapeFluxMasterNode", "Half-Inch Mastering Tape Saturation & Flux Density Engine", DspNodeCategory::DynamicsMaster, "Half-Inch Mastering Tape Saturation & Flux Density Engine")
        .with_param(DspParamSchema::knob("tape_flux_nanoweber", "Reference Fluxivity", 185.0, 510.0, 355.0, "nWb/m", MacroRole::Tone, "Reference Fluxivity parameter"))
        .with_param(DspParamSchema::choice("tape_formulation", "Mastering Tape Stock", &["SM900 High Output (+9dB)", "ATR Magnetics Master (+6dB)", "GP9 Grand Master (+9dB)", "Scotch 250 Vintage (+3dB)"], 0, "Mastering Tape Stock mode selector"))
        .with_param(DspParamSchema::knob("bias_hf_saturation", "High-Frequency Bias Saturation", -3.0, 6.0, 1.5, "dB", MacroRole::Tone, "High-Frequency Bias Saturation parameter"))
        .with_param(DspParamSchema::knob("flux_compression_db", "Head Hysteresis Compression", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Head Hysteresis Compression parameter"))
    );
    descriptors.insert("DialogGatingNode".to_string(), DspNodeDescriptor::new("DialogGatingNode", "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine")
        .with_param(DspParamSchema::knob("speech_presence_threshold_db", "Speech Presence Gate", -40.0, -10.0, -24.0, "dB", MacroRole::Tone, "Speech Presence Gate parameter"))
        .with_param(DspParamSchema::knob("gating_window_ms", "Voice Energy Gating Window", 100.0, 1000.0, 400.0, "ms", MacroRole::Tone, "Voice Energy Gating Window parameter"))
        .with_param(DspParamSchema::knob("ambient_noise_attenuation_db", "Background Attenuation", -30.0, 0.0, -12.0, "dB", MacroRole::Tone, "Background Attenuation parameter"))
        .with_param(DspParamSchema::knob("voice_clarity_lift_db", "Formant Intelligibility Lift", 0.0, 6.0, 2.0, "dB", MacroRole::Tone, "Formant Intelligibility Lift parameter"))
    );
    descriptors.insert("SonarHydrophoneNode".to_string(), DspNodeDescriptor::new("SonarHydrophoneNode", "Underwater Acoustic Hydrophone & Thermal Layer Simulation", DspNodeCategory::DynamicsMaster, "Underwater Acoustic Hydrophone & Thermal Layer Simulation")
        .with_param(DspParamSchema::knob("hydrophone_depth_m", "Sensor Water Depth", 1.0, 1000.0, 75.0, "m", MacroRole::Tone, "Sensor Water Depth parameter"))
        .with_param(DspParamSchema::knob("water_salinity_ppt", "Ocean Salinity", 0.0, 45.0, 35.0, "ppt", MacroRole::Tone, "Ocean Salinity parameter"))
        .with_param(DspParamSchema::knob("sound_channel_axis_depth_m", "SOFAR Channel Axis Depth", 100.0, 1500.0, 800.0, "m", MacroRole::Tone, "SOFAR Channel Axis Depth parameter"))
        .with_param(DspParamSchema::knob("cavitation_noise_level", "Propeller Cavitation Ambient", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Propeller Cavitation Ambient parameter"))
    );
    descriptors.insert("AiAutonomousMasteringEngine".to_string(), DspNodeDescriptor::new("AiAutonomousMasteringEngine", "AI Autonomous Mastering Engine", DspNodeCategory::DynamicsMaster, "Autonomous EBU R128 Spectral Matching & Dynamic Mastering Engine")
        .with_param(DspParamSchema::choice("target_curve", "Target Curve", &["Modern Pop", "EDM Club", "Warm Vintage", "Acoustic Folk", "Classical Concert"], 0, "Target spectral weighting curve selector"))
        .with_param(DspParamSchema::knob("target_lufs", "Target LUFS", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Integrated loudness target level parameter"))
        .with_param(DspParamSchema::knob("ceiling_db", "Ceiling dBFS", -2.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "True peak limiter threshold ceiling parameter"))
        .with_param(DspParamSchema::knob("warmth", "Low-End Warmth", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Sub-bass harmonic density reinforcement parameter"))
        .with_param(DspParamSchema::knob("punch", "Transient Punch", 0.0, 1.0, 0.50, "%", MacroRole::Punch, "Micro-dynamic crest factor enhancement parameter"))
        .with_param(DspParamSchema::knob("stereo_width", "Stereo Width", 0.5, 2.0, 1.15, "x", MacroRole::Space, "Mid-Side spatial stage expansion factor parameter"))
    );
    descriptors.insert("TruePeakLimiter".to_string(), DspNodeDescriptor::new("TruePeakLimiter", "ITU-R BS.1770 True-Peak Limiter & 4x Clipper", DspNodeCategory::DynamicsMaster, "4x polyphase oversampled true-peak brickwall limiter with lookahead buffer, ISP protection, and soft-knee safety clipper")
        .with_param(DspParamSchema::knob("threshold_db", "Threshold", -36.0, 0.0, -0.1, "dB", MacroRole::Punch, "Limiter gain reduction threshold level parameter"))
        .with_param(DspParamSchema::knob("ceiling_db", "Ceiling", -24.0, 0.0, -0.1, "dBTP", MacroRole::Tone, "Inter-sample true peak output ceiling parameter"))
        .with_param(DspParamSchema::knob("knee_db", "Soft Knee", 0.0, 12.0, 1.5, "dB", MacroRole::Tone, "Transition curve smoothing knee parameter"))
        .with_param(DspParamSchema::knob("release_ms", "Release Time", 1.0, 1000.0, 50.0, "ms", MacroRole::Character, "Envelope gain recovery release rate parameter"))
        .with_param(DspParamSchema::knob("lookahead_samples", "Lookahead", 1.0, 255.0, 64.0, "spl", MacroRole::Tone, "Zero-latency lookahead delay circular buffer size parameter"))
        .with_param(DspParamSchema::toggle("isp_protection", "ISP Protection", true, "ITU-R BS.1770 polyphase inter-sample peak detection toggle"))
    );
    descriptors.insert("SurroundLimiterAndLoudness".to_string(), DspNodeDescriptor::new("SurroundLimiterAndLoudness", "7.1.4 / 9.1.6 Multi-Channel True-Peak Surround Limiter", DspNodeCategory::DynamicsMaster, "Mastering-grade multi-channel lookahead limiter with inter-channel linking, true-peak detection, and integrated EBU R128 loudness targeting")
        .with_param(DspParamSchema::knob("target_lufs", "Integrated Target LUFS", -36.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Target program loudness normalization threshold parameter"))
        .with_param(DspParamSchema::knob("ceiling_dbtp", "True-Peak Ceiling", -6.0, 0.0, -0.5, "dBTP", MacroRole::Tone, "Maximum allowable inter-sample true-peak limit parameter"))
        .with_param(DspParamSchema::knob("release_ms", "Limiter Release", 5.0, 1000.0, 50.0, "ms", MacroRole::Punch, "Gain recovery envelope release speed parameter"))
        .with_param(DspParamSchema::knob("interchannel_linking", "Surround Channel Link", 0.0, 1.0, 0.80, "%", MacroRole::Space, "Prevents spatial image collapse by linking surround channel gain reductions parameter"))
        .with_param(DspParamSchema::toggle("true_peak_enable", "True-Peak Oversampling", true, "Enables 4x polyphase inter-sample true-peak detection toggle"))
    );
    descriptors.insert("PeakHeadroomAnalyzer".to_string(), DspNodeDescriptor::new("PeakHeadroomAnalyzer", "Real-Time Inter-Sample True Peak & Headroom Analyzer", DspNodeCategory::DynamicsMaster, "4x/8x oversampled true peak headroom analyzer calculating exact inter-sample peak overshoots and headroom safety margins")
        .with_param(DspParamSchema::knob("target_headroom_db", "Target Headroom", 0.0, 12.0, 1.0, "dB", MacroRole::Punch, "Target headroom safety threshold parameter"))
        .with_param(DspParamSchema::knob("peak_sample_db", "Sample Peak Level", -120.0, 6.0, -12.0, "dB", MacroRole::Tone, "Raw sample peak reading parameter"))
        .with_param(DspParamSchema::knob("true_peak_db", "Inter-Sample True Peak", -120.0, 6.0, -11.5, "dB", MacroRole::Space, "Reconstructed continuous true peak level parameter"))
        .with_param(DspParamSchema::toggle("oversampling_mode", "8x Sinc Oversampling", true, "Enable high-precision 8x polyphase sinc filter oversampling parameter"))
    );
    descriptors.insert("EbuR128LoudnessMeter".to_string(), DspNodeDescriptor::new("EbuR128LoudnessMeter", "EBU R128 & ITU-R BS.1770 Broadcast Loudness Radar & Meter", DspNodeCategory::DynamicsMaster, "Compliant broadcast loudness analyzer computing integrated, short-term, and momentary LUFS with loudness range")
        .with_param(DspParamSchema::knob("target_lufs", "Target Loudness", -36.0, -6.0, -23.0, "LUFS", MacroRole::Punch, "Target broadcast delivery standard loudness parameter"))
        .with_param(DspParamSchema::knob("integrated_lufs", "Integrated LUFS", -70.0, 0.0, -23.0, "LUFS", MacroRole::Tone, "Long-term program gated integrated loudness parameter"))
        .with_param(DspParamSchema::knob("momentary_lufs", "Momentary LUFS", -70.0, 6.0, -21.0, "LUFS", MacroRole::Character, "400ms sliding window instantaneous loudness parameter"))
        .with_param(DspParamSchema::knob("short_term_lufs", "Short-Term LUFS", -70.0, 6.0, -22.5, "LUFS", MacroRole::Space, "3-second sliding window short-term loudness parameter"))
    );
    descriptors.insert("SidechainMatrix".to_string(), DspNodeDescriptor::new("SidechainMatrix", "Multi-Channel Dynamic Sidechain Matrix & Routing Cross-Bar Hub", DspNodeCategory::DynamicsMaster, "16-bus dynamic sidechain matrix routing key signals with independent highpass/lowpass frequency conditioning")
        .with_param(DspParamSchema::knob("source_bus_index", "Sidechain Key Source Bus", 1.0, 16.0, 1.0, "bus", MacroRole::Punch, "Selects the feeding track or submix bus parameter"))
        .with_param(DspParamSchema::log_knob("sidechain_highpass_hz", "Key Highpass Filter", 20.0, 1000.0, 120.0, "Hz", MacroRole::Tone, "Filters low rumble from trigger detector parameter"))
        .with_param(DspParamSchema::log_knob("sidechain_lowpass_hz", "Key Lowpass Filter", 1000.0, 20000.0, 12000.0, "Hz", MacroRole::Character, "Filters harsh transients from trigger detector parameter"))
        .with_param(DspParamSchema::toggle("key_listen", "Key Signal Audition", false, "Listen to filtered sidechain key signal in isolation parameter"))
    );
    descriptors.insert("BandDynamics".to_string(), DspNodeDescriptor::new("BandDynamics", "Per-Band Multiband Dynamic Compressor & Expander Stage", DspNodeCategory::DynamicsMaster, "Precision dynamic processor sub-band stage with independent threshold, ratio, attack, release, and upward/downward transfer function curves")
        .with_param(DspParamSchema::knob("threshold_db", "Band Threshold", -60.0, 0.0, -18.0, "dB", MacroRole::Punch, "Band dynamic compressor/expander threshold parameter"))
        .with_param(DspParamSchema::knob("compression_ratio", "Compression Ratio", 1.0, 20.0, 3.5, ":1", MacroRole::Punch, "Downward dynamic compression slope ratio parameter"))
        .with_param(DspParamSchema::knob("attack_time_ms", "Attack Response", 0.1, 250.0, 15.0, "ms", MacroRole::Punch, "Peak detector dynamic attack smoothing parameter"))
        .with_param(DspParamSchema::knob("release_time_ms", "Release Response", 5.0, 1500.0, 120.0, "ms", MacroRole::Space, "Dynamic gain recovery release envelope parameter"))
        .with_param(DspParamSchema::knob("expansion_ratio", "Expansion Ratio", 0.2, 1.0, 1.0, "exp", MacroRole::Tone, "Downward/upward expansion transfer ratio parameter"))
        .with_param(DspParamSchema::knob("makeup_gain_db", "Band Makeup Gain", -12.0, 24.0, 2.5, "dB", MacroRole::Character, "Post-dynamics band output level makeup gain parameter"))
    );
    descriptors.insert("MasterLimiter".to_string(), DspNodeDescriptor::new("MasterLimiter", "Lookahead Inter-Sample True-Peak Brickwall Mastering Limiter", DspNodeCategory::DynamicsMaster, "Lookahead inter-sample true-peak brickwall mastering limiter with multi-stage release envelopes and soft saturation knee")
        .with_param(DspParamSchema::knob("threshold_db", "Limiter Ceiling Threshold", -24.0, 0.0, -0.2, "dB", MacroRole::Punch, "Peak ceiling threshold for brickwall limiting"))
        .with_param(DspParamSchema::knob("input_gain_db", "Pre-Limiter Input Boost", 0.0, 24.0, 4.0, "dB", MacroRole::Punch, "Input gain pushing program material into limiter"))
        .with_param(DspParamSchema::knob("lookahead_ms", "True Peak Lookahead Time", 0.5, 10.0, 4.0, "ms", MacroRole::Tone, "Lookahead delay buffer allowing zero-overshoot gain reduction"))
        .with_param(DspParamSchema::knob("release_time_ms", "Auto Dynamic Release Time", 10.0, 1000.0, 120.0, "ms", MacroRole::Space, "Gain recovery release envelope smoothing time"))
        .with_param(DspParamSchema::knob("soft_knee_db", "Soft Knee Transition", 0.0, 6.0, 1.5, "dB", MacroRole::Tone, "Curvature width of compression-to-limiting transition knee"))
        .with_param(DspParamSchema::toggle("true_peak_oversampling", "4x Inter-Sample Oversampling", true, "Enables 4x polyphase FIR oversampling for true-peak detection"))
    );
    descriptors.insert("SidechainRoute".to_string(), DspNodeDescriptor::new("SidechainRoute", "Lock-Free Dynamic Sidechain Ducking & Routing Matrix", DspNodeCategory::DynamicsMaster, "Multi-channel sidechain ducking route matrix feeding kick, snare, or dialog signals to compressor sidechain detectors with frequency band splitting")
        .with_param(DspParamSchema::knob("sidechain_source_ch", "Sidechain Source Bus ID", 1.0, 16.0, 1.0, "bus", MacroRole::Tone, "Internal audio bus transmitting control trigger signal"))
        .with_param(DspParamSchema::knob("ducking_amount_db", "Gain Reduction Depth", 0.0, 36.0, 12.0, "dB", MacroRole::Punch, "Maximum attenuation applied to ducked target channel"))
        .with_param(DspParamSchema::knob("ducking_release_ms", "Ducking Release Time", 10.0, 1000.0, 120.0, "ms", MacroRole::Tone, "Time required for ducked audio to recover to unity gain"))
        .with_param(DspParamSchema::log_knob("sidechain_hpf_hz", "Sidechain Filter Low Cut", 20.0, 1000.0, 120.0, "Hz", MacroRole::Tone, "High-pass filter cutoff on detector to ignore sub-bass rumble"))
        .with_param(DspParamSchema::toggle("listen_sidechain", "Audition Sidechain Trigger Input", false, "Solo the incoming sidechain trigger key signal"))
    );
    descriptors.insert("ExportPresetManager".to_string(), DspNodeDescriptor::new("ExportPresetManager", "Multi-Format Stem Export & Mastering Batch HUD", DspNodeCategory::DynamicsMaster, "Batch mastering and multi-track stem exporter with broadcast WAV, FLAC, MP3, and OGG encoding, peak normalization, and dither curves")
        .with_param(DspParamSchema::choice("export_sample_rate", "Export Master Audio Sample Rate", &["44100 Hz (CD)", "48000 Hz (Broadcast)", "88200 Hz (High-Res)", "96000 Hz (Studio Pro)", "192000 Hz (Mastering)"], 1, "Target audio sample rate conversion for exported stems and stereo mixdown"))
        .with_param(DspParamSchema::choice("export_bit_depth", "Export Audio Bit Depth & Lossless Codec", &["16-bit PCM", "24-bit PCM", "32-bit Float", "FLAC Lossless", "MP3 320kbps"], 2, "Audio bit depth resolution and codec format applied during export rendering"))
        .with_param(DspParamSchema::knob("peak_normalization_target_db", "True-Peak Master Normalization Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "True-peak ceiling threshold applied during final export normalization pass"))
        .with_param(DspParamSchema::choice("dither_noise_shaping_mode", "Quantization Dither & Noise Shaping Mode", &["None (Truncate)", "Rectangular TPDF", "Triangular TPDF", "High-Pass Shaped Dither"], 2, "Dither algorithm applied when downscaling from 32/64-bit float to 16/24-bit fixed point"))
        .with_param(DspParamSchema::choice("stem_auto_naming_pattern", "Multi-Track Stem File Naming Template", &["{track_num}_{track_name}", "{bpm}_{key}_{track_name}", "{project}_{track_name}_{date}", "Custom Pattern"], 0, "File naming template applied when batch-rendering multi-track individual stems"))
    );
    descriptors.insert("SessionAnalyticsDashboard".to_string(), DspNodeDescriptor::new("SessionAnalyticsDashboard", "Real-Time Telemetry & DSP Headroom HUD", DspNodeCategory::DynamicsMaster, "Real-time session telemetry monitor tracking CPU thread load, DSP render headroom, plugin latency, and edit activity")
        .with_param(DspParamSchema::knob("dsp_load_sampling_window_ms", "DSP Processing Load Averaging Window", 10.0, 1000.0, 100.0, "ms", MacroRole::Space, "Time window used for computing rolling average CPU load and real-time execution budget"))
        .with_param(DspParamSchema::knob("peak_cpu_overrun_alert_threshold_pct", "Audio Buffer Overrun Alert Threshold", 50.0, 99.0, 90.0, "%", MacroRole::Tone, "CPU utilization threshold that triggers warning indicator before audio clicks occur"))
        .with_param(DspParamSchema::toggle("track_memory_allocations_active", "Monitor Real-Time Audio Heap Allocations", true, "Detects and logs any unexpected memory allocation calls occurring within audio thread loop"))
        .with_param(DspParamSchema::toggle("audio_underrun_event_counter_reset", "Zero Cumulative Audio Buffer Dropouts Counter", false, "Resets count of cumulative xruns and hardware audio underrun incidents"))
        .with_param(DspParamSchema::knob("history_graph_retention_minutes", "Telemetry Performance Timeline History Scope", 5.0, 120.0, 30.0, "min", MacroRole::Character, "Duration of CPU, DSP, and latency history retained in the real-time HUD graph"))
    );
    descriptors.insert("GoldenRenderSuite".to_string(), DspNodeDescriptor::new("GoldenRenderSuite", "Bit-Exact DSP Regression Test Harness HUD", DspNodeCategory::DynamicsMaster, "Bit-exact DSP regression test harness comparing offline renders against golden reference wavs with spectral phase validation")
        .with_param(DspParamSchema::choice("golden_reference_sample_rate", "Regression Test Benchmark Sample Rate", &["44100 Hz", "48000 Hz", "96000 Hz", "192000 Hz"], 1, "Sampling frequency used for generating and verifying bit-exact regression render buffers"))
        .with_param(DspParamSchema::knob("rms_tolerance_threshold_db", "Null-Test RMS Cancellation Floor", -140.0, -60.0, -96.0, "dB", MacroRole::Tone, "RMS difference threshold below which two rendered audio signals are considered functionally identical"))
        .with_param(DspParamSchema::knob("spectral_phase_correlation_min", "Minimum Acceptable Phase Correlation", 0.90, 1.0, 0.999, "ratio", MacroRole::Punch, "Cross-correlation coefficient minimum required between reference and generated audio streams"))
        .with_param(DspParamSchema::toggle("generate_spectral_difference_map", "Generate 2D Spectral Difference Waterfall", true, "Produces high-resolution color heatmaps displaying frequency differences if null test fails"))
        .with_param(DspParamSchema::toggle("abort_suite_on_first_divergence", "Halt Test Suite on First Regression Error", false, "Stops execution immediately upon encountering first audio discrepancy to speed up debugging"))
    );
    descriptors.insert("VideoExportConfig".to_string(), DspNodeDescriptor::new("VideoExportConfig", "Video Soundtrack Sync & Video Renderer HUD", DspNodeCategory::DynamicsMaster, "Video soundtrack synchronization and offline hardware-accelerated H.264/ProRes rendering with SMPTE timecode sync")
        .with_param(DspParamSchema::choice("video_codec_encoder", "Hardware Accelerated Video Codec", &["H.264 / AVC Hardware", "H.265 / HEVC 10-bit", "Apple ProRes 422 HQ", "AV1 Open Source"], 0, "Video compression encoder used for exporting synced musical soundtrack video renders"))
        .with_param(DspParamSchema::choice("output_video_resolution", "Video Output Resolution & Aspect Ratio", &["1080p Full HD (1920x1080)", "4K UHD (3840x2160)", "720p HD (1280x720)", "Instagram Square (1080x1080)"], 0, "Video canvas dimension and aspect ratio for social media or theatrical soundtrack release"))
        .with_param(DspParamSchema::choice("video_framerate_fps", "Video Master Timeline Framerate", &["24.0 fps (Film)", "25.0 fps (PAL)", "29.97 fps (NTSC)", "30.0 fps", "60.0 fps (Smooth)"], 3, "Output framerate synchronizing audio frame buffer boundaries with video display frames"))
        .with_param(DspParamSchema::toggle("render_waveform_overlay", "Burn Oscilloscope Waveform Overlay Into Video", true, "Overlays real-time glowing stereo audio waveform visualizer onto rendered video output"))
        .with_param(DspParamSchema::knob("video_bitrate_mbps", "Video Stream Compression Target Bitrate", 5.0, 100.0, 25.0, "Mbps", MacroRole::Punch, "Video encoding bitrate balancing visual artifact suppression against export file size"))
    );
    descriptors.insert("AiMixBalanceInspector".to_string(), DspNodeDescriptor::new("AiMixBalanceInspector", "AI Spectral Masking & Autonomous Balance Inspector HUD", DspNodeCategory::DynamicsMaster, "Real-time psychoacoustic spectral masking analyzer comparing audio tracks against target genre mastering curves and generating EQ reduction suggestions")
        .with_param(DspParamSchema::choice("target_spectral_tilt", "Target Genre Psychoacoustic Spectral Curve", &["Balanced Commercial Curve", "Warm Analog Tilt", "Bright Pop Presentation", "Heavy Bass Heavy-End"], 0, "Reference frequency balance curve used to calculate mix balance deviations"))
        .with_param(DspParamSchema::knob("masking_detection_sensitivity_db", "Spectral Masking Collision Detection Threshold", 1.0, 18.0, 6.0, "dB", MacroRole::Tone, "Threshold of psychoacoustic energy overlap triggering masking warnings between tracks"))
        .with_param(DspParamSchema::knob("low_end_phase_correlation_threshold", "Sub-Bass Minimum Monocompatibility Phase Floor", 0.5, 1.0, 0.85, "ratio", MacroRole::Character, "Alert threshold warning of dangerous out-of-phase bass cancellation below 120Hz"))
        .with_param(DspParamSchema::toggle("auto_suggest_eq_cuts", "Generate Intelligent Parametric EQ Cut Offsets", true, "Calculates surgical high-Q frequency cut recommendations to resolve frequency clutter"))
        .with_param(DspParamSchema::toggle("visual_masking_heatmap", "Render 2D Multi-Track Energy Collision Heatmap", true, "Draws interactive real-time spectrogram matrix highlighting clashing track pairs"))
    );
    descriptors.insert("AudioPhaseAligner".to_string(), DspNodeDescriptor::new("AudioPhaseAligner", "Sub-Sample Audio Phase Cross-Correlator & Aligner HUD", DspNodeCategory::DynamicsMaster, "High-precision phase correlation and micro-delay alignment processor utilizing sinc-interpolated cross-correlation to align multitrack drum overheads and bass DIs")
        .with_param(DspParamSchema::knob("max_correlation_search_window_ms", "Cross-Correlation Maximum Time Search Window", 1.0, 50.0, 15.0, "ms", MacroRole::Tone, "Maximum time offset evaluated when scanning for optimal peak phase coherence"))
        .with_param(DspParamSchema::toggle("sub_sample_fir_sinc_interpolation", "Sub-Sample Sinc Interpolation Precision", true, "Performs fractional-sample phase shifting using bandlimited Kaiser-windowed sinc filters"))
        .with_param(DspParamSchema::toggle("phase_inversion_flip", "Invert Signal 180-Degree Absolute Polarity", false, "Flips signal phase 180 degrees to test cancellation versus reinforcement"))
        .with_param(DspParamSchema::knob("allpass_dispersion_rotation_deg", "Frequency-Dependent Allpass Phase Rotation", 0.0, 180.0, 0.0, "deg", MacroRole::Space, "Rotates low-frequency phase progressively to align non-linear mic acoustic offsets"))
        .with_param(DspParamSchema::knob("measured_delay_time_samples", "Detected Acoustic Time-of-Flight Latency", -1000.0, 1000.0, 0.0, "samples", MacroRole::Character, "Exact sample offset measured between reference channel and target input signal"))
    );
    descriptors.insert("AudioProjectNormalizer".to_string(), DspNodeDescriptor::new("AudioProjectNormalizer", "True-Peak Headroom & Project Gain Normalization HUD", DspNodeCategory::DynamicsMaster, "Comprehensive multi-track audio project gain normalizer enforcing EBU R128 integrated loudness targets and true-peak intersample headroom")
        .with_param(DspParamSchema::knob("target_loudness_lufs", "Target Integrated Loudness Level", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Target broadcast or streaming platform loudness specification (-14 LUFS Spotify, -23 EBU)"))
        .with_param(DspParamSchema::knob("true_peak_ceiling_dbfs", "Maximum Intersample True-Peak Ceiling", -3.0, 0.0, -1.0, "dBTP", MacroRole::Tone, "Strict intersample ceiling preventing DAC clipping during lossy MP3/AAC re-encoding"))
        .with_param(DspParamSchema::choice("bit_depth_requantize", "Output Word Length & Re-quantization", &["32-bit Float (Preserve)", "24-bit Fixed (Studio Standard)", "16-bit Fixed (Red Book CD)"], 0, "Target PCM bit resolution for normalized audio output files"))
        .with_param(DspParamSchema::choice("dither_algorithm", "Re-quantization Dither & Noise Shaping", &["None (Truncate)", "TPDF Triangular", "Noise Shaped Lipshitz", "Psychoacoustic High-Pass"], 1, "Applies mathematically optimal dither noise to eliminate low-level quantization distortion"))
        .with_param(DspParamSchema::toggle("batch_normalize_all_stems", "Apply Matched Gain Staging Across All Stems", true, "Applies identical normalization offset across all exported stems to preserve master mix balance"))
    );
    descriptors.insert("PerceptualQualityVerifier".to_string(), DspNodeDescriptor::new("PerceptualQualityVerifier", "ITU-R BS.1387 PEAQ Perceptual Audio Quality Verifier HUD", DspNodeCategory::DynamicsMaster, "Objective psychoacoustic audio quality analyzer comparing rendered tracks against golden reference masters utilizing PEAQ Objective Difference Grade metrics")
        .with_param(DspParamSchema::knob("target_peaq_objective_difference_grade", "Minimum Target PEAQ Difference Grade (ODG)", -4.0, 0.0, -0.2, "ODG", MacroRole::Tone, "Strictness threshold (0.0 represents imperceptible difference, -1.0 is perceptible but not annoying)"))
        .with_param(DspParamSchema::knob("perceptual_noise_floor_masking_db", "Psychoacoustic Masking Noise Threshold", -120.0, -40.0, -84.0, "dB", MacroRole::Space, "Threshold below which distortion components are masked by ear auditory filter models"))
        .with_param(DspParamSchema::knob("spectral_flux_harmonic_distortion_max", "Total Harmonic Distortion (THD+N) Tolerance", 0.001, 1.0, 0.05, "%", MacroRole::Character, "Upper ceiling on permitted harmonic and intermodulation distortion products"))
        .with_param(DspParamSchema::toggle("generate_peaq_difference_spectrogram", "Render 2D Difference Spectrogram Heatmap", true, "Generates color-coded frequency-versus-time spectrogram highlighting perceptual deviations"))
        .with_param(DspParamSchema::toggle("strict_compliance_pass_fail", "Enforce Strict Automated Pass/Fail Verdict", true, "Emits pass/fail status flag for CI/CD automated audio testing pipelines"))
    );
    descriptors.insert("InputOutputGainTrim".to_string(), DspNodeDescriptor::new("InputOutputGainTrim", "Dual-Stage Pre/Post Track Channel Gain Trim & Headroom Attenuator HUD", DspNodeCategory::DynamicsMaster, "Dual-stage track channel gain stage providing independent input pre-trim and output post-trim for optimal gain staging into non-linear DSP plugins")
        .with_param(DspParamSchema::knob("pre_dsp_input_gain_trim_db", "Pre-DSP Channel Input Gain Trim", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Input attenuation or boost applied before insert effect chain to establish optimal analog headroom"))
        .with_param(DspParamSchema::knob("post_dsp_output_gain_trim_db", "Post-DSP Channel Output Gain Trim", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Output fader level adjustment compensating for cumulative plugin makeup gain"))
        .with_param(DspParamSchema::choice("gain_stage_linking_mode", "Dual-Stage Gain Linking Coupling", &["Independent Trim", "Inverse Gain Match (Pre + Post = 0dB)", "Input Mute Safe"], 0, "Couples input and output trims so driving hotter input automatically attenuates output by equivalent dB"))
        .with_param(DspParamSchema::choice("analog_headroom_soft_clip", "Analog Headroom Soft Clipping Profile", &["Off (Linear Clean)", "Gentle Analog Warmth (-0.5dB)", "Tape Soft Saturation (-1.0dB)"], 0, "Applies subtle hyperbolic tangent soft saturation to tame unexpected extreme signal excursions"))
        .with_param(DspParamSchema::toggle("enable_gain_staging_bypass", "Bypass Gain Staging Trim Circuit", false, "Passes incoming audio through unaltered for transparent A/B gain comparison"))
    );
    descriptors.insert("TransientShaperMatrix".to_string(), DspNodeDescriptor::new("TransientShaperMatrix", "Multi-Band Attack & Sustain Transient Sculpting Matrix HUD", DspNodeCategory::DynamicsMaster, "Multi-band transient shaper decomposing audio into discrete frequency bands to independently reshape attack transients, sustain tails, and micro-dynamics with zero phase distortion")
        .with_param(DspParamSchema::knob("attack_gain_db", "Initial Attack Transient Punch Gain", -24.0, 24.0, 3.5, "dB", MacroRole::Punch, "Boost or cut applied strictly to the initial onset transient spike"))
        .with_param(DspParamSchema::knob("sustain_gain_db", "Late Ringing Sustain Tail Level Gain", -24.0, 24.0, -2.0, "dB", MacroRole::Character, "Gain attenuation or reinforcement applied to the resonant ring tail"))
        .with_param(DspParamSchema::log_knob("low_band_crossover_hz", "Low-to-Mid Transient Frequency Crossover Split", 40.0, 500.0, 160.0, "Hz", MacroRole::Tone, "Crossover corner dividing sub/punch frequencies from mid transient body"))
        .with_param(DspParamSchema::log_knob("high_band_crossover_hz", "Mid-to-High Transient Frequency Crossover Split", 1000.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Crossover corner isolating high-frequency snap and sibilant transients"))
        .with_param(DspParamSchema::knob("transient_detection_sensitivity", "Wavelet-Based Transient Onset Detection Threshold", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Sensitivity threshold of wavelet energy detector triggering attack shaping"))
    );
    descriptors.insert("DitherNoiseShaper".to_string(), DspNodeDescriptor::new("DitherNoiseShaper", "Psychoacoustic Lipshitz Noise-Shaping & TPDF Dither HUD", DspNodeCategory::DynamicsMaster, "Mastering grade Triangular Probability Density Function (TPDF) dither and psychoacoustic noise shaper relocating quantization noise into inaudible high-frequency masking regions")
        .with_param(DspParamSchema::choice("target_bit_depth", "Destination Word Length Bit Truncation Target", &["16-bit Red Book CD", "24-bit Studio Master", "12-bit Vintage Sampler", "8-bit Lo-Fi Arcade"], 0, "Word bit depth to which audio samples are quantized upon mastering output"))
        .with_param(DspParamSchema::choice("noise_shaping_curve", "Noise Shaping Spectral Contour & Psychoacoustic Weighting", &["Flat Triangular TPDF (Zero Coloration)", "Lipshitz Psychoacoustic Optimal", "Modified E-Weighted High-Pass", "Ultra High-Order F-Weighting"], 1, "Spectral contour shifting quantization distortion away from critical 1kHz-4kHz hearing sensitivity"))
        .with_param(DspParamSchema::toggle("auto_blank_digital_silence", "Auto-Mute Dither Noise Floor During Zero Audio Signal", true, "Silences background dither hiss during passages of pure digital silence"))
        .with_param(DspParamSchema::knob("dither_amplitude_lsb", "Dither Injection Level in Least Significant Bits", 0.5, 2.0, 1.0, "LSB", MacroRole::Tone, "Amplitude of triangular pseudo-random dither noise in LSB units"))
        .with_param(DspParamSchema::toggle("anti_aliasing_dither_filter", "Engage Ultrasonic Reconstruction Anti-Aliasing Filter", true, "Applies steep low-pass filtering to tame ultrasonic noise-shaped energy peaks"))
    );
    descriptors.insert("SidechainSpectralDucker".to_string(), DspNodeDescriptor::new("SidechainSpectralDucker", "Spectral Multi-Band Dynamic Sidechain Frequency Ducker HUD", DspNodeCategory::DynamicsMaster, "Spectral sidechain ducker attenuating only the exact overlapping frequencies of a masking trigger signal (e.g. carving space in a bass synthesizer strictly during kick drum fundamentals)")
        .with_param(DspParamSchema::knob("ducking_depth_db", "Maximum Attenuation Cut Applied at Masking Frequencies", 0.0, 30.0, 9.0, "dB", MacroRole::Punch, "Maximum dB attenuation carved out of masked frequency bins during trigger transients"))
        .with_param(DspParamSchema::choice("sidechain_channel_select", "Trigger Signal Routing Input Channel Source", &["Sidechain Bus 1 (Kick Key)", "Sidechain Bus 2 (Snare Key)", "Sidechain Bus 3 (Lead Vocal)", "Aux External Key Input"], 0, "Audio bus delivering the key trigger signal driving spectral ducking analysis"))
        .with_param(DspParamSchema::log_knob("low_frequency_anchor_hz", "Sub-Bass Spectral Carving Corner Frequency", 30.0, 300.0, 80.0, "Hz", MacroRole::Tone, "Lower frequency boundary focusing ducking energy on kick/bass low-end clarity"))
        .with_param(DspParamSchema::knob("recovery_release_speed_ms", "Spectral Ducker Envelope Recovery Duration", 5.0, 500.0, 45.0, "ms", MacroRole::Space, "Speed at which carved frequency bins recover to full unattenuated amplitude"))
        .with_param(DspParamSchema::toggle("listen_spectral_delta", "Audition Isolated Subtracted Audio Delta Difference", false, "Solos the exact difference audio removed from the signal for precision auditing"))
    );
    descriptors.insert("LinearPhaseDeEsser".to_string(), DspNodeDescriptor::new("LinearPhaseDeEsser", "Split-Band Linear-Phase High-Frequency Sibilance Suppressor HUD", DspNodeCategory::DynamicsMaster, "Split-band linear-phase de-esser targeting harsh vocal sibilants ('s', 'sh', 't') between 4kHz and 10kHz without introducing phase distortion, comb filtering, or tonal dullness")
        .with_param(DspParamSchema::log_knob("sibilance_detection_freq_hz", "Sibilance Detector Center Tuning Frequency", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Center detection frequency focused on vocal harshness and consonant sibilance"))
        .with_param(DspParamSchema::knob("sibilance_reduction_threshold_db", "De-Esser Gain Reduction Activation Threshold", -40.0, 0.0, -16.0, "dB", MacroRole::Punch, "Detector signal level above which high-frequency compression activates"))
        .with_param(DspParamSchema::knob("gain_reduction_limit_db", "Maximum Allowed High-Frequency Sibilance Clamp Limit", -24.0, 0.0, -8.0, "dB", MacroRole::Character, "Safety ceiling capping maximum allowable high-frequency attenuation to avoid lisping"))
        .with_param(DspParamSchema::knob("deesser_filter_bandwidth_q", "Detector Bandpass Sibilance Sensitivity Bandwidth", 0.5, 5.0, 1.8, "Q", MacroRole::Tone, "Selectivity Q bandwidth of sidechain bandpass filter isolating harsh consonants"))
        .with_param(DspParamSchema::toggle("solo_sibilant_band", "Solo Extracted Sibilant Band for Accurate Center Frequency Tuning", false, "Solos sidechain detector audio so user can sweep and pinpoint exact sibilance frequencies"))
    );
    descriptors.insert("StereoGoniometerVisualizer".to_string(), DspNodeDescriptor::new("StereoGoniometerVisualizer", "Lissajous Vector Scope & Stereo Phase Alignment HUD", DspNodeCategory::DynamicsMaster, "Real-time stereo vector scope rendering Left/Right signal trajectories as Lissajous figures, displaying dynamic phase correlation coefficients, mono compatibility, and stereo balance")
        .with_param(DspParamSchema::knob("display_gain_scale", "Lissajous Vector Beam Amplitude Scaling", 0.5, 4.0, 1.2, "x", MacroRole::Tone, "Visual beam amplification expanding subtle low-level stereo trajectory patterns"))
        .with_param(DspParamSchema::knob("phosphor_persistence_ms", "Virtual Oscilloscope CRT Phosphor Decay Time", 20.0, 1000.0, 180.0, "ms", MacroRole::Space, "Visual glow persistence of oscilloscope trace simulating CRT display tubes"))
        .with_param(DspParamSchema::choice("vector_scope_rotation_mode", "Vector Scope Axis Orientation & Geometry", &["45° Diamond Scope (Left/Right)", "90° Cartesian Scope (X/Y)", "Polar Spatial Imager"], 0, "Coordinate frame geometry aligning Left/Right signals along diagonal or vertical axes"))
        .with_param(DspParamSchema::knob("phase_correlation_alert_threshold", "Negative Phase Correlation Warning Alert Boundary", -1.0, 0.0, -0.2, "coeff", MacroRole::Character, "Threshold below which negative Pearson correlation activates visual phase warning"))
        .with_param(DspParamSchema::toggle("mono_fold_sum_check", "Sum Audio Channels to Mono to Inspect Comb-Filtering Cancellation", false, "Rapidly folds stereo signal to mono to verify absence of destructive phase cancellation"))
    );
    descriptors.insert("MasterSafetyClipper".to_string(), DspNodeDescriptor::new("MasterSafetyClipper", "True-Peak Master Safety Brickwall Clipper & DC Offset Barrier HUD", DspNodeCategory::DynamicsMaster, "Final emergency safety barrier on master bus output preventing catastrophic acoustic overloads, speaker damage, hearing fatigue, and DC offset bias with zero audio latency")
        .with_param(DspParamSchema::knob("brickwall_ceiling_dbtp", "Maximum Hard Safety Ceiling True-Peak Level", -6.0, 0.0, -0.1, "dBTP", MacroRole::Punch, "Absolute hard brickwall ceiling threshold above which samples are clipped cleanly"))
        .with_param(DspParamSchema::knob("clipper_knee_softness", "Hyperbolic Soft-Clipping Knee Transition Width", 0.0, 6.0, 1.2, "dB", MacroRole::Character, "Soft knee rounding width applying gentle tanh saturation before hard ceiling"))
        .with_param(DspParamSchema::log_knob("dc_offset_highpass_cutoff_hz", "Sub-Audible DC Blocking High-Pass Filter Corner", 5.0, 40.0, 15.0, "Hz", MacroRole::Tone, "High-pass filter cutoff removing destructive DC voltage offsets from audio path"))
        .with_param(DspParamSchema::toggle("auto_mute_on_infinite_nan", "Auto-Mute Master Bus If Inf or NaN Sample Values Detected", true, "Instantaneous zero-sample mute guard isolating DSP blowups or float NaN/Inf errors"))
        .with_param(DspParamSchema::knob("emergency_attenuation_guard_db", "Emergency Headroom Pad Attenuation Level", -24.0, 0.0, 0.0, "dB", MacroRole::Tone, "Emergency attenuation pad protecting physical monitors during high-volume sessions"))
    );
    descriptors.insert("MaskingReport".to_string(), DspNodeDescriptor::new("MaskingReport", "AI Spectral Masking & Collision Diagnostic HUD", DspNodeCategory::DynamicsMaster, "Intelligent multi-track frequency collision diagnostic report identifying perceptual masking overlap scores, clash frequencies, critical bandwidths, and dynamic EQ unmasking recommendations")
        .with_param(DspParamSchema::knob("overall_masking_score", "Overall Masking Overlap Dissonance Score", 0.0, 1.0, 0.25, "score", MacroRole::Punch, "Normalized scalar indicating severity of auditory masking between compared tracks"))
        .with_param(DspParamSchema::log_knob("conflict_center_freq_hz", "Dominant Frequency Masking Conflict Center", 20.0, 16000.0, 250.0, "Hz", MacroRole::Tone, "Center frequency where spectral energy overlap causes greatest loss of intelligibility"))
        .with_param(DspParamSchema::knob("conflict_bandwidth_q", "Psychoacoustic Masking Critical Bandwidth", 0.5, 8.0, 1.4, "Q", MacroRole::Character, "Selectivity Q factor of the psychoacoustic critical band affected by mutual masking"))
        .with_param(DspParamSchema::knob("recommended_eq_dip_db", "Recommended Dynamic EQ Carve Attenuation", -18.0, 0.0, -3.5, "dB", MacroRole::Punch, "Suggested cut depth applied to conflicting track to carve sonic space"))
        .with_param(DspParamSchema::knob("temporal_overlap_pct", "Temporal Collision Duty Cycle Overlap", 0.0, 100.0, 35.0, "%", MacroRole::Space, "Percentage of time both tracks produce simultaneous energy above threshold"))
    );
    descriptors.insert("AlignmentResult".to_string(), DspNodeDescriptor::new("AlignmentResult", "Multi-Track Phase Alignment & Polarity Match HUD", DspNodeCategory::DynamicsMaster, "Automatic audio track alignment analyzer calculating cross-correlation peak scores, sub-millisecond phase delay offsets, sample lag corrections, and polarity inversion states")
        .with_param(DspParamSchema::knob("phase_delay_samples", "Phase Delay Offset in Samples", -512.0, 512.0, 0.0, "samples", MacroRole::Space, "Discrete integer sample shift correcting time-of-flight acoustic delay between microphones"))
        .with_param(DspParamSchema::knob("phase_offset_ms", "Phase Shift Sub-Millisecond Time Delay", -15.0, 15.0, 0.0, "ms", MacroRole::Tone, "Continuous fractional millisecond time delay offset between track channels"))
        .with_param(DspParamSchema::knob("cross_correlation_score", "Normalized Cross-Correlation Peak Alignment", 0.0, 1.0, 0.98, "corr", MacroRole::Punch, "Maximum Pearson cross-correlation value indicating quality of acoustic match"))
        .with_param(DspParamSchema::toggle("polarity_flip", "Invert Polarity 180 Degrees for Cancellation Check", false, "Flips waveform polarity to eliminate destructive acoustic phase cancellation"))
        .with_param(DspParamSchema::knob("sample_rate_hz", "Reference Alignment Sample Rate", 44100.0, 192000.0, 48000.0, "Hz", MacroRole::Character, "Sampling frequency governing micro-timing correlation calculations"))
    );
    descriptors.insert("ControlEvent".to_string(), DspNodeDescriptor::new("ControlEvent", "Dynamic Sidechain Ducking & Control Matrix Message", DspNodeCategory::DynamicsMaster, "Sub-frame precision control signal message in sidechain compression matrix carrying target parameter ID, trigger amplitude, and source bus routing channel")
        .with_param(DspParamSchema::knob("frame_offset", "Sample Frame Sub-Block Offset", 0.0, 2048.0, 0.0, "frames", MacroRole::Tone, "Sample frame offset within current audio evaluation buffer block"))
        .with_param(DspParamSchema::knob("value", "Sidechain Control Peak Level", -60.0, 12.0, 0.0, "dB", MacroRole::Punch, "Detector control signal payload representing sidechain peak level"))
        .with_param(DspParamSchema::knob("source_id", "Source Track Key Channel Index", 0.0, 64.0, 1.0, "ch", MacroRole::Tone, "Channel index of source audio track feeding control envelope detector"))
        .with_param(DspParamSchema::knob("attack_time_ms", "Detector Attack Smoothing Response", 0.1, 100.0, 2.5, "ms", MacroRole::Punch, "Sidechain detector attack smoothing filter response time"))
        .with_param(DspParamSchema::knob("hold_samples", "Peak Hold Gate Duration", 0.0, 4800.0, 128.0, "samples", MacroRole::Space, "Peak hold gate duration before releasing ducking reduction"))
    );
    descriptors.insert("SidechainBusId".to_string(), DspNodeDescriptor::new("SidechainBusId", "Sidechain Compressor Multi-Bus Routing Matrix Slot", DspNodeCategory::DynamicsMaster, "Dedicated sidechain compression matrix bus slot managing auxiliary send gain, bandpass detector filtering, and audition solo monitoring")
        .with_param(DspParamSchema::knob("bus_index", "Auxiliary Send Channel Bus Index", 0.0, 32.0, 1.0, "bus", MacroRole::Tone, "Auxiliary sidechain routing bus channel slot identifier"))
        .with_param(DspParamSchema::knob("gain_db", "Sidechain Send Input Gain Trim", -48.0, 18.0, 0.0, "dB", MacroRole::Punch, "Sidechain input send gain trim pre-detector"))
        .with_param(DspParamSchema::log_knob("highpass_cutoff_hz", "Detector High-Pass Key Filter", 20.0, 2000.0, 120.0, "Hz", MacroRole::Tone, "Highpass filter on detector key input preventing low rumble pumping"))
        .with_param(DspParamSchema::log_knob("lowpass_cutoff_hz", "Detector Low-Pass Key Filter", 500.0, 20000.0, 8000.0, "Hz", MacroRole::Tone, "Lowpass filter on detector key isolating snare/kick transient energy"))
        .with_param(DspParamSchema::toggle("listen_sidechain", "Solo & Audition Key Detector Signal", false, "Solo and audition sidechain detector filtered key signal"))
    );
    descriptors.insert("RenderResult".to_string(), DspNodeDescriptor::new("RenderResult", "DSP Render Block Performance & Signal Metrics Analyzer", DspNodeCategory::DynamicsMaster, "Real-time and offline DSP block render telemetry measuring true peak signal level, RMS loudness level, integrated LUFS loudness, buffer underrun count, and block execution render time")
        .with_param(DspParamSchema::knob("peak_db", "Rendered Audio True Peak Level", -96.0, 6.0, -0.3, "dBFS", MacroRole::Tone, "True peak audio amplitude in decibels full scale"))
        .with_param(DspParamSchema::knob("rms_db", "Rendered Audio RMS Energy Level", -96.0, 0.0, -14.0, "dBFS", MacroRole::Punch, "Root-mean-square average signal power level"))
        .with_param(DspParamSchema::knob("lufs_integrated", "EBU R128 Integrated Loudness", -48.0, -6.0, -14.0, "LUFS", MacroRole::Tone, "EBU R128 standard integrated program loudness"))
        .with_param(DspParamSchema::knob("underruns", "Real-Time Audio Buffer Underrun Count", 0.0, 100.0, 0.0, "count", MacroRole::Character, "Detected audio output buffer dropout underrun incidents"))
        .with_param(DspParamSchema::knob("render_time_ms", "Block Computation Execution Latency", 0.01, 20.0, 1.25, "ms", MacroRole::Space, "DSP evaluation processing time per audio buffer block"))
    );
    descriptors.insert("DynamicsBandIndex".to_string(), DspNodeDescriptor::new("DynamicsBandIndex", "Multi-Band Dynamics Crossover Band Frequency Indexing", DspNodeCategory::DynamicsMaster, "Crossover frequency distributor and band router managing 4-band multiband compressor/expander splitting with Linkwitz-Riley phase-coherent crossover filters")
        .with_param(DspParamSchema::choice("crossover_slope_order", "Linkwitz-Riley Crossover Filter Slope", &["12 dB/Oct (2nd-Order Low Phase Delay)", "24 dB/Oct (4th-Order Phase Coherent)", "48 dB/Oct (8th-Order Surgical Separation)"], 1, "Crossover filter steepness and phase characteristics"))
        .with_param(DspParamSchema::knob("low_mid_crossover_hz", "Low to Mid Crossover Frequency", 50.0, 500.0, 140.0, "Hz", MacroRole::Tone, "Split point between sub-bass and low-mid bands"))
        .with_param(DspParamSchema::knob("mid_high_crossover_hz", "Mid to High Crossover Frequency", 1000.0, 10000.0, 3200.0, "Hz", MacroRole::Punch, "Split point between midrange body and high-frequency air"))
        .with_param(DspParamSchema::knob("active_band_solo_mute_mask", "Selected Band Solo / Mute Routing", 0.0, 4.0, 0.0, "mask", MacroRole::Character, "Solo selected frequency band for precision monitoring"))
        .with_param(DspParamSchema::knob("sidechain_band_coupling", "Cross-Band Dynamic Sidechain Leakage", 0.0, 1.0, 0.20, "leak", MacroRole::Space, "Coupling factor letting loud bass hits gently compress mids"))
        .with_param(DspParamSchema::toggle("enable_linear_phase_crossover", "Linear-Phase FIR Spectral Splitting", false, "Use linear-phase FIR filtering avoiding all phase distortion at crossovers"))
    );
    descriptors.insert("DitherType".to_string(), DspNodeDescriptor::new("DitherType", "Mastering Triangular PDF & Psychoacoustic Noise-Shaped Dither", DspNodeCategory::DynamicsMaster, "High-fidelity mastering quantizer decorrelating truncation distortion into imperceptible analog-like noise floor with standard TPDF, high-order psychoacoustic equal-loudness noise shaping, and silent auto-blanking")
        .with_param(DspParamSchema::choice("dither_noise_algorithm", "Quantization Noise Shaping Profile", &["None (Direct Truncation)", "TPDF (Triangular Probability Density Function)", "Noise-Shaped Psychoacoustic (E-Weighted Curve)"], 1, "Dither probability distribution and spectral weighting"))
        .with_param(DspParamSchema::choice("target_bit_depth", "Target Quantization Bit Resolution", &["16-Bit (Red Book Audio CD)", "24-Bit (High-Resolution Studio Master)", "32-Bit Float (Native Bus Stream)"], 0, "Target word length for digital audio quantization"))
        .with_param(DspParamSchema::knob("noise_shaping_curve_order", "Psychoacoustic Noise Filter Order", 1.0, 5.0, 3.0, "order", MacroRole::Tone, "Filter slope shifting noise energy into psychoacoustic masked bands"))
        .with_param(DspParamSchema::knob("high_frequency_contour_shelf_db", "High-Frequency Ultrasonic Noise Shelf", -12.0, 12.0, 4.5, "dB", MacroRole::Character, "High frequency emphasis pushing dither noise above 15kHz"))
        .with_param(DspParamSchema::knob("dither_level_lsb_amplitude", "Dither Peak Injection Amplitude", 0.5, 2.0, 1.0, "LSB", MacroRole::Punch, "Amplitude of injected triangular dither noise in LSB units"))
        .with_param(DspParamSchema::toggle("enable_auto_blanking_on_silence", "Auto-Mute Dither Noise During Absolute Silence", true, "Cease dither noise injection when input channel drops below -120dB"))
    );
    descriptors.insert("KSystemScale".to_string(), DspNodeDescriptor::new("KSystemScale", "Bob Katz K-System Broadcast Loudness & Dynamic Range Metering", DspNodeCategory::DynamicsMaster, "Industry standard Bob Katz K-System metering processor providing K-12, K-14, and K-20 calibrated loudness scales with true peak inter-sample detection, acoustic reference SPL calibration, and psychoacoustic dynamic headroom indicators")
        .with_param(DspParamSchema::choice("k_meter_standard", "Bob Katz K-Meter Headroom Calibration", &["K-12 (Broadcast / Pop Radio High Density)", "K-14 (Standard Rock / Pop / Streaming Modern)", "K-20 (Classical / Audiophile / Wide Cinema Dynamic)"], 1, "Selected K-System headroom calibration mode"))
        .with_param(DspParamSchema::knob("target_headroom_db", "Target Dynamic Range Headroom", 6.0, 24.0, 14.0, "dB", MacroRole::Punch, "Target headroom between nominal zero and full scale"))
        .with_param(DspParamSchema::knob("meter_ballistics_integration_ms", "RMS Integration Window Ballistics", 50.0, 1000.0, 300.0, "ms", MacroRole::Tone, "Averaging time constant for RMS energy integration"))
        .with_param(DspParamSchema::knob("red_zone_warning_threshold_db", "Over-Compression Warning Threshold", -3.0, 6.0, 0.0, "dB", MacroRole::Character, "Zone threshold above 0 dB mark indicating crest loss"))
        .with_param(DspParamSchema::knob("spl_acoustic_reference_db", "Calibrated Monitor Listening Level", 60.0, 95.0, 83.0, "dB SPL", MacroRole::Space, "Target listening room acoustic sound pressure level at 0 VU"))
        .with_param(DspParamSchema::toggle("enable_true_peak_intersample_detection", "4x Oversampled True Peak Inter-Sample Metering", true, "Detect inter-sample clip overshoots via 4x oversampling"))
    );
    descriptors.insert("BitDepth".to_string(), DspNodeDescriptor::new("BitDepth", "Master Audio Export Word Length & Quantization Bit Depth", DspNodeCategory::DynamicsMaster, "Master audio export quantization word length selector providing 16-bit, 24-bit, 32-bit float, and 64-bit float rendering with psychoacoustic noise shaping")
        .with_param(DspParamSchema::choice("quantization_word_length", "Audio File Encoding Word Length Bits", &["16-Bit Fixed (Red Book CD Audio Standard)", "24-Bit Fixed (Studio Master High-Resolution Standard)", "32-Bit Floating Point (Zero-Clipping Internal Export)", "64-Bit Double Float (Extreme Precision Scientific Archival)"], 1, "Quantization word length for offline file bounces"))
        .with_param(DspParamSchema::knob("noise_shaping_curve_order", "Psychoacoustic Noise-Shaping Filter Order", 0.0, 3.0, 1.0, "order", MacroRole::Tone, "Shifts quantization noise into psychoacoustically inaudible bands"))
        .with_param(DspParamSchema::knob("dither_amplitude_lsb", "Triangular Dither PDF Amplitude in LSBs", 0.5, 2.0, 1.0, "LSB", MacroRole::Punch, "Injected random dither noise level relative to least significant bit"))
        .with_param(DspParamSchema::knob("inter_sample_peak_headroom_db", "True Peak Inter-Sample Headroom Guard", -3.0, 0.0, -0.3, "dB", MacroRole::Character, "Brickwall peak safety margin preventing DAC reconstruction clip"))
        .with_param(DspParamSchema::knob("dc_offset_rejection_hz", "Highpass DC Offset Rejection Cutoff", 1.0, 20.0, 5.0, "Hz", MacroRole::Space, "Sub-audible DC highpass filter eliminating asymmetry"))
        .with_param(DspParamSchema::toggle("enable_tpdf_triangular_dither", "Enable High-Pass Shaped TPDF Dithering", true, "Linearize quantization distortion with high-pass filtered noise"))
    );
    descriptors.insert("StemExportFormat".to_string(), DspNodeDescriptor::new("StemExportFormat", "Multi-Track Stem Batch Audio Encoding & Export Container", DspNodeCategory::DynamicsMaster, "High-performance stem exporter batch-rendering individual track channels into WAV, FLAC, AIFF, or MP3 with LUFS loudness normalization")
        .with_param(DspParamSchema::choice("audio_container_format", "Audio File Encoding Format Container", &["WAV (Uncompressed Broadcast Linear PCM Container)", "FLAC (Lossless Compressed Audio Archival Format)", "AIFF (Apple Audio Interchange Uncompressed PCM)", "MP3 (MPEG-1 Layer III Perceptual Lossy Audio)"], 0, "File container encoding format for multi-track stem export"))
        .with_param(DspParamSchema::knob("sample_rate_export_hz", "Stem Export Resampling Target Rate", 44100.0, 192000.0, 48000.0, "Hz", MacroRole::Tone, "Output sample rate with zero-phase bandlimited sinc resampling"))
        .with_param(DspParamSchema::knob("flac_compression_level", "Lossless FLAC Encoding Compression Ratio", 0.0, 8.0, 5.0, "level", MacroRole::Punch, "FLAC linear prediction search effort vs file size trade-off"))
        .with_param(DspParamSchema::knob("mp3_cbr_bitrate_kbps", "MP3 Constant Bitrate Stream Bandwidth", 128.0, 320.0, 320.0, "kbps", MacroRole::Character, "Bitrate bandwidth allocation for lossy MP3 encoding"))
        .with_param(DspParamSchema::knob("true_peak_margin_headroom_db", "Export Brickwall Limiter Margin Ceiling", -3.0, 0.0, -1.0, "dB", MacroRole::Space, "Peak limiter margin preventing inter-sample clipping upon export"))
        .with_param(DspParamSchema::toggle("enable_individual_stem_normalization", "Individually Normalize Stems to -14 LUFS", true, "Match international broadcast loudness standards across all stems"))
    );
    descriptors.insert("BitDepthDitherQuantizer".to_string(), DspNodeDescriptor::new("BitDepthDitherQuantizer", "TPDF Psychoacoustic Noise-Shaped Dither & Wordlength Reducer", DspNodeCategory::DynamicsMaster, "Mastering dither engine applying triangular probability density function (TPDF) noise and psychoacoustic curves for transparent wordlength reduction")
        .with_param(DspParamSchema::choice("target_wordlength_bits", "Target Wordlength Bit Depth", &["24-Bit (High Dynamic Range)", "16-Bit (Red Book Audio)", "12-Bit (Vintage Sampler)", "8-Bit (Retro Lo-Fi)"], 1, "Quantization bit depth for final audio rendering and CD mastering"))
        .with_param(DspParamSchema::choice("noise_shaping_curve", "Psychoacoustic Noise Shaping", &["Flat TPDF (Uncolored)", "F-Weighted Psychoacoustic", "Lipshitz Ultra Noise Shape", "High-Pass High Frequency"], 1, "Shifts dither quantization noise into less audible high-frequency spectrum"))
        .with_param(DspParamSchema::knob("dither_amplitude_lsb", "Dither Noise Amplitude", 0.5, 4.0, 2.0, "LSB", MacroRole::Character, "Total dither noise peak-to-peak amplitude measured in Least Significant Bits"))
        .with_param(DspParamSchema::toggle("auto_mute_dither_on_digital_black", "Auto-Mute on Digital Silence", true, "Silences dither generator when audio signal drops to absolute zero"))
        .with_param(DspParamSchema::toggle("dc_dither_bias_guard", "DC Bias Rejection Guard", true, "High-pass filters dither noise to prevent sub-audible DC offset accumulation"))
        .with_param(DspParamSchema::toggle("vintage_quantization_distortion", "Emulate Vintage ADC Truncation", false, "Disables dither to expose gritty harmonic quantization steps"))
    );
    descriptors.insert("LoudnessEbuR128Normalizer".to_string(), DspNodeDescriptor::new("LoudnessEbuR128Normalizer", "ITU-R BS.1770-4 & EBU R128 Dual-Stage Loudness Normalizer", DspNodeCategory::DynamicsMaster, "Automated broadcast and streaming loudness target normalizer computing integrated LUFS, loudness range (LRA), and true-peak gain matching")
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Integrated Loudness", -24.0, -8.0, -14.0, "LUFS", MacroRole::Punch, "Target integrated loudness compliant with streaming platform specifications"))
        .with_param(DspParamSchema::knob("maximum_true_peak_dbfs", "Maximum Intersample True Peak", -3.0, 0.0, -1.0, "dBFS", MacroRole::Punch, "Ceiling limit for 4x oversampled true-peak reconstructive transients"))
        .with_param(DspParamSchema::choice("streaming_target_preset", "Broadcasting & Streaming Profile", &["Streaming Standard (-14 LUFS)", "Club / Electronic (-9 LUFS)", "Broadcast EBU R128 (-23 LUFS)", "Podcast / Voice (-16 LUFS)", "AES Recommended (-16 LUFS)"], 0, "Standard target profiles for major digital streaming services"))
        .with_param(DspParamSchema::toggle("enable_loudness_range_expansion", "Allow Dynamic Loudness Expansion", false, "Allows upward gain expansion for overly compressed material"))
        .with_param(DspParamSchema::knob("gating_threshold_absolute_lufs", "Absolute Gating Threshold", -70.0, -50.0, -70.0, "LUFS", MacroRole::None, "Silence gate threshold preventing pauses from skewing integrated LUFS"))
        .with_param(DspParamSchema::toggle("apply_brickwall_limiter_catch", "Apply Safety True-Peak Catch Limiter", true, "Applies ultra-fast brickwall limiter to catch stray momentary peaks"))
    );
    descriptors.insert("TruePeakIntersampleLimiter".to_string(), DspNodeDescriptor::new("TruePeakIntersampleLimiter", "4x Oversampled True-Peak Brickwall Intersample Limiter", DspNodeCategory::DynamicsMaster, "Lookahead intersample peak limiter with 4x polyphase oversampling, eliminating intersample clipping on DAC reconstruction filters")
        .with_param(DspParamSchema::knob("ceiling_threshold_dbfs", "True-Peak Ceiling Limit", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Absolute brickwall ceiling preventing reconstruction filter overshoots"))
        .with_param(DspParamSchema::knob("lookahead_time_ms", "Lookahead Buffer Time", 0.5, 10.0, 2.0, "ms", MacroRole::None, "Anticipatory lookahead buffer duration allowing zero transient overshoot"))
        .with_param(DspParamSchema::knob("release_time_constant_ms", "Limiter Release Envelope Time", 5.0, 500.0, 50.0, "ms", MacroRole::Punch, "Envelope recovery time determining punch versus continuous loudness"))
        .with_param(DspParamSchema::choice("oversampling_rate", "Polyphase Oversampling Multiplier", &["4x Linear Phase", "8x Ultra Sinc", "2x Low Latency"], 0, "Internal polyphase oversampling rate for accurate intersample peak detection"))
        .with_param(DspParamSchema::choice("transient_character_mode", "Transient Saturation Character", &["Transparent Neutral", "Aggressive Punch", "Soft Tube Warmth", "Analog Tape Saturation"], 0, "Harmonic profile applied when limiting heavy drum peaks"))
        .with_param(DspParamSchema::toggle("auto_release_envelope", "Adaptive Auto-Release Envelope", true, "Dynamically adjusts release speed based on program crest factor"))
    );
    descriptors.insert("DynamicMultibandGate".to_string(), DspNodeDescriptor::new("DynamicMultibandGate", "4-Band Spectral Transient Noise Gate & Drum Bleed Reducer", DspNodeCategory::DynamicsMaster, "Precision frequency-split noise gate with independent lookahead, hysteresis thresholds, and envelope followers on low, low-mid, high-mid, and air bands")
        .with_param(DspParamSchema::knob("gate_crossover_low_hz", "Low / Low-Mid Crossover Cutoff", 60.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Frequency dividing sub-bass fundamental from snare body resonances"))
        .with_param(DspParamSchema::knob("gate_crossover_mid_hz", "Low-Mid / High-Mid Crossover", 500.0, 3000.0, 1500.0, "Hz", MacroRole::Tone, "Frequency separating instrument warmth from cymbal bleed harshness"))
        .with_param(DspParamSchema::knob("gate_crossover_high_hz", "High-Mid / Air Crossover", 3000.0, 12000.0, 6000.0, "Hz", MacroRole::Tone, "Frequency isolating upper treble air and hi-hat stick clicks"))
        .with_param(DspParamSchema::knob("low_band_threshold_db", "Low Band Kick Gate Threshold", -60.0, 0.0, -28.0, "dB", MacroRole::Punch, "Threshold level opening low-band gate for kick drum and floor tom thuds"))
        .with_param(DspParamSchema::knob("high_band_bleed_attenuation_db", "Treble Band Max Bleed Attenuation", -60.0, 0.0, -18.0, "dB", MacroRole::Character, "Depth of cymbal bleed suppression during quiet passages when gate is closed"))
        .with_param(DspParamSchema::toggle("enable_transient_lookahead_buffer", "2 ms Anticipatory Lookahead Buffer", true, "Eliminates clipped transient attack onsets by delaying audio relative to detector"))
    );
    descriptors.insert("NonlinearHammerStrike".to_string(), DspNodeDescriptor::new("NonlinearHammerStrike", "Nonlinear Felt/Wood Hammer Strike & Hysteresis Impact Engine", DspNodeCategory::DynamicsMaster, "Acoustic physical impact solver modeling piano felt, harpsichord plectrum, and clavinet tangent strikes using non-linear stiffness exponent p in [1.5, 3.5] and dynamic contact duration")
        .with_param(DspParamSchema::knob("hammer_felt_elasticity_exponent", "Nonlinear Felt Compression Exponent", 1.5, 3.8, 2.6, "exp", MacroRole::Character, "Nonlinear compression stiffness exponent F proportional to y^p (soft felt ~1.8, hard wood ~3.2)"))
        .with_param(DspParamSchema::knob("hammer_mass_grams", "Striking Hammer Effective Mass", 1.0, 30.0, 8.5, "g", MacroRole::Punch, "Effective striking hammer head mass governing string contact duration"))
        .with_param(DspParamSchema::knob("felt_hysteresis_damping", "Viscoelastic Hysteresis Damping Loss", 0.01, 0.8, 0.25, "hyst", MacroRole::Tone, "Internal viscoelastic hysteresis loss during hammer compression/decompression"))
        .with_param(DspParamSchema::knob("strike_velocity_m_s", "Keybed Dynamic Strike Velocity", 0.1, 8.0, 2.2, "m/s", MacroRole::Punch, "Impact approach velocity driven by keybed depression velocity"))
        .with_param(DspParamSchema::knob("string_strike_position_ratio", "String Fractional Strike Location", 0.05, 0.5, 0.125, "pos", MacroRole::Tone, "Fractional strike distance along string (e.g. 1/8th or 1/7th node suppression)"))
        .with_param(DspParamSchema::knob("tangent_rebound_clearance_ms", "Escapement Rebound Clearance Time", 0.5, 20.0, 3.5, "ms", MacroRole::Space, "Time elapsed before hammer/plectrum escapes out of string vibration envelope"))
    );
    descriptors.insert("AdaptiveNeuralDeEsser".to_string(), DspNodeDescriptor::new("AdaptiveNeuralDeEsser", "Predictive Psychoacoustic Neural Sibilance Suppressor", DspNodeCategory::DynamicsMaster, "Intelligent dynamic sibilance processor combining neural spectral envelope tracking with zero-latency lookahead FIR filters to eliminate harsh 's', 'sh', and 't' transients without dulling vocal brilliance")
        .with_param(DspParamSchema::knob("sibilance_detection_threshold_db", "Sibilance Detection Threshold", -48.0, 0.0, -18.0, "dB", MacroRole::Punch, "Threshold above which high-frequency bursts trigger dynamic attenuation"))
        .with_param(DspParamSchema::knob("adaptive_crossover_frequency_hz", "Dynamic Crossover Split Point", 3000.0, 12000.0, 6800.0, "Hz", MacroRole::Tone, "Corner split frequency separating body of voice from sibilant consonants"))
        .with_param(DspParamSchema::knob("maximum_suppression_limit_db", "Maximum Sibilance Gain Reduction", -36.0, 0.0, -10.0, "dB", MacroRole::Tone, "Maximum allowable downward dynamic gain reduction applied to harsh peaks"))
        .with_param(DspParamSchema::knob("lookahead_prediction_ms", "Neural Predictive Lookahead Window", 0.0, 10.0, 2.5, "ms", MacroRole::Space, "FIR lookahead buffer allowing zero-clipping clampdown on abrupt consonants"))
        .with_param(DspParamSchema::knob("formant_envelope_retention", "Formant Envelope Naturalness Retention", 0.0, 1.0, 0.8, "ratio", MacroRole::Character, "Preserves high-frequency vocal formant structure during heavy de-essing"))
        .with_param(DspParamSchema::toggle("audition_delta_sibilance_solo", "Audition Removed Sibilance Solo", false, "Isolates and monitors exclusively the suppressed sibilant energy"))
    );
    descriptors.insert("MultibandUpwardCompressor".to_string(), DspNodeDescriptor::new("MultibandUpwardCompressor", "Three-Band Upward/Downward Dynamic OTT Detail Compressor", DspNodeCategory::DynamicsMaster, "Hyper-aggressive 3-band upward/downward dynamic processor bringing quiet micro-details forward while clamping unruly transients with independent band thresholds and time constants")
        .with_param(DspParamSchema::knob("upward_compression_depth_db", "Upward Compression Boost Depth", 0.0, 36.0, 12.0, "dB", MacroRole::Punch, "Maximum gain boost applied to quiet audio signals below lower threshold"))
        .with_param(DspParamSchema::knob("downward_limiting_ratio", "Downward Limiting Compression Ratio", 1.0, 20.0, 4.0, ":1", MacroRole::Tone, "Compression ratio clamping signals that exceed upper threshold"))
        .with_param(DspParamSchema::knob("low_mid_crossover_hz", "Bass to Mid Crossover Frequency", 50.0, 800.0, 180.0, "Hz", MacroRole::Tone, "Crossover split point between low and middle dynamic bands"))
        .with_param(DspParamSchema::knob("mid_high_crossover_hz", "Mid to Treble Crossover Frequency", 800.0, 10000.0, 2800.0, "Hz", MacroRole::Tone, "Crossover split point between middle and high dynamic bands"))
        .with_param(DspParamSchema::choice("band_time_constant_preset", "Dynamic Time Constants Profile", &["Fast Aggressive (Modern Electronic)", "Balanced Studio Mastering", "Slow Smooth Acoustic", "Punchy Drum Bus"], 0, "Attack and release ballistic curves optimized for different musical genres"))
        .with_param(DspParamSchema::knob("global_dynamics_mix_ratio", "Parallel Dry/Wet Compression Mix", 0.0, 1.0, 0.65, "mix", MacroRole::Punch, "Dry/wet parallel processing blend maintaining natural attack punch"))
    );
    descriptors.insert("SubharmonicTransientExciter".to_string(), DspNodeDescriptor::new("SubharmonicTransientExciter", "Phase-Aligned Transient-Triggered Sub-Bass Synthesizer", DspNodeCategory::DynamicsMaster, "Precision sub-bass generator analyzing incoming kick drum and bass transients to inject synthetic phase-aligned sinusoidal low-frequency energy (20-80 Hz) without muddying mid-range punch")
        .with_param(DspParamSchema::knob("sub_harmonic_tuning_hz", "Synthesized Sub-Bass Frequency", 20.0, 90.0, 48.0, "Hz", MacroRole::Punch, "Tuned frequency of synthesized low-end sine pulse"))
        .with_param(DspParamSchema::knob("transient_detection_threshold_db", "Kick Transient Detector Threshold", -36.0, 0.0, -12.0, "dB", MacroRole::Punch, "Dynamic threshold triggering sub-bass pulse generator"))
        .with_param(DspParamSchema::knob("sub_pulse_decay_ms", "Sub-Bass Pulse Decay Envelope", 20.0, 800.0, 180.0, "ms", MacroRole::Space, "Decay duration of synthesized sub-bass impulse"))
        .with_param(DspParamSchema::knob("fundamental_phase_sync_deg", "Phase Lock Angle to Transient", 0.0, 360.0, 0.0, "deg", MacroRole::Punch, "Starting phase angle synchronizing sub pulse with incoming acoustic transient"))
        .with_param(DspParamSchema::knob("harmonic_drive_saturation", "Sub Overdrive Saturation Drive", 0.0, 1.0, 0.18, "drive", MacroRole::Tone, "Adds subtle second harmonic overtones to make sub audible on small speakers"))
        .with_param(DspParamSchema::knob("direct_sub_clean_mix", "Direct Sub-Bass Output Mix", 0.0, 1.5, 0.75, "gain", MacroRole::Punch, "Level of synthesized sub-bass blended into master signal"))
    );
    descriptors.insert("TruePeakOversampledLimiter".to_string(), DspNodeDescriptor::new("TruePeakOversampledLimiter", "8x Polyphase FIR True-Peak Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster, "Inter-sample peak (ISP) mastering brickwall limiter featuring 8x polyphase linear-phase FIR oversampling, lookahead transient detection, and psychoacoustic auto-release")
        .with_param(DspParamSchema::knob("ceiling_true_peak_db", "True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Maximum permissible peak level guaranteeing zero inter-sample clipping"))
        .with_param(DspParamSchema::knob("threshold_input_gain_db", "Limiter Input Drive Gain", 0.0, 24.0, 4.5, "dB", MacroRole::Punch, "Input gain pushing program content into brickwall limiting threshold"))
        .with_param(DspParamSchema::knob("lookahead_time_ms", "Transient Lookahead Horizon", 0.1, 10.0, 1.5, "ms", MacroRole::Space, "Lookahead delay allowing zero-overshoot gain trajectory calculation"))
        .with_param(DspParamSchema::choice("auto_release_profile", "Adaptive Auto-Release Curve", &["Instant Fast (EDM/Punch)", "Psychoacoustic Adaptive (Mastering)", "Opto Smooth (Acoustic/Jazz)", "Transparent Dual-Stage (Streaming)"], 1, "Dynamic time-constant smoothing release to eliminate low-frequency pumping"))
        .with_param(DspParamSchema::choice("oversampling_ratio", "FIR Oversampling Factor", &["2x Linear-Phase", "4x Polyphase FIR", "8x Polyphase True-Peak", "16x Ultra-Mastering"], 2, "Reconstruction oversampling rate resolving hidden analog waveform peaks"))
        .with_param(DspParamSchema::toggle("enable_dither_tpdf", "TPDF Mastering Triangular Dither", true, "Injects triangular probability density dither to prevent truncation quantization"))
    );
    descriptors.insert("DynamicTransientDeEsser".to_string(), DspNodeDescriptor::new("DynamicTransientDeEsser", "Adaptive Frequency-Tracking Dynamic Sibilance De-Esser", DspNodeCategory::DynamicsMaster, "Intelligent vocal de-esser featuring dynamic target frequency tracking, psychoacoustic threshold masking, sidechain bandpass audition, and peak-limiting compression")
        .with_param(DspParamSchema::knob("deesser_threshold_db", "Sibilance Detection Threshold", -48.0, 0.0, -22.0, "dB", MacroRole::Punch, "Sidechain signal level triggering dynamic gain reduction on vocal sibilance"))
        .with_param(DspParamSchema::knob("sibilance_center_frequency_hz", "Sibilant Center Frequency", 3000.0, 14000.0, 6800.0, "Hz", MacroRole::Tone, "Target frequency band for harsh 's', 'sh', and 't' phonetic energy"))
        .with_param(DspParamSchema::knob("attenuation_reduction_depth_db", "Maximum Sibilance Gain Reduction", 0.0, 24.0, 8.5, "dB", MacroRole::Punch, "Maximum permissible decibel clamp applied during peak harshness bursts"))
        .with_param(DspParamSchema::knob("tracking_bandwidth_octaves", "Sidechain Filter Bandwidth", 0.2, 2.5, 0.8, "oct", MacroRole::Character, "Selectivity Q bandwidth around sibilance center frequency"))
        .with_param(DspParamSchema::knob("deess_attack_time_ms", "Fast Attack Response Time", 0.1, 20.0, 1.2, "ms", MacroRole::Punch, "Integration rise time reacting to ultra-fast sibilant consonants"))
        .with_param(DspParamSchema::toggle("sidechain_audition_solo", "Audition Sidechain Filter Band Solo", false, "Isolates and solos detector sidechain band to pinpoint harsh frequencies"))
    );
    descriptors.insert("SubharmonicBassReinforcer".to_string(), DspNodeDescriptor::new("SubharmonicBassReinforcer", "Phase-Locked Sub-Octave Fundamental Synthesizer", DspNodeCategory::DynamicsMaster, "Low-end psychoacoustic bass reinforcer synthesizing fundamental sine waves locked to incoming musical transients and sub-octave pitch intervals with dynamic ducking")
        .with_param(DspParamSchema::knob("subharmonic_gain_db", "Synthesized Sub-Bass Level", -36.0, 12.0, 0.0, "dB", MacroRole::Punch, "Volume gain of generated phase-locked subharmonic waveform"))
        .with_param(DspParamSchema::choice("sub_octave_interval", "Subharmonic Frequency Divider", &["1 Octave Below (Half Frequency)", "2 Octaves Below (Sub-Sub Bass)", "Fifth Below (P1/P5 Triad Bass)"], 0, "Frequency division ratio locked to incoming program pitch"))
        .with_param(DspParamSchema::knob("upper_cutoff_frequency_hz", "Sub-Bass Synthesis Low-Pass Filter", 40.0, 150.0, 85.0, "Hz", MacroRole::Tone, "Steep cutoff ceiling keeping generated sub-bass beneath mid bass clarity"))
        .with_param(DspParamSchema::knob("phase_lock_tracking_speed_ms", "Pitch Tracking Phase Lock Speed", 1.0, 30.0, 5.0, "ms", MacroRole::Punch, "Time constant of phase-locked loop (PLL) tracking transient pitch changes"))
        .with_param(DspParamSchema::knob("dynamics_ducking_sensitivity", "Kick Drum Masking Ducking", 0.0, 1.0, 0.45, "duck", MacroRole::Character, "Sidechain ducking reducing sub-bass when intense transients strike"))
        .with_param(DspParamSchema::knob("saturation_odd_harmonics_mix", "Sub-Bass Harmonic Drive Saturation", 0.0, 1.0, 0.25, "sat", MacroRole::Tone, "Adds 3rd and 5th harmonic overtones to make sub-bass audible on small speakers"))
    );
    descriptors.insert("AdaptiveSpectralDenoiser".to_string(), DspNodeDescriptor::new("AdaptiveSpectralDenoiser", "Psychoacoustic Multiband FFT Noise Profiler & Spectral Gate", DspNodeCategory::DynamicsMaster, "High-resolution spectral fingerprint subtraction filter with dynamic psychoacoustic auditory masking thresholding for real-time hiss, hum, and background noise attenuation")
        .with_param(DspParamSchema::knob("noise_reduction_depth_db", "Spectral Denoising Attenuation Depth", 0.0, 40.0, 14.0, "dB", MacroRole::Tone, "Maximum attenuation applied to bins below noise floor threshold"))
        .with_param(DspParamSchema::knob("masking_threshold_bias_db", "Auditory Masking Sensitivity Offset", -12.0, 12.0, 0.0, "dB", MacroRole::Character, "Threshold bias balancing musical transparency against noise reduction aggressively"))
        .with_param(DspParamSchema::knob("spectral_smoothing_factor", "Temporal Spectral Envelope Smoothing", 0.0, 1.0, 0.65, "%", MacroRole::Space, "FFT bin temporal smoothing preventing musical noise and chirping artifacts"))
        .with_param(DspParamSchema::choice("fft_window_size_bins", "FFT Transform Resolution", &["512 Bins (Low Latency)", "1024 Bins (Balanced Studio)", "2048 Bins (Mastering Precision)", "4096 Bins (Surgical Denoise)"], 1, "Frequency analysis window size trading temporal transient response for frequency resolution"))
        .with_param(DspParamSchema::toggle("learn_noise_fingerprint", "Profile Background Stationary Noise", false, "Samples incoming audio frame to capture stationary spectral noise fingerprint"))
        .with_param(DspParamSchema::toggle("audition_noise_residual_delta", "Audition Attenuated Noise Residual", false, "Solos removed spectral noise delta for precise threshold calibration"))
    );
    descriptors.insert("TruePeakMasteringClipper".to_string(), DspNodeDescriptor::new("TruePeakMasteringClipper", "Polyphase Oversampled Inter-Sample Peak Hard/Soft Clipper", DspNodeCategory::DynamicsMaster, "Zero-latency mastering clipper with 8x polyphase reconstruction oversampling preventing inter-sample peak (ISP) digital overs while providing adjustable soft-knee saturation")
        .with_param(DspParamSchema::knob("ceiling_threshold_dbfs", "Brickwall Clipping Peak Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Final output ceiling preventing inter-sample peak clipping on consumer DACs"))
        .with_param(DspParamSchema::knob("clipper_input_gain_db", "Pre-Clipper Input Drive Gain", 0.0, 18.0, 3.0, "dB", MacroRole::Punch, "Input gain pushing program material into nonlinear clipping zone"))
        .with_param(DspParamSchema::knob("soft_knee_radius_db", "Clipping Transition Soft-Knee Radius", 0.0, 6.0, 1.2, "dB", MacroRole::Tone, "Soft-knee rounding radius smoothing transition into hard clipping"))
        .with_param(DspParamSchema::choice("clipper_transfer_curve", "Nonlinear Clipping Transfer Curve", &["Mastering Soft-Knee Tanh", "Aggressive Quadratic Hard-Clip", "Cubic Analog Desk Clip", "Zero-Overshoot ISP Brickwall"], 0, "Mathematical waveshaping function governing harmonic coloration"))
        .with_param(DspParamSchema::choice("oversampling_polyphase_factor", "Polyphase Oversampling Multiplier", &["2x Polyphase", "4x Polyphase", "8x True-Peak Mastering", "16x Ultra-Def"], 2, "Reconstruction oversampling rate eliminating aliasing in nonlinear clipping"))
        .with_param(DspParamSchema::choice("dither_noise_shaping_curve", "Mastering Dither & Noise Shaping", &["None (Bypass)", "TPDF Flat Triangular", "Psychoacoustic High-Pass Shaped", "Mastering Lipshitz-Vanderkooy"], 2, "Final word-length reduction noise shaping preventing truncation distortion"))
    );
    descriptors.insert("HarmonicSubBassExciter".to_string(), DspNodeDescriptor::new("HarmonicSubBassExciter", "Pitch-Tracking Subharmonic Waveform Generator & Bass Enhancer", DspNodeCategory::DynamicsMaster, "Intelligent bass synth module tracking fundamental frequencies between 30 Hz and 150 Hz to synthesize phase-coherent sub-octave sine/triangle tones with dynamic bass envelope clamping")
        .with_param(DspParamSchema::knob("subharmonic_octave_blend_db", "Sub-Octave Synthesizer Blend Level", -30.0, 6.0, 0.0, "dB", MacroRole::Punch, "Volume level of generated sub-octave fundamental added to mix"))
        .with_param(DspParamSchema::knob("direct_fundamental_reinforce_db", "Original Fundamental Presence Boost", -12.0, 12.0, 2.5, "dB", MacroRole::Punch, "Direct amplification applied to detected original low-frequency fundamental"))
        .with_param(DspParamSchema::knob("tracking_frequency_cutoff_hz", "Sub-Bass Tracking Low-Pass Ceiling", 40.0, 200.0, 90.0, "Hz", MacroRole::Tone, "Upper frequency boundary for pitch tracking fundamental extraction"))
        .with_param(DspParamSchema::choice("harmonic_waveform_shape", "Synthesized Sub-Harmonic Waveform", &["Pure Phase-Locked Sine", "Warm Rounded Triangle", "Even-Harmonic Tube Wave", "Octave Sub-Square"], 0, "Waveform timbre used for synthetic sub-octave tone"))
        .with_param(DspParamSchema::knob("dynamics_envelope_attack_ms", "Subharmonic Follower Attack Time", 0.5, 50.0, 8.0, "ms", MacroRole::Punch, "Response speed tracking sudden bass transients without click artifacts"))
        .with_param(DspParamSchema::knob("dynamics_envelope_release_ms", "Subharmonic Follower Release Time", 10.0, 500.0, 120.0, "ms", MacroRole::Space, "Decay smoothing preventing bass rumble and flutter modulation"))
    );
    descriptors.insert("OpticalVactrolCompressor".to_string(), DspNodeDescriptor::new("OpticalVactrolCompressor", "Dual-Time-Constant Optoelectronic Vactrol Leveling Amplifier", DspNodeCategory::DynamicsMaster, "Electro-optical leveling amplifier modeled after the classic LA-2A featuring electroluminescent light source lag, photoresistor memory effect, and program-dependent release envelope")
        .with_param(DspParamSchema::knob("peak_reduction_drive_db", "Optical Cell Peak Reduction Drive", 0.0, 40.0, 18.0, "dB", MacroRole::Punch, "Input gain driving electroluminescent panel into photoresistor cell"))
        .with_param(DspParamSchema::knob("output_make_up_gain_db", "Tube Make-Up Amplifier Gain", 0.0, 30.0, 8.5, "dB", MacroRole::Punch, "Post-attenuation gain restoring program signal to desired output level"))
        .with_param(DspParamSchema::choice("compression_ratio_mode", "Dynamic Compression Ratio Characteristic", &["Compress (3:1 Soft-Knee)", "Limit (10:1 Brickwall)"], 0, "Selects between smooth leveling curve (Compress) and peak clamping (Limit)"))
        .with_param(DspParamSchema::knob("vactrol_photocell_decay_memory", "Photoresistor Chemical Memory Lag", 0.0, 1.0, 0.65, "mem", MacroRole::Character, "Models cadmium sulfide photocell memory causing release to lengthen after sustained peaks"))
        .with_param(DspParamSchema::knob("emphasis_hf_sidechain_filter_db", "High-Frequency Sidechain Pre-Emphasis", 0.0, 12.0, 4.0, "dB", MacroRole::Tone, "Pre-emphasis shelving filter increasing sensitivity to harsh high-frequency transients"))
        .with_param(DspParamSchema::knob("tube_output_stage_warmth", "12AX7 Vacuum Tube Output Saturation", 0.0, 1.0, 0.35, "warm", MacroRole::Tone, "Subtle harmonic coloration from cathode follower tube output transformer stage"))
    );
    descriptors.insert("DynamicLoudnessTargetLeveler".to_string(), DspNodeDescriptor::new("DynamicLoudnessTargetLeveler", "EBU R128 / ITU-R BS.1770-4 Automated Loudness Leveler", DspNodeCategory::DynamicsMaster, "Broadcast-standard loudness normalizer with dialogue gated integration, sliding momentary loudness measurement, and true-peak brickwall protection")
        .with_param(DspParamSchema::knob("target_loudness_lufs", "Target Program Integrated Loudness", -30.0, -9.0, -14.0, "LUFS", MacroRole::Punch, "Target delivery loudness specification (-14 LUFS streaming, -23 LUFS broadcast)"))
        .with_param(DspParamSchema::knob("maximum_true_peak_dbfs", "Ceiling Inter-Sample Peak Limit", -6.0, 0.0, -1.0, "dBTP", MacroRole::Punch, "Brickwall true-peak safety ceiling preventing inter-sample DAC clipping"))
        .with_param(DspParamSchema::knob("leveler_speed_response", "Adaptive RMS/LUFS Gain Ride Speed", 0.1, 5.0, 1.0, "spd", MacroRole::Character, "Dynamics ride speed adjusting program level to target loudness smoothly"))
        .with_param(DspParamSchema::knob("maximum_boost_gain_db", "Maximum Allowable Upward Leveling Boost", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Ceiling limit on upward automatic gain compensation for quiet sections"))
        .with_param(DspParamSchema::knob("maximum_cut_gain_db", "Maximum Allowable Downward Attenuation", 0.0, 24.0, 12.0, "dB", MacroRole::Tone, "Floor limit on downward automatic gain attenuation for loud passages"))
        .with_param(DspParamSchema::toggle("dialogue_gating_speech", "Apply ITU-R BS.1770-4 -10 LU Relative Gating", true, "Excludes silent passages and background noise from integrated loudness metering"))
    );
    descriptors.insert("MultibandTransientHarmonizer".to_string(), DspNodeDescriptor::new("MultibandTransientHarmonizer", "4-Band Dynamic Transient vs. Sustain Harmonizer", DspNodeCategory::DynamicsMaster, "Linear-phase multiband dynamic processor independently extracting transient attacks and resonant sustains to apply targeted harmonic coloration")
        .with_param(DspParamSchema::knob("low_band_transient_boost_db", "Bass Band Attack Transient Punch", -12.0, 12.0, 3.0, "dB", MacroRole::Punch, "Dynamic boost/cut applied specifically to bass transient attacks"))
        .with_param(DspParamSchema::knob("low_mid_sustain_warmth_db", "Low-Mid Sustain Harmonic Warmth Drive", 0.0, 18.0, 2.5, "dB", MacroRole::Tone, "Harmonic overdrive applied to low-mid decaying body resonance"))
        .with_param(DspParamSchema::knob("high_mid_transient_crisp_db", "High-Mid Transient Snap & Crack", -12.0, 12.0, 2.0, "dB", MacroRole::Punch, "Enhances snare snap, pick click, and percussive high-mid edge"))
        .with_param(DspParamSchema::knob("high_band_air_sustain_db", "Air Band Shimmer & Sustain Extension", -12.0, 12.0, 1.5, "dB", MacroRole::Space, "Generates airy harmonic sustain tails above high crossover frequency"))
        .with_param(DspParamSchema::knob("transient_detection_sensitivity", "Envelope Differential Attack Sensitivity", 0.1, 2.0, 1.0, "sens", MacroRole::Character, "Sensitivity threshold separating attack transients from tonal sustain"))
        .with_param(DspParamSchema::toggle("audition_isolated_transients", "Solo Transient Signal Layer Across All Bands", false, "Solos isolated transient attacks for precision threshold calibration"))
    );
    descriptors.insert("MasteringTruePeakDeClipper".to_string(), DspNodeDescriptor::new("MasteringTruePeakDeClipper", "Inter-Sample True Peak Spectral De-Clipper & Reconstructor", DspNodeCategory::DynamicsMaster, "High-order cubic spline and autoregressive spectral reconstruction engine repairing hard digital flat-top clipping and eliminating harsh distortion")
        .with_param(DspParamSchema::knob("clipping_threshold_dbfs", "Flat-Top Digital Clipping Detection Threshold", -6.0, 0.0, -0.2, "dBFS", MacroRole::Tone, "Detection sensitivity threshold where consecutive flattened samples are repaired"))
        .with_param(DspParamSchema::knob("reconstruction_crest_headroom_db", "Dynamic Headroom Expansion Restoration", 0.0, 12.0, 3.5, "dB", MacroRole::Punch, "Headroom margin allocated to unclipped reconstructed waveform peaks"))
        .with_param(DspParamSchema::choice("spline_interpolation_order", "Peak Reconstruction Interpolation Algorithm", &["Natural Cubic Spline", "Hermite Akima Slope Interpolation", "Autoregressive Linear Prediction", "Spectral Sinusoidal Extrapolation"], 1, "Mathematical method used to recreate shaved-off peak contours"))
        .with_param(DspParamSchema::knob("harmonic_decay_estimation", "High-Order Clipping Harmonic Attenuation", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Spectral filtering strength attenuating harsh odd harmonics from clipping"))
        .with_param(DspParamSchema::toggle("audition_clipped_spikes_solo", "Solo Isolated Reconstructed Clipping Spikes", false, "Solos isolated reconstructed peak energy removed or repaired by processor"))
        .with_param(DspParamSchema::toggle("post_repair_true_peak_limit", "True-Peak Brickwall Inter-Sample Safety Guard", true, "Applies transparent lookahead true-peak limiting to prevent post-repair overshoots"))
    );
    descriptors.insert("SpectralTransientDeClicker".to_string(), DspNodeDescriptor::new("SpectralTransientDeClicker", "Surgical Sub-Millisecond Spectral De-Clicker & Plosive Tamer", DspNodeCategory::DynamicsMaster, "Sub-band energy differential transient detector isolating and suppressing vinyl clicks, vocal mouth ticks, and digital buffer discontinuities without dulling high frequencies")
        .with_param(DspParamSchema::knob("click_sensitivity_threshold", "Click Detection Sensitivity Ratio", 0.1, 10.0, 3.5, "sens", MacroRole::Punch, "Threshold ratio above local spectral noise floor triggering click repair"))
        .with_param(DspParamSchema::knob("transient_burst_width_us", "Maximum Click Burst Duration", 20.0, 2000.0, 250.0, "us", MacroRole::Tone, "Maximum temporal duration recognized as impulsive click artifact"))
        .with_param(DspParamSchema::knob("spectral_smoothing_decay_ms", "Autoregressive Repair Window", 0.5, 20.0, 3.0, "ms", MacroRole::Character, "Autoregressive interpolation window duration replacing corrupted samples"))
        .with_param(DspParamSchema::knob("high_frequency_bias_weight", "High Frequency Detection Weight", 0.0, 2.0, 1.2, "bias", MacroRole::Tone, "Weights click detection towards upper frequency spectrum (above 4 kHz)"))
        .with_param(DspParamSchema::knob("crest_factor_headroom_db", "Permitted Crest Factor Headroom", 0.0, 12.0, 4.0, "dB", MacroRole::Punch, "Allowed crest factor before sample is flagged as anomalous transient"))
        .with_param(DspParamSchema::toggle("audition_delta_clicks_solo", "Solo Removed Click Signal Delta", false, "Solos isolated clicks and ticks removed by surgical filter"))
    );
    descriptors.insert("MultichannelLoudnessRadarMeter".to_string(), DspNodeDescriptor::new("MultichannelLoudnessRadarMeter", "EBU R128 / ITU-R BS.1770 Real-Time Loudness Radar Meter", DspNodeCategory::DynamicsMaster, "Mastering loudness analyzer displaying concentric 360-degree radar history of momentary, short-term (3s), and integrated LUFS with Loudness Range (LRA)")
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Program Integrated Loudness", -30.0, -9.0, -14.0, "LUFS", MacroRole::Punch, "Target delivery standard integrated loudness line (Streaming -14, Broadcast -23)"))
        .with_param(DspParamSchema::choice("radar_sweep_duration_sec", "Radar Circular History Sweep Time", &["1 Minute", "4 Minutes", "12 Minutes", "30 Minutes"], 1, "Time period represented by one complete 360-degree radar circle sweep"))
        .with_param(DspParamSchema::knob("momentary_ceiling_lufs", "Momentary Over-Compression Ceiling", -18.0, -3.0, -9.0, "LUFS", MacroRole::Punch, "Upper threshold highlighting momentary over-compression in warning color"))
        .with_param(DspParamSchema::knob("loudness_range_target_lu", "Target Loudness Range (LRA) Bracket", 2.0, 20.0, 8.0, "LU", MacroRole::Character, "Expected Loudness Range (LRA) tolerance interval for program material"))
        .with_param(DspParamSchema::knob("true_peak_hold_ms", "True-Peak Intersample Peak Hold Time", 500.0, 10000.0, 3000.0, "ms", MacroRole::Tone, "True-peak inter-sample maximum hold decay time"))
        .with_param(DspParamSchema::toggle("enable_relative_gate_metering", "Apply ITU -10 LU Relative Gating", true, "Applies ITU -10 LU relative gating to ignore quiet pauses in integrated calculation"))
    );
    descriptors.insert("TubeOptoMasterCompressor".to_string(), DspNodeDescriptor::new("TubeOptoMasterCompressor", "Dual-Stage Vacuum Tube & Optoelectronic Mastering Compressor", DspNodeCategory::DynamicsMaster, "Vintage leveling amplifier combining non-linear vacuum tube push-pull amplification with electroluminescent optical cell two-stage release curves")
        .with_param(DspParamSchema::knob("compression_threshold_db", "Optical Compression Threshold", -36.0, 0.0, -16.0, "dB", MacroRole::Punch, "Threshold level above which optical leveling attenuation activates"))
        .with_param(DspParamSchema::knob("compression_ratio", "Leveling Compression Ratio Slope", 1.2, 20.0, 3.5, ":1", MacroRole::Punch, "Dynamic slope ratio transitioning into gentle soft-knee compression"))
        .with_param(DspParamSchema::knob("opto_cell_memory_decay", "Optocell Memory Lag Release Time", 50.0, 2500.0, 350.0, "ms", MacroRole::Character, "Electroluminescent persistence creating dual-stage release recovery"))
        .with_param(DspParamSchema::knob("tube_harmonic_color_db", "Triode Plate Saturation Harmonic Drive", 0.0, 18.0, 4.0, "dB", MacroRole::Tone, "Triode plate saturation adding even-order musical harmonic thickness"))
        .with_param(DspParamSchema::knob("sidechain_hpf_cutoff_hz", "Detector Sidechain Highpass Filter", 20.0, 300.0, 90.0, "Hz", MacroRole::Tone, "High-pass filter in detector loop preventing bass energy from pumping mix"))
        .with_param(DspParamSchema::toggle("stereo_sidechain_link", "Link Left/Right Optical Detectors", true, "Links left and right detector cells to preserve rock-solid center stereo imaging"))
    );
    descriptors.insert("DynamicSidechainSpectralDucker".to_string(), DspNodeDescriptor::new("DynamicSidechainSpectralDucker", "Frequency-Selective Real-Time Sidechain Spectral Ducker", DspNodeCategory::DynamicsMaster, "Multi-band dynamic spectral carver ducking precise interfering frequency bands in music tracks only when sidechain vocal or kick drum signal occurs")
        .with_param(DspParamSchema::knob("sidechain_trigger_sensitivity_db", "Sidechain Trigger Detection Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Sidechain key input threshold triggering spectral attenuation"))
        .with_param(DspParamSchema::knob("maximum_ducking_attenuation_db", "Maximum Spectral Carving Cut Depth", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Maximum depth of spectral carving applied to conflicting frequencies"))
        .with_param(DspParamSchema::knob("attack_carving_speed_ms", "Spectral Ducking Attack Response Time", 0.5, 50.0, 5.0, "ms", MacroRole::Punch, "Response speed carving space when sidechain transient is detected"))
        .with_param(DspParamSchema::knob("release_recovery_speed_ms", "Spectral Restoration Release Time", 10.0, 500.0, 80.0, "ms", MacroRole::Space, "Recovery time restoring original frequency balance after sidechain ceases"))
        .with_param(DspParamSchema::knob("spectral_focus_center_hz", "Primary Masking Conflict Frequency", 40.0, 10000.0, 1200.0, "Hz", MacroRole::Tone, "Center frequency of primary spectral masking conflict region"))
        .with_param(DspParamSchema::toggle("listen_carved_masking_delta", "Audition Removed Masking Energy Delta", false, "Solos dynamically removed audio energy for transparent mixing inspection"))
    );
    descriptors.insert("VariMuMasterLimiter".to_string(), DspNodeDescriptor::new("VariMuMasterLimiter", "Variable-Mu Valve Mastering Peak Limiter & Bus Glue", DspNodeCategory::DynamicsMaster, "Remote-cutoff vacuum tube limiter with smooth multi-pole feedback sidechain, self-adjusting dynamic ratio, and analog transformer coupling")
        .with_param(DspParamSchema::knob("threshold_ceiling_db", "Variable-Mu Limiting Threshold", -24.0, 0.0, -10.0, "dB", MacroRole::Punch, "Peak ceiling threshold above which tube gain reduction activates"))
        .with_param(DspParamSchema::knob("tube_recovery_time_ms", "Triode Discharge Recovery Time", 50.0, 2000.0, 250.0, "ms", MacroRole::Character, "Program-dependent release recovery time for musical dynamic breathing"))
        .with_param(DspParamSchema::knob("detector_sidechain_hpf_hz", "Sidechain High-Pass Bass Filter", 20.0, 300.0, 80.0, "Hz", MacroRole::Tone, "High-pass filter in sidechain loop preventing heavy bass from pumping limiter"))
        .with_param(DspParamSchema::knob("master_glue_compression_ratio", "Peak Compression Dynamic Ratio", 1.5, 12.0, 4.0, ":1", MacroRole::Punch, "Dynamic slope ratio transitioning gently into soft-knee peak limiting"))
        .with_param(DspParamSchema::knob("output_makeup_gain_db", "Clean Transformer Makeup Gain", -6.0, 18.0, 2.0, "dB", MacroRole::Punch, "Clean output post-limiting gain boost"))
        .with_param(DspParamSchema::toggle("dual_mono_independent_detectors", "Dual-Mono Independent Detector Cell", false, "Unlinks stereo detector cells for widest possible stereo image without cross-pumping"))
    );
    descriptors.insert("TransientSustainDecomposer".to_string(), DspNodeDescriptor::new("TransientSustainDecomposer", "Dual-Path Transient & Sustained Harmonic Splitter", DspNodeCategory::DynamicsMaster, "Real-time non-linear time-frequency decomposition separating audio into sharp percussive transients and continuous tonal harmonics for independent processing")
        .with_param(DspParamSchema::knob("transient_detection_sensitivity", "Transient Attack Detection Sensitivity", 0.1, 10.0, 2.0, "sens", MacroRole::Punch, "Sensitivity of non-linear energy differential detector identifying transients"))
        .with_param(DspParamSchema::knob("transient_path_gain_db", "Isolated Transient Stream Gain", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Gain adjustment applied exclusively to isolated transient drum and pluck attacks"))
        .with_param(DspParamSchema::knob("sustain_path_gain_db", "Isolated Sustained Body Gain", -24.0, 12.0, 0.0, "dB", MacroRole::Space, "Gain adjustment applied exclusively to continuous body tones and reverb tails"))
        .with_param(DspParamSchema::knob("smoothing_transition_time_ms", "Stream Crossfade Transition Time", 1.0, 50.0, 8.0, "ms", MacroRole::Tone, "Temporal boundary crossfade smoothing split signals to prevent phase clicks"))
        .with_param(DspParamSchema::knob("transient_channel_stereo_width", "Transient Layer Stereo Panorama Width", 0.0, 200.0, 100.0, "%", MacroRole::Space, "Stereo width applied specifically to transient stream (0% = mono focus, 200% = wide)"))
        .with_param(DspParamSchema::toggle("solo_transient_layer_only", "Solo Isolated Transient Stream", false, "Solos isolated transient layer for surgical acoustic inspection"))
    );
    descriptors.insert("ParallelUpwardExpansionMaximizer".to_string(), DspNodeDescriptor::new("ParallelUpwardExpansionMaximizer", "Parallel Upward Dynamic Expander & Micro-Dynamic Maximizer", DspNodeCategory::DynamicsMaster, "Dual-envelope parallel upward expansion stage boosting low-level acoustic nuances, reverb tails, and breath details without altering crest factor")
        .with_param(DspParamSchema::knob("expansion_threshold_floor_db", "Upward Expansion Detection Floor", -60.0, -18.0, -36.0, "dB", MacroRole::Punch, "Input level threshold below which quiet details are pushed upward"))
        .with_param(DspParamSchema::knob("upward_expansion_ratio", "Low-Level Dynamic Expansion Slope", 1.1, 4.0, 1.8, ":1", MacroRole::Punch, "Expansion slope ratio lifting whispers, room ambiance, and delicate nuances"))
        .with_param(DspParamSchema::knob("micro_dynamics_boost_gain_db", "Micro-Detail Parallel Boost Gain", 0.0, 18.0, 4.5, "dB", MacroRole::Punch, "Maximum gain boost applied to low-level acoustic details"))
        .with_param(DspParamSchema::knob("attack_envelope_window_ms", "Detector Window Attack Response", 0.5, 100.0, 15.0, "ms", MacroRole::Punch, "Reaction time for level detector smoothing transient bursts"))
        .with_param(DspParamSchema::knob("release_recovery_envelope_ms", "Upward Gain Smooth Decay Release", 20.0, 1000.0, 180.0, "ms", MacroRole::Space, "Smooth decay releasing boosted detail back toward unity gain"))
        .with_param(DspParamSchema::toggle("audition_expanded_component_solo", "Solo Isolated Low-Level Details", false, "Solos dynamically lifted low-level components for surgical calibration"))
    );
    descriptors.insert("OpticalTubeCompressorLimiter".to_string(), DspNodeDescriptor::new("OpticalTubeCompressorLimiter", "Classic Electro-Optical Teletronix Attenuator & Valve Gain Cell", DspNodeCategory::DynamicsMaster, "Vintage T4B optical attenuator simulation with two-stage photoconductive lag decay, high-frequency emphasis, and 12AX7 vacuum tube output makeup")
        .with_param(DspParamSchema::knob("peak_reduction_attenuation_db", "Optical Peak Reduction Attenuation", 0.0, 40.0, 10.0, "dB", MacroRole::Punch, "Threshold level controlling optical photocell light emission and attenuation"))
        .with_param(DspParamSchema::knob("electro_optical_lag_hysteresis", "Photocell Memory Lag Hysteresis", 0.0, 1.0, 0.45, "%", MacroRole::Character, "Multi-stage physical photocell recovery lag creating musical program-dependent release"))
        .with_param(DspParamSchema::knob("vacuum_tube_makeup_gain_db", "Triode Tube Makeup Amplifier Gain", 0.0, 30.0, 8.0, "dB", MacroRole::Punch, "Post-attenuation vacuum tube gain stage adding subtle warm harmonic color"))
        .with_param(DspParamSchema::knob("sidechain_emphasis_freq_hz", "Sidechain Pre-Emphasis Center Frequency", 100.0, 10000.0, 1000.0, "Hz", MacroRole::Tone, "Frequency-dependent sidechain emphasis boosting sibilance or harsh transients"))
        .with_param(DspParamSchema::choice("opto_cell_response_mode", "Optical Response Speed Profile", &["Slow Dual-Stage", "Fast Attack Studio", "Vintage Memory Lag"], 0, "Selects photocell attack and release decay profile"))
        .with_param(DspParamSchema::toggle("link_stereo_optical_detectors", "Stereo Optical Detector Cell Link", true, "Links left and right opto-cells to prevent stereo image shift under heavy compression"))
    );
    descriptors.insert("TransientShaperDualBand".to_string(), DspNodeDescriptor::new("TransientShaperDualBand", "Dual-Band Frequency-Split Transient & Sustain Envelope Shaper", DspNodeCategory::DynamicsMaster, "Splits audio with zero-delay crossover filter into bass and treble streams, applying independent differential envelope followers to reshape attack impact and sustain tail")
        .with_param(DspParamSchema::knob("crossover_frequency_hz", "Bass / Treble Split Crossover", 80.0, 2000.0, 350.0, "Hz", MacroRole::Tone, "Crossover boundary frequency separating low punch from high snap"))
        .with_param(DspParamSchema::knob("low_band_attack_gain_db", "Low Band Punch Attack Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Boosts or attenuates initial low-end transient impact (kick thud)"))
        .with_param(DspParamSchema::knob("low_band_sustain_gain_db", "Low Band Resonance Sustain Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Lengthens or suppresses low-frequency ringing and boom"))
        .with_param(DspParamSchema::knob("high_band_attack_gain_db", "High Band Snap Attack Gain", -24.0, 24.0, 3.0, "dB", MacroRole::Punch, "Sharpens high-frequency transient click and pick attack"))
        .with_param(DspParamSchema::knob("high_band_sustain_gain_db", "High Band Room Sustain Gain", -24.0, 24.0, -2.0, "dB", MacroRole::Punch, "Controls high-frequency reverberant room ambience and cymbal decay"))
        .with_param(DspParamSchema::toggle("clip_transient_peaks_soft_knee", "Soft-Knee Peak Clipper Safety Guard", true, "Applies smooth saturation ceiling preventing excessive boosted transients from clipping"))
    );
    descriptors.insert("SubharmonicBassExciter".to_string(), DspNodeDescriptor::new("SubharmonicBassExciter", "Analog Subharmonic Waveform Synthesizer & Low-End Bass Extender", DspNodeCategory::DynamicsMaster, "Analyzes low-frequency waveform zero-crossings and synthesizes continuous octave-down and two-octave-down subharmonic waveforms with analog saturation")
        .with_param(DspParamSchema::knob("sub_fundamental_octave_blend", "1st Subharmonic (-1 Octave) Level", 0.0, 1.0, 0.5, "%", MacroRole::Punch, "Synthesizes clean fundamental one octave below incoming bass notes"))
        .with_param(DspParamSchema::knob("sub_second_octave_blend", "2nd Subharmonic (-2 Octaves) Level", 0.0, 1.0, 0.2, "%", MacroRole::Punch, "Synthesizes deep earthquake subharmonic two octaves below incoming audio"))
        .with_param(DspParamSchema::knob("lowpass_isolation_cutoff_hz", "Sub Detection Lowpass Filter", 40.0, 250.0, 90.0, "Hz", MacroRole::Tone, "Isolates pure bass frequencies from rest of program material for pitch tracking"))
        .with_param(DspParamSchema::knob("dynamic_sub_drive_saturation", "Subharmonic Drive & Saturation", 0.0, 1.0, 0.35, "drive", MacroRole::Character, "Adds harmonic saturation to synthesized sub waves allowing them to cut on smaller speakers"))
        .with_param(DspParamSchema::knob("bass_mono_fold_hz", "Low Bass Mono Summing Frequency", 30.0, 200.0, 100.0, "Hz", MacroRole::Space, "Folds all stereo information below this cutoff frequency into rock-solid mono center"))
        .with_param(DspParamSchema::toggle("phase_invert_subharmonics", "Phase-Invert Synthesized Subharmonics", false, "Inverts subharmonic phase 180 degrees to resolve destructive acoustic room cancellations"))
    );
    descriptors.insert("MasteringTruePeakDitherLimiter".to_string(), DspNodeDescriptor::new("MasteringTruePeakDitherLimiter", "Integrated True-Peak Intersample Brickwall Limiter & Psychoacoustic Dither", DspNodeCategory::DynamicsMaster, "Mastering final-stage limiter featuring 8x polyphase oversampling for true-peak ISP detection, auto-adapting release envelope, and psychoacoustic noise-shaped dither")
        .with_param(DspParamSchema::knob("ceiling_true_peak_dbfs", "Master True-Peak Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Absolute maximum intersample true-peak limit permitted on final audio output"))
        .with_param(DspParamSchema::knob("intersample_oversampling_ratio", "Oversampling True-Peak Factor", 1.0, 16.0, 8.0, "x", MacroRole::Tone, "Linear-phase FIR oversampling ratio detecting intersample peaks between samples"))
        .with_param(DspParamSchema::knob("release_dynamic_auto_recovery_ms", "Program-Adaptive Release Recovery Time", 5.0, 500.0, 45.0, "ms", MacroRole::Punch, "Auto-adjusting release time recovering gain smoothly without audible pumping"))
        .with_param(DspParamSchema::knob("psychoacoustic_noise_shaping_db", "F-Weighting Dither Noise Shaping Depth", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Pushes quantization noise floor upward into less sensitive 15-20 kHz hearing region"))
        .with_param(DspParamSchema::choice("dither_target_bit_depth", "Target Quantization Export Bit Depth", &["16-Bit Redbook Audio", "24-Bit Studio High-Res", "32-Bit Floating Point"], 0, "Selects output bit resolution for word-length reduction"))
        .with_param(DspParamSchema::toggle("compliance_itu_r_bs1770_check", "ITU-R BS.1770 True-Peak Compliance", true, "Conforms peak detector filtering strictly to ITU-R BS.1770-4 broadcast loudness standard"))
    );
    descriptors.insert("MultibandUpwardDownwardCompressor".to_string(), DspNodeDescriptor::new("MultibandUpwardDownwardCompressor", "4-Band OTT-Style Upward/Downward Dynamic Range Squeezer", DspNodeCategory::DynamicsMaster, "Aggressive multiband dynamic processor performing concurrent upward expansion on quiet details and downward brickwall compression on peaks for hyper-dense modern production")
        .with_param(DspParamSchema::knob("crossover_low_hz", "Sub-Bass / Low-Mid Crossover", 40.0, 500.0, 120.0, "Hz", MacroRole::Tone, "Split point between sub frequencies and midrange body"))
        .with_param(DspParamSchema::knob("crossover_high_hz", "High-Mid / Treble Crossover", 1500.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Split point between core vocal presence and airy transients"))
        .with_param(DspParamSchema::knob("upward_expansion_depth_db", "Upward Compression Lift Gain", 0.0, 36.0, 14.0, "dB", MacroRole::Punch, "Maximum gain boost applied to quiet background ambiance and low-level details"))
        .with_param(DspParamSchema::knob("downward_compression_ratio", "Downward Peak Compression Ratio", 1.0, 20.0, 6.0, ":1", MacroRole::Punch, "Compression ratio taming high-energy transients above threshold"))
        .with_param(DspParamSchema::knob("dynamics_response_time_ms", "Attack / Release Envelope Ballistics", 1.0, 100.0, 25.0, "ms", MacroRole::Punch, "Speed scaling for compressor attack and release timing envelopes"))
        .with_param(DspParamSchema::toggle("enable_true_stereo_sidechain_link", "Stereo Detector Sidechain Linking", true, "Links left and right detection cells to prevent stereo image shifting"))
    );
    descriptors.insert("MasterTapeSaturatorMaximizer".to_string(), DspNodeDescriptor::new("MasterTapeSaturatorMaximizer", "Precision Mastering Tape Saturation, Head Bump & Loudness Maximizer", DspNodeCategory::DynamicsMaster, "Analog master tape emulation coupled to transparent soft-knee intersample peak limiter, delivering harmonic density, low-end magnetic weight, and maximum competitive loudness")
        .with_param(DspParamSchema::knob("tape_flux_drive_level_db", "Magnetic Tape Flux Drive Gain", 0.0, 20.0, 4.5, "dB", MacroRole::Character, "Input level driving tape magnetic oxide past saturation threshold"))
        .with_param(DspParamSchema::knob("low_head_bump_resonance_hz", "Reproduce Head Bump Bass Tuning", 30.0, 90.0, 55.0, "Hz", MacroRole::Punch, "Resonant low-frequency bass enhancement induced by magnetic head geometry"))
        .with_param(DspParamSchema::knob("high_frequency_tape_soften_khz", "High-Frequency Tape Roll-Off Knee", 10.0, 24.0, 18.5, "kHz", MacroRole::Tone, "Tape saturation softening high-frequency transient harshness"))
        .with_param(DspParamSchema::knob("even_odd_harmonic_saturation_blend", "Even vs Odd Harmonic Saturation Balance", -1.0, 1.0, 0.2, "bal", MacroRole::Character, "Balances second-order tube warmth against third-order tape crunch"))
        .with_param(DspParamSchema::knob("brickwall_output_margin_dbfs", "Master Output Brickwall Ceiling", -3.0, 0.0, -0.1, "dBFS", MacroRole::Punch, "Maximum output peak ceiling preventing DAC converter clipping"))
        .with_param(DspParamSchema::toggle("enable_constant_loudness_monitoring", "Auto-Gain Loudness Matched Auditioning", true, "Normalizes output gain so saturation character can be auditioned without loudness bias"))
    );
    descriptors.insert("StereoDynamicDeClipper".to_string(), DspNodeDescriptor::new("StereoDynamicDeClipper", "Adaptive Audio Forensic Waveform Restoration & Peak De-Clipper", DspNodeCategory::DynamicsMaster, "Forensic audio restoration DSP reconstructing sheared or squared-off digital waveform peaks using cubic spline interpolation and adaptive harmonic oversampling")
        .with_param(DspParamSchema::knob("clip_detection_threshold_dbfs", "Clipping Flat-Top Detection Threshold", -6.0, 0.0, -0.2, "dBFS", MacroRole::Punch, "Signal amplitude threshold above which squared-off peaks are detected"))
        .with_param(DspParamSchema::knob("cubic_spline_reconstruction_sharpness", "Cubic Spline Polynomial Tension", 0.1, 2.0, 0.95, "tens", MacroRole::Tone, "Curvature tension used when interpolating missing peak waveform tops"))
        .with_param(DspParamSchema::knob("dynamic_headroom_expansion_db", "Post-Restoration Headroom Attenuation", 0.0, 12.0, 3.5, "dB", MacroRole::Punch, "Attenuates signal gain to create dynamic headroom for reconstructed peak heights"))
        .with_param(DspParamSchema::knob("transient_edge_smoothing_ms", "Discontinuity Edge Smoothing Filter", 0.05, 5.0, 0.4, "ms", MacroRole::Tone, "Smooths harsh high-frequency odd harmonic spikes caused by brickwall clipping edges"))
        .with_param(DspParamSchema::choice("harmonic_aliasing_filter_mode", "Reconstruction Anti-Aliasing Oversampling", &["2x Polyphase FIR", "4x Minimum Phase", "8x Linear Phase Forensic"], 1, "Oversampling resolution used during non-linear curve fitting"))
        .with_param(DspParamSchema::toggle("audition_isolated_reconstructed_peaks", "Solo Reconstructed Peak Material (Delta)", false, "Auditions only the restored peaks that were synthesized back into the audio"))
    );
    descriptors.insert("SonoluminescenceCavitationSonifier".to_string(), DspNodeDescriptor::new("SonoluminescenceCavitationSonifier", "Acoustic Cavitation Bubble Picosecond Shockwave & Luminescence Sonifier", DspNodeCategory::DynamicsMaster, "Non-linear hydrodynamic Rayleigh-Plesset bubble dynamic model transforming intense acoustic peaks into hypersonic shockwave collapses and photonic burst pulses")
        .with_param(DspParamSchema::knob("cavitation_drive_amplitude_db", "Acoustic Pressure Drive Intensity", 0.0, 36.0, 18.0, "dB", MacroRole::Punch, "Sound pressure amplitude driving bubble past Blake cavitation threshold into collapse"))
        .with_param(DspParamSchema::knob("ambient_bubble_radius_microns", "Equilibrium Bubble Equilibrium Radius", 1.0, 50.0, 5.0, "um", MacroRole::Tone, "Initial unperturbed microbubble radius tuning resonance frequency"))
        .with_param(DspParamSchema::knob("liquid_viscous_damping_poise", "Liquid Medium Viscosity Damping", 0.005, 0.1, 0.01, "P", MacroRole::Character, "Fluid dynamic viscosity slowing bubble expansion and rounding collapse wavefronts"))
        .with_param(DspParamSchema::knob("shockwave_hypersonic_compression", "Hypersonic In-Rushing Gas Shock Mach", 0.1, 5.0, 1.8, "Mach", MacroRole::Punch, "Peak gas speed during final picosecond collapse producing intense sonic crackles"))
        .with_param(DspParamSchema::knob("luminescence_thermal_flash_target", "Core Quasi-Thermal Flash Temperature", 1000.0, 50000.0, 12000.0, "K", MacroRole::Tone, "Adiabatic compression core temperature converting acoustic energy into photon flash"))
        .with_param(DspParamSchema::toggle("enable_single_bubble_synchronous_mode", "Synchronous Single-Bubble Cavitation Mode", true, "Locks bubble collapse cycle to acoustic carrier fundamental for tonal resonance"))
    );
    descriptors.insert("AiAutonomousMasteringChain".to_string(), DspNodeDescriptor::new("AiAutonomousMasteringChain", "Autonomous Multi-Stage AI Mastering Balance & Loudness Chain", DspNodeCategory::DynamicsMaster, "Machine learning automated mastering chain executing neural genre spectral target curve matching, dynamic crest factor optimization, and True-Peak limiting")
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Master Integrated Loudness Target", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Target delivery loudness (e.g., -14 LUFS for streaming, -9 LUFS for club master)"))
        .with_param(DspParamSchema::knob("spectral_profile_matching_strength", "Genre Spectral Reference Matching", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Strength of automated dynamic FIR curve matching reference master profile"))
        .with_param(DspParamSchema::knob("dynamic_crest_expansion_factor", "Dynamic Crest Factor Transient Preserver", 0.5, 2.0, 1.0, "factor", MacroRole::Character, "Expands or controls punchiness and crest factor ratio during heavy limiting"))
        .with_param(DspParamSchema::knob("stereo_image_clarity_enhancement", "Stereo Width & Correlation Enhancer", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Expands side channel high-frequency clarity while locking sub-bass to center"))
        .with_param(DspParamSchema::knob("ceiling_true_peak_margin_db", "Brickwall True-Peak Output Ceiling", -3.0, 0.0, -0.8, "dBTP", MacroRole::Punch, "Maximum permissible inter-sample peak level preventing lossy codec clipping"))
        .with_param(DspParamSchema::toggle("enable_intersample_oversampled_guard", "8x Oversampled Intersample Peak Sentry", true, "Over-samples peak detector 8x to trap inter-sample DAC overs"))
    );
    descriptors.insert("SpectralEntropyTransientGate".to_string(), DspNodeDescriptor::new("SpectralEntropyTransientGate", "Spectral Entropy & Shannon Information Transient Dynamic Gate", DspNodeCategory::DynamicsMaster, "Intelligent dynamic gate analyzing informational entropy and spectral distribution to separate transient strikes from ambient noise floors")
        .with_param(DspParamSchema::knob("entropy_gate_threshold", "Shannon Entropy Gate Threshold", 0.1, 1.0, 0.45, "entropy", MacroRole::Punch, "Normalized entropy threshold separating structured tones from noisy air"))
        .with_param(DspParamSchema::knob("spectral_crest_factor_bias", "High-Frequency Spectral Crest Weight", 0.0, 1.0, 0.7, "weight", MacroRole::Tone, "Bias weighting transient spikes in presence frequencies over low rumble"))
        .with_param(DspParamSchema::knob("gate_open_attack_time_ms", "Gate Response Trigger Attack Time", 0.1, 50.0, 2.0, "ms", MacroRole::Punch, "Speed at which gate passes audio when entropy indicates transient onset"))
        .with_param(DspParamSchema::knob("gate_closure_hold_time_ms", "Gate Open Hold Duration", 5.0, 500.0, 45.0, "ms", MacroRole::Space, "Minimum hold duration preventing chatter between close acoustic hits"))
        .with_param(DspParamSchema::knob("ambient_floor_attenuation_db", "Closed Gate Floor Attenuation Depth", -80.0, 0.0, -36.0, "dB", MacroRole::Character, "Max attenuation applied during silent periods when no transient is detected"))
        .with_param(DspParamSchema::toggle("enable_lookahead_transient_window", "Zero-Phase Lookahead Transient Detection", true, "Utilizes short internal lookahead to preserve earliest micro-transient wavefront"))
    );
    descriptors.insert("OptoelectronicVactrolOptocoupler".to_string(), DspNodeDescriptor::new("OptoelectronicVactrolOptocoupler", "Cadmium Sulfide (CdS) Dual-Vactrol Non-Linear Leveler", DspNodeCategory::DynamicsMaster, "Analog optical optocoupler circuit modeling LED luminescence lag, photoresistor photoconductive decay memory, and classic sluggish optical compression")
        .with_param(DspParamSchema::knob("optocoupler_drive_level_db", "Optical LED Driver Excitation Level", -30.0, 20.0, 0.0, "dB", MacroRole::Punch, "Current drive feeding emitter LED: higher drives induce deep compression"))
        .with_param(DspParamSchema::knob("photoresistor_dark_resistance_mohm", "Photoresistor Dark Resistance", 0.5, 50.0, 10.0, "MOhm", MacroRole::Tone, "Maximum unilluminated resistance determining release plateau characteristics"))
        .with_param(DspParamSchema::knob("light_history_memory_decay_s", "Photoconductive Memory Decay Lag", 0.05, 5.0, 0.8, "s", MacroRole::Space, "Memory lag constant slowing release speed following loud, sustained audio"))
        .with_param(DspParamSchema::knob("optical_compression_ratio", "Optocoupler Compression Slope", 1.5, 20.0, 4.0, ":1", MacroRole::Punch, "Non-linear resistance curve steepness across light excitation intensity"))
        .with_param(DspParamSchema::knob("vintage_vactrol_cell_type", "Vintage Vactrol Cell Selection", 0.0, 3.0, 0.0, "cell", MacroRole::Character, "Selects authentic response curves of VTL5C1, VTL5C3, VTL5C4, or modern LED-LDR"))
        .with_param(DspParamSchema::toggle("enable_parasitic_capacitance_lag", "Parasitic Junction Capacitance Filter", true, "Emulates high-frequency roll-off inherent to cadmium sulfide semiconductor layers"))
    );
    descriptors.insert("EbuLoudnessAutoTargetLeveler".to_string(), DspNodeDescriptor::new("EbuLoudnessAutoTargetLeveler", "EBU R128 / ITU-R BS.1770 Autonomous Broadcast Leveler", DspNodeCategory::DynamicsMaster, "Precision multi-stage broadcast loudness leveler automatically normalizing integrated, short-term, and momentary LUFS with zero transient pumping")
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Integrated Broadcast LUFS", -24.0, -9.0, -14.0, "LUFS", MacroRole::Punch, "Target integrated loudness for broadcast, streaming, or CD mastering standards"))
        .with_param(DspParamSchema::knob("loudness_range_lra_max_lu", "Maximum Permitted Loudness Range (LRA)", 2.0, 18.0, 7.0, "LU", MacroRole::Character, "Constrains program dynamic range (LRA) to prevent extreme jumps between sections"))
        .with_param(DspParamSchema::knob("momentary_lookahead_time_ms", "Momentary Loudness Lookahead Window", 5.0, 250.0, 50.0, "ms", MacroRole::Space, "Lookahead detector window capturing fast dialogue bursts and sudden loud hits"))
        .with_param(DspParamSchema::knob("maximum_recovery_rate_db_s", "Slow Automatic Gain Recovery Rate", 0.1, 5.0, 0.8, "dB/s", MacroRole::Punch, "Slow transparent ballistics speed adjusting long-term loudness without audible pumping"))
        .with_param(DspParamSchema::knob("gating_relative_threshold_lu", "ITU-R BS.1770 Gating Silence Threshold", -20.0, -8.0, -10.0, "LU", MacroRole::Tone, "Relative silence threshold freezing gain integration during quiet musical pauses"))
        .with_param(DspParamSchema::toggle("enable_k_weighting_psychoacoustic_filter", "Pre-Filter K-Weighting High-Shelf & HPF", true, "Complies with standard K-weighting pre-filter curve modeling human ear loudness response"))
    );
    descriptors.insert("DynamicSpectralDeEsser".to_string(), DspNodeDescriptor::new("DynamicSpectralDeEsser", "Phase-Neutral Spectral Sibilance Suppressor & Split-Band De-Esser", DspNodeCategory::DynamicsMaster, "Surgically identifies vocal sibilance, harsh cymbals, and digital bite across dynamic high-frequency bands using linear-phase splitting and psychoacoustic transient detection")
        .with_param(DspParamSchema::knob("deesser_frequency_crossover_hz", "Sibilance Band High Crossover", 2000.0, 14000.0, 6200.0, "Hz", MacroRole::Tone, "Crossover frequency separating sibilant highs from natural vocal body"))
        .with_param(DspParamSchema::knob("sibilance_reduction_threshold_db", "Sibilance Dynamic Detection Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Threshold above which sibilant energy triggers dynamic attenuation"))
        .with_param(DspParamSchema::knob("attenuation_reduction_depth_db", "Maximum Sibilant Suppression Depth", 0.0, 24.0, 8.5, "dB", MacroRole::Punch, "Maximum decibels of sibilant harshness suppression"))
        .with_param(DspParamSchema::knob("spectral_listen_monitoring_mode", "Sidechain Sibilance Band Audition Mode", 0.0, 1.0, 0.0, "mode", MacroRole::Character, "Solos isolated sidechain sibilance band for surgical tuning"))
        .with_param(DspParamSchema::knob("adaptive_release_recovery_speed_ms", "Adaptive Ballistics Recovery Release", 5.0, 150.0, 25.0, "ms", MacroRole::Space, "Fast release ballistics preventing unnatural vocal dullness"))
        .with_param(DspParamSchema::toggle("enable_split_band_dynamic_ducking", "Split-Band Selective Frequency Ducking", true, "Ducks only the sibilant high frequencies while leaving the broadband vocal untouched"))
    );
    descriptors.insert("AdaptiveNoiseGateTransientExpander".to_string(), DspNodeDescriptor::new("AdaptiveNoiseGateTransientExpander", "Adaptive Noise Gate & Transient Downward Expander", DspNodeCategory::DynamicsMaster, "Multi-stage optical-style noise gate and downward expander with psychoacoustic frequency weighting, hysteresis hold, and lookahead attack for transient preservation")
        .with_param(DspParamSchema::knob("gate_threshold_db", "Downward Expander Detection Threshold", -80.0, -10.0, -42.0, "dB", MacroRole::Punch, "Signal level below which attenuation and expansion begin"))
        .with_param(DspParamSchema::knob("expansion_ratio", "Dynamic Downward Expansion Ratio", 1.0, 20.0, 4.0, ":1", MacroRole::Punch, "Downwards expansion slope steepness applied to signals below threshold"))
        .with_param(DspParamSchema::knob("hysteresis_headroom_db", "Chatter Suppression Hysteresis Margin", 0.0, 12.0, 3.5, "dB", MacroRole::Character, "Prevents rapid chatter by requiring signal to exceed threshold plus hysteresis"))
        .with_param(DspParamSchema::knob("gate_lookahead_ms", "Optical Transient Lookahead Buffer", 0.0, 20.0, 2.5, "ms", MacroRole::Tone, "Lookahead delay buffer allowing instant transient preservation"))
        .with_param(DspParamSchema::knob("release_envelope_decay_ms", "Smooth Envelope Decay Release Speed", 5.0, 1000.0, 80.0, "ms", MacroRole::Space, "Decay time for gate closure smoothing tail transitions"))
        .with_param(DspParamSchema::toggle("enable_sidechain_bandpass_filter", "Critical Band Vocal Sidechain Isolation", true, "Restricts detection circuit to critical frequency band (500 Hz - 4 kHz)"))
    );
    descriptors.insert("MasteringTruePeakLimiterOversampled".to_string(), DspNodeDescriptor::new("MasteringTruePeakLimiterOversampled", "8x Oversampled Linear-Phase True Peak Brickwall Limiter", DspNodeCategory::DynamicsMaster, "Complies with ITU-R BS.1770-4 with 8x polyphase oversampling, inter-sample peak detection, lookahead transient safety ceiling, and TPDF dither")
        .with_param(DspParamSchema::knob("ceiling_threshold_dbfs", "True-Peak Maximum Output Ceiling", -12.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Hard maximum output ceiling preventing DAC inter-sample clipping"))
        .with_param(DspParamSchema::knob("input_makeup_boost_gain_db", "Loudness Maximizer Makeup Boost", 0.0, 24.0, 4.0, "dB", MacroRole::Punch, "Input gain driving mix into limiter ceiling for competitive loudness"))
        .with_param(DspParamSchema::knob("limiter_lookahead_time_ms", "Inter-Sample Lookahead Window", 0.5, 15.0, 4.0, "ms", MacroRole::Tone, "Pre-delay buffer allowing zero-overshoot gain trajectory calculation"))
        .with_param(DspParamSchema::knob("adaptive_release_time_ms", "Program-Dependent Adaptive Release", 10.0, 500.0, 65.0, "ms", MacroRole::Space, "Program-dependent release ballistics avoiding pumping on heavy bass"))
        .with_param(DspParamSchema::knob("intersample_peak_headroom_margin_db", "Codec Streaming ISP Safety Margin", 0.05, 1.0, 0.2, "dB", MacroRole::Character, "Safety buffer margin below 0 dBFS protecting streaming codecs"))
        .with_param(DspParamSchema::toggle("enable_tpdf_triangular_dither", "Triangular Probability Density Dither", true, "Adds 24-bit/16-bit TPDF dither with psychoacoustic noise shaping"))
    );
    descriptors.insert("LookaheadDynamicDeEsserSibilance".to_string(), DspNodeDescriptor::new("LookaheadDynamicDeEsserSibilance", "Lookahead Dynamic Spectral De-Esser", DspNodeCategory::DynamicsMaster, "Precision vocal sibilance processor featuring zero-latency lookahead envelope tracking, split-band or wideband attenuation, and adaptive threshold calibration")
        .with_param(DspParamSchema::knob("sibilance_center_frequency_hz", "Sibilance Detection Center Frequency", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Center tuning frequency isolating vocal 'S', 'T', and 'Ch' sounds"))
        .with_param(DspParamSchema::knob("deesser_threshold_dbfs", "Dynamic Attenuation Engagement Threshold", -40.0, 0.0, -18.0, "dBFS", MacroRole::Punch, "Detection threshold activating dynamic sibilance attenuation"))
        .with_param(DspParamSchema::knob("sibilance_reduction_ratio", "Downward Compression Ratio Slope", 1.5, 10.0, 4.0, ":1", MacroRole::Punch, "Compression slope applied specifically to detected sibilant energy"))
        .with_param(DspParamSchema::knob("lookahead_buffer_time_ms", "Transient Lookahead Anticipation Time", 0.5, 10.0, 2.5, "ms", MacroRole::Tone, "Pre-delay lookahead buffer ensuring transparent attack clamping"))
        .with_param(DspParamSchema::knob("sibilance_recovery_release_ms", "Adaptive Sibilance Gain Recovery Time", 5.0, 150.0, 35.0, "ms", MacroRole::Space, "Release recovery time restoring vocal presence after harsh sibilance"))
        .with_param(DspParamSchema::toggle("enable_audition_sidechain_monitor", "Solo Sibilance Detection Filter Path", false, "Solos isolated high-frequency sidechain signal for exact tuning"))
    );
    descriptors.insert("MultibandDownwardSpillExpander".to_string(), DspNodeDescriptor::new("MultibandDownwardSpillExpander", "Multiband Downward Drum & Stage Spill Expander", DspNodeCategory::DynamicsMaster, "3-band surgical downward expander attenuating headphone bleed, cymbal spill, and guitar amplifier background noise between musical phrases with gentle ratio slopes")
        .with_param(DspParamSchema::knob("low_band_expansion_threshold_db", "Low Band Bleed Threshold", -60.0, -10.0, -36.0, "dB", MacroRole::Punch, "Expansion engagement threshold for low frequency band"))
        .with_param(DspParamSchema::knob("mid_band_expansion_threshold_db", "Mid Band Bleed Threshold", -60.0, -10.0, -32.0, "dB", MacroRole::Tone, "Expansion engagement threshold for mid vocal/instrument band"))
        .with_param(DspParamSchema::knob("high_band_expansion_threshold_db", "High Band Bleed Threshold", -60.0, -10.0, -28.0, "dB", MacroRole::Tone, "Expansion engagement threshold for high cymbal bleed band"))
        .with_param(DspParamSchema::knob("expansion_ratio_attenuation_slope", "Downward Expansion Attenuation Ratio", 1.2, 8.0, 2.5, ":1", MacroRole::Character, "Gentle expansion slope avoiding unnatural noise-gate gating chatter"))
        .with_param(DspParamSchema::knob("release_envelope_hysteresis_ms", "Hold & Release Smoothing Envelope", 10.0, 500.0, 60.0, "ms", MacroRole::Space, "Smooth release envelope avoiding musical tail truncation"))
        .with_param(DspParamSchema::toggle("enable_sidechain_frequency_weighting", "Vocal/Snare Sidechain Spectral Weighting", true, "Weights sidechain detection prioritizing intended musical attacks"))
    );
    descriptors.insert("DynamicSpectralCompressorDeEsser".to_string(), DspNodeDescriptor::new("DynamicSpectralCompressorDeEsser", "Dynamic Multi-Band Spectral De-Esser", DspNodeCategory::DynamicsMaster, "High-precision split-band dynamic sibilance tamer with spectral envelope detection, psychoacoustic threshold weighting, and lookahead attack")
        .with_param(DspParamSchema::knob("sibilance_center_frequency_hz", "Sibilance Target Center Frequency", 3000.0, 14000.0, 6800.0, "Hz", MacroRole::Tone, "Center of harsh sibilant friction frequency band"))
        .with_param(DspParamSchema::knob("sibilance_bandwidth_octaves", "Sibilance Detection Bandwidth", 0.5, 3.0, 1.2, "oct", MacroRole::Character, "Width of sibilant detection filter in octaves"))
        .with_param(DspParamSchema::knob("dynamic_reduction_threshold_db", "Dynamic Suppression Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Threshold above which sibilant reduction activates"))
        .with_param(DspParamSchema::knob("compression_ratio", "Gain Reduction Compression Ratio", 1.0, 20.0, 4.5, ":1", MacroRole::Tone, "Ratio of attenuation applied to excessive sibilant energy"))
        .with_param(DspParamSchema::knob("detector_lookahead_ms", "Lookahead Transient Buffer Delay", 0.0, 10.0, 2.0, "ms", MacroRole::Space, "Eliminates transient overshoot on explosive consonantal plosives"))
        .with_param(DspParamSchema::toggle("enable_split_band_solo", "Solo Sibilance Sidechain Band", false, "Solos filtered sibilant detector signal for precise band alignment"))
    );
    descriptors.insert("MultibandTruePeakMasterLimiter".to_string(), DspNodeDescriptor::new("MultibandTruePeakMasterLimiter", "Multiband Inter-Sample True-Peak Limiter", DspNodeCategory::DynamicsMaster, "4-band oversampled mastering limiter with ITU-R BS.1770 compliant inter-sample true-peak estimation, dynamic crest factor preservation, and psychoacoustic transient release")
        .with_param(DspParamSchema::knob("master_ceiling_level_dbfs", "Limiter True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Absolute brickwall ceiling guarding against inter-sample DAC clipping"))
        .with_param(DspParamSchema::knob("master_pre_gain_drive_db", "Master Input Threshold Pre-Gain", -6.0, 24.0, 3.5, "dB", MacroRole::Tone, "Pre-limiter input gain maximizing loudness and density"))
        .with_param(DspParamSchema::knob("release_recovery_time_ms", "Transient Envelope Release Recovery", 5.0, 500.0, 45.0, "ms", MacroRole::Character, "Dynamic release time tracking audio crest factor for zero pumping"))
        .with_param(DspParamSchema::knob("low_band_saturation_guard_db", "Sub-Bass Limiting Soft-Clip Saturation", 0.0, 6.0, 1.5, "dB", MacroRole::Punch, "Prevents sub-bass distortion by soft-clipping bass peaks prior to limiter"))
        .with_param(DspParamSchema::knob("oversampling_order_factor", "Linear Phase Oversampling Rate (2x/4x/8x)", 2.0, 8.0, 4.0, "x", MacroRole::Space, "Upsampling multiplier for inter-sample peak detection accuracy"))
        .with_param(DspParamSchema::toggle("enable_dither_noise_shaping", "TPDF Dither & Psychoacoustic Noise Shaping", true, "Applies high-frequency psychoacoustic triangular dither to 24-bit/16-bit output"))
    );
    descriptors.insert("NonlinearOptocouplerOpticalCompressor".to_string(), DspNodeDescriptor::new("NonlinearOptocouplerOpticalCompressor", "Electro-Optical Optocoupler Program-Dependent Compressor", DspNodeCategory::DynamicsMaster, "Physical emulation of T4B electro-optical attenuator cells with dual-decay memory curves, photoconductive cadmium sulfide luminescence lag, and non-linear knee compression")
        .with_param(DspParamSchema::knob("peak_reduction_gain_db", "Optical Cell Peak Reduction", -60.0, 0.0, -18.0, "dB", MacroRole::Punch, "Drives audio signal into photoconductive attenuator cell"))
        .with_param(DspParamSchema::knob("luminescent_primary_release_ms", "Electroluminescent Primary Decay", 10.0, 500.0, 60.0, "ms", MacroRole::Tone, "Initial rapid release time constant of optical phosphor glow"))
        .with_param(DspParamSchema::knob("photocell_memory_lag_seconds", "CdS Photoconductive Memory Secondary Lag", 0.1, 10.0, 1.5, "sec", MacroRole::Space, "Long secondary decay memory tail characteristic of heavily compressed audio"))
        .with_param(DspParamSchema::knob("frequency_weighted_pre_emphasis_db", "HF Sidechain Pre-Emphasis Boost", 0.0, 12.0, 3.5, "dB", MacroRole::Character, "High-frequency sidechain emphasis preventing low-end pumping on vocals"))
        .with_param(DspParamSchema::knob("makeup_amplification_gain_db", "Tube Output Makeup Stage Gain", 0.0, 40.0, 12.0, "dB", MacroRole::Punch, "Post-attenuation 12AX7 makeup amplifier stage level"))
        .with_param(DspParamSchema::toggle("enable_program_dependent_dual_knees", "Historical Program-Dependent Dual Knee", true, "Dynamically widens compression knee based on incoming peak density"))
    );
    descriptors.insert("MultibandMidSideMasteringCompressor".to_string(), DspNodeDescriptor::new("MultibandMidSideMasteringCompressor", "Tri-Band Mid-Side Precision Mastering Compressor", DspNodeCategory::DynamicsMaster, "Linear-phase 3-band mastering dynamic processor with independent mid (center) and side (width) compression ratios, program-dependent timing, and stereo image widening")
        .with_param(DspParamSchema::knob("low_mid_crossover_frequency_hz", "Low-to-Mid Crossover Frequency", 60.0, 400.0, 140.0, "Hz", MacroRole::Tone, "Linear-phase crossover frequency separating low end from midrange"))
        .with_param(DspParamSchema::knob("mid_high_crossover_frequency_hz", "Mid-to-High Crossover Frequency", 1500.0, 8000.0, 3200.0, "Hz", MacroRole::Tone, "Linear-phase crossover frequency separating midrange from high frequencies"))
        .with_param(DspParamSchema::knob("mid_channel_compression_ratio", "Mid Center Punch Compression Ratio", 1.0, 10.0, 2.2, ":1", MacroRole::Punch, "Compression ratio focused on center mono energy (kick, snare, lead vocals)"))
        .with_param(DspParamSchema::knob("side_channel_compression_ratio", "Side Ambience Compression Ratio", 1.0, 10.0, 1.8, ":1", MacroRole::Space, "Compression ratio controlling wide stereo reverb, guitars, and room mics"))
        .with_param(DspParamSchema::knob("stereo_side_channel_gain_db", "Stereo Spatial Width Makeup Gain", -12.0, 6.0, 1.2, "dB", MacroRole::Character, "Direct gain boost on side channels for expansive stereo image control"))
        .with_param(DspParamSchema::toggle("enable_linear_phase_crossover_filter", "FIR Linear-Phase Crossover Lattice", true, "Eliminates phase smearing at crossover points preserving pristine transient impact"))
    );
    descriptors.insert("MultibandTransientKneeExpander".to_string(), DspNodeDescriptor::new("MultibandTransientKneeExpander", "Quad-Band Downward Spill & Transient Expander", DspNodeCategory::DynamicsMaster, "Mastering-grade 4-band downward expander with transparent linear-phase crossovers, soft-knee threshold curves, and per-band transient punch recovery")
        .with_param(DspParamSchema::knob("expansion_ratio_slope", "Downward Expansion Ratio", 1.1, 8.0, 2.5, ":1", MacroRole::Punch, "Slope of dynamic attenuation applied to signals below expansion threshold"))
        .with_param(DspParamSchema::knob("downward_threshold_db", "Global Expansion Floor Threshold", -60.0, -10.0, -32.0, "dB", MacroRole::Tone, "Signal level below which downward attenuation activates to clean acoustic spill"))
        .with_param(DspParamSchema::knob("quad_crossover_low_hz", "Low-to-Mid Crossover Split Frequency", 60.0, 500.0, 160.0, "Hz", MacroRole::Character, "Linear-phase split separating low-end rumble from mid-range percussion"))
        .with_param(DspParamSchema::knob("quad_crossover_high_hz", "Mid-to-High Crossover Split Frequency", 1500.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Split point isolating high-frequency cymbal bleed and room hiss"))
        .with_param(DspParamSchema::knob("envelope_hold_duration_ms", "Transient Peak Hold Duration", 0.0, 100.0, 15.0, "ms", MacroRole::Space, "Prevents chattering and pumping on fast transient drum fills"))
        .with_param(DspParamSchema::toggle("enable_transient_lookahead_gate", "Zero-Distortion Lookahead Detection", true, "Applies 2ms lookahead buffer allowing completely clickless expansion envelope onset"))
    );
    descriptors.insert("DualStageOpticalVactrolLimiter".to_string(), DspNodeDescriptor::new("DualStageOpticalVactrolLimiter", "Dual-Cell Vactrol Optoisolator Fast Peak Limiter", DspNodeCategory::DynamicsMaster, "Dual-cell resistive optoisolator (vactrol) peak limiter reproducing cadmium sulfide photoresistor turn-on speed, sluggish multi-second decay tails, and tube makeup warmth")
        .with_param(DspParamSchema::knob("vactrol_led_threshold_db", "Vactrol LED Conduction Threshold", -40.0, 0.0, -14.0, "dB", MacroRole::Punch, "Signal level that illuminates internal gallium arsenide emitter LED"))
        .with_param(DspParamSchema::knob("primary_peak_limiting_ratio", "Optical Compression Ratio", 2.0, 40.0, 10.0, ":1", MacroRole::Tone, "Limiting ratio steepness governing peak compression strength"))
        .with_param(DspParamSchema::knob("vactrol_photoresistor_memory_ms", "CdS Photoconductive Memory Decay", 50.0, 5000.0, 850.0, "ms", MacroRole::Space, "Sluggish photoresistor recovery time producing smooth, pumping-free release"))
        .with_param(DspParamSchema::knob("analog_tube_warmth_drive_db", "Post-Limiter Tube Makeup Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Character, "Triode makeup amplification stage adding harmonic richness to peak-limited audio"))
        .with_param(DspParamSchema::knob("output_ceiling_brickwall_db", "Absolute Output Safety Ceiling", -12.0, 0.0, -0.5, "dB", MacroRole::Punch, "Rigid output ceiling protecting digital converters from inter-sample clipping"))
        .with_param(DspParamSchema::toggle("enable_fast_transient_led_shunting", "Ultra-Fast Shunt Diode Peak Protection", true, "Engages secondary fast LED shunt for zero-overshoot percussive snare catching"))
    );
    descriptors.insert("MultibandUpwardDownwardDynamicExpander".to_string(), DspNodeDescriptor::new("MultibandUpwardDownwardDynamicExpander", "Dual-Direction Multiband Dynamic Expander", DspNodeCategory::DynamicsMaster, "4-band mastering dynamics processor offering simultaneous upward expansion for transient punch and downward gating for noise floor suppression with linear-phase crossover")
        .with_param(DspParamSchema::knob("upward_expansion_ratio", "Upward Dynamic Expansion Ratio", 1.0, 4.0, 1.6, ":1", MacroRole::Punch, "Ratio of dynamic amplification applied to loud peaks exceeding upper threshold"))
        .with_param(DspParamSchema::knob("upward_expansion_threshold_db", "Upward Peak Threshold Level", -24.0, 0.0, -10.0, "dB", MacroRole::Tone, "Threshold above which percussive transients receive dynamic punch enhancement"))
        .with_param(DspParamSchema::knob("downward_gate_threshold_db", "Downward Noise Floor Threshold", -70.0, -20.0, -42.0, "dB", MacroRole::Space, "Threshold below which room noise and bleed are smoothly attenuated"))
        .with_param(DspParamSchema::knob("crossover_split_frequency_hz", "Mid-Band Boundary Crossover Frequency", 200.0, 5000.0, 1200.0, "Hz", MacroRole::Character, "Linear-phase split point separating body from presence bands"))
        .with_param(DspParamSchema::knob("hysteresis_chatter_protection_db", "Dynamic Gate Hysteresis Window", 0.5, 12.0, 3.0, "dB", MacroRole::Tone, "Prevents threshold chatter by holding downward expansion open"))
        .with_param(DspParamSchema::toggle("enable_sidechain_rms_detection", "True RMS Energy Detector Smoothing", true, "Uses root-mean-square level detection for smooth musical gain changes"))
    );
    descriptors.insert("MasterStereoVariMuLimiter".to_string(), DspNodeDescriptor::new("MasterStereoVariMuLimiter", "Stereo Variable-Mu Remote-Cutoff Tube Limiter", DspNodeCategory::DynamicsMaster, "Mastering-grade Variable-Mu compressor/limiter modeling remote-cutoff triode/pentode tube grid biasing, program-dependent attack/release auto-recovery, and sidechain HPF")
        .with_param(DspParamSchema::knob("threshold_input_drive_db", "Vari-Mu Tube Grid Threshold Drive", -30.0, 20.0, 0.0, "dB", MacroRole::Punch, "Input level driving signal onto non-linear remote-cutoff tube grid curve"))
        .with_param(DspParamSchema::knob("compression_ratio_slope", "Progressive Compression Ratio", 1.5, 20.0, 4.0, ":1", MacroRole::Tone, "Smoothly steepening ratio characteristic of variable-mu valve circuits"))
        .with_param(DspParamSchema::knob("attack_time_constant_ms", "Limiter Attack Response Speed", 0.1, 50.0, 12.0, "ms", MacroRole::Punch, "Speed at which tube grid bias voltage responds to transient surges"))
        .with_param(DspParamSchema::knob("release_time_constant_ms", "Program-Dependent Release Recovery", 50.0, 2500.0, 350.0, "ms", MacroRole::Space, "Base release time constant with multi-stage RC network recovery"))
        .with_param(DspParamSchema::knob("sidechain_highpass_filter_hz", "Sidechain Bass Isolation Highpass", 20.0, 300.0, 90.0, "Hz", MacroRole::Character, "Prevents heavy bass frequencies from pumping the master stereo bus"))
        .with_param(DspParamSchema::toggle("enable_dual_mono_unlinked_mode", "Unlinked Stereo Channel Detection", false, "Operates left and right channel sidechains independently to preserve maximum width"))
    );
    descriptors.insert("ParallelMultibandTransientMasterLimiter".to_string(), DspNodeDescriptor::new("ParallelMultibandTransientMasterLimiter", "Parallel 5-Band Transient Clamping Master Limiter", DspNodeCategory::DynamicsMaster, "Mastering brickwall limiter with 5-band Linkwitz-Riley splitting, inter-sample true peak detection, zero lookahead latency transient clamping, and soft knee saturation")
        .with_param(DspParamSchema::knob("master_ceiling_margin_db", "True Peak Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Absolute ceiling margin preventing digital overshoots during DAC reconstruction"))
        .with_param(DspParamSchema::knob("transient_clip_threshold_db", "Inter-Band Transient Clip Threshold", -12.0, 0.0, -1.5, "dB", MacroRole::Tone, "Fast clipping threshold shaving off sharp micro-transients before limiting"))
        .with_param(DspParamSchema::knob("multiband_release_ballistics_ms", "Program-Adaptive Release Time", 5.0, 500.0, 45.0, "ms", MacroRole::Character, "Dynamic release time adjusting automatically based on crest factor"))
        .with_param(DspParamSchema::knob("low_band_crest_factor_clamp", "Sub-Bass Crest Factor Density", 0.5, 3.0, 1.2, "crest", MacroRole::Punch, "Independent limiter aggression on sub-bass band below 90 Hz"))
        .with_param(DspParamSchema::knob("stereo_link_coupling_percent", "Stereo Gain Reduction Linking", 0.0, 100.0, 85.0, "%", MacroRole::Space, "Inter-channel gain reduction linking percentage preserving stereo imaging"))
        .with_param(DspParamSchema::toggle("enable_intersample_oversampling", "8x True-Peak Polyphase Oversampling", true, "Enables 8x polyphase true-peak intersample oversampling engine"))
    );
    descriptors.insert("DualDiodeBridgeCompressorLimiter".to_string(), DspNodeDescriptor::new("DualDiodeBridgeCompressorLimiter", "Discrete Diode Bridge Vintage Dynamic Limiter", DspNodeCategory::DynamicsMaster, "Discrete British diode bridge compressor emulation featuring nonlinear dynamic attenuation curve, fast attack characteristics, diode thermal matching, and colored vintage tone")
        .with_param(DspParamSchema::knob("compressor_threshold_level_db", "Diode Bridge Gain Reduction Threshold", -40.0, 10.0, -14.0, "dB", MacroRole::Punch, "Input level threshold above which the diode bridge attenuates signal"))
        .with_param(DspParamSchema::knob("compression_ratio_setting", "Compression Slope Ratio Factor", 1.5, 20.0, 4.0, ":1", MacroRole::Character, "Dynamic compression ratio steepness above threshold"))
        .with_param(DspParamSchema::knob("compressor_attack_speed_us", "Ultra-Fast Transient Attack Time", 100.0, 10000.0, 1200.0, "us", MacroRole::Tone, "Diode bridge response speed to sharp transient peaks"))
        .with_param(DspParamSchema::knob("compressor_auto_recovery_release_ms", "Dynamic Release Recovery Time", 50.0, 1500.0, 200.0, "ms", MacroRole::Space, "Release recovery time restoring diode bridge attenuation to unity"))
        .with_param(DspParamSchema::knob("makeup_gain_level_db", "Post-Attenuation Output Makeup Gain", 0.0, 30.0, 8.0, "dB", MacroRole::Punch, "Output amplifier makeup gain compensating for compression loss"))
        .with_param(DspParamSchema::toggle("enable_sidechain_highpass_filter", "120 Hz Sidechain Highpass Filter", true, "Inserts 120 Hz 6dB/oct sidechain highpass filter to prevent bass pumping"))
    );
    descriptors.insert("MultibandMasteringLinearPhaseLimiter".to_string(), DspNodeDescriptor::new("MultibandMasteringLinearPhaseLimiter", "Mastering 5-Band Linear-Phase Peak Limiter", DspNodeCategory::DynamicsMaster, "Mastering-grade 5-band linear phase brickwall peak limiter with inter-sample peak (ISP) oversampled true-peak detection and adaptive multi-channel psychoacoustic release")
        .with_param(DspParamSchema::knob("master_brickwall_ceiling_dbfs", "True-Peak Output Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Maximum permissible digital output true-peak level guaranteed against inter-sample overshoots"))
        .with_param(DspParamSchema::knob("limiter_threshold_input_drive_db", "Input Drive Maximizer Threshold", -18.0, 12.0, 0.0, "dB", MacroRole::Punch, "Master input boost driving audio signal into the 5 split linear-phase limiter bands"))
        .with_param(DspParamSchema::knob("multiband_spectral_balance_weight", "Low-High Dynamic Band Linking", 0.0, 1.0, 0.65, "link", MacroRole::Tone, "Cross-band gain reduction coupling preserving tonal balance during heavy limiting"))
        .with_param(DspParamSchema::knob("adaptive_release_inertia_ms", "Adaptive Psychoacoustic Release Inertia", 10.0, 500.0, 85.0, "ms", MacroRole::Space, "Release ballistics time constant automatically speeding up for transient clicks and slowing for bass"))
        .with_param(DspParamSchema::knob("inter_sample_oversampling_factor", "ISP True-Peak Oversampling Rate", 1.0, 8.0, 4.0, "x", MacroRole::Character, "Internal oversampling multiple for precise detection of inter-sample analog reconstruction peaks"))
        .with_param(DspParamSchema::toggle("enable_auto_makeup_gain", "Auto Equal-Loudness Makeup Gain", true, "Automatically compensates master output loudness to match perceived acoustic RMS level"))
    );
    descriptors.insert("OpticalVactrolPeakMasterCompressor".to_string(), DspNodeDescriptor::new("OpticalVactrolPeakMasterCompressor", "Vintage Optical Cadmium Sulfide Vactrol Compressor", DspNodeCategory::DynamicsMaster, "Vintage optical compressor modeled after passive electroluminescent panel and dual photo-conductive cadmium sulfide vactrol cells with dual-stage memory lag")
        .with_param(DspParamSchema::knob("vactrol_peak_reduction_gain_db", "Vactrol Peak Reduction Drive", 0.0, 40.0, 16.0, "dB", MacroRole::Punch, "Input drive level exciting the electroluminescent light cell against photo-resistors"))
        .with_param(DspParamSchema::knob("compression_ratio_slope", "Vactrol Dynamic Attenuation Ratio", 1.5, 20.0, 4.0, ":1", MacroRole::Punch, "Dynamic ratio curvature smoothly steepening with deeper gain reduction"))
        .with_param(DspParamSchema::knob("cds_photoresistor_memory_decay_ms", "CdS Photoresistor Primary Decay Memory", 20.0, 1500.0, 320.0, "ms", MacroRole::Space, "Primary photo-conductive decay release time of the cadmium sulfide crystal lattice"))
        .with_param(DspParamSchema::knob("secondary_slow_tail_lag_ms", "Secondary Deep Memory Tail Decay Lag", 100.0, 5000.0, 1800.0, "ms", MacroRole::Space, "Secondary long release tail triggered after sustained heavy compression"))
        .with_param(DspParamSchema::knob("high_frequency_emphasis_sidechain", "High-Frequency Sidechain Pre-Emphasis", 0.0, 1.0, 0.45, "emph", MacroRole::Tone, "Boosts high frequencies in sidechain to prevent high-energy bass from choking high frequencies"))
        .with_param(DspParamSchema::toggle("enable_compressor_limiter_mode_switch", "Limiter vs Compressor Ratio Profile", false, "Switches from gentle 4:1 musical leveling curve to steep >15:1 peak brickwall limiting"))
    );
    descriptors.insert("VariableMuDualSidechainMasterCompressor".to_string(), DspNodeDescriptor::new("VariableMuDualSidechainMasterCompressor", "Mastering Variable-Mu Remote-Cutoff Tube Compressor", DspNodeCategory::DynamicsMaster, "Mastering-grade vintage Variable-Mu tube leveling amplifier modeling remote-cutoff triode grid bias curvature, dual stereo sidechains, and program-dependent release")
        .with_param(DspParamSchema::knob("compressor_threshold_drive_db", "Input Drive Level Against Tube Threshold", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Input gain driving program audio into the curved non-linear tube grid conduction zone"))
        .with_param(DspParamSchema::knob("tube_variable_mu_ratio_slope", "Variable-Mu Smooth Compression Ratio Curvature", 1.2, 8.0, 2.5, ":1", MacroRole::Punch, "Dynamic ratio curvature smoothly increasing from gentle 1.5:1 up to 8:1 on heavy peaks"))
        .with_param(DspParamSchema::knob("program_dependent_attack_ms", "Fast Transient Ballistic Attack Time", 0.5, 100.0, 12.0, "ms", MacroRole::Punch, "Initial charging speed of grid capacitance responding to fast audio transience"))
        .with_param(DspParamSchema::knob("dual_stage_release_recovery_ms", "Dual-Stage Slow Program Release Recovery", 50.0, 2500.0, 380.0, "ms", MacroRole::Space, "Secondary slow electrolytic capacitor discharge time preventing dynamic pumping"))
        .with_param(DspParamSchema::knob("sidechain_highpass_shelf_hz", "Sidechain Bass Attenuation High-Pass Shelf", 20.0, 300.0, 90.0, "Hz", MacroRole::Tone, "Highpass filter in detection circuit preventing low-end energy from choking stereo program"))
        .with_param(DspParamSchema::toggle("enable_mid_side_unlinked_detection", "Mid/Side Dual Unlinked Sidechain Detection", true, "Operates compression independently on Mid and Side channels for enhanced stereo width stability"))
    );
    descriptors.insert("DynamicTransientMultiBandPeakClipper".to_string(), DspNodeDescriptor::new("DynamicTransientMultiBandPeakClipper", "Linear-Phase 4-Band Dynamic Mastering Peak Clipper", DspNodeCategory::DynamicsMaster, "Ultra-transparent 4-band mastering soft clipper utilizing linear-phase FIR crossover filters, dynamic psychoacoustic thresholds, and 16x oversampled ISP suppression")
        .with_param(DspParamSchema::knob("master_ceiling_threshold_dbfs", "Master True-Peak Ceiling Clip Threshold", -6.0, 0.0, -0.2, "dBFS", MacroRole::Punch, "Absolute digital brickwall ceiling above which waveshaping saturation is applied"))
        .with_param(DspParamSchema::knob("low_band_clip_knee_softness", "Sub-Bass Soft Saturation Knee Rounding", 0.01, 1.0, 0.45, "knee", MacroRole::Tone, "Curvature of low-end polynomial clipping knee preventing harsh bass intermodulation"))
        .with_param(DspParamSchema::knob("mid_band_transient_punch_boost", "Mid-Band Transient Snap Punch Preservation", 0.0, 6.0, 1.8, "dB", MacroRole::Punch, "Pre-emphasis punch applied to snare and percussion attack before soft clipping"))
        .with_param(DspParamSchema::knob("high_band_spectral_crest_protection", "High-Frequency Splash Harmonic Limiting", 0.0, 1.0, 0.72, "prot", MacroRole::Tone, "Suppresses high-frequency clipping harmonic spray above 10 kHz preserving cymbal air"))
        .with_param(DspParamSchema::knob("linear_phase_crossover_steepness_db", "Linear-Phase FIR Crossover Filter Slope", 12.0, 96.0, 48.0, "dB/oct", MacroRole::Character, "Steepness of 4-band split crossover filters with zero group-delay phase distortion"))
        .with_param(DspParamSchema::toggle("enable_sixteen_times_isp_oversampling", "16x Polyphase Inter-Sample Peak Oversampling", true, "Runs internal clipping core at 16x oversampling with minimum-phase decimation filters"))
    );
    descriptors.insert("OpticalVactrolFeedbackMasterLimiter".to_string(), DspNodeDescriptor::new("OpticalVactrolFeedbackMasterLimiter", "Precision Optical Vactrol Program-Adaptive Master Limiter", DspNodeCategory::DynamicsMaster, "Mastering true-peak brickwall limiter utilizing analog optocoupler (vactrol) photo-resistor nonlinear ballistic decay curves with lookahead transient capture and transparent gain recovery")
        .with_param(DspParamSchema::knob("ceiling_true_peak_limit_dbfs", "Master Output True-Peak Ceiling Limit", -6.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "Absolute brickwall output ceiling preventing DAC inter-sample clipping on streaming encodes"))
        .with_param(DspParamSchema::knob("input_drive_gain_db", "Limiter Threshold Input Drive Boost", 0.0, 18.0, 4.5, "dB", MacroRole::Punch, "Input gain driving program audio into the optical gain reduction knee"))
        .with_param(DspParamSchema::knob("photocell_release_memory_inertia_ms", "Vactrol Photocell Cadmium-Sulfide Memory Inertia", 20.0, 1500.0, 140.0, "ms", MacroRole::Space, "Simulates optical LDR memory lag providing smooth release without pumping artifacts"))
        .with_param(DspParamSchema::knob("lookahead_transient_buffer_ms", "Linear-Phase Transient Lookahead Buffer Delay", 0.5, 10.0, 3.0, "ms", MacroRole::Punch, "Pre-delay buffer allowing limiter to smoothly attenuate prior to transient arrival"))
        .with_param(DspParamSchema::knob("stereo_sidechain_link_percentage", "Stereo Sidechain Detection Link Ratio", 0.0, 100.0, 80.0, "%", MacroRole::Tone, "Links L/R detection channels; lower values preserve wide stereo punch, higher values prevent image drift"))
        .with_param(DspParamSchema::toggle("enable_inter_sample_peak_clipping_guard", "4x Oversampled Inter-Sample Peak (ISP) Detector", true, "Detects hidden inter-sample reconstruction peaks before D/A filter reconstruction"))
    );
    descriptors.insert("MultibandUpwardExpansionTransientSnap".to_string(), DspNodeDescriptor::new("MultibandUpwardExpansionTransientSnap", "Linear-Phase 4-Band Dynamic Upward Expander & Transient Snapper", DspNodeCategory::DynamicsMaster, "Four-band linear-phase upward dynamic expander designed to unpack over-compressed recordings, inject life and dynamic range back into transients, and enhance micro-dynamics")
        .with_param(DspParamSchema::knob("expansion_ratio_slope", "Dynamic Upward Expansion Slope Ratio", 1.0, 3.0, 1.45, "ratio", MacroRole::Punch, "Amount of gain boost applied to signals rising above expansion threshold"))
        .with_param(DspParamSchema::knob("expansion_threshold_level_db", "Upward Expansion Triggering Threshold", -40.0, -6.0, -18.0, "dB", MacroRole::Punch, "Signal level above which the expander kicks in to boost transient peaks"))
        .with_param(DspParamSchema::knob("low_band_punch_reinforcement_db", "Low-Band Kick Drum Punch Upward Boost", 0.0, 12.0, 3.5, "dB", MacroRole::Punch, "Specific upward dynamic boost applied to sub/bass band (below 120 Hz)"))
        .with_param(DspParamSchema::knob("high_band_snare_crack_boost_db", "High-Mid Snare & Percussion Attack Snap", 0.0, 12.0, 4.0, "dB", MacroRole::Tone, "Specific upward boost applied to snare transient band (2-6 kHz)"))
        .with_param(DspParamSchema::knob("fir_crossover_linear_phase_depth", "FIR Linear Phase Crossover Filter Steepness", 12.0, 72.0, 36.0, "dB/oct", MacroRole::Tone, "Zero-phase crossover steepness keeping all 4 bands aligned with zero phase smearing"))
        .with_param(DspParamSchema::toggle("enable_envelope_crest_factor_detection", "Crest-Factor Dynamic Sniffing Mode", true, "Only expands high crest-factor impulses (drum hits) while ignoring sustained pad passages"))
    );
    descriptors.insert("ParallelFeedbackFeedforwardOptoCompressor".to_string(), DspNodeDescriptor::new("ParallelFeedbackFeedforwardOptoCompressor", "Dual Feedback / Feedforward Mastering Optical Compressor", DspNodeCategory::DynamicsMaster, "Mastering optical compressor providing blendable feedback (vintage musical program-dependent) and feedforward (modern surgical transient clamping) electro-optical sidechain circuits")
        .with_param(DspParamSchema::knob("threshold_level_db", "Master Compression Threshold Level", -36.0, 0.0, -14.0, "dB", MacroRole::Punch, "Input level above which compression gain reduction begins"))
        .with_param(DspParamSchema::knob("optical_compression_ratio", "Optical Gain Reduction Compression Ratio", 1.2, 20.0, 3.5, "ratio", MacroRole::Punch, "Slope ratio of optical compression curve above threshold"))
        .with_param(DspParamSchema::knob("feedback_feedforward_topology_blend", "Circuit Topology Blend (Feedback vs Feedforward)", 0.0, 1.0, 0.35, "blend", MacroRole::Tone, "0.0: 100% Vintage Feedback (LA-2A style), 1.0: 100% Modern Feedforward surgical"))
        .with_param(DspParamSchema::knob("luminescent_panel_attack_time_ms", "Electro-Luminescent Panel Optical Attack Time", 0.5, 50.0, 10.0, "ms", MacroRole::Punch, "Rise time of light emission driving cadmium-sulfide photocell"))
        .with_param(DspParamSchema::knob("photocell_two_stage_release_sec", "Photocell Dual-Stage Memory Release Time", 0.05, 3.0, 0.45, "s", MacroRole::Space, "Two-stage release time simulating photocell physical light decay memory"))
        .with_param(DspParamSchema::toggle("enable_highpass_sidechain_filter", "120 Hz Low-Cut Sidechain Bass Protection", true, "Filters low bass from sidechain detector preventing kick drum pumping"))
    );
    descriptors.insert("MultibandSubHarmonicPunchTransientExpander".to_string(), DspNodeDescriptor::new("MultibandSubHarmonicPunchTransientExpander", "Tri-Band Sub-Harmonic Punch & Transient Expander", DspNodeCategory::DynamicsMaster, "Specialized three-band dynamic punch processor analyzing sub-bass transient energy, synthesizing coherent sub-harmonic thump, and dynamically expanding kick drum attack fronts")
        .with_param(DspParamSchema::knob("sub_harmonic_thump_boost_db", "Sub-Harmonic Fundamental Thump Injection", 0.0, 18.0, 4.5, "dB", MacroRole::Punch, "Level of synthesized sub-octave fundamental injected into low transient hits"))
        .with_param(DspParamSchema::knob("transient_attack_crest_multiplier", "Transient Onset Crest Factor Expansion Gain", 1.0, 3.0, 1.5, "x", MacroRole::Punch, "Gain multiplier applied to initial transient strike wavefront"))
        .with_param(DspParamSchema::knob("sub_crossover_corner_freq_hz", "Sub-Bass Crossover Split Frequency", 50.0, 160.0, 85.0, "Hz", MacroRole::Tone, "Crossover boundary separating low-end thump from mid punch"))
        .with_param(DspParamSchema::knob("transient_detection_hold_time_ms", "Attack Detection Window Hold Time", 1.0, 30.0, 8.0, "ms", MacroRole::Punch, "Duration of dynamic attack boost before returning to normal program gain"))
        .with_param(DspParamSchema::knob("stereo_center_mono_clamping", "Sub-Bass Stereo Field Mono Anchor", 0.0, 1.0, 1.0, "mono", MacroRole::Tone, "Forces expanded sub frequencies into 100% center mono to protect vinyl and club systems"))
        .with_param(DspParamSchema::toggle("enable_soft_clip_headroom_safety", "Analog Diode Soft-Clip Peak Limiting Safety", true, "Clips excessive expanded peaks with warm diode saturation preventing digital overshoots"))
    );
    descriptors.insert("MultibandMasteringFeedForwardVcaCompressor".to_string(), DspNodeDescriptor::new("MultibandMasteringFeedForwardVcaCompressor", "4-Band Mastering Feed-Forward VCA Compressor", DspNodeCategory::DynamicsMaster, "4-band mastering-grade feed-forward voltage-controlled amplifier (VCA) compressor with ultra-clean logarithmic sidechains, program-dependent RMS detection, and dry/wet parallel blending")
        .with_param(DspParamSchema::knob("vca_threshold_db", "Master Compressor Threshold Level", -40.0, 0.0, -16.0, "dB", MacroRole::Punch, "Signal level initiating gain reduction across frequency bands"))
        .with_param(DspParamSchema::knob("logarithmic_vca_ratio", "Logarithmic VCA Compression Ratio", 1.2, 10.0, 2.5, "ratio", MacroRole::Punch, "Compression slope above threshold (2.0: gentle glue, 4.0: firm leveling)"))
        .with_param(DspParamSchema::knob("rms_integration_time_ms", "Sidechain RMS Integration Window", 5.0, 200.0, 35.0, "ms", MacroRole::Tone, "Time window for true power RMS detection preventing low-end pumping"))
        .with_param(DspParamSchema::knob("program_dependent_release_ms", "Program-Adaptive Release Envelope", 20.0, 1200.0, 180.0, "ms", MacroRole::Space, "Release time automatically speeding up on transients and slowing down on sustained passages"))
        .with_param(DspParamSchema::knob("parallel_dry_wet_blend", "Parallel Compression Dry/Wet Mix", 0.0, 1.0, 0.4, "mix", MacroRole::Character, "Blends uncompressed original with squashed signal for New York compression punch"))
        .with_param(DspParamSchema::toggle("enable_sidechain_highpass_tilt", "6 dB/oct Sidechain Low-Cut Tilt Filter", true, "Rolls off sub frequencies in detection path preventing kick drum from triggering false pumping"))
    );
    descriptors.insert("MidSidePsychoacousticPunchLimiter".to_string(), DspNodeDescriptor::new("MidSidePsychoacousticPunchLimiter", "Mid/Side Psychoacoustic Brickwall Punch Limiter", DspNodeCategory::DynamicsMaster, "Mastering true-peak brickwall limiter with Mid/Side independent transient contouring, psychoacoustic lookahead overshoot suppression, and zero transient smearing")
        .with_param(DspParamSchema::knob("ceiling_true_peak_db", "Master True-Peak Brickwall Ceiling", -3.0, -0.1, -0.8, "dB", MacroRole::Punch, "Maximum permissible peak level guaranteeing zero inter-sample clipping on streaming services"))
        .with_param(DspParamSchema::knob("mid_channel_punch_drive_db", "Mid Channel Center Punch Input Drive", 0.0, 18.0, 4.5, "dB", MacroRole::Punch, "Drives center content (kick, snare, lead vocals) for maximum frontal energy"))
        .with_param(DspParamSchema::knob("side_channel_air_gain_db", "Side Channel Stereo Air Gain Offset", -6.0, 6.0, 0.5, "dB", MacroRole::Space, "Adjusts relative gain of side stereo field to maintain wide panoramic image"))
        .with_param(DspParamSchema::knob("lookahead_buffer_time_ms", "Anticipatory Lookahead Window", 0.5, 10.0, 3.5, "ms", MacroRole::Tone, "Lookahead delay allowing gain reduction to smoothly ramp down before transient wavefront arrives"))
        .with_param(DspParamSchema::knob("psychoacoustic_transient_release", "Fast Transient Recovery Time Constant", 1.0, 50.0, 8.0, "ms", MacroRole::Tone, "Ultra-fast release envelope dedicated to short percussive spikes preserving punch"))
        .with_param(DspParamSchema::toggle("enable_intersample_true_peak_4x", "4x ITU-R BS.1770 True-Peak Oversampling", true, "Oversamples signal by 4x to accurately catch inter-sample peaks between digital samples"))
    );
    descriptors.insert("MultibandMasteringOpticalFeedbackLimiter".to_string(), DspNodeDescriptor::new("MultibandMasteringOpticalFeedbackLimiter", "4-Band Mastering Optical Feedback Limiter", DspNodeCategory::DynamicsMaster, "Precision 4-band mastering brickwall limiter utilizing vintage optical CdS photocell resistance decay curves, zero-overshoot lookahead soft clipping, and program-dependent release")
        .with_param(DspParamSchema::knob("master_ceiling_threshold_dbtp", "True Peak Master Ceiling Output", -12.0, 0.0, -0.3, "dBFS", MacroRole::Punch, "True peak master ceiling output threshold ensuring 0 intersample overshoots"))
        .with_param(DspParamSchema::knob("optical_cell_lag_attack_ms", "Photoconductive Optocell Lag Attack", 0.1, 10.0, 1.2, "ms", MacroRole::Tone, "Optocoupler light-dependent resistor (LDR) photoconductive attack latency"))
        .with_param(DspParamSchema::knob("dual_stage_optical_release_sec", "Program-Dependent Dual-Stage Release", 0.05, 2.5, 0.35, "s", MacroRole::Tone, "Nonlinear release: rapid recovery from short peaks, slow recovery from sustained loud audio"))
        .with_param(DspParamSchema::knob("multiband_crossover_balance_db", "4-Band Crossover Dynamic Balance", -6.0, 6.0, 0.0, "dB", MacroRole::Character, "Relative balance between low, low-mid, high-mid, and high limiter bands"))
        .with_param(DspParamSchema::knob("lookahead_window_ms", "Lookahead Buffer Window Delay", 0.5, 10.0, 3.0, "ms", MacroRole::Punch, "Lookahead delay buffer preventing transient clip distortion"))
        .with_param(DspParamSchema::toggle("enable_inter_sample_peak_detection", "4x Oversampled Inter-Sample Peak (ISP) Catcher", true, "4x oversampling filter detecting and catching inter-sample analog peaks"))
    );
    descriptors.insert("DynamicTransientSpectralSnapProcessor".to_string(), DspNodeDescriptor::new("DynamicTransientSpectralSnapProcessor", "Sub-Band Dynamic Transient Snap Processor", DspNodeCategory::DynamicsMaster, "Sub-band wavelet transient processor decomposing incoming audio into independent attack wavefronts and sustaining decay bodies to sculpt punch, snap, and crack with surgical precision")
        .with_param(DspParamSchema::knob("transient_attack_boost_db", "Attack Wavefront Boost / Attenuation", -12.0, 18.0, 4.5, "dB", MacroRole::Punch, "Amplification or attenuation of initial percussive attack wavefronts"))
        .with_param(DspParamSchema::knob("sustain_body_length_multiplier", "Sustain Decay Body Duration Multiplier", 0.2, 3.0, 1.0, "x", MacroRole::Space, "Expands or shortens the decay sustain tail of drums, guitars, and mixes"))
        .with_param(DspParamSchema::knob("transient_spectral_focus_hz", "Transient Sub-Band Detection Focus", 200.0, 8000.0, 2400.0, "Hz", MacroRole::Tone, "Center frequency for transient detection (lower for kick punch, higher for snare crack)"))
        .with_param(DspParamSchema::knob("detection_sensitivity_threshold", "Wavefront Energy Derivative Sensitivity", 0.1, 1.0, 0.55, "sens", MacroRole::Character, "Sensitivity of energy derivative envelope detector separating attack from sustain"))
        .with_param(DspParamSchema::knob("soft_clip_output_safety_db", "Soft-Clip Peak Limiting Safety Threshold", -6.0, 0.0, -0.5, "dB", MacroRole::Punch, "Soft saturation ceiling catching extreme boosted attack transients"))
        .with_param(DspParamSchema::toggle("enable_sidechain_highpass_detector", "Detector 80Hz Sub-Bass High-Pass Filter", true, "Applies 80 Hz high-pass to detector to avoid erratic triggers from sub-bass rumble"))
    );
    descriptors.insert("MultibandMasteringVcaFeedbackBusCompressor".to_string(), DspNodeDescriptor::new("MultibandMasteringVcaFeedbackBusCompressor", "Multiband Mastering VCA Feedback Bus Compressor", DspNodeCategory::DynamicsMaster, "Precision 3-band mastering console compressor using feedback VCA topology, program-dependent auto-release, and stereo punch sidechain")
        .with_param(DspParamSchema::knob("threshold", "Master Bus Compression Threshold", -40.0, 0.0, -16.0, "dB", MacroRole::Punch, "Input signal level threshold at which gain reduction begins"))
        .with_param(DspParamSchema::knob("ratio", "VCA Feedback Compression Ratio", 1.2, 10.0, 2.5, ":1", MacroRole::Punch, "Gain reduction ratio above threshold"))
        .with_param(DspParamSchema::knob("attack", "Envelope Attack Time", 0.1, 100.0, 30.0, "ms", MacroRole::Punch, "Time taken to reach full compression upon transient onset"))
        .with_param(DspParamSchema::knob("auto_release_mix", "Program Auto-Release Weight", 0.0, 1.0, 0.75, "auto", MacroRole::Punch, "Blends intelligent program-dependent release with manual setting"))
        .with_param(DspParamSchema::knob("sidechain_highpass", "Sidechain Low-Cut Filter", 20.0, 250.0, 85.0, "Hz", MacroRole::Tone, "Prevents sub-bass thump from pumping mid and high frequencies"))
        .with_param(DspParamSchema::knob("makeup_gain", "Master Output Makeup Gain", 0.0, 20.0, 2.5, "dB", MacroRole::Punch, "Master output makeup gain"))
    );
    descriptors.insert("PsychoacousticSpectralCrestFactorLimiter".to_string(), DspNodeDescriptor::new("PsychoacousticSpectralCrestFactorLimiter", "Psychoacoustic Spectral Crest Factor Limiter", DspNodeCategory::DynamicsMaster, "True-peak mastering brickwall limiter calculating sub-band crest factor to minimize audible distortion and maximize loudness")
        .with_param(DspParamSchema::knob("ceiling", "True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBTP", MacroRole::Punch, "Maximum inter-sample true peak level permitted at output"))
        .with_param(DspParamSchema::knob("crest_sensitivity", "Spectral Crest Sensitivity", 0.0, 1.0, 0.65, "sens", MacroRole::Punch, "Balances transient punch preservation against aggregate RMS density"))
        .with_param(DspParamSchema::knob("transient_release", "Fast Micro-Transient Release", 1.0, 100.0, 8.0, "ms", MacroRole::Punch, "Release speed for isolated fast transient bursts"))
        .with_param(DspParamSchema::knob("rms_recovery", "Sustained Body Recovery Time", 50.0, 1000.0, 180.0, "ms", MacroRole::Punch, "Recovery time constant for heavy sustained musical passages"))
        .with_param(DspParamSchema::knob("psychoacoustic_masking", "Critical Band Masking Weight", 0.0, 1.0, 0.85, "mask", MacroRole::Character, "Leverages human auditory masking curves to hide limiter distortion"))
        .with_param(DspParamSchema::knob("lookahead", "Zero-Distortion Lookahead Time", 0.5, 10.0, 3.0, "ms", MacroRole::Punch, "Lookahead analysis buffer preventing clipping overshoots"))
    );
    descriptors.insert("MasteringMidSideMultibandExpander".to_string(), DspNodeDescriptor::new("MasteringMidSideMultibandExpander", "Mastering Mid-Side Multiband Expander", DspNodeCategory::DynamicsMaster, "3-band mid-side downward and upward dynamic expander for restoring over-compressed mix punch, enhancing stereo ambiance, and cleaning background noise")
        .with_param(DspParamSchema::knob("mid_expand_ratio", "Mid Channel Dynamic Expand Ratio", 1.0, 4.0, 1.4, ":1", MacroRole::Punch, "Downward dynamic expansion ratio for center mid channel punch recovery"))
        .with_param(DspParamSchema::knob("side_expand_ratio", "Side Channel Dynamic Expand Ratio", 1.0, 4.0, 1.8, ":1", MacroRole::Space, "Expansion ratio on side channel widening spatial ambiance and stereo reverb"))
        .with_param(DspParamSchema::knob("low_mid_crossover", "Low / Mid Crossover Frequency", 50.0, 500.0, 140.0, "Hz", MacroRole::Tone, "Crossover split frequency separating sub-bass dynamics from midrange"))
        .with_param(DspParamSchema::knob("mid_high_crossover", "Mid / High Crossover Frequency", 1000.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Crossover split frequency separating midrange from high-frequency air"))
        .with_param(DspParamSchema::knob("expansion_threshold", "Global Dynamic Expand Threshold", -50.0, -10.0, -28.0, "dB", MacroRole::Punch, "Input level threshold below which downward expansion attenuates signal"))
        .with_param(DspParamSchema::knob("release_ballistics_time", "Expander Release Recovery Time", 20.0, 1000.0, 180.0, "ms", MacroRole::Punch, "Ballistic recovery time constant for natural musical decay envelope"))
    );
    descriptors.insert("PsychoacousticMultiStageClippingLimiter".to_string(), DspNodeDescriptor::new("PsychoacousticMultiStageClippingLimiter", "Psychoacoustic Multi-Stage Clipping Limiter", DspNodeCategory::DynamicsMaster, "Hybrid mastering stage combining soft knee asymptotic clipping with lookahead brickwall peak limitation, minimizing transient smearing while maximizing apparent volume")
        .with_param(DspParamSchema::knob("ceiling_threshold", "True-Peak Output Brickwall Ceiling", -6.0, 0.0, -0.2, "dBTP", MacroRole::Punch, "Strict inter-sample maximum peak ceiling for streaming broadcast"))
        .with_param(DspParamSchema::knob("soft_knee_clip_curve", "Asymptotic Soft-Clipper Knee", 0.0, 1.0, 0.45, "knee", MacroRole::Punch, "Smoothness of polynomial soft clipping curve shaving fast micro-transients"))
        .with_param(DspParamSchema::knob("oversampling_factor", "Polyphase Anti-Aliasing Oversampling", 2.0, 16.0, 8.0, "x", MacroRole::Character, "Oversampling rate eliminating audible intermodulation foldback artifacts"))
        .with_param(DspParamSchema::knob("transient_punch_drive", "Pre-Limiter Transient Punch Drive", 0.0, 18.0, 4.0, "dB", MacroRole::Punch, "Input gain pushing mix transients into the multi-stage limiter core"))
        .with_param(DspParamSchema::knob("lookahead_safety_ms", "Predictive Lookahead Window", 0.5, 8.0, 2.5, "ms", MacroRole::Punch, "Lookahead analysis buffer preventing sudden transient overshoots"))
        .with_param(DspParamSchema::knob("true_peak_inter_sample_clamp", "Inter-Sample Peak Guard Clamping", 0.0, 1.0, 1.0, "guard", MacroRole::Punch, "Detects and eliminates reconstructed analog inter-sample peak overshoots"))
    );
    descriptors.insert("MultibandMasteringDiodeBridgeLimiter".to_string(), DspNodeDescriptor::new("MultibandMasteringDiodeBridgeLimiter", "Multiband Mastering Diode-Bridge Limiter", DspNodeCategory::DynamicsMaster, "Class-A diode bridge mastering limiter with Linkwitz-Riley 4-band crossover, ultra-fast transient clamping, analog diode temperature compensation, and zero overshoots")
        .with_param(DspParamSchema::knob("threshold_ceiling_db", "Master Limiting Threshold Ceiling", -20.0, 0.0, -0.3, "dB", MacroRole::Punch, "Absolute brickwall limiting threshold ceiling for broadcast and streaming masters"))
        .with_param(DspParamSchema::knob("diode_bridge_bias_current", "Diode Bridge Bias Operating Point", 0.1, 5.0, 1.2, "mA", MacroRole::Character, "Diode bridge conduction quiescent current setting vintage limiting knee softness"))
        .with_param(DspParamSchema::knob("crossover_low_mid_freq", "Low-Mid Band Crossover Split", 80.0, 600.0, 250.0, "Hz", MacroRole::Tone, "Linkwitz-Riley 4th-order crossover dividing low and low-mid dynamics bands"))
        .with_param(DspParamSchema::knob("crossover_mid_high_freq", "Mid-High Band Crossover Split", 1000.0, 8000.0, 3500.0, "Hz", MacroRole::Tone, "Crossover frequency isolating presence and top-end sibilance for transparent limiting"))
        .with_param(DspParamSchema::knob("fast_recovery_release_ms", "Adaptive Transient Release Speed", 5.0, 500.0, 45.0, "ms", MacroRole::Punch, "Program-dependent dual-time constant release recovering smoothly after loud peaks"))
        .with_param(DspParamSchema::knob("stereo_sidechain_link", "Stereo Sidechain Detection Link", 0.0, 1.0, 0.85, "link", MacroRole::Space, "Inter-channel sidechain linking preventing stereo image shifting during heavy gain reduction"))
    );
    descriptors.insert("DynamicPsychoacousticTransientExciter".to_string(), DspNodeDescriptor::new("DynamicPsychoacousticTransientExciter", "Dynamic Psychoacoustic Transient Exciter", DspNodeCategory::DynamicsMaster, "Frequency-selective transient shaping processor generating psychoacoustic overtone harmonics in high-frequency bands during attack onsets without lifting background noise floors")
        .with_param(DspParamSchema::knob("transient_attack_boost", "Attack Edge Transient Gain", -12.0, 12.0, 4.5, "dB", MacroRole::Punch, "Differential envelope detector gain boosting or attenuating initial note attacks"))
        .with_param(DspParamSchema::knob("harmonic_excitation_drive", "High-Frequency Harmonic Generator", 0.0, 1.0, 0.55, "drive", MacroRole::Tone, "Psychoacoustic harmonic generator adding sheen and presence to percussive transients"))
        .with_param(DspParamSchema::knob("excitation_corner_frequency", "Exciter High-Pass Filter Cutoff", 2000.0, 12000.0, 5000.0, "Hz", MacroRole::Tone, "High-pass sidechain split ensuring only high-frequency attacks trigger excitation"))
        .with_param(DspParamSchema::knob("transient_decay_envelope", "Transient Window Integration Time", 2.0, 80.0, 18.0, "ms", MacroRole::Punch, "Duration of the transient attack detection window"))
        .with_param(DspParamSchema::knob("dynamic_noise_gate_floor", "Background Ambience Gate Threshold", -80.0, -24.0, -54.0, "dB", MacroRole::Character, "Downwards expander threshold muting excitation during steady sustain passages"))
        .with_param(DspParamSchema::knob("wet_dry_master_blend", "Transient Processor Wet/Dry Mix", 0.0, 1.0, 0.70, "mix", MacroRole::Space, "Parallel blend ratio between unprocessed audio and transient-enhanced signal"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "CompressorNode" => Box::new(
            GenericDspNodeUi::new("CompressorNode", "VCA Studio Compressor with RMS Detection", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold", "Threshold", -48.0, 0.0, -18.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("ratio", "Compression Ratio", 1.0, 20.0, 4.0, ":1", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("attack", "Attack Time", 0.1, 200.0, 15.0, "ms", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_log("release", "Release Time", 10.0, 2000.0, 150.0, "ms", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("makeup", "Makeup Gain", 0.0, 24.0, 3.0, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("knee", "Soft Knee Width", 0.0, 18.0, 3.0, "dB", (180, 195, 215)))
        ),
        "MultibandCompressorNode" => Box::new(
            GenericDspNodeUi::new("MultibandCompressorNode", "3-Band Linear-Phase Dynamics Processor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_thresh", "Low Band Threshold", -40.0, 0.0, -18.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mid_thresh", "Mid Band Threshold", -40.0, 0.0, -16.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_thresh", "High Band Threshold", -40.0, 0.0, -14.0, "dB", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_log("low_cross", "Low/Mid Crossover", 60.0, 500.0, 180.0, "Hz", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_log("high_cross", "Mid/High Crossover", 1000.0, 10000.0, 3500.0, "Hz", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("master_gain", "Master Output Trim", -12.0, 12.0, 0.0, "dB", (180, 195, 215)))
        ),
        "MultibandDynamicsProcessor" => Box::new(
            GenericDspNodeUi::new("MultibandDynamicsProcessor", "Tri-Band Expander/Compressor Dynamics Matrix", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_comp", "Low Band Compression", 0.0, 100.0, 35.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mid_comp", "Mid Band Compression", 0.0, 100.0, 30.0, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_comp", "High Band Compression", 0.0, 100.0, 25.0, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_log("crossover_1", "Low/Mid Crossover", 80.0, 600.0, 200.0, "Hz", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_log("crossover_2", "Mid/High Crossover", 1500.0, 8000.0, 4000.0, "Hz", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("saturation", "Tape Saturation Drive", 0.0, 1.0, 0.2, "%", (180, 195, 215)))
        ),
        "OpticalCompressorNode" => Box::new(
            GenericDspNodeUi::new("OpticalCompressorNode", "Vintage LA-2A Optical Photocell Dynamics", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("peak_reduct", "Peak Reduction", 0.0, 100.0, 45.0, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 100.0, 50.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("limit_mode", "Compress / Limit Mode", false, (255, 61, 113)))
        ),
        "NoiseGateNode" => Box::new(
            GenericDspNodeUi::new("NoiseGateNode", "Fast Transient Expander & Noise Gate", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold", "Gate Threshold", -80.0, 0.0, -45.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("attack", "Fast Attack", 0.05, 50.0, 1.0, "ms", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("hold", "Hold Duration", 0.0, 500.0, 20.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("release", "Smooth Release", 5.0, 1000.0, 80.0, "ms", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("range", "Floor Attenuation Range", -80.0, 0.0, -60.0, "dB", (255, 61, 113)))
        ),
        "DeesserNode" => Box::new(
            GenericDspNodeUi::new("DeesserNode", "Dynamic High-Frequency Sibilance Suppressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold", "De-Esser Threshold", -40.0, 0.0, -20.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("freq", "Sibilance Center Freq", 3000.0, 12000.0, 6500.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("listen", "Delta Audition Listen", false, (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("reduction", "Max Reduction Cap", -24.0, 0.0, -12.0, "dB", (153, 102, 255)))
        ),
        "MultibandClipperNode" => Box::new(
            GenericDspNodeUi::new("MultibandClipperNode", "Tri-Band Mastering Clipper & Saturation Stage", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_clip_ceiling_db", "Low Band Ceiling", -12.0, 0.0, -0.5, "dBFS", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mid_clip_ceiling_db", "Mid Band Ceiling", -12.0, 0.0, -0.3, "dBFS", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_clip_ceiling_db", "High Band Ceiling", -12.0, 0.0, -0.1, "dBFS", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("softness_knee", "Soft Clipping Knee", 0.0, 1.0, 0.35, "%", (0, 230, 118)))
        ),
        "MultibandDecompressorNode" => Box::new(
            GenericDspNodeUi::new("MultibandDecompressorNode", "Multiband Upward Dynamic Range Decompressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("expansion_ratio", "Upward Expansion Ratio", 1.0, 4.0, 1.5, ":1", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("upward_threshold_db", "Decompression Threshold", -40.0, -10.0, -24.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("crossover_low_hz", "Low/Mid Split Frequency", 60.0, 500.0, 160.0, "Hz", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_log("crossover_high_hz", "Mid/High Split Frequency", 1500.0, 10000.0, 4000.0, "Hz", (153, 102, 255)))
        ),
        "MultibandExpanderNode" => Box::new(
            GenericDspNodeUi::new("MultibandExpanderNode", "Tri-Band Downward Transient Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("downward_ratio", "Expansion Ratio", 1.0, 8.0, 2.0, ":1", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("gate_threshold_db", "Expansion Floor Gate", -60.0, -10.0, -32.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("range_cap_db", "Max Dynamic Range Cap", -40.0, 0.0, -18.0, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("release_time_ms", "Expander Recovery Time", 10.0, 500.0, 85.0, "ms", (0, 230, 118)))
        ),
        "MultibandSaturatorNode" => Box::new(
            GenericDspNodeUi::new("MultibandSaturatorNode", "3-Band Frequency-Split Saturation Warmer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_drive_x", "Low Band Tape Warmth", 1.0, 10.0, 2.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mid_drive_x", "Mid Band Tube Drive", 1.0, 10.0, 3.2, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_drive_x", "High Band Diode Sheen", 1.0, 10.0, 1.8, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("tape_tube_blend", "Tape vs Tube Color Blend", 0.0, 1.0, 0.5, "%", (255, 61, 113)))
        ),
        "UpwardCompressorNode" => Box::new(
            GenericDspNodeUi::new("UpwardCompressorNode", "OTT-Style Upward Audio Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("upward_depth", "Upward Compression Depth", 0.0, 100.0, 65.0, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("downward_depth", "Downward Compression Depth", 0.0, 100.0, 40.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("dynamics_speed", "Dynamics Ballistics Speed", 0.1, 10.0, 2.5, "x", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("high_lift_db", "High Air Presence Lift", 0.0, 18.0, 4.0, "dB", (153, 102, 255)))
        ),
        "VariMuMasterNode" => Box::new(
            GenericDspNodeUi::new("VariMuMasterNode", "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("time_constant", "Fairchild Time Constant", vec!["Pos 1 (Fast 0.2s/0.3s)".into(), "Pos 2 (0.2s/0.8s)".into(), "Pos 3 (0.4s/2.0s)".into(), "Pos 4 (0.8s/5.0s)".into(), "Pos 5 (Auto Fast)".into(), "Pos 6 (Auto Program)".into()], 4, (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("dc_threshold", "Vari-Mu Tube Threshold", -30.0, 0.0, -12.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("tube_bias_mode", "Tube Bias Configuration", vec!["Standard Push-Pull".into(), "Aggressive Mid Push".into(), "Clean Master Linear".into()], 0, (255, 61, 113)))
                .with_param(DspParamDescriptor::new_bool("link_channels", "Stereo Linked Sidechain", true, (0, 230, 118)))
        ),
        "DynamicCrestShaperNode" => Box::new(
            GenericDspNodeUi::new("DynamicCrestShaperNode", "Intelligent Peak-to-RMS Crest Factor Shaper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_crest_db", "Target Crest Factor", 6.0, 20.0, 12.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("transient_priority", "Transient vs Body Weight", 0.0, 1.0, 0.6, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("smoothing_window_ms", "RMS Detection Window", 10.0, 500.0, 100.0, "ms", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_bool("auto_makeup", "Auto Crest Makeup Gain", true, (153, 102, 255)))
        ),
        "ParallelTransientSaturatorNode" => Box::new(
            GenericDspNodeUi::new("ParallelTransientSaturatorNode", "Parallel Wet/Dry Transient Driver & Saturator", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("transient_drive_db", "Transient Punch Drive", 0.0, 24.0, 6.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("sustain_warmth_db", "Sustain Body Saturation", 0.0, 24.0, 4.5, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("parallel_mix", "Parallel Wet Blend", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("clipping_mode", "Clipping Topology", vec!["Soft Germanium Diode".into(), "Hard Silicon Diode".into(), "Asymmetric Tube Triode".into()], 0, (153, 102, 255)))
        ),
        "TransientClipperNode" => Box::new(
            GenericDspNodeUi::new("TransientClipperNode", "Sub-Millisecond True Transient Soft Clipper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_db", "Clipping Knee Threshold", -18.0, 0.0, -3.0, "dBFS", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("clip_hardness", "Clipper Hardness Exponent", 0.1, 5.0, 1.2, "", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_int("oversampling_order", "Anti-Aliasing Oversampling", 1, 16, 4, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("ceiling_db", "True Peak Limit Ceiling", -6.0, 0.0, -0.1, "dBTP", (0, 230, 118)))
        ),
        "TransientDeclickerNode" => Box::new(
            GenericDspNodeUi::new("TransientDeclickerNode", "Surgical Sample De-Clicker & Pop Filter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Detection Threshold", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("click_width_samples", "Max Click Width", 2, 128, 16, "samples", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("spectral_inpaint_depth", "Inpainting Reconstruction", 0.0, 1.0, 0.9, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_bool("listen_delta", "Audition Isolated Clicks", false, (255, 61, 113)))
        ),
        "TransientDesignerNode" => Box::new(
            GenericDspNodeUi::new("TransientDesignerNode", "Dual-Band Attack & Sustain Transient Designer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("attack_gain_db", "Attack Transient Gain", -15.0, 15.0, 3.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("sustain_gain_db", "Sustain Tail Gain", -15.0, 15.0, -2.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("split_frequency_hz", "Band Split Frequency", 100.0, 5000.0, 1200.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("output_gain_db", "Master Output Level", -12.0, 12.0, 0.0, "dB", (0, 230, 118)))
        ),
        "TransientGateNode" => Box::new(
            GenericDspNodeUi::new("TransientGateNode", "Envelope-Triggered Transient Noise Gate", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_db", "Trigger Threshold", -70.0, 0.0, -40.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("attack_fast_ms", "Fast Attack Speed", 0.01, 10.0, 0.5, "ms", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("hold_time_ms", "Hold Gate Duration", 0.0, 250.0, 15.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("floor_attenuation_db", "Gate Attenuation Floor", -80.0, 0.0, -50.0, "dB", (255, 61, 113)))
        ),
        "TransientReconstructorNode" => Box::new(
            GenericDspNodeUi::new("TransientReconstructorNode", "Spectral Attack Transient Reconstructor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("attack_restoration_db", "Attack Restoration Boost", 0.0, 18.0, 5.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("harmonic_synthesis_q", "Harmonic Phase Match Q", 1.0, 20.0, 6.0, "Q", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("onset_sensitivity", "Onset Trigger Sensitivity", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("wet_dry", "Reconstructed Blend", 0.0, 1.0, 0.6, "%", (153, 102, 255)))
        ),
        "TransientShaperNode" => Box::new(
            GenericDspNodeUi::new("TransientShaperNode", "Zero-Latency Transient Punch & Sustain Shaper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("punch_level_db", "Transient Punch Level", -12.0, 12.0, 4.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("body_level_db", "Body / Sustain Tail Level", -12.0, 12.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("recovery_speed_ms", "Transient Recovery Speed", 5.0, 200.0, 45.0, "ms", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_bool("clip_protection", "Soft Clip Brickwall Guard", true, (255, 61, 113)))
        ),
        "TransientUnwrapperNode" => Box::new(
            GenericDspNodeUi::new("TransientUnwrapperNode", "De-Compression & Dynamic Unwrapper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("crest_expansion_ratio", "Dynamic Unwrapping Ratio", 1.0, 3.5, 1.6, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("unwrapper_threshold_db", "Unwrapping Floor Level", -36.0, -6.0, -18.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("transient_preserve", "Preserve Fast Transients", true, (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("gain_trim_db", "Post-Unwrap Output Trim", -12.0, 12.0, -1.0, "dB", (153, 102, 255)))
        ),
        "LimiterNode" => Box::new(
            GenericDspNodeUi::new("LimiterNode", "True-Peak Lookahead Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling", "True Peak Ceiling", -12.0, 0.0, -0.3, "dBFS", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("threshold", "Limiter Threshold", -24.0, 0.0, -3.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("release_ms", "Lookahead Release", 5.0, 500.0, 50.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("oversampling", "4x ISP Oversampling", true, (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer", 0.5, 10.0, 3.0, "ms", (153, 102, 255)))
        ),
        "EbuLoudnessRadar" => Box::new(
            GenericDspNodeUi::new("EbuLoudnessRadar", "ITU-R BS.1770 / EBU R128 Loudness Radar Meter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_lufs", "Target Integrated Loudness", -24.0, -9.0, -14.0, "LUFS", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("integrated_gate", "Relative Silence Gate", -70.0, -10.0, -23.0, "LUFS", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("max_momentary", "Max Momentary True Peak", -18.0, 0.0, -1.0, "dBTP", (255, 61, 113)))
        ),
        "LufsMeterNode" => Box::new(
            GenericDspNodeUi::new("LufsMeterNode", "Multi-Scale Momentary & Integrated LUFS Meter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("momentary_window", "Momentary Window", 100.0, 1000.0, 400.0, "ms", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("short_term_window", "Short-Term Window", 1000.0, 5000.0, 3000.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("relative_threshold", "Relative Gate Threshold", -20.0, -5.0, -10.0, "LU", (255, 171, 0)))
        ),
        "MidSideNode" => Box::new(
            GenericDspNodeUi::new("MidSideNode", "Mid/Side Matrix Encoder & Stereo Width Imager", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("width", "Stereo Field Width", 0.0, 3.0, 1.0, "x", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("mid_gain", "Mid Channel Gain", -12.0, 12.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("side_gain", "Side Channel Gain", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("mono_sub", "Mono Bass Crossover", 20.0, 250.0, 90.0, "Hz", (0, 230, 118)))
        ),
        "StereoImager" => Box::new(
            GenericDspNodeUi::new("StereoImager", "Phase-Safe Psychoacoustic Stereo Imager", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("width_factor", "Stereo Width Expansion", 0.0, 2.5, 1.2, "x", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_log("bass_mono_cutoff", "Bass Mono Filter Corner", 30.0, 300.0, 120.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("phase_correlation_protect", "Phase Correlation Protection", true, (0, 230, 118)))
        ),
        "TpdfDitherNode" => Box::new(
            GenericDspNodeUi::new("TpdfDitherNode", "Triangular PDF Dither & Noise Shaping Quantizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("bit_depth", "Target Bit Depth", vec!["24-bit HD".into(), "16-bit Red Book CD".into(), "12-bit Vintage".into(), "8-bit Lo-Fi".into()], 1, (255, 51, 153)))
                .with_param(DspParamDescriptor::new_enum("noise_shaping", "Noise Shaping Curve", vec!["None (Flat TPDF)".into(), "Lipshitz Moderate".into(), "High-Pass Ultra".into(), "Psychoacoustic F-Weight".into()], 1, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("auto_blank", "Auto Blank On Digital Silence", true, (0, 230, 118)))
        ),
        "AdaptivePsychoacousticLoudness" => Box::new(
            GenericDspNodeUi::new("AdaptivePsychoacousticLoudness", "ISO 226 Equal Loudness Contour Balancing Engine", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_phon", "Target Phon Loudness", 40.0, 100.0, 75.0, "phon", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("phon_compensation_intensity", "Equal Loudness Compensation", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("low_contour_boost_db", "Fletcher-Munson Sub Boost", 0.0, 12.0, 3.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_contour_presence_db", "Treble Contour Presence", 0.0, 6.0, 1.8, "dB", (0, 230, 118)))
        ),
        "DynamicStereoWidthNode" => Box::new(
            GenericDspNodeUi::new("DynamicStereoWidthNode", "Frequency-Dependent Mid/Side Dynamic Stereo Widener", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("stereo_expansion_factor", "Side Band Expansion", 0.0, 3.0, 1.35, "x", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_log("bass_mono_crossover_hz", "Mono Bass Frequency Cut", 30.0, 250.0, 100.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("high_side_air_gain_db", "Side Channel Air Boost", 0.0, 6.0, 1.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("phase_correlation_protect", "Auto Anti-Phase Limiter", true, (0, 230, 118)))
        ),
        "KSystemMeterNode" => Box::new(
            GenericDspNodeUi::new("KSystemMeterNode", "Bob Katz K-12 / K-14 / K-20 Metering Bridge", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("meter_scale", "K-Scale Target Reference", vec!["K-12 (Broadcast / Pop)".into(), "K-14 (Standard Music)".into(), "K-20 (Audiophile / Film Dynamic)".into()], 1, (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("headroom_target_db", "0 VU Calibration Offset", 0.0, 20.0, 14.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("peak_hold_time_ms", "Peak Hold Persistence", 100.0, 3000.0, 1500.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("monitor_calibration_db", "SPL Reference Calibration", 60.0, 90.0, 83.0, "dBSPL", (0, 230, 118)))
        ),
        "EqualLoudnessContourNode" => Box::new(
            GenericDspNodeUi::new("EqualLoudnessContourNode", "Fletcher-Munson Psychoacoustic Equal Loudness Compensator", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("phon_level", "Listening Volume (Phon)", 30.0, 100.0, 80.0, "phon", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_enum("curve_version", "Contour Standard", vec!["ISO 226:2003 Standard".into(), "Fletcher-Munson (1933)".into(), "Robinson-Dadson (1956)".into()], 0, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("bass_contour_trim", "Low Contour Trim", -6.0, 6.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("treble_contour_trim", "High Contour Trim", -6.0, 6.0, 0.0, "dB", (0, 230, 118)))
        ),
        "MasterLimiterRadarNode" => Box::new(
            GenericDspNodeUi::new("MasterLimiterRadarNode", "True-Peak Limiter with Integrated Loudness Radar", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("true_peak_ceiling_dbtp", "True-Peak Ceiling", -6.0, 0.0, -0.3, "dBTP", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Integrated LUFS", -24.0, -8.0, -14.0, "LUFS", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("lookahead_time_ms", "True-Peak Lookahead", 1.0, 10.0, 4.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("release_curve_adaptive", "Adaptive Program Release", true, (0, 230, 118)))
        ),
        "MeterBridgeNode" => Box::new(
            GenericDspNodeUi::new("MeterBridgeNode", "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("meter_type", "Ballistics Standard", vec!["True-Peak ISP (ITU-R BS.1770)".into(), "VU Meter (ANSI C16.5)".into(), "Nordic PPM (IEC 60268-10)".into(), "BBC Type II PPM".into()], 0, (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("integration_window_ms", "RMS Integration Window", 50.0, 1000.0, 300.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("vu_reference_db", "VU Zero Reference", -24.0, -10.0, -18.0, "dBFS", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("peak_decay_speed", "Decay Ballistics Rate", 5.0, 60.0, 20.0, "dB/s", (0, 230, 118)))
        ),
        "MidSideFocuserNode" => Box::new(
            GenericDspNodeUi::new("MidSideFocuserNode", "Surgical Mid/Side Bass Mono & High-Side Air Focuser", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_log("bass_mono_freq_hz", "Mono Bass Frequency Cut", 30.0, 300.0, 110.0, "Hz", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("high_side_boost_db", "Side Channel Air Sheen", 0.0, 6.0, 1.8, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("side_dynamic_compression", "Side Band Dynamic Tame", 0.0, 1.0, 0.4, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("mid_presence_gain_db", "Mid Channel Vocal Presence", -6.0, 6.0, 0.8, "dB", (0, 230, 118)))
        ),
        "MultibandImagerNode" => Box::new(
            GenericDspNodeUi::new("MultibandImagerNode", "4-Band Mastering Stereo Width & Phase Correlator", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("band1_sub_width", "Sub Width (<120Hz)", 0.0, 2.0, 0.0, "x", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("band2_low_width", "Low-Mid Width (120-1kHz)", 0.0, 2.0, 0.9, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("band3_mid_width", "High-Mid Width (1k-6kHz)", 0.0, 2.0, 1.25, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("band4_high_width", "Air Width (>6kHz)", 0.0, 2.0, 1.4, "x", (0, 230, 118)))
        ),
        "OversampledLimiterNode" => Box::new(
            GenericDspNodeUi::new("OversampledLimiterNode", "16x Polyphase Oversampled Intersample Peak Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("oversampling_rate", "Polyphase Oversampling", vec!["2x Polyphase".into(), "4x High Quality".into(), "8x Mastering Grade".into(), "16x Ultra Fidelity".into()], 2, (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("brickwall_ceiling_dbfs", "Ceiling Level", -6.0, 0.0, -0.2, "dBFS", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("release_time_ms", "Lookahead Release Speed", 5.0, 400.0, 45.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("isp_guard_active", "Intersample Peak Guard", true, (0, 230, 118)))
        ),
        "PolarPhaseCorrelatorNode" => Box::new(
            GenericDspNodeUi::new("PolarPhaseCorrelatorNode", "Polar Goniometer Lissajous Phase Correlation Display", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("phosphor_persistence_ms", "Goniometer Phosphor Decay", 50.0, 1000.0, 300.0, "ms", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("polar_scaling", "Polar Radius Scale", 0.5, 3.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("lissajous_resolution", "Display Point Resolution", 128, 2048, 512, "pts", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("phase_cancel_alert", "Phase Inversion Warning", true, (255, 61, 113)))
        ),
        "ResonanceSuppressorNode" => Box::new(
            GenericDspNodeUi::new("ResonanceSuppressorNode", "Automatic Dynamic Harmonic Notch Resonance Suppressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("suppression_depth_db", "Dynamic Notch Depth", 0.0, 18.0, 6.0, "dB", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("notch_q_sharpness", "Dynamic Q Sharpness", 2.0, 50.0, 15.0, "Q", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("attack_speed_ms", "Resonance Lock Attack", 1.0, 100.0, 12.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("max_active_notches", "Concurrent Notch Filters", 1, 16, 6, "filters", (0, 230, 118)))
        ),
        "StereoWidenerNode" => Box::new(
            GenericDspNodeUi::new("StereoWidenerNode", "Haas Effect & Phase-Coherent Stereo Field Widener", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("haas_delay_ms", "Haas Delay Offset", 0.1, 30.0, 12.0, "ms", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("mid_side_ratio", "Side vs Mid Ratio", 0.0, 3.0, 1.25, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("center_channel_retention", "Center Lead Vocal Retain", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("phase_filter_q", "Allpass Phase Filter Q", 0.5, 5.0, 1.2, "Q", (0, 230, 118)))
        ),
        "TapeFluxMasterNode" => Box::new(
            GenericDspNodeUi::new("TapeFluxMasterNode", "Half-Inch Mastering Tape Saturation & Flux Density Engine", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("tape_flux_nanoweber", "Reference Fluxivity", 185.0, 510.0, 355.0, "nWb/m", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_enum("tape_formulation", "Mastering Tape Stock", vec!["SM900 High Output (+9dB)".into(), "ATR Magnetics Master (+6dB)".into(), "GP9 Grand Master (+9dB)".into(), "Scotch 250 Vintage (+3dB)".into()], 0, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("bias_hf_saturation", "High-Frequency Bias Saturation", -3.0, 6.0, 1.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("flux_compression_db", "Head Hysteresis Compression", 0.0, 6.0, 1.8, "dB", (0, 230, 118)))
        ),
        "DialogGatingNode" => Box::new(
            GenericDspNodeUi::new("DialogGatingNode", "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("speech_presence_threshold_db", "Speech Presence Gate", -40.0, -10.0, -24.0, "dB", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("gating_window_ms", "Voice Energy Gating Window", 100.0, 1000.0, 400.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("ambient_noise_attenuation_db", "Background Attenuation", -30.0, 0.0, -12.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("voice_clarity_lift_db", "Formant Intelligibility Lift", 0.0, 6.0, 2.0, "dB", (0, 230, 118)))
        ),
        "SonarHydrophoneNode" => Box::new(
            GenericDspNodeUi::new("SonarHydrophoneNode", "Underwater Acoustic Hydrophone & Thermal Layer Simulation", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("hydrophone_depth_m", "Sensor Water Depth", 1.0, 1000.0, 75.0, "m", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("water_salinity_ppt", "Ocean Salinity", 0.0, 45.0, 35.0, "ppt", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("sound_channel_axis_depth_m", "SOFAR Channel Axis Depth", 100.0, 1500.0, 800.0, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("cavitation_noise_level", "Propeller Cavitation Ambient", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
        ),
        "AiAutonomousMasteringEngine" => Box::new(
            GenericDspNodeUi::new("AiAutonomousMasteringEngine", "AI Autonomous Mastering Engine", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("target_curve", "Target Curve", vec!["Modern Pop".into(), "EDM Club".into(), "Warm Vintage".into(), "Acoustic Folk".into(), "Classical Concert".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("target_lufs", "Target LUFS", -24.0, -6.0, -14.0, "LUFS", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("ceiling_db", "Ceiling dBFS", -2.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("punch", "Transient Punch", 0.0, 1.0, 0.50, "%", (239, 68, 68)))
        ),
        "PeakHeadroomAnalyzer" => Box::new(
            GenericDspNodeUi::new("PeakHeadroomAnalyzer", "Real-Time Inter-Sample True Peak & Headroom Analyzer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_headroom_db", "Target Headroom", 0.0, 12.0, 1.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("peak_sample_db", "Sample Peak Level", -120.0, 6.0, -12.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("true_peak_db", "Inter-Sample True Peak", -120.0, 6.0, -11.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("oversampling_mode", "8x Sinc Oversampling", true, (99, 102, 241)))
        ),
        "EbuR128LoudnessMeter" => Box::new(
            GenericDspNodeUi::new("EbuR128LoudnessMeter", "EBU R128 & ITU-R BS.1770 Broadcast Loudness Radar & Meter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_lufs", "Target Loudness", -36.0, -6.0, -23.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("integrated_lufs", "Integrated LUFS", -70.0, 0.0, -23.0, "LUFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("momentary_lufs", "Momentary LUFS", -70.0, 6.0, -21.0, "LUFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("short_term_lufs", "Short-Term LUFS", -70.0, 6.0, -22.5, "LUFS", (16, 185, 129)))
        ),
        "SidechainMatrix" => Box::new(
            GenericDspNodeUi::new("SidechainMatrix", "Multi-Channel Dynamic Sidechain Matrix & Routing Cross-Bar Hub", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("source_bus_index", "Sidechain Key Source Bus", 1.0, 16.0, 1.0, "bus", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("sidechain_highpass_hz", "Key Highpass Filter", 20.0, 1000.0, 120.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("sidechain_lowpass_hz", "Key Lowpass Filter", 1000.0, 20000.0, 12000.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("key_listen", "Key Signal Audition", false, (16, 185, 129)))
        ),
        "BandDynamics" => Box::new(
            GenericDspNodeUi::new("BandDynamics", "Per-Band Multiband Dynamic Compressor & Expander Stage", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_db", "Band Threshold", -60.0, 0.0, -18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio", "Compression Ratio", 1.0, 20.0, 3.5, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attack_time_ms", "Attack Response", 0.1, 250.0, 15.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("release_time_ms", "Release Response", 5.0, 1500.0, 120.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("expansion_ratio", "Expansion Ratio", 0.2, 1.0, 1.0, "exp", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("makeup_gain_db", "Band Makeup Gain", -12.0, 24.0, 2.5, "dB", (168, 85, 247)))
        ),
        "MasterLimiter" => Box::new(
            GenericDspNodeUi::new("MasterLimiter", "Lookahead Inter-Sample True-Peak Brickwall Mastering Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_db", "Limiter Ceiling Threshold", -24.0, 0.0, -0.2, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("input_gain_db", "Pre-Limiter Input Boost", 0.0, 24.0, 4.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lookahead_ms", "True Peak Lookahead Time", 0.5, 10.0, 4.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("release_time_ms", "Auto Dynamic Release Time", 10.0, 1000.0, 120.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("soft_knee_db", "Soft Knee Transition", 0.0, 6.0, 1.5, "dB", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_bool("true_peak_oversampling", "4x Inter-Sample Oversampling", true, (236, 72, 153)))
        ),
        "SidechainRoute" => Box::new(
            GenericDspNodeUi::new("SidechainRoute", "Lock-Free Dynamic Sidechain Ducking & Routing Matrix", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sidechain_source_ch", "Sidechain Source Bus ID", 1.0, 16.0, 1.0, "bus", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ducking_amount_db", "Gain Reduction Depth", 0.0, 36.0, 12.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ducking_release_ms", "Ducking Release Time", 10.0, 1000.0, 120.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("sidechain_hpf_hz", "Sidechain Filter Low Cut", 20.0, 1000.0, 120.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("listen_sidechain", "Audition Sidechain Trigger Input", false, (16, 185, 129)))
        ),
        "ExportPresetManager" => Box::new(
            GenericDspNodeUi::new("ExportPresetManager", "Multi-Format Stem Export & Mastering Batch HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("export_sample_rate", "Export Master Audio Sample Rate", vec!["44100 Hz (CD)".into(), "48000 Hz (Broadcast)".into(), "88200 Hz (High-Res)".into(), "96000 Hz (Studio Pro)".into(), "192000 Hz (Mastering)".into()], 1, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("export_bit_depth", "Export Audio Bit Depth & Lossless Codec", vec!["16-bit PCM".into(), "24-bit PCM".into(), "32-bit Float".into(), "FLAC Lossless".into(), "MP3 320kbps".into()], 2, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("peak_normalization_target_db", "True-Peak Master Normalization Ceiling", -6.0, 0.0, -0.3, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("dither_noise_shaping_mode", "Quantization Dither & Noise Shaping Mode", vec!["None (Truncate)".into(), "Rectangular TPDF".into(), "Triangular TPDF".into(), "High-Pass Shaped Dither".into()], 2, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("stem_auto_naming_pattern", "Multi-Track Stem File Naming Template", vec!["{track_num}_{track_name}".into(), "{bpm}_{key}_{track_name}".into(), "{project}_{track_name}_{date}".into(), "Custom Pattern".into()], 0, (34, 197, 94)))
        ),
        "SessionAnalyticsDashboard" => Box::new(
            GenericDspNodeUi::new("SessionAnalyticsDashboard", "Real-Time Telemetry & DSP Headroom HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("dsp_load_sampling_window_ms", "DSP Processing Load Averaging Window", 10.0, 1000.0, 100.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("peak_cpu_overrun_alert_threshold_pct", "Audio Buffer Overrun Alert Threshold", 50.0, 99.0, 90.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("track_memory_allocations_active", "Monitor Real-Time Audio Heap Allocations", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("audio_underrun_event_counter_reset", "Zero Cumulative Audio Buffer Dropouts Counter", false, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("history_graph_retention_minutes", "Telemetry Performance Timeline History Scope", 5.0, 120.0, 30.0, "min", (34, 197, 94)))
        ),
        "GoldenRenderSuite" => Box::new(
            GenericDspNodeUi::new("GoldenRenderSuite", "Bit-Exact DSP Regression Test Harness HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("golden_reference_sample_rate", "Regression Test Benchmark Sample Rate", vec!["44100 Hz".into(), "48000 Hz".into(), "96000 Hz".into(), "192000 Hz".into()], 1, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("rms_tolerance_threshold_db", "Null-Test RMS Cancellation Floor", -140.0, -60.0, -96.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_phase_correlation_min", "Minimum Acceptable Phase Correlation", 0.90, 1.0, 0.999, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("generate_spectral_difference_map", "Generate 2D Spectral Difference Waterfall", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("abort_suite_on_first_divergence", "Halt Test Suite on First Regression Error", false, (34, 197, 94)))
        ),
        "VideoExportConfig" => Box::new(
            GenericDspNodeUi::new("VideoExportConfig", "Video Soundtrack Sync & Video Renderer HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("video_codec_encoder", "Hardware Accelerated Video Codec", vec!["H.264 / AVC Hardware".into(), "H.265 / HEVC 10-bit".into(), "Apple ProRes 422 HQ".into(), "AV1 Open Source".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("output_video_resolution", "Video Output Resolution & Aspect Ratio", vec!["1080p Full HD (1920x1080)".into(), "4K UHD (3840x2160)".into(), "720p HD (1280x720)".into(), "Instagram Square (1080x1080)".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("video_framerate_fps", "Video Master Timeline Framerate", vec!["24.0 fps (Film)".into(), "25.0 fps (PAL)".into(), "29.97 fps (NTSC)".into(), "30.0 fps".into(), "60.0 fps (Smooth)".into()], 3, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("render_waveform_overlay", "Burn Oscilloscope Waveform Overlay Into Video", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("video_bitrate_mbps", "Video Stream Compression Target Bitrate", 5.0, 100.0, 25.0, "Mbps", (34, 197, 94)))
        ),
        "AiMixBalanceInspector" => Box::new(
            GenericDspNodeUi::new("AiMixBalanceInspector", "AI Spectral Masking & Autonomous Balance Inspector HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("target_spectral_tilt", "Target Genre Psychoacoustic Spectral Curve", vec!["Balanced Commercial Curve".into(), "Warm Analog Tilt".into(), "Bright Pop Presentation".into(), "Heavy Bass Heavy-End".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("masking_detection_sensitivity_db", "Spectral Masking Collision Detection Threshold", 1.0, 18.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_end_phase_correlation_threshold", "Sub-Bass Minimum Monocompatibility Phase Floor", 0.5, 1.0, 0.85, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("auto_suggest_eq_cuts", "Generate Intelligent Parametric EQ Cut Offsets", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("visual_masking_heatmap", "Render 2D Multi-Track Energy Collision Heatmap", true, (34, 197, 94)))
        ),
        "AudioPhaseAligner" => Box::new(
            GenericDspNodeUi::new("AudioPhaseAligner", "Sub-Sample Audio Phase Cross-Correlator & Aligner HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("max_correlation_search_window_ms", "Cross-Correlation Maximum Time Search Window", 1.0, 50.0, 15.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("sub_sample_fir_sinc_interpolation", "Sub-Sample Sinc Interpolation Precision", true, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("phase_inversion_flip", "Invert Signal 180-Degree Absolute Polarity", false, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("allpass_dispersion_rotation_deg", "Frequency-Dependent Allpass Phase Rotation", 0.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("measured_delay_time_samples", "Detected Acoustic Time-of-Flight Latency", -1000.0, 1000.0, 0.0, "samples", (34, 197, 94)))
        ),
        "AudioProjectNormalizer" => Box::new(
            GenericDspNodeUi::new("AudioProjectNormalizer", "True-Peak Headroom & Project Gain Normalization HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_loudness_lufs", "Target Integrated Loudness Level", -24.0, -6.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("true_peak_ceiling_dbfs", "Maximum Intersample True-Peak Ceiling", -3.0, 0.0, -1.0, "dBTP", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("bit_depth_requantize", "Output Word Length & Re-quantization", vec!["32-bit Float (Preserve)".into(), "24-bit Fixed (Studio Standard)".into(), "16-bit Fixed (Red Book CD)".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("dither_algorithm", "Re-quantization Dither & Noise Shaping", vec!["None (Truncate)".into(), "TPDF Triangular".into(), "Noise Shaped Lipshitz".into(), "Psychoacoustic High-Pass".into()], 1, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("batch_normalize_all_stems", "Apply Matched Gain Staging Across All Stems", true, (34, 197, 94)))
        ),
        "PerceptualQualityVerifier" => Box::new(
            GenericDspNodeUi::new("PerceptualQualityVerifier", "ITU-R BS.1387 PEAQ Perceptual Audio Quality Verifier HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_peaq_objective_difference_grade", "Minimum Target PEAQ Difference Grade (ODG)", -4.0, 0.0, -0.2, "ODG", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("perceptual_noise_floor_masking_db", "Psychoacoustic Masking Noise Threshold", -120.0, -40.0, -84.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_flux_harmonic_distortion_max", "Total Harmonic Distortion (THD+N) Tolerance", 0.001, 1.0, 0.05, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("generate_peaq_difference_spectrogram", "Render 2D Difference Spectrogram Heatmap", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("strict_compliance_pass_fail", "Enforce Strict Automated Pass/Fail Verdict", true, (34, 197, 94)))
        ),
        "InputOutputGainTrim" => Box::new(
            GenericDspNodeUi::new("InputOutputGainTrim", "Dual-Stage Pre/Post Track Channel Gain Trim & Headroom Attenuator HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("pre_dsp_input_gain_trim_db", "Pre-DSP Channel Input Gain Trim", -24.0, 24.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("post_dsp_output_gain_trim_db", "Post-DSP Channel Output Gain Trim", -24.0, 24.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("gain_stage_linking_mode", "Dual-Stage Gain Linking Coupling", vec!["Independent Trim".into(), "Inverse Gain Match (Pre + Post = 0dB)".into(), "Input Mute Safe".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("analog_headroom_soft_clip", "Analog Headroom Soft Clipping Profile", vec!["Off (Linear Clean)".into(), "Gentle Analog Warmth (-0.5dB)".into(), "Tape Soft Saturation (-1.0dB)".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_gain_staging_bypass", "Bypass Gain Staging Trim Circuit", false, (34, 197, 94)))
        ),
        "TransientShaperMatrix" => Box::new(
            GenericDspNodeUi::new("TransientShaperMatrix", "Multi-Band Attack & Sustain Transient Sculpting Matrix HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("attack_gain_db", "Initial Attack Transient Punch Gain", -24.0, 24.0, 3.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sustain_gain_db", "Late Ringing Sustain Tail Level Gain", -24.0, 24.0, -2.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("low_band_crossover_hz", "Low-to-Mid Transient Frequency Crossover Split", 40.0, 500.0, 160.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_log("high_band_crossover_hz", "Mid-to-High Transient Frequency Crossover Split", 1000.0, 12000.0, 4500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_detection_sensitivity", "Wavelet-Based Transient Onset Detection Threshold", 0.0, 1.0, 0.75, "%", (34, 197, 94)))
        ),
        "DitherNoiseShaper" => Box::new(
            GenericDspNodeUi::new("DitherNoiseShaper", "Psychoacoustic Lipshitz Noise-Shaping & TPDF Dither HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_enum("target_bit_depth", "Destination Word Length Bit Truncation Target", vec!["16-bit Red Book CD".into(), "24-bit Studio Master".into(), "12-bit Vintage Sampler".into(), "8-bit Lo-Fi Arcade".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("noise_shaping_curve", "Noise Shaping Spectral Contour & Psychoacoustic Weighting", vec!["Flat Triangular TPDF (Zero Coloration)".into(), "Lipshitz Psychoacoustic Optimal".into(), "Modified E-Weighted High-Pass".into(), "Ultra High-Order F-Weighting".into()], 1, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("auto_blank_digital_silence", "Auto-Mute Dither Noise Floor During Zero Audio Signal", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dither_amplitude_lsb", "Dither Injection Level in Least Significant Bits", 0.5, 2.0, 1.0, "LSB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("anti_aliasing_dither_filter", "Engage Ultrasonic Reconstruction Anti-Aliasing Filter", true, (34, 197, 94)))
        ),
        "SidechainSpectralDucker" => Box::new(
            GenericDspNodeUi::new("SidechainSpectralDucker", "Spectral Multi-Band Dynamic Sidechain Frequency Ducker HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ducking_depth_db", "Maximum Attenuation Cut Applied at Masking Frequencies", 0.0, 30.0, 9.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("sidechain_channel_select", "Trigger Signal Routing Input Channel Source", vec!["Sidechain Bus 1 (Kick Key)".into(), "Sidechain Bus 2 (Snare Key)".into(), "Sidechain Bus 3 (Lead Vocal)".into(), "Aux External Key Input".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("low_frequency_anchor_hz", "Sub-Bass Spectral Carving Corner Frequency", 30.0, 300.0, 80.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("recovery_release_speed_ms", "Spectral Ducker Envelope Recovery Duration", 5.0, 500.0, 45.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("listen_spectral_delta", "Audition Isolated Subtracted Audio Delta Difference", false, (34, 197, 94)))
        ),
        "LinearPhaseDeEsser" => Box::new(
            GenericDspNodeUi::new("LinearPhaseDeEsser", "Split-Band Linear-Phase High-Frequency Sibilance Suppressor HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_log("sibilance_detection_freq_hz", "Sibilance Detector Center Tuning Frequency", 3000.0, 12000.0, 6500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sibilance_reduction_threshold_db", "De-Esser Gain Reduction Activation Threshold", -40.0, 0.0, -16.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("gain_reduction_limit_db", "Maximum Allowed High-Frequency Sibilance Clamp Limit", -24.0, 0.0, -8.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("deesser_filter_bandwidth_q", "Detector Bandpass Sibilance Sensitivity Bandwidth", 0.5, 5.0, 1.8, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("solo_sibilant_band", "Solo Extracted Sibilant Band for Accurate Center Frequency Tuning", false, (34, 197, 94)))
        ),
        "StereoGoniometerVisualizer" => Box::new(
            GenericDspNodeUi::new("StereoGoniometerVisualizer", "Lissajous Vector Scope & Stereo Phase Alignment HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("display_gain_scale", "Lissajous Vector Beam Amplitude Scaling", 0.5, 4.0, 1.2, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("phosphor_persistence_ms", "Virtual Oscilloscope CRT Phosphor Decay Time", 20.0, 1000.0, 180.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("vector_scope_rotation_mode", "Vector Scope Axis Orientation & Geometry", vec!["45° Diamond Scope (Left/Right)".into(), "90° Cartesian Scope (X/Y)".into(), "Polar Spatial Imager".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_correlation_alert_threshold", "Negative Phase Correlation Warning Alert Boundary", -1.0, 0.0, -0.2, "coeff", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("mono_fold_sum_check", "Sum Audio Channels to Mono to Inspect Comb-Filtering Cancellation", false, (34, 197, 94)))
        ),
        "MasterSafetyClipper" => Box::new(
            GenericDspNodeUi::new("MasterSafetyClipper", "True-Peak Master Safety Brickwall Clipper & DC Offset Barrier HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("brickwall_ceiling_dbtp", "Maximum Hard Safety Ceiling True-Peak Level", -6.0, 0.0, -0.1, "dBTP", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("clipper_knee_softness", "Hyperbolic Soft-Clipping Knee Transition Width", 0.0, 6.0, 1.2, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("dc_offset_highpass_cutoff_hz", "Sub-Audible DC Blocking High-Pass Filter Corner", 5.0, 40.0, 15.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("auto_mute_on_infinite_nan", "Auto-Mute Master Bus If Inf or NaN Sample Values Detected", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("emergency_attenuation_guard_db", "Emergency Headroom Pad Attenuation Level", -24.0, 0.0, 0.0, "dB", (34, 197, 94)))
        ),
        "MaskingReport" => Box::new(
            GenericDspNodeUi::new("MaskingReport", "AI Spectral Masking & Collision Diagnostic HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("overall_masking_score", "Overall Masking Overlap Dissonance Score", 0.0, 1.0, 0.25, "score", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("conflict_center_freq_hz", "Dominant Frequency Masking Conflict Center", 20.0, 16000.0, 250.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("conflict_bandwidth_q", "Psychoacoustic Masking Critical Bandwidth", 0.5, 8.0, 1.4, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("recommended_eq_dip_db", "Recommended Dynamic EQ Carve Attenuation", -18.0, 0.0, -3.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("temporal_overlap_pct", "Temporal Collision Duty Cycle Overlap", 0.0, 100.0, 35.0, "%", (34, 197, 94)))
        ),
        "AlignmentResult" => Box::new(
            GenericDspNodeUi::new("AlignmentResult", "Multi-Track Phase Alignment & Polarity Match HUD", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("phase_delay_samples", "Phase Delay Offset in Samples", -512.0, 512.0, 0.0, "samples", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("phase_offset_ms", "Phase Shift Sub-Millisecond Time Delay", -15.0, 15.0, 0.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cross_correlation_score", "Normalized Cross-Correlation Peak Alignment", 0.0, 1.0, 0.98, "corr", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("polarity_flip", "Invert Polarity 180 Degrees for Cancellation Check", false, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sample_rate_hz", "Reference Alignment Sample Rate", 44100.0, 192000.0, 48000.0, "Hz", (34, 197, 94)))
        ),
        "ControlEvent" => Box::new(
            GenericDspNodeUi::new("ControlEvent", "Dynamic Sidechain Ducking & Control Matrix Message", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("frame_offset", "Sample Frame Sub-Block Offset", 0.0, 2048.0, 0.0, "frames", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("value", "Sidechain Control Peak Level", -60.0, 12.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("source_id", "Source Track Key Channel Index", 0.0, 64.0, 1.0, "ch", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("attack_time_ms", "Detector Attack Smoothing Response", 0.1, 100.0, 2.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("hold_samples", "Peak Hold Gate Duration", 0.0, 4800.0, 128.0, "samples", (34, 197, 94)))
        ),
        "SidechainBusId" => Box::new(
            GenericDspNodeUi::new("SidechainBusId", "Sidechain Compressor Multi-Bus Routing Matrix Slot", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("bus_index", "Auxiliary Send Channel Bus Index", 0.0, 32.0, 1.0, "bus", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("gain_db", "Sidechain Send Input Gain Trim", -48.0, 18.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("highpass_cutoff_hz", "Detector High-Pass Key Filter", 20.0, 2000.0, 120.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_log("lowpass_cutoff_hz", "Detector Low-Pass Key Filter", 500.0, 20000.0, 8000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("listen_sidechain", "Solo & Audition Key Detector Signal", false, (34, 197, 94)))
        ),
        "RenderResult" => Box::new(
            GenericDspNodeUi::new("RenderResult", "DSP Render Block Performance & Signal Metrics Analyzer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("peak_db", "Rendered Audio True Peak Level", -96.0, 6.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("rms_db", "Rendered Audio RMS Energy Level", -96.0, 0.0, -14.0, "dBFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lufs_integrated", "EBU R128 Integrated Loudness", -48.0, -6.0, -14.0, "LUFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("underruns", "Real-Time Audio Buffer Underrun Count", 0.0, 100.0, 0.0, "count", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("render_time_ms", "Block Computation Execution Latency", 0.01, 20.0, 1.25, "ms", (34, 197, 94)))
        ),
        "DynamicsBandIndex" => Box::new(
            GenericDspNodeUi::new("DynamicsBandIndex", "Multi-Band Dynamics Crossover Band Frequency Indexing", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("crossover_slope_order", "Linkwitz-Riley Crossover Filter Slope", 0.0, 2.0, 1.0, "slope", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_mid_crossover_hz", "Low to Mid Crossover Frequency", 50.0, 500.0, 140.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("mid_high_crossover_hz", "Mid to High Crossover Frequency", 1000.0, 10000.0, 3200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("active_band_solo_mute_mask", "Selected Band Solo / Mute Routing", 0.0, 4.0, 0.0, "mask", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sidechain_band_coupling", "Cross-Band Dynamic Sidechain Leakage", 0.0, 1.0, 0.20, "leak", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_linear_phase_crossover", "Linear-Phase FIR Spectral Splitting", false, (14, 165, 233)))
        ),
        "DitherType" => Box::new(
            GenericDspNodeUi::new("DitherType", "Mastering Triangular PDF & Psychoacoustic Noise-Shaped Dither", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("dither_noise_algorithm", "Quantization Noise Shaping Profile", 0.0, 2.0, 1.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("target_bit_depth", "Target Quantization Bit Resolution", 0.0, 2.0, 0.0, "bits", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("noise_shaping_curve_order", "Psychoacoustic Noise Filter Order", 1.0, 5.0, 3.0, "ord", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_contour_shelf_db", "High-Frequency Ultrasonic Noise Shelf", -12.0, 12.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dither_level_lsb_amplitude", "Dither Peak Injection Amplitude", 0.5, 2.0, 1.0, "LSB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_auto_blanking_on_silence", "Auto-Mute Dither Noise During Absolute Silence", true, (14, 165, 233)))
        ),
        "KSystemScale" => Box::new(
            GenericDspNodeUi::new("KSystemScale", "Bob Katz K-System Broadcast Loudness & Dynamic Range Metering", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("k_meter_standard", "Bob Katz K-Meter Headroom Calibration", 0.0, 2.0, 1.0, "scale", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("target_headroom_db", "Target Dynamic Range Headroom", 6.0, 24.0, 14.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("meter_ballistics_integration_ms", "RMS Integration Window Ballistics", 50.0, 1000.0, 300.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("red_zone_warning_threshold_db", "Over-Compression Warning Threshold", -3.0, 6.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spl_acoustic_reference_db", "Calibrated Monitor Listening Level", 60.0, 95.0, 83.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_true_peak_intersample_detection", "4x Oversampled True Peak Inter-Sample Metering", true, (14, 165, 233)))
        ),
        "BitDepth" => Box::new(
            GenericDspNodeUi::new("BitDepth", "Master Audio Export Word Length & Quantization Bit Depth", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("quantization_word_length", "Audio File Encoding Word Length Bits", 0.0, 3.0, 1.0, "bits", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("noise_shaping_curve_order", "Psychoacoustic Noise-Shaping Filter Order", 0.0, 3.0, 1.0, "order", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dither_amplitude_lsb", "Triangular Dither PDF Amplitude in LSBs", 0.5, 2.0, 1.0, "LSB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("inter_sample_peak_headroom_db", "True Peak Inter-Sample Headroom Guard", -3.0, 0.0, -0.3, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dc_offset_rejection_hz", "Highpass DC Offset Rejection Cutoff", 1.0, 20.0, 5.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_tpdf_triangular_dither", "Enable High-Pass Shaped TPDF Dithering", true, (14, 165, 233)))
        ),
        "StemExportFormat" => Box::new(
            GenericDspNodeUi::new("StemExportFormat", "Multi-Track Stem Batch Audio Encoding & Export Container", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("audio_container_format", "Audio File Encoding Format Container", 0.0, 3.0, 0.0, "fmt", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sample_rate_export_hz", "Stem Export Resampling Target Rate", 44100.0, 192000.0, 48000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("flac_compression_level", "Lossless FLAC Encoding Compression Ratio", 0.0, 8.0, 5.0, "level", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("mp3_cbr_bitrate_kbps", "MP3 Constant Bitrate Stream Bandwidth", 128.0, 320.0, 320.0, "kbps", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("true_peak_margin_headroom_db", "Export Brickwall Limiter Margin Ceiling", -3.0, 0.0, -1.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_individual_stem_normalization", "Individually Normalize Stems to -14 LUFS", true, (14, 165, 233)))
        ),
        "BitDepthDitherQuantizer" => Box::new(
            GenericDspNodeUi::new("BitDepthDitherQuantizer", "TPDF Psychoacoustic Noise-Shaped Dither & Wordlength Reducer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_wordlength_bits", "Target Wordlength Bit Depth", 0.0, 3.0, 1.0, "bits", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("noise_shaping_curve", "Psychoacoustic Noise Shaping", 0.0, 3.0, 1.0, "shape", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dither_amplitude_lsb", "Dither Noise Amplitude", 0.5, 4.0, 2.0, "LSB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("auto_mute_dither_on_digital_black", "Auto-Mute on Digital Silence", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("dc_dither_bias_guard", "DC Bias Rejection Guard", true, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("vintage_quantization_distortion", "Emulate Vintage ADC Truncation", false, (14, 165, 233)))
        ),
        "LoudnessEbuR128Normalizer" => Box::new(
            GenericDspNodeUi::new("LoudnessEbuR128Normalizer", "ITU-R BS.1770-4 & EBU R128 Dual-Stage Loudness Normalizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Integrated Loudness", -24.0, -8.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("maximum_true_peak_dbfs", "Maximum Intersample True Peak", -3.0, 0.0, -1.0, "dBFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("streaming_target_preset", "Broadcasting & Streaming Profile", 0.0, 4.0, 0.0, "prof", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_loudness_range_expansion", "Allow Dynamic Loudness Expansion", false, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gating_threshold_absolute_lufs", "Absolute Gating Threshold", -70.0, -50.0, -70.0, "LUFS", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("apply_brickwall_limiter_catch", "Apply Safety True-Peak Catch Limiter", true, (14, 165, 233)))
        ),
        "TruePeakIntersampleLimiter" => Box::new(
            GenericDspNodeUi::new("TruePeakIntersampleLimiter", "4x Oversampled True-Peak Brickwall Intersample Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_threshold_dbfs", "True-Peak Ceiling Limit", -6.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("lookahead_time_ms", "Lookahead Buffer Time", 0.5, 10.0, 2.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("release_time_constant_ms", "Limiter Release Envelope Time", 5.0, 500.0, 50.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("oversampling_rate", "Polyphase Oversampling Multiplier", 0.0, 2.0, 0.0, "over", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_character_mode", "Transient Saturation Character", 0.0, 3.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("auto_release_envelope", "Adaptive Auto-Release Envelope", true, (14, 165, 233)))
        ),
        "DynamicMultibandGate" => Box::new(
            GenericDspNodeUi::new("DynamicMultibandGate", "4-Band Spectral Transient Noise Gate & Drum Bleed Reducer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("gate_crossover_low_hz", "Low / Low-Mid Crossover Cutoff", 60.0, 300.0, 120.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("gate_crossover_mid_hz", "Low-Mid / High-Mid Crossover", 500.0, 3000.0, 1500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("gate_crossover_high_hz", "High-Mid / Air Crossover", 3000.0, 12000.0, 6000.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("low_band_threshold_db", "Low Band Kick Gate Threshold", -60.0, 0.0, -28.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_band_bleed_attenuation_db", "Treble Band Max Bleed Attenuation", -60.0, 0.0, -18.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_transient_lookahead_buffer", "2 ms Anticipatory Lookahead Buffer", true, (14, 165, 233)))
        ),
        "NonlinearHammerStrike" => Box::new(
            GenericDspNodeUi::new("NonlinearHammerStrike", "Nonlinear Felt/Wood Hammer Strike & Hysteresis Impact Engine", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("hammer_felt_elasticity_exponent", "Nonlinear Felt Compression Exponent", 1.5, 3.8, 2.6, "exp", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("hammer_mass_grams", "Striking Hammer Effective Mass", 1.0, 30.0, 8.5, "g", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("felt_hysteresis_damping", "Viscoelastic Hysteresis Damping Loss", 0.01, 0.8, 0.25, "hyst", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("strike_velocity_m_s", "Keybed Dynamic Strike Velocity", 0.1, 8.0, 2.2, "m/s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("string_strike_position_ratio", "String Fractional Strike Location", 0.05, 0.5, 0.125, "pos", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("tangent_rebound_clearance_ms", "Escapement Rebound Clearance Time", 0.5, 20.0, 3.5, "ms", (14, 165, 233)))
        ),
        "AdaptiveNeuralDeEsser" => Box::new(
            GenericDspNodeUi::new("AdaptiveNeuralDeEsser", "Predictive Psychoacoustic Neural Sibilance Suppressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sibilance_detection_threshold_db", "Sibilance Detection Threshold", -48.0, 0.0, -18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("adaptive_crossover_frequency_hz", "Dynamic Crossover Split Point", 3000.0, 12000.0, 6800.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("maximum_suppression_limit_db", "Maximum Sibilance Gain Reduction", -36.0, 0.0, -10.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lookahead_prediction_ms", "Neural Predictive Lookahead Window", 0.0, 10.0, 2.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_envelope_retention", "Formant Envelope Naturalness Retention", 0.0, 1.0, 0.8, "ratio", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_delta_sibilance_solo", "Audition Removed Sibilance Solo", false, (14, 165, 233)))
        ),
        "MultibandUpwardCompressor" => Box::new(
            GenericDspNodeUi::new("MultibandUpwardCompressor", "Three-Band Upward/Downward Dynamic OTT Detail Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("upward_compression_depth_db", "Upward Compression Boost Depth", 0.0, 36.0, 12.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("downward_limiting_ratio", "Downward Limiting Compression Ratio", 1.0, 20.0, 4.0, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_mid_crossover_hz", "Bass to Mid Crossover Frequency", 50.0, 800.0, 180.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("mid_high_crossover_hz", "Mid to Treble Crossover Frequency", 800.0, 10000.0, 2800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("band_time_constant_preset", "Dynamic Time Constants Profile", 0.0, 3.0, 0.0, "preset", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("global_dynamics_mix_ratio", "Parallel Dry/Wet Compression Mix", 0.0, 1.0, 0.65, "mix", (14, 165, 233)))
        ),
        "SubharmonicTransientExciter" => Box::new(
            GenericDspNodeUi::new("SubharmonicTransientExciter", "Phase-Aligned Transient-Triggered Sub-Bass Synthesizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_tuning_hz", "Synthesized Sub-Bass Frequency", 20.0, 90.0, 48.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_detection_threshold_db", "Kick Transient Detector Threshold", -36.0, 0.0, -12.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_pulse_decay_ms", "Sub-Bass Pulse Decay Envelope", 20.0, 800.0, 180.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("fundamental_phase_sync_deg", "Phase Lock Angle to Transient", 0.0, 360.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("harmonic_drive_saturation", "Sub Overdrive Saturation Drive", 0.0, 1.0, 0.18, "drive", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("direct_sub_clean_mix", "Direct Sub-Bass Output Mix", 0.0, 1.5, 0.75, "gain", (14, 165, 233)))
        ),
        "TruePeakOversampledLimiter" => Box::new(
            GenericDspNodeUi::new("TruePeakOversampledLimiter", "8x Polyphase FIR True-Peak Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_true_peak_db", "True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("threshold_input_gain_db", "Limiter Input Drive Gain", 0.0, 24.0, 4.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lookahead_time_ms", "Transient Lookahead Horizon", 0.1, 10.0, 1.5, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("auto_release_profile", "Adaptive Auto-Release Curve", 0.0, 3.0, 1.0, "mode", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("oversampling_ratio", "FIR Oversampling Factor", 0.0, 3.0, 2.0, "mult", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_dither_tpdf", "TPDF Mastering Triangular Dither", true, (14, 165, 233)))
        ),
        "DynamicTransientDeEsser" => Box::new(
            GenericDspNodeUi::new("DynamicTransientDeEsser", "Adaptive Frequency-Tracking Dynamic Sibilance De-Esser", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("deesser_threshold_db", "Sibilance Detection Threshold", -48.0, 0.0, -22.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sibilance_center_frequency_hz", "Sibilant Center Frequency", 3000.0, 14000.0, 6800.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attenuation_reduction_depth_db", "Maximum Sibilance Gain Reduction", 0.0, 24.0, 8.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tracking_bandwidth_octaves", "Sidechain Filter Bandwidth", 0.2, 2.5, 0.8, "oct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("deess_attack_time_ms", "Fast Attack Response Time", 0.1, 20.0, 1.2, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("sidechain_audition_solo", "Audition Sidechain Filter Band Solo", false, (14, 165, 233)))
        ),
        "SubharmonicBassReinforcer" => Box::new(
            GenericDspNodeUi::new("SubharmonicBassReinforcer", "Phase-Locked Sub-Octave Fundamental Synthesizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("subharmonic_gain_db", "Synthesized Sub-Bass Level", -36.0, 12.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_octave_interval", "Subharmonic Frequency Divider", 0.0, 2.0, 0.0, "intv", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("upper_cutoff_frequency_hz", "Sub-Bass Synthesis Low-Pass Filter", 40.0, 150.0, 85.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_lock_tracking_speed_ms", "Pitch Tracking Phase Lock Speed", 1.0, 30.0, 5.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamics_ducking_sensitivity", "Kick Drum Masking Ducking", 0.0, 1.0, 0.45, "duck", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("saturation_odd_harmonics_mix", "Sub-Bass Harmonic Drive Saturation", 0.0, 1.0, 0.25, "sat", (14, 165, 233)))
        ),
        "AdaptiveSpectralDenoiser" => Box::new(
            GenericDspNodeUi::new("AdaptiveSpectralDenoiser", "Psychoacoustic Multiband FFT Noise Profiler & Spectral Gate", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("noise_reduction_depth_db", "Spectral Denoising Attenuation Depth", 0.0, 40.0, 14.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("masking_threshold_bias_db", "Auditory Masking Sensitivity Offset", -12.0, 12.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothing_factor", "Temporal Spectral Envelope Smoothing", 0.0, 1.0, 0.65, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("fft_window_size_bins", "FFT Transform Resolution", 0.0, 3.0, 1.0, "bins", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("learn_noise_fingerprint", "Profile Background Stationary Noise", false, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_noise_residual_delta", "Audition Attenuated Noise Residual", false, (14, 165, 233)))
        ),
        "TruePeakMasteringClipper" => Box::new(
            GenericDspNodeUi::new("TruePeakMasteringClipper", "Polyphase Oversampled Inter-Sample Peak Hard/Soft Clipper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_threshold_dbfs", "Brickwall Clipping Peak Ceiling", -6.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("clipper_input_gain_db", "Pre-Clipper Input Drive Gain", 0.0, 18.0, 3.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("soft_knee_radius_db", "Clipping Transition Soft-Knee Radius", 0.0, 6.0, 1.2, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("clipper_transfer_curve", "Nonlinear Clipping Transfer Curve", 0.0, 3.0, 0.0, "curve", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("oversampling_polyphase_factor", "Polyphase Oversampling Multiplier", 0.0, 3.0, 2.0, "x", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("dither_noise_shaping_curve", "Mastering Dither & Noise Shaping", 0.0, 3.0, 2.0, "dith", (14, 165, 233)))
        ),
        "HarmonicSubBassExciter" => Box::new(
            GenericDspNodeUi::new("HarmonicSubBassExciter", "Pitch-Tracking Subharmonic Waveform Generator & Bass Enhancer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("subharmonic_octave_blend_db", "Sub-Octave Synthesizer Blend Level", -30.0, 6.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("direct_fundamental_reinforce_db", "Original Fundamental Presence Boost", -12.0, 12.0, 2.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("tracking_frequency_cutoff_hz", "Sub-Bass Tracking Low-Pass Ceiling", 40.0, 200.0, 90.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonic_waveform_shape", "Synthesized Sub-Harmonic Waveform", 0.0, 3.0, 0.0, "wave", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamics_envelope_attack_ms", "Subharmonic Follower Attack Time", 0.5, 50.0, 8.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("dynamics_envelope_release_ms", "Subharmonic Follower Release Time", 10.0, 500.0, 120.0, "ms", (14, 165, 233)))
        ),
        "OpticalVactrolCompressor" => Box::new(
            GenericDspNodeUi::new("OpticalVactrolCompressor", "Dual-Time-Constant Optoelectronic Vactrol Leveling Amplifier", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("peak_reduction_drive_db", "Optical Cell Peak Reduction Drive", 0.0, 40.0, 18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("output_make_up_gain_db", "Tube Make-Up Amplifier Gain", 0.0, 30.0, 8.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio_mode", "Dynamic Compression Ratio Characteristic", 0.0, 1.0, 0.0, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vactrol_photocell_decay_memory", "Photoresistor Chemical Memory Lag", 0.0, 1.0, 0.65, "mem", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("emphasis_hf_sidechain_filter_db", "High-Frequency Sidechain Pre-Emphasis", 0.0, 12.0, 4.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("tube_output_stage_warmth", "12AX7 Vacuum Tube Output Saturation", 0.0, 1.0, 0.35, "warm", (14, 165, 233)))
        ),
        "DynamicLoudnessTargetLeveler" => Box::new(
            GenericDspNodeUi::new("DynamicLoudnessTargetLeveler", "EBU R128 / ITU-R BS.1770-4 Automated Loudness Leveler", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_loudness_lufs", "Target Program Integrated Loudness", -30.0, -9.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("maximum_true_peak_dbfs", "Ceiling Inter-Sample Peak Limit", -6.0, 0.0, -1.0, "dBTP", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("leveler_speed_response", "Adaptive RMS/LUFS Gain Ride Speed", 0.1, 5.0, 1.0, "spd", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("maximum_boost_gain_db", "Maximum Allowable Upward Leveling Boost", 0.0, 18.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("maximum_cut_gain_db", "Maximum Allowable Downward Attenuation", 0.0, 24.0, 12.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("dialogue_gating_speech", "Apply ITU-R BS.1770-4 -10 LU Relative Gating", true, (14, 165, 233)))
        ),
        "MultibandTransientHarmonizer" => Box::new(
            GenericDspNodeUi::new("MultibandTransientHarmonizer", "4-Band Dynamic Transient vs. Sustain Harmonizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_band_transient_boost_db", "Bass Band Attack Transient Punch", -12.0, 12.0, 3.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_mid_sustain_warmth_db", "Low-Mid Sustain Harmonic Warmth Drive", 0.0, 18.0, 2.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_mid_transient_crisp_db", "High-Mid Transient Snap & Crack", -12.0, 12.0, 2.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_band_air_sustain_db", "Air Band Shimmer & Sustain Extension", -12.0, 12.0, 1.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_detection_sensitivity", "Envelope Differential Attack Sensitivity", 0.1, 2.0, 1.0, "sens", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_isolated_transients", "Solo Transient Signal Layer Across All Bands", false, (14, 165, 233)))
        ),
        "MasteringTruePeakDeClipper" => Box::new(
            GenericDspNodeUi::new("MasteringTruePeakDeClipper", "Inter-Sample True Peak Spectral De-Clipper & Reconstructor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("clipping_threshold_dbfs", "Flat-Top Digital Clipping Detection Threshold", -6.0, 0.0, -0.2, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("reconstruction_crest_headroom_db", "Dynamic Headroom Expansion Restoration", 0.0, 12.0, 3.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spline_interpolation_order", "Peak Reconstruction Interpolation Algorithm", 0.0, 3.0, 1.0, "alg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonic_decay_estimation", "High-Order Clipping Harmonic Attenuation", 0.0, 1.0, 0.7, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("audition_clipped_spikes_solo", "Solo Isolated Reconstructed Clipping Spikes", false, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("post_repair_true_peak_limit", "True-Peak Brickwall Inter-Sample Safety Guard", true, (14, 165, 233)))
        ),
        "SpectralTransientDeClicker" => Box::new(
            GenericDspNodeUi::new("SpectralTransientDeClicker", "Surgical Sub-Millisecond Spectral De-Clicker & Plosive Tamer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("click_sensitivity_threshold", "Click Detection Sensitivity Ratio", 0.1, 10.0, 3.5, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_burst_width_us", "Maximum Click Burst Duration", 20.0, 2000.0, 250.0, "us", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothing_decay_ms", "Autoregressive Repair Window", 0.5, 20.0, 3.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_bias_weight", "High Frequency Detection Weight", 0.0, 2.0, 1.2, "bias", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("crest_factor_headroom_db", "Permitted Crest Factor Headroom", 0.0, 12.0, 4.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_delta_clicks_solo", "Solo Removed Click Signal Delta", false, (14, 165, 233)))
        ),
        "MultichannelLoudnessRadarMeter" => Box::new(
            GenericDspNodeUi::new("MultichannelLoudnessRadarMeter", "EBU R128 / ITU-R BS.1770 Real-Time Loudness Radar Meter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Program Integrated Loudness", -30.0, -9.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("momentary_ceiling_lufs", "Momentary Over-Compression Ceiling", -18.0, -3.0, -9.0, "LUFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("loudness_range_target_lu", "Target Loudness Range (LRA) Bracket", 2.0, 20.0, 8.0, "LU", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("true_peak_hold_ms", "True-Peak Intersample Peak Hold Time", 500.0, 10000.0, 3000.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("radar_history_speed", "Radar History Refresh Interval", 0.1, 5.0, 1.0, "s", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_relative_gate_metering", "Apply ITU -10 LU Relative Gating", true, (14, 165, 233)))
        ),
        "TubeOptoMasterCompressor" => Box::new(
            GenericDspNodeUi::new("TubeOptoMasterCompressor", "Dual-Stage Vacuum Tube & Optoelectronic Mastering Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("compression_threshold_db", "Optical Compression Threshold", -36.0, 0.0, -16.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio", "Leveling Compression Ratio Slope", 1.2, 20.0, 3.5, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("opto_cell_memory_decay", "Optocell Memory Lag Release Time", 50.0, 2500.0, 350.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tube_harmonic_color_db", "Triode Plate Saturation Harmonic Drive", 0.0, 18.0, 4.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sidechain_hpf_cutoff_hz", "Detector Sidechain Highpass Filter", 20.0, 300.0, 90.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("stereo_sidechain_link", "Link Left/Right Optical Detectors", true, (14, 165, 233)))
        ),
        "DynamicSidechainSpectralDucker" => Box::new(
            GenericDspNodeUi::new("DynamicSidechainSpectralDucker", "Frequency-Selective Real-Time Sidechain Spectral Ducker", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sidechain_trigger_sensitivity_db", "Sidechain Trigger Detection Threshold", -40.0, 0.0, -18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("maximum_ducking_attenuation_db", "Maximum Spectral Carving Cut Depth", 0.0, 24.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attack_carving_speed_ms", "Spectral Ducking Attack Response Time", 0.5, 50.0, 5.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("release_recovery_speed_ms", "Spectral Restoration Release Time", 10.0, 500.0, 80.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_focus_center_hz", "Primary Masking Conflict Frequency", 40.0, 10000.0, 1200.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("listen_carved_masking_delta", "Audition Removed Masking Energy Delta", false, (14, 165, 233)))
        ),
        "VariMuMasterLimiter" => Box::new(
            GenericDspNodeUi::new("VariMuMasterLimiter", "Variable-Mu Valve Mastering Peak Limiter & Bus Glue", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_ceiling_db", "Variable-Mu Limiting Threshold", -24.0, 0.0, -10.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tube_recovery_time_ms", "Triode Discharge Recovery Time", 50.0, 2000.0, 250.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("detector_sidechain_hpf_hz", "Sidechain High-Pass Bass Filter", 20.0, 300.0, 80.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("master_glue_compression_ratio", "Peak Compression Dynamic Ratio", 1.5, 12.0, 4.0, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("output_makeup_gain_db", "Clean Transformer Makeup Gain", -6.0, 18.0, 2.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("dual_mono_independent_detectors", "Dual-Mono Independent Detector Cell", false, (14, 165, 233)))
        ),
        "TransientSustainDecomposer" => Box::new(
            GenericDspNodeUi::new("TransientSustainDecomposer", "Dual-Path Transient & Sustained Harmonic Splitter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("transient_detection_sensitivity", "Transient Attack Detection Sensitivity", 0.1, 10.0, 2.0, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_path_gain_db", "Isolated Transient Stream Gain", -24.0, 12.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sustain_path_gain_db", "Isolated Sustained Body Gain", -24.0, 12.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("smoothing_transition_time_ms", "Stream Crossfade Transition Time", 1.0, 50.0, 8.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_channel_stereo_width", "Transient Layer Stereo Panorama Width", 0.0, 200.0, 100.0, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("solo_transient_layer_only", "Solo Isolated Transient Stream", false, (14, 165, 233)))
        ),
        "ParallelUpwardExpansionMaximizer" => Box::new(
            GenericDspNodeUi::new("ParallelUpwardExpansionMaximizer", "Parallel Upward Dynamic Expander & Micro-Dynamic Maximizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("expansion_threshold_floor_db", "Upward Expansion Detection Floor", -60.0, -18.0, -36.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("upward_expansion_ratio", "Low-Level Dynamic Expansion Slope", 1.1, 4.0, 1.8, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("micro_dynamics_boost_gain_db", "Micro-Detail Parallel Boost Gain", 0.0, 18.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("attack_envelope_window_ms", "Detector Window Attack Response", 0.5, 100.0, 15.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("release_recovery_envelope_ms", "Upward Gain Smooth Decay Release", 20.0, 1000.0, 180.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_expanded_component_solo", "Solo Isolated Low-Level Details", false, (14, 165, 233)))
        ),
        "OpticalTubeCompressorLimiter" => Box::new(
            GenericDspNodeUi::new("OpticalTubeCompressorLimiter", "Classic Electro-Optical Teletronix Attenuator & Valve Gain Cell", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("peak_reduction_attenuation_db", "Optical Peak Reduction Attenuation", 0.0, 40.0, 10.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("electro_optical_lag_hysteresis", "Photocell Memory Lag Hysteresis", 0.0, 1.0, 0.45, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("vacuum_tube_makeup_gain_db", "Triode Tube Makeup Amplifier Gain", 0.0, 30.0, 8.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sidechain_emphasis_freq_hz", "Sidechain Pre-Emphasis Center Frequency", 100.0, 10000.0, 1000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("opto_cell_response_mode", "Optical Response Speed Profile Mode", 0.0, 2.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("link_stereo_optical_detectors", "Stereo Optical Detector Cell Link", true, (14, 165, 233)))
        ),
        "TransientShaperDualBand" => Box::new(
            GenericDspNodeUi::new("TransientShaperDualBand", "Dual-Band Frequency-Split Transient & Sustain Envelope Shaper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("crossover_frequency_hz", "Bass / Treble Split Crossover", 80.0, 2000.0, 350.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_band_attack_gain_db", "Low Band Punch Attack Gain", -24.0, 24.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("low_band_sustain_gain_db", "Low Band Resonance Sustain Gain", -24.0, 24.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_band_attack_gain_db", "High Band Snap Attack Gain", -24.0, 24.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_band_sustain_gain_db", "High Band Room Sustain Gain", -24.0, 24.0, -2.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("clip_transient_peaks_soft_knee", "Soft-Knee Peak Clipper Safety Guard", true, (14, 165, 233)))
        ),
        "SubharmonicBassExciter" => Box::new(
            GenericDspNodeUi::new("SubharmonicBassExciter", "Analog Subharmonic Waveform Synthesizer & Low-End Bass Extender", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sub_fundamental_octave_blend", "1st Subharmonic (-1 Octave) Level", 0.0, 1.0, 0.5, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_second_octave_blend", "2nd Subharmonic (-2 Octaves) Level", 0.0, 1.0, 0.2, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("lowpass_isolation_cutoff_hz", "Sub Detection Lowpass Filter", 40.0, 250.0, 90.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dynamic_sub_drive_saturation", "Subharmonic Drive & Saturation", 0.0, 1.0, 0.35, "drive", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("bass_mono_fold_hz", "Low Bass Mono Summing Frequency", 30.0, 200.0, 100.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("phase_invert_subharmonics", "Phase-Invert Synthesized Subharmonics", false, (14, 165, 233)))
        ),
        "MasteringTruePeakDitherLimiter" => Box::new(
            GenericDspNodeUi::new("MasteringTruePeakDitherLimiter", "Integrated True-Peak Intersample Brickwall Limiter & Psychoacoustic Dither", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_true_peak_dbfs", "Master True-Peak Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("intersample_oversampling_ratio", "Oversampling True-Peak Factor", 1.0, 16.0, 8.0, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("release_dynamic_auto_recovery_ms", "Program-Adaptive Release Recovery Time", 5.0, 500.0, 45.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_noise_shaping_db", "F-Weighting Dither Noise Shaping Depth", 0.0, 18.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dither_target_bit_depth", "Target Quantization Export Bit Depth Mode", 0.0, 2.0, 0.0, "bits", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("compliance_itu_r_bs1770_check", "ITU-R BS.1770 True-Peak Compliance", true, (14, 165, 233)))
        ),
        "MultibandUpwardDownwardCompressor" => Box::new(
            GenericDspNodeUi::new("MultibandUpwardDownwardCompressor", "4-Band OTT-Style Upward/Downward Dynamic Range Squeezer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("crossover_low_hz", "Sub-Bass / Low-Mid Crossover", 40.0, 500.0, 120.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossover_high_hz", "High-Mid / Treble Crossover", 1500.0, 12000.0, 4500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("upward_expansion_depth_db", "Upward Compression Lift Gain", 0.0, 36.0, 14.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("downward_compression_ratio", "Downward Peak Compression Ratio", 1.0, 20.0, 6.0, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamics_response_time_ms", "Attack / Release Envelope Ballistics", 1.0, 100.0, 25.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_true_stereo_sidechain_link", "Stereo Detector Sidechain Linking", true, (14, 165, 233)))
        ),
        "MasterTapeSaturatorMaximizer" => Box::new(
            GenericDspNodeUi::new("MasterTapeSaturatorMaximizer", "Precision Mastering Tape Saturation, Head Bump & Loudness Maximizer", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("tape_flux_drive_level_db", "Magnetic Tape Flux Drive Gain", 0.0, 20.0, 4.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_head_bump_resonance_hz", "Reproduce Head Bump Bass Tuning", 30.0, 90.0, 55.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_tape_soften_khz", "High-Frequency Tape Roll-Off Knee", 10.0, 24.0, 18.5, "kHz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("even_odd_harmonic_saturation_blend", "Even vs Odd Harmonic Saturation Balance", -1.0, 1.0, 0.2, "bal", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("brickwall_output_margin_dbfs", "Master Output Brickwall Ceiling", -3.0, 0.0, -0.1, "dBFS", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_constant_loudness_monitoring", "Auto-Gain Loudness Matched Auditioning", true, (14, 165, 233)))
        ),
        "StereoDynamicDeClipper" => Box::new(
            GenericDspNodeUi::new("StereoDynamicDeClipper", "Adaptive Audio Forensic Waveform Restoration & Peak De-Clipper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("clip_detection_threshold_dbfs", "Clipping Flat-Top Detection Threshold", -6.0, 0.0, -0.2, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cubic_spline_reconstruction_sharpness", "Cubic Spline Polynomial Tension", 0.1, 2.0, 0.95, "tens", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dynamic_headroom_expansion_db", "Post-Restoration Headroom Attenuation", 0.0, 12.0, 3.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_edge_smoothing_ms", "Discontinuity Edge Smoothing Filter", 0.05, 5.0, 0.4, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("harmonic_aliasing_filter_mode", "Reconstruction Anti-Aliasing Oversampling Mode", 0.0, 2.0, 1.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_isolated_reconstructed_peaks", "Solo Reconstructed Peak Material (Delta)", false, (14, 165, 233)))
        ),
        "SonoluminescenceCavitationSonifier" => Box::new(
            GenericDspNodeUi::new("SonoluminescenceCavitationSonifier", "Acoustic Cavitation Bubble Picosecond Shockwave & Luminescence Sonifier", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("cavitation_drive_amplitude_db", "Acoustic Pressure Drive Intensity", 0.0, 36.0, 18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ambient_bubble_radius_microns", "Equilibrium Bubble Equilibrium Radius", 1.0, 50.0, 5.0, "um", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("liquid_viscous_damping_poise", "Liquid Medium Viscosity Damping", 0.005, 0.1, 0.01, "P", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("shockwave_hypersonic_compression", "Hypersonic In-Rushing Gas Shock Mach", 0.1, 5.0, 1.8, "Mach", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("luminescence_thermal_flash_target", "Core Quasi-Thermal Flash Temperature", 1000.0, 50000.0, 12000.0, "K", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_single_bubble_synchronous_mode", "Synchronous Single-Bubble Cavitation Mode", true, (14, 165, 233)))
        ),
        "AiAutonomousMasteringChain" => Box::new(
            GenericDspNodeUi::new("AiAutonomousMasteringChain", "Autonomous Multi-Stage AI Mastering Balance & Loudness Chain", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Master Integrated Loudness Target", -24.0, -6.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_profile_matching_strength", "Genre Spectral Reference Matching", 0.0, 1.0, 0.75, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dynamic_crest_expansion_factor", "Dynamic Crest Factor Transient Preserver", 0.5, 2.0, 1.0, "factor", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("stereo_image_clarity_enhancement", "Stereo Width & Correlation Enhancer", 0.0, 1.0, 0.35, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ceiling_true_peak_margin_db", "Brickwall True-Peak Output Ceiling", -3.0, 0.0, -0.8, "dBTP", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_intersample_oversampled_guard", "8x Oversampled Intersample Peak Sentry", true, (14, 165, 233)))
        ),
        "SpectralEntropyTransientGate" => Box::new(
            GenericDspNodeUi::new("SpectralEntropyTransientGate", "Spectral Entropy & Shannon Information Transient Dynamic Gate", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("entropy_gate_threshold", "Shannon Entropy Gate Threshold", 0.1, 1.0, 0.45, "entropy", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_crest_factor_bias", "High-Frequency Spectral Crest Weight", 0.0, 1.0, 0.7, "weight", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("gate_open_attack_time_ms", "Gate Response Trigger Attack Time", 0.1, 50.0, 2.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("gate_closure_hold_time_ms", "Gate Open Hold Duration", 5.0, 500.0, 45.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ambient_floor_attenuation_db", "Closed Gate Floor Attenuation Depth", -80.0, 0.0, -36.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_lookahead_transient_window", "Zero-Phase Lookahead Transient Detection", true, (14, 165, 233)))
        ),
        "OptoelectronicVactrolOptocoupler" => Box::new(
            GenericDspNodeUi::new("OptoelectronicVactrolOptocoupler", "Cadmium Sulfide (CdS) Dual-Vactrol Non-Linear Leveler", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("optocoupler_drive_level_db", "Optical LED Driver Excitation Level", -30.0, 20.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("photoresistor_dark_resistance_mohm", "Photoresistor Dark Resistance", 0.5, 50.0, 10.0, "MOhm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("light_history_memory_decay_s", "Photoconductive Memory Decay Lag", 0.05, 5.0, 0.8, "s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("optical_compression_ratio", "Optocoupler Compression Slope", 1.5, 20.0, 4.0, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vintage_vactrol_cell_type", "Vintage Vactrol Cell Selection", 0.0, 3.0, 0.0, "cell", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_parasitic_capacitance_lag", "Parasitic Junction Capacitance Filter", true, (14, 165, 233)))
        ),
        "EbuLoudnessAutoTargetLeveler" => Box::new(
            GenericDspNodeUi::new("EbuLoudnessAutoTargetLeveler", "EBU R128 / ITU-R BS.1770 Autonomous Broadcast Leveler", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Integrated Broadcast LUFS", -24.0, -9.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("loudness_range_lra_max_lu", "Maximum Permitted Loudness Range (LRA)", 2.0, 18.0, 7.0, "LU", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("momentary_lookahead_time_ms", "Momentary Loudness Lookahead Window", 5.0, 250.0, 50.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("maximum_recovery_rate_db_s", "Slow Automatic Gain Recovery Rate", 0.1, 5.0, 0.8, "dB/s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gating_relative_threshold_lu", "ITU-R BS.1770 Gating Silence Threshold", -20.0, -8.0, -10.0, "LU", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_k_weighting_psychoacoustic_filter", "Pre-Filter K-Weighting High-Shelf & HPF", true, (14, 165, 233)))
        ),
        "DynamicSpectralDeEsser" => Box::new(
            GenericDspNodeUi::new("DynamicSpectralDeEsser", "Phase-Neutral Spectral Sibilance Suppressor & Split-Band De-Esser", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("deesser_frequency_crossover_hz", "Sibilance Band High Crossover", 2000.0, 14000.0, 6200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sibilance_reduction_threshold_db", "Sibilance Dynamic Detection Threshold", -40.0, 0.0, -18.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attenuation_reduction_depth_db", "Maximum Sibilant Suppression Depth", 0.0, 24.0, 8.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_listen_monitoring_mode", "Sidechain Sibilance Band Audition Mode", 0.0, 1.0, 0.0, "mode", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("adaptive_release_recovery_speed_ms", "Adaptive Ballistics Recovery Release", 5.0, 150.0, 25.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_split_band_dynamic_ducking", "Split-Band Selective Frequency Ducking", true, (14, 165, 233)))
        ),
        "AdaptiveNoiseGateTransientExpander" => Box::new(
            GenericDspNodeUi::new("AdaptiveNoiseGateTransientExpander", "Adaptive Noise Gate & Transient Downward Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("gate_threshold_db", "Downward Expander Detection Threshold", -80.0, -10.0, -42.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("expansion_ratio", "Dynamic Downward Expansion Ratio", 1.0, 20.0, 4.0, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hysteresis_headroom_db", "Chatter Suppression Hysteresis Margin", 0.0, 12.0, 3.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("gate_lookahead_ms", "Optical Transient Lookahead Buffer", 0.0, 20.0, 2.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("release_envelope_decay_ms", "Smooth Envelope Decay Release Speed", 5.0, 1000.0, 80.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_bandpass_filter", "Critical Band Vocal Sidechain Isolation", true, (14, 165, 233)))
        ),
        "MasteringTruePeakLimiterOversampled" => Box::new(
            GenericDspNodeUi::new("MasteringTruePeakLimiterOversampled", "8x Oversampled Linear-Phase True Peak Brickwall Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_threshold_dbfs", "True-Peak Maximum Output Ceiling", -12.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("input_makeup_boost_gain_db", "Loudness Maximizer Makeup Boost", 0.0, 24.0, 4.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("limiter_lookahead_time_ms", "Inter-Sample Lookahead Window", 0.5, 15.0, 4.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("adaptive_release_time_ms", "Program-Dependent Adaptive Release", 10.0, 500.0, 65.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("intersample_peak_headroom_margin_db", "Codec Streaming ISP Safety Margin", 0.05, 1.0, 0.2, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_tpdf_triangular_dither", "Triangular Probability Density Dither", true, (14, 165, 233)))
        ),
        "LookaheadDynamicDeEsserSibilance" => Box::new(
            GenericDspNodeUi::new("LookaheadDynamicDeEsserSibilance", "Lookahead Dynamic Spectral De-Esser", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sibilance_center_frequency_hz", "Sibilance Detection Center Frequency", 3000.0, 12000.0, 6500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("deesser_threshold_dbfs", "Dynamic Attenuation Engagement Threshold", -40.0, 0.0, -18.0, "dBFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sibilance_reduction_ratio", "Downward Compression Ratio Slope", 1.5, 10.0, 4.0, ":1", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lookahead_buffer_time_ms", "Transient Lookahead Anticipation Time", 0.5, 10.0, 2.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sibilance_recovery_release_ms", "Adaptive Sibilance Gain Recovery Time", 5.0, 150.0, 35.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_audition_sidechain_monitor", "Solo Sibilance Detection Filter Path", false, (14, 165, 233)))
        ),
        "MultibandDownwardSpillExpander" => Box::new(
            GenericDspNodeUi::new("MultibandDownwardSpillExpander", "Multiband Downward Drum & Stage Spill Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_band_expansion_threshold_db", "Low Band Bleed Threshold", -60.0, -10.0, -36.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_band_expansion_threshold_db", "Mid Band Bleed Threshold", -60.0, -10.0, -32.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_band_expansion_threshold_db", "High Band Bleed Threshold", -60.0, -10.0, -28.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("expansion_ratio_attenuation_slope", "Downward Expansion Attenuation Ratio", 1.2, 8.0, 2.5, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("release_envelope_hysteresis_ms", "Hold & Release Smoothing Envelope", 10.0, 500.0, 60.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_frequency_weighting", "Vocal/Snare Sidechain Spectral Weighting", true, (14, 165, 233)))
        ),
        "DynamicSpectralCompressorDeEsser" => Box::new(
            GenericDspNodeUi::new("DynamicSpectralCompressorDeEsser", "Dynamic Multi-Band Spectral De-Esser", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sibilance_center_frequency_hz", "Sibilance Target Center Frequency", 3000.0, 14000.0, 6800.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sibilance_bandwidth_octaves", "Sibilance Detection Bandwidth", 0.5, 3.0, 1.2, "oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dynamic_reduction_threshold_db", "Dynamic Suppression Threshold", -40.0, 0.0, -18.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio", "Gain Reduction Compression Ratio", 1.0, 20.0, 4.5, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("detector_lookahead_ms", "Lookahead Transient Buffer Delay", 0.0, 10.0, 2.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_split_band_solo", "Solo Sibilance Sidechain Band", false, (14, 165, 233)))
        ),
        "MultibandTruePeakMasterLimiter" => Box::new(
            GenericDspNodeUi::new("MultibandTruePeakMasterLimiter", "Multiband Inter-Sample True-Peak Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("master_ceiling_level_dbfs", "Limiter True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("master_pre_gain_drive_db", "Master Input Threshold Pre-Gain", -6.0, 24.0, 3.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("release_recovery_time_ms", "Transient Envelope Release Recovery", 5.0, 500.0, 45.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("low_band_saturation_guard_db", "Sub-Bass Limiting Soft-Clip Saturation", 0.0, 6.0, 1.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("oversampling_order_factor", "Linear Phase Oversampling Rate (2x/4x/8x)", 2.0, 8.0, 4.0, "x", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_dither_noise_shaping", "TPDF Dither & Psychoacoustic Noise Shaping", true, (14, 165, 233)))
        ),
        "NonlinearOptocouplerOpticalCompressor" => Box::new(
            GenericDspNodeUi::new("NonlinearOptocouplerOpticalCompressor", "Electro-Optical Optocoupler Program-Dependent Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("peak_reduction_gain_db", "Optical Cell Peak Reduction", -60.0, 0.0, -18.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("luminescent_primary_release_ms", "Electroluminescent Primary Decay", 10.0, 500.0, 60.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("photocell_memory_lag_seconds", "CdS Photoconductive Memory Secondary Lag", 0.1, 10.0, 1.5, "sec", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("frequency_weighted_pre_emphasis_db", "HF Sidechain Pre-Emphasis Boost", 0.0, 12.0, 3.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("makeup_amplification_gain_db", "Tube Output Makeup Stage Gain", 0.0, 40.0, 12.0, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_program_dependent_dual_knees", "Historical Program-Dependent Dual Knee", true, (34, 197, 94)))
        ),
        "MultibandMidSideMasteringCompressor" => Box::new(
            GenericDspNodeUi::new("MultibandMidSideMasteringCompressor", "Tri-Band Mid-Side Precision Mastering Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("low_mid_crossover_frequency_hz", "Low-to-Mid Crossover Frequency", 60.0, 400.0, 140.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("mid_high_crossover_frequency_hz", "Mid-to-High Crossover Frequency", 1500.0, 8000.0, 3200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_channel_compression_ratio", "Mid Center Punch Compression Ratio", 1.0, 10.0, 2.2, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("side_channel_compression_ratio", "Side Ambience Compression Ratio", 1.0, 10.0, 1.8, ":1", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stereo_side_channel_gain_db", "Stereo Spatial Width Makeup Gain", -12.0, 6.0, 1.2, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_linear_phase_crossover_filter", "FIR Linear-Phase Crossover Lattice", true, (34, 197, 94)))
        ),
        "MultibandTransientKneeExpander" => Box::new(
            GenericDspNodeUi::new("MultibandTransientKneeExpander", "Quad-Band Downward Spill & Transient Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("expansion_ratio_slope", "Downward Expansion Ratio", 1.1, 8.0, 2.5, ":1", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("downward_threshold_db", "Global Expansion Floor Threshold", -60.0, -10.0, -32.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("quad_crossover_low_hz", "Low-to-Mid Crossover Split Frequency", 60.0, 500.0, 160.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("quad_crossover_high_hz", "Mid-to-High Crossover Split Frequency", 1500.0, 12000.0, 4500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("envelope_hold_duration_ms", "Transient Peak Hold Duration", 0.0, 100.0, 15.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_transient_lookahead_gate", "Zero-Distortion Lookahead Detection", true, (34, 197, 94)))
        ),
        "DualStageOpticalVactrolLimiter" => Box::new(
            GenericDspNodeUi::new("DualStageOpticalVactrolLimiter", "Dual-Cell Vactrol Optoisolator Fast Peak Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("vactrol_led_threshold_db", "Vactrol LED Conduction Threshold", -40.0, 0.0, -14.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("primary_peak_limiting_ratio", "Optical Compression Ratio", 2.0, 40.0, 10.0, ":1", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vactrol_photoresistor_memory_ms", "CdS Photoconductive Memory Decay", 50.0, 5000.0, 850.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("analog_tube_warmth_drive_db", "Post-Limiter Tube Makeup Drive", 0.0, 24.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("output_ceiling_brickwall_db", "Absolute Output Safety Ceiling", -12.0, 0.0, -0.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_fast_transient_led_shunting", "Ultra-Fast Shunt Diode Peak Protection", true, (34, 197, 94)))
        ),
        "MultibandUpwardDownwardDynamicExpander" => Box::new(
            GenericDspNodeUi::new("MultibandUpwardDownwardDynamicExpander", "Dual-Direction Multiband Dynamic Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("upward_expansion_ratio", "Upward Dynamic Expansion Ratio", 1.0, 4.0, 1.6, ":1", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("upward_expansion_threshold_db", "Upward Peak Threshold Level", -24.0, 0.0, -10.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("downward_gate_threshold_db", "Downward Noise Floor Threshold", -70.0, -20.0, -42.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("crossover_split_frequency_hz", "Mid-Band Boundary Crossover Frequency", 200.0, 5000.0, 1200.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hysteresis_chatter_protection_db", "Dynamic Gate Hysteresis Window", 0.5, 12.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_rms_detection", "True RMS Energy Detector Smoothing", true, (34, 197, 94)))
        ),
        "MasterStereoVariMuLimiter" => Box::new(
            GenericDspNodeUi::new("MasterStereoVariMuLimiter", "Stereo Variable-Mu Remote-Cutoff Tube Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_input_drive_db", "Vari-Mu Tube Grid Threshold Drive", -30.0, 20.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio_slope", "Progressive Compression Ratio", 1.5, 20.0, 4.0, ":1", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("attack_time_constant_ms", "Limiter Attack Response Speed", 0.1, 50.0, 12.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("release_time_constant_ms", "Program-Dependent Release Recovery", 50.0, 2500.0, 350.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sidechain_highpass_filter_hz", "Sidechain Bass Isolation Highpass", 20.0, 300.0, 90.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_dual_mono_unlinked_mode", "Unlinked Stereo Channel Detection", false, (34, 197, 94)))
        ),
        "ParallelMultibandTransientMasterLimiter" => Box::new(
            GenericDspNodeUi::new("ParallelMultibandTransientMasterLimiter", "Parallel 5-Band Transient Clamping Master Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("master_ceiling_margin_db", "True Peak Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_clip_threshold_db", "Inter-Band Transient Clip Threshold", -12.0, 0.0, -1.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("multiband_release_ballistics_ms", "Program-Adaptive Release Time", 5.0, 500.0, 45.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_band_crest_factor_clamp", "Sub-Bass Crest Factor Density", 0.5, 3.0, 1.2, "crest", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_link_coupling_percent", "Stereo Gain Reduction Linking", 0.0, 100.0, 85.0, "%", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_intersample_oversampling", "8x True-Peak Polyphase Oversampling", true, (34, 197, 94)))
        ),
        "DualDiodeBridgeCompressorLimiter" => Box::new(
            GenericDspNodeUi::new("DualDiodeBridgeCompressorLimiter", "Discrete Diode Bridge Vintage Dynamic Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("compressor_threshold_level_db", "Diode Bridge Gain Reduction Threshold", -40.0, 10.0, -14.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio_setting", "Compression Slope Ratio Factor", 1.5, 20.0, 4.0, ":1", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("compressor_attack_speed_us", "Ultra-Fast Transient Attack Time", 100.0, 10000.0, 1200.0, "us", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("compressor_auto_recovery_release_ms", "Dynamic Release Recovery Time", 50.0, 1500.0, 200.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("makeup_gain_level_db", "Post-Attenuation Output Makeup Gain", 0.0, 30.0, 8.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_highpass_filter", "120 Hz Sidechain Highpass Filter", true, (34, 197, 94)))
        ),
        "MultibandMasteringLinearPhaseLimiter" => Box::new(
            GenericDspNodeUi::new("MultibandMasteringLinearPhaseLimiter", "Mastering 5-Band Linear-Phase Peak Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("master_brickwall_ceiling_dbfs", "True-Peak Output Brickwall Ceiling", -6.0, 0.0, -0.3, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("limiter_threshold_input_drive_db", "Input Drive Maximizer Threshold", -18.0, 12.0, 0.0, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("multiband_spectral_balance_weight", "Low-High Dynamic Band Linking", 0.0, 1.0, 0.65, "link", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("adaptive_release_inertia_ms", "Adaptive Psychoacoustic Release Inertia", 10.0, 500.0, 85.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("inter_sample_oversampling_factor", "ISP True-Peak Oversampling Rate", 1.0, 8.0, 4.0, "x", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_auto_makeup_gain", "Auto Equal-Loudness Makeup Gain", true, (34, 197, 94)))
        ),
        "OpticalVactrolPeakMasterCompressor" => Box::new(
            GenericDspNodeUi::new("OpticalVactrolPeakMasterCompressor", "Vintage Optical Cadmium Sulfide Vactrol Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("vactrol_peak_reduction_gain_db", "Vactrol Peak Reduction Drive", 0.0, 40.0, 16.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("compression_ratio_slope", "Vactrol Dynamic Attenuation Ratio", 1.5, 20.0, 4.0, ":1", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cds_photoresistor_memory_decay_ms", "CdS Photoresistor Primary Decay Memory", 20.0, 1500.0, 320.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("secondary_slow_tail_lag_ms", "Secondary Deep Memory Tail Decay Lag", 100.0, 5000.0, 1800.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_emphasis_sidechain", "High-Frequency Sidechain Pre-Emphasis", 0.0, 1.0, 0.45, "emph", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_compressor_limiter_mode_switch", "Limiter vs Compressor Ratio Profile", false, (34, 197, 94)))
        ),
        "VariableMuDualSidechainMasterCompressor" => Box::new(
            GenericDspNodeUi::new("VariableMuDualSidechainMasterCompressor", "Mastering Variable-Mu Remote-Cutoff Tube Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("compressor_threshold_drive_db", "Input Drive Level Against Tube Threshold", -24.0, 12.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tube_variable_mu_ratio_slope", "Variable-Mu Smooth Compression Ratio Curvature", 1.2, 8.0, 2.5, ":1", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("program_dependent_attack_ms", "Fast Transient Ballistic Attack Time", 0.5, 100.0, 12.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dual_stage_release_recovery_ms", "Dual-Stage Slow Program Release Recovery", 50.0, 2500.0, 380.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sidechain_highpass_shelf_hz", "Sidechain Bass Attenuation High-Pass Shelf", 20.0, 300.0, 90.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_mid_side_unlinked_detection", "Mid/Side Dual Unlinked Sidechain Detection", true, (34, 197, 94)))
        ),
        "DynamicTransientMultiBandPeakClipper" => Box::new(
            GenericDspNodeUi::new("DynamicTransientMultiBandPeakClipper", "Linear-Phase 4-Band Dynamic Mastering Peak Clipper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("master_ceiling_threshold_dbfs", "Master True-Peak Ceiling Clip Threshold", -6.0, 0.0, -0.2, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("low_band_clip_knee_softness", "Sub-Bass Soft Saturation Knee Rounding", 0.01, 1.0, 0.45, "knee", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("mid_band_transient_punch_boost", "Mid-Band Transient Snap Punch Preservation", 0.0, 6.0, 1.8, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_band_spectral_crest_protection", "High-Frequency Splash Harmonic Limiting", 0.0, 1.0, 0.72, "prot", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("linear_phase_crossover_steepness_db", "Linear-Phase FIR Crossover Filter Slope", 12.0, 96.0, 48.0, "dB/oct", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_sixteen_times_isp_oversampling", "16x Polyphase Inter-Sample Peak Oversampling", true, (34, 197, 94)))
        ),
        "OpticalVactrolFeedbackMasterLimiter" => Box::new(
            GenericDspNodeUi::new("OpticalVactrolFeedbackMasterLimiter", "Precision Optical Vactrol Program-Adaptive Master Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_true_peak_limit_dbfs", "Master Output True-Peak Ceiling Limit", -6.0, 0.0, -0.3, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("input_drive_gain_db", "Limiter Threshold Input Drive Boost", 0.0, 18.0, 4.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("photocell_release_memory_inertia_ms", "Vactrol Photocell Cadmium-Sulfide Memory Inertia", 20.0, 1500.0, 140.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("lookahead_transient_buffer_ms", "Linear-Phase Transient Lookahead Buffer Delay", 0.5, 10.0, 3.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_sidechain_link_percentage", "Stereo Sidechain Detection Link Ratio", 0.0, 100.0, 80.0, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_inter_sample_peak_clipping_guard", "4x Oversampled Inter-Sample Peak (ISP) Detector", true, (34, 197, 94)))
        ),
        "MultibandUpwardExpansionTransientSnap" => Box::new(
            GenericDspNodeUi::new("MultibandUpwardExpansionTransientSnap", "Linear-Phase 4-Band Dynamic Upward Expander & Transient Snapper", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("expansion_ratio_slope", "Dynamic Upward Expansion Slope Ratio", 1.0, 3.0, 1.45, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("expansion_threshold_level_db", "Upward Expansion Triggering Threshold", -40.0, -6.0, -18.0, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("low_band_punch_reinforcement_db", "Low-Band Kick Drum Punch Upward Boost", 0.0, 12.0, 3.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_band_snare_crack_boost_db", "High-Mid Snare & Percussion Attack Snap", 0.0, 12.0, 4.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fir_crossover_linear_phase_depth", "FIR Linear Phase Crossover Filter Steepness", 12.0, 72.0, 36.0, "dB/oct", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_envelope_crest_factor_detection", "Crest-Factor Dynamic Sniffing Mode", true, (34, 197, 94)))
        ),
        "ParallelFeedbackFeedforwardOptoCompressor" => Box::new(
            GenericDspNodeUi::new("ParallelFeedbackFeedforwardOptoCompressor", "Dual Feedback / Feedforward Mastering Optical Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_level_db", "Master Compression Threshold Level", -36.0, 0.0, -14.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("optical_compression_ratio", "Optical Gain Reduction Compression Ratio", 1.2, 20.0, 3.5, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("feedback_feedforward_topology_blend", "Circuit Topology Blend (Feedback vs Feedforward)", 0.0, 1.0, 0.35, "blend", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("luminescent_panel_attack_time_ms", "Electro-Luminescent Panel Optical Attack Time", 0.5, 50.0, 10.0, "ms", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("photocell_two_stage_release_sec", "Photocell Dual-Stage Memory Release Time", 0.05, 3.0, 0.45, "s", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_highpass_sidechain_filter", "120 Hz Low-Cut Sidechain Bass Protection", true, (34, 197, 94)))
        ),
        "MultibandSubHarmonicPunchTransientExpander" => Box::new(
            GenericDspNodeUi::new("MultibandSubHarmonicPunchTransientExpander", "Tri-Band Sub-Harmonic Punch & Transient Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_thump_boost_db", "Sub-Harmonic Fundamental Thump Injection", 0.0, 18.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_attack_crest_multiplier", "Transient Onset Crest Factor Expansion Gain", 1.0, 3.0, 1.5, "x", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("sub_crossover_corner_freq_hz", "Sub-Bass Crossover Split Frequency", 50.0, 160.0, 85.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_detection_hold_time_ms", "Attack Detection Window Hold Time", 1.0, 30.0, 8.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_center_mono_clamping", "Sub-Bass Stereo Field Mono Anchor", 0.0, 1.0, 1.0, "mono", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_soft_clip_headroom_safety", "Analog Diode Soft-Clip Peak Limiting Safety", true, (34, 197, 94)))
        ),
        "MultibandMasteringFeedForwardVcaCompressor" => Box::new(
            GenericDspNodeUi::new("MultibandMasteringFeedForwardVcaCompressor", "4-Band Mastering Feed-Forward VCA Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("vca_threshold_db", "Master Compressor Threshold Level", -40.0, 0.0, -16.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("logarithmic_vca_ratio", "Logarithmic VCA Compression Ratio", 1.2, 10.0, 2.5, "ratio", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("rms_integration_time_ms", "Sidechain RMS Integration Window", 5.0, 200.0, 35.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("program_dependent_release_ms", "Program-Adaptive Release Envelope", 20.0, 1200.0, 180.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("parallel_dry_wet_blend", "Parallel Compression Dry/Wet Mix", 0.0, 1.0, 0.4, "mix", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_highpass_tilt", "6 dB/oct Sidechain Low-Cut Tilt Filter", true, (34, 197, 94)))
        ),
        "MidSidePsychoacousticPunchLimiter" => Box::new(
            GenericDspNodeUi::new("MidSidePsychoacousticPunchLimiter", "Mid/Side Psychoacoustic Brickwall Punch Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_true_peak_db", "Master True-Peak Brickwall Ceiling", -3.0, -0.1, -0.8, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("mid_channel_punch_drive_db", "Mid Channel Center Punch Input Drive", 0.0, 18.0, 4.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("side_channel_air_gain_db", "Side Channel Stereo Air Gain Offset", -6.0, 6.0, 0.5, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("lookahead_buffer_time_ms", "Anticipatory Lookahead Window", 0.5, 10.0, 3.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_transient_release", "Fast Transient Recovery Time Constant", 1.0, 50.0, 8.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_intersample_true_peak_4x", "4x ITU-R BS.1770 True-Peak Oversampling", true, (34, 197, 94)))
        ),
        "MultibandMasteringOpticalFeedbackLimiter" => Box::new(
            GenericDspNodeUi::new("MultibandMasteringOpticalFeedbackLimiter", "4-Band Mastering Optical Feedback Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("master_ceiling_threshold_dbtp", "True Peak Master Ceiling Output", -12.0, 0.0, -0.3, "dBFS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("optical_cell_lag_attack_ms", "Photoconductive Optocell Lag Attack", 0.1, 10.0, 1.2, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dual_stage_optical_release_sec", "Program-Dependent Dual-Stage Release", 0.05, 2.5, 0.35, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("multiband_crossover_balance_db", "4-Band Crossover Dynamic Balance", -6.0, 6.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lookahead_window_ms", "Lookahead Buffer Window Delay", 0.5, 10.0, 3.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_inter_sample_peak_detection", "4x Oversampled Inter-Sample Peak (ISP) Catcher", true, (34, 197, 94)))
        ),
        "DynamicTransientSpectralSnapProcessor" => Box::new(
            GenericDspNodeUi::new("DynamicTransientSpectralSnapProcessor", "Sub-Band Dynamic Transient Snap Processor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("transient_attack_boost_db", "Attack Wavefront Boost / Attenuation", -12.0, 18.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sustain_body_length_multiplier", "Sustain Decay Body Duration Multiplier", 0.2, 3.0, 1.0, "x", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("transient_spectral_focus_hz", "Transient Sub-Band Detection Focus", 200.0, 8000.0, 2400.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("detection_sensitivity_threshold", "Wavefront Energy Derivative Sensitivity", 0.1, 1.0, 0.55, "sens", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("soft_clip_output_safety_db", "Soft-Clip Peak Limiting Safety Threshold", -6.0, 0.0, -0.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_highpass_detector", "Detector 80Hz Sub-Bass High-Pass Filter", true, (34, 197, 94)))
        ),
        "MultibandMasteringVcaFeedbackBusCompressor" => Box::new(
            GenericDspNodeUi::new("MultibandMasteringVcaFeedbackBusCompressor", "Multiband Mastering VCA Feedback Bus Compressor", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold", "Master Bus Compression Threshold", -40.0, 0.0, -16.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ratio", "VCA Feedback Compression Ratio", 1.2, 10.0, 2.5, ":1", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attack", "Envelope Attack Time", 0.1, 100.0, 30.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("auto_release_mix", "Program Auto-Release Weight", 0.0, 1.0, 0.75, "auto", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sidechain_highpass", "Sidechain Low-Cut Filter", 20.0, 250.0, 85.0, "Hz", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("makeup_gain", "Master Output Makeup Gain", 0.0, 20.0, 2.5, "dB", (34, 197, 94)))
        ),
        "PsychoacousticSpectralCrestFactorLimiter" => Box::new(
            GenericDspNodeUi::new("PsychoacousticSpectralCrestFactorLimiter", "Psychoacoustic Spectral Crest Factor Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling", "True-Peak Output Ceiling", -6.0, 0.0, -0.3, "dBTP", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crest_sensitivity", "Spectral Crest Sensitivity", 0.0, 1.0, 0.65, "sens", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_release", "Fast Micro-Transient Release", 1.0, 100.0, 8.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("rms_recovery", "Sustained Body Recovery Time", 50.0, 1000.0, 180.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_masking", "Critical Band Masking Weight", 0.0, 1.0, 0.85, "mask", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("lookahead", "Zero-Distortion Lookahead Time", 0.5, 10.0, 3.0, "ms", (34, 197, 94)))
        ),
        "MasteringMidSideMultibandExpander" => Box::new(
            GenericDspNodeUi::new("MasteringMidSideMultibandExpander", "Mastering Mid-Side Multiband Expander", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("mid_expand_ratio", "Mid Channel Dynamic Expand Ratio", 1.0, 4.0, 1.4, ":1", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("side_expand_ratio", "Side Channel Dynamic Expand Ratio", 1.0, 4.0, 1.8, ":1", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("low_mid_crossover", "Low / Mid Crossover Frequency", 50.0, 500.0, 140.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_high_crossover", "Mid / High Crossover Frequency", 1000.0, 12000.0, 4500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("expansion_threshold", "Global Dynamic Expand Threshold", -50.0, -10.0, -28.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("release_ballistics_time", "Expander Release Recovery Time", 20.0, 1000.0, 180.0, "ms", (34, 197, 94)))
        ),
        "PsychoacousticMultiStageClippingLimiter" => Box::new(
            GenericDspNodeUi::new("PsychoacousticMultiStageClippingLimiter", "Psychoacoustic Multi-Stage Clipping Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("ceiling_threshold", "True-Peak Output Brickwall Ceiling", -6.0, 0.0, -0.2, "dBTP", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("soft_knee_clip_curve", "Asymptotic Soft-Clipper Knee", 0.0, 1.0, 0.45, "knee", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("oversampling_factor", "Polyphase Anti-Aliasing Oversampling", 2.0, 16.0, 8.0, "x", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_punch_drive", "Pre-Limiter Transient Punch Drive", 0.0, 18.0, 4.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("lookahead_safety_ms", "Predictive Lookahead Window", 0.5, 8.0, 2.5, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("true_peak_inter_sample_clamp", "Inter-Sample Peak Guard Clamping", 0.0, 1.0, 1.0, "guard", (34, 197, 94)))
        ),
        "MultibandMasteringDiodeBridgeLimiter" => Box::new(
            GenericDspNodeUi::new("MultibandMasteringDiodeBridgeLimiter", "Multiband Mastering Diode-Bridge Limiter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("threshold_ceiling_db", "Master Limiting Threshold Ceiling", -20.0, 0.0, -0.3, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diode_bridge_bias_current", "Diode Bridge Bias Operating Point", 0.1, 5.0, 1.2, "mA", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossover_low_mid_freq", "Low-Mid Band Crossover Split", 80.0, 600.0, 250.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossover_mid_high_freq", "Mid-High Band Crossover Split", 1000.0, 8000.0, 3500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fast_recovery_release_ms", "Adaptive Transient Release Speed", 5.0, 500.0, 45.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stereo_sidechain_link", "Stereo Sidechain Detection Link", 0.0, 1.0, 0.85, "link", (34, 197, 94)))
        ),
        "DynamicPsychoacousticTransientExciter" => Box::new(
            GenericDspNodeUi::new("DynamicPsychoacousticTransientExciter", "Dynamic Psychoacoustic Transient Exciter", DspNodeCategory::DynamicsMaster)
                .with_param(DspParamDescriptor::new_linear("transient_attack_boost", "Attack Edge Transient Gain", -12.0, 12.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonic_excitation_drive", "High-Frequency Harmonic Generator", 0.0, 1.0, 0.55, "drive", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("excitation_corner_frequency", "Exciter High-Pass Filter Cutoff", 2000.0, 12000.0, 5000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_decay_envelope", "Transient Window Integration Time", 2.0, 80.0, 18.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("dynamic_noise_gate_floor", "Background Ambience Gate Threshold", -80.0, -24.0, -54.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("wet_dry_master_blend", "Transient Processor Wet/Dry Mix", 0.0, 1.0, 0.70, "mix", (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
