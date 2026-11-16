// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `SpectralResynthesis`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("SpectralNeuralResynthesizerNode".to_string(), DspNodeDescriptor::new("SpectralNeuralResynthesizerNode", "Latent FFT Timbre Morphing Resynthesizer", DspNodeCategory::SpectralResynthesis, "Latent FFT Timbre Morphing Resynthesizer")
        .with_param(DspParamSchema::knob("latent_timbre", "Latent Timbre Coordinate", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Coordinate parameter"))
        .with_param(DspParamSchema::knob("spectral_res", "FFT Bin Resolution", 64.0, 1024.0, 256.0, "bins", MacroRole::Character, "FFT Bin Resolution integer control"))
        .with_param(DspParamSchema::knob("transient_boost", "Transient Preservation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Transient Preservation parameter"))
        .with_param(DspParamSchema::knob("smear", "Phase Spectral Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Phase Spectral Smear parameter"))
    );
    descriptors.insert("MultiChannelSpectralEqualizerNode".to_string(), DspNodeDescriptor::new("MultiChannelSpectralEqualizerNode", "64-Band Multi-Channel Spectral Sculptor", DspNodeCategory::SpectralResynthesis, "64-Band Multi-Channel Spectral Sculptor")
        .with_param(DspParamSchema::knob("tilt_slope", "Spectral Tilt Slope", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "Spectral Tilt Slope parameter"))
        .with_param(DspParamSchema::knob("air_band", "High Air Presence", 0.0, 12.0, 2.0, "dB", MacroRole::Tone, "High Air Presence parameter"))
        .with_param(DspParamSchema::knob("low_sub", "Sub Bass Focus", 0.0, 12.0, 1.5, "dB", MacroRole::Tone, "Sub Bass Focus parameter"))
        .with_param(DspParamSchema::knob("dynamic_unmask", "Dynamic Masking Reduction", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Dynamic Masking Reduction parameter"))
    );
    descriptors.insert("SpectralMatchingEqNode".to_string(), DspNodeDescriptor::new("SpectralMatchingEqNode", "AI 128-Band Spectral Curve Matching EQ", DspNodeCategory::SpectralResynthesis, "AI 128-Band Spectral Curve Matching EQ")
        .with_param(DspParamSchema::choice("target_curve", "Target Spectral Profile", &["Reference Track Target", "Harman Target", "Pink Noise Curve (1/f)", "Commercial Master"], 0, "Target Spectral Profile mode selector"))
        .with_param(DspParamSchema::knob("matching_intensity", "Curve Match Depth", 0.0, 100.0, 75.0, "%", MacroRole::Tone, "Curve Match Depth parameter"))
        .with_param(DspParamSchema::knob("smoothing_octaves", "Spectral Smoothing", 0.1, 2.0, 0.5, "oct", MacroRole::Tone, "Spectral Smoothing parameter"))
        .with_param(DspParamSchema::knob("gain_limit_db", "Max Boost/Cut Limit", 1.0, 18.0, 6.0, "dB", MacroRole::Tone, "Max Boost/Cut Limit parameter"))
    );
    descriptors.insert("SpectralTiltNode".to_string(), DspNodeDescriptor::new("SpectralTiltNode", "Linear Phase Psychoacoustic Spectral Tilt Filter", DspNodeCategory::SpectralResynthesis, "Linear Phase Psychoacoustic Spectral Tilt Filter")
        .with_param(DspParamSchema::knob("tilt_slope_db_oct", "Bode Spectral Tilt", -6.0, 6.0, 1.5, "dB/oct", MacroRole::Tone, "Bode Spectral Tilt parameter"))
        .with_param(DspParamSchema::log_knob("pivot_freq_hz", "Center Pivot Frequency", 200.0, 5000.0, 1000.0, "Hz", MacroRole::Tone, "Center Pivot Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("air_lift_db", "Top End Air Sheen", 0.0, 6.0, 1.2, "dB", MacroRole::Tone, "Top End Air Sheen parameter"))
        .with_param(DspParamSchema::knob("sub_weight_db", "Sub-Bass Weight", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "Sub-Bass Weight parameter"))
    );
    descriptors.insert("VocoderMatrixNode".to_string(), DspNodeDescriptor::new("VocoderMatrixNode", "32-Band Spectral Vocoder Analysis & Synthesis Matrix", DspNodeCategory::SpectralResynthesis, "32-Band Spectral Vocoder Analysis & Synthesis Matrix")
        .with_param(DspParamSchema::knob("band_count", "Analysis Filter Bands", 8.0, 32.0, 24.0, "bands", MacroRole::Character, "Analysis Filter Bands integer control"))
        .with_param(DspParamSchema::knob("formant_warp", "Spectral Formant Warp", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Spectral Formant Warp parameter"))
        .with_param(DspParamSchema::knob("sibilance_thru", "Unvoiced Noise Pass-thru", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Unvoiced Noise Pass-thru parameter"))
        .with_param(DspParamSchema::knob("carrier_drive", "Carrier Excitation Drive", 1.0, 10.0, 2.5, "x", MacroRole::Tone, "Carrier Excitation Drive parameter"))
    );
    descriptors.insert("AuditoryRoughnessNode".to_string(), DspNodeDescriptor::new("AuditoryRoughnessNode", "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer")
        .with_param(DspParamSchema::knob("dissonance_threshold", "Roughness Detection Sens", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Roughness Detection Sens parameter"))
        .with_param(DspParamSchema::knob("roughness_scaling", "Plomp-Levelt Critical Scale", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Plomp-Levelt Critical Scale parameter"))
        .with_param(DspParamSchema::choice("critical_bandwidth_mode", "Filter Bank Standard", &["Bark Scale (Zwicker)", "ERB Scale (Moore-Glasberg)", "Mel Scale"], 1, "Filter Bank Standard mode selector"))
        .with_param(DspParamSchema::knob("masking_depth", "Simultaneous Masking Depth", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Simultaneous Masking Depth parameter"))
    );
    descriptors.insert("SpectralAlignerNode".to_string(), DspNodeDescriptor::new("SpectralAlignerNode", "Multi-Track Phase & Spectral Coherence Auto-Aligner", DspNodeCategory::SpectralResynthesis, "Multi-Track Phase & Spectral Coherence Auto-Aligner")
        .with_param(DspParamSchema::knob("reference_track_id", "Reference Master Track ID", 0.0, 64.0, 0.0, "trk", MacroRole::Character, "Reference Master Track ID integer control"))
        .with_param(DspParamSchema::knob("phase_correlation_target", "Target Phase Alignment", 0.5, 1.0, 0.95, "%", MacroRole::Tone, "Target Phase Alignment parameter"))
        .with_param(DspParamSchema::knob("delay_offset_samples", "Sub-Sample Delay Offset", -500.0, 500.0, 0.0, "samples", MacroRole::Tone, "Sub-Sample Delay Offset parameter"))
        .with_param(DspParamSchema::toggle("polarity_auto_invert", "Auto Polarity Inversion", true, "Auto Polarity Inversion toggle switch"))
    );
    descriptors.insert("SpectralDebleedNode".to_string(), DspNodeDescriptor::new("SpectralDebleedNode", "Microphone Acoustic Bleed Elimination Matrix", DspNodeCategory::SpectralResynthesis, "Microphone Acoustic Bleed Elimination Matrix")
        .with_param(DspParamSchema::knob("bleed_suppression_db", "Bleed Isolation Depth", 0.0, 30.0, 14.0, "dB", MacroRole::Tone, "Bleed Isolation Depth parameter"))
        .with_param(DspParamSchema::knob("spectral_gate_threshold", "Spectral Gate Floor", -60.0, -10.0, -35.0, "dB", MacroRole::Tone, "Spectral Gate Floor parameter"))
        .with_param(DspParamSchema::knob("crosstalk_estimate_ms", "Acoustic Delay Estimate", 0.1, 50.0, 8.5, "ms", MacroRole::Tone, "Acoustic Delay Estimate parameter"))
        .with_param(DspParamSchema::toggle("transient_preserve", "Preserve Direct Transients", true, "Preserve Direct Transients toggle switch"))
    );
    descriptors.insert("SpectralDeEsserNode".to_string(), DspNodeDescriptor::new("SpectralDeEsserNode", "Dynamic Spectral Sibilance & Harshness Reducer", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Sibilance & Harshness Reducer")
        .with_param(DspParamSchema::log_knob("sibilance_center_hz", "Harshness Center Freq", 3000.0, 14000.0, 7200.0, "Hz", MacroRole::Tone, "Harshness Center Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("bandwidth_oct", "Sibilance Bandwidth", 0.2, 3.0, 1.2, "oct", MacroRole::Tone, "Sibilance Bandwidth parameter"))
        .with_param(DspParamSchema::knob("max_reduction_db", "Max Dynamic Reduction", 0.0, 24.0, 9.0, "dB", MacroRole::Tone, "Max Dynamic Reduction parameter"))
        .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer Time", 0.0, 10.0, 2.5, "ms", MacroRole::Tone, "Lookahead Buffer Time parameter"))
    );
    descriptors.insert("SpectralFlatnessNode".to_string(), DspNodeDescriptor::new("SpectralFlatnessNode", "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer", DspNodeCategory::SpectralResynthesis, "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer")
        .with_param(DspParamSchema::knob("entropy_threshold", "Spectral Flatness Cutoff", 0.0, 1.0, 0.45, "entropy", MacroRole::Tone, "Spectral Flatness Cutoff parameter"))
        .with_param(DspParamSchema::knob("analysis_window_size", "FFT Window Analysis Size", 256.0, 4096.0, 1024.0, "bins", MacroRole::Character, "FFT Window Analysis Size integer control"))
        .with_param(DspParamSchema::knob("tonal_split_gain", "Tonal Harmonic Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Tonal Harmonic Level parameter"))
        .with_param(DspParamSchema::knob("noise_split_gain", "Noise Residual Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Noise Residual Level parameter"))
    );
    descriptors.insert("SpectralGrainCloudNode".to_string(), DspNodeDescriptor::new("SpectralGrainCloudNode", "Spectral Domain Grain Scatter & Diffusion Engine", DspNodeCategory::SpectralResynthesis, "Spectral Domain Grain Scatter & Diffusion Engine")
        .with_param(DspParamSchema::knob("grain_density_spectral", "Frequency Bin Grains", 10.0, 500.0, 120.0, "grains/s", MacroRole::Tone, "Frequency Bin Grains parameter"))
        .with_param(DspParamSchema::knob("time_smear_factor", "Phase Time Smear", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Phase Time Smear parameter"))
        .with_param(DspParamSchema::knob("phase_randomization", "Bin Phase Chaos", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Bin Phase Chaos parameter"))
        .with_param(DspParamSchema::knob("diffusion_wet_mix", "Grain Cloud Blend", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Grain Cloud Blend parameter"))
    );
    descriptors.insert("SpectralMaskingNode".to_string(), DspNodeDescriptor::new("SpectralMaskingNode", "Psychoacoustic Simultaneous Spectral Masking Visualizer", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Simultaneous Spectral Masking Visualizer")
        .with_param(DspParamSchema::knob("masking_threshold_offset_db", "Masking Threshold Bias", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Masking Threshold Bias parameter"))
        .with_param(DspParamSchema::knob("critical_band_scale", "Critical Band Width Scale", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Critical Band Width Scale parameter"))
        .with_param(DspParamSchema::choice("bark_scale_mode", "Psychoacoustic Standard", &["Bark Scale Matrix", "ERB Auditory Filter", "Equivalent Rectangular Band"], 0, "Psychoacoustic Standard mode selector"))
        .with_param(DspParamSchema::knob("alert_color_intensity", "Masking Flare Brightness", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Masking Flare Brightness parameter"))
    );
    descriptors.insert("SpectralMorphNode".to_string(), DspNodeDescriptor::new("SpectralMorphNode", "Latent FFT Magnitude & Phase Interpolation Morph Engine", DspNodeCategory::SpectralResynthesis, "Latent FFT Magnitude & Phase Interpolation Morph Engine")
        .with_param(DspParamSchema::knob("morph_position", "Spectral Morph Interpolation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Spectral Morph Interpolation parameter"))
        .with_param(DspParamSchema::knob("spectral_smear", "Cross-Bin Spectral Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Cross-Bin Spectral Smear parameter"))
        .with_param(DspParamSchema::choice("phase_alignment_mode", "Phase Interpolation Mode", &["Magnitude Only (Zero Phase)", "Phase Unwrapped Morph", "Cross-Correlation Aligned"], 1, "Phase Interpolation Mode mode selector"))
        .with_param(DspParamSchema::toggle("harmonic_lock", "Lock Harmonic Partials", true, "Lock Harmonic Partials toggle switch"))
    );
    descriptors.insert("SpectralReshaperNode".to_string(), DspNodeDescriptor::new("SpectralReshaperNode", "Dynamic Spectral Envelope & Formant Reshaper", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Envelope & Formant Reshaper")
        .with_param(DspParamSchema::knob("formant_shift_semitones", "Spectral Envelope Shift", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Spectral Envelope Shift parameter"))
        .with_param(DspParamSchema::knob("spectral_envelope_gain", "Envelope Reshape Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Envelope Reshape Gain parameter"))
        .with_param(DspParamSchema::knob("smoothing_factor", "Envelope Cepstral Smoothing", 0.1, 5.0, 1.2, "oct", MacroRole::Tone, "Envelope Cepstral Smoothing parameter"))
        .with_param(DspParamSchema::toggle("dynamic_tracking", "Dynamic Pitch Tracking", true, "Dynamic Pitch Tracking toggle switch"))
    );
    descriptors.insert("SpectralResynthesisNode".to_string(), DspNodeDescriptor::new("SpectralResynthesisNode", "Harmonic Additive Spectral Resynthesis Engine", DspNodeCategory::SpectralResynthesis, "Harmonic Additive Spectral Resynthesis Engine")
        .with_param(DspParamSchema::knob("max_partials_count", "Additive Sinusoidal Partials", 8.0, 256.0, 64.0, "partials", MacroRole::Character, "Additive Sinusoidal Partials integer control"))
        .with_param(DspParamSchema::knob("spectral_threshold_db", "Noise Floor Cutoff", -80.0, -20.0, -50.0, "dB", MacroRole::Tone, "Noise Floor Cutoff parameter"))
        .with_param(DspParamSchema::knob("inharmonic_drift", "Partial Inharmonicity", 0.0, 2.0, 0.0, "", MacroRole::Tone, "Partial Inharmonicity parameter"))
        .with_param(DspParamSchema::knob("harmonic_additive_mix", "Synthesized Partials Level", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Synthesized Partials Level parameter"))
    );
    descriptors.insert("SpectralUnmaskerNode".to_string(), DspNodeDescriptor::new("SpectralUnmaskerNode", "AI-Driven Intelligent Sidechain Spectral Unmasker", DspNodeCategory::SpectralResynthesis, "AI-Driven Intelligent Sidechain Spectral Unmasker")
        .with_param(DspParamSchema::knob("sidechain_input_id", "Sidechain Trigger Track", 0.0, 64.0, 1.0, "trk", MacroRole::Character, "Sidechain Trigger Track integer control"))
        .with_param(DspParamSchema::knob("unmasking_depth_db", "Dynamic Unmask Depth", 0.0, 18.0, 5.0, "dB", MacroRole::Tone, "Dynamic Unmask Depth parameter"))
        .with_param(DspParamSchema::knob("frequency_resolution_bins", "Spectral Bin Resolution", 32.0, 512.0, 128.0, "bins", MacroRole::Character, "Spectral Bin Resolution integer control"))
        .with_param(DspParamSchema::knob("adaptation_speed_ms", "Fast Reaction Speed", 5.0, 250.0, 35.0, "ms", MacroRole::Tone, "Fast Reaction Speed parameter"))
    );
    descriptors.insert("Spectrogram3DNode".to_string(), DspNodeDescriptor::new("Spectrogram3DNode", "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer", DspNodeCategory::SpectralResynthesis, "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer")
        .with_param(DspParamSchema::knob("fft_size_bins", "FFT Analysis Size", 256.0, 4096.0, 1024.0, "bins", MacroRole::Character, "FFT Analysis Size integer control"))
        .with_param(DspParamSchema::knob("waterfall_depth_frames", "Waterfall History Buffer", 30.0, 300.0, 120.0, "frames", MacroRole::Character, "Waterfall History Buffer integer control"))
        .with_param(DspParamSchema::choice("color_palette", "Spectrogram Palette", &["Cyberpunk Neon", "Thermal Infrared", "Obsidian Ice", "Grayscale Contrast"], 0, "Spectrogram Palette mode selector"))
        .with_param(DspParamSchema::toggle("frequency_scale_log", "Logarithmic Frequency Grid", true, "Logarithmic Frequency Grid toggle switch"))
    );
    descriptors.insert("StereoVectorscopeNode".to_string(), DspNodeDescriptor::new("StereoVectorscopeNode", "High-Speed Phosphor Stereophonic Vectorscope Analyzer", DspNodeCategory::SpectralResynthesis, "High-Speed Phosphor Stereophonic Vectorscope Analyzer")
        .with_param(DspParamSchema::knob("phosphor_decay_rate", "Phosphor Glow Persistence", 0.05, 1.0, 0.35, "s", MacroRole::Tone, "Phosphor Glow Persistence parameter"))
        .with_param(DspParamSchema::knob("vectorscope_gain", "Lissajous Input Scale", 0.5, 4.0, 1.0, "x", MacroRole::Tone, "Lissajous Input Scale parameter"))
        .with_param(DspParamSchema::knob("lissajous_dot_size", "Vector Trace Thickness", 1.0, 5.0, 1.8, "pt", MacroRole::Tone, "Vector Trace Thickness parameter"))
        .with_param(DspParamSchema::toggle("correlation_meter_bar", "Show Phase Bar Meter", true, "Show Phase Bar Meter toggle switch"))
    );
    descriptors.insert("SpectrogramArtNode".to_string(), DspNodeDescriptor::new("SpectrogramArtNode", "Spectrogram Art Image-to-Audio Synthesizer", DspNodeCategory::SpectralResynthesis, "Transforms visual bitmap images into acoustic frequency spectra visible on any 2D/3D waterfall spectrogram")
        .with_param(DspParamSchema::knob("base_freq", "Base Fundamental Frequency", 50.0, 500.0, 100.0, "Hz", MacroRole::Tone, "Lowest acoustic frequency mapped to bottom image edge parameter"))
        .with_param(DspParamSchema::knob("freq_bandwidth", "Frequency Bandwidth", 1000.0, 20000.0, 8000.0, "Hz", MacroRole::Tone, "Total vertical frequency spectrum height parameter"))
        .with_param(DspParamSchema::knob("scan_rate", "Horizontal Scan Rate", 1.0, 100.0, 25.0, "cols/s", MacroRole::Punch, "Bitmap pixel column playback scanning speed parameter"))
        .with_param(DspParamSchema::knob("intensity_contrast", "Contrast & Luminance", 0.1, 3.0, 1.2, "x", MacroRole::Character, "Greyscale pixel luminance to harmonic partial gain exponent parameter"))
    );
    descriptors.insert("SpectrogramArtMorphNode".to_string(), DspNodeDescriptor::new("SpectrogramArtMorphNode", "Dual Spectrogram Image Morphing Synthesizer", DspNodeCategory::SpectralResynthesis, "Real-time cross-dissolve and frequency warping between two visual spectrogram bitmap art templates")
        .with_param(DspParamSchema::knob("morph_factor", "Image Morph Blend", 0.0, 1.0, 0.0, "%", MacroRole::Space, "Cross-fading interpolation between Image A and Image B parameter"))
        .with_param(DspParamSchema::knob("base_freq", "Base Fundamental Frequency", 50.0, 500.0, 100.0, "Hz", MacroRole::Tone, "Lowest acoustic partial frequency parameter"))
        .with_param(DspParamSchema::knob("freq_bandwidth", "Frequency Bandwidth", 1000.0, 20000.0, 8000.0, "Hz", MacroRole::Tone, "Total spectral bandwidth span parameter"))
        .with_param(DspParamSchema::knob("scan_rate", "Horizontal Scan Rate", 1.0, 100.0, 25.0, "cols/s", MacroRole::Punch, "Scanning speed across image frames parameter"))
    );
    descriptors.insert("SpectralResynthesisEngine".to_string(), DspNodeDescriptor::new("SpectralResynthesisEngine", "Harmonic Profile Spectral Resynthesis Engine", DspNodeCategory::SpectralResynthesis, "Additive sinusoidal analysis and re-synthesis with independent control of inharmonicity, odd/even balance, and formants")
        .with_param(DspParamSchema::knob("fundamental", "Fundamental Frequency", 20.0, 2000.0, 220.0, "Hz", MacroRole::Tone, "Fundamental harmonic frequency anchor parameter"))
        .with_param(DspParamSchema::knob("morph_amount", "Spectral Profile Morph", 0.0, 1.0, 0.0, "%", MacroRole::Space, "Interpolation between Harmonic Profile A and Profile B parameter"))
        .with_param(DspParamSchema::knob("inharmonicity", "Partial Inharmonicity", 0.0, 0.5, 0.0, "%", MacroRole::Character, "Partial frequency stretching coefficient parameter"))
        .with_param(DspParamSchema::knob("odd_even_balance", "Odd / Even Harmonic Balance", -1.0, 1.0, 0.0, "bal", MacroRole::Tone, "Odd versus even harmonic partial energy balance parameter"))
        .with_param(DspParamSchema::knob("spectral_tilt", "Spectral Spectral Slope", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High-frequency partial rolloff slope parameter"))
        .with_param(DspParamSchema::knob("formant_shift", "Formant Envelope Shift", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Vocal and acoustic formant filter scaling factor parameter"))
    );
    descriptors.insert("SpectrogramArtEngine".to_string(), DspNodeDescriptor::new("SpectrogramArtEngine", "Visual Spectrogram BMP/PNG Image-to-Sound Synthesizer Engine", DspNodeCategory::SpectralResynthesis, "Phase-coherent additive synthesis engine transforming 2D bitmap images and spectrogram paintings into high-resolution sound")
        .with_param(DspParamSchema::log_knob("min_freq_hz", "Lowest Spectrogram Frequency", 20.0, 500.0, 60.0, "Hz", MacroRole::Tone, "Bottom row image pixel frequency mapping parameter"))
        .with_param(DspParamSchema::log_knob("max_freq_hz", "Highest Spectrogram Frequency", 1000.0, 22000.0, 12000.0, "Hz", MacroRole::Character, "Top row image pixel frequency mapping parameter"))
        .with_param(DspParamSchema::knob("time_scale_sec", "Playback Duration", 0.5, 60.0, 8.0, "s", MacroRole::Space, "Duration to scan image columns from left to right parameter"))
        .with_param(DspParamSchema::knob("brightness_gain", "Pixel Brightness Amplitude", 0.0, 3.0, 1.0, "x", MacroRole::Punch, "Luminance to oscillator partial amplitude gain scaling parameter"))
    );
    descriptors.insert("SpectrogramArtMorpher".to_string(), DspNodeDescriptor::new("SpectrogramArtMorpher", "2D Spectrogram Visual Morph & Cross-Synthesis Morpher", DspNodeCategory::SpectralResynthesis, "Spectral interpolation morpher blending between two spectrogram images with optical flow frequency tracking")
        .with_param(DspParamSchema::knob("morph_position", "Spectrogram Morph Position", 0.0, 1.0, 0.50, "%", MacroRole::Tone, "Linear or non-linear interpolation between Image A and Image B parameter"))
        .with_param(DspParamSchema::knob("spectral_smear", "Temporal Smear Window", 0.0, 1.0, 0.20, "%", MacroRole::Space, "Frequency bin blur creating dreamy spectral trails parameter"))
        .with_param(DspParamSchema::knob("phase_coherence", "Phase Coherence Lock", 0.0, 1.0, 0.85, "%", MacroRole::Character, "Preserves audio punch across fast morph transitions parameter"))
        .with_param(DspParamSchema::toggle("harmonic_preserve", "Harmonic Peak Preservation", true, "Prioritize musical pitch peaks over background noise texture parameter"))
    );
    descriptors.insert("SpectrumMatchingAnalyzer".to_string(), DspNodeDescriptor::new("SpectrumMatchingAnalyzer", "Multi-Band Target Spectrum Curve Matching & Correction Generator", DspNodeCategory::SpectralResynthesis, "Precision spectral analysis node capturing reference audio frequency envelopes and calculating matching FIR/IIR correction curves")
        .with_param(DspParamSchema::choice("num_bands", "Frequency Band Resolution", &["32 Bands (Fast)", "64 Bands (Standard)", "128 Bands (Mastering)", "256 Bands (Ultra)"], 2, "Resolution of frequency analysis filter bank"))
        .with_param(DspParamSchema::knob("matching_intensity", "Spectral Convergence Amount", 0.0, 1.0, 0.75, "%", MacroRole::Character, "Percentage of target reference curve applied to source"))
        .with_param(DspParamSchema::knob("smoothing_q", "Band Curve Smoothing", 0.2, 5.0, 1.5, "Q", MacroRole::Tone, "Interpolation smoothing factor across frequency bands"))
        .with_param(DspParamSchema::knob("max_gain_db", "Max Boost / Cut Boundary", 1.0, 18.0, 6.0, "dB", MacroRole::Punch, "Maximum allowable boost or cut applied per band"))
        .with_param(DspParamSchema::toggle("freeze_snapshot", "Freeze Reference Spectrum", false, "Locks the current target reference spectral snapshot"))
    );
    descriptors.insert("VowelTrajectoryGraph".to_string(), DspNodeDescriptor::new("VowelTrajectoryGraph", "2D F1/F2 Vowel Formant Trajectory & Morphing Spline HUD", DspNodeCategory::SpectralResynthesis, "Acoustic vowel chart space (Peterson-Barney F1 vs F2) interpolating formant resonance trajectories across Japanese, Italian, and English vowels")
        .with_param(DspParamSchema::knob("vowel_interpolation_speed_ms", "Formant Morphing Transition Duration", 10.0, 1000.0, 150.0, "ms", MacroRole::Space, "Time required to transition between vowel waypoints along trajectory spline"))
        .with_param(DspParamSchema::knob("f1_base_frequency_hz", "Vowel First Formant (F1) Jaw/Mouth Height", 200.0, 1200.0, 500.0, "Hz", MacroRole::Tone, "Primary low formant frequency representing open/closed mouth aperture"))
        .with_param(DspParamSchema::knob("f2_base_frequency_hz", "Vowel Second Formant (F2) Tongue Position", 600.0, 3500.0, 1500.0, "Hz", MacroRole::Character, "Secondary formant frequency representing front/back tongue advancement in vocal tract"))
        .with_param(DspParamSchema::knob("vocal_tract_scale_factor", "Vocal Tract Anatomy Scale Scaling Factor", 0.7, 1.4, 1.0, "scale", MacroRole::Punch, "Global frequency scaling shifting formant profiles between bass, tenor, and soprano"))
        .with_param(DspParamSchema::choice("trajectory_loop_mode", "Spline Trajectory Navigation Cycle Mode", &["One-Shot", "Ping-Pong Loop", "Continuous Forward Loop", "LFO Modulated"], 1, "Playback direction and loop behavior across designated vowel chart waypoints"))
    );
    descriptors.insert("VowelCylinderAreaSynthesizer".to_string(), DspNodeDescriptor::new("VowelCylinderAreaSynthesizer", "44-Cylinder Acoustic Area Vocal Tract Formant Synthesizer HUD", DspNodeCategory::SpectralResynthesis, "Acoustic waveguide physical model synthesizing human vocal tract geometry from 44 discrete cross-sectional tube area cylinders with lip radiation")
        .with_param(DspParamSchema::knob("glottis_vocal_tract_length_cm", "Acoustic Vocal Tract Length", 12.0, 22.0, 17.5, "cm", MacroRole::Tone, "Physical length of Kelly-Lochbaum vocal tube determining baseline formant frequency spacing"))
        .with_param(DspParamSchema::knob("lip_radiation_aperture_cm2", "Lip Termination Radiation Aperture Area", 0.2, 8.0, 2.5, "cm²", MacroRole::Character, "Cross-sectional area of cylinder 44 governing high-frequency spherical wave acoustic radiation"))
        .with_param(DspParamSchema::knob("pharyngeal_constriction_pos", "Tongue & Pharyngeal Constriction Center", 0.0, 1.0, 0.45, "norm", MacroRole::Tone, "Normalized anatomical position along 44-cylinder vocal tract representing primary phonetic constriction"))
        .with_param(DspParamSchema::knob("vocal_tract_damping_factor", "Acoustic Tube Wall Viscous Damping", 0.8, 0.999, 0.985, "Q", MacroRole::Punch, "Energy absorption coefficient modeling mucosal tissue damping and heat conduction along tube walls"))
        .with_param(DspParamSchema::toggle("acoustic_nasal_coupling", "Engage Velopharyngeal Nasal Cavity Coupling", false, "Opens secondary acoustic waveguide branch modeling nasal consonant resonances /m/ and /n/"))
    );
    descriptors.insert("FormantTrajectoryPoint".to_string(), DspNodeDescriptor::new("FormantTrajectoryPoint", "2D Vocal Tract Formant Space Spline Trajectory Interpolator HUD", DspNodeCategory::SpectralResynthesis, "Interpolates multi-dimensional vocal tract acoustic coordinates (F1, F2, F3, jaw opening, tongue position, velum aperture) along continuous spline paths")
        .with_param(DspParamSchema::choice("vowel_phoneme_target", "Target Vocal Phoneme & Articulatory Vowel Space", &["Vowel /a/ (Father - Low Back)", "Vowel /i/ (See - High Front)", "Vowel /u/ (Boot - High Back Rounded)", "Vowel /e/ (Bed - Mid Front)", "Vowel /o/ (Law - Mid Back)"], 0, "Preset vocal tract physical target coordinates"))
        .with_param(DspParamSchema::log_knob("formant_f1_hz", "First Formant Frequency F1 (Jaw Aperture & Vowel Height)", 150.0, 1200.0, 700.0, "Hz", MacroRole::Tone, "Acoustic resonance frequency governed primarily by vertical jaw displacement"))
        .with_param(DspParamSchema::log_knob("formant_f2_hz", "Second Formant Frequency F2 (Tongue Advancement)", 500.0, 3500.0, 1200.0, "Hz", MacroRole::Tone, "Resonant frequency governed by anterior/posterior tongue position"))
        .with_param(DspParamSchema::knob("nasal_velum_coupling", "Nasal Port Velum Acoustic Opening Ratio", 0.0, 1.0, 0.15, "%", MacroRole::Character, "Soft palate port opening introducing nasal anti-resonances"))
        .with_param(DspParamSchema::knob("spline_interpolation_smoothness", "Articulatory Gesture Spline Transition Duration", 10.0, 500.0, 80.0, "ms", MacroRole::Space, "Temporal glide duration between consecutive phonetic vowel coordinates"))
    );
    descriptors.insert("SpectrogramArtConfig".to_string(), DspNodeDescriptor::new("SpectrogramArtConfig", "Visual Spectrogram Art Sonification Engine Configuration", DspNodeCategory::SpectralResynthesis, "Configuration manager for translating visual bitmap images into additive audio oscillator banks with logarithmic frequency spacing")
        .with_param(DspParamSchema::choice("num_frequency_bins", "Additive Spectral Sine Oscillator Count", &["64 Partials", "128 Partials", "256 Partials", "512 Partials"], 2, "Number of vertical frequency bins synthesized from image height"))
        .with_param(DspParamSchema::log_knob("min_freq_hz", "Bottom Row Base Frequency", 20.0, 500.0, 60.0, "Hz", MacroRole::Tone, "Lowest frequency assigned to image bottom baseline"))
        .with_param(DspParamSchema::log_knob("max_freq_hz", "Top Row Ceiling Frequency", 1000.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Highest frequency assigned to image top border"))
        .with_param(DspParamSchema::knob("scan_speed_pps", "Horizontal Scanline Speed", 10.0, 500.0, 100.0, "px/s", MacroRole::Space, "Playback speed traversing image from left to right in pixels/sec"))
        .with_param(DspParamSchema::knob("gamma_contrast", "Luminance-to-Amplitude Gamma", 0.2, 3.0, 1.2, "gamma", MacroRole::Punch, "Non-linear exponent mapping pixel intensity to partial volume"))
    );
    descriptors.insert("SpectrogramImage".to_string(), DspNodeDescriptor::new("SpectrogramImage", "2D Visual Sound Painting Raster Canvas & Image Buffer", DspNodeCategory::SpectralResynthesis, "2D image buffer holding visual raster artwork for optical audio synthesis, supporting real-time brightness curves, color palettes, and inversion")
        .with_param(DspParamSchema::knob("brightness", "Visual Canvas Master Brightness", 0.0, 2.0, 1.0, "gain", MacroRole::Tone, "Overall brightness multiplier scaling sound volume"))
        .with_param(DspParamSchema::knob("contrast", "Visual Dynamic Contrast Ratio", 0.5, 3.0, 1.0, "contrast", MacroRole::Punch, "Contrast steepness accentuating loud versus quiet partials"))
        .with_param(DspParamSchema::toggle("invert_colors", "Negative Invert Pixel Luminance", false, "Inverts black-and-white pixel amplitudes"))
        .with_param(DspParamSchema::choice("color_mapping", "Color Palette Interpretation Mode", &["Grayscale Luminance", "RGB to Pitch-Triad", "Thermal Spectral", "Cyberpunk Neon"], 0, "Color extraction mode mapping hue/saturation to sound"))
        .with_param(DspParamSchema::choice("horizontal_resolution", "Raster Canvas Horizontal Buffer Width", &["256 Pixels", "512 Pixels", "1024 Pixels", "2048 Pixels"], 1, "Image width defining temporal detail of sonified painting"))
    );
    descriptors.insert("ImageNoteTrigger".to_string(), DspNodeDescriptor::new("ImageNoteTrigger", "Optical Pixel Note Trigger & Coordinates-to-MIDI Translator", DspNodeCategory::SpectralResynthesis, "Scans spectrogram image columns for high-intensity pixel peaks, triggering polyphonic MIDI notes and harmonic partial velocities")
        .with_param(DspParamSchema::knob("threshold", "Pixel Intensity Detection Threshold", 0.01, 0.95, 0.20, "thresh", MacroRole::Punch, "Minimum optical luminance required to trigger note event"))
        .with_param(DspParamSchema::knob("max_simultaneous_notes", "Polyphonic Optical Note Limit", 1.0, 32.0, 8.0, "notes", MacroRole::Character, "Maximum concurrent notes emitted by visual scanline"))
        .with_param(DspParamSchema::choice("scale_quantize", "Optical Frequency Scale Snapping", &["Continuous Microtonal", "12-TET Chromatic", "Major Pentatonic", "Minor Blues", "Dorian Mode"], 1, "Musical scale quantization grid constraining emitted pitches"))
        .with_param(DspParamSchema::knob("velocity_scale", "Luminance-to-Velocity Sensitivity", 0.1, 2.0, 1.0, "vel", MacroRole::Punch, "Sensitivity scaling pixel luminance into MIDI velocity"))
        .with_param(DspParamSchema::knob("hysteresis_ms", "Note Re-trigger Lockout Time", 5.0, 200.0, 30.0, "ms", MacroRole::Space, "Minimum duration before identical note can re-trigger"))
    );
    descriptors.insert("SpectralMorphConfig".to_string(), DspNodeDescriptor::new("SpectralMorphConfig", "Dynamic Dual-Spectrum Morphing & Cross-Synthesis Engine", DspNodeCategory::SpectralResynthesis, "Calculates spectral interpolation between two distinct FFT frequency profiles, enabling smooth timbre morphing and spectral hybrid synthesis")
        .with_param(DspParamSchema::knob("morph_factor", "Timbral Cross-Morphing Position", 0.0, 1.0, 0.50, "%", MacroRole::Tone, "Linear interpolation coordinate between Source A and Source B"))
        .with_param(DspParamSchema::knob("spectral_tilt", "High-Frequency Spectral Tilt", -12.0, 12.0, 0.0, "dB/oct", MacroRole::Character, "Spectral balance slope boosting or cutting upper harmonics"))
        .with_param(DspParamSchema::choice("phase_coupling", "FFT Bin Phase Reconciliation Mode", &["Linear Phase Blend", "Minimum Phase Warping", "Random Phase Scrambler", "Locked Source A"], 0, "Phase reconstruction method preventing comb filtering during morphing"))
        .with_param(DspParamSchema::knob("formant_preserve", "Acoustic Vocal Formant Preservation", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Locks resonant envelope peaks during inter-sound interpolation"))
        .with_param(DspParamSchema::knob("smear_bandwidth", "Spectral Energy Diffusion Smear", 0.0, 1000.0, 100.0, "Hz", MacroRole::Space, "Gaussian partial blurring across adjacent frequency bins"))
    );
    descriptors.insert("FormantBusSnapshot".to_string(), DspNodeDescriptor::new("FormantBusSnapshot", "Anthropomorphic Vocal Tract & F1-F4 Formant Envelope HUD", DspNodeCategory::SpectralResynthesis, "Human vocal tract acoustic tube and vocal resonator filter bus providing real-time vowel morphing across F1-F4 formant frequencies, vocal bandwidths, nasal pharynx coupling, and larynx throat depth")
        .with_param(DspParamSchema::knob("vowel_morph_xy", "Continuous Vowel Space Morph Coordinate", 0.0, 1.0, 0.3, "morph", MacroRole::Tone, "Continuous vowel formant morphing between [a, e, i, o, u]"))
        .with_param(DspParamSchema::knob("formant_shift_cents", "Acoustic Vocal Tract Length Pitch Shift", -1200.0, 1200.0, 0.0, "cents", MacroRole::Tone, "Global formant scaling simulating vocal tract length changes"))
        .with_param(DspParamSchema::knob("nasal_coupling_amount", "Soft Palate Velum Nasal Coupling", 0.0, 1.0, 0.15, "norm", MacroRole::Space, "Soft palate velum opening coupling acoustic nasal resonance"))
        .with_param(DspParamSchema::knob("throat_larynx_depth", "Pharyngeal Cavity Throat Larynx Depth", 0.5, 2.0, 1.0, "geom", MacroRole::Punch, "Pharyngeal cavity throat depth and chest cavity resonance"))
        .with_param(DspParamSchema::knob("vocal_formant_q_factor", "Formant Filter Bandwidth Sharpness Q", 1.0, 20.0, 6.5, "Q", MacroRole::Tone, "Bandwidth sharpness of individual formant band filters"))
    );
    descriptors.insert("HarmonicProfile".to_string(), DspNodeDescriptor::new("HarmonicProfile", "Additive Harmonic Partial Distribution & Centroid Envelope HUD", DspNodeCategory::SpectralResynthesis, "Spectral distribution and harmonic synthesis contour node controlling odd-to-even partial balance, high-frequency spectral rolloff slope, inharmonicity stretch, and spectral tilt centroid")
        .with_param(DspParamSchema::knob("odd_even_balance", "Odd-To-Even Harmonic Partial Ratio", -1.0, 1.0, 0.0, "bal", MacroRole::Tone, "Balance between odd harmonics (clarinet) and even harmonics (strings)"))
        .with_param(DspParamSchema::knob("spectral_tilt_slope", "High-Frequency Spectral Tilt Rolloff", -24.0, 6.0, -6.0, "dB/oct", MacroRole::Tone, "High-frequency partial attenuation rolloff rate"))
        .with_param(DspParamSchema::knob("inharmonic_stretch", "Wire Dispersion Inharmonicity Stretch", 0.0, 0.1, 0.002, "factor", MacroRole::Tone, "Stiff bar or heavy wire overtone stretching factor"))
        .with_param(DspParamSchema::knob("harmonic_count", "Synthesized Fourier Partial Count", 1.0, 128.0, 32.0, "partials", MacroRole::Punch, "Active synthesized Fourier sinusoidal partial count"))
        .with_param(DspParamSchema::knob("spectral_centroid_shift", "Spectral Mass Centroid Brightness Shift", -1000.0, 2000.0, 0.0, "Hz", MacroRole::Tone, "Center of spectral mass frequency brightness shift"))
    );
    descriptors.insert("VowelNode".to_string(), DspNodeDescriptor::new("VowelNode", "IPA Vowel Formant Coordinate & Articulatory Space HUD", DspNodeCategory::SpectralResynthesis, "International Phonetic Alphabet (IPA) vowel formant chart coordinate node specifying F1-F4 formant frequencies, resonance bandwidths, tongue backness position, tongue height, and lip opening")
        .with_param(DspParamSchema::knob("formant_f1_hz", "First Formant F1 (Vowel Height)", 200.0, 1200.0, 500.0, "Hz", MacroRole::Tone, "First formant center frequency inversely proportional to vowel height"))
        .with_param(DspParamSchema::knob("formant_f2_hz", "Second Formant F2 (Vowel Backness)", 600.0, 3000.0, 1500.0, "Hz", MacroRole::Tone, "Second formant center frequency proportional to tongue frontness"))
        .with_param(DspParamSchema::knob("tongue_position", "Tongue Backness (Back to Front)", 0.0, 1.0, 0.5, "norm", MacroRole::Tone, "Physiological tongue constriction location from velum to palate"))
        .with_param(DspParamSchema::knob("tongue_height", "Tongue Elevation (Open to Close)", 0.0, 1.0, 0.6, "norm", MacroRole::Character, "Vertical tongue distance toward roof of oral cavity"))
        .with_param(DspParamSchema::knob("lip_opening", "Lip Aperture (Rounded to Spread)", 0.1, 2.5, 1.0, "area", MacroRole::Space, "Frontal lip radiating area opening and protrusion parameter"))
    );
    descriptors.insert("HarmonicPartial".to_string(), DspNodeDescriptor::new("HarmonicPartial", "Sinusoidal Additive Partial Oscillator & Phase Tracker HUD", DspNodeCategory::SpectralResynthesis, "Individual sinusoidal partial state in an additive synthesis engine managing frequency ratio relative to fundamental, linear amplitude, smoothing target, continuous phase, and inharmonic detuning")
        .with_param(DspParamSchema::knob("frequency_ratio", "Harmonic Partial Multiplier Ratio", 0.25, 32.0, 1.0, "ratio", MacroRole::Tone, "Harmonic overtone integer or non-integer frequency multiplier"))
        .with_param(DspParamSchema::knob("partial_amplitude", "Sinusoidal Partial Linear Amplitude", 0.0, 1.0, 0.75, "amp", MacroRole::Punch, "Instantaneous normalized linear amplitude of this sinusoidal oscillator"))
        .with_param(DspParamSchema::knob("smoothing_target_amplitude", "Interpolation Target Amplitude", 0.0, 1.0, 0.75, "amp", MacroRole::Character, "Target amplitude level approaching exponentially to prevent click artifacts"))
        .with_param(DspParamSchema::knob("phase_accumulator_rad", "Continuous Partial Phase", 0.0, 6.283, 0.0, "rad", MacroRole::Space, "Instantaneous phase angle in radians within [0, 2*PI) cycle"))
        .with_param(DspParamSchema::knob("inharmonic_offset_hz", "Microtonal Inharmonic Frequency Offset", -100.0, 100.0, 0.0, "Hz", MacroRole::Tone, "Detuning offset in Hertz simulating wire stiffness inharmonic overtone stretch"))
    );
    descriptors.insert("QuantumTomographyData".to_string(), DspNodeDescriptor::new("QuantumTomographyData", "Quantum Audio State Density Matrix Tomography HUD", DspNodeCategory::SpectralResynthesis, "Quantum state density matrix tomography visualizer and parameter controller plotting Bloch sphere coordinates (x, y, z), quantum state purity, and von Neumann entanglement entropy")
        .with_param(DspParamSchema::knob("bloch_vector_x", "Bloch Sphere State Projection X", -1.0, 1.0, 0.5, "x", MacroRole::Tone, "Bloch sphere coordinate X representing quantum phase coherence on equatorial plane"))
        .with_param(DspParamSchema::knob("bloch_vector_y", "Bloch Sphere State Projection Y", -1.0, 1.0, 0.0, "y", MacroRole::Tone, "Bloch sphere coordinate Y representing complex imaginary phase angle component"))
        .with_param(DspParamSchema::knob("bloch_vector_z", "Bloch Sphere State Projection Z (Population Inversion)", -1.0, 1.0, 0.866, "z", MacroRole::Punch, "Bloch sphere coordinate Z representing population inversion between audio states |0> and |1>"))
        .with_param(DspParamSchema::knob("state_purity", "Quantum Density Matrix State Purity", 0.0, 1.0, 1.0, "pure", MacroRole::Character, "State purity gamma = Tr(rho^2) where 1.0 indicates pure state and <1.0 indicates mixed state"))
        .with_param(DspParamSchema::knob("entanglement_entropy", "Von Neumann Entanglement Entropy", 0.0, 1.0, 0.0, "bits", MacroRole::Space, "Von Neumann entropy S(rho) quantifying entanglement between bipartite audio subsystems"))
    );
    descriptors.insert("Complex32".to_string(), DspNodeDescriptor::new("Complex32", "Cartesian Complex Wave Amplitude State Vector", DspNodeCategory::SpectralResynthesis, "Cartesian complex number amplitude representation containing real in-phase and imaginary quadrature wave components with phase angle calculation and quantum state coherence tracking")
        .with_param(DspParamSchema::knob("re", "Real In-Phase Amplitude", -2.0, 2.0, 1.0, "re", MacroRole::Tone, "In-phase real component of complex audio wavefunction"))
        .with_param(DspParamSchema::knob("im", "Imaginary Quadrature Amplitude", -2.0, 2.0, 0.0, "im", MacroRole::Tone, "Quadrature imaginary component of complex audio wavefunction"))
        .with_param(DspParamSchema::knob("magnitude", "Complex Absolute Magnitude", 0.0, 2.0, 1.0, "mag", MacroRole::Character, "Absolute complex magnitude envelope r = sqrt(re^2 + im^2)"))
        .with_param(DspParamSchema::knob("phase_rad", "Complex Argument Phase Angle", -3.14159, 3.14159, 0.0, "rad", MacroRole::Space, "Complex argument phase angle theta = atan2(im, re)"))
        .with_param(DspParamSchema::knob("coherence_time_ms", "Quantum Phase Coherence Lifetime", 0.1, 500.0, 25.0, "ms", MacroRole::Punch, "Quantum phase coherence decay relaxation constant"))
    );
    descriptors.insert("SpectrogramFrequencyMapping".to_string(), DspNodeDescriptor::new("SpectrogramFrequencyMapping", "Image Spectrogram Art Frequency Scale & Bin Mapping", DspNodeCategory::SpectralResynthesis, "Converts 2D visual graphic artwork into musical audio spectrograms with choice of Linear Hertz, Logarithmic Octaves, Psychoacoustic Mel Scale, Bark Bands, and ERB auditory spacing")
        .with_param(DspParamSchema::choice("frequency_bin_scaling_scale", "Visual-to-Audio Frequency Distribution Scale", &["Linear Scale (Scientific Equal-Hertz Spacing)", "Logarithmic Scale (Octave Musical Equal-Temperament)", "Mel Scale (Human Pitch Psychoacoustics)", "Bark Scale (Critical Auditory Bandwidths)", "Equivalent Rectangular Bandwidth (ERB)"], 1, "Frequency distribution formula across image vertical axis"))
        .with_param(DspParamSchema::log_knob("lowest_mapped_frequency_hz", "Image Bottom Pixel Lowest Audio Frequency", 20.0, 500.0, 65.41, "Hz", MacroRole::Tone, "Audio frequency mapped to the bottom row of pixels"))
        .with_param(DspParamSchema::log_knob("highest_mapped_frequency_hz", "Image Top Pixel Highest Audio Frequency", 2000.0, 20000.0, 14000.0, "Hz", MacroRole::Character, "Audio frequency mapped to the top row of pixels"))
        .with_param(DspParamSchema::knob("oscillator_bank_partial_density", "Total Active Sine Partial Oscillators", 64.0, 1024.0, 256.0, "partials", MacroRole::Punch, "Number of vertical sine partial bins rendered"))
        .with_param(DspParamSchema::knob("time_scan_duration_seconds", "Image Horizontal Scan Duration Length", 1.0, 60.0, 8.0, "s", MacroRole::Space, "Duration in seconds to scan across the image width"))
        .with_param(DspParamSchema::toggle("quantize_frequencies_to_midi_notes", "Snap Bin Frequencies to Nearest Chromatic Semitone", false, "Quantize partial frequencies to standard 12-TET MIDI notes"))
    );
    descriptors.insert("SpectrogramColorMapping".to_string(), DspNodeDescriptor::new("SpectrogramColorMapping", "Spectrogram Pixel Color to Audio Amplitude & Phase Engine", DspNodeCategory::SpectralResynthesis, "Visual color space translation decoding pixel RGB values into partial audio amplitude, stereo panning, initial phase offset, and frequency modulation")
        .with_param(DspParamSchema::choice("pixel_color_decoding_mode", "Pixel Color Translation Engine Mode", &["Monochrome Luminance -> Amplitude Only", "Thermal Heatmap -> Amplitude + Harmonic Exciter", "RGB Triple -> (Red: Amp, Green: Pan, Blue: Phase)", "Alpha Transparency -> Gain Ducking Mask"], 2, "Color channel to audio parameter translation mapping"))
        .with_param(DspParamSchema::knob("dynamic_range_headroom_db", "Visual Black-to-White Dynamic Range Headroom", 24.0, 96.0, 60.0, "dB", MacroRole::Punch, "Total dynamic range span between dark and bright pixels"))
        .with_param(DspParamSchema::knob("color_saturation_to_stereo_spread", "Color Saturation Modulating Stereo Width", 0.0, 1.0, 0.65, "width", MacroRole::Space, "Pixel saturation widening stereo panorama spread"))
        .with_param(DspParamSchema::knob("gamma_contrast_curve", "Pixel Brightness Gamma Exponent Curve", 0.5, 3.0, 1.2, "gamma", MacroRole::Tone, "Gamma transfer curve modifying perceived visual contrast"))
        .with_param(DspParamSchema::knob("noise_threshold_floor", "Black Pixel Noise Floor Gate Cutoff", 0.0, 0.2, 0.03, "floor", MacroRole::Character, "Threshold below which pixels are treated as silence"))
        .with_param(DspParamSchema::toggle("invert_color_polarity", "Invert Black & White Pixel Amplitudes", false, "Invert color values so white is silent and black is maximum gain"))
    );
    descriptors.insert("ColorMappingMode".to_string(), DspNodeDescriptor::new("ColorMappingMode", "Spectrogram Art Optical Color-to-Harmonic Audio Mapping", DspNodeCategory::SpectralResynthesis, "Translates 2D bitmap pixel RGB chromaticity and luminance values into additive sinusoidal harmonics, stereo spatial panning, and harmonic overtone brightness")
        .with_param(DspParamSchema::choice("color_mode", "Pixel Chromaticity to Harmonic Mapping Engine", &["Grayscale Monochrome", "RGB Color-to-Note Harmonic"], 0, "Selects whether image luminance drives pure sine banks or RGB channels drive distinct harmonic voices"))
        .with_param(DspParamSchema::knob("red_channel_pitch_offset", "Red Color Channel Harmonic Interval Offset", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Semitone pitch shift applied to partials triggered by image red channel chromaticity"))
        .with_param(DspParamSchema::knob("green_channel_resonance", "Green Color Channel Filter Resonance Q", 0.0, 1.0, 0.70, "ratio", MacroRole::Space, "Resonant filter emphasis applied to partials triggered by image green chromaticity"))
        .with_param(DspParamSchema::knob("blue_channel_stereo_pan", "Blue Color Channel Stereo Panning Bias", -1.0, 1.0, 0.0, "pan", MacroRole::Space, "Spatial stereo field placement bias modulated by image blue color intensity"))
        .with_param(DspParamSchema::knob("luminance_threshold", "Optical Pixel Dark Noise Floor Threshold", 0.0, 1.0, 0.08, "thresh", MacroRole::Punch, "Luminance cutoff below which pixels generate no audible sine oscillator partials"))
        .with_param(DspParamSchema::knob("gamma_curve_exponent", "Non-linear Pixel Brightness Contrast Gamma", 0.2, 3.0, 1.0, "gamma", MacroRole::Tone, "Dynamic contrast curve exponent shaping perceptual dynamic range of visual sonification"))
    );
    descriptors.insert("FrequencyMapping".to_string(), DspNodeDescriptor::new("FrequencyMapping", "Spectrogram Art Vertical Pixel Frequency Axis Scale", DspNodeCategory::SpectralResynthesis, "Configures vertical pixel coordinate calibration across audio frequency spectrum, mapping bitmap height to linear Hertz or musical logarithmic pitch intervals")
        .with_param(DspParamSchema::choice("frequency_scale", "Vertical Y-Axis Frequency Mapping Calibration", &["Linear Hertz", "Musical Logarithmic"], 1, "Determines whether vertical pixel rows scale linearly in Hertz or logarithmically in musical pitch"))
        .with_param(DspParamSchema::knob("base_frequency_hz", "Bottom Pixel Row Fundamental Audio Frequency", 20.0, 1000.0, 55.0, "Hz", MacroRole::Tone, "Audio frequency corresponding to row Y=0 at the bottom edge of the image canvas"))
        .with_param(DspParamSchema::knob("ceiling_frequency_hz", "Top Pixel Row Maximum Nyquist Limit Frequency", 2000.0, 22050.0, 16000.0, "Hz", MacroRole::Tone, "Audio frequency corresponding to the topmost row of pixels in the spectrogram canvas"))
        .with_param(DspParamSchema::knob("octaves_per_height", "Logarithmic Octave Span Across Canvas Height", 1.0, 10.0, 8.0, "oct", MacroRole::Space, "Total musical octave range spanned from bottom to top of the visual sonification canvas"))
        .with_param(DspParamSchema::knob("cent_quantization_step", "Chromatic Note Intonation Quantization Step", 0.0, 100.0, 0.0, "cents", MacroRole::Punch, "Snaps vertical oscillator frequencies to nearest 12-TET or microtonal pitch grid steps"))
        .with_param(DspParamSchema::toggle("anti_aliasing_oversampling", "2x Sub-Pixel Anti-Aliased Sine Interpolation", true, "Interpolates pixel values between vertical scanlines to prevent high-frequency aliasing"))
    );
    descriptors.insert("SpectralPhaseVocoderPitchShifter".to_string(), DspNodeDescriptor::new("SpectralPhaseVocoderPitchShifter", "STFT Phase-Vocoder Independent Pitch & Formant Transposer", DspNodeCategory::SpectralResynthesis, "Spectral pitch shifter decoupling fundamental pitch shift from acoustic vocal/instrument tract formant characteristics")
        .with_param(DspParamSchema::knob("pitch_shift_semitones", "Chromatic Pitch Shift", -24.0, 24.0, 0.0, "semi", MacroRole::Tone, "Coarse pitch transposition in chromatic semitones without altering duration"))
        .with_param(DspParamSchema::knob("pitch_fine_cents", "Pitch Fine Detune", -50.0, 50.0, 0.0, "cents", MacroRole::Tone, "Microtonal pitch detuning in cents"))
        .with_param(DspParamSchema::knob("formant_shift_semitones", "Formant Spectral Shift", -12.0, 12.0, 0.0, "semi", MacroRole::Character, "Shifts spectral envelope formants to simulate vocal tract size modifications"))
        .with_param(DspParamSchema::choice("spectral_window_size", "FFT Analysis Window Size", &["1024 Samples", "2048 Samples (Optimal)", "4096 Samples (High-Res)", "8192 Samples"], 1, "Spectral bin resolution vs temporal transient response trade-off"))
        .with_param(DspParamSchema::choice("phase_locking_algorithm", "Phase Coherence Locking", &["Identity Phase Locking", "Strict Peak Locking", "Loose Ambient Locking"], 0, "Preserves phase relationships across spectral peaks to eliminate phasiness distortion"))
        .with_param(DspParamSchema::toggle("transient_detector_bypass", "Transient Bypass Preservation", true, "Detects drum transients and bypasses phase vocoder to preserve crisp attack onsets"))
    );
    descriptors.insert("SpectralMorphHarmonizer".to_string(), DspNodeDescriptor::new("SpectralMorphHarmonizer", "FFT Cross-Synthesis Formant Morpher & Chord Harmonizer", DspNodeCategory::SpectralResynthesis, "Dual-carrier spectral cross-synthesis vocoder applying vocal formant envelopes onto harmonic chords with pitch-corrected phase tracking and partial filtering")
        .with_param(DspParamSchema::knob("formant_carrier_blend", "Formant Filter vs Carrier Synth Blend", 0.0, 1.0, 0.5, "ratio", MacroRole::Tone, "Balance between source modulator formant contours and synthesized carrier chord"))
        .with_param(DspParamSchema::choice("fft_analysis_resolution", "Spectral FFT Analysis Resolution", &["1024 Bins (Fast Response)", "2048 Bins (Balanced)", "4096 Bins (Deep Clarity)"], 1, "FFT block size determining spectral formant resolution versus temporal latency"))
        .with_param(DspParamSchema::choice("harmonic_voicing_intervals", "Polyphonic Chord Harmonizer Preset", &["Octave & Fifth (+12/+7)", "Major Triad (+4/+7)", "Minor Triad (+3/+7)", "Open Fifth (+7/+12)"], 0, "Chord interval pattern automatically generated beneath lead vocal or melody"))
        .with_param(DspParamSchema::knob("spectral_tilt_emphasis_db", "High-Frequency Spectral Tilt", -18.0, 18.0, 3.0, "dB", MacroRole::Tone, "Linear frequency slope boosting high vocoder partials for vocal lyric intelligibility"))
        .with_param(DspParamSchema::knob("partial_tracking_bandwidth_hz", "Sine Partial Tracking Bandwidth", 20.0, 500.0, 100.0, "Hz", MacroRole::Character, "Frequency tolerance bandwidth for tracking moving harmonic partial peaks"))
        .with_param(DspParamSchema::toggle("consonant_unvoiced_passthrough", "Pass-Through Unvoiced Sibilance Noise", true, "Injects unvoiced consonants ('s', 't', 'k') directly to preserve diction clarity"))
    );
    descriptors.insert("ZeroGravityFluidResonator".to_string(), DspNodeDescriptor::new("ZeroGravityFluidResonator", "Hydrodynamic Microgravity Bubble & Droplet Acoustic Synthesizer", DspNodeCategory::SpectralResynthesis, "Acoustic physical synthesizer modeling bubble cavitation, droplet surface tension oscillations, and fluid splash dynamics in microgravity environments with Minnaert resonant frequencies")
        .with_param(DspParamSchema::knob("fluid_surface_tension_n_m", "Fluid Surface Tension Coefficient", 0.01, 0.1, 0.0728, "N/m", MacroRole::Tone, "Fluid surface tension (0.0728 N/m for water, 0.022 for alcohol, 0.48 for liquid metal)"))
        .with_param(DspParamSchema::knob("bubble_radius_distribution_mm", "Minnaert Cavity Bubble Radius", 0.1, 10.0, 1.8, "mm", MacroRole::Character, "Median bubble cavity radius governing Minnaert acoustic chirp frequency"))
        .with_param(DspParamSchema::knob("cavitation_implosion_intensity", "Cavitation Bubble Collapse Intensity", 0.0, 1.0, 0.45, "depth", MacroRole::Punch, "Shockwave impulse sharpness from collapsing vapor micro-bubbles"))
        .with_param(DspParamSchema::knob("capillary_wave_modal_damping", "Capillary Surface Wave Damping", 0.01, 0.8, 0.15, "damp", MacroRole::Space, "Viscous damping of droplet shape deformation harmonics"))
        .with_param(DspParamSchema::knob("fluid_viscosity_cst", "Kinematic Fluid Viscosity", 0.5, 50.0, 1.0, "cSt", MacroRole::Tone, "Kinematic fluid viscosity smoothing acoustic surface transients"))
        .with_param(DspParamSchema::knob("ambient_pressure_kpa", "Surrounding Ambient Atmospheric Pressure", 10.0, 200.0, 101.3, "kPa", MacroRole::Space, "Surrounding atmospheric ambient pressure tuning fundamental pitch"))
    );
    descriptors.insert("HarmonicExcitanceSynthesizer".to_string(), DspNodeDescriptor::new("HarmonicExcitanceSynthesizer", "Chebyshev Polynomial Psychoacoustic Harmonic Exciter", DspNodeCategory::SpectralResynthesis, "Harmonic generation processor utilizing high-order Chebyshev polynomial transfer functions and auditory critical band masking to inject coherent harmonic sheen and fundamental presence")
        .with_param(DspParamSchema::choice("chebyshev_order_profile", "Chebyshev Harmonic Order Series", &["Even Octaves (T2 + T4)", "Odd Formants (T3 + T5)", "Full Harmonic Series (T2..T8)", "High Sheen Air (T6..T12)"], 2, "Polynomial series generating mathematically exact harmonic overtones"))
        .with_param(DspParamSchema::knob("critical_band_masking_db", "Psychoacoustic Masking Threshold", -48.0, 0.0, -18.0, "dB", MacroRole::Tone, "Masking threshold preventing generation of audible distortion below program content"))
        .with_param(DspParamSchema::knob("odd_even_harmonic_balance", "Odd/Even Harmonic Balance", -1.0, 1.0, 0.1, "bal", MacroRole::Character, "Balance between tube-like even harmonics (-1.0) and tape/transistor odd harmonics (+1.0)"))
        .with_param(DspParamSchema::knob("transient_flux_sensitivity", "Transient Dynamic Flux Tracking", 0.0, 1.0, 0.6, "flux", MacroRole::Punch, "Dynamic envelope follower boosting exciter drive exclusively on transient peaks"))
        .with_param(DspParamSchema::knob("sub_octave_reinforcement", "Sub-Harmonic Fundamental Weight", 0.0, 1.0, 0.25, "weight", MacroRole::Punch, "Synthesizes psychoacoustic sub-bass fundamental from upper partial intervals"))
        .with_param(DspParamSchema::knob("air_band_sheen_khz", "Ultra-High Air Band Sheen Frequency", 8.0, 24.0, 14.5, "kHz", MacroRole::Tone, "Corner frequency for high-shelf ultrasonic harmonic injection"))
    );
    descriptors.insert("SpectralFormantMorpher".to_string(), DspNodeDescriptor::new("SpectralFormantMorpher", "Dual-Vowel Formant Filter Bank & Vocal Tract Crossfader", DspNodeCategory::SpectralResynthesis, "Continuous vowel transformation engine extracting and applying acoustic vocal tract formants (F1-F5) with LPC envelope interpolation, glottal excitation coupling, and pharyngeal volume scaling")
        .with_param(DspParamSchema::choice("vowel_source_a", "Source Vowel Formant Profile A", &["Vowel /a/ (Open Father)", "Vowel /i/ (Close See)", "Vowel /u/ (Boot)", "Vowel /e/ (Bed)", "Vowel /o/ (Thought)"], 0, "Starting acoustic vowel formant constellation"))
        .with_param(DspParamSchema::choice("vowel_target_b", "Target Vowel Formant Profile B", &["Vowel /a/ (Open Father)", "Vowel /i/ (Close See)", "Vowel /u/ (Boot)", "Vowel /e/ (Bed)", "Vowel /o/ (Thought)"], 1, "Destination acoustic vowel formant constellation"))
        .with_param(DspParamSchema::knob("vowel_morph_crossfade", "Vowel Formant Interpolation Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Character, "Continuous interpolation slider traversing between Vowel A and Vowel B"))
        .with_param(DspParamSchema::knob("formant_shift_semitones", "Vocal Tract Scale Gender Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Geometric scaling of vocal tract dimensions (deep masculine to high feminine/child)"))
        .with_param(DspParamSchema::knob("glottal_excitation_mix", "Internal Glottal Pulse Injection", 0.0, 1.0, 0.3, "mix", MacroRole::Punch, "Blends synthetic glottal buzz with incoming audio excitation"))
        .with_param(DspParamSchema::knob("throat_cavity_scale_ratio", "Pharyngeal Throat Cavity Resonance", 0.5, 2.0, 1.0, "scale", MacroRole::Space, "Low-frequency throat cavity acoustic volume expansion"))
    );
    descriptors.insert("SpectralInharmonicResonator".to_string(), DspNodeDescriptor::new("SpectralInharmonicResonator", "Stiff Inharmonic Bar & Bell Partial Stretch Resonator", DspNodeCategory::SpectralResynthesis, "Fractional modal resonator bank modeling elastic stiffness and high-order partial frequency stretching in steel bars, tubular bells, and thick idiophone plates")
        .with_param(DspParamSchema::knob("stiffness_dispersion_coeff", "Stiffness Dispersion Coefficient", 0.0, 1.0, 0.25, "stiff", MacroRole::Tone, "Material elasticity constant stretching higher-order modal partials"))
        .with_param(DspParamSchema::choice("modal_partial_count", "Resonant Modal Partial Count", &["8 Partials (Lo-Fi Chime)", "16 Partials (Tubular Bell)", "32 Partials (Orchestral Carillon)", "64 Partials (Hyper-Resonant Bar)"], 2, "Density of discrete fractional sine oscillators in resonator bank"))
        .with_param(DspParamSchema::knob("inharmonic_stretch_power", "Inharmonic Stretch Exponent", 1.0, 3.5, 1.75, "pow", MacroRole::Punch, "Polynomial power factor expanding overtone intervals beyond harmonic series"))
        .with_param(DspParamSchema::knob("decay_frequency_damping", "High-Frequency Modal Loss Damping", 0.01, 1.0, 0.35, "damp", MacroRole::Space, "Frequency-dependent loss accelerating decay of ultrasonic partials"))
        .with_param(DspParamSchema::knob("strike_excitation_position", "Bar Strike Excitation Position", 0.01, 0.99, 0.18, "pos", MacroRole::Character, "Physical impact point suppressing modal nodes matching strike geometry"))
        .with_param(DspParamSchema::knob("even_odd_mode_coupling", "Even/Odd Mode Energy Coupling", 0.0, 1.0, 0.5, "mix", MacroRole::Tone, "Cross-coupling coefficient between symmetric and anti-symmetric vibration modes"))
    );
    descriptors.insert("FormantVowelMorphMatrix".to_string(), DspNodeDescriptor::new("FormantVowelMorphMatrix", "2D Vocal Tract Formant Interpolator & Nasal Resonance Matrix", DspNodeCategory::SpectralResynthesis, "Continuous 2D formant vowel space morpher controlling 5 parallel bandpass resonators (F1-F5), nasal cavity antiresonances, and larynx height scaling")
        .with_param(DspParamSchema::knob("vowel_coord_x", "Vowel Space Coordinate X", 0.0, 1.0, 0.35, "x", MacroRole::Tone, "Vowel space horizontal axis mapping between /i/ (front) and /u/ (back)"))
        .with_param(DspParamSchema::knob("vowel_coord_y", "Vowel Space Coordinate Y", 0.0, 1.0, 0.65, "y", MacroRole::Tone, "Vowel space vertical axis mapping between /a/ (open) and /u/ (closed)"))
        .with_param(DspParamSchema::knob("formant_q_factor", "Vocal Tract Formant Bandwidth Q", 2.0, 30.0, 12.0, "Q", MacroRole::Character, "Sharpness of formant resonance peaks shaping vocal vowel articulation"))
        .with_param(DspParamSchema::knob("larynx_tract_length_cm", "Acoustic Vocal Tract Length", 12.0, 22.0, 17.2, "cm", MacroRole::Punch, "Physical vocal tract dimension scaling all formant frequencies proportionally"))
        .with_param(DspParamSchema::knob("nasal_cavity_coupling", "Velopharyngeal Nasal Coupling", 0.0, 1.0, 0.15, "coup", MacroRole::Character, "Acoustic shunt admittance into nasal cavity injecting nasal zeros and anti-formants"))
        .with_param(DspParamSchema::choice("morph_trajectory_profile", "Automated Vowel Path Trajectory", &["Static Coordinate", "Circular Vowel LFO", "Dynamic Velocity Drive", "Chaos Wanderer"], 0, "Autonomous modulation pattern traversing vowel coordinate space"))
    );
    descriptors.insert("SpectralResonanceSuppressor".to_string(), DspNodeDescriptor::new("SpectralResonanceSuppressor", "Dynamic Peak Tracking Notch Filter & Harshness Tamer", DspNodeCategory::SpectralResynthesis, "Real-time psychoacoustic spectral peak detector dynamically generating narrow anti-resonance filters to eradicate harsh room modes, cymbal piercing spikes, and cabinet resonances")
        .with_param(DspParamSchema::knob("suppression_depth_db", "Dynamic Notch Suppression Limit", 0.0, 24.0, 6.5, "dB", MacroRole::Tone, "Maximum dynamic cut applied to detected resonant peaks"))
        .with_param(DspParamSchema::knob("resonance_detection_sensitivity", "Spectral Peak Detection Sensitivity", 0.0, 1.0, 0.68, "sens", MacroRole::Character, "Threshold for identifying ringing resonances above psychoacoustic masking curve"))
        .with_param(DspParamSchema::knob("filter_q_sharpness", "Anti-Resonance Notch Filter Q", 1.0, 50.0, 18.0, "Q", MacroRole::Tone, "Sharpness of dynamic notch filters preventing damage to adjacent musical content"))
        .with_param(DspParamSchema::knob("transient_protection_level", "Transient Attack Protection Guard", 0.0, 1.0, 0.75, "prot", MacroRole::Punch, "Transient onset detector holding off suppression during percussive strikes"))
        .with_param(DspParamSchema::knob("target_frequency_low_bound_hz", "Processing Range Low Frequency Boundary", 100.0, 2000.0, 800.0, "Hz", MacroRole::Tone, "Lower frequency limit for automated resonance hunting"))
        .with_param(DspParamSchema::knob("target_frequency_high_bound_hz", "Processing Range High Frequency Boundary", 2000.0, 18000.0, 9500.0, "Hz", MacroRole::Tone, "Upper frequency limit targeting harsh sibilance and piercing resonances"))
    );
    descriptors.insert("SpectralVocoderModulatorBank".to_string(), DspNodeDescriptor::new("SpectralVocoderModulatorBank", "32-Band Constant-Q Channel Vocoder & Carrier Matrix", DspNodeCategory::SpectralResynthesis, "Classic 32-band analog vocoder topology with third-octave Butterworth bandpass filters, envelope follower smoothing, formant shift transpose, and carrier synthesis routing")
        .with_param(DspParamSchema::choice("vocoder_frequency_band_count", "Filter Bank Channel Resolution", &["8 Bands (Vintage Lo-Fi)", "16 Bands (Classic Synth Vocoder)", "24 Bands (Modern Vocal Clarity)", "32 Bands (Ultra-Definition Vocoder)"], 3, "Number of parallel bandpass analysis and synthesis channels across the audio band"))
        .with_param(DspParamSchema::knob("formant_spectral_shift_ratio", "Formant Frequency Transposition Factor", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Shifts synthesis filter center frequencies relative to analysis filters to alter vocal timbre"))
        .with_param(DspParamSchema::knob("envelope_follower_decay_speed_ms", "Envelope Follower Integration Time Constant", 2.0, 100.0, 15.0, "ms", MacroRole::Character, "Response time smoothing speech modulation envelopes across bands"))
        .with_param(DspParamSchema::choice("carrier_synthesizer_waveform", "Internal Carrier Synthesizer Sound Source", &["Rich Harmonic Saw Wave", "PWM Variable Pulse", "White Noise Unvoiced Sibilance", "External Audio Carrier Input"], 0, "Selects internal carrier oscillator waveform or external carrier audio stream"))
        .with_param(DspParamSchema::knob("sibilance_high_frequency_bypass_db", "Unvoiced Consonant Sibilance Pass-Through", -24.0, 6.0, -3.0, "dB", MacroRole::Punch, "Injects high-frequency consonant sibilance directly for crystal clear speech intelligibility"))
        .with_param(DspParamSchema::knob("stereo_band_interleaving_spread", "Stereo Channel Alternating Band Panning", 0.0, 1.0, 0.45, "%", MacroRole::Space, "Pans odd and even synthesis bands left and right for an expansive stereo image"))
    );
    descriptors.insert("AiIntelligentSpectralUnmasker".to_string(), DspNodeDescriptor::new("AiIntelligentSpectralUnmasker", "AI Deep Psychoacoustic Spectral Conflict Unmasker", DspNodeCategory::SpectralResynthesis, "Neural and psychoacoustic frequency collision detector dynamically carving complementary spectral notches between conflicting audio stems in real-time")
        .with_param(DspParamSchema::knob("unmasking_intensity_db", "Spectral Carving Notching Depth", 0.0, 24.0, 6.5, "dB", MacroRole::Tone, "Maximum dynamic attenuation applied to conflicting psychoacoustic masking bins"))
        .with_param(DspParamSchema::knob("auditory_masking_curve", "Bark Scale Auditory Spread Function", 0.0, 3.0, 1.0, "curve", MacroRole::Character, "Selects psychoacoustic spreading function slope modeling upward spread of masking"))
        .with_param(DspParamSchema::knob("transient_transparency", "Transient Pass-Through Preservation", 0.0, 1.0, 0.85, "%", MacroRole::Punch, "Transient preservation ratio bypassing spectral carving during dynamic attacks"))
        .with_param(DspParamSchema::knob("smoothing_attack_ms", "Spectral Envelope Attack Time", 0.5, 50.0, 5.0, "ms", MacroRole::Punch, "Attack smoothing coefficient for dynamic spectral notch attenuation"))
        .with_param(DspParamSchema::knob("smoothing_release_ms", "Spectral Envelope Release Time", 5.0, 500.0, 50.0, "ms", MacroRole::Space, "Release recovery time for spectral bins following masking cessation"))
        .with_param(DspParamSchema::toggle("sidechain_listen_delta", "Audition Carved Masking Delta", false, "Solos removed spectral interference energy for surgical monitoring"))
    );
    descriptors.insert("InverseFftSpectrogramSonifier".to_string(), DspNodeDescriptor::new("InverseFftSpectrogramSonifier", "Visual-to-Audio Inverse FFT Spectrogram Sonifier", DspNodeCategory::SpectralResynthesis, "Raster image and spectrogram art synthesizer translating 2D bitmap pixels, brightness levels, and color channels into additive sinusoidal spectra")
        .with_param(DspParamSchema::knob("image_scan_speed_px_sec", "Horizontal Pixel Column Scan Rate", 10.0, 500.0, 100.0, "px/s", MacroRole::Tone, "Horizontal image scanning rate in pixels per second controlling tempo"))
        .with_param(DspParamSchema::knob("frequency_span_low_hz", "Lowest Raster Row Acoustic Frequency", 20.0, 500.0, 80.0, "Hz", MacroRole::Tone, "Base acoustic pitch mapped to bottom edge of raster image"))
        .with_param(DspParamSchema::knob("frequency_span_high_hz", "Highest Raster Row Acoustic Frequency", 2000.0, 22000.0, 16000.0, "Hz", MacroRole::Tone, "Ceiling acoustic pitch mapped to top edge of raster image"))
        .with_param(DspParamSchema::knob("brightness_gamma_curve", "Pixel Luminosity Dynamic Response", 0.5, 3.0, 1.4, "gamma", MacroRole::Character, "Gamma power curve mapping pixel luminosity to partial harmonic amplitude"))
        .with_param(DspParamSchema::knob("color_stereo_spread", "RGB Color Hue to Stereo Pan Mapping", 0.0, 1.0, 0.7, "spread", MacroRole::Space, "Stereo position spread derived from image color hue spectrum"))
        .with_param(DspParamSchema::choice("harmonic_quantization_scale", "Frequency Axis Microtonal Quantization", &["Continuous Linear/Log Raster", "12-TET Chromatic Snap", "Microtonal 31-EDO Scale", "Just Intonation Harmonic Series"], 0, "Snaps frequency bins to musical scale intervals or leaves unquantized"))
    );
    descriptors.insert("NeuralTimbreMorphSynthesizer".to_string(), DspNodeDescriptor::new("NeuralTimbreMorphSynthesizer", "Latent-Space Timbral Vector Interpolator & Neural Resynthesizer", DspNodeCategory::SpectralResynthesis, "Deep autoencoder latent-space timbre morpher blending acoustic sound traits across multi-dimensional embedding coordinates in real-time")
        .with_param(DspParamSchema::knob("latent_coord_x", "Latent Vector Dimension X (Brightness)", -1.0, 1.0, 0.0, "x", MacroRole::Character, "Latent embedding coordinate X (Spectral Brightness vs Wood/Body Warmth)"))
        .with_param(DspParamSchema::knob("latent_coord_y", "Latent Vector Dimension Y (Harmonicity)", -1.0, 1.0, 0.0, "y", MacroRole::Tone, "Latent embedding coordinate Y (Acoustic Harmonicity vs Noise/Breathiness)"))
        .with_param(DspParamSchema::knob("timbre_morph_glide_ms", "Latent Coordinate Interpolation Glide", 5.0, 2000.0, 150.0, "ms", MacroRole::Space, "Interpolation slew rate gliding between latent acoustic targets"))
        .with_param(DspParamSchema::knob("decoder_temperature", "Neural Decoder Softmax Temperature", 0.1, 2.0, 0.8, "temp", MacroRole::Tone, "Neural decoding stochastic variance controlling timbral crispness"))
        .with_param(DspParamSchema::knob("formant_envelope_preserve", "Acoustic Formant Envelope Retention", 0.0, 1.0, 0.75, "%", MacroRole::Character, "Preserves acoustic formant vocal resonance structure during morphing"))
        .with_param(DspParamSchema::toggle("quantize_to_acoustic_anchor", "Snap Coordinates to Acoustic Anchors", false, "Snaps continuous latent coordinates to nearest sampled acoustic instrument prototype"))
    );
    descriptors.insert("CepstralEnvelopeMorphSynthesizer".to_string(), DspNodeDescriptor::new("CepstralEnvelopeMorphSynthesizer", "Real-Time True Envelope Cepstral Morphing Synthesizer", DspNodeCategory::SpectralResynthesis, "Spectral resynthesis engine calculating smoothed cepstral envelope spectral representations, allowing non-linear cross-synthesis and formant warping between acoustic sources")
        .with_param(DspParamSchema::knob("cepstral_order_coefficients", "Envelope Cepstral Truncation Order", 4.0, 128.0, 32.0, "coeffs", MacroRole::Tone, "Number of cepstral liftering coefficients smoothing the spectral envelope"))
        .with_param(DspParamSchema::knob("spectral_tilt_bias", "Spectral Spectral Tilt Spectral Bias", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Linear spectral tilt balancing low vs high frequency energy distribution"))
        .with_param(DspParamSchema::knob("envelope_interpolation_ratio", "Cepstral Morphing Cross-Synthesis Ratio", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Continuous interpolation between carrier spectral envelope and modulator cepstral envelope"))
        .with_param(DspParamSchema::knob("formant_shift_semitones", "Formant Center Frequency Pitch Shift", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Shifts vocal/instrument formant envelope independent of fundamental pitch"))
        .with_param(DspParamSchema::knob("carrier_residual_gain", "Synthesized Carrier Harmonic Level", -36.0, 6.0, -6.0, "dB", MacroRole::Punch, "Output level of resynthesized excitation carrier harmonic components"))
        .with_param(DspParamSchema::toggle("enable_harmonic_peak_locking", "Lock Harmonic Spectral Saliencies", true, "Locks cepstral envelope peaks to exact harmonic sinusoidal bins preventing spectral smear"))
    );
    descriptors.insert("HarmonicAdditiveResynthesisEngine".to_string(), DspNodeDescriptor::new("HarmonicAdditiveResynthesisEngine", "128-Partial Additive Resynthesis Engine & Spectral Modeler", DspNodeCategory::SpectralResynthesis, "128-partial discrete additive sinusoidal synthesis engine with dynamic harmonic tilt, inharmonic dispersion, and real-time phase alignment")
        .with_param(DspParamSchema::knob("partial_count_active", "Active Partial Voice Count", 1.0, 128.0, 32.0, "partials", MacroRole::Tone, "Number of active harmonic sinusoids calculated in additive resynthesis"))
        .with_param(DspParamSchema::knob("spectral_centroid_tilt_db", "Spectral Centroid Tilt Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Slope filter tilting harmonic energy toward higher or lower partials"))
        .with_param(DspParamSchema::knob("inharmonicity_dispersion_index", "Inharmonic Partial Frequency Dispersion", 0.0, 1.0, 0.15, "%", MacroRole::Character, "Stiff-string inharmonicity coefficient displacing partials from integer ratios"))
        .with_param(DspParamSchema::knob("phase_alignment_coherence", "Partial Phase Coherence Coupling", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Degree of phase synchronization across partial bank for coherent waveforms"))
        .with_param(DspParamSchema::knob("dynamic_odd_even_harmonic_ratio", "Odd vs Even Harmonic Energy Balance", -1.0, 1.0, 0.0, "bal", MacroRole::Tone, "Crossfade ratio between odd-order and even-order harmonic partials"))
        .with_param(DspParamSchema::toggle("enable_spectral_anti_aliasing", "Nyquist Spectral Anti-Aliasing Guard", true, "Automatically mutes partials exceeding Nyquist frequency to avoid aliasing folds"))
    );
    descriptors.insert("SpectralContrastCompressor".to_string(), DspNodeDescriptor::new("SpectralContrastCompressor", "FFT Spectral Contrast Dynamic Masker & Formant Peak Sharpener", DspNodeCategory::SpectralResynthesis, "Frequency-domain dynamics processor calculating moving contrast ratio between spectral peaks and spectral valleys to sharpen timbre or smooth harsh resonances")
        .with_param(DspParamSchema::knob("spectral_contrast_ratio", "Spectral Peak-To-Valley Contrast Ratio", 0.1, 5.0, 1.8, ":1", MacroRole::Punch, "Compresses or expands disparity between spectral formant peaks and intermediate valleys"))
        .with_param(DspParamSchema::knob("fft_bin_smoothing_width", "FFT Frequency Bin Smoothing Bandwidth", 1.0, 64.0, 8.0, "bins", MacroRole::Tone, "Neighborhood averaging filter smoothing spectral envelope transitions across bins"))
        .with_param(DspParamSchema::knob("dynamic_spectral_threshold_db", "Dynamic Contrast Processing Threshold", -60.0, 0.0, -24.0, "dB", MacroRole::Punch, "Input level threshold below which spectral contrast expansion or reduction occurs"))
        .with_param(DspParamSchema::knob("peak_valley_balance", "Formant Peak vs Valley Weighting", -1.0, 1.0, 0.25, "bal", MacroRole::Tone, "Shifts focus between attenuating resonant formant peaks or lifting silent valleys"))
        .with_param(DspParamSchema::knob("spectral_floor_noise_suppression", "Spectral Noise Floor Gating Threshold", 0.0, 1.0, 0.4, "%", MacroRole::Character, "Gates low-level spectral noise bins to keep background silent between note attacks"))
        .with_param(DspParamSchema::toggle("linear_frequency_bin_spacing", "Linear vs Mel-Scale Frequency Binning", false, "Toggles between linear FFT binning and psychoacoustic Mel-scale frequency grouping"))
    );
    descriptors.insert("WaveletPacketResynthesisEngine".to_string(), DspNodeDescriptor::new("WaveletPacketResynthesisEngine", "Wavelet Packet Multiresolution Decomposition & Resynthesis Engine", DspNodeCategory::SpectralResynthesis, "Multiresolution discrete wavelet packet transform splitting audio into logarithmic frequency subbands with thresholded transient de-noising and dynamic subband energy balancing")
        .with_param(DspParamSchema::knob("decomposition_depth_levels", "Wavelet Decomposition Depth Levels", 1.0, 8.0, 4.0, "lvl", MacroRole::Tone, "Tree decomposition depth controlling frequency resolution across subbands"))
        .with_param(DspParamSchema::knob("threshold_denoise_ratio_db", "Subband Threshold De-Noise Gate", -60.0, 0.0, -28.0, "dB", MacroRole::Character, "Threshold below which noise wavelets are attenuated to purify signal"))
        .with_param(DspParamSchema::knob("subband_energy_tilt_db", "Logarithmic Subband Energy Spectral Tilt", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Linear tilt modifying energy distribution across high vs low wavelet subbands"))
        .with_param(DspParamSchema::choice("wavelet_family_basis", "Mother Wavelet Mathematical Basis", &["Daubechies D4 (Sharp)", "Daubechies D8 (Smooth)", "Symlet Sym4 (Symmetric)", "Coiflet Coif2 (Orthogonal)"], 1, "Selects mother wavelet orthonormal basis function for signal decomposition"))
        .with_param(DspParamSchema::choice("phase_reconstruction_mode", "Wavelet Phase Reconstruction Mode", &["Linear Zero-Phase", "Minimum Phase Causal", "Dispersion Morphing"], 0, "Phase coherence algorithm used when synthesizing reconstructed output"))
        .with_param(DspParamSchema::toggle("enable_perfect_reconstruction_guard", "Perfect Reconstruction Math Guard", true, "Guarantees mathematically bit-perfect lossless reconstruction when processing is dry"))
    );
    descriptors.insert("SpectralFreezeGranularSmear".to_string(), DspNodeDescriptor::new("SpectralFreezeGranularSmear", "Continuous FFT Phase Randomizer & Spectral Smear Cloud", DspNodeCategory::SpectralResynthesis, "High-resolution FFT spectral freeze and stochastic phase-diffusion buffer creating vast lush ambient drones and infinite sustained spectral smears")
        .with_param(DspParamSchema::knob("smear_window_size_ms", "Spectral Analysis Window Size", 10.0, 250.0, 65.0, "ms", MacroRole::Space, "FFT analysis frame window size balancing transient accuracy and frequency resolution"))
        .with_param(DspParamSchema::knob("spectral_diffusion_depth", "Phase Diffusion Randomization Depth", 0.0, 1.0, 0.75, "%", MacroRole::Space, "Amount of randomized angular phase noise injected into FFT magnitude bins"))
        .with_param(DspParamSchema::knob("spectral_brightness_tilt", "Frozen Spectrum Brightness Tilt", -12.0, 12.0, 2.0, "dB", MacroRole::Tone, "Spectral shelf dampening or accentuating high-frequency spectral drone shimmer"))
        .with_param(DspParamSchema::knob("bin_retention_threshold_db", "Spectral Bin Magnitude Gating Threshold", -70.0, 0.0, -35.0, "dB", MacroRole::Character, "Filters out quiet background noise bins to retain prominent musical formants"))
        .with_param(DspParamSchema::choice("phase_scramble_mode", "Phase Scrambling Topology Profile", &["Uniform White Noise", "Brownian Drift (Lush)", "Chaotic Strange Swarm"], 1, "Selects stochastic phase evolution topology for spectral blur"))
        .with_param(DspParamSchema::toggle("freeze_spectral_evolution", "Infinite Spectral Drone Freeze", false, "Locks current spectral snapshot in place indefinitely without decay"))
    );
    descriptors.insert("QuantumMolecularBondResonator".to_string(), DspNodeDescriptor::new("QuantumMolecularBondResonator", "Quantum Molecular Bond Vibrational Frequency Resonator Bank", DspNodeCategory::SpectralResynthesis, "Physical resonator bank simulating quantum mechanical stretching, bending, and torsional vibrational modes of covalent molecular structures")
        .with_param(DspParamSchema::knob("molecular_bond_stiffness_nm", "Covalent Bond Spring Constant Stiffness", 10.0, 500.0, 120.0, "N/m", MacroRole::Punch, "Stiffness of interatomic bond governing fundamental vibrational frequency"))
        .with_param(DspParamSchema::knob("atomic_mass_amu", "Constituent Atomic Mass Unit", 1.0, 250.0, 16.0, "amu", MacroRole::Tone, "Effective reduced mass of vibrating atomic nuclei shaping modal overtone ratios"))
        .with_param(DspParamSchema::knob("anharmonic_potential_depth", "Morse Anharmonic Potential Depth", 0.0, 1.0, 0.25, "Morse", MacroRole::Character, "Non-linear Morse potential curvature introducing authentic overtone detuning"))
        .with_param(DspParamSchema::knob("vibrational_damping_ps", "Intramolecular Relaxation Lifetime", 0.1, 50.0, 5.0, "ps", MacroRole::Space, "Time constant governing phonon dissipation into surrounding solvent matrix"))
        .with_param(DspParamSchema::knob("thermal_temperature_kelvin", "Ambient Absolute Temperature", 0.0, 1000.0, 293.15, "K", MacroRole::Tone, "Thermal population of higher vibrational quantum states inducing spectral broadening"))
        .with_param(DspParamSchema::toggle("enable_quantum_zero_point_energy", "Quantum Zero-Point Energy Fluctuation", true, "Maintains minimum zero-point ground state vibration preventing complete silence"))
    );
    descriptors.insert("VocalPitchFormantRetuner".to_string(), DspNodeDescriptor::new("VocalPitchFormantRetuner", "Neural Vocal Pitch & Formant Retuner with Gender Transposition", DspNodeCategory::SpectralResynthesis, "Sub-millisecond pitch correction engine with neural formant throat-length preservation, natural vibrato tracking, and microtonal scale snapping")
        .with_param(DspParamSchema::knob("pitch_correction_speed_ms", "Pitch Retune Correction Speed", 0.0, 100.0, 20.0, "ms", MacroRole::Punch, "Speed of pitch snapping: 0 ms for robotic hard-tune effect, 25 ms for natural transparent tuning"))
        .with_param(DspParamSchema::knob("pitch_quantize_scale_snap", "Musical Scale Snapping Pull Factor", 0.0, 1.0, 1.0, "%", MacroRole::Character, "Pulls pitch toward the nearest note in the selected scale"))
        .with_param(DspParamSchema::knob("formant_throat_length_shift", "Vocal Tract Throat Length Shift", -12.0, 12.0, 0.0, "semitones", MacroRole::Tone, "Shifts vocal formants independently from pitch to alter perceived physical throat size"))
        .with_param(DspParamSchema::knob("vocal_gender_morph_ratio", "Male to Female Timbral Gender Morph", -1.0, 1.0, 0.0, "morph", MacroRole::Tone, "Smooth morphing factor from deep masculine resonance to bright feminine presence"))
        .with_param(DspParamSchema::knob("natural_vibrato_preservation", "Natural Vibrato Depth Preservation", 0.0, 1.0, 0.8, "%", MacroRole::Space, "Preserves singer's natural vocal vibrato while centering mean fundamental pitch"))
        .with_param(DspParamSchema::toggle("enable_neural_glottal_pulse_modeling", "Neural Glottal Pulse Resynthesis", true, "Synthesizes authentic physiological glottal pulse excitation during pitch transposition"))
    );
    descriptors.insert("MultibandSpectralContrastExpander".to_string(), DspNodeDescriptor::new("MultibandSpectralContrastExpander", "Bark-Scale Psychoacoustic Spectral Contrast Enhancer", DspNodeCategory::SpectralResynthesis, "Dynamic spectral contrast expander measuring spectral peaks vs valleys within critical Bark auditory filters to heighten timbral resolution")
        .with_param(DspParamSchema::knob("contrast_expansion_ratio", "Spectral Peak-to-Valley Expand Ratio", 1.0, 4.0, 1.6, "x", MacroRole::Punch, "Expansion slope pulling spectral peaks upward while pushing valley floors down"))
        .with_param(DspParamSchema::knob("bark_filter_band_count", "Critical Bark Frequency Band Subdivisions", 6.0, 32.0, 24.0, "bands", MacroRole::Tone, "Auditory Bark scale frequency bands evaluated across the audible spectrum"))
        .with_param(DspParamSchema::knob("valley_spectral_attenuation_db", "Spectral Valley Damping Floor", 0.0, 18.0, 4.5, "dB", MacroRole::Character, "Maximum depth by which masking spectral clutter between peaks is attenuated"))
        .with_param(DspParamSchema::knob("transient_contrast_boost_db", "Harmonic Formant Peak Accentuation", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Peak reinforcement enhancing definition of vocal formants and solo lines"))
        .with_param(DspParamSchema::knob("spectral_smoothing_window_ms", "Inter-Band Smoothing Window Time", 2.0, 100.0, 15.0, "ms", MacroRole::Space, "Time smoothing envelope preventing unnatural spectral flutter or musical noise"))
        .with_param(DspParamSchema::toggle("enable_formant_envelope_preservation", "Preserve Global Formant Spectral Tilting", true, "Maintains overall tonal balance while sharpening individual harmonic peaks"))
    );
    descriptors.insert("WaveletTransientSplayDeconstruct".to_string(), DspNodeDescriptor::new("WaveletTransientSplayDeconstruct", "Continuous Wavelet Transform Multi-Scale Transient Splay & Deconstruct", DspNodeCategory::SpectralResynthesis, "Deconstructs complex audio wavefronts across continuous Morlet wavelet scales to isolate micro-transients, body sustain, and diffuse tails with zero pre-ringing")
        .with_param(DspParamSchema::knob("wavelet_scale_resolution_octaves", "Dyadic Wavelet Scale Decomposition", 1.0, 12.0, 6.0, "oct", MacroRole::Tone, "Number of dyadic wavelet decomposition octaves analyzed"))
        .with_param(DspParamSchema::knob("morlet_wavelet_time_frequency_tradeoff", "Morlet Mother Wavelet Center Q", 0.5, 8.0, 2.0, "Q", MacroRole::Punch, "Morlet mother wavelet center frequency trading time vs frequency resolution"))
        .with_param(DspParamSchema::knob("transient_energy_threshold_db", "Micro-Transient Separation Threshold", -60.0, 0.0, -24.0, "dB", MacroRole::Punch, "Threshold separating sharp transient coefficients from continuous sustain"))
        .with_param(DspParamSchema::knob("body_resonance_isolation_ratio", "Tonal Body Sustain Multiplier", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Gain multiplier applied to tonal body sustain coefficients"))
        .with_param(DspParamSchema::knob("transient_time_splay_spread_ms", "Transient Decorrelation Time Splay", 0.0, 50.0, 8.0, "ms", MacroRole::Space, "Time-offsets isolated transients to create psychoacoustic stereo width"))
        .with_param(DspParamSchema::toggle("enable_perfect_reconstruction_frame", "Biorthogonal Dual Frame Reconstruction", true, "Enforces biorthogonal dual frame ensuring exact bit-perfect reconstruction when untouched"))
    );
    descriptors.insert("BilateralSpectralDenoise".to_string(), DspNodeDescriptor::new("BilateralSpectralDenoise", "Bilateral Spectral Domain Non-Local Means Audio Denoiser", DspNodeCategory::SpectralResynthesis, "Advanced bilateral spectral denoising algorithm filtering background hiss, hum, and continuous noise without musical smearing or watery artifacts")
        .with_param(DspParamSchema::knob("spectral_noise_reduction_depth_db", "Spectral Denoising Attenuation Depth", 0.0, 36.0, 14.0, "dB", MacroRole::Punch, "Maximum decibels of unwanted background noise attenuation"))
        .with_param(DspParamSchema::knob("noise_profile_learning_adaptation", "Continuous Noise Profile Adaptation Rate", 0.0, 1.0, 0.5, "learn", MacroRole::Character, "Speed of continuous background noise profile tracking and updating"))
        .with_param(DspParamSchema::knob("bilateral_spectral_kernel_width", "Bilateral Spectral Neighborhood Radius", 1.0, 16.0, 4.0, "bins", MacroRole::Tone, "Frequency neighborhood radius preserving steep harmonic formant peaks"))
        .with_param(DspParamSchema::knob("musical_noise_artifact_suppression", "Musical Noise Chirp Artifact Suppression", 0.0, 1.0, 0.8, "%", MacroRole::Character, "Suppresses isolated fluttering FFT bins ('musical chirps') in silent gaps"))
        .with_param(DspParamSchema::knob("transient_preservation_sensitivity", "Consonant & Transient Protection Gate", 0.1, 2.0, 1.2, "sens", MacroRole::Punch, "Protects fast drum attacks and speech consonants from noise reduction clamping"))
        .with_param(DspParamSchema::toggle("enable_residual_noise_listen_mode", "Residual Difference Noise Audition", false, "Allows auditioning the removed noise residue to verify zero musical loss"))
    );
    descriptors.insert("DynamicSpectralTransientSplitter".to_string(), DspNodeDescriptor::new("DynamicSpectralTransientSplitter", "STFT Dynamic Spectral Transient & Sustain Splitter", DspNodeCategory::SpectralResynthesis, "Separates audio in real-time into pure transient bursts and sustained tonal components using short-time Fourier transform phase continuity and energy derivative tracking")
        .with_param(DspParamSchema::knob("transient_detection_sensitivity_db", "Transient Energy Derivative Sensitivity", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Energy rise threshold triggering transient component isolation"))
        .with_param(DspParamSchema::knob("transient_sustain_balance_db", "Transient vs Sustain Output Balance", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Relative volume balance between transient attack and sustained body"))
        .with_param(DspParamSchema::knob("spectral_fft_window_size", "STFT Transform Window Resolution", 256.0, 4096.0, 1024.0, "bins", MacroRole::Character, "STFT analysis window length trading temporal vs frequency resolution"))
        .with_param(DspParamSchema::knob("transient_decay_hold_time_ms", "Transient Envelope Isolation Window", 1.0, 100.0, 15.0, "ms", MacroRole::Punch, "Decay window length allocated to isolated transient bursts"))
        .with_param(DspParamSchema::knob("sustain_spectral_smoothing_pct", "Tonal Body Temporal Smoothing", 0.0, 100.0, 65.0, "%", MacroRole::Space, "Temporal smoothing applied to tonal sustain path preventing gating flutter"))
        .with_param(DspParamSchema::toggle("enable_transient_solo_listen", "Audition Isolated Transient Path Solo", false, "Mutes tonal sustain path to solo isolated transient spikes for calibration"))
    );
    descriptors.insert("MorphologicalSpectrogramFilter".to_string(), DspNodeDescriptor::new("MorphologicalSpectrogramFilter", "Mathematical Morphology Spectrogram Filter", DspNodeCategory::SpectralResynthesis, "Applies morphological grayscale operators (erosion, dilation, opening, and closing) to STFT time-frequency frames for surgical transient preservation and noise floor suppression")
        .with_param(DspParamSchema::knob("structuring_element_time_bins", "Structuring Element Time Width", 1.0, 32.0, 5.0, "bins", MacroRole::Tone, "Temporal footprint length of the morphological structural kernel"))
        .with_param(DspParamSchema::knob("structuring_element_freq_bins", "Structuring Element Frequency Height", 1.0, 32.0, 3.0, "bins", MacroRole::Punch, "Spectral bandwidth height of the morphological structural kernel"))
        .with_param(DspParamSchema::knob("morphological_operation_mode", "Morphology Mode (Open/Close/Erode/Dilate)", 0.0, 3.0, 0.0, "mode", MacroRole::Character, "Mathematical morphological filter operation type"))
        .with_param(DspParamSchema::knob("spectral_mask_contrast_gain_db", "Spectral Contrast Elevation Gain", 0.0, 36.0, 12.0, "dB", MacroRole::Tone, "Contrast dynamic range expansion applied to morphological mask"))
        .with_param(DspParamSchema::knob("temporal_frame_overlap_ratio", "STFT Analysis Frame Overlap", 2.0, 8.0, 4.0, "x", MacroRole::Space, "Overlap factor for Short-Time Fourier Transform windowing"))
        .with_param(DspParamSchema::toggle("enable_residual_signal_monitoring", "Solo Morphological Residual Difference", false, "Solos discarded audio signal for precise artifact auditioning"))
    );
    descriptors.insert("AdaptiveSpectralSubtractDeNoiser".to_string(), DspNodeDescriptor::new("AdaptiveSpectralSubtractDeNoiser", "Adaptive Minimum-Statistics Spectral De-Noiser", DspNodeCategory::SpectralResynthesis, "Real-time FFT noise reduction engine utilizing continuous minimum-statistics noise floor tracking, Wiener post-filtering, and musical noise suppression")
        .with_param(DspParamSchema::knob("noise_reduction_depth_db", "Spectral Subtraction Attenuation Depth", 0.0, 48.0, 18.0, "dB", MacroRole::Tone, "Maximum attenuation applied to stationary noise frequency bins"))
        .with_param(DspParamSchema::knob("noise_profile_learning_rate", "Minimum Statistics Tracking Speed", 0.1, 10.0, 1.5, "s", MacroRole::Character, "Adaptation speed for background acoustic noise floor estimation"))
        .with_param(DspParamSchema::knob("transient_preservation_threshold_db", "Transient Signal Guard Threshold", 3.0, 24.0, 12.0, "dB", MacroRole::Punch, "Transient onset sensitivity preventing attack blurring or pre-echo"))
        .with_param(DspParamSchema::knob("musical_noise_oversubtraction_factor", "Oversubtraction Factor (Musical Tone Guard)", 1.0, 4.0, 1.6, "mult", MacroRole::Space, "Oversubtraction ratio eliminating chirping musical noise artifacts"))
        .with_param(DspParamSchema::knob("spectral_smoothing_bandwidth_hz", "Spectral Mask Inter-Bin Smoothing", 20.0, 500.0, 120.0, "Hz", MacroRole::Tone, "Frequency bandwidth for Gaussian smoothing across adjacent STFT bins"))
        .with_param(DspParamSchema::toggle("enable_residual_noise_audition", "Listen Residual Noise Delta", false, "Solos removed background noise components for surgical threshold tuning"))
    );
    descriptors.insert("DynamicSpectralTransientExciter".to_string(), DspNodeDescriptor::new("DynamicSpectralTransientExciter", "Spectral Transient Dynamic Accentuator", DspNodeCategory::SpectralResynthesis, "FFT phase-vocoder spectral transient separator splitting incoming audio into sinusoidal stationary components and non-stationary transient bursts with independent dynamics")
        .with_param(DspParamSchema::knob("transient_boost_gain_db", "Transient Burst Amplification Gain", -18.0, 24.0, 6.0, "dB", MacroRole::Punch, "Gain applied exclusively to explosive transient energy"))
        .with_param(DspParamSchema::knob("tonal_bed_sustain_gain_db", "Tonal Splay Stationary Component Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Gain applied to stationary pitched and drone spectral lines"))
        .with_param(DspParamSchema::knob("spectral_flux_threshold_db", "Spectral Flux Transient Detector Sensitivity", 1.0, 20.0, 6.5, "dB", MacroRole::Character, "Sensitivity threshold for identifying transient onsets across STFT bins"))
        .with_param(DspParamSchema::knob("high_frequency_transient_tilt_db", "High-Frequency Transient Brightness Tilt", -12.0, 12.0, 3.0, "dB", MacroRole::Tone, "Shelving boost on upper transient components adding snappy sheen"))
        .with_param(DspParamSchema::knob("transient_release_smoothing_ms", "Transient Envelope Decay Recovery", 1.0, 100.0, 15.0, "ms", MacroRole::Space, "Release duration for transient amplification envelope"))
        .with_param(DspParamSchema::toggle("enable_transient_only_monitoring", "Solo Extracted Transient Signal", false, "Mutes tonal audio allowing surgical monitoring of isolated transients"))
    );
    descriptors.insert("QuantumEntanglementPhaseVocoder".to_string(), DspNodeDescriptor::new("QuantumEntanglementPhaseVocoder", "Quantum Entangled Multi-Channel Phase Vocoder", DspNodeCategory::SpectralResynthesis, "Bipartite entangled STFT phase vocoder synchronizing phase unwrapping across arbitrary stereo and multichannel stems without phase smearing or transient diffusion")
        .with_param(DspParamSchema::knob("entanglement_correlation_fidelity", "Quantum State Entanglement Fidelity", 0.0, 1.0, 0.95, "fid", MacroRole::Character, "Cross-channel phase coherence and quantum correlation rigidity"))
        .with_param(DspParamSchema::knob("stft_fft_analysis_window_size", "Analysis Frame Window Size", 256.0, 8192.0, 2048.0, "samples", MacroRole::Tone, "Size of analysis STFT frame balancing frequency resolution vs temporal precision"))
        .with_param(DspParamSchema::knob("time_stretch_expansion_ratio", "Time-Scale Invariance Expansion Ratio", 0.1, 8.0, 1.5, "ratio", MacroRole::Space, "Temporal time-stretching factor preserving fundamental and formant pitches"))
        .with_param(DspParamSchema::knob("spectral_bin_phase_locking_tightness", "Inter-Bin Phase Locking Rigidity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Locks adjacent spectral bin phases together preventing reverberant smearing"))
        .with_param(DspParamSchema::knob("transient_preservation_threshold_db", "Transient Preservation Detection Threshold", -60.0, 0.0, -24.0, "dB", MacroRole::Punch, "Sensitivity for bypassing phase vocoder processing on sharp percussion attacks"))
        .with_param(DspParamSchema::toggle("enable_zero_phase_reconstruction_lattice", "Zero-Phase Reconstruction Lattice", true, "Eliminates inter-channel phase cancellation during deep pitch transposition"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "SpectralNeuralResynthesizerNode" => Box::new(
            GenericDspNodeUi::new("SpectralNeuralResynthesizerNode", "Latent FFT Timbre Morphing Resynthesizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("latent_timbre", "Latent Timbre Coordinate", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("spectral_res", "FFT Bin Resolution", 64, 1024, 256, "bins", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("transient_boost", "Transient Preservation", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("smear", "Phase Spectral Smear", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
        ),
        "MultiChannelSpectralEqualizerNode" => Box::new(
            GenericDspNodeUi::new("MultiChannelSpectralEqualizerNode", "64-Band Multi-Channel Spectral Sculptor", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("tilt_slope", "Spectral Tilt Slope", -6.0, 6.0, 0.0, "dB/oct", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("air_band", "High Air Presence", 0.0, 12.0, 2.0, "dB", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("low_sub", "Sub Bass Focus", 0.0, 12.0, 1.5, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("dynamic_unmask", "Dynamic Masking Reduction", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
        ),
        "SpectralMatchingEqNode" => Box::new(
            GenericDspNodeUi::new("SpectralMatchingEqNode", "AI 128-Band Spectral Curve Matching EQ", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_enum("target_curve", "Target Spectral Profile", vec!["Reference Track Target".into(), "Harman Target".into(), "Pink Noise Curve (1/f)".into(), "Commercial Master".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("matching_intensity", "Curve Match Depth", 0.0, 100.0, 75.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("smoothing_octaves", "Spectral Smoothing", 0.1, 2.0, 0.5, "oct", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("gain_limit_db", "Max Boost/Cut Limit", 1.0, 18.0, 6.0, "dB", (0, 230, 118)))
        ),
        "SpectralTiltNode" => Box::new(
            GenericDspNodeUi::new("SpectralTiltNode", "Linear Phase Psychoacoustic Spectral Tilt Filter", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("tilt_slope_db_oct", "Bode Spectral Tilt", -6.0, 6.0, 1.5, "dB/oct", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("pivot_freq_hz", "Center Pivot Frequency", 200.0, 5000.0, 1000.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("air_lift_db", "Top End Air Sheen", 0.0, 6.0, 1.2, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("sub_weight_db", "Sub-Bass Weight", -6.0, 6.0, 0.0, "dB", (0, 230, 118)))
        ),
        "VocoderMatrixNode" => Box::new(
            GenericDspNodeUi::new("VocoderMatrixNode", "32-Band Spectral Vocoder Analysis & Synthesis Matrix", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_int("band_count", "Analysis Filter Bands", 8, 32, 24, "bands", (0, 184, 217)))
                .with_param(DspParamDescriptor::new_linear("formant_warp", "Spectral Formant Warp", 0.5, 2.0, 1.0, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("sibilance_thru", "Unvoiced Noise Pass-thru", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("carrier_drive", "Carrier Excitation Drive", 1.0, 10.0, 2.5, "x", (0, 230, 118)))
        ),
        "AuditoryRoughnessNode" => Box::new(
            GenericDspNodeUi::new("AuditoryRoughnessNode", "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("dissonance_threshold", "Roughness Detection Sens", 0.0, 1.0, 0.65, "%", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("roughness_scaling", "Plomp-Levelt Critical Scale", 0.1, 3.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("critical_bandwidth_mode", "Filter Bank Standard", vec!["Bark Scale (Zwicker)".into(), "ERB Scale (Moore-Glasberg)".into(), "Mel Scale".into()], 1, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("masking_depth", "Simultaneous Masking Depth", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "SpectralAlignerNode" => Box::new(
            GenericDspNodeUi::new("SpectralAlignerNode", "Multi-Track Phase & Spectral Coherence Auto-Aligner", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_int("reference_track_id", "Reference Master Track ID", 0, 64, 0, "trk", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("phase_correlation_target", "Target Phase Alignment", 0.5, 1.0, 0.95, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("delay_offset_samples", "Sub-Sample Delay Offset", -500.0, 500.0, 0.0, "samples", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("polarity_auto_invert", "Auto Polarity Inversion", true, (0, 230, 118)))
        ),
        "SpectralDebleedNode" => Box::new(
            GenericDspNodeUi::new("SpectralDebleedNode", "Microphone Acoustic Bleed Elimination Matrix", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("bleed_suppression_db", "Bleed Isolation Depth", 0.0, 30.0, 14.0, "dB", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_gate_threshold", "Spectral Gate Floor", -60.0, -10.0, -35.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("crosstalk_estimate_ms", "Acoustic Delay Estimate", 0.1, 50.0, 8.5, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("transient_preserve", "Preserve Direct Transients", true, (0, 230, 118)))
        ),
        "SpectralDeEsserNode" => Box::new(
            GenericDspNodeUi::new("SpectralDeEsserNode", "Dynamic Spectral Sibilance & Harshness Reducer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_log("sibilance_center_hz", "Harshness Center Freq", 3000.0, 14000.0, 7200.0, "Hz", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("bandwidth_oct", "Sibilance Bandwidth", 0.2, 3.0, 1.2, "oct", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("max_reduction_db", "Max Dynamic Reduction", 0.0, 24.0, 9.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer Time", 0.0, 10.0, 2.5, "ms", (0, 230, 118)))
        ),
        "SpectralFlatnessNode" => Box::new(
            GenericDspNodeUi::new("SpectralFlatnessNode", "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("entropy_threshold", "Spectral Flatness Cutoff", 0.0, 1.0, 0.45, "entropy", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_int("analysis_window_size", "FFT Window Analysis Size", 256, 4096, 1024, "bins", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("tonal_split_gain", "Tonal Harmonic Level", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("noise_split_gain", "Noise Residual Level", -12.0, 12.0, 0.0, "dB", (0, 230, 118)))
        ),
        "SpectralGrainCloudNode" => Box::new(
            GenericDspNodeUi::new("SpectralGrainCloudNode", "Spectral Domain Grain Scatter & Diffusion Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("grain_density_spectral", "Frequency Bin Grains", 10.0, 500.0, 120.0, "grains/s", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("time_smear_factor", "Phase Time Smear", 0.0, 1.0, 0.35, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("phase_randomization", "Bin Phase Chaos", 0.0, 1.0, 0.4, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("diffusion_wet_mix", "Grain Cloud Blend", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
        ),
        "SpectralMaskingNode" => Box::new(
            GenericDspNodeUi::new("SpectralMaskingNode", "Psychoacoustic Simultaneous Spectral Masking Visualizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("masking_threshold_offset_db", "Masking Threshold Bias", -12.0, 12.0, 0.0, "dB", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("critical_band_scale", "Critical Band Width Scale", 0.5, 2.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("bark_scale_mode", "Psychoacoustic Standard", vec!["Bark Scale Matrix".into(), "ERB Auditory Filter".into(), "Equivalent Rectangular Band".into()], 0, (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("alert_color_intensity", "Masking Flare Brightness", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
        ),
        "SpectralMorphNode" => Box::new(
            GenericDspNodeUi::new("SpectralMorphNode", "Latent FFT Magnitude & Phase Interpolation Morph Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("morph_position", "Spectral Morph Interpolation", 0.0, 1.0, 0.5, "%", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_smear", "Cross-Bin Spectral Smear", 0.0, 1.0, 0.2, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("phase_alignment_mode", "Phase Interpolation Mode", vec!["Magnitude Only (Zero Phase)".into(), "Phase Unwrapped Morph".into(), "Cross-Correlation Aligned".into()], 1, (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("harmonic_lock", "Lock Harmonic Partials", true, (0, 230, 118)))
        ),
        "SpectralReshaperNode" => Box::new(
            GenericDspNodeUi::new("SpectralReshaperNode", "Dynamic Spectral Envelope & Formant Reshaper", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Spectral Envelope Shift", -24.0, 24.0, 0.0, "st", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_envelope_gain", "Envelope Reshape Gain", -18.0, 18.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("smoothing_factor", "Envelope Cepstral Smoothing", 0.1, 5.0, 1.2, "oct", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("dynamic_tracking", "Dynamic Pitch Tracking", true, (0, 230, 118)))
        ),
        "SpectralResynthesisNode" => Box::new(
            GenericDspNodeUi::new("SpectralResynthesisNode", "Harmonic Additive Spectral Resynthesis Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_int("max_partials_count", "Additive Sinusoidal Partials", 8, 256, 64, "partials", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_threshold_db", "Noise Floor Cutoff", -80.0, -20.0, -50.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_drift", "Partial Inharmonicity", 0.0, 2.0, 0.0, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("harmonic_additive_mix", "Synthesized Partials Level", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
        ),
        "SpectralUnmaskerNode" => Box::new(
            GenericDspNodeUi::new("SpectralUnmaskerNode", "AI-Driven Intelligent Sidechain Spectral Unmasker", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_int("sidechain_input_id", "Sidechain Trigger Track", 0, 64, 1, "trk", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("unmasking_depth_db", "Dynamic Unmask Depth", 0.0, 18.0, 5.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("frequency_resolution_bins", "Spectral Bin Resolution", 32, 512, 128, "bins", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("adaptation_speed_ms", "Fast Reaction Speed", 5.0, 250.0, 35.0, "ms", (0, 230, 118)))
        ),
        "Spectrogram3DNode" => Box::new(
            GenericDspNodeUi::new("Spectrogram3DNode", "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_int("fft_size_bins", "FFT Analysis Size", 256, 4096, 1024, "bins", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_int("waterfall_depth_frames", "Waterfall History Buffer", 30, 300, 120, "frames", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("color_palette", "Spectrogram Palette", vec!["Cyberpunk Neon".into(), "Thermal Infrared".into(), "Obsidian Ice".into(), "Grayscale Contrast".into()], 0, (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("frequency_scale_log", "Logarithmic Frequency Grid", true, (0, 230, 118)))
        ),
        "StereoVectorscopeNode" => Box::new(
            GenericDspNodeUi::new("StereoVectorscopeNode", "High-Speed Phosphor Stereophonic Vectorscope Analyzer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("phosphor_decay_rate", "Phosphor Glow Persistence", 0.05, 1.0, 0.35, "s", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("vectorscope_gain", "Lissajous Input Scale", 0.5, 4.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("lissajous_dot_size", "Vector Trace Thickness", 1.0, 5.0, 1.8, "pt", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("correlation_meter_bar", "Show Phase Bar Meter", true, (0, 230, 118)))
        ),
        "SpectrogramArtEngine" => Box::new(
            GenericDspNodeUi::new("SpectrogramArtEngine", "Visual Spectrogram BMP/PNG Image-to-Sound Synthesizer Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_log("min_freq_hz", "Lowest Spectrogram Frequency", 20.0, 500.0, 60.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("max_freq_hz", "Highest Spectrogram Frequency", 1000.0, 22000.0, 12000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("time_scale_sec", "Playback Duration", 0.5, 60.0, 8.0, "s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("brightness_gain", "Pixel Brightness Amplitude", 0.0, 3.0, 1.0, "x", (99, 102, 241)))
        ),
        "SpectrogramArtMorpher" => Box::new(
            GenericDspNodeUi::new("SpectrogramArtMorpher", "2D Spectrogram Visual Morph & Cross-Synthesis Morpher", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("morph_position", "Spectrogram Morph Position", 0.0, 1.0, 0.50, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_smear", "Temporal Smear Window", 0.0, 1.0, 0.20, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("phase_coherence", "Phase Coherence Lock", 0.0, 1.0, 0.85, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("harmonic_preserve", "Harmonic Ridge Strategy", vec!["Ridge Peak Lock".into(), "Smooth Linear Crossfade".into()], 0, (99, 102, 241)))
        ),
        "SpectrumMatchingAnalyzer" => Box::new(
            GenericDspNodeUi::new("SpectrumMatchingAnalyzer", "Multi-Band Target Spectrum Curve Matching & Correction Generator", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_enum("num_bands", "Frequency Band Resolution", vec!["32 Bands (Fast)".into(), "64 Bands (Standard)".into(), "128 Bands (Mastering)".into(), "256 Bands (Ultra)".into()], 2, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("matching_intensity", "Spectral Convergence Amount", 0.0, 1.0, 0.75, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("smoothing_q", "Band Curve Smoothing", 0.2, 5.0, 1.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("max_gain_db", "Max Boost / Cut Boundary", 1.0, 18.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("freeze_snapshot", "Freeze Reference Spectrum", false, (16, 185, 129)))
        ),
        "VowelTrajectoryGraph" => Box::new(
            GenericDspNodeUi::new("VowelTrajectoryGraph", "2D F1/F2 Vowel Formant Trajectory & Morphing Spline HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("vowel_interpolation_speed_ms", "Formant Morphing Transition Duration", 10.0, 1000.0, 150.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("f1_base_frequency_hz", "Vowel First Formant (F1) Jaw/Mouth Height", 200.0, 1200.0, 500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("f2_base_frequency_hz", "Vowel Second Formant (F2) Tongue Position", 600.0, 3500.0, 1500.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_scale_factor", "Vocal Tract Anatomy Scale Scaling Factor", 0.7, 1.4, 1.0, "scale", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("trajectory_loop_mode", "Spline Trajectory Navigation Cycle Mode", vec!["One-Shot".into(), "Ping-Pong Loop".into(), "Continuous Forward Loop".into(), "LFO Modulated".into()], 1, (34, 197, 94)))
        ),
        "VowelCylinderAreaSynthesizer" => Box::new(
            GenericDspNodeUi::new("VowelCylinderAreaSynthesizer", "44-Cylinder Acoustic Area Vocal Tract Formant Synthesizer HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("glottis_vocal_tract_length_cm", "Acoustic Vocal Tract Length", 12.0, 22.0, 17.5, "cm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("lip_radiation_aperture_cm2", "Lip Termination Radiation Aperture Area", 0.2, 8.0, 2.5, "cm²", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pharyngeal_constriction_pos", "Tongue & Pharyngeal Constriction Center", 0.0, 1.0, 0.45, "norm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_damping_factor", "Acoustic Tube Wall Viscous Damping", 0.8, 0.999, 0.985, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("acoustic_nasal_coupling", "Engage Velopharyngeal Nasal Cavity Coupling", false, (34, 197, 94)))
        ),
        "FormantTrajectoryPoint" => Box::new(
            GenericDspNodeUi::new("FormantTrajectoryPoint", "2D Vocal Tract Formant Space Spline Trajectory Interpolator HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_enum("vowel_phoneme_target", "Target Vocal Phoneme & Articulatory Vowel Space", vec!["Vowel /a/ (Father - Low Back)".into(), "Vowel /i/ (See - High Front)".into(), "Vowel /u/ (Boot - High Back Rounded)".into(), "Vowel /e/ (Bed - Mid Front)".into(), "Vowel /o/ (Law - Mid Back)".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("formant_f1_hz", "First Formant Frequency F1 (Jaw Aperture & Vowel Height)", 150.0, 1200.0, 700.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("formant_f2_hz", "Second Formant Frequency F2 (Tongue Advancement)", 500.0, 3500.0, 1200.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("nasal_velum_coupling", "Nasal Port Velum Acoustic Opening Ratio", 0.0, 1.0, 0.15, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spline_interpolation_smoothness", "Articulatory Gesture Spline Transition Duration", 10.0, 500.0, 80.0, "ms", (34, 197, 94)))
        ),
        "SpectrogramArtConfig" => Box::new(
            GenericDspNodeUi::new("SpectrogramArtConfig", "Visual Spectrogram Art Sonification Engine Configuration HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_enum("num_frequency_bins", "Additive Spectral Sine Oscillator Count", vec!["64 Partials".into(), "128 Partials".into(), "256 Partials".into(), "512 Partials".into()], 2, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("min_freq_hz", "Bottom Row Base Frequency", 20.0, 500.0, 60.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("max_freq_hz", "Top Row Ceiling Frequency", 1000.0, 20000.0, 12000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("scan_speed_pps", "Horizontal Scanline Speed", 10.0, 500.0, 100.0, "px/s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("gamma_contrast", "Luminance-to-Amplitude Gamma", 0.2, 3.0, 1.2, "gamma", (34, 197, 94)))
        ),
        "SpectrogramImage" => Box::new(
            GenericDspNodeUi::new("SpectrogramImage", "2D Visual Sound Painting Raster Canvas & Image Buffer HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("brightness", "Visual Canvas Master Brightness", 0.0, 2.0, 1.0, "gain", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("contrast", "Visual Dynamic Contrast Ratio", 0.5, 3.0, 1.0, "contrast", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("invert_colors", "Negative Invert Pixel Luminance", 0.0, 1.0, 0.0, "bool", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("color_mapping", "Color Palette Interpretation Mode", vec!["Grayscale Luminance".into(), "RGB to Pitch-Triad".into(), "Thermal Spectral".into(), "Cyberpunk Neon".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("horizontal_resolution", "Raster Canvas Horizontal Buffer Width", vec!["256 Pixels".into(), "512 Pixels".into(), "1024 Pixels".into(), "2048 Pixels".into()], 1, (34, 197, 94)))
        ),
        "ImageNoteTrigger" => Box::new(
            GenericDspNodeUi::new("ImageNoteTrigger", "Optical Pixel Note Trigger & Coordinates-to-MIDI Translator HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("threshold", "Pixel Intensity Detection Threshold", 0.01, 0.95, 0.20, "thresh", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("max_simultaneous_notes", "Polyphonic Optical Note Limit", 1.0, 32.0, 8.0, "notes", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("scale_quantize", "Optical Frequency Scale Snapping", vec!["Continuous Microtonal".into(), "12-TET Chromatic".into(), "Major Pentatonic".into(), "Minor Blues".into(), "Dorian Mode".into()], 1, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("velocity_scale", "Luminance-to-Velocity Sensitivity", 0.1, 2.0, 1.0, "vel", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("hysteresis_ms", "Note Re-trigger Lockout Time", 5.0, 200.0, 30.0, "ms", (34, 197, 94)))
        ),
        "SpectralMorphConfig" => Box::new(
            GenericDspNodeUi::new("SpectralMorphConfig", "Dynamic Dual-Spectrum Morphing & Cross-Synthesis Engine HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("morph_factor", "Timbral Cross-Morphing Position", 0.0, 1.0, 0.50, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt", "High-Frequency Spectral Tilt", -12.0, 12.0, 0.0, "dB/oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("phase_coupling", "FFT Bin Phase Reconciliation Mode", vec!["Linear Phase Blend".into(), "Minimum Phase Warping".into(), "Random Phase Scrambler".into(), "Locked Source A".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_preserve", "Acoustic Vocal Formant Preservation", 0.0, 1.0, 0.70, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("smear_bandwidth", "Spectral Energy Diffusion Smear", 0.0, 1000.0, 100.0, "Hz", (34, 197, 94)))
        ),
        "FormantBusSnapshot" => Box::new(
            GenericDspNodeUi::new("FormantBusSnapshot", "Anthropomorphic Vocal Tract & F1-F4 Formant Envelope HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("vowel_morph_xy", "Continuous Vowel Space Morph Coordinate", 0.0, 1.0, 0.3, "morph", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_shift_cents", "Acoustic Vocal Tract Length Pitch Shift", -1200.0, 1200.0, 0.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("nasal_coupling_amount", "Soft Palate Velum Nasal Coupling", 0.0, 1.0, 0.15, "norm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("throat_larynx_depth", "Pharyngeal Cavity Throat Larynx Depth", 0.5, 2.0, 1.0, "geom", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vocal_formant_q_factor", "Formant Filter Bandwidth Sharpness Q", 1.0, 20.0, 6.5, "Q", (34, 197, 94)))
        ),
        "HarmonicProfile" => Box::new(
            GenericDspNodeUi::new("HarmonicProfile", "Additive Harmonic Partial Distribution & Centroid Envelope HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("odd_even_balance", "Odd-To-Even Harmonic Partial Ratio", -1.0, 1.0, 0.0, "bal", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_slope", "High-Frequency Spectral Tilt Rolloff", -24.0, 6.0, -6.0, "dB/oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_stretch", "Wire Dispersion Inharmonicity Stretch", 0.0, 0.1, 0.002, "factor", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("harmonic_count", "Synthesized Fourier Partial Count", 1.0, 128.0, 32.0, "partials", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_centroid_shift", "Spectral Mass Centroid Brightness Shift", -1000.0, 2000.0, 0.0, "Hz", (34, 197, 94)))
        ),
        "VowelNode" => Box::new(
            GenericDspNodeUi::new("VowelNode", "IPA Vowel Formant Coordinate & Articulatory Space HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("formant_f1_hz", "First Formant F1 (Vowel Height)", 200.0, 1200.0, 500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_f2_hz", "Second Formant F2 (Vowel Backness)", 600.0, 3000.0, 1500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("tongue_position", "Tongue Backness (Back to Front)", 0.0, 1.0, 0.5, "norm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tongue_height", "Tongue Elevation (Open to Close)", 0.0, 1.0, 0.6, "norm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lip_opening", "Lip Aperture (Rounded to Spread)", 0.1, 2.5, 1.0, "area", (34, 197, 94)))
        ),
        "HarmonicPartial" => Box::new(
            GenericDspNodeUi::new("HarmonicPartial", "Sinusoidal Additive Partial Oscillator & Phase Tracker HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("frequency_ratio", "Harmonic Partial Multiplier Ratio", 0.25, 32.0, 1.0, "ratio", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("partial_amplitude", "Sinusoidal Partial Linear Amplitude", 0.0, 1.0, 0.75, "amp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("smoothing_target_amplitude", "Interpolation Target Amplitude", 0.0, 1.0, 0.75, "amp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_accumulator_rad", "Continuous Partial Phase", 0.0, 6.283, 0.0, "rad", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_offset_hz", "Microtonal Inharmonic Frequency Offset", -100.0, 100.0, 0.0, "Hz", (34, 197, 94)))
        ),
        "QuantumTomographyData" => Box::new(
            GenericDspNodeUi::new("QuantumTomographyData", "Quantum Audio State Density Matrix Tomography HUD", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("bloch_vector_x", "Bloch Sphere State Projection X", -1.0, 1.0, 0.5, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bloch_vector_y", "Bloch Sphere State Projection Y", -1.0, 1.0, 0.0, "y", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("bloch_vector_z", "Bloch Sphere State Projection Z (Population Inversion)", -1.0, 1.0, 0.866, "z", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("state_purity", "Quantum Density Matrix State Purity", 0.0, 1.0, 1.0, "pure", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("entanglement_entropy", "Von Neumann Entanglement Entropy", 0.0, 1.0, 0.0, "bits", (34, 197, 94)))
        ),
        "Complex32" => Box::new(
            GenericDspNodeUi::new("Complex32", "Cartesian Complex Wave Amplitude State Vector", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("re", "Real In-Phase Amplitude", -2.0, 2.0, 1.0, "re", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("im", "Imaginary Quadrature Amplitude", -2.0, 2.0, 0.0, "im", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("magnitude", "Complex Absolute Magnitude", 0.0, 2.0, 1.0, "mag", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_rad", "Complex Argument Phase Angle", -3.14159, 3.14159, 0.0, "rad", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("coherence_time_ms", "Quantum Phase Coherence Lifetime", 0.1, 500.0, 25.0, "ms", (34, 197, 94)))
        ),
        "SpectrogramFrequencyMapping" => Box::new(
            GenericDspNodeUi::new("SpectrogramFrequencyMapping", "Image Spectrogram Art Frequency Scale & Bin Mapping", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("frequency_bin_scaling_scale", "Visual-to-Audio Frequency Distribution Scale", 0.0, 4.0, 1.0, "scale", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("lowest_mapped_frequency_hz", "Image Bottom Pixel Lowest Audio Frequency", 20.0, 500.0, 65.41, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("highest_mapped_frequency_hz", "Image Top Pixel Highest Audio Frequency", 2000.0, 20000.0, 14000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("oscillator_bank_partial_density", "Total Active Sine Partial Oscillators", 64.0, 1024.0, 256.0, "pts", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("time_scan_duration_seconds", "Image Horizontal Scan Duration Length", 1.0, 60.0, 8.0, "s", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("quantize_frequencies_to_midi_notes", "Snap Bin Frequencies to Nearest Chromatic Semitone", false, (14, 165, 233)))
        ),
        "SpectrogramColorMapping" => Box::new(
            GenericDspNodeUi::new("SpectrogramColorMapping", "Spectrogram Pixel Color to Audio Amplitude & Phase Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("pixel_color_decoding_mode", "Pixel Color Translation Engine Mode", 0.0, 3.0, 2.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("dynamic_range_headroom_db", "Visual Black-to-White Dynamic Range Headroom", 24.0, 96.0, 60.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("color_saturation_to_stereo_spread", "Color Saturation Modulating Stereo Width", 0.0, 1.0, 0.65, "width", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gamma_contrast_curve", "Pixel Brightness Gamma Exponent Curve", 0.5, 3.0, 1.2, "gamma", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("noise_threshold_floor", "Black Pixel Noise Floor Gate Cutoff", 0.0, 0.2, 0.03, "floor", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("invert_color_polarity", "Invert Black & White Pixel Amplitudes", false, (14, 165, 233)))
        ),
        "ColorMappingMode" => Box::new(
            GenericDspNodeUi::new("ColorMappingMode", "Spectrogram Art Optical Color-to-Harmonic Audio Mapping", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("color_mode", "Pixel Chromaticity to Harmonic Mapping Engine", 0.0, 1.0, 0.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("red_channel_pitch_offset", "Red Color Channel Harmonic Interval Offset", -24.0, 24.0, 0.0, "st", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("green_channel_resonance", "Green Color Channel Filter Resonance Q", 0.0, 1.0, 0.70, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("blue_channel_stereo_pan", "Blue Color Channel Stereo Panning Bias", -1.0, 1.0, 0.0, "pan", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("luminance_threshold", "Optical Pixel Dark Noise Floor Threshold", 0.0, 1.0, 0.08, "thresh", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_linear("gamma_curve_exponent", "Non-linear Pixel Brightness Contrast Gamma", 0.2, 3.0, 1.0, "gamma", (52, 211, 153)))
        ),
        "FrequencyMapping" => Box::new(
            GenericDspNodeUi::new("FrequencyMapping", "Spectrogram Art Vertical Pixel Frequency Axis Scale", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("frequency_scale", "Vertical Y-Axis Frequency Mapping Calibration", 0.0, 1.0, 1.0, "scale", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("base_frequency_hz", "Bottom Pixel Row Fundamental Audio Frequency", 20.0, 1000.0, 55.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("ceiling_frequency_hz", "Top Pixel Row Maximum Nyquist Limit Frequency", 2000.0, 22050.0, 16000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("octaves_per_height", "Logarithmic Octave Span Across Canvas Height", 1.0, 10.0, 8.0, "oct", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("cent_quantization_step", "Chromatic Note Intonation Quantization Step", 0.0, 100.0, 0.0, "cents", (14, 165, 233)))
                .with_param(DspParamDescriptor::new_bool("anti_aliasing_oversampling", "2x Sub-Pixel Anti-Aliased Sine Interpolation", true, (52, 211, 153)))
        ),
        "SpectralPhaseVocoderPitchShifter" => Box::new(
            GenericDspNodeUi::new("SpectralPhaseVocoderPitchShifter", "STFT Phase-Vocoder Independent Pitch & Formant Transposer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("pitch_shift_semitones", "Chromatic Pitch Shift", -24.0, 24.0, 0.0, "semi", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_fine_cents", "Pitch Fine Detune", -50.0, 50.0, 0.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Formant Spectral Shift", -12.0, 12.0, 0.0, "semi", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_window_size", "FFT Analysis Window Size", 0.0, 3.0, 1.0, "win", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_locking_algorithm", "Phase Coherence Locking", 0.0, 2.0, 0.0, "lock", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("transient_detector_bypass", "Transient Bypass Preservation", true, (14, 165, 233)))
        ),
        "SpectralMorphHarmonizer" => Box::new(
            GenericDspNodeUi::new("SpectralMorphHarmonizer", "FFT Cross-Synthesis Formant Morpher & Chord Harmonizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("formant_carrier_blend", "Formant Filter vs Carrier Synth Blend", 0.0, 1.0, 0.5, "ratio", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("fft_analysis_resolution", "Spectral FFT Analysis Resolution", 0.0, 2.0, 1.0, "res", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("harmonic_voicing_intervals", "Polyphonic Chord Harmonizer Preset", 0.0, 3.0, 0.0, "preset", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_emphasis_db", "High-Frequency Spectral Tilt", -18.0, 18.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("partial_tracking_bandwidth_hz", "Sine Partial Tracking Bandwidth", 20.0, 500.0, 100.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("consonant_unvoiced_passthrough", "Pass-Through Unvoiced Sibilance Noise", true, (14, 165, 233)))
        ),
        "ZeroGravityFluidResonator" => Box::new(
            GenericDspNodeUi::new("ZeroGravityFluidResonator", "Hydrodynamic Microgravity Bubble & Droplet Acoustic Synthesizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("fluid_surface_tension_n_m", "Fluid Surface Tension Coefficient", 0.01, 0.1, 0.0728, "N/m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bubble_radius_distribution_mm", "Minnaert Cavity Bubble Radius", 0.1, 10.0, 1.8, "mm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cavitation_implosion_intensity", "Cavitation Bubble Collapse Intensity", 0.0, 1.0, 0.45, "depth", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("capillary_wave_modal_damping", "Capillary Surface Wave Damping", 0.01, 0.8, 0.15, "damp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fluid_viscosity_cst", "Kinematic Fluid Viscosity", 0.5, 50.0, 1.0, "cSt", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("ambient_pressure_kpa", "Surrounding Ambient Atmospheric Pressure", 10.0, 200.0, 101.3, "kPa", (14, 165, 233)))
        ),
        "HarmonicExcitanceSynthesizer" => Box::new(
            GenericDspNodeUi::new("HarmonicExcitanceSynthesizer", "Chebyshev Polynomial Psychoacoustic Harmonic Exciter", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("chebyshev_order_profile", "Chebyshev Harmonic Order Series", 0.0, 3.0, 2.0, "order", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("critical_band_masking_db", "Psychoacoustic Masking Threshold", -48.0, 0.0, -18.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("odd_even_harmonic_balance", "Odd/Even Harmonic Balance", -1.0, 1.0, 0.1, "bal", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_flux_sensitivity", "Transient Dynamic Flux Tracking", 0.0, 1.0, 0.6, "flux", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_octave_reinforcement", "Sub-Harmonic Fundamental Weight", 0.0, 1.0, 0.25, "weight", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("air_band_sheen_khz", "Ultra-High Air Band Sheen Frequency", 8.0, 24.0, 14.5, "kHz", (14, 165, 233)))
        ),
        "SpectralFormantMorpher" => Box::new(
            GenericDspNodeUi::new("SpectralFormantMorpher", "Dual-Vowel Formant Filter Bank & Vocal Tract Crossfader", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("vowel_source_a", "Source Vowel Formant Profile A", 0.0, 4.0, 0.0, "vowel", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vowel_target_b", "Target Vowel Formant Profile B", 0.0, 4.0, 1.0, "vowel", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("vowel_morph_crossfade", "Vowel Formant Interpolation Morph", 0.0, 1.0, 0.0, "morph", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Vocal Tract Scale Gender Shift", -12.0, 12.0, 0.0, "st", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("glottal_excitation_mix", "Internal Glottal Pulse Injection", 0.0, 1.0, 0.3, "mix", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("throat_cavity_scale_ratio", "Pharyngeal Throat Cavity Resonance", 0.5, 2.0, 1.0, "scale", (14, 165, 233)))
        ),
        "SpectralInharmonicResonator" => Box::new(
            GenericDspNodeUi::new("SpectralInharmonicResonator", "Stiff Inharmonic Bar & Bell Partial Stretch Resonator", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("stiffness_dispersion_coeff", "Stiffness Dispersion Coefficient", 0.0, 1.0, 0.25, "stiff", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("modal_partial_count", "Resonant Modal Partial Count", 0.0, 3.0, 2.0, "idx", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_stretch_power", "Inharmonic Stretch Exponent", 1.0, 3.5, 1.75, "pow", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("decay_frequency_damping", "High-Frequency Modal Loss Damping", 0.01, 1.0, 0.35, "damp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("strike_excitation_position", "Bar Strike Excitation Position", 0.01, 0.99, 0.18, "pos", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("even_odd_mode_coupling", "Even/Odd Mode Energy Coupling", 0.0, 1.0, 0.5, "mix", (14, 165, 233)))
        ),
        "FormantVowelMorphMatrix" => Box::new(
            GenericDspNodeUi::new("FormantVowelMorphMatrix", "2D Vocal Tract Formant Interpolator & Nasal Resonance Matrix", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("vowel_coord_x", "Vowel Space Coordinate X", 0.0, 1.0, 0.35, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vowel_coord_y", "Vowel Space Coordinate Y", 0.0, 1.0, 0.65, "y", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_q_factor", "Vocal Tract Formant Bandwidth Q", 2.0, 30.0, 12.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("larynx_tract_length_cm", "Acoustic Vocal Tract Length", 12.0, 22.0, 17.2, "cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_coupling", "Velopharyngeal Nasal Coupling", 0.0, 1.0, 0.15, "coup", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("morph_trajectory_profile", "Automated Vowel Path Trajectory", 0.0, 3.0, 0.0, "traj", (14, 165, 233)))
        ),
        "SpectralResonanceSuppressor" => Box::new(
            GenericDspNodeUi::new("SpectralResonanceSuppressor", "Dynamic Peak Tracking Notch Filter & Harshness Tamer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("suppression_depth_db", "Dynamic Notch Suppression Limit", 0.0, 24.0, 6.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_detection_sensitivity", "Spectral Peak Detection Sensitivity", 0.0, 1.0, 0.68, "sens", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_q_sharpness", "Anti-Resonance Notch Filter Q", 1.0, 50.0, 18.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_protection_level", "Transient Attack Protection Guard", 0.0, 1.0, 0.75, "prot", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("target_frequency_low_bound_hz", "Processing Range Low Frequency Boundary", 100.0, 2000.0, 800.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("target_frequency_high_bound_hz", "Processing Range High Frequency Boundary", 2000.0, 18000.0, 9500.0, "Hz", (14, 165, 233)))
        ),
        "SpectralVocoderModulatorBank" => Box::new(
            GenericDspNodeUi::new("SpectralVocoderModulatorBank", "32-Band Constant-Q Channel Vocoder & Carrier Matrix", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("vocoder_frequency_band_count", "Filter Bank Channel Resolution", 0.0, 3.0, 3.0, "bands", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_spectral_shift_ratio", "Formant Frequency Transposition Factor", 0.5, 2.0, 1.0, "x", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("envelope_follower_decay_speed_ms", "Envelope Follower Integration Time Constant", 2.0, 100.0, 15.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("carrier_synthesizer_waveform", "Internal Carrier Synthesizer Sound Source", 0.0, 3.0, 0.0, "src", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sibilance_high_frequency_bypass_db", "Unvoiced Consonant Sibilance Pass-Through", -24.0, 6.0, -3.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("stereo_band_interleaving_spread", "Stereo Channel Alternating Band Panning", 0.0, 1.0, 0.45, "%", (14, 165, 233)))
        ),
        "AiIntelligentSpectralUnmasker" => Box::new(
            GenericDspNodeUi::new("AiIntelligentSpectralUnmasker", "AI Deep Psychoacoustic Spectral Conflict Unmasker", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("unmasking_intensity_db", "Spectral Carving Notching Depth", 0.0, 24.0, 6.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("auditory_masking_curve", "Bark Scale Auditory Spread Function", 0.0, 3.0, 1.0, "curve", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_transparency", "Transient Pass-Through Preservation", 0.0, 1.0, 0.85, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("smoothing_attack_ms", "Spectral Envelope Attack Time", 0.5, 50.0, 5.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("smoothing_release_ms", "Spectral Envelope Release Time", 5.0, 500.0, 50.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("sidechain_listen_delta", "Audition Carved Masking Delta", false, (14, 165, 233)))
        ),
        "InverseFftSpectrogramSonifier" => Box::new(
            GenericDspNodeUi::new("InverseFftSpectrogramSonifier", "Visual-to-Audio Inverse FFT Spectrogram Sonifier", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("image_scan_speed_px_sec", "Horizontal Pixel Column Scan Rate", 10.0, 500.0, 100.0, "px/s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("frequency_span_low_hz", "Lowest Raster Row Acoustic Frequency", 20.0, 500.0, 80.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("frequency_span_high_hz", "Highest Raster Row Acoustic Frequency", 2000.0, 22000.0, 16000.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("brightness_gamma_curve", "Pixel Luminosity Dynamic Response", 0.5, 3.0, 1.4, "gamma", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("color_stereo_spread", "RGB Color Hue to Stereo Pan Mapping", 0.0, 1.0, 0.7, "spread", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("harmonic_quantization_scale", "Frequency Axis Microtonal Quantization", 0.0, 3.0, 0.0, "scale", (14, 165, 233)))
        ),
        "NeuralTimbreMorphSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralTimbreMorphSynthesizer", "Latent-Space Timbral Vector Interpolator & Neural Resynthesizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("latent_coord_x", "Latent Vector Dimension X (Brightness)", -1.0, 1.0, 0.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("latent_coord_y", "Latent Vector Dimension Y (Harmonicity)", -1.0, 1.0, 0.0, "y", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("timbre_morph_glide_ms", "Latent Coordinate Interpolation Glide", 5.0, 2000.0, 150.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("decoder_temperature", "Neural Decoder Softmax Temperature", 0.1, 2.0, 0.8, "temp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_envelope_preserve", "Acoustic Formant Envelope Retention", 0.0, 1.0, 0.75, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("quantize_to_acoustic_anchor", "Snap Coordinates to Acoustic Anchors", false, (14, 165, 233)))
        ),
        "CepstralEnvelopeMorphSynthesizer" => Box::new(
            GenericDspNodeUi::new("CepstralEnvelopeMorphSynthesizer", "Real-Time True Envelope Cepstral Morphing Synthesizer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("cepstral_order_coefficients", "Envelope Cepstral Truncation Order", 4.0, 128.0, 32.0, "coeffs", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_bias", "Spectral Spectral Tilt Spectral Bias", -12.0, 12.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("envelope_interpolation_ratio", "Cepstral Morphing Cross-Synthesis Ratio", 0.0, 1.0, 0.5, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Formant Center Frequency Pitch Shift", -24.0, 24.0, 0.0, "st", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("carrier_residual_gain", "Synthesized Carrier Harmonic Level", -36.0, 6.0, -6.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_harmonic_peak_locking", "Lock Harmonic Spectral Saliencies", true, (14, 165, 233)))
        ),
        "HarmonicAdditiveResynthesisEngine" => Box::new(
            GenericDspNodeUi::new("HarmonicAdditiveResynthesisEngine", "128-Partial Additive Resynthesis Engine & Spectral Modeler", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("partial_count_active", "Active Partial Voice Count", 1.0, 128.0, 32.0, "partials", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_centroid_tilt_db", "Spectral Centroid Tilt Gain", -24.0, 24.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("inharmonicity_dispersion_index", "Inharmonic Partial Frequency Dispersion", 0.0, 1.0, 0.15, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_alignment_coherence", "Partial Phase Coherence Coupling", 0.0, 1.0, 0.8, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamic_odd_even_harmonic_ratio", "Odd vs Even Harmonic Energy Balance", -1.0, 1.0, 0.0, "bal", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_spectral_anti_aliasing", "Nyquist Spectral Anti-Aliasing Guard", true, (14, 165, 233)))
        ),
        "SpectralContrastCompressor" => Box::new(
            GenericDspNodeUi::new("SpectralContrastCompressor", "FFT Spectral Contrast Dynamic Masker & Formant Peak Sharpener", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("spectral_contrast_ratio", "Spectral Peak-To-Valley Contrast Ratio", 0.1, 5.0, 1.8, ":1", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("fft_bin_smoothing_width", "FFT Frequency Bin Smoothing Bandwidth", 1.0, 64.0, 8.0, "bins", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dynamic_spectral_threshold_db", "Dynamic Contrast Processing Threshold", -60.0, 0.0, -24.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("peak_valley_balance", "Formant Peak vs Valley Weighting", -1.0, 1.0, 0.25, "bal", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_floor_noise_suppression", "Spectral Noise Floor Gating Threshold", 0.0, 1.0, 0.4, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("linear_frequency_bin_spacing", "Linear vs Mel-Scale Frequency Binning", false, (14, 165, 233)))
        ),
        "WaveletPacketResynthesisEngine" => Box::new(
            GenericDspNodeUi::new("WaveletPacketResynthesisEngine", "Wavelet Packet Multiresolution Decomposition & Resynthesis Engine", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("decomposition_depth_levels", "Wavelet Decomposition Depth Levels", 1.0, 8.0, 4.0, "lvl", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("threshold_denoise_ratio_db", "Subband Threshold De-Noise Gate", -60.0, 0.0, -28.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("subband_energy_tilt_db", "Logarithmic Subband Energy Spectral Tilt", -18.0, 18.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("wavelet_family_basis", "Mother Wavelet Mathematical Basis Mode", 0.0, 3.0, 1.0, "mode", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_reconstruction_mode", "Wavelet Phase Reconstruction Mode", 0.0, 2.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_perfect_reconstruction_guard", "Perfect Reconstruction Math Guard", true, (14, 165, 233)))
        ),
        "SpectralFreezeGranularSmear" => Box::new(
            GenericDspNodeUi::new("SpectralFreezeGranularSmear", "Continuous FFT Phase Randomizer & Spectral Smear Cloud", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("smear_window_size_ms", "Spectral Analysis Window Size", 10.0, 250.0, 65.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_diffusion_depth", "Phase Diffusion Randomization Depth", 0.0, 1.0, 0.75, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_brightness_tilt", "Frozen Spectrum Brightness Tilt", -12.0, 12.0, 2.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("bin_retention_threshold_db", "Spectral Bin Magnitude Gating Threshold", -70.0, 0.0, -35.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_scramble_mode", "Phase Scrambling Topology Profile Mode", 0.0, 2.0, 1.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("freeze_spectral_evolution", "Infinite Spectral Drone Freeze", false, (14, 165, 233)))
        ),
        "QuantumMolecularBondResonator" => Box::new(
            GenericDspNodeUi::new("QuantumMolecularBondResonator", "Quantum Molecular Bond Vibrational Frequency Resonator Bank", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("molecular_bond_stiffness_nm", "Covalent Bond Spring Constant Stiffness", 10.0, 500.0, 120.0, "N/m", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("atomic_mass_amu", "Constituent Atomic Mass Unit", 1.0, 250.0, 16.0, "amu", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("anharmonic_potential_depth", "Morse Anharmonic Potential Depth", 0.0, 1.0, 0.25, "Morse", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vibrational_damping_ps", "Intramolecular Relaxation Lifetime", 0.1, 50.0, 5.0, "ps", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("thermal_temperature_kelvin", "Ambient Absolute Temperature", 0.0, 1000.0, 293.15, "K", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_quantum_zero_point_energy", "Quantum Zero-Point Energy Fluctuation", true, (14, 165, 233)))
        ),
        "VocalPitchFormantRetuner" => Box::new(
            GenericDspNodeUi::new("VocalPitchFormantRetuner", "Neural Vocal Pitch & Formant Retuner with Gender Transposition", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("pitch_correction_speed_ms", "Pitch Retune Correction Speed", 0.0, 100.0, 20.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pitch_quantize_scale_snap", "Musical Scale Snapping Pull Factor", 0.0, 1.0, 1.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_throat_length_shift", "Vocal Tract Throat Length Shift", -12.0, 12.0, 0.0, "semitones", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vocal_gender_morph_ratio", "Male to Female Timbral Gender Morph", -1.0, 1.0, 0.0, "morph", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("natural_vibrato_preservation", "Natural Vibrato Depth Preservation", 0.0, 1.0, 0.8, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_neural_glottal_pulse_modeling", "Neural Glottal Pulse Resynthesis", true, (14, 165, 233)))
        ),
        "MultibandSpectralContrastExpander" => Box::new(
            GenericDspNodeUi::new("MultibandSpectralContrastExpander", "Bark-Scale Psychoacoustic Spectral Contrast Enhancer", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("contrast_expansion_ratio", "Spectral Peak-to-Valley Expand Ratio", 1.0, 4.0, 1.6, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bark_filter_band_count", "Critical Bark Frequency Band Subdivisions", 6.0, 32.0, 24.0, "bands", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("valley_spectral_attenuation_db", "Spectral Valley Damping Floor", 0.0, 18.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_contrast_boost_db", "Harmonic Formant Peak Accentuation", 0.0, 12.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothing_window_ms", "Inter-Band Smoothing Window Time", 2.0, 100.0, 15.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_formant_envelope_preservation", "Preserve Global Formant Spectral Tilting", true, (14, 165, 233)))
        ),
        "WaveletTransientSplayDeconstruct" => Box::new(
            GenericDspNodeUi::new("WaveletTransientSplayDeconstruct", "Continuous Wavelet Transform Multi-Scale Transient Splay & Deconstruct", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("wavelet_scale_resolution_octaves", "Dyadic Wavelet Scale Decomposition", 1.0, 12.0, 6.0, "oct", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("morlet_wavelet_time_frequency_tradeoff", "Morlet Mother Wavelet Center Q", 0.5, 8.0, 2.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_energy_threshold_db", "Micro-Transient Separation Threshold", -60.0, 0.0, -24.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("body_resonance_isolation_ratio", "Tonal Body Sustain Multiplier", 0.0, 2.0, 1.0, "x", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_time_splay_spread_ms", "Transient Decorrelation Time Splay", 0.0, 50.0, 8.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_perfect_reconstruction_frame", "Biorthogonal Dual Frame Reconstruction", true, (14, 165, 233)))
        ),
        "BilateralSpectralDenoise" => Box::new(
            GenericDspNodeUi::new("BilateralSpectralDenoise", "Bilateral Spectral Domain Non-Local Means Audio Denoiser", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("spectral_noise_reduction_depth_db", "Spectral Denoising Attenuation Depth", 0.0, 36.0, 14.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("noise_profile_learning_adaptation", "Continuous Noise Profile Adaptation Rate", 0.0, 1.0, 0.5, "learn", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("bilateral_spectral_kernel_width", "Bilateral Spectral Neighborhood Radius", 1.0, 16.0, 4.0, "bins", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("musical_noise_artifact_suppression", "Musical Noise Chirp Artifact Suppression", 0.0, 1.0, 0.8, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation_sensitivity", "Consonant & Transient Protection Gate", 0.1, 2.0, 1.2, "sens", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_residual_noise_listen_mode", "Residual Difference Noise Audition", false, (14, 165, 233)))
        ),
        "DynamicSpectralTransientSplitter" => Box::new(
            GenericDspNodeUi::new("DynamicSpectralTransientSplitter", "STFT Dynamic Spectral Transient & Sustain Splitter", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("transient_detection_sensitivity_db", "Transient Energy Derivative Sensitivity", -40.0, 0.0, -18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_sustain_balance_db", "Transient vs Sustain Output Balance", -24.0, 24.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_fft_window_size", "STFT Transform Window Resolution", 256.0, 4096.0, 1024.0, "bins", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_decay_hold_time_ms", "Transient Envelope Isolation Window", 1.0, 100.0, 15.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sustain_spectral_smoothing_pct", "Tonal Body Temporal Smoothing", 0.0, 100.0, 65.0, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_transient_solo_listen", "Audition Isolated Transient Path Solo", false, (14, 165, 233)))
        ),
        "MorphologicalSpectrogramFilter" => Box::new(
            GenericDspNodeUi::new("MorphologicalSpectrogramFilter", "Mathematical Morphology Spectrogram Filter", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("structuring_element_time_bins", "Structuring Element Time Width", 1.0, 32.0, 5.0, "bins", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("structuring_element_freq_bins", "Structuring Element Frequency Height", 1.0, 32.0, 3.0, "bins", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("morphological_operation_mode", "Morphology Mode (Open/Close/Erode/Dilate)", 0.0, 3.0, 0.0, "mode", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_mask_contrast_gain_db", "Spectral Contrast Elevation Gain", 0.0, 36.0, 12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("temporal_frame_overlap_ratio", "STFT Analysis Frame Overlap", 2.0, 8.0, 4.0, "x", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_residual_signal_monitoring", "Solo Morphological Residual Difference", false, (14, 165, 233)))
        ),
        "AdaptiveSpectralSubtractDeNoiser" => Box::new(
            GenericDspNodeUi::new("AdaptiveSpectralSubtractDeNoiser", "Adaptive Minimum-Statistics Spectral De-Noiser", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("noise_reduction_depth_db", "Spectral Subtraction Attenuation Depth", 0.0, 48.0, 18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("noise_profile_learning_rate", "Minimum Statistics Tracking Speed", 0.1, 10.0, 1.5, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation_threshold_db", "Transient Signal Guard Threshold", 3.0, 24.0, 12.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("musical_noise_oversubtraction_factor", "Oversubtraction Factor (Musical Tone Guard)", 1.0, 4.0, 1.6, "mult", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothing_bandwidth_hz", "Spectral Mask Inter-Bin Smoothing", 20.0, 500.0, 120.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_residual_noise_audition", "Listen Residual Noise Delta", false, (14, 165, 233)))
        ),
        "DynamicSpectralTransientExciter" => Box::new(
            GenericDspNodeUi::new("DynamicSpectralTransientExciter", "Spectral Transient Dynamic Accentuator", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("transient_boost_gain_db", "Transient Burst Amplification Gain", -18.0, 24.0, 6.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tonal_bed_sustain_gain_db", "Tonal Splay Stationary Component Gain", -18.0, 18.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_flux_threshold_db", "Spectral Flux Transient Detector Sensitivity", 1.0, 20.0, 6.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_transient_tilt_db", "High-Frequency Transient Brightness Tilt", -12.0, 12.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_release_smoothing_ms", "Transient Envelope Decay Recovery", 1.0, 100.0, 15.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_transient_only_monitoring", "Solo Extracted Transient Signal", false, (14, 165, 233)))
        ),
        "QuantumEntanglementPhaseVocoder" => Box::new(
            GenericDspNodeUi::new("QuantumEntanglementPhaseVocoder", "Quantum Entangled Multi-Channel Phase Vocoder", DspNodeCategory::SpectralResynthesis)
                .with_param(DspParamDescriptor::new_linear("entanglement_correlation_fidelity", "Quantum State Entanglement Fidelity", 0.0, 1.0, 0.95, "fid", (217, 70, 239)))
                .with_param(DspParamDescriptor::new_linear("stft_fft_analysis_window_size", "Analysis Frame Window Size", 256.0, 8192.0, 2048.0, "samples", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("time_stretch_expansion_ratio", "Time-Scale Invariance Expansion Ratio", 0.1, 8.0, 1.5, "ratio", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("spectral_bin_phase_locking_tightness", "Inter-Bin Phase Locking Rigidity", 0.0, 1.0, 0.8, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation_threshold_db", "Transient Preservation Detection Threshold", -60.0, 0.0, -24.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_phase_reconstruction_lattice", "Zero-Phase Reconstruction Lattice", true, (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
