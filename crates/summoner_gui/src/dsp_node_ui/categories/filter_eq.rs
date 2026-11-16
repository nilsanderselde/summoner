// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `FilterEq`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("FilterSVF".to_string(), DspNodeDescriptor::new("FilterSVF", "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)", DspNodeCategory::FilterEq, "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)")
        .with_param(DspParamSchema::choice("mode", "Filter Topology", &["Lowpass 12dB", "Highpass 12dB", "Bandpass", "Notch"], 0, "Filter Topology mode selector"))
        .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1500.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 10.0, 1.2, "", MacroRole::Tone, "Resonance Q parameter"))
        .with_param(DspParamSchema::knob("drive", "Pre-filter Drive", 0.0, 18.0, 0.0, "dB", MacroRole::Tone, "Pre-filter Drive parameter"))
    );
    descriptors.insert("FilterLadder".to_string(), DspNodeDescriptor::new("FilterLadder", "24dB/Oct Transistor Moog-Style Ladder Filter", DspNodeCategory::FilterEq, "24dB/Oct Transistor Moog-Style Ladder Filter")
        .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 10.0, 1.5, "", MacroRole::Tone, "Resonance Q parameter"))
        .with_param(DspParamSchema::knob("drive", "Drive Saturation", 0.0, 24.0, 3.0, "dB", MacroRole::Tone, "Drive Saturation parameter"))
        .with_param(DspParamSchema::knob("key_track", "Key Tracking", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Key Tracking parameter"))
    );
    descriptors.insert("FilterLadder8".to_string(), DspNodeDescriptor::new("FilterLadder8", "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter", DspNodeCategory::FilterEq, "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter")
        .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 12.0, 2.0, "", MacroRole::Tone, "Resonance Q parameter"))
        .with_param(DspParamSchema::knob("pole_count", "Cascaded Poles", 2.0, 8.0, 8.0, "poles", MacroRole::Character, "Cascaded Poles integer control"))
        .with_param(DspParamSchema::knob("saturation", "Non-linear Warmth", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Non-linear Warmth parameter"))
    );
    descriptors.insert("FilterComb".to_string(), DspNodeDescriptor::new("FilterComb", "Feedback Comb Resonator Filter", DspNodeCategory::FilterEq, "Feedback Comb Resonator Filter")
        .with_param(DspParamSchema::knob("delay_ms", "Delay Time", 0.1, 50.0, 5.0, "ms", MacroRole::Tone, "Delay Time parameter"))
        .with_param(DspParamSchema::knob("feedback", "Feedback Loop", -0.99, 0.99, 0.7, "%", MacroRole::Tone, "Feedback Loop parameter"))
        .with_param(DspParamSchema::knob("damping", "High Damping", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "High Damping parameter"))
        .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
    );
    descriptors.insert("FilterBiquad".to_string(), DspNodeDescriptor::new("FilterBiquad", "Direct Form II Transposed Biquad Multi-Mode Filter", DspNodeCategory::FilterEq, "Direct Form II Transposed Biquad Multi-Mode Filter")
        .with_param(DspParamSchema::choice("filter_type", "Biquad Mode", &["Lowpass", "Highpass", "Bandpass", "Notch", "Peaking", "LowShelf", "HighShelf"], 0, "Biquad Mode mode selector"))
        .with_param(DspParamSchema::log_knob("cutoff", "Center / Cutoff Freq", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Center / Cutoff Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("q_factor", "Filter Q Factor", 0.1, 20.0, 1.0, "Q", MacroRole::Tone, "Filter Q Factor parameter"))
        .with_param(DspParamSchema::knob("gain_db", "Band Boost / Cut", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Band Boost / Cut parameter"))
    );
    descriptors.insert("DcBlockFilter".to_string(), DspNodeDescriptor::new("DcBlockFilter", "Zero-Phase DC Offset Removal Highpass Filter", DspNodeCategory::FilterEq, "Zero-Phase DC Offset Removal Highpass Filter")
        .with_param(DspParamSchema::knob("pole_radius", "Pole Radius (R)", 0.9, 0.9999, 0.995, "", MacroRole::Tone, "Pole Radius (R) parameter"))
        .with_param(DspParamSchema::knob("highpass_hz", "Corner Highpass Freq", 1.0, 50.0, 10.0, "Hz", MacroRole::Tone, "Corner Highpass Freq parameter"))
        .with_param(DspParamSchema::toggle("enabled", "DC Block Active", true, "DC Block Active toggle switch"))
    );
    descriptors.insert("LowCutFilter".to_string(), DspNodeDescriptor::new("LowCutFilter", "Precision Sub-Sonic Low-Cut Highpass Filter", DspNodeCategory::FilterEq, "Precision Sub-Sonic Low-Cut Highpass Filter")
        .with_param(DspParamSchema::log_knob("cutoff", "Low Cut Frequency", 10.0, 1000.0, 30.0, "Hz", MacroRole::Tone, "Low Cut Frequency logarithmic parameter"))
        .with_param(DspParamSchema::choice("slope_order", "Filter Slope", &["6 dB/oct", "12 dB/oct", "18 dB/oct", "24 dB/oct", "48 dB/oct"], 1, "Filter Slope mode selector"))
        .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", MacroRole::Tone, "Resonance Q parameter"))
    );
    descriptors.insert("HighCutFilter".to_string(), DspNodeDescriptor::new("HighCutFilter", "Anti-Aliasing High-Cut Lowpass Filter", DspNodeCategory::FilterEq, "Anti-Aliasing High-Cut Lowpass Filter")
        .with_param(DspParamSchema::log_knob("cutoff", "High Cut Frequency", 1000.0, 20000.0, 16000.0, "Hz", MacroRole::Tone, "High Cut Frequency logarithmic parameter"))
        .with_param(DspParamSchema::choice("slope_order", "Filter Slope", &["6 dB/oct", "12 dB/oct", "18 dB/oct", "24 dB/oct", "48 dB/oct"], 1, "Filter Slope mode selector"))
        .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", MacroRole::Tone, "Resonance Q parameter"))
    );
    descriptors.insert("ParametricEqNode".to_string(), DspNodeDescriptor::new("ParametricEqNode", "4-Band Precision Parametric Equalizer", DspNodeCategory::FilterEq, "4-Band Precision Parametric Equalizer")
        .with_param(DspParamSchema::knob("low_shelf", "Low Shelf (100Hz)", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Low Shelf (100Hz) parameter"))
        .with_param(DspParamSchema::knob("low_mid", "Low Mid Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Low Mid Gain parameter"))
        .with_param(DspParamSchema::knob("high_mid", "High Mid Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High Mid Gain parameter"))
        .with_param(DspParamSchema::knob("high_shelf", "High Shelf (10kHz)", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High Shelf (10kHz) parameter"))
        .with_param(DspParamSchema::log_knob("mid_freq", "Parametric Mid Freq", 200.0, 8000.0, 1000.0, "Hz", MacroRole::Tone, "Parametric Mid Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("mid_q", "Parametric Mid Q", 0.2, 10.0, 1.0, "Q", MacroRole::Tone, "Parametric Mid Q parameter"))
    );
    descriptors.insert("FormantFilterNode".to_string(), DspNodeDescriptor::new("FormantFilterNode", "Vowel Formant Filter Matrix", DspNodeCategory::FilterEq, "Vowel Formant Filter Matrix")
        .with_param(DspParamSchema::choice("vowel", "Target Vowel", &["Vowel A (Ah)", "Vowel E (Eh)", "Vowel I (Ee)", "Vowel O (Oh)", "Vowel U (Oo)"], 0, "Target Vowel mode selector"))
        .with_param(DspParamSchema::knob("formant_shift", "Formant Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Formant Shift parameter"))
        .with_param(DspParamSchema::knob("throat_len", "Acoustic Throat Length", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Acoustic Throat Length parameter"))
        .with_param(DspParamSchema::knob("resonance_q", "Formant Resonance Q", 1.0, 20.0, 5.0, "Q", MacroRole::Tone, "Formant Resonance Q parameter"))
    );
    descriptors.insert("ModalFilterNode".to_string(), DspNodeDescriptor::new("ModalFilterNode", "High-Order Acoustic Modal Resonator Bank", DspNodeCategory::FilterEq, "High-Order Acoustic Modal Resonator Bank")
        .with_param(DspParamSchema::log_knob("base_freq", "Fundamental Modal Freq", 50.0, 8000.0, 440.0, "Hz", MacroRole::Tone, "Fundamental Modal Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("mode_count", "Active Modal Poles", 1.0, 16.0, 8.0, "modes", MacroRole::Character, "Active Modal Poles integer control"))
        .with_param(DspParamSchema::knob("decay_rate", "Modal Decay Time", 0.05, 5.0, 1.2, "s", MacroRole::Tone, "Modal Decay Time parameter"))
        .with_param(DspParamSchema::knob("inharmonicity", "Inharmonic Mode Shift", 0.0, 2.0, 0.15, "", MacroRole::Tone, "Inharmonic Mode Shift parameter"))
    );
    descriptors.insert("HornReflectionFilter".to_string(), DspNodeDescriptor::new("HornReflectionFilter", "Acoustic Bell Flare & Throat Reflection Filter", DspNodeCategory::FilterEq, "Acoustic Bell Flare & Throat Reflection Filter")
        .with_param(DspParamSchema::knob("bell_flare", "Bell Flare Expansion", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Bell Flare Expansion parameter"))
        .with_param(DspParamSchema::knob("throat_diameter", "Throat Diameter", 5.0, 50.0, 15.0, "mm", MacroRole::Tone, "Throat Diameter parameter"))
        .with_param(DspParamSchema::knob("reflection_coeff", "End Reflection Coefficient", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "End Reflection Coefficient parameter"))
    );
    descriptors.insert("AutomatedDynamicEqNode".to_string(), DspNodeDescriptor::new("AutomatedDynamicEqNode", "AI-Driven Intelligent Masking Reduction Dynamic EQ", DspNodeCategory::FilterEq, "AI-Driven Intelligent Masking Reduction Dynamic EQ")
        .with_param(DspParamSchema::knob("target_lufs", "Target Program LUFS", -24.0, -8.0, -14.0, "LUFS", MacroRole::Tone, "Target Program LUFS parameter"))
        .with_param(DspParamSchema::knob("low_dynamic_cut", "Dynamic Mud Cut (250Hz)", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Dynamic Mud Cut (250Hz) parameter"))
        .with_param(DspParamSchema::knob("high_air_lift", "Dynamic Air Presence (12kHz)", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Dynamic Air Presence (12kHz) parameter"))
        .with_param(DspParamSchema::knob("adaptation_speed", "Neural Adaptation Speed", 10.0, 500.0, 100.0, "ms", MacroRole::Tone, "Neural Adaptation Speed parameter"))
    );
    descriptors.insert("SubharmonicQuantumTunnelingFilter".to_string(), DspNodeDescriptor::new("SubharmonicQuantumTunnelingFilter", "Subharmonic Quantum Tunneling Resonant Filter", DspNodeCategory::FilterEq, "Subharmonic Quantum Tunneling Resonant Filter")
        .with_param(DspParamSchema::knob("barrier_width_nm", "Quantum Barrier Width", 0.5, 10.0, 2.5, "nm", MacroRole::Tone, "Quantum Barrier Width parameter"))
        .with_param(DspParamSchema::knob("tunneling_prob", "Tunneling Probability", 0.01, 1.0, 0.35, "%", MacroRole::Tone, "Tunneling Probability parameter"))
        .with_param(DspParamSchema::knob("resonance_q", "Subharmonic Q Factor", 1.0, 30.0, 8.0, "Q", MacroRole::Tone, "Subharmonic Q Factor parameter"))
        .with_param(DspParamSchema::log_knob("cutoff_hz", "Filter Cutoff Corner", 20.0, 12000.0, 800.0, "Hz", MacroRole::Tone, "Filter Cutoff Corner logarithmic parameter"))
    );
    descriptors.insert("MetamaterialRefractionFilter".to_string(), DspNodeDescriptor::new("MetamaterialRefractionFilter", "Negative Index Acoustic Metamaterial Filter", DspNodeCategory::FilterEq, "Negative Index Acoustic Metamaterial Filter")
        .with_param(DspParamSchema::knob("refractive_index", "Negative Index (n)", -5.0, -0.1, -1.4, "n", MacroRole::Tone, "Negative Index (n) parameter"))
        .with_param(DspParamSchema::knob("slab_thickness_mm", "Metamaterial Thickness", 1.0, 50.0, 12.0, "mm", MacroRole::Tone, "Metamaterial Thickness parameter"))
        .with_param(DspParamSchema::knob("evanescent_gain", "Evanescent Amplification", 0.0, 12.0, 3.5, "dB", MacroRole::Tone, "Evanescent Amplification parameter"))
        .with_param(DspParamSchema::knob("resonance_q", "Acoustic Bandgap Q", 1.0, 25.0, 6.0, "Q", MacroRole::Tone, "Acoustic Bandgap Q parameter"))
    );
    descriptors.insert("SpatialAcousticHologramFilter".to_string(), DspNodeDescriptor::new("SpatialAcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Filter", DspNodeCategory::FilterEq, "Spatial Acoustic Hologram Reconstruction Filter")
        .with_param(DspParamSchema::knob("hologram_resolution", "Holographic Phase Steps", 16.0, 256.0, 64.0, "steps", MacroRole::Character, "Holographic Phase Steps integer control"))
        .with_param(DspParamSchema::knob("phase_offset_rad", "Acoustic Phase Bias", 0.0, 6.28, 0.0, "rad", MacroRole::Tone, "Acoustic Phase Bias parameter"))
        .with_param(DspParamSchema::knob("depth_focus_m", "Focal Distance", 0.2, 10.0, 2.0, "m", MacroRole::Tone, "Focal Distance parameter"))
        .with_param(DspParamSchema::knob("spatial_dispersion", "Diffractive Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Diffractive Smear parameter"))
    );
    descriptors.insert("LinearPhaseCrossoverNode".to_string(), DspNodeDescriptor::new("LinearPhaseCrossoverNode", "4-Way Linear Phase Mastering Crossover Filter", DspNodeCategory::FilterEq, "4-Way Linear Phase Mastering Crossover Filter")
        .with_param(DspParamSchema::log_knob("low_cross_hz", "Low Crossover Split", 40.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Low Crossover Split logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("mid_cross_hz", "Mid Crossover Split", 400.0, 3000.0, 1200.0, "Hz", MacroRole::Tone, "Mid Crossover Split logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("high_cross_hz", "High Crossover Split", 3000.0, 16000.0, 6000.0, "Hz", MacroRole::Tone, "High Crossover Split logarithmic parameter"))
        .with_param(DspParamSchema::knob("fir_tap_order", "FIR Filter Tap Count", 256.0, 4096.0, 1024.0, "taps", MacroRole::Character, "FIR Filter Tap Count integer control"))
    );
    descriptors.insert("CombResonatorNode".to_string(), DspNodeDescriptor::new("CombResonatorNode", "Tuned Feedback Comb Filter Resonator", DspNodeCategory::FilterEq, "Tuned Feedback Comb Filter Resonator")
        .with_param(DspParamSchema::log_knob("frequency_hz", "Comb Fundamental Pitch", 20.0, 5000.0, 440.0, "Hz", MacroRole::Tone, "Comb Fundamental Pitch logarithmic parameter"))
        .with_param(DspParamSchema::knob("feedback_gain", "Feedback Loop Resonance", -0.99, 0.99, 0.85, "%", MacroRole::Tone, "Feedback Loop Resonance parameter"))
        .with_param(DspParamSchema::knob("damping_ratio", "High Frequency Damping", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "High Frequency Damping parameter"))
        .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Blend", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Blend parameter"))
    );
    descriptors.insert("AudiogramLoudnessModel".to_string(), DspNodeDescriptor::new("AudiogramLoudnessModel", "Adaptive Psychoacoustic Audiogram Loudness Model", DspNodeCategory::FilterEq, "Equal-loudness compensation curve compensating for individual hearing profiles and Fletcher-Munson contours")
        .with_param(DspParamSchema::knob("loss_db_low", "Low-Frequency Loss Compensation (250Hz)", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Bass hearing threshold compensation parameter"))
        .with_param(DspParamSchema::knob("loss_db_mid", "Mid-Frequency Loss Compensation (1kHz)", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Mid-range vocal clarity hearing compensation parameter"))
        .with_param(DspParamSchema::knob("loss_db_high", "High-Frequency Loss Compensation (4kHz)", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Treble presence hearing threshold compensation parameter"))
    );
    descriptors.insert("HeadBumpFilter".to_string(), DspNodeDescriptor::new("HeadBumpFilter", "Magnetic Tape Head-Bump Resonator & Low-End Peaking", DspNodeCategory::FilterEq, "Accurate analog tape reproductive head-bump peaking filter modeling low-frequency magnetic gap resonances at 7.5, 15, and 30 IPS")
        .with_param(DspParamSchema::choice("speed_ips", "Tape Operational Speed", &["7.5 IPS (Warm 50Hz)", "15 IPS (Punchy 80Hz)", "30 IPS (HiFi 120Hz)"], 1, "Tape operational speed mode selector"))
        .with_param(DspParamSchema::log_knob("center_freq_hz", "Resonance Center", 30.0, 200.0, 80.0, "Hz", MacroRole::Tone, "Head bump resonance center frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("q_factor", "Peaking Q", 0.5, 3.0, 1.2, "Q", MacroRole::Character, "Peaking resonance bandwidth parameter"))
        .with_param(DspParamSchema::knob("boost_db", "Head Bump Boost", 0.0, 6.0, 2.5, "dB", MacroRole::Punch, "Low frequency tape head boost gain parameter"))
    );
    descriptors.insert("LinkwitzRiley4BandCrossover".to_string(), DspNodeDescriptor::new("LinkwitzRiley4BandCrossover", "Linkwitz-Riley 4-Band 24dB/oct Phase-Flat Mastering Crossover", DspNodeCategory::FilterEq, "Precision LR4 (4th-order 24dB/octave) 4-band mastering crossover network ensuring zero magnitude ripple and flat phase summation across all split points")
        .with_param(DspParamSchema::log_knob("split_low_hz", "Low/Mid Crossover", 40.0, 500.0, 120.0, "Hz", MacroRole::Tone, "Sub-bass to low-mid split frequency parameter"))
        .with_param(DspParamSchema::log_knob("split_mid_hz", "Mid/High Crossover", 500.0, 4000.0, 2500.0, "Hz", MacroRole::Tone, "Low-mid to high-mid split frequency parameter"))
        .with_param(DspParamSchema::log_knob("split_air_hz", "High/Air Crossover", 4000.0, 16000.0, 8000.0, "Hz", MacroRole::Tone, "High-mid to air band split frequency parameter"))
        .with_param(DspParamSchema::toggle("phase_align", "Linear Phase Summation", true, "Allpass phase delay compensation across all crossover bands"))
    );
    descriptors.insert("FormantVowelMatrix".to_string(), DspNodeDescriptor::new("FormantVowelMatrix", "5-Formant Acoustic Vowel Morphing Filter Matrix", DspNodeCategory::FilterEq, "Parallel 5-formant vocal vowel filter matrix simulating acoustic vocal tract oral and nasal resonances with smooth vowel interpolation and tract scaling")
        .with_param(DspParamSchema::knob("vowel_formant_morph", "Vowel Formant Morph A-E-I-O-U", 0.0, 4.0, 0.0, "idx", MacroRole::Tone, "Continuous vocalic formant interpolation index parameter"))
        .with_param(DspParamSchema::knob("vocal_tract_size", "Vocal Tract Geometry Scale", 0.5, 2.0, 1.0, "x", MacroRole::Character, "Acoustic vocal tract length scaling factor parameter"))
        .with_param(DspParamSchema::knob("formant_resonance_q", "Formant Filter Resonance Q", 1.0, 30.0, 8.0, "Q", MacroRole::Tone, "Formant filter resonance peak focus parameter"))
        .with_param(DspParamSchema::knob("consonant_brightness", "Fricative Consonant Air", 0.0, 1.0, 0.35, "%", MacroRole::Space, "High-frequency fricative consonant airy shimmer parameter"))
    );
    descriptors.insert("SubHarmonicQuantumTunnelingFilter".to_string(), DspNodeDescriptor::new("SubHarmonicQuantumTunnelingFilter", "Sub-Harmonic Quantum Tunneling Filter", DspNodeCategory::FilterEq, "Quantum mechanical tunneling transmission filter creating non-linear sub-harmonics without zero-crossing step distortion")
        .with_param(DspParamSchema::knob("barrier_height", "Tunneling Potential Barrier V0", 0.1, 5.0, 1.2, "V", MacroRole::Tone, "Height of quantum potential energy barrier parameter"))
        .with_param(DspParamSchema::knob("subharmonic_ratio", "Subharmonic Frequency Division", 0.1, 4.0, 1.0, "x", MacroRole::Punch, "Phase advancement rate of generated subharmonic component parameter"))
        .with_param(DspParamSchema::knob("transmission_prob", "Tunneling Probability T", 0.01, 1.0, 0.45, "%", MacroRole::Character, "Exponential wave transmission probability factor parameter"))
        .with_param(DspParamSchema::knob("harmonic_mix", "Subharmonic Wet Level", 0.0, 1.0, 0.30, "%", MacroRole::Space, "Wet mix level of generated quantum tunneling subharmonic parameter"))
    );
    descriptors.insert("CrystalModalFilter".to_string(), DspNodeDescriptor::new("CrystalModalFilter", "Narrowband Piezoelectric Crystal Modal Resonant Filter", DspNodeCategory::FilterEq, "Ultra-high-Q piezoelectric quartz crystal resonant filter simulating crystal radio IF tanks and ringing glass overtones")
        .with_param(DspParamSchema::log_knob("center_freq", "Crystal Resonance Frequency", 50.0, 15000.0, 2400.0, "Hz", MacroRole::Tone, "Series resonance frequency of quartz crystal slice parameter"))
        .with_param(DspParamSchema::knob("crystal_q", "Crystal Quality Factor Q", 10.0, 500.0, 85.0, "Q", MacroRole::Character, "Sharpness and ringing decay time of crystalline mode parameter"))
        .with_param(DspParamSchema::knob("piezo_gain", "Piezoelectric Coupling Gain", 0.0, 4.0, 1.2, "x", MacroRole::Punch, "Piezoelectric electromechanical voltage transfer ratio parameter"))
        .with_param(DspParamSchema::knob("damping_factor", "Crystal Mechanical Loss", 0.0001, 0.05, 0.005, "eta", MacroRole::Space, "Internal viscous quartz friction coefficient parameter"))
    );
    descriptors.insert("LinkwitzRiley4WaySplitter".to_string(), DspNodeDescriptor::new("LinkwitzRiley4WaySplitter", "Linkwitz-Riley 4-Way Phase-Aligned Crossover Splitter", DspNodeCategory::FilterEq, "4th-order Linkwitz-Riley 24dB/oct phase-aligned multi-band crossover splitter with cascaded allpass compensation")
        .with_param(DspParamSchema::log_knob("crossover_freq_hz", "Crossover Frequency", 40.0, 16000.0, 1000.0, "Hz", MacroRole::Tone, "Linkwitz-Riley 24dB/oct cutoff frequency parameter"))
        .with_param(DspParamSchema::toggle("phase_compensation", "Allpass Phase Alignment", true, "Cascaded allpass compensation preserving flat acoustic sum parameter"))
        .with_param(DspParamSchema::knob("branch_attenuation_db", "Crossfade Bleed", -12.0, 0.0, 0.0, "dB", MacroRole::Punch, "Attenuates crossover transition band bleed parameter"))
        .with_param(DspParamSchema::toggle("solo_split_branch", "Solo High Branch", false, "Audition isolated high frequency split branch parameter"))
    );
    descriptors.insert("DispersionAllpassChain".to_string(), DspNodeDescriptor::new("DispersionAllpassChain", "Multi-Stage Allpass Phase Dispersion Delay Ladder", DspNodeCategory::FilterEq, "Cascaded allpass delay structure producing frequency-dependent phase delay and stiffness dispersion for physical strings, tines, and brass bells")
        .with_param(DspParamSchema::knob("dispersion_stages_count", "Cascaded Stages", 1.0, 32.0, 8.0, "stages", MacroRole::Character, "Active number of cascaded first/second-order allpass stages parameter"))
        .with_param(DspParamSchema::knob("inharmonicity_coefficient_b", "Inharmonicity Factor B", 0.0001, 0.1, 0.012, "coeff", MacroRole::Tone, "Dispersion inharmonicity coefficient B governing frequency spread parameter"))
        .with_param(DspParamSchema::knob("dispersion_cutoff_hz", "Corner Break Freq", 100.0, 18000.0, 2400.0, "Hz", MacroRole::Tone, "Phase dispersion transition transition corner frequency parameter"))
        .with_param(DspParamSchema::knob("delay_phase_warp", "Phase Warp Angle", 0.0, 1.0, 0.45, "warp", MacroRole::Punch, "Non-linear phase curvature warping factor parameter"))
        .with_param(DspParamSchema::toggle("bypass_high_frequencies", "Bypass High Frequencies", false, "Bypass allpass phase dispersion above Nyquist transition parameter"))
    );
    descriptors.insert("TineDispersionAllpass".to_string(), DspNodeDescriptor::new("TineDispersionAllpass", "Electric Piano Tine Inharmonic Dispersion Allpass Filter", DspNodeCategory::FilterEq, "Specialized bi-quadratic allpass filter modeling the high inharmonic dispersion and stiff cantilever beam bending of Rhodes/Wurlitzer electric piano tines")
        .with_param(DspParamSchema::knob("tine_inharmonicity", "Tine Inharmonicity", 0.001, 0.5, 0.065, "disp", MacroRole::Tone, "Cantilever beam bending stiffness and inharmonicity coefficient parameter"))
        .with_param(DspParamSchema::knob("center_frequency_hz", "Center Frequency", 50.0, 8000.0, 880.0, "Hz", MacroRole::Tone, "Bi-quadratic allpass center tuning frequency parameter"))
        .with_param(DspParamSchema::knob("phase_slope_q", "Phase Dispersion Q", 0.1, 10.0, 1.8, "Q", MacroRole::Character, "Allpass phase delay transition resonance Q parameter"))
        .with_param(DspParamSchema::knob("clamping_stiffness", "Mount Clamping Rigidity", 0.1, 2.0, 1.0, "stiff", MacroRole::Punch, "Tine tonebar mechanical clamping mount rigidity parameter"))
        .with_param(DspParamSchema::knob("pickup_coupling_depth", "Pickup Flux Coupling", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Electromagnetic pickup flux interaction depth parameter"))
    );
    descriptors.insert("SitarDispersionAllpass".to_string(), DspNodeDescriptor::new("SitarDispersionAllpass", "Sitar String Stiff Dispersion Allpass Stage", DspNodeCategory::FilterEq, "Allpass filter chain capturing the high tension and flexural stiffness dispersion along classical steel and bronze sitar strings")
        .with_param(DspParamSchema::knob("string_stiffness_parameter", "String Stiffness B", 0.0001, 0.05, 0.008, "B", MacroRole::Tone, "Flexural stiffness parameter B of the primary playing string parameter"))
        .with_param(DspParamSchema::knob("taraf_coupling_cutoff_hz", "Taraf Coupling Cutoff", 200.0, 12000.0, 3500.0, "Hz", MacroRole::Space, "High-frequency cutoff for energy transfer into sympathetic taraf strings parameter"))
        .with_param(DspParamSchema::knob("non_linear_tension_mod", "Tension Modulation Depth", 0.0, 1.0, 0.40, "%", MacroRole::Character, "Dynamic pitch glide modulation depth under large amplitudes parameter"))
        .with_param(DspParamSchema::knob("high_freq_allpass_damping", "High Freq Dispersion Loss", 0.01, 1.0, 0.15, "loss", MacroRole::Tone, "Viscous air damping on high-frequency dispersed harmonics parameter"))
        .with_param(DspParamSchema::knob("bridge_offset_phase", "Jawari Contact Angle", 0.0, 360.0, 45.0, "deg", MacroRole::Punch, "Parabolic jawari bridge boundary contact angle phase parameter"))
    );
    descriptors.insert("MetamaterialRefractionFilterNode".to_string(), DspNodeDescriptor::new("MetamaterialRefractionFilterNode", "Negative-Index Acoustic Metamaterial Refractive Prism Filter", DspNodeCategory::FilterEq, "Negative-index acoustic metamaterial refractive prism bandpass filter with anomalous acoustic refraction and sub-wavelength resonance")
        .with_param(DspParamSchema::log_knob("center_frequency_hz", "Refraction Band Center", 100.0, 16000.0, 1200.0, "Hz", MacroRole::Tone, "Center passband frequency of negative refraction region"))
        .with_param(DspParamSchema::knob("refractive_index", "Effective Acoustic Index n", -3.0, 3.0, -1.2, "n", MacroRole::Character, "Effective acoustic refractive index"))
        .with_param(DspParamSchema::knob("unit_cell_resonance_q", "Unit-Cell Resonator Q", 1.0, 50.0, 12.0, "Q", MacroRole::Punch, "Quality factor of sub-wavelength Helmholtz inclusions"))
        .with_param(DspParamSchema::knob("dispersion_steepness", "Spatial Dispersion Gradient", 0.1, 10.0, 2.5, "grad", MacroRole::Tone, "Rate of group velocity variation across transition frequency"))
        .with_param(DspParamSchema::knob("evanescent_gain_db", "Evanescent Wave Amplification", -24.0, 24.0, 6.0, "dB", MacroRole::Space, "Super-resolution near-field evanescent wave restoration gain"))
    );
    descriptors.insert("Butterworth2ndOrder".to_string(), DspNodeDescriptor::new("Butterworth2ndOrder", "2nd-Order Maximally Flat Butterworth Filter", DspNodeCategory::FilterEq, "Precision 2nd-order Butterworth biquad filter section offering maximally flat passband response across Lowpass, Highpass, and Allpass modes")
        .with_param(DspParamSchema::choice("filter_type", "Filter Response Mode", &["Lowpass (12 dB/oct)", "Highpass (12 dB/oct)", "Allpass Phase Match"], 0, "Frequency attenuation filter mode"))
        .with_param(DspParamSchema::log_knob("cutoff_frequency_hz", "Cutoff Frequency (-3 dB)", 10.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Half-power corner frequency"))
        .with_param(DspParamSchema::knob("q_resonance", "Quality Factor (Q)", 0.1, 10.0, 0.707, "Q", MacroRole::Punch, "Filter pole Q resonance (0.7071 = Butterworth critical damping)"))
        .with_param(DspParamSchema::knob("gain_db", "Section Gain Offset", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Output makeup gain boost or cut"))
        .with_param(DspParamSchema::toggle("bypass", "Hard Bypass Filter", false, "Bypasses filter processing with bit-transparent passthrough"))
    );
    descriptors.insert("ModalFilterSection".to_string(), DspNodeDescriptor::new("ModalFilterSection", "Narrowband Resonant Modal Filter Section", DspNodeCategory::FilterEq, "Second-order resonant bandpass modal filter section modeling individual structural resonance modes in idiophones, membranes, and strings")
        .with_param(DspParamSchema::log_knob("freq_hz", "Modal Peak Frequency", 20.0, 16000.0, 440.0, "Hz", MacroRole::Tone, "Tuned resonance center frequency of this modal partial"))
        .with_param(DspParamSchema::knob("t60_sec", "Modal Ringdown Decay T60", 0.01, 20.0, 2.0, "s", MacroRole::Space, "Time required for vibration energy of this mode to decay by 60 dB"))
        .with_param(DspParamSchema::knob("amplitude", "Modal Relative Amplitude", 0.0, 2.0, 1.0, "amp", MacroRole::Punch, "Initial modal gain weight when excited by strike impulse"))
        .with_param(DspParamSchema::knob("phase", "Initial Mode Phase", 0.0, 360.0, 0.0, "deg", MacroRole::Character, "Starting phase angle of mode oscillation"))
        .with_param(DspParamSchema::knob("damping_scale", "Damping Coefficient Scale", 0.1, 5.0, 1.0, "scale", MacroRole::Tone, "Multiplier on decay rate for dynamic damping modulation"))
    );
    descriptors.insert("DispersionAllpass".to_string(), DspNodeDescriptor::new("DispersionAllpass", "1st-Order Inharmonic Phase Dispersion Allpass", DspNodeCategory::FilterEq, "First-order allpass phase filter simulating inharmonic flexural wave dispersion in stiff piano strings, steel bars, and metal plates")
        .with_param(DspParamSchema::knob("coefficient", "Allpass Alpha Coefficient", -0.95, 0.95, 0.35, "α", MacroRole::Character, "Allpass feedback coefficient governing phase curvature"))
        .with_param(DspParamSchema::log_knob("frequency_warp_hz", "Dispersion Corner Frequency", 100.0, 10000.0, 2000.0, "Hz", MacroRole::Tone, "Transition frequency where maximum phase delay shift occurs"))
        .with_param(DspParamSchema::knob("inharmonicity_b", "Stiffness Inharmonicity B", 0.0001, 0.05, 0.005, "B", MacroRole::Tone, "Physical stiffness coefficient shifting upper harmonics sharp"))
        .with_param(DspParamSchema::knob("phase_delay_samples", "Group Delay at DC", 0.1, 10.0, 1.2, "smp", MacroRole::Space, "Nominal low-frequency phase delay in samples"))
        .with_param(DspParamSchema::toggle("bypass", "Bypass Dispersion", false, "Disables phase delay shift"))
    );
    descriptors.insert("ConsoleBiquad".to_string(), DspNodeDescriptor::new("ConsoleBiquad", "Analog Console Desk Transformer Peaking EQ Biquad", DspNodeCategory::FilterEq, "Second-order analog-modeled peaking biquad filter section simulating vintage British and American console transformer LF/HF resonant bumps")
        .with_param(DspParamSchema::log_knob("freq_hz", "EQ Center Frequency", 10.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Center frequency of bell or transformer shelf curve"))
        .with_param(DspParamSchema::knob("gain_db", "EQ Boost / Cut Gain", -24.0, 24.0, 3.0, "dB", MacroRole::Punch, "Peaking boost or cut amplitude in decibels"))
        .with_param(DspParamSchema::knob("q_factor", "Bandwidth Q Factor", 0.1, 10.0, 1.0, "Q", MacroRole::Tone, "Resonance bandwidth of console equalizer bell"))
        .with_param(DspParamSchema::knob("analog_warmth", "Non-linear Analog Warmth", 0.0, 1.0, 0.40, "%", MacroRole::Character, "Subtle soft clipping inside biquad delay feedback loops"))
        .with_param(DspParamSchema::toggle("bypass", "Bypass Equalizer Band", false, "Completely bypasses biquad filter section"))
    );
    descriptors.insert("VowelGraph".to_string(), DspNodeDescriptor::new("VowelGraph", "Acoustic Vowel Formant Transition & Trajectory Graph", DspNodeCategory::FilterEq, "Topological formant filter network navigating continuous vocal tract trajectories [i], [e], [a], [o], [u] with 2D vowel plane interpolation")
        .with_param(DspParamSchema::log_knob("formant_f1_hz", "Formant F1 Frequency", 150.0, 1500.0, 500.0, "Hz", MacroRole::Tone, "First acoustic formant frequency governing vowel height/openness"))
        .with_param(DspParamSchema::log_knob("formant_f2_hz", "Formant F2 Frequency", 500.0, 4000.0, 1500.0, "Hz", MacroRole::Tone, "Second acoustic formant frequency governing vowel backness/frontness"))
        .with_param(DspParamSchema::knob("vowel_position_x", "Vowel Chart Openness X", 0.0, 1.0, 0.50, "%", MacroRole::Character, "Continuous 2D vowel chart horizontal position"))
        .with_param(DspParamSchema::knob("vowel_position_y", "Vowel Chart Backness Y", 0.0, 1.0, 0.50, "%", MacroRole::Character, "Continuous 2D vowel chart vertical position"))
        .with_param(DspParamSchema::knob("nasalization", "Velum Nasalization", 0.0, 1.0, 0.15, "%", MacroRole::Punch, "Velopharyngeal port coupling introducing nasal resonance formant antizero"))
    );
    descriptors.insert("LadderFilterBode".to_string(), DspNodeDescriptor::new("LadderFilterBode", "Moog 24dB/oct Diode/Transistor Ladder Filter Bode Plot HUD", DspNodeCategory::FilterEq, "Interactive 4-pole transistor and diode ladder filter visualizer rendering frequency response curve, resonance peak, self-oscillation limit cycle, and non-linear drive")
        .with_param(DspParamSchema::log_knob("cutoff_freq_hz", "Moog Ladder Cutoff Frequency", 20.0, 20000.0, 1450.0, "Hz", MacroRole::Tone, "Cutoff corner frequency of the 4-pole cascaded filter stages"))
        .with_param(DspParamSchema::knob("resonance_q", "4-Pole Ladder Resonance Q", 0.0, 10.0, 6.5, "Q", MacroRole::Punch, "Feedback gain boosting cutoff peak into screaming self-oscillation"))
        .with_param(DspParamSchema::knob("drive_saturation", "Transistor Non-Linear Saturation Drive", 0.0, 100.0, 35.0, "%", MacroRole::Character, "Differential transistor pair overdrive generating warm analog harmonics"))
        .with_param(DspParamSchema::choice("topology", "Circuit Topology Emulation", &["Moog Transistor 4-Pole", "TB-303 Diode Ladder", "SEM 2-Pole SVF", "Korg MS-20 Sallen-Key"], 0, "Analog filter circuit component modeling character"))
        .with_param(DspParamSchema::knob("key_tracking_percent", "Keyboard Pitch Tracking Amount", 0.0, 100.0, 50.0, "%", MacroRole::Tone, "Keyboard note tracking scaling cutoff frequency with MIDI pitch"))
    );
    descriptors.insert("VowelSpace".to_string(), DspNodeDescriptor::new("VowelSpace", "2D IPA International Phonetic Alphabet Vowel Space Formant HUD", DspNodeCategory::FilterEq, "Interactive 2D vowel formant canvas mapping F1/F2 frequency coordinates across international phonetic alphabet vowel spaces with glide morphing")
        .with_param(DspParamSchema::log_knob("f1_hz", "Formant F1 Frequency (Vowel Height / Openness)", 200.0, 950.0, 280.0, "Hz", MacroRole::Tone, "First formant resonant frequency governing jaw openness"))
        .with_param(DspParamSchema::log_knob("f2_hz", "Formant F2 Frequency (Vowel Backness)", 600.0, 2600.0, 2250.0, "Hz", MacroRole::Tone, "Second formant resonant frequency governing tongue forward/back position"))
        .with_param(DspParamSchema::log_knob("f3_hz", "Formant F3 Frequency", 1800.0, 3200.0, 2500.0, "Hz", MacroRole::Character, "Third formant frequency establishing individual vocal tract timbre and rhotacism"))
        .with_param(DspParamSchema::choice("vowel_selection", "Target IPA Vowel Anchor", &["Close Front [i]", "Close Back [u]", "Open Front [a]", "Open Back [ɑ]", "Mid Central Schwa [ə]", "Near-Close [ɪ]"], 0, "Reference standard international phonetic vowel chart coordinate"))
        .with_param(DspParamSchema::knob("morph_slew_ms", "Formant Transition Glide Time", 5.0, 500.0, 45.0, "ms", MacroRole::Space, "Portamento interpolation duration when transitioning between vowels"))
    );
    descriptors.insert("SpectralMorphEngine".to_string(), DspNodeDescriptor::new("SpectralMorphEngine", "FFT Spectral Morphing & Cross-Synthesis Engine HUD", DspNodeCategory::FilterEq, "Real-time FFT dual-source spectral cross-synthesis interpolating magnitude and phase spectra with spectral envelope preservation")
        .with_param(DspParamSchema::knob("morph_factor", "Spectral Magnitude Morph Position", 0.0, 1.0, 0.50, "%", MacroRole::Tone, "Continuous morphing interpolation between Spectrum Source A and Source B"))
        .with_param(DspParamSchema::choice("fft_window_size", "Short-Time Fourier Transform Window Size", &["256 Bins (Ultra Fast)", "512 Bins (Transient)", "1024 Bins (Balanced)", "2048 Bins (High Res)", "4096 Bins (Deep Lows)"], 2, "FFT analysis window length balancing time and frequency resolution"))
        .with_param(DspParamSchema::knob("spectral_tilt_db", "Spectral Envelope Spectral Tilt Balance", -12.0, 12.0, 0.0, "dB", MacroRole::Character, "Linear spectral slope boosting high-frequency bins relative to fundamentals"))
        .with_param(DspParamSchema::toggle("transient_bypass", "Transient Phase Preservation Bypass", true, "Preserves sharp attack transients un-smeared outside the FFT phase bin synthesis"))
        .with_param(DspParamSchema::choice("phase_lock_mode", "FFT Bin Phase Synchronization Mode", &["Strict Phase Lock", "Randomized Diffusion", "Minimum Phase Warping"], 0, "Phase coherence locking preventing dispersion and comb-filtering artifacts"))
    );
    descriptors.insert("BesselFilterBank".to_string(), DspNodeDescriptor::new("BesselFilterBank", "Bessel Linear-Phase Group-Delay Preserving Filter Bank HUD", DspNodeCategory::FilterEq, "Linear-phase Bessel filter bank engineered for maximum group-delay flatness across passbands, preserving the exact time-domain pulse shapes and transients of acoustic audio signals")
        .with_param(DspParamSchema::choice("filter_order", "Bessel Polynomial Filter Topology & Slope Steepness", &["2nd-Order (12dB/oct)", "4th-Order (24dB/oct)", "6th-Order (36dB/oct)", "8th-Order (48dB/oct)"], 1, "Number of poles in Bessel polynomial governing attenuation slope steepness"))
        .with_param(DspParamSchema::choice("filter_mode", "Active Spectral Passband Filtering Response", &["Lowpass Transient Preserving", "Highpass DC/Rumble Cleanse", "Bandpass Resonance Band", "Notch Band Reject"], 0, "Passband routing selecting lowpass, highpass, bandpass, or notch characteristics"))
        .with_param(DspParamSchema::log_knob("cutoff_frequency_hz", "Corner Cutoff Transition Frequency", 20.0, 20000.0, 2500.0, "Hz", MacroRole::Tone, "Corner transition frequency where passband group delay flatness terminates"))
        .with_param(DspParamSchema::knob("group_delay_flatness_factor", "Group Delay Flatness Slew Factor", 0.5, 2.0, 1.0, "factor", MacroRole::Character, "Fine-tunes polynomial damping factor to maximize time-domain pulse fidelity"))
        .with_param(DspParamSchema::toggle("time_domain_overshoot_suppression", "Suppress Time-Domain Step-Response Overshoot / Ringing", true, "Enforces zero step-response overshoot characteristic unique to Bessel filters"))
    );
    descriptors.insert("StateVariableOversampler".to_string(), DspNodeDescriptor::new("StateVariableOversampler", "8x Polyphase Oversampling State-Variable Multi-Mode Filter HUD", DspNodeCategory::FilterEq, "Polyphase 8x oversampled state-variable filter utilizing zero-delay feedback (ZDF) topology and nonlinear drive saturation to produce pristine analog resonance without digital cramping")
        .with_param(DspParamSchema::log_knob("cutoff_frequency_hz", "Cutoff Corner Resonant Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Resonant cutoff frequency with zero cramping near the Nyquist limit"))
        .with_param(DspParamSchema::knob("resonance_q", "Self-Oscillating Resonance Peak Sharpness", 0.5, 25.0, 2.8, "Q", MacroRole::Punch, "Filter resonance quality factor with smooth tanh self-oscillation limiting"))
        .with_param(DspParamSchema::choice("filter_output_blend", "State-Variable Multi-Mode Output Pole Selection", &["12dB/oct Lowpass", "12dB/oct Highpass", "6dB/oct Bandpass", "Variable Notch Reject", "Allpass Phase Disperser"], 0, "Simultaneous state-variable filter output tap selection"))
        .with_param(DspParamSchema::choice("polyphase_oversampling_factor", "Anti-Aliasing Polyphase FIR Oversampling Multiplier", &["1x (Native Sample Rate)", "2x Low-Latency Polyphase", "4x Studio Standard", "8x Ultra Mastering Quality"], 3, "Polyphase FIR upsampling and decimation factor suppressing nonlinear aliasing"))
        .with_param(DspParamSchema::knob("analog_transistor_saturation", "Nonlinear Transistor Core Overdrive & Thermal Saturation", 0.0, 12.0, 1.5, "dB", MacroRole::Character, "Simulated analog differential pair transistor overdrive warming the filter core"))
    );
    descriptors.insert("ParametricDynamicEq".to_string(), DspNodeDescriptor::new("ParametricDynamicEq", "4-Band Dynamic Parametric Equalizer with Envelope Detectors HUD", DspNodeCategory::FilterEq, "4-band surgical dynamic equalizer with independent per-band threshold, ratio, attack, and release detectors, reacting dynamically to frequency spikes and harsh resonances in real time")
        .with_param(DspParamSchema::log_knob("band2_center_frequency_hz", "Mid-Band Dynamic Resonant Peak Frequency", 100.0, 10000.0, 1000.0, "Hz", MacroRole::Tone, "Center frequency of primary dynamic parametric filter bell"))
        .with_param(DspParamSchema::knob("band2_dynamic_range_db", "Maximum Dynamic Gain Expansion or Cut Attenuation", -18.0, 18.0, -6.0, "dB", MacroRole::Punch, "Maximum permissible dynamic gain boost or compression attenuation limit"))
        .with_param(DspParamSchema::knob("band2_threshold_db", "Dynamic Detector Level Activation Threshold", -48.0, 0.0, -18.0, "dB", MacroRole::Tone, "Sidechain RMS level above which dynamic EQ processing begins to act"))
        .with_param(DspParamSchema::knob("band2_ratio", "Dynamic Compression/Expansion Slope Ratio", 1.0, 10.0, 2.5, ":1", MacroRole::Character, "Slope ratio determining how aggressively gain responds to signal exceeding threshold"))
        .with_param(DspParamSchema::knob("band2_attack_ms", "Transient Attack Smoothing Time Constant", 0.5, 100.0, 12.0, "ms", MacroRole::Space, "Envelope detector attack time constant governing onset response speed"))
    );
    descriptors.insert("EqBand".to_string(), DspNodeDescriptor::new("EqBand", "Parametric Equalizer Biquad Filter Band", DspNodeCategory::FilterEq, "Surgical minimum-phase parametric biquad equalizer filter band with proportional-Q response, frequency modulation, and dynamic threshold gain")
        .with_param(DspParamSchema::log_knob("frequency_hz", "Center Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Center resonant frequency of the equalizer band"))
        .with_param(DspParamSchema::knob("gain_db", "Bell Gain Boost/Cut", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Peak/shelf amplification or attenuation gain in decibels"))
        .with_param(DspParamSchema::knob("q_factor", "Bandwidth Quality Factor Q", 0.1, 30.0, 1.414, "Q", MacroRole::Tone, "Filter resonance sharpness and bandwidth selectivity"))
        .with_param(DspParamSchema::choice("filter_slope", "Filter Response Slope & Shape", &["Bell Peaking", "Low Shelf", "High Shelf", "Bandpass", "Notch"], 0, "Selects active biquad filter mathematical topology"))
        .with_param(DspParamSchema::knob("dynamic_threshold_db", "Dynamic Expansion/Compression Threshold", -60.0, 0.0, -18.0, "dB", MacroRole::Punch, "Sidechain signal level triggering dynamic gain adjustments"))
    );
    descriptors.insert("AutoWahFilterMode".to_string(), DspNodeDescriptor::new("AutoWahFilterMode", "Dynamic Auto-Wah State-Variable Filter & Envelope Sweep Engine", DspNodeCategory::FilterEq, "Tactile auto-wah envelope follower modulating a multi-mode state-variable filter (Lowpass, Bandpass, Highpass, Peaking) with variable sensitivity, Q resonance, and up/down sweep direction")
        .with_param(DspParamSchema::choice("filter_sweep_topology", "Auto-Wah Filter Sweep Topology", &["Bandpass Vocal Wah (Classic Q)", "Lowpass Funk Sweep", "Highpass Laser Sweep", "Dual Bandpass Formant Wah"], 0, "Filter topology swept by dynamic envelope follower"))
        .with_param(DspParamSchema::knob("envelope_sensitivity", "Input Level Envelope Detection Sensitivity", 0.1, 10.0, 2.5, "gain", MacroRole::Punch, "Input gain driving envelope follower threshold"))
        .with_param(DspParamSchema::knob("filter_resonance_q", "Filter Resonant Peak Sharpness Q", 0.5, 30.0, 8.0, "Q", MacroRole::Tone, "Resonance peak intensity producing vocal wah vowel sound"))
        .with_param(DspParamSchema::choice("sweep_direction", "Envelope Cutoff Frequency Sweep Direction", &["Sweep Up (Level -> Higher Freq)", "Sweep Down (Level -> Lower Freq)", "Bipolar LFO / Follower Mix"], 0, "Direction of cutoff excursion with increasing signal volume"))
        .with_param(DspParamSchema::log_knob("base_frequency_hz", "Base Filter Frequency at Zero Input", 100.0, 2500.0, 450.0, "Hz", MacroRole::Character, "Resting center frequency before envelope modulation"))
        .with_param(DspParamSchema::knob("attack_release_speed_ms", "Envelope Follower Reaction Time", 5.0, 200.0, 25.0, "ms", MacroRole::Space, "Envelope follower integration time constant"))
    );
    descriptors.insert("SpectralMorphMode".to_string(), DspNodeDescriptor::new("SpectralMorphMode", "Frequency-Domain FFT Partial Morphing & Cross-Synthesis", DspNodeCategory::FilterEq, "Spectral interpolation engine performing seamless transformations between two live audio spectra with linear crossfade, spectral envelope warping, phase vocoder lock, and formant preservation")
        .with_param(DspParamSchema::choice("spectral_interpolation_algorithm", "Spectral Frequency Morphing Algorithm", &["Linear Partial Amplitude Crossfade", "Spectral Envelope Dynamic Warping", "Phase Vocoder Instantaneous Frequency Morph", "Formant-Preserving Vocal Cross-Synthesis"], 1, "Spectral interpolation mathematical algorithm"))
        .with_param(DspParamSchema::knob("morph_interpolation_factor", "Source A to Source B Morph Position", 0.0, 1.0, 0.5, "pos", MacroRole::Punch, "Continuous interpolation position between source A (0.0) and B (1.0)"))
        .with_param(DspParamSchema::knob("spectral_tilt_balance", "High-to-Low Frequency Energy Tilt", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "Spectral tilt applied across morphed partials"))
        .with_param(DspParamSchema::knob("formant_center_frequency_shift", "Formant Envelope Pitch Scaling Offset", 0.5, 2.0, 1.0, "scale", MacroRole::Character, "Formant envelope scaling independent of source pitch"))
        .with_param(DspParamSchema::knob("spectral_smearing_time_ms", "Inter-Frame FFT Smearing & Ghosting", 0.0, 100.0, 15.0, "ms", MacroRole::Space, "Temporal smearing time constant smoothing spectral transitions"))
        .with_param(DspParamSchema::toggle("lock_spectral_phase_coherence", "Enforce Instantaneous Phase Coherence", true, "Lock phase relationships across bins to prevent phase cancelation"))
    );
    descriptors.insert("AutoWahDirection".to_string(), DspNodeDescriptor::new("AutoWahDirection", "Envelope Follower Auto-Wah Frequency Sweep Direction", DspNodeCategory::FilterEq, "Dynamic envelope follower filter sweep direction controller selecting upward vocal funk quack vs. downward ducking sweeps with asymmetric ballistics and resonance boost")
        .with_param(DspParamSchema::choice("sweep_direction_mode", "Envelope Follower Sweep Direction", &["Upward Sweep (Classic Vocal Quack)", "Downward Sweep (Sub-Bass Ducking Quack)", "Bidirectional Symmetrical Inversion", "Bipolar S-Curve Morph"], 0, "Direction of filter center frequency deflection under envelope amplitude"))
        .with_param(DspParamSchema::knob("envelope_attack_time_ms", "Detector Attack Ballistics", 1.0, 100.0, 12.0, "ms", MacroRole::Punch, "Attack slew rate of envelope peak detector"))
        .with_param(DspParamSchema::knob("envelope_release_time_ms", "Detector Release Ballistics", 10.0, 1000.0, 180.0, "ms", MacroRole::Space, "Release decay slew rate of envelope detector"))
        .with_param(DspParamSchema::knob("filter_resonance_q", "Peak Resonance Q Sharpness", 1.0, 25.0, 8.5, "Q", MacroRole::Tone, "Resonance peak sharpness at wah center frequency"))
        .with_param(DspParamSchema::knob("envelope_sensitivity_db", "Input Dynamic Sensitivity Threshold", -40.0, 10.0, -14.0, "dB", MacroRole::Character, "Input audio level threshold driving maximum sweep"))
        .with_param(DspParamSchema::toggle("invert_sweep_polarities", "Invert Sweep Polarities on High Transients", false, "Invert sweep direction dynamically on sharp transient hits"))
    );
    descriptors.insert("FilterType".to_string(), DspNodeDescriptor::new("FilterType", "Universal Biquad Direct-Form II Transposed Filter Topology Engine", DspNodeCategory::FilterEq, "Universal direct-form II transposed biquad filter core providing lowpass, highpass, bandpass, notch, parametric peaking bell, and low/high shelf responses with smooth coefficient interpolation and internal drive saturation")
        .with_param(DspParamSchema::choice("filter_topology_type", "Biquad Filter Transfer Mode", &["Lowpass (12/24dB/oct Cutoff)", "Highpass (Low Rumble Elimination)", "Bandpass (Resonant Peak Pass)", "Notch (Harmonic Rejection)", "Peaking (Parametric Bell EQ)", "Low Shelf (Bass Boost/Cut)", "High Shelf (Air Treble Boost/Cut)"], 0, "Selected frequency filtering transfer architecture"))
        .with_param(DspParamSchema::knob("cutoff_center_frequency_hz", "Cutoff / Center Resonant Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Corner cutoff or center frequency of biquad filter"))
        .with_param(DspParamSchema::knob("resonance_q_factor", "Resonance Quality Q Factor / Bandwidth", 0.1, 30.0, 0.707, "Q", MacroRole::Character, "Resonance peaking sharpness or filter bandwidth"))
        .with_param(DspParamSchema::knob("peaking_shelf_gain_db", "Bell Peaking / Shelf Boost Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Gain adjustment for parametric peaking and shelf modes"))
        .with_param(DspParamSchema::knob("saturation_drive_amount", "Internal Core Saturation Drive", 0.0, 1.0, 0.15, "drive", MacroRole::Space, "Soft saturation applied to filter feedback state variables"))
        .with_param(DspParamSchema::toggle("enable_double_precision_64bit_eval", "64-Bit Double Precision Internal Float DSP", true, "Prevent limit cycles and numerical truncation noise in low bass"))
    );
    descriptors.insert("DcOffsetRemoverFilter".to_string(), DspNodeDescriptor::new("DcOffsetRemoverFilter", "Subsonic Zero-Phase DC Offset Eliminator & High-Pass Filter", DspNodeCategory::FilterEq, "Precision subsonic filter removing DC electrical offset and infrasonic rumble with zero group delay distortion")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "High-Pass Cutoff Frequency", 1.0, 30.0, 10.0, "Hz", MacroRole::Tone, "Subsonic cutoff corner frequency eliminating electrical DC and stage rumble"))
        .with_param(DspParamSchema::choice("filter_slope_steepness", "Filter Slope Steepness", &["12 dB/Oct Butterworth", "24 dB/Oct Linkwitz-Riley", "48 dB/Oct Steep Elliptic", "6 dB/Oct Gentle Quasi-Linear"], 1, "Attenuation slope steepness below cutoff frequency"))
        .with_param(DspParamSchema::toggle("linear_phase_mode", "Linear-Phase Zero Group Delay", true, "Eliminates low-frequency phase rotation to preserve drum transient impact"))
        .with_param(DspParamSchema::knob("dc_tracker_time_constant_sec", "DC Tracking Time Constant", 0.1, 5.0, 1.0, "s", MacroRole::None, "Sliding window integration time for tracking drifting DC bias offsets"))
        .with_param(DspParamSchema::toggle("bipolar_asymmetry_correction", "Correct Asymmetric Waveforms", true, "Balances positive and negative peak excursions for maximum headroom"))
        .with_param(DspParamSchema::toggle("infrasonic_energy_meter", "Display Infrasonic Energy Readout", true, "Renders real-time VU meter displaying energy below 20 Hz"))
    );
    descriptors.insert("MasterLinearPhaseEq".to_string(), DspNodeDescriptor::new("MasterLinearPhaseEq", "64-Bit Linear-Phase Mastering Equalizer & Brickwall Crossover", DspNodeCategory::FilterEq, "Mastering-grade linear-phase FIR equalizer featuring symmetrical impulse response, zero phase shift distortion across critical crossover frequencies, and variable FIR tap resolution")
        .with_param(DspParamSchema::knob("filter_bands_count", "Active Linear Phase Bands", 2.0, 12.0, 8.0, "bands", MacroRole::Tone, "Number of active parametric linear-phase filter bands"))
        .with_param(DspParamSchema::choice("fir_impulse_length_taps", "FIR Convolution Tap Resolution", &["2048 Taps (Low Latency)", "4096 Taps (Studio Standard)", "8192 Taps (Mastering Pristine)", "16384 Taps (Ultra Sinc)"], 2, "FIR kernel length determining low-frequency frequency resolution and stopband steepness"))
        .with_param(DspParamSchema::knob("crossover_phase_coupling", "Adjacent Band Phase Coupling", 0.0, 1.0, 1.0, "ratio", MacroRole::Character, "Phase coherence smoothing between neighboring crossover split points"))
        .with_param(DspParamSchema::knob("pre_ringing_suppression_db", "Pre-Ringing Transient Suppression", -48.0, 0.0, -24.0, "dB", MacroRole::Space, "Attenuates acausal pre-ringing artifacts preceding transient drum hits"))
        .with_param(DspParamSchema::knob("analog_phase_morph_ratio", "Minimum-to-Linear Phase Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Tone, "Smoothly interpolates filter kernel from pure linear phase (0.0) to analog minimum phase (1.0)"))
        .with_param(DspParamSchema::toggle("enable_high_precision_64bit", "64-Bit Float Double Precision Processing", true, "Executes all FIR convolutions with IEEE 754 double precision float math"))
    );
    descriptors.insert("DynamicResonanceSuppressor".to_string(), DspNodeDescriptor::new("DynamicResonanceSuppressor", "Spectral Auto-Tracking Dynamic Resonance Suppressor & De-Ringer", DspNodeCategory::FilterEq, "Intelligent dynamic spectral resonance suppressor tracking narrowband room modes, harsh cabinet peaks, and excessive vocal sibilance with zero-latency transient preservation")
        .with_param(DspParamSchema::knob("spectral_detection_sensitivity", "Resonance Detection Sensitivity", 0.1, 1.0, 0.65, "sens", MacroRole::Punch, "Threshold sensitivity for identifying standing wave resonances and harsh overtones"))
        .with_param(DspParamSchema::knob("suppression_depth_db", "Maximum Dynamic Suppression Depth", -24.0, 0.0, -6.0, "dB", MacroRole::Tone, "Maximum allowable dynamic gain cut applied to detected resonant peaks"))
        .with_param(DspParamSchema::knob("sharpness_q_selectivity", "Filter Bandwidth Selectivity Q", 1.0, 50.0, 12.0, "Q", MacroRole::Character, "Q factor governing tightness of dynamic notch carving"))
        .with_param(DspParamSchema::knob("attack_response_time_ms", "Detector Attack Response Time", 0.5, 50.0, 5.0, "ms", MacroRole::Punch, "Speed at which dynamic cuts clamp down on emerging resonant ringing"))
        .with_param(DspParamSchema::knob("release_recovery_time_ms", "Detector Release Recovery Time", 5.0, 500.0, 60.0, "ms", MacroRole::Space, "Recovery time constant releasing filter cuts once resonance decays"))
        .with_param(DspParamSchema::toggle("monitor_delta_audition", "Audition Suppressed Delta Signal Solo", false, "Isolates and monitors exclusively the removed resonant artifacts"))
    );
    descriptors.insert("SpectralDeHumFilter".to_string(), DspNodeDescriptor::new("SpectralDeHumFilter", "Adaptive AC Mains Hum & Harmonic Buzz Eliminator Filter", DspNodeCategory::FilterEq, "Zero-latency adaptive notch filter array hunting 50 Hz and 60 Hz AC electrical hum and up to 32 harmonics without attenuating fundamental musical bass frequencies")
        .with_param(DspParamSchema::choice("mains_base_frequency_hz", "AC Mains Fundamental Frequency", &["50 Hz (Europe/Asia 230V)", "60 Hz (Americas 120V)", "Auto-Detect Drift"], 1, "Base electrical mains frequency anchoring harmonic notch series"))
        .with_param(DspParamSchema::knob("harmonic_notch_count", "Tracked Harmonic Notches Count", 1.0, 32.0, 16.0, "harmonics", MacroRole::Tone, "Number of upper harmonic overtone notches deployed up to 4 kHz"))
        .with_param(DspParamSchema::knob("notch_bandwidth_q", "Surgical Notch Filter Q-Factor", 10.0, 100.0, 40.0, "Q", MacroRole::Character, "Sharpness of notch filters preventing loss of musical content adjacent to hum"))
        .with_param(DspParamSchema::knob("attenuation_notch_depth_db", "Hum Notch Attenuation Depth", -80.0, -12.0, -48.0, "dB", MacroRole::Tone, "Rejection attenuation applied at base mains frequency and harmonic multiples"))
        .with_param(DspParamSchema::toggle("dynamic_harmonic_tracking", "Dynamic Frequency Drift Tracking", true, "Continuously tracks small sub-Hertz fluctuations in mains generator frequency"))
        .with_param(DspParamSchema::knob("high_pass_ground_lift_hz", "Sub-Bass Highpass Ground Lift", 10.0, 60.0, 20.0, "Hz", MacroRole::None, "Auxiliary steep highpass eliminating severe ground loop DC offsets"))
    );
    descriptors.insert("NonlinearJawariBridge".to_string(), DspNodeDescriptor::new("NonlinearJawariBridge", "Sitar Non-Linear Curved Jawari Bridge & Tarab Sympathetic Resonator", DspNodeCategory::FilterEq, "Physical modeling of the parabolic curved deer-horn/camel-bone Jawari bridge producing dynamic time-varying spectral buzz, coupled with a 13-string sympathetic Tarab modal resonator and gourd soundbox")
        .with_param(DspParamSchema::choice("bridge_curvature_profile", "Jawari Parabolic Curvature Saddle", &["Flat Modern Ebony", "Gentle Sloping Deer Horn", "High Arch Camel Bone", "Extreme Sitar Wide Curve"], 1, "Parabolic curvature profile of the bridge saddle surface"))
        .with_param(DspParamSchema::knob("jawari_buzz_contact_depth", "Grazing String Buzz Contact Depth", 0.0, 1.0, 0.75, "depth", MacroRole::Character, "String-to-bridge grazing contact depth modulating higher harmonic emergence"))
        .with_param(DspParamSchema::choice("raga_scale_tuning", "Sympathetic Tarab Raga Tuning Scale", &["Raga Yaman (Kalyan)", "Raga Bhairav (Dawn)", "Raga Bilawal (Morning)", "Raga Darbari Kanhada (Midnight)", "Raga Todi (Morning)", "Raga Kafi (Folk)", "Raga Bhairavi (Finale)"], 0, "Scale tuning system for the 13 sympathetic Tarab string modal bank"))
        .with_param(DspParamSchema::knob("tarab_sympathetic_coupling", "Tarab Sympathetic Acoustic Transfer", 0.0, 1.0, 0.65, "gain", MacroRole::Space, "Acoustic transfer from main melody string into sympathetic strings"))
        .with_param(DspParamSchema::knob("kaddu_gourd_air_resonance", "Kaddu Gourd Helmholtz Air Cavity", 80.0, 400.0, 142.0, "Hz", MacroRole::Tone, "Helmholtz resonant air mode of the dried Kaddu gourd body cavity"))
        .with_param(DspParamSchema::knob("tabli_spruce_wood_warmth", "Flat Spruce Tabli Soundboard Warmth", 0.0, 1.0, 0.8, "warmth", MacroRole::Tone, "Flat spruce tabli soundboard damping and mid-range richness"))
    );
    descriptors.insert("CassottoToneChamber".to_string(), DspNodeDescriptor::new("CassottoToneChamber", "Cassotto Wooden Tone Chamber & Soundbox Resonator", DspNodeCategory::FilterEq, "Physical modeling of enclosed wooden Cassotto tone chambers found in concert bandoneons and double-cassotto accordions, imparting warm vocal formants and steep acoustic overtone attenuation")
        .with_param(DspParamSchema::choice("chamber_acoustic_profile", "Cassotto Tone Chamber Geometry Profile", &["Doble Cassotto Bandoneon", "Standard Accordion Maple Chamber", "Vintage Concertina Spruce Body", "Harmonium Resonant Soundbox", "Open-Air Bayan (Direct)"], 0, "Chamber geometry, wood species, and acoustic modal signature"))
        .with_param(DspParamSchema::knob("cassotto_warmth_factor", "Low-Mid Cavity Formant Warmth", 0.0, 1.0, 0.82, "warmth", MacroRole::Tone, "Depth of low-mid acoustic cavity formant enhancement (400-900 Hz)"))
        .with_param(DspParamSchema::knob("high_harmonic_attenuation_db", "Upper Metallic Overtones Attenuation", -36.0, 0.0, -18.0, "dB", MacroRole::Tone, "Stopband acoustic filtering of abrasive metallic reed harmonics above 3 kHz"))
        .with_param(DspParamSchema::knob("chamber_volume_liters", "Internal Chamber Enclosure Volume", 0.5, 8.0, 2.4, "L", MacroRole::Space, "Internal physical volume of the enclosed wooden chamber"))
        .with_param(DspParamSchema::knob("reedblock_air_coupling", "Reedblock to Chamber Air Coupling", 0.1, 1.0, 0.7, "ratio", MacroRole::Punch, "Acoustic impedance matching between vibrating reed tongues and air column"))
        .with_param(DspParamSchema::knob("box_wood_resonance_q", "Cavity Air Resonance Q Factor", 1.0, 10.0, 3.5, "Q", MacroRole::Character, "Q factor of primary cavity air-box resonances"))
    );
    descriptors.insert("WoodwindToneholeGrid".to_string(), DspNodeDescriptor::new("WoodwindToneholeGrid", "Woodwind Tonehole Lattice Acoustic Scattering Junction", DspNodeCategory::FilterEq, "Acoustic waveguide tonehole lattice modeling shunt impedance discontinuities, high-pass acoustic radiation through open fingerholes, and closed hole acoustic inertance corrections")
        .with_param(DspParamSchema::choice("active_fingering_pattern", "Acoustic Woodwind Fingering State", &["All Toneholes Closed (Lowest Fundamental)", "Bottom Tonehole Open", "Forked Baroque Fingering", "Cross-Finger Woodwind Split", "All Toneholes Open (Bell Radiator)"], 0, "Discrete acoustic fingering configuration along the bore"))
        .with_param(DspParamSchema::knob("tonehole_chimney_height_mm", "Tonehole Chimney Height / Wall", 1.0, 8.0, 2.5, "mm", MacroRole::Tone, "Physical chimney height and wall thickness setting acoustic inertance"))
        .with_param(DspParamSchema::knob("tonehole_inner_radius_mm", "Tonehole Aperture Inner Radius", 1.5, 8.0, 4.5, "mm", MacroRole::Tone, "Radius of tonehole apertures governing radiation cutoff frequency"))
        .with_param(DspParamSchema::knob("radiation_loss_damping", "Open Tonehole Acoustic Radiation Damping", 0.0, 1.0, 0.45, "loss", MacroRole::Space, "Fraction of acoustic wave radiated into room vs reflected back upstream"))
        .with_param(DspParamSchema::knob("end_correction_ratio", "Aperture Acoustic End Correction", 0.2, 1.5, 0.61, "delta", MacroRole::Character, "Acoustic length end correction at open tonehole apertures"))
        .with_param(DspParamSchema::knob("bore_thermal_viscous_losses", "Bore Wall Thermal Viscous Loss", 0.01, 0.5, 0.08, "loss/m", MacroRole::Tone, "Wall friction and thermal viscous dissipation along tube walls"))
    );
    descriptors.insert("AcousticHornReflection".to_string(), DspNodeDescriptor::new("AcousticHornReflection", "Acoustic Horn Bell Impedance Matcher & Flare Radiator", DspNodeCategory::FilterEq, "Physical acoustic model of brass instrument bells, saxophone flares, and phonograph horns calculating frequency-dependent reflection coefficients, radiation resistance, and cutoff highpass behavior")
        .with_param(DspParamSchema::choice("horn_flare_geometry", "Acoustic Flare Geometry Curve", &["Exponential Acoustic Flare", "Bessel Conical Horn", "Hyperbolic Catenoidal Flare", "Tractrix Studio Horn"], 0, "Mathematical flare expansion curve governing radiation impedance"))
        .with_param(DspParamSchema::knob("bell_opening_radius_cm", "Bell Aperture Mouth Radius", 2.0, 40.0, 12.5, "cm", MacroRole::Tone, "Aperture radius at bell mouth determining low-frequency radiation cutoff"))
        .with_param(DspParamSchema::knob("horn_axial_length_cm", "Horn Axial Flare Length", 10.0, 200.0, 65.0, "cm", MacroRole::Space, "Total physical length of horn flare transition"))
        .with_param(DspParamSchema::knob("flare_cutoff_frequency_hz", "Calculated Radiation Cutoff Freq", 100.0, 1500.0, 380.0, "Hz", MacroRole::Tone, "Calculated cutoff frequency below which acoustic waves undergo total reflection"))
        .with_param(DspParamSchema::knob("radiation_directivity_factor", "High-Frequency Axial Directivity", 0.0, 1.0, 0.7, "dir", MacroRole::Character, "High-frequency acoustic beaming directivity along horn axis"))
        .with_param(DspParamSchema::knob("viscous_throat_wall_loss", "Throat Wall Viscous Dissipation", 0.01, 0.5, 0.06, "loss", MacroRole::Tone, "Acoustic wave energy dissipation inside narrow throat tube"))
    );
    descriptors.insert("SoundboardBridgeCoupler".to_string(), DspNodeDescriptor::new("SoundboardBridgeCoupler", "Spruce Soundboard Downbearing Bridge Impedance Matrix", DspNodeCategory::FilterEq, "Multi-point acoustic impedance matrix modeling the transfer of string vibrational shear and bending moments across the bridge downbearing angle into orthotropic spruce soundboard plates")
        .with_param(DspParamSchema::choice("spruce_grain_orientation", "Orthotropic Spruce Plate Grain Profile", &["Radial Quarter-Sawn Sitka", "European Alpine Val di Fiemme", "Adirondack Red Spruce", "Laminated Multi-Ply Soundboard"], 1, "Wood grain orientation and velocity of sound along grain"))
        .with_param(DspParamSchema::knob("bridge_downbearing_angle_deg", "Bridge Downbearing Break Angle", 0.5, 6.0, 2.2, "deg", MacroRole::Punch, "String break downbearing angle over bridge pins providing vertical static load"))
        .with_param(DspParamSchema::knob("soundboard_rib_stiffening", "Perpendicular Rib Stiffening Profile", 0.1, 1.0, 0.65, "stiff", MacroRole::Tone, "Perpendicular spruce ribbing profile distributing high frequencies"))
        .with_param(DspParamSchema::knob("plate_modal_density_cutoff_hz", "Plate Modal Transition Frequency", 100.0, 1200.0, 450.0, "Hz", MacroRole::Tone, "Frequency transition from discrete room-like plate modes to statistical reverberation"))
        .with_param(DspParamSchema::knob("bridge_admittance_mobility", "Bridge Mechanical Mobility Admittance", 0.01, 0.8, 0.18, "mob", MacroRole::Space, "Mechanical admittance of the bridge setting string decay rates"))
        .with_param(DspParamSchema::knob("soundboard_rim_clamping_stiffness", "Rim Perimeter Boundary Clamping", 0.1, 1.0, 0.75, "clamp", MacroRole::Character, "Acoustic boundary clamping firmness at cast-iron rim or guitar kerfing"))
    );
    descriptors.insert("SpectralTransientIsolator".to_string(), DspNodeDescriptor::new("SpectralTransientIsolator", "High-Resolution STFT Transient/Tonal Splitter & Isolator", DspNodeCategory::FilterEq, "Short-Time Fourier Transform phase-vocoder spectral decomposing audio in real time into pure non-stationary transient bursts and stationary tonal sinusoidal partials")
        .with_param(DspParamSchema::knob("transient_threshold_sensitivity", "Transient Detection Sensitivity", 0.1, 1.0, 0.55, "sens", MacroRole::Punch, "Spectral flux threshold separating sudden attack energy from sustained tones"))
        .with_param(DspParamSchema::knob("separation_smoothness_ms", "Spectral Mask Temporal Smoothing", 0.5, 50.0, 8.0, "ms", MacroRole::Space, "Temporal integration time constant preventing spectral chatter/burbling"))
        .with_param(DspParamSchema::choice("spectral_fft_resolution", "STFT Transform Window Resolution", &["256 Samples (Ultra Snappy)", "512 Samples (Standard Balance)", "1024 Samples (Sharp Tonal Focus)", "2048 Samples (Surgical Frequency)"], 1, "FFT frame length balancing temporal attack precision with spectral selectivity"))
        .with_param(DspParamSchema::knob("transient_boost_db", "Isolated Transient Channel Gain", -24.0, 18.0, 0.0, "dB", MacroRole::Punch, "Gain multiplier applied to separated transient impulse signal"))
        .with_param(DspParamSchema::knob("tonal_decay_preservation", "Tonal Resonance Sustain Preservation", 0.0, 1.0, 0.85, "pres", MacroRole::Tone, "Weighting factor prioritizing continuous musical sustain in stationary channel"))
        .with_param(DspParamSchema::toggle("audition_transient_channel_only", "Audition Separated Transient Solo", false, "Mutes tonal background to audition exclusively separated transient clicks"))
    );
    descriptors.insert("BesselLinearPhaseCrossover".to_string(), DspNodeDescriptor::new("BesselLinearPhaseCrossover", "Linear-Phase Bessel Multi-Band Crossover Splitter", DspNodeCategory::FilterEq, "Multi-way linear-phase audio crossover filter dividing full-spectrum signals into isolated sub-bands with maximally flat group delay, zero phase distortion, and transparent transient response")
        .with_param(DspParamSchema::knob("crossover_split_frequency_hz", "Band Crossover Split Frequency", 60.0, 8000.0, 450.0, "Hz", MacroRole::Tone, "Transition frequency between lower and upper split bands"))
        .with_param(DspParamSchema::choice("fir_tap_order", "FIR Impulse Kernel Resolution", &["512 Taps (Low Latency Tracking)", "1024 Taps (Studio Production)", "2048 Taps (Mastering Pristine)", "4096 Taps (Ultra Steep)"], 1, "Number of FIR taps governing filter transition steepness and frequency precision"))
        .with_param(DspParamSchema::knob("stopband_attenuation_db", "Stopband Rejection Attenuation", -96.0, -24.0, -60.0, "dB", MacroRole::Tone, "Attenuation depth applied to out-of-band spectral energy"))
        .with_param(DspParamSchema::knob("sub_band_phase_alignment_deg", "Crossover Band Phase Alignment", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Micro-phase adjustment between separated crossover outputs"))
        .with_param(DspParamSchema::knob("transient_smear_damping", "Pre-Ringing Smear Damping Factor", 0.0, 1.0, 0.7, "damp", MacroRole::Punch, "Tapering window minimizing acausal pre-ringing artifacts on percussive hits"))
        .with_param(DspParamSchema::toggle("enable_band_phase_inversion", "Invert Split Band Polarity", false, "Inverts polarity of upper split band by 180 degrees"))
    );
    descriptors.insert("DynamicPhaseRotator".to_string(), DspNodeDescriptor::new("DynamicPhaseRotator", "Vocal Crest-Factor Phase Rotation & Asymmetry Optimizer", DspNodeCategory::FilterEq, "Cascaded allpass phase rotation network designed to disperse asymmetric vocal and brass waveforms, centering peak positive/negative excursions to recover up to 4 dB of pristine master headroom")
        .with_param(DspParamSchema::knob("center_rotation_frequency_hz", "Phase Rotation Center Frequency", 50.0, 1000.0, 160.0, "Hz", MacroRole::Tone, "Center frequency of allpass cascade targeting fundamental voice pitch"))
        .with_param(DspParamSchema::choice("allpass_cascade_stages", "Allpass Filter Cascaded Stages", &["2-Stage Gentle Rotation", "4-Stage Standard Disperser", "8-Stage Symmetrical Center", "16-Stage Full Dispersion"], 1, "Number of series allpass filter stages rotating harmonic phase angles"))
        .with_param(DspParamSchema::knob("phase_angle_shift_deg", "Manual Phase Shift Angle", 0.0, 180.0, 90.0, "deg", MacroRole::Character, "Base phase shift angle rotating positive vs negative peak excursions"))
        .with_param(DspParamSchema::toggle("dynamic_pitch_tracking_lock", "Pitch-Tracked Center Frequency Lock", true, "Automatically tunes allpass center frequency to detected fundamental voice pitch"))
        .with_param(DspParamSchema::knob("highpass_rumble_cutoff_hz", "Sub-Sonic DC Rumble Highpass Cutoff", 10.0, 80.0, 30.0, "Hz", MacroRole::Tone, "Steep highpass filter eliminating sub-bass asymmetry and microphone thumps"))
        .with_param(DspParamSchema::knob("peak_crest_factor_reduction_db", "Target Crest Factor Headroom Gain", 0.5, 6.0, 2.5, "dB", MacroRole::Punch, "Effective peak headroom recovered by symmetrizing waveform excursions"))
    );
    descriptors.insert("ResonantCombLatticeFilter".to_string(), DspNodeDescriptor::new("ResonantCombLatticeFilter", "Multi-Tap Resonant Comb Lattice & Allpass Dispersion Filter", DspNodeCategory::FilterEq, "Interconnected feedback and feedforward comb filter lattice incorporating embedded allpass delay dispersion to generate metallic, flanged, and acoustic resonator timbers")
        .with_param(DspParamSchema::knob("fundamental_comb_tuning_hz", "Lattice Fundamental Tuning", 20.0, 2000.0, 146.83, "Hz", MacroRole::Tone, "Base delay period tuning resonant comb harmonic pitch series"))
        .with_param(DspParamSchema::knob("feedback_comb_gain", "Lattice Feedback Gain", -0.99, 0.99, 0.88, "fb", MacroRole::Punch, "Recirculating feedback energy governing resonance decay ringing length"))
        .with_param(DspParamSchema::choice("allpass_dispersion_stages", "Allpass Delay Dispersion Stages", &["1 Stage (Subtle)", "2 Stages (Metallic)", "4 Stages (Phased Ring)", "8 Stages (Acoustic Dispersion)"], 1, "Cascaded allpass phases spreading high frequencies apart from low fundamentals"))
        .with_param(DspParamSchema::knob("damping_high_frequency_hz", "Internal High-Cut Loss Damping", 500.0, 20000.0, 6500.0, "Hz", MacroRole::Space, "Low-pass damping within recirculation loop simulating air viscous loss"))
        .with_param(DspParamSchema::knob("stereo_comb_cross_coupling", "Stereo Channel Cross-Coupling", 0.0, 1.0, 0.35, "cross", MacroRole::Space, "Inter-channel feedback exchange yielding wide spatial diffuse resonance"))
        .with_param(DspParamSchema::knob("feedforward_ratio", "Feedforward Direct Interference", -1.0, 1.0, 0.5, "ff", MacroRole::Character, "Feedforward cancellation factor sculpting comb filter notch depths"))
    );
    descriptors.insert("LinearPhaseMultibandSplitter".to_string(), DspNodeDescriptor::new("LinearPhaseMultibandSplitter", "5-Band Zero-Phase FIR Mastering Crossover Splitter", DspNodeCategory::FilterEq, "High-precision 5-band linear-phase FIR crossover network providing perfect phase coherence and transient preservation for multi-band dynamics and processing chains")
        .with_param(DspParamSchema::knob("crossover_1_sub_bass_hz", "Band 1/2 Sub-Bass Split Frequency", 40.0, 200.0, 90.0, "Hz", MacroRole::Punch, "Split frequency separating Sub Bass (Band 1) from Low Mids (Band 2)"))
        .with_param(DspParamSchema::knob("crossover_2_low_mid_hz", "Band 2/3 Low-Mid Split Frequency", 150.0, 800.0, 350.0, "Hz", MacroRole::Tone, "Split frequency separating Low Mids (Band 2) from Body Mids (Band 3)"))
        .with_param(DspParamSchema::knob("crossover_3_mid_high_hz", "Band 3/4 Mid-High Split Frequency", 800.0, 4000.0, 2400.0, "Hz", MacroRole::Tone, "Split frequency separating Body Mids (Band 3) from High Mids (Band 4)"))
        .with_param(DspParamSchema::knob("crossover_4_air_band_hz", "Band 4/5 High Air Split Frequency", 4000.0, 16000.0, 8500.0, "Hz", MacroRole::Character, "Split frequency separating High Mids (Band 4) from Ultra Air (Band 5)"))
        .with_param(DspParamSchema::choice("fir_filter_taps_count", "Linear-Phase FIR Filter Kernel Length", &["256 Taps (Low Latency)", "512 Taps (Standard Studio)", "1024 Taps (High Precision)", "2048 Taps (Mastering Lab)"], 2, "Window length of FIR convolution kernel governing frequency resolution"))
        .with_param(DspParamSchema::choice("solo_active_band_channel", "Band Monitoring & Audition Solo", &["All Bands (Summed)", "Solo Band 1 (Sub)", "Solo Band 2 (Low Mid)", "Solo Band 3 (Mid)", "Solo Band 4 (High Mid)", "Solo Band 5 (Air)"], 0, "Selects a single isolated frequency band for surgical monitoring"))
    );
    descriptors.insert("BilinearDiodeLadderFilter".to_string(), DspNodeDescriptor::new("BilinearDiodeLadderFilter", "4-Pole Bilinear Transform Non-Linear Diode Ladder Filter", DspNodeCategory::FilterEq, "Discrete-time zero-delay feedback (ZDF) diode ladder filter emulating TB-303 / EMS VCS3 diode clipper matrix with temperature drift and voltage sag")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Filter Cutoff Frequency", 20.0, 20000.0, 850.0, "Hz", MacroRole::Tone, "Resonant filter cutoff frequency controlling low-pass edge"))
        .with_param(DspParamSchema::knob("resonance_amount", "Feedback Resonance Intensity", 0.0, 1.0, 0.65, "%", MacroRole::Character, "Self-oscillation feedback intensity with automatic non-linear soft limiting"))
        .with_param(DspParamSchema::knob("diode_thermal_voltage_mv", "Diode Junction Thermal Voltage", 15.0, 45.0, 26.0, "mV", MacroRole::Tone, "Semiconductor junction thermal voltage setting nonlinear saturation threshold"))
        .with_param(DspParamSchema::knob("diode_saturation_drive_db", "Diode Ladder Input Overdrive", 0.0, 24.0, 4.5, "dB", MacroRole::Punch, "Harmonic overdrive pushing diode ladder stages into warm asymmetry"))
        .with_param(DspParamSchema::choice("slope_topology_mode", "Ladder Pole Configuration", &["4-Pole 24dB/oct Diode Ladder", "3-Pole 18dB/oct Acid Cut", "2-Pole 12dB/oct Resonant", "Asymmetric Saturated Ladder"], 0, "Filter slope attenuation profile and circuit pole selection"))
        .with_param(DspParamSchema::toggle("enable_zdf_anti_cramping", "Zero-Delay Feedback Anti-Cramping", true, "Preserves analog resonance response up to Nyquist frequency without frequency cramping"))
    );
    descriptors.insert("PiezoelectricCrystalFilter".to_string(), DspNodeDescriptor::new("PiezoelectricCrystalFilter", "High-Q Quartz Crystal Lattice Bandpass Filter", DspNodeCategory::FilterEq, "Piezoelectric quartz crystal resonator simulation modeling ultra-sharp motional inductance, parallel capacitance, and crystal shear resonance")
        .with_param(DspParamSchema::knob("crystal_center_freq_hz", "Motional Series Resonant Frequency", 100.0, 18000.0, 1000.0, "Hz", MacroRole::Tone, "Series resonance frequency determined by physical quartz cut thickness"))
        .with_param(DspParamSchema::knob("motional_q_factor", "Quartz Crystal Mechanical Quality Factor Q", 100.0, 50000.0, 5000.0, "Q", MacroRole::Tone, "Ultra-narrow resonance Q producing ringing crystalline bell tones"))
        .with_param(DspParamSchema::knob("parallel_resonance_offset_cents", "Anti-Resonant Pole Frequency Offset", 5.0, 200.0, 35.0, "cents", MacroRole::Character, "Offset frequency of parallel anti-resonance zero notch"))
        .with_param(DspParamSchema::knob("crystal_electrode_capacitance_pf", "Holder Electrostatic Shunt Capacitance", 1.0, 50.0, 7.5, "pF", MacroRole::Tone, "Static shunt capacitance of gold/silver electrode plating"))
        .with_param(DspParamSchema::knob("drive_level_nonlinearity", "High-Drive Crystal Nonlinear Elasticity", 0.0, 1.0, 0.25, "%", MacroRole::Punch, "Acoustic saturation and duffing spring nonlinearity at high drive levels"))
        .with_param(DspParamSchema::knob("temperature_drift_ppm", "Ambient Temperature Frequency Drift", -50.0, 50.0, 0.0, "ppm", MacroRole::Space, "Temperature-induced resonant frequency drift in parts per million"))
    );
    descriptors.insert("PhaseAlignedStereoCrossover".to_string(), DspNodeDescriptor::new("PhaseAlignedStereoCrossover", "All-Pass Phase-Compensated Linear Crossover Matrix", DspNodeCategory::FilterEq, "4-way splitting network with complementary Linkwitz-Riley and all-pass delay compensation ensuring perfect phase alignment and 0 dB summing ripple")
        .with_param(DspParamSchema::knob("low_crossover_freq_hz", "Sub/Low Splitting Crossover Frequency", 40.0, 250.0, 90.0, "Hz", MacroRole::Tone, "Split point dividing sub-bass from low-mid frequency bands"))
        .with_param(DspParamSchema::knob("mid_crossover_freq_hz", "Low-Mid/High-Mid Crossover Frequency", 250.0, 2500.0, 800.0, "Hz", MacroRole::Tone, "Split point dividing low-mid from high-mid presence bands"))
        .with_param(DspParamSchema::knob("high_crossover_freq_hz", "High-Mid/Treble Crossover Frequency", 2500.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Split point dividing high-mid from high-frequency sparkle band"))
        .with_param(DspParamSchema::choice("filter_slope_steepness", "Crossover Filter Slope Order", &["12 dB/oct (Linkwitz-Riley 2nd)", "24 dB/oct (Linkwitz-Riley 4th)", "48 dB/oct (Linkwitz-Riley 8th)"], 1, "Filter order determining steepness of transition bands between splits"))
        .with_param(DspParamSchema::knob("delay_phase_fine_trim_us", "Sub-Sample Inter-Band Phase Alignment Trim", -500.0, 500.0, 0.0, "us", MacroRole::Punch, "Microsecond fine phase alignment trim compensating for transducer latency"))
        .with_param(DspParamSchema::toggle("enable_allpass_phase_matching", "Cascaded All-Pass Group Delay Equalization", true, "Inserts complementary all-pass networks to equalize group delay across bands"))
    );
    descriptors.insert("BilinearStateVariableMorphFilter".to_string(), DspNodeDescriptor::new("BilinearStateVariableMorphFilter", "Zero-Delay Feedback State-Variable Morphing Filter", DspNodeCategory::FilterEq, "Trapezoidal integrated zero-delay feedback state variable filter (SVF) with continuous seamless morphing from Lowpass through Bandpass to Highpass and Notch")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Filter Corner Cutoff Frequency", 10.0, 22000.0, 1500.0, "Hz", MacroRole::Tone, "Filter cutoff corner frequency with exact bilinear analog frequency mapping"))
        .with_param(DspParamSchema::knob("resonance_q_factor", "Resonance Peak Quality Factor (Q)", 0.5, 30.0, 2.8, "Q", MacroRole::Character, "Filter resonance Q with internal non-linear soft diode clipping saturation"))
        .with_param(DspParamSchema::choice("filter_morph_mode", "State Variable Output Mode Tap", &["Lowpass (12 dB/oct)", "Bandpass Peak", "Highpass (12 dB/oct)", "Notch / Band Reject", "Allpass Phase Shift"], 0, "Continuous state variable output tap configuration"))
        .with_param(DspParamSchema::knob("morph_crossfade_ratio", "Continuous Morph Interpolation Ratio", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Continuous interpolation between filter response types"))
        .with_param(DspParamSchema::knob("drive_saturation_gain", "Nonlinear Diode Feedback Drive", 1.0, 10.0, 1.5, "drv", MacroRole::Punch, "Input drive pushing filter feedback path into musical overdrive"))
        .with_param(DspParamSchema::knob("keytrack_pitch_scaling", "Keyboard Pitch Tracking Amount", 0.0, 2.0, 1.0, "amt", MacroRole::Tone, "Keytracking modulation scaling cutoff frequency with MIDI note pitch"))
    );
    descriptors.insert("HarmonicResonanceCombBank".to_string(), DspNodeDescriptor::new("HarmonicResonanceCombBank", "64-Tap Tuned Harmonic Comb Filter Resonator Bank", DspNodeCategory::FilterEq, "Polyphonic comb filter array with 64 delay lines tuned to natural harmonic overtone series, frequency-dependent damping, and bipolar feedback resonance")
        .with_param(DspParamSchema::knob("fundamental_frequency_hz", "Base Harmonic Series Pitch", 20.0, 2000.0, 110.0, "Hz", MacroRole::Tone, "Base pitch defining harmonic series intervals of comb filter teeth"))
        .with_param(DspParamSchema::knob("comb_decay_time_seconds", "Resonant Ringing Decay Time", 0.05, 10.0, 1.5, "s", MacroRole::Space, "Resonant ringing duration across harmonic comb teeth"))
        .with_param(DspParamSchema::knob("high_frequency_damping_hz", "Feedback High Frequency Damping", 500.0, 20000.0, 6000.0, "Hz", MacroRole::Tone, "Lowpass filter in feedback loop dampening upper harmonic resonance"))
        .with_param(DspParamSchema::knob("odd_even_harmonic_balance", "Odd vs Even Harmonic Tooth Balance", -1.0, 1.0, 0.0, "bal", MacroRole::Character, "Emphasizes odd vs even harmonic comb teeth (-1.0 odd, +1.0 even)"))
        .with_param(DspParamSchema::knob("stereo_comb_detune_cents", "Stereo Comb Spread Detuning", 0.0, 50.0, 8.0, "cents", MacroRole::Space, "Slight pitch offset between left and right comb channels for stereo width"))
        .with_param(DspParamSchema::toggle("bipolar_comb_phase_invert", "Invert Feedback Polarity (Peak/Notch)", false, "Inverts comb feedback polarity shifting peaks to harmonic notches"))
    );
    descriptors.insert("LinearPhaseMultibandIsolator".to_string(), DspNodeDescriptor::new("LinearPhaseMultibandIsolator", "5-Band FIR Linear-Phase Frequency Isolator & Crossover", DspNodeCategory::FilterEq, "Zero-phase frequency isolating crossover with steep Kaiser-window FIR filter banks and individual band mute/solo/gain routing for surgical mastering")
        .with_param(DspParamSchema::knob("crossover_low_hz", "Sub / Bass Crossover Frequency", 40.0, 400.0, 120.0, "Hz", MacroRole::Tone, "Sub-bass to low-mid transition crossover frequency"))
        .with_param(DspParamSchema::knob("crossover_low_mid_hz", "Low-Mid / Mid Crossover Frequency", 200.0, 2000.0, 600.0, "Hz", MacroRole::Tone, "Low-mid to body resonance crossover frequency"))
        .with_param(DspParamSchema::knob("crossover_high_mid_hz", "Mid / High-Mid Crossover Frequency", 1000.0, 8000.0, 3200.0, "Hz", MacroRole::Tone, "Midrange vocal clarity to presence crossover frequency"))
        .with_param(DspParamSchema::knob("crossover_high_hz", "Presence / Air Crossover Frequency", 4000.0, 16000.0, 9000.0, "Hz", MacroRole::Tone, "Presence to ultra-high air band crossover frequency"))
        .with_param(DspParamSchema::choice("fir_filter_order_taps", "FIR Filter Kernel Length (Phase Fidelity)", &["128 Taps (Low Latency)", "512 Taps (Studio FIR)", "2048 Taps (Mastering Pristine)"], 1, "Length of linear-phase FIR impulse response kernel"))
        .with_param(DspParamSchema::choice("solo_band_audition_mode", "Band Audition & Solo Isolation", &["All Bands Summed", "Low Band Solo", "Low-Mid Solo", "High-Mid Solo", "High Band Solo"], 0, "Isolates and solos individual frequency bands for surgical monitoring"))
        .with_param(DspParamSchema::toggle("enable_pre_ringing_suppression", "Psychoacoustic Pre-Ringing Suppression", true, "Attenuates psychoacoustic pre-ringing artifacts inherent to steep linear-phase filters"))
    );
    descriptors.insert("MicrotonalSpectralResonatorBank".to_string(), DspNodeDescriptor::new("MicrotonalSpectralResonatorBank", "MTS-ESP Microtonal Spectral Resonator Filter Bank", DspNodeCategory::FilterEq, "Polyphonic bank of 32 ultra-narrow resonant bandpass filters tuned to non-standard microtonal tuning grids and historical temperaments")
        .with_param(DspParamSchema::knob("fundamental_anchor_pitch_hz", "Concert Root Tuning Pitch (A4)", 20.0, 1000.0, 220.0, "Hz", MacroRole::Tone, "Anchor concert frequency defining base fundamental of resonator grid"))
        .with_param(DspParamSchema::knob("resonator_q_bandwidth", "Resonant Tooth Quality Factor (Q)", 2.0, 200.0, 45.0, "Q", MacroRole::Character, "Resonance sharpness narrowing frequency teeth into ringing bell-like poles"))
        .with_param(DspParamSchema::knob("resonance_ringdown_decay_seconds", "Harmonic Resonator Ringdown Time", 0.1, 15.0, 2.5, "s", MacroRole::Space, "Sustain duration of self-resonating filter teeth after input excitation"))
        .with_param(DspParamSchema::choice("tuning_scale_selection", "Microtonal Scale & Temperament", &["12-EDO Equal Temperament", "19-EDO Equal Temperament", "31-EDO Huygens", "Bohlen-Pierce Tritave", "Just Intonation (5-Limit)", "Pythagorean Tuning"], 0, "Selects mathematical microtonal temperament system"))
        .with_param(DspParamSchema::knob("excitation_input_gain_db", "Excitation Signal Drive Gain", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Input gain driving resonance bank into excitation ringing"))
        .with_param(DspParamSchema::toggle("enable_mts_esp_realtime_tuning_sync", "MTS-ESP Dynamic Tuning Broadcast Sync", true, "Receives dynamic microtonal master tuning updates via MTS-ESP protocol"))
    );
    descriptors.insert("BesselFilterArrayEqualizer".to_string(), DspNodeDescriptor::new("BesselFilterArrayEqualizer", "Maximally Flat Linear Delay Bessel Filter Array EQ", DspNodeCategory::FilterEq, "Linear-delay multi-band parametric equalizer employing analog-matched Bessel polynomial filters for zero transient ringing and perfect pulse fidelity")
        .with_param(DspParamSchema::knob("low_shelf_cutoff_frequency_hz", "Bessel Low-Shelf Corner Frequency", 20.0, 500.0, 80.0, "Hz", MacroRole::Tone, "Corner frequency of maximally flat group delay low-shelf filter"))
        .with_param(DspParamSchema::knob("low_shelf_gain_boost_db", "Bessel Low-Shelf Gain Adjustment", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Gain boost or cut applied with constant group delay to low frequencies"))
        .with_param(DspParamSchema::knob("parametric_mid_frequency_hz", "Bessel Mid-Band Center Frequency", 200.0, 8000.0, 1200.0, "Hz", MacroRole::Tone, "Center tuning frequency of parametric bell resonance band"))
        .with_param(DspParamSchema::knob("parametric_mid_gain_db", "Bessel Mid-Band Gain Adjustment", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Mid-band boost/cut preserving natural phase and transient pulse attack"))
        .with_param(DspParamSchema::knob("high_shelf_cutoff_frequency_hz", "Bessel High-Shelf Corner Frequency", 2000.0, 20000.0, 8000.0, "Hz", MacroRole::Tone, "Corner frequency of high-shelf filter sculpting sheen without smearing transients"))
        .with_param(DspParamSchema::knob("high_shelf_gain_boost_db", "Bessel High-Shelf Gain Adjustment", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High-shelf gain boost or attenuation"))
        .with_param(DspParamSchema::choice("filter_order_poles", "Bessel Polynomial Order (Slope)", &["2-Pole (12 dB/oct)", "4-Pole (24 dB/oct)", "6-Pole (36 dB/oct)"], 1, "Order of Thomson/Bessel polynomial filter kernel"))
        .with_param(DspParamSchema::toggle("enable_analog_phase_matching", "Analog Group Delay Symmetrical Match", true, "Ensures perfectly linear group delay across entire passband eliminating phase dispersion"))
    );
    descriptors.insert("SallenKeyResonantFilter".to_string(), DspNodeDescriptor::new("SallenKeyResonantFilter", "Discrete Sallen-Key 12 dB/Oct Dual-Pole Resonant Filter", DspNodeCategory::FilterEq, "Op-amp active filter circuit modeling finite bandwidth, input clipping, non-linear capacitor ESR, and warm self-oscillating resonance peaks")
        .with_param(DspParamSchema::knob("filter_cutoff_frequency_hz", "Sallen-Key Cutoff Corner Frequency", 15.0, 22000.0, 1200.0, "Hz", MacroRole::Tone, "Corner cutoff frequency of 2-pole active RC filter network"))
        .with_param(DspParamSchema::knob("resonance_q_factor", "Filter Resonance Quality Factor (Q)", 0.5, 25.0, 2.5, "Q", MacroRole::Character, "Feedback loop damping coefficient controlling resonance sharpness and oscillation"))
        .with_param(DspParamSchema::choice("filter_response_mode", "Active Filter Pole Configuration", &["Low-Pass (12 dB/oct)", "High-Pass (12 dB/oct)", "Band-Pass 6 dB"], 0, "Selects low-pass, high-pass, or band-pass active RC topology"))
        .with_param(DspParamSchema::knob("op_amp_saturation_drive", "Op-Amp Slew & Voltage Clipping Drive", 1.0, 10.0, 1.2, "drv", MacroRole::Punch, "Drives op-amp stage into non-linear musical voltage rail clipping"))
        .with_param(DspParamSchema::knob("filter_key_tracking_amount", "MIDI Note Pitch Cutoff Tracking", 0.0, 2.0, 1.0, "ratio", MacroRole::Tone, "Keyboard note pitch scaling shifting cutoff proportionally to played notes"))
        .with_param(DspParamSchema::toggle("enable_soft_diode_feedback_clip", "Diode Clipper Feedback Stabilization", true, "Silicon diode clipping in feedback loop preventing uncontrollable oscillation runaway"))
    );
    descriptors.insert("StateVariableMorphLadderFilter".to_string(), DspNodeDescriptor::new("StateVariableMorphLadderFilter", "Cascaded Transistor Ladder & 4-Pole State-Variable Hybrid Morphing Filter", DspNodeCategory::FilterEq, "Hybrid continuous-morph filter topology interpolating smoothly across lowpass, bandpass, highpass, and notch modes with transistor ladder drive saturation")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Master Filter Cutoff Frequency", 10.0, 22000.0, 1200.0, "Hz", MacroRole::Tone, "Corner frequency of the hybrid state-variable ladder filter"))
        .with_param(DspParamSchema::knob("resonance_q_feedback", "Resonant Peak Q Feedback Ratio", 0.5, 25.0, 2.0, "Q", MacroRole::Tone, "Resonance intensity up to self-oscillating clean sinusoidal feedback"))
        .with_param(DspParamSchema::knob("filter_topology_morph_ratio", "Continuous Topology Mode Morph", 0.0, 3.0, 0.0, "mode", MacroRole::Tone, "Continuous morphing parameter: 0=Lowpass, 1=Bandpass, 2=Highpass, 3=Notch"))
        .with_param(DspParamSchema::knob("transistor_saturation_drive", "Transistor Ladder Input Overdrive", 0.0, 1.0, 0.25, "drive", MacroRole::Character, "Non-linear hyperbolic tangent saturation in feedback ladder loop"))
        .with_param(DspParamSchema::knob("self_oscillation_damping", "Self-Oscillation Peak Limiter Damping", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Soft-clamping damping threshold stabilizing wild self-resonant oscillation"))
        .with_param(DspParamSchema::toggle("enable_zero_delay_feedback", "Zero-Delay Feedback (ZDF) Trapezoidal Solver", true, "Solves feedback delay algebraically for pristine analog frequency and phase response"))
    );
    descriptors.insert("SpectralPeakTrackingFilter".to_string(), DspNodeDescriptor::new("SpectralPeakTrackingFilter", "Autonomous Fundamental & Spectral Peak Tracking Adaptive Dynamic Filter", DspNodeCategory::FilterEq, "Real-time FFT pitch-follower steering steep second-order biquad filter poles directly onto incoming melodic pitch fundamentals or harmonics")
        .with_param(DspParamSchema::knob("pitch_tracking_sensitivity", "Pitch Tracker Detector Sensitivity", 0.1, 1.0, 0.85, "sens", MacroRole::Character, "Confidence threshold required for pitch estimator to update center frequency"))
        .with_param(DspParamSchema::knob("tracking_bandwidth_q", "Bandwidth Resonance Q Factor", 1.0, 30.0, 8.0, "Q", MacroRole::Tone, "Sharpness of tracked resonant filter bell centered on detected pitch"))
        .with_param(DspParamSchema::knob("peak_boost_cut_gain_db", "Tracked Frequency Boost / Cut Gain", -18.0, 18.0, 4.0, "dB", MacroRole::Tone, "Gain applied dynamically at detected fundamental frequency"))
        .with_param(DspParamSchema::knob("tracking_glide_speed_ms", "Pitch Tracking Slewing Time", 1.0, 200.0, 25.0, "ms", MacroRole::Punch, "Portamento slewing speed smoothing jumps between tracked note transitions"))
        .with_param(DspParamSchema::choice("harmonic_tracking_order", "Tracked Harmonic Overtone Target", &["1st (Fundamental)", "2nd (Octave)", "3rd (Fifth)", "4th (Double Octave)"], 0, "Selects which harmonic overtone of detected note the filter locks onto"))
        .with_param(DspParamSchema::toggle("freeze_tracked_frequency", "Freeze Current Center Frequency", false, "Locks current tracked center frequency in place, disabling adaptive tracking updates"))
    );
    descriptors.insert("ResonantDiodeLadderFilter".to_string(), DspNodeDescriptor::new("ResonantDiodeLadderFilter", "TB-303 Style 18/24 dB Diode Ladder Lowpass Filter with Saturation", DspNodeCategory::FilterEq, "Discrete transistor-diode ladder filter with non-linear diode conductance curve, squelchy high-Q resonance character, and bass bleed compensation")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Diode Ladder Cutoff Frequency", 20.0, 18000.0, 1500.0, "Hz", MacroRole::Tone, "Master frequency corner of 18/24 dB per octave diode filter"))
        .with_param(DspParamSchema::knob("resonance_feedback_q", "Acid Squelch Resonance Q Peak", 0.5, 30.0, 4.5, "Q", MacroRole::Tone, "Drives diode ladder into trademark self-oscillating acid resonance"))
        .with_param(DspParamSchema::knob("diode_conduction_saturation", "Diode Conduction Soft Saturation", 0.0, 1.0, 0.45, "%", MacroRole::Character, "Emulates dynamic voltage drops across four series diode stages under signal drive"))
        .with_param(DspParamSchema::knob("highpass_bass_bleed_compensation", "Resonance Low-End Pass-Through Bleed", 0.0, 1.0, 0.3, "%", MacroRole::Punch, "Compensates for inherent low-end drop when resonance is cranked high"))
        .with_param(DspParamSchema::knob("envelope_modulation_depth", "Filter Cutoff Envelope Modulation", -1.0, 1.0, 0.6, "amt", MacroRole::Tone, "Depth of external ADSR or accent envelope applied to filter cutoff frequency"))
        .with_param(DspParamSchema::toggle("enable_exponential_thermal_drift", "Analog Thermal Component Drift", true, "Simulates slight temperature-dependent transistor offset variations for vintage vibe"))
    );
    descriptors.insert("MultibandMidSideEqualizer".to_string(), DspNodeDescriptor::new("MultibandMidSideEqualizer", "5-Band Precision Mastering Mid/Side Minimum Phase EQ", DspNodeCategory::FilterEq, "Linear-phase and minimum-phase switchable 5-band equalizer processing Mid (sum) and Side (difference) channels independently with dedicated mono bass guard")
        .with_param(DspParamSchema::knob("mid_low_shelf_gain_db", "Mid Channel Sub Bass Shelf Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Punch, "Low shelf gain applied exclusively to mono center phantom image"))
        .with_param(DspParamSchema::knob("side_high_shelf_gain_db", "Side Channel Air Band Shelf Gain", -12.0, 12.0, 1.5, "dB", MacroRole::Space, "High-frequency air shelf boosting wide ambient stereo side information"))
        .with_param(DspParamSchema::knob("mid_presence_bell_hz", "Mid Channel Vocal Presence Frequency", 1000.0, 8000.0, 3200.0, "Hz", MacroRole::Tone, "Parametric bell frequency targeting center vocal and snare drum clarity"))
        .with_param(DspParamSchema::knob("side_presence_bell_hz", "Side Channel Width Focus Frequency", 1000.0, 8000.0, 5500.0, "Hz", MacroRole::Space, "Parametric bell frequency shaping ambient stereo width and guitars"))
        .with_param(DspParamSchema::knob("mono_low_cutoff_hz", "Mono Low Bass Guard Cutoff Frequency", 20.0, 200.0, 75.0, "Hz", MacroRole::Punch, "High-pass filter on Side channel guaranteeing clean centered sub-bass"))
        .with_param(DspParamSchema::toggle("solo_side_channel_monitoring", "Solo Side Channel Stereo Difference", false, "Solos side information to audition stereo reverb, panning, and micro-dynamics"))
    );
    descriptors.insert("BiquadFormantMorphFilter".to_string(), DspNodeDescriptor::new("BiquadFormantMorphFilter", "3-Vowel Morphing Triple-Biquad Vocal Resonator", DspNodeCategory::FilterEq, "Tri-formant vocal tract filter utilizing three parallel second-order resonant bandpass filters morphing dynamically between vowel resonances (A-E-I-O-U) across an interactive 2D plane")
        .with_param(DspParamSchema::knob("vowel_morph_xy_coordinate", "2D Formant Vowel Coordinate (A-E-I-O-U)", 0.0, 4.0, 1.2, "pos", MacroRole::Tone, "Navigates continuous vowel space interpolating formant frequencies F1, F2, and F3"))
        .with_param(DspParamSchema::knob("formant_q_bandwidth_focus", "Formant Peak Resonant Sharpness (Q)", 2.0, 35.0, 12.0, "Q", MacroRole::Tone, "Resonance factor of biquad filters accentuating vocal articulation"))
        .with_param(DspParamSchema::knob("nasal_cavity_anti_resonance_db", "Nasal Cavity Anti-Resonant Notch Depth", -24.0, 0.0, -6.0, "dB", MacroRole::Character, "Introduces realistic nasal anti-resonance dip between first and second formants"))
        .with_param(DspParamSchema::knob("throat_vocal_tract_length_cm", "Acoustic Vocal Tract Length", 10.0, 24.0, 17.5, "cm", MacroRole::Tone, "Length of vocal tract scaling all resonant formants up or down (male/female/child)"))
        .with_param(DspParamSchema::knob("morph_transition_slew_ms", "Vowel Transition Slewing Speed", 5.0, 300.0, 45.0, "ms", MacroRole::Punch, "Smoothing time when morphing rapidly between phonetic vowel coordinates"))
        .with_param(DspParamSchema::toggle("enable_parallel_formant_routing", "Parallel vs Series Filter Architecture", true, "Toggles between parallel vowel summing and cascaded series filtering"))
    );
    descriptors.insert("LinearPhaseTiltEqualizer".to_string(), DspNodeDescriptor::new("LinearPhaseTiltEqualizer", "Zero-Phase 180-Degree Pivot Spectral Balance & Sub Sentry", DspNodeCategory::FilterEq, "Zero-latency linear-phase FIR tilt equalizer tilting overall audio spectrum around an adjustable pivot frequency without introducing phase distortion, coupled to steep sub-bass cleanup")
        .with_param(DspParamSchema::knob("tilt_pivot_frequency_hz", "Spectral Tilt Center Pivot Frequency", 200.0, 3000.0, 1000.0, "Hz", MacroRole::Tone, "Anchor frequency around which bass and treble tilt inversely"))
        .with_param(DspParamSchema::knob("spectral_tilt_slope_db_oct", "Spectral Tilt Slope Angle", -6.0, 6.0, 1.5, "dB/oct", MacroRole::Tone, "Gain slope boosting highs while cutting lows, or vice versa"))
        .with_param(DspParamSchema::knob("sub_acoustic_infrasound_cutoff_hz", "Infrasonic Sub-Bass High-Pass Cutoff", 15.0, 60.0, 25.0, "Hz", MacroRole::Punch, "Steep 24 dB/oct high-pass filter eradicating non-audible subwoofer rumble"))
        .with_param(DspParamSchema::knob("air_band_lift_gain_db", "Ultra-High Air Band Radiance Shelf", -6.0, 12.0, 2.0, "dB", MacroRole::Space, "Smooth high-shelf boost above 16 kHz delivering open acoustic gloss"))
        .with_param(DspParamSchema::choice("fir_filter_order_taps", "Linear-Phase FIR Convolution Length", &["1024 Taps (Low Latency)", "2048 Taps (Balanced)", "4096 Taps (Ultra Precision)"], 1, "Number of FIR taps ensuring steep phase-neutral frequency response"))
        .with_param(DspParamSchema::toggle("bypass_phase_linearization", "Minimum Phase Low-Latency Mode", false, "Switches from linear-phase FIR to minimum-phase IIR for zero-latency tracking"))
    );
    descriptors.insert("ResonantMoogLadderDualCutoff".to_string(), DspNodeDescriptor::new("ResonantMoogLadderDualCutoff", "Dual-Slope Transistor Ladder Filter with Asymmetric Diode Saturation", DspNodeCategory::FilterEq, "Authentic analog four-stage transistor ladder lowpass filter with non-linear hyperbolic tangent saturation, switchable 12/24 dB slope, and resonance bass-loss compensation")
        .with_param(DspParamSchema::knob("ladder_cutoff_frequency_hz", "Transistor Ladder Cutoff Frequency", 15.0, 22000.0, 850.0, "Hz", MacroRole::Tone, "Master cutoff frequency modulating base of transistor ladder rungs"))
        .with_param(DspParamSchema::knob("resonance_feedback_intensity", "Ladder Self-Oscillation Resonance", 0.0, 4.0, 1.8, "res", MacroRole::Tone, "Feedback gain pushing filter into pure sine wave self-oscillation"))
        .with_param(DspParamSchema::knob("passband_gain_loss_compensation", "Bass Passband Drop Compensation", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Restores low frequencies naturally attenuated by high ladder resonance"))
        .with_param(DspParamSchema::knob("ladder_drive_saturation_db", "Input Differential Amplifier Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Character, "Overdrives differential pair into warm asymmetric transistor clipping"))
        .with_param(DspParamSchema::choice("slope_steepness_db_oct", "Ladder Slope Filter Order", &["12 dB/Oct (2-Pole Warm)", "18 dB/Oct (3-Pole Acid)", "24 dB/Oct (4-Pole Classic)"], 2, "Selects number of active ladder RC poles for cutoff sharpness"))
        .with_param(DspParamSchema::toggle("enable_self_oscillation_limiter", "Soft-Saturating Feedback Peak Limiter", true, "Prevents runaway explosive feedback amplitude when filter self-oscillates"))
    );
    descriptors.insert("MetamaterialNegativeIndexFilter".to_string(), DspNodeDescriptor::new("MetamaterialNegativeIndexFilter", "Negative Effective Bulk Modulus Acoustic Metamaterial Filter", DspNodeCategory::FilterEq, "Subwavelength resonant labyrinth metamaterial simulating negative refractive index, backward wave phase propagation, and ultra-narrow acoustic bandgaps")
        .with_param(DspParamSchema::knob("negative_modulus_center_freq_hz", "Metamaterial Negative Modulus Center", 100.0, 15000.0, 1250.0, "Hz", MacroRole::Tone, "Resonance frequency where acoustic bulk modulus reverses sign"))
        .with_param(DspParamSchema::knob("metamaterial_bandgap_width_cents", "Acoustic Bandgap Rejection Bandwidth", 50.0, 2400.0, 450.0, "cents", MacroRole::Tone, "Width of stopband region where acoustic waves cannot propagate through metamaterial"))
        .with_param(DspParamSchema::knob("effective_refractive_index", "Effective Acoustic Refractive Index (n)", -5.0, -0.1, -1.2, "n", MacroRole::Character, "Negative index of refraction reversing phase velocity relative to group velocity"))
        .with_param(DspParamSchema::knob("subwavelength_labyrinth_cells", "Labyrinth Unit Cell Cascades", 2.0, 16.0, 6.0, "cells", MacroRole::Space, "Number of coiling space-curling labyrinth unit cells steepening filter skirts"))
        .with_param(DspParamSchema::knob("evanescent_wave_amplification_db", "Evanescent Subwavelength Amplification", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Super-resolution near-field wave amplification restoring subwavelength details"))
        .with_param(DspParamSchema::toggle("enable_backward_phase_propagation", "Backward Wave Phase Propagation Mode", true, "Inverts phase response producing anomalous dispersive spatial phase delays"))
    );
    descriptors.insert("AutomatedDynamicPsychoacousticEq".to_string(), DspNodeDescriptor::new("AutomatedDynamicPsychoacousticEq", "Automated Psychoacoustic Spectral Masking & Collision Dynamic EQ", DspNodeCategory::FilterEq, "Intelligent dynamic EQ automatically identifying auditory masking collisions across critical Bark bands and dynamically carving frequency clashes")
        .with_param(DspParamSchema::knob("masking_detection_sensitivity", "Critical Bark Masking Detection Sens", 0.1, 2.0, 1.0, "sens", MacroRole::Character, "Threshold for psychoacoustic masking detection: higher values trigger earlier carving"))
        .with_param(DspParamSchema::knob("dynamic_carve_max_cut_db", "Maximum Dynamic Spectral Cut Depth", 0.0, 18.0, 6.0, "dB", MacroRole::Punch, "Maximum attenuation applied to masking frequency pockets"))
        .with_param(DspParamSchema::knob("critical_band_bark_resolution", "Auditory Filter Bank Bark Resolution", 1.0, 24.0, 12.0, "bands", MacroRole::Tone, "Number of psychoacoustic Bark scale frequency bands evaluated simultaneously"))
        .with_param(DspParamSchema::knob("dynamic_attack_recovery_ms", "Dynamic Carving Attack & Recovery Time", 5.0, 500.0, 65.0, "ms", MacroRole::Punch, "Ballistics time constant returning carved filter band to unity gain"))
        .with_param(DspParamSchema::knob("sidechain_unmask_listening_focus", "External Sidechain Unmask Priority", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Prioritizes sidechain input over local track to guarantee vocal/kick clearance"))
        .with_param(DspParamSchema::toggle("enable_equal_loudness_fletcher_munson", "ISO 226 Fletcher-Munson Equal Loudness", true, "Weights masking calculation according to human ear frequency sensitivity curve"))
    );
    descriptors.insert("AsymmetricClippingDiodeLadder".to_string(), DspNodeDescriptor::new("AsymmetricClippingDiodeLadder", "Asymmetric Germanium Diode Ladder Resonant Lowpass Filter", DspNodeCategory::FilterEq, "Transistor/diode ladder 24 dB/octave filter incorporating non-linear diode clipping in each stage for rich self-oscillation and vowel grit")
        .with_param(DspParamSchema::knob("filter_cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Corner frequency of the four-pole 24 dB/octave resonant lowpass ladder"))
        .with_param(DspParamSchema::knob("ladder_resonance_q", "Four-Pole Feedback Resonance", 0.0, 10.0, 3.5, "Q", MacroRole::Punch, "Feedback resonance intensity from subtle peak to screaming self-oscillation"))
        .with_param(DspParamSchema::knob("diode_drive_saturation", "Diode Stage Non-Linear Saturation", 0.0, 24.0, 8.0, "dB", MacroRole::Punch, "Pre-filter input overdrive pushing ladder stages into non-linear conduction"))
        .with_param(DspParamSchema::knob("germanium_forward_voltage_drop", "Diode Forward Voltage Drop (Vf)", 0.1, 0.8, 0.28, "V", MacroRole::Character, "Forward conduction threshold setting soft clipping onset and harmonic richness"))
        .with_param(DspParamSchema::knob("thermal_drift_detuning_cents", "Diode Junction Thermal Drift", 0.0, 50.0, 5.0, "cents", MacroRole::Space, "Subtle random thermal pitch drift emulating vintage analog ladder temperatures"))
        .with_param(DspParamSchema::toggle("enable_self_oscillation_limiting", "Limit Self-Oscillation Peak Amplitude", true, "Smoothly clamps runaway resonance to prevent digital clipping while preserving tone"))
    );
    descriptors.insert("CavityResonanceFormantFilter".to_string(), DspNodeDescriptor::new("CavityResonanceFormantFilter", "Coupled Acoustic Cavity Multi-Chamber Formant Filter", DspNodeCategory::FilterEq, "Acoustic physical model of three coupled Helmholtz resonators and acoustic chambers, synthesizing authentic vocal, animal, and horn vocal tract formant peaks")
        .with_param(DspParamSchema::knob("cavity_f1_pharyngeal_cutoff_hz", "Pharyngeal Cavity Formant F1", 150.0, 1200.0, 500.0, "Hz", MacroRole::Tone, "Primary pharyngeal acoustic cavity formant resonance frequency"))
        .with_param(DspParamSchema::knob("cavity_f2_oral_chamber_cutoff_hz", "Oral Cavity Formant F2", 500.0, 3500.0, 1800.0, "Hz", MacroRole::Tone, "Secondary oral cavity mouth constriction formant resonance"))
        .with_param(DspParamSchema::knob("cavity_f3_nasal_coupling_cutoff_hz", "Nasal Apex Formant F3", 1800.0, 6000.0, 2800.0, "Hz", MacroRole::Tone, "Tertiary nasal chamber and vocal tract acoustic apex formant"))
        .with_param(DspParamSchema::knob("inter_cavity_acoustic_coupling", "Inter-Chamber Acoustic Impedance Coupling", 0.0, 1.0, 0.45, "%", MacroRole::Character, "Mutual acoustic impedance coupling between adjacent resonant cavities"))
        .with_param(DspParamSchema::knob("vocal_tract_wall_damping_loss", "Compliant Wall Flesh Absorption Loss", 0.01, 0.99, 0.18, "loss", MacroRole::Punch, "Viscous friction and compliant flesh absorption damping formant peaks"))
        .with_param(DspParamSchema::toggle("enable_nasal_velum_port_bypass", "Nasal Port Velum Anti-Resonances", false, "Opens acoustic velum port introducing characteristic nasal anti-resonances and dips"))
    );
    descriptors.insert("ResonantPhasedCombArray".to_string(), DspNodeDescriptor::new("ResonantPhasedCombArray", "Stereo Cross-Feedback Phased Comb Resonator Array", DspNodeCategory::FilterEq, "High-order stereo comb filter array with negative/positive feedback cross-coupling, fractional delay modulation, and spectral phase dispersal for metallic tuned percussion and flanging")
        .with_param(DspParamSchema::knob("comb_pitch_fundamental_hz", "Comb Tooth Fundamental Frequency", 20.0, 4000.0, 220.0, "Hz", MacroRole::Tone, "Fundamental frequency and comb tooth spacing interval"))
        .with_param(DspParamSchema::knob("feedback_resonance_amount", "Regenerative Comb Feedback Ratio", -0.99, 0.99, 0.88, "fb", MacroRole::Tone, "Regeneration amount: positive for fluty hollow tones, negative for sharp woody pluck"))
        .with_param(DspParamSchema::knob("stereo_cross_coupling_ratio", "Stereo Cross-Coupled Channel Feedback", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Cross-feedback routing left comb output into right comb input"))
        .with_param(DspParamSchema::knob("high_frequency_damping_cutoff_hz", "Feedback High-Frequency Damping", 1000.0, 20000.0, 8500.0, "Hz", MacroRole::Character, "One-pole lowpass filter inside comb feedback loop dampening highs"))
        .with_param(DspParamSchema::knob("fractional_delay_detune_cents", "Stereo Delay Fractional Detune Offset", 0.0, 100.0, 12.0, "ct", MacroRole::Space, "Stereo channel delay detuning creating rich binaural shimmer"))
        .with_param(DspParamSchema::toggle("enable_allpass_phase_dispersion_chain", "Allpass Phase Dispersive Bell Cascade", true, "Inserts phase allpass filters dispersing comb harmonics into bell-like inharmonicity"))
    );
    descriptors.insert("NonlinearWaveguideFeedbackComb".to_string(), DspNodeDescriptor::new("NonlinearWaveguideFeedbackComb", "Nonlinear Waveguide Feedback Comb Resonator", DspNodeCategory::FilterEq, "Coupled waveguide delay line with feedback tanh saturation, fractional delay interpolation, and high-frequency loss damping simulating physical acoustic wave reflection and harmonic ringing")
        .with_param(DspParamSchema::knob("comb_delay_time_ms", "Resonant Comb Delay Base Period", 0.1, 100.0, 12.5, "ms", MacroRole::Tone, "Resonant comb delay time defining fundamental pitch frequency"))
        .with_param(DspParamSchema::knob("feedback_saturation_gain_db", "Feedback Nonlinear Saturation Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Nonlinear tanh overdrive driving saturation in the feedback loop"))
        .with_param(DspParamSchema::knob("damping_lowpass_cutoff_hz", "Feedback Damping Lowpass Cutoff", 200.0, 20000.0, 6500.0, "Hz", MacroRole::Tone, "High-frequency attenuation damping acoustic reflections per cycle"))
        .with_param(DspParamSchema::knob("cross_channel_phase_inversion", "Stereo Anti-Phase Cross-Feedback", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Phase-inverted stereo feedback cross-coupling for ambient spread"))
        .with_param(DspParamSchema::knob("cubic_nonlinearity_blend", "Odd Harmonic Cubic Distortion Ratio", 0.0, 1.0, 0.4, "%", MacroRole::Character, "Soft cubic polynomial distortion adding odd overtones"))
        .with_param(DspParamSchema::toggle("enable_dc_blocking_filter", "Feedback DC-Blocking Highpass", true, "Applies single-pole highpass filter preventing DC buildup in the feedback loop"))
    );
    descriptors.insert("MultiResonanceFormantBank".to_string(), DspNodeDescriptor::new("MultiResonanceFormantBank", "8-Pole Parallel Acoustic Formant Filter Bank", DspNodeCategory::FilterEq, "Eight parallel resonant bandpass filters modeling anthropomorphic vocal tracts, brass bell acoustics, and ethnic wind instrument resonances with interpolated phoneme morphing")
        .with_param(DspParamSchema::knob("vowel_phoneme_morph", "Anthropomorphic Vowel Phoneme Morph", 0.0, 5.0, 1.0, "vowel", MacroRole::Character, "Continuous interpolation across vowel formant profiles (A, E, I, O, U)"))
        .with_param(DspParamSchema::knob("formant_bandwidth_q", "Formant Peak Resonance Sharpness", 1.0, 30.0, 8.5, "Q", MacroRole::Tone, "Resonance sharpness of the formant filter bandpass poles"))
        .with_param(DspParamSchema::knob("formant_shift_semitones", "Global Formant Transposition", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Global formant frequency transposition independent of pitch"))
        .with_param(DspParamSchema::knob("nasal_cavity_coupling_ratio", "Nasal Cavity Velum Anti-Resonance", 0.0, 1.0, 0.3, "%", MacroRole::Character, "Additional anti-resonance zero representing velum nasal coupling"))
        .with_param(DspParamSchema::knob("formant_even_odd_blend", "Even vs Odd Formant Harmonic Bias", -1.0, 1.0, 0.0, "bal", MacroRole::Punch, "Balances weight of even vs odd harmonic formant peaks"))
        .with_param(DspParamSchema::toggle("enable_linear_phase_anti_comb", "Linear-Phase Inter-Pole Phase Alignment", true, "Corrects phase cancellation between adjacent resonant bandpass poles"))
    );
    descriptors.insert("ZeroPhaseLinearMasteringEqualizer".to_string(), DspNodeDescriptor::new("ZeroPhaseLinearMasteringEqualizer", "8-Band Zero-Phase Linear-Phase Mastering EQ", DspNodeCategory::FilterEq, "Surgical 8-band mastering equalizer using ultra-long partitioned FIR convolution to achieve perfect linear phase response without transient pre-ringing artifacts")
        .with_param(DspParamSchema::knob("fir_impulse_length_samples", "Linear Phase FIR Partition Window", 1024.0, 16384.0, 8192.0, "smp", MacroRole::Tone, "Length of linear-phase FIR impulse response kernel"))
        .with_param(DspParamSchema::knob("pre_ringing_suppression_pct", "Psychoacoustic Pre-Echo Suppressor", 0.0, 100.0, 75.0, "%", MacroRole::Punch, "Suppresses audible pre-ringing artifacts before sharp transients"))
        .with_param(DspParamSchema::knob("sub_low_band_gain_db", "Band 1 Sub-Bass Precision Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Punch, "Band 1 low shelf mastering gain adjustment"))
        .with_param(DspParamSchema::knob("mid_presence_band_gain_db", "Band 4 Vocal Presence Center Gain", -18.0, 18.0, 0.5, "dB", MacroRole::Tone, "Band 4 parametric presence boost/cut with surgical bandwidth"))
        .with_param(DspParamSchema::knob("high_air_band_gain_db", "Band 8 Ultra-High Air Silk Gain", -18.0, 18.0, 1.2, "dB", MacroRole::Tone, "Band 8 high shelf silky air band mastering adjustment"))
        .with_param(DspParamSchema::toggle("enable_dual_mono_stereo_unlink", "Dual-Mono Channel Independent Processing", false, "Allows independent EQ curves on left and right stereo channels"))
    );
    descriptors.insert("MorphingStateVariableLadderFilter".to_string(), DspNodeDescriptor::new("MorphingStateVariableLadderFilter", "Morphing State-Variable Ladder Filter", DspNodeCategory::FilterEq, "Zero-delay feedback state-variable filter smoothly morphing continuously between 4-pole low-pass, band-pass, high-pass, and all-pass response topologies with non-linear resonance saturation")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Filter Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Corner frequency of the multi-mode resonant filter core"))
        .with_param(DspParamSchema::knob("resonance_emphasis_q", "Resonant Peak Resonance Q", 0.5, 30.0, 3.5, "Q", MacroRole::Punch, "Feedback resonance intensity up to self-oscillating limit cycle"))
        .with_param(DspParamSchema::knob("filter_topology_morph_position", "Topology Morph (LP -> BP -> HP -> Notch)", 0.0, 3.0, 0.0, "mode", MacroRole::Character, "Continuous interpolation across filter response shapes"))
        .with_param(DspParamSchema::knob("resonance_diode_saturation_drive", "Diode Feedback Soft-Clipper Drive", 0.0, 18.0, 4.0, "dB", MacroRole::Tone, "Drives feedback path into analog diode clipping, stabilizing resonance"))
        .with_param(DspParamSchema::knob("slope_steepness_poles", "Filter Attenuation Slope (6/12/18/24 dB/oct)", 1.0, 4.0, 4.0, "poles", MacroRole::Punch, "Effective number of cascaded filter poles"))
        .with_param(DspParamSchema::toggle("enable_analog_drift_warmth", "Thermal Component Drift Warmth", true, "Injects subtle low-frequency component value jitter modeling vintage hardware"))
    );
    descriptors.insert("FerrofluidMagneticViscosityAbsorber".to_string(), DspNodeDescriptor::new("FerrofluidMagneticViscosityAbsorber", "Ferrofluid Magnetic Core Variable-Q Notch Filter", DspNodeCategory::FilterEq, "Magnetorheological audio filter modeling nanoparticle magnetic suspension viscosity changes under electromagnetic field excitation, producing tunable resonance spikes")
        .with_param(DspParamSchema::knob("electromagnet_flux_density_tesla", "Magnetic Field Flux Density", 0.0, 2.5, 0.85, "T", MacroRole::Tone, "Intensity of magnetic flux aligning iron oxide nanoparticles in fluid core"))
        .with_param(DspParamSchema::knob("nanoparticle_carrier_viscosity_cst", "Carrier Fluid Kinematic Viscosity", 1.0, 100.0, 25.0, "cSt", MacroRole::Character, "Base mechanical damping viscosity of suspension hydrocarbon oil"))
        .with_param(DspParamSchema::knob("filter_center_frequency_hz", "Active Resonant Center Frequency", 50.0, 16000.0, 1200.0, "Hz", MacroRole::Tone, "Center tuning frequency of magnetic fluid acoustic peak"))
        .with_param(DspParamSchema::knob("magnetic_shear_damping_q", "Magnetic Rheological Q Resonance", 0.5, 40.0, 8.5, "Q", MacroRole::Punch, "Filter resonance quality factor adjusted by field-induced fluid shear yield stress"))
        .with_param(DspParamSchema::knob("filter_notch_attenuation_depth_db", "Rejection Notch Attenuation Depth", -60.0, 0.0, -36.0, "dB", MacroRole::Space, "Depth of spectral notch attenuation at core resonant frequency"))
        .with_param(DspParamSchema::toggle("enable_magnetic_hysteresis_loop", "Magnetic Core Hysteresis Saturation", true, "Simulates non-linear magnetic saturation and core memory lag"))
    );
    descriptors.insert("NonlinearDiodeLadderStereoFilter".to_string(), DspNodeDescriptor::new("NonlinearDiodeLadderStereoFilter", "Vintage EMS Diode Ladder Filter with Voltage Sag", DspNodeCategory::FilterEq, "Famous 4-pole diode ladder low-pass filter with thermal voltage dependence, asymmetrical diode conduction, and resonance self-oscillation without amplitude collapse")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1400.0, "Hz", MacroRole::Tone, "Primary low-pass cutoff frequency of 4-stage diode ladder"))
        .with_param(DspParamSchema::knob("resonance_self_oscillation", "Diode Ladder Q Resonance", 0.0, 1.2, 0.75, "res", MacroRole::Punch, "Feedback resonance reaching pure sinusoidal self-oscillation above 1.0"))
        .with_param(DspParamSchema::knob("diode_thermal_voltage_mv", "Diode Junction Thermal Voltage Ut", 20.0, 35.0, 26.0, "mV", MacroRole::Character, "Governs diode conduction curve steepness and warm saturation color"))
        .with_param(DspParamSchema::knob("input_overdrive_saturation_db", "Input Preamp Diode Clipping Drive", -12.0, 36.0, 6.0, "dB", MacroRole::Tone, "Pre-filter drive level pushing diode stages into asymmetrical clipping"))
        .with_param(DspParamSchema::knob("power_rail_voltage_sag_pct", "Power Rail Dynamic Voltage Sag", 0.0, 1.0, 0.3, "%", MacroRole::Punch, "Transient power rail droop softening loud bass attacks and adding organic compression"))
        .with_param(DspParamSchema::toggle("enable_stereo_spread_offset", "Dual Channel Stereo Cutoff Spread", true, "Applies subtle opposite cutoff offsets to left and right channels for stereo width"))
    );
    descriptors.insert("AdaptiveFormantMorphingResonator".to_string(), DspNodeDescriptor::new("AdaptiveFormantMorphingResonator", "Vocal Tract Vowel Space Morphing Formant Filter", DspNodeCategory::FilterEq, "Dynamic five-formant parallel biquad resonator bank with continuous geometric interpolation across International Phonetic Alphabet (IPA) vowel space with lip radiation")
        .with_param(DspParamSchema::knob("vowel_phoneme_position_x", "Front-to-Back Vowel Position (X)", -1.0, 1.0, 0.0, "pos", MacroRole::Tone, "Horizontal vowel position interpolating between /i/ (front) and /u/ (back)"))
        .with_param(DspParamSchema::knob("vowel_tongue_height_position_y", "Open-to-Close Tongue Height (Y)", -1.0, 1.0, 0.35, "pos", MacroRole::Character, "Vertical tongue position interpolating between /a/ (open) and /i/ (close)"))
        .with_param(DspParamSchema::knob("formant_bandwidth_q_scaling", "Resonant Formant Peak Q Sharpness", 0.5, 20.0, 6.0, "Q", MacroRole::Punch, "Resonance sharpness of all five individual vocal tract formant poles"))
        .with_param(DspParamSchema::knob("nasal_cavity_anti_resonance_hz", "Velopharyngeal Nasal Antizero", 800.0, 3000.0, 1400.0, "Hz", MacroRole::Tone, "Spectral zero simulating nasal cavity coupling attenuation"))
        .with_param(DspParamSchema::knob("throat_formant_depth_db", "Subglottal Throat Resonance Gain", 0.0, 24.0, 12.0, "dB", MacroRole::Space, "Depth of resonant vowel formants relative to dry vocal excitation"))
        .with_param(DspParamSchema::toggle("enable_vocal_tract_lip_radiation", "Spherical Lip Aperture Radiation", true, "Applies +6dB/octave high-frequency radiation filter modeling acoustic lip impedance"))
    );
    descriptors.insert("ResonantCapacitorSwitchedLadderFilter".to_string(), DspNodeDescriptor::new("ResonantCapacitorSwitchedLadderFilter", "Switched-Capacitor Discrete Analog Emulation Filter", DspNodeCategory::FilterEq, "Discrete-time switched-capacitor analog filter simulation modeling non-ideal MOS switch resistance, clock feedthrough spikes, dynamic charge conservation, and steep ladder resonance")
        .with_param(DspParamSchema::knob("filter_cutoff_clock_frequency_hz", "Switching Master Clock Cutoff", 20.0, 20000.0, 2200.0, "Hz", MacroRole::Tone, "Master non-overlapping clock frequency driving virtual capacitor charge switches"))
        .with_param(DspParamSchema::knob("switched_capacitor_charge_q", "Resonance Feedback Q Factor", 0.5, 30.0, 8.5, "Q", MacroRole::Punch, "Feedback resonance sharpness with bounded saturation self-oscillation"))
        .with_param(DspParamSchema::knob("clock_leakage_feedthrough_db", "MOSFET Clock Feedthrough Leakage", -80.0, -12.0, -42.0, "dB", MacroRole::Character, "High-frequency switching clock noise injected into audio passband"))
        .with_param(DspParamSchema::knob("charge_transfer_nonlinearity", "Nonlinear Switch On-Resistance", 0.01, 1.0, 0.22, "ron", MacroRole::Tone, "Voltage-dependent resistance of virtual MOS transistors causing warm saturation"))
        .with_param(DspParamSchema::knob("resonance_feedback_damping", "High-Frequency Self-Oscillation Damping", 0.01, 0.5, 0.08, "loss", MacroRole::Space, "Damps extreme high-frequency ringing to preserve clean musical punch"))
        .with_param(DspParamSchema::toggle("enable_discrete_charge_injection", "Dynamic Charge Injection Modeling", true, "Simulates channel charge injection during MOSFET switch turn-off transitions"))
    );
    descriptors.insert("MorphingEllipticCauerLadderFilter".to_string(), DspNodeDescriptor::new("MorphingEllipticCauerLadderFilter", "Morphing Elliptic / Cauer Active Ladder Filter", DspNodeCategory::FilterEq, "8th-order analog-modeled Elliptic (Cauer) ladder filter with equiripple passband and stopband ripple, zero-delay feedback (ZDF) topology, and razor-sharp transition bands")
        .with_param(DspParamSchema::knob("filter_cutoff_frequency_hz", "Elliptic Filter Cutoff Frequency", 20.0, 20000.0, 1800.0, "Hz", MacroRole::Tone, "Corner frequency where passband ends and sharp transition begins"))
        .with_param(DspParamSchema::knob("passband_equiripple_db", "Passband Ripple Toleration", 0.01, 3.0, 0.5, "dB", MacroRole::Punch, "Permissible ripple in passband (higher ripple sharpens transition slope)"))
        .with_param(DspParamSchema::knob("stopband_attenuation_floor_db", "Stopband Rejection Floor", 24.0, 96.0, 60.0, "dB", MacroRole::Character, "Minimum attenuation floor enforced across entire stopband region"))
        .with_param(DspParamSchema::knob("resonance_pole_q_factor", "Cutoff Resonance Peak Q Factor", 0.5, 25.0, 4.5, "Q", MacroRole::Tone, "Resonant peaking at cutoff corner with non-linear saturation clamping"))
        .with_param(DspParamSchema::knob("morph_filter_topology_type", "Filter Response Shape Morph (LP to HP)", 0.0, 2.0, 0.0, "type", MacroRole::Space, "Continuous morphing between Lowpass (0.0), Bandpass (1.0), and Highpass (2.0)"))
        .with_param(DspParamSchema::toggle("enable_zero_delay_feedback", "Zero-Delay Feedback (ZDF) Solvers", true, "Solves non-linear implicit delay feedback loop using Newton-Raphson iteration"))
    );
    descriptors.insert("DualPeakFormantVowelBiquadFilter".to_string(), DspNodeDescriptor::new("DualPeakFormantVowelBiquadFilter", "Dual-Peak Vocal Vowel Formant Biquad Filter", DspNodeCategory::FilterEq, "Formant speech filter with two resonant biquad poles tracking F1 and F2 vowel frequencies, vocal tract width scaling, vowel phoneme morphing, and throat warmth")
        .with_param(DspParamSchema::knob("formant_f1_frequency_hz", "Formant F1 Frequency (Jaw Opening)", 200.0, 1200.0, 650.0, "Hz", MacroRole::Tone, "First vocal formant pole governing vowel openness (close /i/ vs open /a/)"))
        .with_param(DspParamSchema::knob("formant_f2_frequency_hz", "Formant F2 Frequency (Tongue Shape)", 500.0, 3500.0, 1800.0, "Hz", MacroRole::Character, "Second vocal formant pole governing front vs back tongue position"))
        .with_param(DspParamSchema::knob("formant_q_sharpness_factor", "Formant Resonant Peak Sharpness Q", 1.0, 30.0, 8.0, "Q", MacroRole::Punch, "Resonance sharpness of both vocal tract formant filter poles"))
        .with_param(DspParamSchema::knob("vowel_crossfade_morph_index", "Vowel Preset Morph (A-E-I-O-U)", 0.0, 4.0, 0.0, "morph", MacroRole::Space, "Smooth interpolation across classic vowels: /a/, /e/, /i/, /o/, /u/"))
        .with_param(DspParamSchema::knob("throat_resonance_saturation_db", "Subglottic Throat Drive Saturation", 0.0, 24.0, 5.0, "dB", MacroRole::Tone, "Asymmetric saturation adding gravelly warmth to the vocal tract filter"))
        .with_param(DspParamSchema::toggle("enable_speech_lip_radiation_boost", "High-Frequency Lip Radiation (+6dB)", true, "Applies lip radiation high-frequency lift for vocal intelligibility"))
    );
    descriptors.insert("SpectralFormantMorphingVowelMatrix".to_string(), DspNodeDescriptor::new("SpectralFormantMorphingVowelMatrix", "Multi-Vowel 5-Formant Dynamic Morphing Matrix", DspNodeCategory::FilterEq, "Surgical 5-formant vocal tract filter capable of 2D vowel space interpolation, vocal tract elongation, larynx height shifting, and nasal anti-resonance poles")
        .with_param(DspParamSchema::knob("vowel_xy_space_interpolation_x", "Vowel Space Dimension X (Front-Back)", 0.0, 1.0, 0.35, "pos", MacroRole::Tone, "Horizontal position in vowel space controlling tongue advancement"))
        .with_param(DspParamSchema::knob("vowel_xy_space_interpolation_y", "Vowel Space Dimension Y (High-Low)", 0.0, 1.0, 0.65, "pos", MacroRole::Character, "Vertical position in vowel space controlling jaw opening and tongue height"))
        .with_param(DspParamSchema::knob("larynx_tract_length_scaling", "Vocal Tract Physical Length Scale", 0.7, 1.4, 1.0, "scale", MacroRole::Punch, "Scales all 5 formant frequencies simultaneously simulating physical tract size"))
        .with_param(DspParamSchema::knob("formant_bandwidth_q_scaling", "Formant Resonance Bandwidth Q", 0.5, 8.0, 2.8, "Q", MacroRole::Tone, "Resonance sharpness of all vocal tract formant peaks"))
        .with_param(DspParamSchema::knob("nasal_cavity_anti_resonance_depth_db", "Velopharyngeal Nasal Anti-Pole Depth", -24.0, 0.0, -8.0, "dB", MacroRole::Space, "Depth of nasal tract anti-formant notch filter around 800 Hz"))
        .with_param(DspParamSchema::toggle("enable_formant_glottal_coupling", "Dynamic Fundamental Pitch Coupling", true, "Modulates formant frequencies dynamically based on incoming fundamental pitch"))
    );
    descriptors.insert("MultibandSpectralDeEsserDynamic".to_string(), DspNodeDescriptor::new("MultibandSpectralDeEsserDynamic", "Linear-Phase Dynamic Sibilance Spectral De-Esser", DspNodeCategory::FilterEq, "Surgical dynamic vocal sibilance processor combining linear-phase split-band detection, lookahead thresholding, adaptive frequency notch centering, and harmonic preservation")
        .with_param(DspParamSchema::knob("sibilance_detection_center_hz", "De-Esser Detection Frequency Center", 3000.0, 12000.0, 6800.0, "Hz", MacroRole::Tone, "Center frequency of sibilance detection filter targeting harsh 's' and 't' sounds"))
        .with_param(DspParamSchema::knob("dynamic_reduction_threshold_db", "Sibilance Suppression Threshold", -36.0, 0.0, -18.0, "dB", MacroRole::Punch, "Threshold above which dynamic sibilance suppression activates"))
        .with_param(DspParamSchema::knob("attenuation_range_limit_db", "Maximum De-Essing Attenuation Depth", -24.0, 0.0, -9.0, "dB", MacroRole::Tone, "Maximum permissible reduction to prevent dulling vocal tone"))
        .with_param(DspParamSchema::knob("sibilance_bandwidth_q_width", "Detection Filter Bandwidth Q Factor", 0.5, 5.0, 1.6, "Q", MacroRole::Character, "Bandwidth Q width of the dynamic attenuation notch filter"))
        .with_param(DspParamSchema::knob("deesser_attack_lookahead_ms", "Lookahead Transient Buffer Delay", 0.0, 10.0, 2.5, "ms", MacroRole::Space, "Lookahead delay buffer allowing zero-clipping catch of sharp sibilance"))
        .with_param(DspParamSchema::toggle("enable_listen_sibilance_delta_mode", "Listen Sibilance Delta Solo Audition", false, "Solo audition mode monitoring only the removed sibilant frequencies"))
    );
    descriptors.insert("MorphingStateVariableLadderCombFilter".to_string(), DspNodeDescriptor::new("MorphingStateVariableLadderCombFilter", "Hybrid ZDF SVF & Resonant Comb Filter Matrix", DspNodeCategory::FilterEq, "Hybrid zero-delay feedback state variable filter cascaded with a resonant feedback comb filter, offering continuous morphing from 4-pole lowpass to high-order comb filtering")
        .with_param(DspParamSchema::knob("filter_cutoff_frequency_hz", "ZDF Filter Master Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Corner cutoff frequency of the zero-delay feedback state variable filter"))
        .with_param(DspParamSchema::knob("resonance_peak_q_factor", "State Variable Resonance Peak (Q)", 0.5, 25.0, 3.8, "Q", MacroRole::Character, "Resonance feedback sharpness at filter cutoff frequency approaching self-oscillation"))
        .with_param(DspParamSchema::knob("comb_delay_frequency_tuning_hz", "Feedback Comb Resonant Pitch Tuning", 20.0, 5000.0, 240.0, "Hz", MacroRole::Tone, "Fundamental frequency of the recirculating comb delay loop"))
        .with_param(DspParamSchema::knob("comb_recirculation_feedback", "Comb Loop Resonance Feedback Ratio", -0.99, 0.99, 0.78, "fdbk", MacroRole::Character, "Gain factor of recirculating comb delay line dictating metallic resonant decay time"))
        .with_param(DspParamSchema::knob("svf_to_comb_morph_blend", "SVF to Comb Filter Morphology Blend", 0.0, 1.0, 0.5, "morph", MacroRole::Tone, "Continuous crossfade between pure analog SVF filter and deep metallic comb filtering"))
        .with_param(DspParamSchema::toggle("enable_filter_nonlinear_saturation", "ZDF Resonant Non-linear Core Saturation", true, "Tanh saturator in the feedback loop for vintage analog warmth without digital clipping"))
    );
    descriptors.insert("DualSvfMorphingResonanceEqualizer".to_string(), DspNodeDescriptor::new("DualSvfMorphingResonanceEqualizer", "Dual Parallel State-Variable Morphing Equalizer", DspNodeCategory::FilterEq, "Dual parallel 2-pole state variable filter pair with continuous morphing from lowpass through notch/bandpass to highpass, cross-resonance feedback, and stereo panning offset")
        .with_param(DspParamSchema::knob("filter_a_cutoff_frequency_hz", "Filter A Center Cutoff Frequency", 20.0, 20000.0, 480.0, "Hz", MacroRole::Tone, "Cutoff/center frequency for the primary state-variable filter branch"))
        .with_param(DspParamSchema::knob("filter_b_cutoff_frequency_hz", "Filter B Center Cutoff Frequency", 20.0, 20000.0, 2800.0, "Hz", MacroRole::Tone, "Cutoff/center frequency for the secondary state-variable filter branch"))
        .with_param(DspParamSchema::knob("filter_a_morph_topology", "Filter A Response Topology Morph", 0.0, 3.0, 0.0, "topo", MacroRole::Tone, "Continuous morphing: 0=Lowpass, 1=Bandpass, 2=Highpass, 3=Notch"))
        .with_param(DspParamSchema::knob("filter_b_morph_topology", "Filter B Response Topology Morph", 0.0, 3.0, 2.0, "topo", MacroRole::Tone, "Continuous morphing: 0=Lowpass, 1=Bandpass, 2=Highpass, 3=Notch"))
        .with_param(DspParamSchema::knob("filter_cross_resonance_coupling", "Filter A/B Cross-Resonance Coupling Feedback", 0.0, 1.0, 0.28, "coup", MacroRole::Character, "Cross-modulates resonance between Filter A and B creating complex vowel formant trajectories"))
        .with_param(DspParamSchema::toggle("enable_stereo_anti_phase_spread", "Stereo Anti-Phase Cutoff Panning Spread", true, "Shifts Filter A cutoff left and Filter B cutoff right for wide binaural filtering"))
    );
    descriptors.insert("PassivePultecInductorProgramEqualizer".to_string(), DspNodeDescriptor::new("PassivePultecInductorProgramEqualizer", "Passive Inductor-Capacitor Vintage Program Equalizer", DspNodeCategory::FilterEq, "Exact circuit modeling of the legendary passive LC program equalizer featuring interactive simultaneous low-shelf boost and attenuate curves and toroidal inductor saturation")
        .with_param(DspParamSchema::knob("low_shelf_frequency_select_hz", "Low-End Stepped Frequency Selector", 20.0, 100.0, 60.0, "Hz", MacroRole::Tone, "Selectable stepped bass frequency band (20, 30, 60, or 100 Hz) for shelf filter"))
        .with_param(DspParamSchema::knob("low_shelf_boost_gain_db", "Simultaneous Low-Frequency Boost Amount", 0.0, 16.0, 5.5, "dB", MacroRole::Punch, "Inductor-capacitor resonant low-shelf gain boost providing massive sub-bass weight"))
        .with_param(DspParamSchema::knob("low_shelf_attenuate_gain_db", "Simultaneous Low-Frequency Dip Attenuation", 0.0, 16.0, 4.0, "dB", MacroRole::Punch, "Resistive bass attenuation interacting with boost to create the classic Pultec mid scoop"))
        .with_param(DspParamSchema::knob("high_peak_bandwidth_q", "Toroidal Inductor High-Frequency Peak Q", 0.5, 10.0, 2.5, "Q", MacroRole::Tone, "Sharpness (Broad to Sharp) of high-frequency inductive bell filter peak curve"))
        .with_param(DspParamSchema::knob("high_peak_boost_gain_db", "Toroidal Inductor High Bell Peak Boost", 0.0, 18.0, 4.5, "dB", MacroRole::Tone, "High-frequency toroidal inductor bell peak boost adding brilliant silk and air"))
        .with_param(DspParamSchema::toggle("enable_inter_winding_transformer_core", "Triad Input/Output Transformer Magnetics", true, "Simulates magnetic core saturation and passive insertion loss makeup amplifier gain"))
    );
    descriptors.insert("MorphingLadderSallenKeyHybridFilter".to_string(), DspNodeDescriptor::new("MorphingLadderSallenKeyHybridFilter", "Hybrid ZDF Moog Ladder & Sallen-Key State Filter", DspNodeCategory::FilterEq, "Zero-delay feedback hybrid 4-pole transistor ladder filter cascaded with a 2-pole Sallen-Key filter with smooth topology morphing and diode limiter feedback saturation")
        .with_param(DspParamSchema::knob("master_cutoff_frequency_hz", "Master Cutoff Corner Frequency", 15.0, 22000.0, 1800.0, "Hz", MacroRole::Tone, "Corner cutoff frequency of the cascaded hybrid zero-delay feedback filter network"))
        .with_param(DspParamSchema::knob("resonance_feedback_intensity", "Self-Oscillation Resonance Feedback Amount", 0.0, 1.05, 0.68, "res", MacroRole::Character, "Internal feedback gain driving filter into pure sinusoidal self-oscillation"))
        .with_param(DspParamSchema::knob("ladder_to_sallen_morph_ratio", "Transistor Ladder vs Sallen-Key Topology Morph", 0.0, 1.0, 0.35, "morph", MacroRole::Character, "Continuous crossfade between 24dB/oct transistor ladder and 12dB/oct MS-20 style Sallen-Key"))
        .with_param(DspParamSchema::knob("diode_limiter_saturation_drive", "Feedback Diode Symmetrical Limiter Drive", 0.1, 8.0, 1.8, "drive", MacroRole::Punch, "Drive level pushing feedback loop into dual anti-parallel silicon diodes for warm clipping"))
        .with_param(DspParamSchema::knob("four_pole_bass_compensation", "Resonance Bass Drop Makeup Compensation", 0.0, 1.0, 0.5, "comp", MacroRole::Punch, "Compensates natural low-frequency insertion loss as resonance feedback is increased"))
        .with_param(DspParamSchema::toggle("enable_analog_thermal_frequency_drift", "Transistor Junction Thermal Drift Simulation", true, "Injects microscopic thermal pitch drift into transistor ladder differential pairs"))
    );
    descriptors.insert("DualResonantMorphingEllipticFilter".to_string(), DspNodeDescriptor::new("DualResonantMorphingEllipticFilter", "Cauer Elliptic ZDF Dual-Resonant Equal-Ripple Filter", DspNodeCategory::FilterEq, "Zero-delay feedback (ZDF) Cauer elliptic filter featuring sharp brickwall transition slopes, tunable passband/stopband equiripples, and dual morphable resonant transmission zero poles")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Elliptic Filter Cutoff Corner Frequency", 20.0, 20000.0, 1500.0, "Hz", MacroRole::Tone, "Transition frequency boundary between passband and stopband"))
        .with_param(DspParamSchema::knob("stopband_attenuation_depth_db", "Stopband Minimum Attenuation Floor", 24.0, 96.0, 60.0, "dB", MacroRole::Tone, "Depth of stopband rejection setting steepness of the Cauer elliptic transition curve"))
        .with_param(DspParamSchema::knob("passband_equiripple_peak_db", "Passband Equiripple Resonant Peak Height", 0.1, 6.0, 1.2, "dB", MacroRole::Character, "Height of the Chebyshev-style ripple peaks distributed across the passband"))
        .with_param(DspParamSchema::knob("dual_pole_resonance_split_octaves", "Dual Resonant Peaks Spectral Separation", 0.0, 4.0, 1.5, "oct", MacroRole::Tone, "Splits filter resonance into two independently morphable formant-like peaks"))
        .with_param(DspParamSchema::knob("lowpass_to_highpass_morph_blend", "Lowpass to Highpass Topology Continuous Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Character, "Smoothly interpolates filter poles and transmission zeros from lowpass to highpass"))
        .with_param(DspParamSchema::toggle("enable_nonlinear_zdf_saturation", "ZDF Integrator Diode Clipping Feedback Loop", true, "Enables soft nonlinear saturation inside filter state integrators preventing harsh digital blowout"))
    );
    descriptors.insert("ParametricStateVariableFormantPeakFilter".to_string(), DspNodeDescriptor::new("ParametricStateVariableFormantPeakFilter", "Parametric Vocal Formant Peak State-Variable Filter", DspNodeCategory::FilterEq, "Three-peak state-variable resonant filter mapped to human vocal tract vowel formants (F1, F2, F3) with continuous 2D vowel matrix morphing between A, E, I, O, U and throat size scaling")
        .with_param(DspParamSchema::knob("vowel_coordinate_x_front_back", "Vocal Vowel Trajectory Front/Back Position (F2)", 0.0, 1.0, 0.45, "pos", MacroRole::Tone, "Vowel position mapping front vowels (I, E) to back vowels (O, U)"))
        .with_param(DspParamSchema::knob("vowel_coordinate_y_open_closed", "Vocal Vowel Trajectory Open/Closed Jaw (F1)", 0.0, 1.0, 0.65, "open", MacroRole::Tone, "Vowel jaw openness mapping closed vowels (I, U) to open vowels (A, O)"))
        .with_param(DspParamSchema::knob("vocal_tract_length_scale_factor", "Anatomical Vocal Tract Length Scale (Throat Size)", 0.7, 1.5, 1.0, "scale", MacroRole::Character, "Scales all formant frequencies simultaneously simulating child/female vs bass male vocal tracts"))
        .with_param(DspParamSchema::knob("formant_bandwidth_q_sharpness", "Formant Resonant Bandwidth Sharpness (Q)", 2.0, 30.0, 12.0, "Q", MacroRole::Punch, "Sharpness and acoustic focus of vowel peak resonances"))
        .with_param(DspParamSchema::knob("formant_glottal_breath_drive_db", "Glottal Whisper Breath Harmonic Drive", 0.0, 18.0, 3.5, "dB", MacroRole::Character, "Adds soft vocal breath turbulence and odd harmonics into the filter peaks"))
        .with_param(DspParamSchema::toggle("enable_subharmonic_laryngeal_growl", "Laryngeal Ventricular Band Subharmonic Growl", false, "Adds throat-singing subharmonic growl resonance around 60-80 Hz"))
    );
    descriptors.insert("MorphingButterworthChebyshevHybridFilter".to_string(), DspNodeDescriptor::new("MorphingButterworthChebyshevHybridFilter", "ZDF Morphing Butterworth / Chebyshev Hybrid Filter", DspNodeCategory::FilterEq, "Zero-delay feedback (ZDF) filter with continuous polynomial morphing between maximally flat Butterworth response and steep, ripple-controlled Chebyshev Type I response with resonant Q control")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Master Filter Cutoff Corner Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Cutoff frequency transition point"))
        .with_param(DspParamSchema::knob("butterworth_to_chebyshev_morph", "Filter Polynomial Characteristic Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Tone, "0.0: Maximally flat Butterworth, 1.0: Steep equiripple Chebyshev Type I"))
        .with_param(DspParamSchema::knob("chebyshev_passband_ripple_db", "Chebyshev Passband Equiripple Depth", 0.05, 3.0, 0.5, "dB", MacroRole::Character, "Height of passband ripple peaks setting transition steepness"))
        .with_param(DspParamSchema::knob("filter_resonance_q_peak", "Resonance Feedback Quality Factor (Q)", 0.5, 25.0, 2.5, "Q", MacroRole::Punch, "Resonant feedback peak height at cutoff frequency"))
        .with_param(DspParamSchema::knob("filter_slope_steepness_poles", "Filter Order Pole Count (12/24/36 dB/oct)", 2.0, 6.0, 4.0, "poles", MacroRole::Tone, "Number of reactive filter poles setting transition slope steepness"))
        .with_param(DspParamSchema::toggle("enable_diode_ladder_saturation", "Nonlinear Sallen-Key Diode Limiting Feedback", true, "Soft clips filter resonance state preventing digital overflow during aggressive sweeps"))
    );
    descriptors.insert("FormantVocalTractNasalBranchResonator".to_string(), DspNodeDescriptor::new("FormantVocalTractNasalBranchResonator", "Biomechanical Vocal Tract Nasal Branch Resonator", DspNodeCategory::FilterEq, "Physical transmission-line model of the human vocal tract incorporating a parallel nasal cavity branch with movable velum porting, creating authentic nasalized vowels (M, N, NG, French vowels)")
        .with_param(DspParamSchema::knob("velum_opening_aperture_ratio", "Velopharyngeal Port Velum Opening Ratio", 0.0, 1.0, 0.45, "velum", MacroRole::Tone, "Aperture of soft palate opening connecting oral tract to nasal cavity"))
        .with_param(DspParamSchema::knob("nasal_cavity_volume_scale", "Nasal Cavity Acoustic Volume Scaling", 0.5, 2.0, 1.0, "scale", MacroRole::Character, "Physical volume of nasal passages tuning characteristic nasal formants (Fn1 ~250 Hz, Fn2 ~2000 Hz)"))
        .with_param(DspParamSchema::knob("oral_tract_tongue_position_x", "Oral Tract Tongue Front/Back Position", 0.0, 1.0, 0.5, "pos", MacroRole::Tone, "Tongue hump position moving formants between front and back vowels"))
        .with_param(DspParamSchema::knob("oral_tract_jaw_openness_y", "Oral Tract Jaw Mandible Openness", 0.0, 1.0, 0.6, "open", MacroRole::Tone, "Degree of mouth opening setting primary first formant F1"))
        .with_param(DspParamSchema::knob("nasal_wall_mucous_absorption_damping", "Nasal Mucosa Viscous Acoustic Absorption", 0.1, 2.0, 0.85, "damp", MacroRole::Tone, "High viscous acoustic damping caused by soft mucous tissue in nasal conchae"))
        .with_param(DspParamSchema::toggle("enable_turbulent_nasal_fricative_air", "Nasal Airflow Turbulence & Fricative Whistle", false, "Adds turbulence noise generated by narrow airflow through nostrils"))
    );
    descriptors.insert("NonlinearMoogTransistorLadder24dbFilter".to_string(), DspNodeDescriptor::new("NonlinearMoogTransistorLadder24dbFilter", "ZDF 4-Pole 24 dB/oct Transistor Ladder Filter", DspNodeCategory::FilterEq, "Zero-delay feedback (ZDF) analog-modeled 4-pole 24 dB/octave transistor ladder filter with temperature-compensated hyperbolic tangent (tanh) pole saturation and resonance bass compensation")
        .with_param(DspParamSchema::knob("ladder_cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1800.0, "Hz", MacroRole::Tone, "Cutoff frequency determined by control voltage driving transistor differential pairs"))
        .with_param(DspParamSchema::knob("resonance_feedback_amount", "Ladder Resonance Self-Oscillation (Q)", 0.0, 4.2, 2.8, "res", MacroRole::Character, "4th-pole feedback amount: at 4.0 filter enters pure sine wave self-oscillation"))
        .with_param(DspParamSchema::knob("transistor_tanh_saturation_drive", "Differential Pair Tanh Overdrive", 0.0, 24.0, 8.0, "dB", MacroRole::Punch, "Input drive producing asymmetrical harmonic saturation across ladder stages"))
        .with_param(DspParamSchema::knob("passband_bass_drop_compensation", "Resonance Bass Drop Loss Compensation", 0.0, 1.0, 0.65, "comp", MacroRole::Punch, "Restores low-end punch that traditionally vanishes at high resonance settings"))
        .with_param(DspParamSchema::knob("thermal_drift_voltage_mv", "Ladder Semiconductor Thermal Drift", -10.0, 10.0, 0.5, "mV", MacroRole::Tone, "Slight thermal asymmetry between ladder rungs adding rich analog imperfections"))
        .with_param(DspParamSchema::toggle("enable_smooth_resonance_limiting", "Nonlinear Feedback Diode Resonance Limiter", true, "Soft-clips self-oscillation peaks preventing digital harshness when filter self-oscillates"))
    );
    descriptors.insert("MorphingFormantVowelBandpassFilterBank".to_string(), DspNodeDescriptor::new("MorphingFormantVowelBandpassFilterBank", "5-Formant Phonetic Vowel Resonator Filter Bank", DspNodeCategory::FilterEq, "Dynamic 5-formant parallel bandpass resonator array morphing continuously between phonetic vowel registers (A, E, I, O, U) with vocal tract scale shifting and throat formant warmth")
        .with_param(DspParamSchema::knob("vowel_phoneme_morph_position", "Vowel Phoneme Morph (A - E - I - O - U)", 0.0, 4.0, 1.0, "pos", MacroRole::Tone, "Continuous interpolation across five primary vowel formant profiles"))
        .with_param(DspParamSchema::knob("formant_frequency_shift_semitones", "Formant Frequency Transpose Shift", -12.0, 12.0, 0.0, "st", MacroRole::Character, "Shifts entire formant cluster up or down changing perceived throat length"))
        .with_param(DspParamSchema::knob("formant_resonance_q_bandwidth", "Formant Resonator Peak Sharpness (Q)", 2.0, 30.0, 12.0, "Q", MacroRole::Tone, "Bandwidth sharpness of parallel second-order bandpass resonators"))
        .with_param(DspParamSchema::knob("throat_depth_sub_formant_db", "Sub-Glottal Throat Resonance Boost", 0.0, 12.0, 3.5, "dB", MacroRole::Punch, "Low-frequency chest resonance (~180 Hz) adding vocal authority"))
        .with_param(DspParamSchema::knob("vocal_tract_gender_scaling", "Vocal Tract Anatomy Scale (Child to Bass)", 0.6, 1.4, 1.0, "scale", MacroRole::Tone, "Physical vocal tract scaling factor (0.8: soprano, 1.0: tenor, 1.25: bass)"))
        .with_param(DspParamSchema::toggle("enable_dynamic_nasal_coupling", "Secondary Nasal Cavity Coupling Pole", true, "Adds anti-resonance dip and 1 kHz nasal peak for organic human vocal articulation"))
    );
    descriptors.insert("NonlinearDiodeLadderZdf24dbLowpassFilter".to_string(), DspNodeDescriptor::new("NonlinearDiodeLadderZdf24dbLowpassFilter", "Nonlinear Diode Ladder ZDF 24dB/oct Filter", DspNodeCategory::FilterEq, "Zero-delay feedback (ZDF) vintage diode ladder filter (EMS VCS3 / TB-303 topology) with exponential diode junction nonlinearities, resonant screaming self-oscillation, and passband drop compensation")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Ladder Cutoff Corner Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Filter corner cutoff frequency governed by ladder control current"))
        .with_param(DspParamSchema::knob("resonance_feedback_q", "Diode Ladder Resonance Feedback (Q)", 0.5, 12.0, 4.2, "Q", MacroRole::Character, "Resonance feedback gain pushing ladder into rich singing self-oscillation"))
        .with_param(DspParamSchema::knob("diode_saturation_thermal_voltage_mv", "Diode Junction Thermal Voltage (Vt)", 15.0, 45.0, 26.0, "mV", MacroRole::Character, "Diode junction thermal voltage (Vt) scaling soft saturation compression"))
        .with_param(DspParamSchema::knob("passband_bass_drop_compensation", "Passband Low-End Punch Compensation", 0.0, 1.0, 0.6, "comp", MacroRole::Punch, "Restores low-end punch lost at high resonance settings"))
        .with_param(DspParamSchema::knob("input_overdrive_drive_gain_db", "Input Diode Stage Drive Amplification", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Pushes input signal into first diode stage for gritty acid overdrive"))
        .with_param(DspParamSchema::toggle("enable_adaa_second_order_antialiasing", "Second-Order ADAA Anti-Aliasing", true, "Second-order Anti-Derivative Anti-Aliasing for pristine resonance sweeping"))
    );
    descriptors.insert("MorphingSpectralVowelResonanceFilter".to_string(), DspNodeDescriptor::new("MorphingSpectralVowelResonanceFilter", "3-Formant Morphing Vowel Resonance Filter", DspNodeCategory::FilterEq, "Triple formant vocal tract resonator filter featuring 2D continuous phonetic vowel interpolation [A-E-I-O-U], throat/nasal cavity branch pole-zero pairs, and vocal tract length gender scaling")
        .with_param(DspParamSchema::knob("vowel_interpolation_x", "Horizontal Formant Coordinate (Front-Back)", 0.0, 1.0, 0.25, "X", MacroRole::Tone, "Horizontal vowel vowel formant coordinate (Front vs Back vowels)"))
        .with_param(DspParamSchema::knob("vowel_interpolation_y", "Vertical Formant Coordinate (Open-Closed)", 0.0, 1.0, 0.5, "Y", MacroRole::Tone, "Vertical vowel formant coordinate (Open vs Closed vowels)"))
        .with_param(DspParamSchema::knob("vocal_tract_length_scaling", "Vocal Tract Length Scaling Factor", 0.7, 1.4, 1.0, "scale", MacroRole::Character, "Scales all formant frequencies: <1.0 = child/female, >1.0 = deep baritone"))
        .with_param(DspParamSchema::knob("formant_resonance_q_sharpness", "Formant Peak Q Sharpness / Articulation", 2.0, 25.0, 8.5, "Q", MacroRole::Character, "Resonance peaking of formant bandpass filters shaping speech articulation"))
        .with_param(DspParamSchema::knob("nasal_coupling_bandwidth_hz", "Velum Nasal Cavity Anti-Resonance Width", 50.0, 500.0, 180.0, "Hz", MacroRole::Tone, "Velum port opening coupling nasal cavity anti-resonances (~800 Hz)"))
        .with_param(DspParamSchema::toggle("enable_formant_jitter_vibrato", "Organic Vocal Fold Jitter / Vibrato", true, "Adds micro-fluctuations simulating involuntary human vocal fold tremors"))
    );
    descriptors.insert("NonlinearSallenKeyActive12dbFilter".to_string(), DspNodeDescriptor::new("NonlinearSallenKeyActive12dbFilter", "Nonlinear Sallen-Key Active 12dB Filter", DspNodeCategory::FilterEq, "Active 2nd-order analog Sallen-Key lowpass/highpass filter with op-amp slew limiting, back-to-back diode saturation, and Q compensation")
        .with_param(DspParamSchema::knob("cutoff", "Filter Resonant Cutoff Frequency", 10.0, 22000.0, 1200.0, "Hz", MacroRole::Tone, "Cutoff corner frequency of the Sallen-Key RC network"))
        .with_param(DspParamSchema::knob("resonance_q", "Feedback Resonance Q Factor", 0.5, 25.0, 4.2, "Q", MacroRole::Character, "Resonance peaking intensity at cutoff frequency"))
        .with_param(DspParamSchema::knob("opamp_slew_rate", "Op-Amp Slew Rate Limit", 0.1, 50.0, 8.0, "V/us", MacroRole::Punch, "Operational amplifier slew rate limit generating dynamic softening"))
        .with_param(DspParamSchema::knob("diode_feedback_clip", "Feedback Diode Soft-Clipping", 0.0, 1.0, 0.35, "sat", MacroRole::Punch, "Back-to-back diodes across feedback loop taming self-oscillation"))
        .with_param(DspParamSchema::knob("filter_mode", "Filter State Low/High Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Tone, "Continuous response morph: 0.0 = Lowpass, 1.0 = Highpass"))
        .with_param(DspParamSchema::knob("input_drive", "Core Preamplifier Input Drive", 0.0, 24.0, 3.0, "dB", MacroRole::Punch, "Input drive gain into the active filter core"))
    );
    descriptors.insert("SpectralCombFilterBankFormantMorpher".to_string(), DspNodeDescriptor::new("SpectralCombFilterBankFormantMorpher", "Spectral Comb Filter Bank Formant Morpher", DspNodeCategory::FilterEq, "Parallel bank of 8 tuned feedback comb filters arranged to morph input timbre continuously through vowel formant resonances")
        .with_param(DspParamSchema::knob("formant_vowel", "Vowel Formant Morph Target", 0.0, 4.0, 0.0, "vowel", MacroRole::Character, "Continuous morphing across vowel formant spaces: A - E - I - O - U"))
        .with_param(DspParamSchema::knob("resonance_feedback", "Comb Peak Feedback Intensity", 0.0, 0.99, 0.88, "fb", MacroRole::Character, "Internal comb filter feedback setting vocal peak sharp sharpness"))
        .with_param(DspParamSchema::knob("comb_spacing", "Harmonic Comb Spacing Ratio", 0.5, 2.0, 1.0, "ratio", MacroRole::Tone, "Stretches or compresses relative comb frequency peak spacings"))
        .with_param(DspParamSchema::knob("pitch_tracking", "Key Pitch Tracking Amount", 0.0, 1.0, 0.50, "track", MacroRole::Tone, "Scales comb filter frequencies with incoming musical note pitch"))
        .with_param(DspParamSchema::knob("damping_hf", "Comb Loop High Damping", 1000.0, 20000.0, 9500.0, "Hz", MacroRole::Tone, "High frequency attenuation inside comb recirculation paths"))
        .with_param(DspParamSchema::knob("dry_wet_mix", "Formant Resonator Wet / Dry", 0.0, 1.0, 0.70, "mix", MacroRole::Space, "Wet/dry mix between direct sound and vowel formant bank"))
    );
    descriptors.insert("NonlinearKorg35StateVariable24dbFilter".to_string(), DspNodeDescriptor::new("NonlinearKorg35StateVariable24dbFilter", "Nonlinear Korg 35 State Variable 24dB Filter", DspNodeCategory::FilterEq, "Discrete operational amplifier and diode nonlinear model of the classic 1970s Korg 35 filter circuit with aggressive self-oscillating resonance and diode-clamped feedback")
        .with_param(DspParamSchema::knob("cutoff_frequency", "Filter Cutoff Corner Frequency", 15.0, 22000.0, 950.0, "Hz", MacroRole::Tone, "Master cutoff frequency of the cascaded 4-pole Korg 35 discrete network"))
        .with_param(DspParamSchema::knob("resonance_peak", "Feedback Screaming Resonance Peak", 0.0, 1.0, 0.75, "peak", MacroRole::Character, "Feedback resonance intensity leading into raucous screaming self-oscillation"))
        .with_param(DspParamSchema::knob("lowpass_highpass_morph", "Filter Mode Lowpass/Highpass Morph", 0.0, 1.0, 0.0, "morph", MacroRole::Tone, "Continuous topology response morph: 0.0 = Lowpass, 1.0 = Highpass"))
        .with_param(DspParamSchema::knob("diode_saturation_threshold", "Diode Limiter Clamping Voltage", 0.1, 1.5, 0.65, "V", MacroRole::Punch, "Clipping threshold of internal feedback diodes taming runaway peaks"))
        .with_param(DspParamSchema::knob("pre_filter_gain", "Core Input Overdrive Gain", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Pre-filter stage overdrive gain saturating the discrete CA3080 OTA cores"))
        .with_param(DspParamSchema::knob("key_pitch_tracking", "Keyboard Note Pitch Tracking", 0.0, 1.0, 0.50, "track", MacroRole::Tone, "Scales cutoff corner frequency proportionally to musical note pitch"))
    );
    descriptors.insert("SpectralMultiNotchPhaseCombFilter".to_string(), DspNodeDescriptor::new("SpectralMultiNotchPhaseCombFilter", "Spectral Multi-Notch Phase Comb Filter", DspNodeCategory::FilterEq, "High-precision multi-stage allpass phase delay network producing intricate barber-pole phaser cancellation notches with bi-directional swept comb modulation")
        .with_param(DspParamSchema::knob("center_frequency", "Notch Bank Center Anchor Frequency", 50.0, 10000.0, 1200.0, "Hz", MacroRole::Tone, "Center anchoring frequency for the cascading allpass phase-shift stages"))
        .with_param(DspParamSchema::knob("notch_spread_octaves", "Notch Frequency Spread Width", 0.2, 4.0, 1.5, "oct", MacroRole::Character, "Logarithmic frequency distance between consecutive phase cancellation notches"))
        .with_param(DspParamSchema::knob("stage_cascade_depth", "Active Allpass Stage Count", 4.0, 24.0, 12.0, "stages", MacroRole::Character, "Number of cascaded 1st-order allpass phase shift stages in series"))
        .with_param(DspParamSchema::knob("feedback_phase_color", "Resonant Notch Feedback Amount", -0.95, 0.95, 0.60, "fb", MacroRole::Character, "Regenerative feedback deepening cancellation nulls into vocal peaks"))
        .with_param(DspParamSchema::knob("lfo_sweep_speed", "Triangular LFO Sweep Frequency", 0.02, 10.0, 0.45, "Hz", MacroRole::Character, "Rate of the internal LFO sweeping notch frequencies across the spectrum"))
        .with_param(DspParamSchema::knob("stereo_phase_quadrature", "Stereo L/R Sweep Phase Offset", 0.0, 180.0, 90.0, "deg", MacroRole::Space, "Phase difference between left and right channels creating wide panning phaser"))
    );
    descriptors.insert("NonlinearSteinerParkerSynth12dbFilter".to_string(), DspNodeDescriptor::new("NonlinearSteinerParkerSynth12dbFilter", "Nonlinear Steiner-Parker Synth 12dB Multi-Mode Filter", DspNodeCategory::FilterEq, "Diode-switched active multi-mode resonant filter reproducing the quirky aggressive saturation curves, self-oscillation howl, and mode morphing of the 1970s Synthasystem")
        .with_param(DspParamSchema::knob("cutoff_frequency_hz", "Filter Cutoff Tuning Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Two-pole state variable filter center cutoff frequency"))
        .with_param(DspParamSchema::knob("resonance_peak_q", "Steiner Resonance Feedback", 0.5, 25.0, 4.5, "res", MacroRole::Character, "Resonance gain pushing the diode core into screaming self-oscillation"))
        .with_param(DspParamSchema::knob("filter_topology_mode", "Multi-Mode (LP / BP / HP / Notch)", 0.0, 3.0, 0.0, "mode", MacroRole::Tone, "Continuous interpolation between Low-Pass (0), Band-Pass (1), High-Pass (2), and Notch (3)"))
        .with_param(DspParamSchema::knob("diode_drive_saturation", "Diode Switching Asymmetry Drive", 0.0, 1.0, 0.50, "drive", MacroRole::Punch, "Input drive level exciting the diode switching core into gritty harmonic distortion"))
        .with_param(DspParamSchema::knob("envelope_mod_depth", "Bi-Polar Envelope Mod Depth", -1.0, 1.0, 0.60, "env", MacroRole::Tone, "Modulation depth from external control envelope sweeping filter cutoff"))
        .with_param(DspParamSchema::knob("keytrack_pitch_scaling", "Keyboard Center Note Tracking", 0.0, 1.0, 1.0, "track", MacroRole::Tone, "Keyboard tracking ratio scaling cutoff frequency with played MIDI notes"))
    );
    descriptors.insert("MorphingSpectralFormantResonatorBank".to_string(), DspNodeDescriptor::new("MorphingSpectralFormantResonatorBank", "Morphing Spectral Formant Resonator Bank", DspNodeCategory::FilterEq, "Cascaded parallel 5-pole vocal formant filter bank with real-time vowel interpolation (A-E-I-O-U), vocal tract elongation, and nasal resonance cavity injection")
        .with_param(DspParamSchema::knob("vowel_morph_position", "Continuous Vowel Morph Coordinate", 0.0, 4.0, 0.0, "vowel", MacroRole::Tone, "Continuous vowel space morphing: 0 = A, 1 = E, 2 = I, 3 = O, 4 = U"))
        .with_param(DspParamSchema::knob("vocal_tract_gender_scale", "Vocal Tract Throat Length Scale", 0.5, 1.5, 1.0, "scale", MacroRole::Tone, "Geometric vocal tract scaling: < 1.0 = child/soprano, > 1.0 = deep baritone"))
        .with_param(DspParamSchema::knob("formant_bandwidth_q", "Resonance Formant Peak Sharpness", 2.0, 30.0, 8.0, "Q", MacroRole::Character, "Filter Q factor setting bandwidth sharpness of the 5 parallel resonant poles"))
        .with_param(DspParamSchema::knob("nasal_cavity_injection", "Nasal Cavity Resonant Antiformant", 0.0, 1.0, 0.25, "nasal", MacroRole::Character, "Secondary nasal cavity coupling introducing characteristic spectral antiformants"))
        .with_param(DspParamSchema::knob("harmonic_drive_overtone", "Excitation Non-Linear Saturation", 0.0, 1.0, 0.30, "drive", MacroRole::Punch, "Soft saturation before formant filters simulating vocal cord glottal drive"))
        .with_param(DspParamSchema::knob("filter_wet_dry_mix", "Formant Bank Parallel Output Mix", 0.0, 1.0, 1.0, "mix", MacroRole::Space, "Dry/wet balance between input signal and resonant formant output"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "FilterSVF" => Box::new(
            GenericDspNodeUi::new("FilterSVF", "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("mode", "Filter Topology", vec!["Lowpass 12dB".into(), "Highpass 12dB".into(), "Bandpass".into(), "Notch".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1500.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 10.0, 1.2, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("drive", "Pre-filter Drive", 0.0, 18.0, 0.0, "dB", (255, 61, 113)))
        ),
        "FilterLadder" => Box::new(
            GenericDspNodeUi::new("FilterLadder", "24dB/Oct Transistor Moog-Style Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 10.0, 1.5, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("drive", "Drive Saturation", 0.0, 24.0, 3.0, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("key_track", "Key Tracking", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
        ),
        "FilterLadder8" => Box::new(
            GenericDspNodeUi::new("FilterLadder8", "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1000.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 12.0, 2.0, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("pole_count", "Cascaded Poles", 2, 8, 8, "poles", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("saturation", "Non-linear Warmth", 0.0, 1.0, 0.4, "%", (255, 61, 113)))
        ),
        "FilterComb" => Box::new(
            GenericDspNodeUi::new("FilterComb", "Feedback Comb Resonator Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("delay_ms", "Delay Time", 0.1, 50.0, 5.0, "ms", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("feedback", "Feedback Loop", -0.99, 0.99, 0.7, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("damping", "High Damping", 0.0, 1.0, 0.2, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "FilterBiquad" => Box::new(
            GenericDspNodeUi::new("FilterBiquad", "Direct Form II Transposed Biquad Multi-Mode Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("filter_type", "Biquad Mode", vec!["Lowpass".into(), "Highpass".into(), "Bandpass".into(), "Notch".into(), "Peaking".into(), "LowShelf".into(), "HighShelf".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_log("cutoff", "Center / Cutoff Freq", 20.0, 20000.0, 1000.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("q_factor", "Filter Q Factor", 0.1, 20.0, 1.0, "Q", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("gain_db", "Band Boost / Cut", -24.0, 24.0, 0.0, "dB", (0, 230, 118)))
        ),
        "DcBlockFilter" => Box::new(
            GenericDspNodeUi::new("DcBlockFilter", "Zero-Phase DC Offset Removal Highpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("pole_radius", "Pole Radius (R)", 0.9, 0.9999, 0.995, "", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("highpass_hz", "Corner Highpass Freq", 1.0, 50.0, 10.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("enabled", "DC Block Active", true, (0, 230, 118)))
        ),
        "LowCutFilter" => Box::new(
            GenericDspNodeUi::new("LowCutFilter", "Precision Sub-Sonic Low-Cut Highpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff", "Low Cut Frequency", 10.0, 1000.0, 30.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("slope_order", "Filter Slope", vec!["6 dB/oct".into(), "12 dB/oct".into(), "18 dB/oct".into(), "24 dB/oct".into(), "48 dB/oct".into()], 1, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", (255, 171, 0)))
        ),
        "HighCutFilter" => Box::new(
            GenericDspNodeUi::new("HighCutFilter", "Anti-Aliasing High-Cut Lowpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff", "High Cut Frequency", 1000.0, 20000.0, 16000.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("slope_order", "Filter Slope", vec!["6 dB/oct".into(), "12 dB/oct".into(), "18 dB/oct".into(), "24 dB/oct".into(), "48 dB/oct".into()], 1, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", (255, 171, 0)))
        ),
        "ParametricEqNode" => Box::new(
            GenericDspNodeUi::new("ParametricEqNode", "4-Band Precision Parametric Equalizer", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("low_shelf", "Low Shelf (100Hz)", -18.0, 18.0, 0.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("low_mid", "Low Mid Gain", -18.0, 18.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("high_mid", "High Mid Gain", -18.0, 18.0, 0.0, "dB", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("high_shelf", "High Shelf (10kHz)", -18.0, 18.0, 0.0, "dB", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_log("mid_freq", "Parametric Mid Freq", 200.0, 8000.0, 1000.0, "Hz", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("mid_q", "Parametric Mid Q", 0.2, 10.0, 1.0, "Q", (0, 184, 217)))
        ),
        "FormantFilterNode" => Box::new(
            GenericDspNodeUi::new("FormantFilterNode", "Vowel Formant Filter Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("vowel", "Target Vowel", vec!["Vowel A (Ah)".into(), "Vowel E (Eh)".into(), "Vowel I (Ee)".into(), "Vowel O (Oh)".into(), "Vowel U (Oo)".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("formant_shift", "Formant Shift", -12.0, 12.0, 0.0, "st", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("throat_len", "Acoustic Throat Length", 0.5, 2.0, 1.0, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "Formant Resonance Q", 1.0, 20.0, 5.0, "Q", (0, 230, 118)))
        ),
        "ModalFilterNode" => Box::new(
            GenericDspNodeUi::new("ModalFilterNode", "High-Order Acoustic Modal Resonator Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("base_freq", "Fundamental Modal Freq", 50.0, 8000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("mode_count", "Active Modal Poles", 1, 16, 8, "modes", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("decay_rate", "Modal Decay Time", 0.05, 5.0, 1.2, "s", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("inharmonicity", "Inharmonic Mode Shift", 0.0, 2.0, 0.15, "", (0, 230, 118)))
        ),
        "HornReflectionFilter" => Box::new(
            GenericDspNodeUi::new("HornReflectionFilter", "Acoustic Bell Flare & Throat Reflection Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("bell_flare", "Bell Flare Expansion", 0.1, 3.0, 1.0, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("throat_diameter", "Throat Diameter", 5.0, 50.0, 15.0, "mm", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("reflection_coeff", "End Reflection Coefficient", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
        ),
        "AutomatedDynamicEqNode" => Box::new(
            GenericDspNodeUi::new("AutomatedDynamicEqNode", "AI-Driven Intelligent Masking Reduction Dynamic EQ", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("target_lufs", "Target Program LUFS", -24.0, -8.0, -14.0, "LUFS", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("low_dynamic_cut", "Dynamic Mud Cut (250Hz)", 0.0, 12.0, 3.0, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("high_air_lift", "Dynamic Air Presence (12kHz)", 0.0, 12.0, 2.5, "dB", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("adaptation_speed", "Neural Adaptation Speed", 10.0, 500.0, 100.0, "ms", (255, 171, 0)))
        ),
        "SubharmonicQuantumTunnelingFilter" => Box::new(
            GenericDspNodeUi::new("SubharmonicQuantumTunnelingFilter", "Subharmonic Quantum Tunneling Resonant Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("barrier_width_nm", "Quantum Barrier Width", 0.5, 10.0, 2.5, "nm", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("tunneling_prob", "Tunneling Probability", 0.01, 1.0, 0.35, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "Subharmonic Q Factor", 1.0, 30.0, 8.0, "Q", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_log("cutoff_hz", "Filter Cutoff Corner", 20.0, 12000.0, 800.0, "Hz", (0, 230, 118)))
        ),
        "MetamaterialRefractionFilter" => Box::new(
            GenericDspNodeUi::new("MetamaterialRefractionFilter", "Negative Index Acoustic Metamaterial Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("refractive_index", "Negative Index (n)", -5.0, -0.1, -1.4, "n", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("slab_thickness_mm", "Metamaterial Thickness", 1.0, 50.0, 12.0, "mm", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("evanescent_gain", "Evanescent Amplification", 0.0, 12.0, 3.5, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "Acoustic Bandgap Q", 1.0, 25.0, 6.0, "Q", (153, 102, 255)))
        ),
        "SpatialAcousticHologramFilter" => Box::new(
            GenericDspNodeUi::new("SpatialAcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_int("hologram_resolution", "Holographic Phase Steps", 16, 256, 64, "steps", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("phase_offset_rad", "Acoustic Phase Bias", 0.0, 6.28, 0.0, "rad", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("depth_focus_m", "Focal Distance", 0.2, 10.0, 2.0, "m", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("spatial_dispersion", "Diffractive Smear", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
        ),
        "LinearPhaseCrossoverNode" => Box::new(
            GenericDspNodeUi::new("LinearPhaseCrossoverNode", "4-Way Linear Phase Mastering Crossover Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("low_cross_hz", "Low Crossover Split", 40.0, 300.0, 120.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("mid_cross_hz", "Mid Crossover Split", 400.0, 3000.0, 1200.0, "Hz", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("high_cross_hz", "High Crossover Split", 3000.0, 16000.0, 6000.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_int("fir_tap_order", "FIR Filter Tap Count", 256, 4096, 1024, "taps", (0, 230, 118)))
        ),
        "CombResonatorNode" => Box::new(
            GenericDspNodeUi::new("CombResonatorNode", "Tuned Feedback Comb Filter Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("frequency_hz", "Comb Fundamental Pitch", 20.0, 5000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("feedback_gain", "Feedback Loop Resonance", -0.99, 0.99, 0.85, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("damping_ratio", "High Frequency Damping", 0.0, 1.0, 0.25, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Blend", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "HeadBumpFilter" => Box::new(
            GenericDspNodeUi::new("HeadBumpFilter", "Magnetic Tape Head-Bump Resonator & Low-End Peaking", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("speed_ips", "Tape Operational Speed", vec!["7.5 IPS (Warm 50Hz)".into(), "15 IPS (Punchy 80Hz)".into(), "30 IPS (HiFi 120Hz)".into()], 1, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("center_freq_hz", "Resonance Center", 30.0, 200.0, 80.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("q_factor", "Peaking Q", 0.5, 3.0, 1.2, "Q", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("boost_db", "Head Bump Boost", 0.0, 6.0, 2.5, "dB", (16, 185, 129)))
        ),
        "LinkwitzRiley4BandCrossover" => Box::new(
            GenericDspNodeUi::new("LinkwitzRiley4BandCrossover", "Linkwitz-Riley 4-Band 24dB/oct Phase-Flat Mastering Crossover", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("split_low_hz", "Low/Mid Crossover", 40.0, 500.0, 120.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("split_mid_hz", "Mid/High Crossover", 500.0, 4000.0, 2500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("split_air_hz", "High/Air Crossover", 4000.0, 16000.0, 8000.0, "Hz", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_bool("phase_align", "Linear Phase Summation", true, (16, 185, 129)))
        ),
        "FormantVowelMatrix" => Box::new(
            GenericDspNodeUi::new("FormantVowelMatrix", "5-Formant Acoustic Vowel Morphing Filter Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_formant_morph", "Vowel Formant Morph A-E-I-O-U", 0.0, 4.0, 0.0, "idx", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_size", "Vocal Tract Geometry Scale", 0.5, 2.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_resonance_q", "Formant Filter Resonance Q", 1.0, 30.0, 8.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("consonant_brightness", "Fricative Consonant Air", 0.0, 1.0, 0.35, "%", (16, 185, 129)))
        ),
        "SubHarmonicQuantumTunnelingFilter" => Box::new(
            GenericDspNodeUi::new("SubHarmonicQuantumTunnelingFilter", "Sub-Harmonic Quantum Tunneling Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("barrier_height", "Tunneling Potential Barrier V0", 0.1, 5.0, 1.2, "V", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("subharmonic_ratio", "Subharmonic Frequency Division", 0.1, 4.0, 1.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transmission_prob", "Tunneling Probability T", 0.01, 1.0, 0.45, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("harmonic_mix", "Subharmonic Wet Level", 0.0, 1.0, 0.30, "%", (99, 102, 241)))
        ),
        "CrystalModalFilter" => Box::new(
            GenericDspNodeUi::new("CrystalModalFilter", "Narrowband Piezoelectric Crystal Modal Resonant Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("center_freq", "Crystal Resonance Frequency", 50.0, 15000.0, 2400.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crystal_q", "Crystal Quality Factor Q", 10.0, 500.0, 85.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("piezo_gain", "Piezoelectric Coupling Gain", 0.0, 4.0, 1.2, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("damping_factor", "Crystal Mechanical Loss", 0.0001, 0.05, 0.005, "eta", (99, 102, 241)))
        ),
        "LinkwitzRiley4WaySplitter" => Box::new(
            GenericDspNodeUi::new("LinkwitzRiley4WaySplitter", "Linkwitz-Riley 4-Way Phase-Aligned Crossover Splitter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("crossover_freq_hz", "Crossover Frequency", 40.0, 16000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("phase_compensation", "Allpass Phase Alignment", true, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("branch_attenuation_db", "Crossfade Bleed", -12.0, 0.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("solo_split_branch", "Solo High Branch", false, (16, 185, 129)))
        ),
        "DispersionAllpassChain" => Box::new(
            GenericDspNodeUi::new("DispersionAllpassChain", "Multi-Stage Allpass Phase Dispersion Delay Ladder", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("dispersion_stages_count", "Cascaded Stages", 1.0, 32.0, 8.0, "stages", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("inharmonicity_coefficient_b", "Inharmonicity Factor B", 0.0001, 0.1, 0.012, "coeff", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dispersion_cutoff_hz", "Corner Break Freq", 100.0, 18000.0, 2400.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("delay_phase_warp", "Phase Warp Angle", 0.0, 1.0, 0.45, "warp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bypass_high_frequencies", "Bypass High Frequencies", false, (16, 185, 129)))
        ),
        "TineDispersionAllpass" => Box::new(
            GenericDspNodeUi::new("TineDispersionAllpass", "Electric Piano Tine Inharmonic Dispersion Allpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("tine_inharmonicity", "Tine Inharmonicity", 0.001, 0.5, 0.065, "disp", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("center_frequency_hz", "Center Frequency", 50.0, 8000.0, 880.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("phase_slope_q", "Phase Dispersion Q", 0.1, 10.0, 1.8, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("clamping_stiffness", "Mount Clamping Rigidity", 0.1, 2.0, 1.0, "stiff", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pickup_coupling_depth", "Pickup Flux Coupling", 0.0, 1.0, 0.70, "%", (16, 185, 129)))
        ),
        "SitarDispersionAllpass" => Box::new(
            GenericDspNodeUi::new("SitarDispersionAllpass", "Sitar String Stiff Dispersion Allpass Stage", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("string_stiffness_parameter", "String Stiffness B", 0.0001, 0.05, 0.008, "B", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("taraf_coupling_cutoff_hz", "Taraf Coupling Cutoff", 200.0, 12000.0, 3500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("non_linear_tension_mod", "Tension Modulation Depth", 0.0, 1.0, 0.40, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_freq_allpass_damping", "High Freq Dispersion Loss", 0.01, 1.0, 0.15, "loss", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("bridge_offset_phase", "Jawari Contact Angle", 0.0, 360.0, 45.0, "deg", (16, 185, 129)))
        ),
        "MetamaterialRefractionFilterNode" => Box::new(
            GenericDspNodeUi::new("MetamaterialRefractionFilterNode", "Negative-Index Acoustic Metamaterial Refractive Prism Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("center_frequency_hz", "Refraction Band Center", 100.0, 16000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("refractive_index", "Effective Acoustic Index n", -3.0, 3.0, -1.2, "n", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("unit_cell_resonance_q", "Unit-Cell Resonator Q", 1.0, 50.0, 12.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dispersion_steepness", "Spatial Dispersion Gradient", 0.1, 10.0, 2.5, "grad", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("evanescent_gain_db", "Evanescent Wave Amplification", -24.0, 24.0, 6.0, "dB", (16, 185, 129)))
        ),
        "Butterworth2ndOrder" => Box::new(
            GenericDspNodeUi::new("Butterworth2ndOrder", "2nd-Order Maximally Flat Butterworth Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("filter_type", "Filter Response Mode", vec!["Lowpass (12 dB/oct)".into(), "Highpass (12 dB/oct)".into(), "Allpass Phase Match".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("cutoff_frequency_hz", "Cutoff Frequency (-3 dB)", 10.0, 20000.0, 1000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("q_resonance", "Quality Factor (Q)", 0.1, 10.0, 0.707, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("gain_db", "Section Gain Offset", -24.0, 24.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bypass", "Hard Bypass Filter", false, (16, 185, 129)))
        ),
        "ModalFilterSection" => Box::new(
            GenericDspNodeUi::new("ModalFilterSection", "Narrowband Resonant Modal Filter Section", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("freq_hz", "Modal Peak Frequency", 20.0, 16000.0, 440.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("t60_sec", "Modal Ringdown Decay T60", 0.01, 20.0, 2.0, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("amplitude", "Modal Relative Amplitude", 0.0, 2.0, 1.0, "amp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase", "Initial Mode Phase", 0.0, 360.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("damping_scale", "Damping Coefficient Scale", 0.1, 5.0, 1.0, "scale", (16, 185, 129)))
        ),
        "DispersionAllpass" => Box::new(
            GenericDspNodeUi::new("DispersionAllpass", "1st-Order Inharmonic Phase Dispersion Allpass", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("coefficient", "Allpass Alpha Coefficient", -0.95, 0.95, 0.35, "α", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("frequency_warp_hz", "Dispersion Corner Frequency", 100.0, 10000.0, 2000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("inharmonicity_b", "Stiffness Inharmonicity B", 0.0001, 0.05, 0.005, "B", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_delay_samples", "Group Delay at DC", 0.1, 10.0, 1.2, "smp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bypass", "Bypass Dispersion", false, (16, 185, 129)))
        ),
        "ConsoleBiquad" => Box::new(
            GenericDspNodeUi::new("ConsoleBiquad", "Analog Console Desk Transformer Peaking EQ Biquad", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("freq_hz", "EQ Center Frequency", 10.0, 20000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("gain_db", "EQ Boost / Cut Gain", -24.0, 24.0, 3.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("q_factor", "Bandwidth Q Factor", 0.1, 10.0, 1.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("analog_warmth", "Non-linear Analog Warmth", 0.0, 1.0, 0.40, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bypass", "Bypass Equalizer Band", false, (16, 185, 129)))
        ),
        "VowelGraph" => Box::new(
            GenericDspNodeUi::new("VowelGraph", "Acoustic Vowel Formant Transition & Trajectory Graph", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("formant_f1_hz", "Formant F1 Frequency", 150.0, 1500.0, 500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("formant_f2_hz", "Formant F2 Frequency", 500.0, 4000.0, 1500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("vowel_position_x", "Vowel Chart Openness X", 0.0, 1.0, 0.50, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vowel_position_y", "Vowel Chart Backness Y", 0.0, 1.0, 0.50, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nasalization", "Velum Nasalization", 0.0, 1.0, 0.15, "%", (16, 185, 129)))
        ),
        "LadderFilterBode" => Box::new(
            GenericDspNodeUi::new("LadderFilterBode", "Moog 24dB/oct Diode/Transistor Ladder Filter Bode Plot HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff_freq_hz", "Moog Ladder Cutoff Frequency", 20.0, 20000.0, 1450.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "4-Pole Ladder Resonance Q", 0.0, 10.0, 6.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("drive_saturation", "Transistor Non-Linear Saturation Drive", 0.0, 100.0, 35.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("topology", "Circuit Topology Emulation", vec!["Moog Transistor 4-Pole".into(), "TB-303 Diode Ladder".into(), "SEM 2-Pole SVF".into(), "Korg MS-20 Sallen-Key".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("key_tracking_percent", "Keyboard Pitch Tracking Amount", 0.0, 100.0, 50.0, "%", (34, 197, 94)))
        ),
        "VowelSpace" => Box::new(
            GenericDspNodeUi::new("VowelSpace", "2D IPA International Phonetic Alphabet Vowel Space Formant HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("f1_hz", "Formant F1 Frequency (Vowel Height / Openness)", 200.0, 950.0, 280.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_log("f2_hz", "Formant F2 Frequency (Vowel Backness)", 600.0, 2600.0, 2250.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("f3_hz", "Formant F3 Frequency", 1800.0, 3200.0, 2500.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("vowel_selection", "Target IPA Vowel Anchor", vec!["Close Front [i]".into(), "Close Back [u]".into(), "Open Front [a]".into(), "Open Back [ɑ]".into(), "Mid Central Schwa [ə]".into(), "Near-Close [ɪ]".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("morph_slew_ms", "Formant Transition Glide Time", 5.0, 500.0, 45.0, "ms", (34, 197, 94)))
        ),
        "SpectralMorphEngine" => Box::new(
            GenericDspNodeUi::new("SpectralMorphEngine", "FFT Spectral Morphing & Cross-Synthesis Engine HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("morph_factor", "Spectral Magnitude Morph Position", 0.0, 1.0, 0.50, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("fft_window_size", "Short-Time Fourier Transform Window Size", vec!["256 Bins (Ultra Fast)".into(), "512 Bins (Transient)".into(), "1024 Bins (Balanced)".into(), "2048 Bins (High Res)".into(), "4096 Bins (Deep Lows)".into()], 2, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_db", "Spectral Envelope Spectral Tilt Balance", -12.0, 12.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("transient_bypass", "Transient Phase Preservation Bypass", true, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_enum("phase_lock_mode", "FFT Bin Phase Synchronization Mode", vec!["Strict Phase Lock".into(), "Randomized Diffusion".into(), "Minimum Phase Warping".into()], 0, (168, 85, 247)))
        ),
        "BesselFilterBank" => Box::new(
            GenericDspNodeUi::new("BesselFilterBank", "Bessel Linear-Phase Group-Delay Preserving Filter Bank HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_enum("filter_order", "Bessel Polynomial Filter Topology & Slope Steepness", vec!["2nd-Order (12dB/oct)".into(), "4th-Order (24dB/oct)".into(), "6th-Order (36dB/oct)".into(), "8th-Order (48dB/oct)".into()], 1, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("filter_mode", "Active Spectral Passband Filtering Response", vec!["Lowpass Transient Preserving".into(), "Highpass DC/Rumble Cleanse".into(), "Bandpass Resonance Band".into(), "Notch Band Reject".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_log("cutoff_frequency_hz", "Corner Cutoff Transition Frequency", 20.0, 20000.0, 2500.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("group_delay_flatness_factor", "Group Delay Flatness Slew Factor", 0.5, 2.0, 1.0, "factor", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("time_domain_overshoot_suppression", "Suppress Time-Domain Step-Response Overshoot / Ringing", true, (34, 197, 94)))
        ),
        "StateVariableOversampler" => Box::new(
            GenericDspNodeUi::new("StateVariableOversampler", "8x Polyphase Oversampling State-Variable Multi-Mode Filter HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff_frequency_hz", "Cutoff Corner Resonant Frequency", 20.0, 20000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "Self-Oscillating Resonance Peak Sharpness", 0.5, 25.0, 2.8, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("filter_output_blend", "State-Variable Multi-Mode Output Pole Selection", vec!["12dB/oct Lowpass".into(), "12dB/oct Highpass".into(), "6dB/oct Bandpass".into(), "Variable Notch Reject".into(), "Allpass Phase Disperser".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("polyphase_oversampling_factor", "Anti-Aliasing Polyphase FIR Oversampling Multiplier", vec!["1x (Native Sample Rate)".into(), "2x Low-Latency Polyphase".into(), "4x Studio Standard".into(), "8x Ultra Mastering Quality".into()], 3, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("analog_transistor_saturation", "Nonlinear Transistor Core Overdrive & Thermal Saturation", 0.0, 12.0, 1.5, "dB", (34, 197, 94)))
        ),
        "ParametricDynamicEq" => Box::new(
            GenericDspNodeUi::new("ParametricDynamicEq", "4-Band Dynamic Parametric Equalizer with Envelope Detectors HUD", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("band2_center_frequency_hz", "Mid-Band Dynamic Resonant Peak Frequency", 100.0, 10000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("band2_dynamic_range_db", "Maximum Dynamic Gain Expansion or Cut Attenuation", -18.0, 18.0, -6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("band2_threshold_db", "Dynamic Detector Level Activation Threshold", -48.0, 0.0, -18.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("band2_ratio", "Dynamic Compression/Expansion Slope Ratio", 1.0, 10.0, 2.5, ":1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("band2_attack_ms", "Transient Attack Smoothing Time Constant", 0.5, 100.0, 12.0, "ms", (34, 197, 94)))
        ),
        "EqBand" => Box::new(
            GenericDspNodeUi::new("EqBand", "Parametric Equalizer Biquad Filter Band", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("frequency_hz", "Center Frequency", 20.0, 20000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("gain_db", "Bell Gain Boost/Cut", -24.0, 24.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("q_factor", "Bandwidth Quality Factor Q", 0.1, 30.0, 1.414, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("filter_slope", "Filter Response Slope & Shape", vec!["Bell Peaking".into(), "Low Shelf".into(), "High Shelf".into(), "Bandpass".into(), "Notch".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamic_threshold_db", "Dynamic Expansion/Compression Threshold", -60.0, 0.0, -18.0, "dB", (34, 197, 94)))
        ),
        "AutoWahFilterMode" => Box::new(
            GenericDspNodeUi::new("AutoWahFilterMode", "Dynamic Auto-Wah State-Variable Filter & Envelope Sweep Engine", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_sweep_topology", "Auto-Wah Filter Sweep Topology", 0.0, 3.0, 0.0, "topo", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("envelope_sensitivity", "Input Level Envelope Detection Sensitivity", 0.1, 10.0, 2.5, "gain", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_resonance_q", "Filter Resonant Peak Sharpness Q", 0.5, 30.0, 8.0, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sweep_direction", "Envelope Cutoff Frequency Sweep Direction", 0.0, 2.0, 0.0, "dir", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("base_frequency_hz", "Base Filter Frequency at Zero Input", 100.0, 2500.0, 450.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("attack_release_speed_ms", "Envelope Follower Reaction Time", 5.0, 200.0, 25.0, "ms", (249, 115, 22)))
        ),
        "SpectralMorphMode" => Box::new(
            GenericDspNodeUi::new("SpectralMorphMode", "Frequency-Domain FFT Partial Morphing & Cross-Synthesis", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("spectral_interpolation_algorithm", "Spectral Frequency Morphing Algorithm", 0.0, 3.0, 1.0, "algo", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("morph_interpolation_factor", "Source A to Source B Morph Position", 0.0, 1.0, 0.5, "pos", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_balance", "High-to-Low Frequency Energy Tilt", -6.0, 6.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_center_frequency_shift", "Formant Envelope Pitch Scaling Offset", 0.5, 2.0, 1.0, "scale", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_smearing_time_ms", "Inter-Frame FFT Smearing & Ghosting", 0.0, 100.0, 15.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("lock_spectral_phase_coherence", "Enforce Instantaneous Phase Coherence", true, (14, 165, 233)))
        ),
        "AutoWahDirection" => Box::new(
            GenericDspNodeUi::new("AutoWahDirection", "Envelope Follower Auto-Wah Frequency Sweep Direction", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("sweep_direction_mode", "Envelope Follower Sweep Direction", 0.0, 3.0, 0.0, "dir", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("envelope_attack_time_ms", "Detector Attack Ballistics", 1.0, 100.0, 12.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("envelope_release_time_ms", "Detector Release Ballistics", 10.0, 1000.0, 180.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("filter_resonance_q", "Peak Resonance Q Sharpness", 1.0, 25.0, 8.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("envelope_sensitivity_db", "Input Dynamic Sensitivity Threshold", -40.0, 10.0, -14.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("invert_sweep_polarities", "Invert Sweep Polarities on High Transients", false, (14, 165, 233)))
        ),
        "FilterType" => Box::new(
            GenericDspNodeUi::new("FilterType", "Universal Biquad Direct-Form II Transposed Filter Topology Engine", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_topology_type", "Biquad Filter Transfer Mode", 0.0, 6.0, 0.0, "mode", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cutoff_center_frequency_hz", "Cutoff / Center Resonant Frequency", 20.0, 20000.0, 1000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("resonance_q_factor", "Resonance Quality Q Factor / Bandwidth", 0.1, 30.0, 0.707, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("peaking_shelf_gain_db", "Bell Peaking / Shelf Boost Gain", -24.0, 24.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("saturation_drive_amount", "Internal Core Saturation Drive", 0.0, 1.0, 0.15, "drv", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_double_precision_64bit_eval", "64-Bit Double Precision Internal Float DSP", true, (14, 165, 233)))
        ),
        "DcOffsetRemoverFilter" => Box::new(
            GenericDspNodeUi::new("DcOffsetRemoverFilter", "Subsonic Zero-Phase DC Offset Eliminator & High-Pass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "High-Pass Cutoff Frequency", 1.0, 30.0, 10.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("filter_slope_steepness", "Filter Slope Steepness", 0.0, 3.0, 1.0, "slope", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("linear_phase_mode", "Linear-Phase Zero Group Delay", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dc_tracker_time_constant_sec", "DC Tracking Time Constant", 0.1, 5.0, 1.0, "s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bipolar_asymmetry_correction", "Correct Asymmetric Waveforms", true, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("infrasonic_energy_meter", "Display Infrasonic Energy Readout", true, (14, 165, 233)))
        ),
        "MasterLinearPhaseEq" => Box::new(
            GenericDspNodeUi::new("MasterLinearPhaseEq", "64-Bit Linear-Phase Mastering Equalizer & Brickwall Crossover", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_bands_count", "Active Linear Phase Bands", 2.0, 12.0, 8.0, "bands", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("fir_impulse_length_taps", "FIR Convolution Tap Resolution", 0.0, 3.0, 2.0, "taps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossover_phase_coupling", "Adjacent Band Phase Coupling", 0.0, 1.0, 1.0, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pre_ringing_suppression_db", "Pre-Ringing Transient Suppression", -48.0, 0.0, -24.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("analog_phase_morph_ratio", "Minimum-to-Linear Phase Morph", 0.0, 1.0, 0.0, "morph", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_high_precision_64bit", "64-Bit Float Double Precision Processing", true, (14, 165, 233)))
        ),
        "DynamicResonanceSuppressor" => Box::new(
            GenericDspNodeUi::new("DynamicResonanceSuppressor", "Spectral Auto-Tracking Dynamic Resonance Suppressor & De-Ringer", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("spectral_detection_sensitivity", "Resonance Detection Sensitivity", 0.1, 1.0, 0.65, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("suppression_depth_db", "Maximum Dynamic Suppression Depth", -24.0, 0.0, -6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sharpness_q_selectivity", "Filter Bandwidth Selectivity Q", 1.0, 50.0, 12.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("attack_response_time_ms", "Detector Attack Response Time", 0.5, 50.0, 5.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("release_recovery_time_ms", "Detector Release Recovery Time", 5.0, 500.0, 60.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("monitor_delta_audition", "Audition Suppressed Delta Signal Solo", false, (14, 165, 233)))
        ),
        "SpectralDeHumFilter" => Box::new(
            GenericDspNodeUi::new("SpectralDeHumFilter", "Adaptive AC Mains Hum & Harmonic Buzz Eliminator Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("mains_base_frequency_hz", "AC Mains Fundamental Frequency", 0.0, 2.0, 1.0, "base", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_notch_count", "Tracked Harmonic Notches Count", 1.0, 32.0, 16.0, "harmonics", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("notch_bandwidth_q", "Surgical Notch Filter Q-Factor", 10.0, 100.0, 40.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("attenuation_notch_depth_db", "Hum Notch Attenuation Depth", -80.0, -12.0, -48.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("dynamic_harmonic_tracking", "Dynamic Frequency Drift Tracking", true, (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("high_pass_ground_lift_hz", "Sub-Bass Highpass Ground Lift", 10.0, 60.0, 20.0, "Hz", (14, 165, 233)))
        ),
        "NonlinearJawariBridge" => Box::new(
            GenericDspNodeUi::new("NonlinearJawariBridge", "Sitar Non-Linear Curved Jawari Bridge & Tarab Sympathetic Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("bridge_curvature_profile", "Jawari Parabolic Curvature Saddle", 0.0, 3.0, 1.0, "curve", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("jawari_buzz_contact_depth", "Grazing String Buzz Contact Depth", 0.0, 1.0, 0.75, "depth", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("raga_scale_tuning", "Sympathetic Tarab Raga Tuning Scale", 0.0, 6.0, 0.0, "raga", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tarab_sympathetic_coupling", "Tarab Sympathetic Acoustic Transfer", 0.0, 1.0, 0.65, "gain", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("kaddu_gourd_air_resonance", "Kaddu Gourd Helmholtz Air Cavity", 80.0, 400.0, 142.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("tabli_spruce_wood_warmth", "Flat Spruce Tabli Soundboard Warmth", 0.0, 1.0, 0.8, "warmth", (14, 165, 233)))
        ),
        "CassottoToneChamber" => Box::new(
            GenericDspNodeUi::new("CassottoToneChamber", "Cassotto Wooden Tone Chamber & Soundbox Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("chamber_acoustic_profile", "Cassotto Tone Chamber Geometry Profile", 0.0, 4.0, 0.0, "profile", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cassotto_warmth_factor", "Low-Mid Cavity Formant Warmth", 0.0, 1.0, 0.82, "warmth", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_harmonic_attenuation_db", "Upper Metallic Overtones Attenuation", -36.0, 0.0, -18.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("chamber_volume_liters", "Internal Chamber Enclosure Volume", 0.5, 8.0, 2.4, "L", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("reedblock_air_coupling", "Reedblock to Chamber Air Coupling", 0.1, 1.0, 0.7, "ratio", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("box_wood_resonance_q", "Cavity Air Resonance Q Factor", 1.0, 10.0, 3.5, "Q", (14, 165, 233)))
        ),
        "WoodwindToneholeGrid" => Box::new(
            GenericDspNodeUi::new("WoodwindToneholeGrid", "Woodwind Tonehole Lattice Acoustic Scattering Junction", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("active_fingering_pattern", "Acoustic Woodwind Fingering State", 0.0, 4.0, 0.0, "fingering", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tonehole_chimney_height_mm", "Tonehole Chimney Height / Wall", 1.0, 8.0, 2.5, "mm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("tonehole_inner_radius_mm", "Tonehole Aperture Inner Radius", 1.5, 8.0, 4.5, "mm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("radiation_loss_damping", "Open Tonehole Acoustic Radiation Damping", 0.0, 1.0, 0.45, "loss", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("end_correction_ratio", "Aperture Acoustic End Correction", 0.2, 1.5, 0.61, "delta", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("bore_thermal_viscous_losses", "Bore Wall Thermal Viscous Loss", 0.01, 0.5, 0.08, "loss/m", (14, 165, 233)))
        ),
        "AcousticHornReflection" => Box::new(
            GenericDspNodeUi::new("AcousticHornReflection", "Acoustic Horn Bell Impedance Matcher & Flare Radiator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("horn_flare_geometry", "Acoustic Flare Geometry Curve", 0.0, 3.0, 0.0, "geom", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bell_opening_radius_cm", "Bell Aperture Mouth Radius", 2.0, 40.0, 12.5, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("horn_axial_length_cm", "Horn Axial Flare Length", 10.0, 200.0, 65.0, "cm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("flare_cutoff_frequency_hz", "Calculated Radiation Cutoff Freq", 100.0, 1500.0, 380.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("radiation_directivity_factor", "High-Frequency Axial Directivity", 0.0, 1.0, 0.7, "dir", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("viscous_throat_wall_loss", "Throat Wall Viscous Dissipation", 0.01, 0.5, 0.06, "loss", (14, 165, 233)))
        ),
        "SoundboardBridgeCoupler" => Box::new(
            GenericDspNodeUi::new("SoundboardBridgeCoupler", "Spruce Soundboard Downbearing Bridge Impedance Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("spruce_grain_orientation", "Orthotropic Spruce Plate Grain Profile", 0.0, 3.0, 1.0, "grain", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bridge_downbearing_angle_deg", "Bridge Downbearing Break Angle", 0.5, 6.0, 2.2, "deg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("soundboard_rib_stiffening", "Perpendicular Rib Stiffening Profile", 0.1, 1.0, 0.65, "stiff", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("plate_modal_density_cutoff_hz", "Plate Modal Transition Frequency", 100.0, 1200.0, 450.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("bridge_admittance_mobility", "Bridge Mechanical Mobility Admittance", 0.01, 0.8, 0.18, "mob", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("soundboard_rim_clamping_stiffness", "Rim Perimeter Boundary Clamping", 0.1, 1.0, 0.75, "clamp", (14, 165, 233)))
        ),
        "SpectralTransientIsolator" => Box::new(
            GenericDspNodeUi::new("SpectralTransientIsolator", "High-Resolution STFT Transient/Tonal Splitter & Isolator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("transient_threshold_sensitivity", "Transient Detection Sensitivity", 0.1, 1.0, 0.55, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("separation_smoothness_ms", "Spectral Mask Temporal Smoothing", 0.5, 50.0, 8.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_fft_resolution", "STFT Transform Window Resolution", 0.0, 3.0, 1.0, "res", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_boost_db", "Isolated Transient Channel Gain", -24.0, 18.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("tonal_decay_preservation", "Tonal Resonance Sustain Preservation", 0.0, 1.0, 0.85, "pres", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("audition_transient_channel_only", "Audition Separated Transient Solo", false, (14, 165, 233)))
        ),
        "BesselLinearPhaseCrossover" => Box::new(
            GenericDspNodeUi::new("BesselLinearPhaseCrossover", "Linear-Phase Bessel Multi-Band Crossover Splitter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("crossover_split_frequency_hz", "Band Crossover Split Frequency", 60.0, 8000.0, 450.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("fir_tap_order", "FIR Impulse Kernel Resolution", 0.0, 3.0, 1.0, "taps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stopband_attenuation_db", "Stopband Rejection Attenuation", -96.0, -24.0, -60.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_band_phase_alignment_deg", "Crossover Band Phase Alignment", -180.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transient_smear_damping", "Pre-Ringing Smear Damping Factor", 0.0, 1.0, 0.7, "damp", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_band_phase_inversion", "Invert Split Band Polarity", false, (14, 165, 233)))
        ),
        "DynamicPhaseRotator" => Box::new(
            GenericDspNodeUi::new("DynamicPhaseRotator", "Vocal Crest-Factor Phase Rotation & Asymmetry Optimizer", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("center_rotation_frequency_hz", "Phase Rotation Center Frequency", 50.0, 1000.0, 160.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("allpass_cascade_stages", "Allpass Filter Cascaded Stages", 0.0, 3.0, 1.0, "stages", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("phase_angle_shift_deg", "Manual Phase Shift Angle", 0.0, 180.0, 90.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("dynamic_pitch_tracking_lock", "Pitch-Tracked Center Frequency Lock", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("highpass_rumble_cutoff_hz", "Sub-Sonic DC Rumble Highpass Cutoff", 10.0, 80.0, 30.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("peak_crest_factor_reduction_db", "Target Crest Factor Headroom Gain", 0.5, 6.0, 2.5, "dB", (14, 165, 233)))
        ),
        "ResonantCombLatticeFilter" => Box::new(
            GenericDspNodeUi::new("ResonantCombLatticeFilter", "Multi-Tap Resonant Comb Lattice & Allpass Dispersion Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("fundamental_comb_tuning_hz", "Lattice Fundamental Tuning", 20.0, 2000.0, 146.83, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("feedback_comb_gain", "Lattice Feedback Gain", -0.99, 0.99, 0.88, "fb", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("allpass_dispersion_stages", "Allpass Delay Dispersion Stages", 0.0, 3.0, 1.0, "stg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("damping_high_frequency_hz", "Internal High-Cut Loss Damping", 500.0, 20000.0, 6500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_comb_cross_coupling", "Stereo Channel Cross-Coupling", 0.0, 1.0, 0.35, "cross", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("feedforward_ratio", "Feedforward Direct Interference", -1.0, 1.0, 0.5, "ff", (14, 165, 233)))
        ),
        "LinearPhaseMultibandSplitter" => Box::new(
            GenericDspNodeUi::new("LinearPhaseMultibandSplitter", "5-Band Zero-Phase FIR Mastering Crossover Splitter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("crossover_1_sub_bass_hz", "Band 1/2 Sub-Bass Split Frequency", 40.0, 200.0, 90.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossover_2_low_mid_hz", "Band 2/3 Low-Mid Split Frequency", 150.0, 800.0, 350.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossover_3_mid_high_hz", "Band 3/4 Mid-High Split Frequency", 800.0, 4000.0, 2400.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crossover_4_air_band_hz", "Band 4/5 High Air Split Frequency", 4000.0, 16000.0, 8500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fir_filter_taps_count", "Linear-Phase FIR Filter Kernel Length", 0.0, 3.0, 2.0, "taps", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("solo_active_band_channel", "Band Monitoring & Audition Solo", 0.0, 5.0, 0.0, "solo", (14, 165, 233)))
        ),
        "BilinearDiodeLadderFilter" => Box::new(
            GenericDspNodeUi::new("BilinearDiodeLadderFilter", "4-Pole Bilinear Transform Non-Linear Diode Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_log("cutoff_frequency_hz", "Filter Cutoff Frequency", 20.0, 20000.0, 850.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_amount", "Feedback Resonance Intensity", 0.0, 1.0, 0.65, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_thermal_voltage_mv", "Diode Junction Thermal Voltage", 15.0, 45.0, 26.0, "mV", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diode_saturation_drive_db", "Diode Ladder Input Overdrive", 0.0, 24.0, 4.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("slope_topology_mode", "Ladder Pole Configuration", 0.0, 3.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_zdf_anti_cramping", "Zero-Delay Feedback Anti-Cramping", true, (14, 165, 233)))
        ),
        "PiezoelectricCrystalFilter" => Box::new(
            GenericDspNodeUi::new("PiezoelectricCrystalFilter", "High-Q Quartz Crystal Lattice Bandpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("crystal_center_freq_hz", "Motional Series Resonant Frequency", 100.0, 18000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("motional_q_factor", "Quartz Crystal Mechanical Quality Factor Q", 100.0, 50000.0, 5000.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("parallel_resonance_offset_cents", "Anti-Resonant Pole Frequency Offset", 5.0, 200.0, 35.0, "cents", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crystal_electrode_capacitance_pf", "Holder Electrostatic Shunt Capacitance", 1.0, 50.0, 7.5, "pF", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("drive_level_nonlinearity", "High-Drive Crystal Nonlinear Elasticity", 0.0, 1.0, 0.25, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("temperature_drift_ppm", "Ambient Temperature Frequency Drift", -50.0, 50.0, 0.0, "ppm", (14, 165, 233)))
        ),
        "PhaseAlignedStereoCrossover" => Box::new(
            GenericDspNodeUi::new("PhaseAlignedStereoCrossover", "All-Pass Phase-Compensated Linear Crossover Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("low_crossover_freq_hz", "Sub/Low Splitting Crossover Frequency", 40.0, 250.0, 90.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("mid_crossover_freq_hz", "Low-Mid/High-Mid Crossover Frequency", 250.0, 2500.0, 800.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_crossover_freq_hz", "High-Mid/Treble Crossover Frequency", 2500.0, 12000.0, 4500.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_slope_steepness", "Crossover Filter Slope Order", 0.0, 2.0, 1.0, "slope", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("delay_phase_fine_trim_us", "Sub-Sample Inter-Band Phase Alignment Trim", -500.0, 500.0, 0.0, "us", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_allpass_phase_matching", "Cascaded All-Pass Group Delay Equalization", true, (14, 165, 233)))
        ),
        "BilinearStateVariableMorphFilter" => Box::new(
            GenericDspNodeUi::new("BilinearStateVariableMorphFilter", "Zero-Delay Feedback State-Variable Morphing Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Filter Corner Cutoff Frequency", 10.0, 22000.0, 1500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_q_factor", "Resonance Peak Quality Factor (Q)", 0.5, 30.0, 2.8, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("morph_crossfade_ratio", "Continuous Morph Interpolation Ratio", 0.0, 1.0, 0.0, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("drive_saturation_gain", "Nonlinear Diode Feedback Drive", 1.0, 10.0, 1.5, "drv", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("keytrack_pitch_scaling", "Keyboard Pitch Tracking Amount", 0.0, 2.0, 1.0, "amt", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_analog_saturation", "Model Diode Soft Clipping Non-Linearity", true, (14, 165, 233)))
        ),
        "HarmonicResonanceCombBank" => Box::new(
            GenericDspNodeUi::new("HarmonicResonanceCombBank", "64-Tap Tuned Harmonic Comb Filter Resonator Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("fundamental_frequency_hz", "Base Harmonic Series Pitch", 20.0, 2000.0, 110.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("comb_decay_time_seconds", "Resonant Ringing Decay Time", 0.05, 10.0, 1.5, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_damping_hz", "Feedback High Frequency Damping", 500.0, 20000.0, 6000.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("odd_even_harmonic_balance", "Odd vs Even Harmonic Tooth Balance", -1.0, 1.0, 0.0, "bal", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_comb_detune_cents", "Stereo Comb Spread Detuning", 0.0, 50.0, 8.0, "cents", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("bipolar_comb_phase_invert", "Invert Feedback Polarity (Peak/Notch)", false, (14, 165, 233)))
        ),
        "LinearPhaseMultibandIsolator" => Box::new(
            GenericDspNodeUi::new("LinearPhaseMultibandIsolator", "5-Band FIR Linear-Phase Frequency Isolator & Crossover", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("crossover_low_hz", "Sub / Bass Crossover Frequency", 40.0, 400.0, 120.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crossover_low_mid_hz", "Low-Mid / Mid Crossover Frequency", 200.0, 2000.0, 600.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("crossover_high_mid_hz", "Mid / High-Mid Crossover Frequency", 1000.0, 8000.0, 3200.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("crossover_high_hz", "Presence / Air Crossover Frequency", 4000.0, 16000.0, 9000.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fir_filter_order_taps", "FIR Filter Kernel Length Mode", 0.0, 2.0, 1.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_pre_ringing_suppression", "Psychoacoustic Pre-Ringing Suppression", true, (14, 165, 233)))
        ),
        "MicrotonalSpectralResonatorBank" => Box::new(
            GenericDspNodeUi::new("MicrotonalSpectralResonatorBank", "MTS-ESP Microtonal Spectral Resonator Filter Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("fundamental_anchor_pitch_hz", "Concert Root Tuning Pitch (A4)", 20.0, 1000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonator_q_bandwidth", "Resonant Tooth Quality Factor (Q)", 2.0, 200.0, 45.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("resonance_ringdown_decay_seconds", "Harmonic Resonator Ringdown Time", 0.1, 15.0, 2.5, "s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tuning_scale_selection", "Microtonal Scale Mode", 0.0, 5.0, 0.0, "scale", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("excitation_input_gain_db", "Excitation Signal Drive Gain", -24.0, 12.0, 0.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_mts_esp_realtime_tuning_sync", "MTS-ESP Dynamic Tuning Broadcast Sync", true, (14, 165, 233)))
        ),
        "BesselFilterArrayEqualizer" => Box::new(
            GenericDspNodeUi::new("BesselFilterArrayEqualizer", "Maximally Flat Linear Delay Bessel Filter Array EQ", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("low_shelf_cutoff_frequency_hz", "Bessel Low-Shelf Corner Frequency", 20.0, 500.0, 80.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("low_shelf_gain_boost_db", "Bessel Low-Shelf Gain Adjustment", -18.0, 18.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("parametric_mid_frequency_hz", "Bessel Mid-Band Center Frequency", 200.0, 8000.0, 1200.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("parametric_mid_gain_db", "Bessel Mid-Band Gain Adjustment", -18.0, 18.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_shelf_cutoff_frequency_hz", "Bessel High-Shelf Corner Frequency", 2000.0, 20000.0, 8000.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_analog_phase_matching", "Analog Group Delay Symmetrical Match", true, (14, 165, 233)))
        ),
        "SallenKeyResonantFilter" => Box::new(
            GenericDspNodeUi::new("SallenKeyResonantFilter", "Discrete Sallen-Key 12 dB/Oct Dual-Pole Resonant Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_cutoff_frequency_hz", "Sallen-Key Cutoff Corner Frequency", 15.0, 22000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_q_factor", "Filter Resonance Quality Factor (Q)", 0.5, 25.0, 2.5, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("op_amp_saturation_drive", "Op-Amp Slew & Voltage Clipping Drive", 1.0, 10.0, 1.2, "drv", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_key_tracking_amount", "MIDI Note Pitch Cutoff Tracking", 0.0, 2.0, 1.0, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("filter_response_mode", "Filter Response Topology", 0.0, 2.0, 0.0, "mode", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_soft_diode_feedback_clip", "Diode Clipper Feedback Stabilization", true, (14, 165, 233)))
        ),
        "StateVariableMorphLadderFilter" => Box::new(
            GenericDspNodeUi::new("StateVariableMorphLadderFilter", "Cascaded Transistor Ladder & 4-Pole State-Variable Hybrid Morphing Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Master Filter Cutoff Frequency", 10.0, 22000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_q_feedback", "Resonant Peak Q Feedback Ratio", 0.5, 25.0, 2.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_topology_morph_ratio", "Continuous Topology Mode Morph", 0.0, 3.0, 0.0, "mode", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transistor_saturation_drive", "Transistor Ladder Input Overdrive", 0.0, 1.0, 0.25, "drive", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("self_oscillation_damping", "Self-Oscillation Peak Limiter Damping", 0.0, 1.0, 0.6, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_delay_feedback", "Zero-Delay Feedback (ZDF) Trapezoidal Solver", true, (14, 165, 233)))
        ),
        "SpectralPeakTrackingFilter" => Box::new(
            GenericDspNodeUi::new("SpectralPeakTrackingFilter", "Autonomous Fundamental & Spectral Peak Tracking Adaptive Dynamic Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_sensitivity", "Pitch Tracker Detector Sensitivity", 0.1, 1.0, 0.85, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tracking_bandwidth_q", "Bandwidth Resonance Q Factor", 1.0, 30.0, 8.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("peak_boost_cut_gain_db", "Tracked Frequency Boost / Cut Gain", -18.0, 18.0, 4.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tracking_glide_speed_ms", "Pitch Tracking Slewing Time", 1.0, 200.0, 25.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("harmonic_tracking_order", "Tracked Harmonic Overtone Target Mode", 0.0, 3.0, 0.0, "ord", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("freeze_tracked_frequency", "Freeze Current Center Frequency", false, (14, 165, 233)))
        ),
        "ResonantDiodeLadderFilter" => Box::new(
            GenericDspNodeUi::new("ResonantDiodeLadderFilter", "TB-303 Style 18/24 dB Diode Ladder Lowpass Filter with Saturation", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Diode Ladder Cutoff Frequency", 20.0, 18000.0, 1500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_q", "Acid Squelch Resonance Q Peak", 0.5, 30.0, 4.5, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_conduction_saturation", "Diode Conduction Soft Saturation", 0.0, 1.0, 0.45, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("highpass_bass_bleed_compensation", "Resonance Low-End Pass-Through Bleed", 0.0, 1.0, 0.3, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("envelope_modulation_depth", "Filter Cutoff Envelope Modulation", -1.0, 1.0, 0.6, "amt", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_exponential_thermal_drift", "Analog Thermal Component Drift", true, (14, 165, 233)))
        ),
        "MultibandMidSideEqualizer" => Box::new(
            GenericDspNodeUi::new("MultibandMidSideEqualizer", "5-Band Precision Mastering Mid/Side Minimum Phase EQ", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("mid_low_shelf_gain_db", "Mid Channel Sub Bass Shelf Gain", -12.0, 12.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("side_high_shelf_gain_db", "Side Channel Air Band Shelf Gain", -12.0, 12.0, 1.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("mid_presence_bell_hz", "Mid Channel Vocal Presence Frequency", 1000.0, 8000.0, 3200.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("side_presence_bell_hz", "Side Channel Width Focus Frequency", 1000.0, 8000.0, 5500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("mono_low_cutoff_hz", "Mono Low Bass Guard Cutoff Frequency", 20.0, 200.0, 75.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("solo_side_channel_monitoring", "Solo Side Channel Stereo Difference", false, (14, 165, 233)))
        ),
        "BiquadFormantMorphFilter" => Box::new(
            GenericDspNodeUi::new("BiquadFormantMorphFilter", "3-Vowel Morphing Triple-Biquad Vocal Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_morph_xy_coordinate", "2D Formant Vowel Coordinate (A-E-I-O-U)", 0.0, 4.0, 1.2, "pos", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_q_bandwidth_focus", "Formant Peak Resonant Sharpness (Q)", 2.0, 35.0, 12.0, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_anti_resonance_db", "Nasal Cavity Anti-Resonant Notch Depth", -24.0, 0.0, -6.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("throat_vocal_tract_length_cm", "Acoustic Vocal Tract Length", 10.0, 24.0, 17.5, "cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("morph_transition_slew_ms", "Vowel Transition Slewing Speed", 5.0, 300.0, 45.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_parallel_formant_routing", "Parallel vs Series Filter Architecture", true, (14, 165, 233)))
        ),
        "LinearPhaseTiltEqualizer" => Box::new(
            GenericDspNodeUi::new("LinearPhaseTiltEqualizer", "Zero-Phase 180-Degree Pivot Spectral Balance & Sub Sentry", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("tilt_pivot_frequency_hz", "Spectral Tilt Center Pivot Frequency", 200.0, 3000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_slope_db_oct", "Spectral Tilt Slope Angle", -6.0, 6.0, 1.5, "dB/oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_acoustic_infrasound_cutoff_hz", "Infrasonic Sub-Bass High-Pass Cutoff", 15.0, 60.0, 25.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("air_band_lift_gain_db", "Ultra-High Air Band Radiance Shelf", -6.0, 12.0, 2.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fir_filter_order_taps", "Linear-Phase FIR Convolution Length Mode", 0.0, 2.0, 1.0, "taps", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("bypass_phase_linearization", "Minimum Phase Low-Latency Mode", false, (14, 165, 233)))
        ),
        "ResonantMoogLadderDualCutoff" => Box::new(
            GenericDspNodeUi::new("ResonantMoogLadderDualCutoff", "Dual-Slope Transistor Ladder Filter with Asymmetric Diode Saturation", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("ladder_cutoff_frequency_hz", "Transistor Ladder Cutoff Frequency", 15.0, 22000.0, 850.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_intensity", "Ladder Self-Oscillation Resonance", 0.0, 4.0, 1.8, "res", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("passband_gain_loss_compensation", "Bass Passband Drop Compensation", 0.0, 1.0, 0.75, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ladder_drive_saturation_db", "Input Differential Amplifier Drive", 0.0, 24.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("slope_steepness_db_oct", "Ladder Slope Filter Order Mode", 0.0, 2.0, 2.0, "pole", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_self_oscillation_limiter", "Soft-Saturating Feedback Peak Limiter", true, (14, 165, 233)))
        ),
        "MetamaterialNegativeIndexFilter" => Box::new(
            GenericDspNodeUi::new("MetamaterialNegativeIndexFilter", "Negative Effective Bulk Modulus Acoustic Metamaterial Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("negative_modulus_center_freq_hz", "Metamaterial Negative Modulus Center", 100.0, 15000.0, 1250.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("metamaterial_bandgap_width_cents", "Acoustic Bandgap Rejection Bandwidth", 50.0, 2400.0, 450.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("effective_refractive_index", "Effective Acoustic Refractive Index (n)", -5.0, -0.1, -1.2, "n", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("subwavelength_labyrinth_cells", "Labyrinth Unit Cell Cascades", 2.0, 16.0, 6.0, "cells", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("evanescent_wave_amplification_db", "Evanescent Subwavelength Amplification", 0.0, 24.0, 6.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_backward_phase_propagation", "Backward Wave Phase Propagation Mode", true, (14, 165, 233)))
        ),
        "AutomatedDynamicPsychoacousticEq" => Box::new(
            GenericDspNodeUi::new("AutomatedDynamicPsychoacousticEq", "Automated Psychoacoustic Spectral Masking & Collision Dynamic EQ", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("masking_detection_sensitivity", "Critical Bark Masking Detection Sens", 0.1, 2.0, 1.0, "sens", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("dynamic_carve_max_cut_db", "Maximum Dynamic Spectral Cut Depth", 0.0, 18.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("critical_band_bark_resolution", "Auditory Filter Bank Bark Resolution", 1.0, 24.0, 12.0, "bands", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dynamic_attack_recovery_ms", "Dynamic Carving Attack & Recovery Time", 5.0, 500.0, 65.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sidechain_unmask_listening_focus", "External Sidechain Unmask Priority", 0.0, 1.0, 0.5, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_equal_loudness_fletcher_munson", "ISO 226 Fletcher-Munson Equal Loudness", true, (14, 165, 233)))
        ),
        "AsymmetricClippingDiodeLadder" => Box::new(
            GenericDspNodeUi::new("AsymmetricClippingDiodeLadder", "Asymmetric Germanium Diode Ladder Resonant Lowpass Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("ladder_resonance_q", "Four-Pole Feedback Resonance", 0.0, 10.0, 3.5, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_drive_saturation", "Diode Stage Non-Linear Saturation", 0.0, 24.0, 8.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("germanium_forward_voltage_drop", "Diode Forward Voltage Drop (Vf)", 0.1, 0.8, 0.28, "V", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("thermal_drift_detuning_cents", "Diode Junction Thermal Drift", 0.0, 50.0, 5.0, "cents", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_self_oscillation_limiting", "Limit Self-Oscillation Peak Amplitude", true, (14, 165, 233)))
        ),
        "CavityResonanceFormantFilter" => Box::new(
            GenericDspNodeUi::new("CavityResonanceFormantFilter", "Coupled Acoustic Cavity Multi-Chamber Formant Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cavity_f1_pharyngeal_cutoff_hz", "Pharyngeal Cavity Formant F1", 150.0, 1200.0, 500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cavity_f2_oral_chamber_cutoff_hz", "Oral Cavity Formant F2", 500.0, 3500.0, 1800.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cavity_f3_nasal_coupling_cutoff_hz", "Nasal Apex Formant F3", 1800.0, 6000.0, 2800.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("inter_cavity_acoustic_coupling", "Inter-Chamber Acoustic Impedance Coupling", 0.0, 1.0, 0.45, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_wall_damping_loss", "Compliant Wall Flesh Absorption Loss", 0.01, 0.99, 0.18, "loss", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_nasal_velum_port_bypass", "Nasal Port Velum Anti-Resonances", false, (14, 165, 233)))
        ),
        "ResonantPhasedCombArray" => Box::new(
            GenericDspNodeUi::new("ResonantPhasedCombArray", "Stereo Cross-Feedback Phased Comb Resonator Array", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("comb_pitch_fundamental_hz", "Comb Tooth Fundamental Frequency", 20.0, 4000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("feedback_resonance_amount", "Regenerative Comb Feedback Ratio", -0.99, 0.99, 0.88, "fb", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_cross_coupling_ratio", "Stereo Cross-Coupled Channel Feedback", 0.0, 1.0, 0.35, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_damping_cutoff_hz", "Feedback High-Frequency Damping", 1000.0, 20000.0, 8500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fractional_delay_detune_cents", "Stereo Delay Fractional Detune Offset", 0.0, 100.0, 12.0, "ct", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_allpass_phase_dispersion_chain", "Allpass Phase Dispersive Bell Cascade", true, (14, 165, 233)))
        ),
        "NonlinearWaveguideFeedbackComb" => Box::new(
            GenericDspNodeUi::new("NonlinearWaveguideFeedbackComb", "Nonlinear Waveguide Feedback Comb Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("comb_delay_time_ms", "Resonant Comb Delay Base Period", 0.1, 100.0, 12.5, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("feedback_saturation_gain_db", "Feedback Nonlinear Saturation Drive", 0.0, 24.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("damping_lowpass_cutoff_hz", "Feedback Damping Lowpass Cutoff", 200.0, 20000.0, 6500.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("cross_channel_phase_inversion", "Stereo Anti-Phase Cross-Feedback", 0.0, 1.0, 0.25, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("cubic_nonlinearity_blend", "Odd Harmonic Cubic Distortion Ratio", 0.0, 1.0, 0.4, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_dc_blocking_filter", "Feedback DC-Blocking Highpass", true, (14, 165, 233)))
        ),
        "MultiResonanceFormantBank" => Box::new(
            GenericDspNodeUi::new("MultiResonanceFormantBank", "8-Pole Parallel Acoustic Formant Filter Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_phoneme_morph", "Anthropomorphic Vowel Phoneme Morph", 0.0, 5.0, 1.0, "vowel", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_bandwidth_q", "Formant Peak Resonance Sharpness", 1.0, 30.0, 8.5, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Global Formant Transposition", -24.0, 24.0, 0.0, "st", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_coupling_ratio", "Nasal Cavity Velum Anti-Resonance", 0.0, 1.0, 0.3, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_even_odd_blend", "Even vs Odd Formant Harmonic Bias", -1.0, 1.0, 0.0, "bal", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_linear_phase_anti_comb", "Linear-Phase Inter-Pole Phase Alignment", true, (14, 165, 233)))
        ),
        "ZeroPhaseLinearMasteringEqualizer" => Box::new(
            GenericDspNodeUi::new("ZeroPhaseLinearMasteringEqualizer", "8-Band Zero-Phase Linear-Phase Mastering EQ", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("fir_impulse_length_samples", "Linear Phase FIR Partition Window", 1024.0, 16384.0, 8192.0, "smp", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pre_ringing_suppression_pct", "Psychoacoustic Pre-Echo Suppressor", 0.0, 100.0, 75.0, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_low_band_gain_db", "Band 1 Sub-Bass Precision Gain", -18.0, 18.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("mid_presence_band_gain_db", "Band 4 Vocal Presence Center Gain", -18.0, 18.0, 0.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_air_band_gain_db", "Band 8 Ultra-High Air Silk Gain", -18.0, 18.0, 1.2, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_dual_mono_stereo_unlink", "Dual-Mono Channel Independent Processing", false, (14, 165, 233)))
        ),
        "MorphingStateVariableLadderFilter" => Box::new(
            GenericDspNodeUi::new("MorphingStateVariableLadderFilter", "Morphing State-Variable Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Filter Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_emphasis_q", "Resonant Peak Resonance Q", 0.5, 30.0, 3.5, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_topology_morph_position", "Topology Morph (LP -> BP -> HP -> Notch)", 0.0, 3.0, 0.0, "mode", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("resonance_diode_saturation_drive", "Diode Feedback Soft-Clipper Drive", 0.0, 18.0, 4.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("slope_steepness_poles", "Filter Attenuation Slope (6/12/18/24 dB/oct)", 1.0, 4.0, 4.0, "poles", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_analog_drift_warmth", "Thermal Component Drift Warmth", true, (14, 165, 233)))
        ),
        "FerrofluidMagneticViscosityAbsorber" => Box::new(
            GenericDspNodeUi::new("FerrofluidMagneticViscosityAbsorber", "Ferrofluid Magnetic Core Variable-Q Notch Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("electromagnet_flux_density_tesla", "Magnetic Field Flux Density", 0.0, 2.5, 0.85, "T", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nanoparticle_carrier_viscosity_cst", "Carrier Fluid Kinematic Viscosity", 1.0, 100.0, 25.0, "cSt", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_center_frequency_hz", "Active Resonant Center Frequency", 50.0, 16000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("magnetic_shear_damping_q", "Magnetic Rheological Q Resonance", 0.5, 40.0, 8.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_notch_attenuation_depth_db", "Rejection Notch Attenuation Depth", -60.0, 0.0, -36.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_magnetic_hysteresis_loop", "Magnetic Core Hysteresis Saturation", true, (34, 197, 94)))
        ),
        "NonlinearDiodeLadderStereoFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearDiodeLadderStereoFilter", "Vintage EMS Diode Ladder Filter with Voltage Sag", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1400.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_self_oscillation", "Diode Ladder Q Resonance", 0.0, 1.2, 0.75, "res", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diode_thermal_voltage_mv", "Diode Junction Thermal Voltage Ut", 20.0, 35.0, 26.0, "mV", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("input_overdrive_saturation_db", "Input Preamp Diode Clipping Drive", -12.0, 36.0, 6.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("power_rail_voltage_sag_pct", "Power Rail Dynamic Voltage Sag", 0.0, 1.0, 0.3, "%", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_stereo_spread_offset", "Dual Channel Stereo Cutoff Spread", true, (34, 197, 94)))
        ),
        "AdaptiveFormantMorphingResonator" => Box::new(
            GenericDspNodeUi::new("AdaptiveFormantMorphingResonator", "Vocal Tract Vowel Space Morphing Formant Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_phoneme_position_x", "Front-to-Back Vowel Position (X)", -1.0, 1.0, 0.0, "pos", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vowel_tongue_height_position_y", "Open-to-Close Tongue Height (Y)", -1.0, 1.0, 0.35, "pos", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_bandwidth_q_scaling", "Resonant Formant Peak Q Sharpness", 0.5, 20.0, 6.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_anti_resonance_hz", "Velopharyngeal Nasal Antizero", 800.0, 3000.0, 1400.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("throat_formant_depth_db", "Subglottal Throat Resonance Gain", 0.0, 24.0, 12.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_vocal_tract_lip_radiation", "Spherical Lip Aperture Radiation", true, (34, 197, 94)))
        ),
        "ResonantCapacitorSwitchedLadderFilter" => Box::new(
            GenericDspNodeUi::new("ResonantCapacitorSwitchedLadderFilter", "Switched-Capacitor Discrete Analog Emulation Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_cutoff_clock_frequency_hz", "Switching Master Clock Cutoff", 20.0, 20000.0, 2200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("switched_capacitor_charge_q", "Resonance Feedback Q Factor", 0.5, 30.0, 8.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("clock_leakage_feedthrough_db", "MOSFET Clock Feedthrough Leakage", -80.0, -12.0, -42.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("charge_transfer_nonlinearity", "Nonlinear Switch On-Resistance", 0.01, 1.0, 0.22, "ron", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_damping", "High-Frequency Self-Oscillation Damping", 0.01, 0.5, 0.08, "loss", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_discrete_charge_injection", "Dynamic Charge Injection Modeling", true, (34, 197, 94)))
        ),
        "MorphingEllipticCauerLadderFilter" => Box::new(
            GenericDspNodeUi::new("MorphingEllipticCauerLadderFilter", "Morphing Elliptic / Cauer Active Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_cutoff_frequency_hz", "Elliptic Filter Cutoff Frequency", 20.0, 20000.0, 1800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("passband_equiripple_db", "Passband Ripple Toleration", 0.01, 3.0, 0.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("stopband_attenuation_floor_db", "Stopband Rejection Floor", 24.0, 96.0, 60.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("resonance_pole_q_factor", "Cutoff Resonance Peak Q Factor", 0.5, 25.0, 4.5, "Q", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("morph_filter_topology_type", "Filter Response Shape Morph (LP to HP)", 0.0, 2.0, 0.0, "type", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_delay_feedback", "Zero-Delay Feedback (ZDF) Solvers", true, (34, 197, 94)))
        ),
        "DualPeakFormantVowelBiquadFilter" => Box::new(
            GenericDspNodeUi::new("DualPeakFormantVowelBiquadFilter", "Dual-Peak Vocal Vowel Formant Biquad Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("formant_f1_frequency_hz", "Formant F1 Frequency (Jaw Opening)", 200.0, 1200.0, 650.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_f2_frequency_hz", "Formant F2 Frequency (Tongue Shape)", 500.0, 3500.0, 1800.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_q_sharpness_factor", "Formant Resonant Peak Sharpness Q", 1.0, 30.0, 8.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vowel_crossfade_morph_index", "Vowel Preset Morph (A-E-I-O-U)", 0.0, 4.0, 0.0, "morph", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("throat_resonance_saturation_db", "Subglottic Throat Drive Saturation", 0.0, 24.0, 5.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_speech_lip_radiation_boost", "High-Frequency Lip Radiation (+6dB)", true, (34, 197, 94)))
        ),
        "SpectralFormantMorphingVowelMatrix" => Box::new(
            GenericDspNodeUi::new("SpectralFormantMorphingVowelMatrix", "Multi-Vowel 5-Formant Dynamic Morphing Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_xy_space_interpolation_x", "Vowel Space Dimension X (Front-Back)", 0.0, 1.0, 0.35, "pos", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vowel_xy_space_interpolation_y", "Vowel Space Dimension Y (High-Low)", 0.0, 1.0, 0.65, "pos", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("larynx_tract_length_scaling", "Vocal Tract Physical Length Scale", 0.7, 1.4, 1.0, "scale", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_bandwidth_q_scaling", "Formant Resonance Bandwidth Q", 0.5, 8.0, 2.8, "Q", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_anti_resonance_depth_db", "Velopharyngeal Nasal Anti-Pole Depth", -24.0, 0.0, -8.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_formant_glottal_coupling", "Dynamic Fundamental Pitch Coupling", true, (34, 197, 94)))
        ),
        "MultibandSpectralDeEsserDynamic" => Box::new(
            GenericDspNodeUi::new("MultibandSpectralDeEsserDynamic", "Linear-Phase Dynamic Sibilance Spectral De-Esser", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("sibilance_detection_center_hz", "De-Esser Detection Frequency Center", 3000.0, 12000.0, 6800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamic_reduction_threshold_db", "Sibilance Suppression Threshold", -36.0, 0.0, -18.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("attenuation_range_limit_db", "Maximum De-Essing Attenuation Depth", -24.0, 0.0, -9.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sibilance_bandwidth_q_width", "Detection Filter Bandwidth Q Factor", 0.5, 5.0, 1.6, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("deesser_attack_lookahead_ms", "Lookahead Transient Buffer Delay", 0.0, 10.0, 2.5, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_listen_sibilance_delta_mode", "Listen Sibilance Delta Solo Audition", false, (34, 197, 94)))
        ),
        "MorphingStateVariableLadderCombFilter" => Box::new(
            GenericDspNodeUi::new("MorphingStateVariableLadderCombFilter", "Hybrid ZDF SVF & Resonant Comb Filter Matrix", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_cutoff_frequency_hz", "ZDF Filter Master Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_peak_q_factor", "State Variable Resonance Peak (Q)", 0.5, 25.0, 3.8, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("comb_delay_frequency_tuning_hz", "Feedback Comb Resonant Pitch Tuning", 20.0, 5000.0, 240.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("comb_recirculation_feedback", "Comb Loop Resonance Feedback Ratio", -0.99, 0.99, 0.78, "fdbk", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("svf_to_comb_morph_blend", "SVF to Comb Filter Morphology Blend", 0.0, 1.0, 0.5, "morph", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_filter_nonlinear_saturation", "ZDF Resonant Non-linear Core Saturation", true, (34, 197, 94)))
        ),
        "DualSvfMorphingResonanceEqualizer" => Box::new(
            GenericDspNodeUi::new("DualSvfMorphingResonanceEqualizer", "Dual Parallel State-Variable Morphing Equalizer", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("filter_a_cutoff_frequency_hz", "Filter A Center Cutoff Frequency", 20.0, 20000.0, 480.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("filter_b_cutoff_frequency_hz", "Filter B Center Cutoff Frequency", 20.0, 20000.0, 2800.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("filter_a_morph_topology", "Filter A Response Topology Morph", 0.0, 3.0, 0.0, "topo", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("filter_b_morph_topology", "Filter B Response Topology Morph", 0.0, 3.0, 2.0, "topo", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_cross_resonance_coupling", "Filter A/B Cross-Resonance Coupling Feedback", 0.0, 1.0, 0.28, "coup", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_stereo_anti_phase_spread", "Stereo Anti-Phase Cutoff Panning Spread", true, (34, 197, 94)))
        ),
        "PassivePultecInductorProgramEqualizer" => Box::new(
            GenericDspNodeUi::new("PassivePultecInductorProgramEqualizer", "Passive Inductor-Capacitor Vintage Program Equalizer", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("low_shelf_frequency_select_hz", "Low-End Stepped Frequency Selector", 20.0, 100.0, 60.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("low_shelf_boost_gain_db", "Simultaneous Low-Frequency Boost Amount", 0.0, 16.0, 5.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("low_shelf_attenuate_gain_db", "Simultaneous Low-Frequency Dip Attenuation", 0.0, 16.0, 4.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_peak_bandwidth_q", "Toroidal Inductor High-Frequency Peak Q", 0.5, 10.0, 2.5, "Q", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("high_peak_boost_gain_db", "Toroidal Inductor High Bell Peak Boost", 0.0, 18.0, 4.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_inter_winding_transformer_core", "Triad Input/Output Transformer Magnetics", true, (34, 197, 94)))
        ),
        "MorphingLadderSallenKeyHybridFilter" => Box::new(
            GenericDspNodeUi::new("MorphingLadderSallenKeyHybridFilter", "Hybrid ZDF Moog Ladder & Sallen-Key State Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("master_cutoff_frequency_hz", "Master Cutoff Corner Frequency", 15.0, 22000.0, 1800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_intensity", "Self-Oscillation Resonance Feedback Amount", 0.0, 1.05, 0.68, "res", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("ladder_to_sallen_morph_ratio", "Transistor Ladder vs Sallen-Key Topology Morph", 0.0, 1.0, 0.35, "morph", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("diode_limiter_saturation_drive", "Feedback Diode Symmetrical Limiter Drive", 0.1, 8.0, 1.8, "drive", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("four_pole_bass_compensation", "Resonance Bass Drop Makeup Compensation", 0.0, 1.0, 0.5, "comp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_analog_thermal_frequency_drift", "Transistor Junction Thermal Drift Simulation", true, (34, 197, 94)))
        ),
        "DualResonantMorphingEllipticFilter" => Box::new(
            GenericDspNodeUi::new("DualResonantMorphingEllipticFilter", "Cauer Elliptic ZDF Dual-Resonant Equal-Ripple Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Elliptic Filter Cutoff Corner Frequency", 20.0, 20000.0, 1500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stopband_attenuation_depth_db", "Stopband Minimum Attenuation Floor", 24.0, 96.0, 60.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("passband_equiripple_peak_db", "Passband Equiripple Resonant Peak Height", 0.1, 6.0, 1.2, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dual_pole_resonance_split_octaves", "Dual Resonant Peaks Spectral Separation", 0.0, 4.0, 1.5, "oct", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("lowpass_to_highpass_morph_blend", "Lowpass to Highpass Topology Continuous Morph", 0.0, 1.0, 0.0, "morph", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_nonlinear_zdf_saturation", "ZDF Integrator Diode Clipping Feedback Loop", true, (34, 197, 94)))
        ),
        "ParametricStateVariableFormantPeakFilter" => Box::new(
            GenericDspNodeUi::new("ParametricStateVariableFormantPeakFilter", "Parametric Vocal Formant Peak State-Variable Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_coordinate_x_front_back", "Vocal Vowel Trajectory Front/Back Position (F2)", 0.0, 1.0, 0.45, "pos", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vowel_coordinate_y_open_closed", "Vocal Vowel Trajectory Open/Closed Jaw (F1)", 0.0, 1.0, 0.65, "open", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_length_scale_factor", "Anatomical Vocal Tract Length Scale (Throat Size)", 0.7, 1.5, 1.0, "scale", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_bandwidth_q_sharpness", "Formant Resonant Bandwidth Sharpness (Q)", 2.0, 30.0, 12.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_glottal_breath_drive_db", "Glottal Whisper Breath Harmonic Drive", 0.0, 18.0, 3.5, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_subharmonic_laryngeal_growl", "Laryngeal Ventricular Band Subharmonic Growl", false, (34, 197, 94)))
        ),
        "MorphingButterworthChebyshevHybridFilter" => Box::new(
            GenericDspNodeUi::new("MorphingButterworthChebyshevHybridFilter", "ZDF Morphing Butterworth / Chebyshev Hybrid Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Master Filter Cutoff Corner Frequency", 20.0, 20000.0, 1200.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("butterworth_to_chebyshev_morph", "Filter Polynomial Characteristic Morph", 0.0, 1.0, 0.0, "morph", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("chebyshev_passband_ripple_db", "Chebyshev Passband Equiripple Depth", 0.05, 3.0, 0.5, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_resonance_q_peak", "Resonance Feedback Quality Factor (Q)", 0.5, 25.0, 2.5, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_slope_steepness_poles", "Filter Order Pole Count (12/24/36 dB/oct)", 2.0, 6.0, 4.0, "poles", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_diode_ladder_saturation", "Nonlinear Sallen-Key Diode Limiting Feedback", true, (34, 197, 94)))
        ),
        "FormantVocalTractNasalBranchResonator" => Box::new(
            GenericDspNodeUi::new("FormantVocalTractNasalBranchResonator", "Biomechanical Vocal Tract Nasal Branch Resonator", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("velum_opening_aperture_ratio", "Velopharyngeal Port Velum Opening Ratio", 0.0, 1.0, 0.45, "velum", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_volume_scale", "Nasal Cavity Acoustic Volume Scaling", 0.5, 2.0, 1.0, "scale", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("oral_tract_tongue_position_x", "Oral Tract Tongue Front/Back Position", 0.0, 1.0, 0.5, "pos", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("oral_tract_jaw_openness_y", "Oral Tract Jaw Mandible Openness", 0.0, 1.0, 0.6, "open", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("nasal_wall_mucous_absorption_damping", "Nasal Mucosa Viscous Acoustic Absorption", 0.1, 2.0, 0.85, "damp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_turbulent_nasal_fricative_air", "Nasal Airflow Turbulence & Fricative Whistle", false, (34, 197, 94)))
        ),
        "NonlinearMoogTransistorLadder24dbFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearMoogTransistorLadder24dbFilter", "ZDF 4-Pole 24 dB/oct Transistor Ladder Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("ladder_cutoff_frequency_hz", "Ladder Filter Cutoff Frequency", 20.0, 20000.0, 1800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_amount", "Ladder Resonance Self-Oscillation (Q)", 0.0, 4.2, 2.8, "res", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transistor_tanh_saturation_drive", "Differential Pair Tanh Overdrive", 0.0, 24.0, 8.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("passband_bass_drop_compensation", "Resonance Bass Drop Loss Compensation", 0.0, 1.0, 0.65, "comp", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("thermal_drift_voltage_mv", "Ladder Semiconductor Thermal Drift", -10.0, 10.0, 0.5, "mV", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_smooth_resonance_limiting", "Nonlinear Feedback Diode Resonance Limiter", true, (34, 197, 94)))
        ),
        "MorphingFormantVowelBandpassFilterBank" => Box::new(
            GenericDspNodeUi::new("MorphingFormantVowelBandpassFilterBank", "5-Formant Phonetic Vowel Resonator Filter Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_phoneme_morph_position", "Vowel Phoneme Morph (A - E - I - O - U)", 0.0, 4.0, 1.0, "pos", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_frequency_shift_semitones", "Formant Frequency Transpose Shift", -12.0, 12.0, 0.0, "st", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_resonance_q_bandwidth", "Formant Resonator Peak Sharpness (Q)", 2.0, 30.0, 12.0, "Q", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("throat_depth_sub_formant_db", "Sub-Glottal Throat Resonance Boost", 0.0, 12.0, 3.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_gender_scaling", "Vocal Tract Anatomy Scale (Child to Bass)", 0.6, 1.4, 1.0, "scale", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_dynamic_nasal_coupling", "Secondary Nasal Cavity Coupling Pole", true, (34, 197, 94)))
        ),
        "NonlinearDiodeLadderZdf24dbLowpassFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearDiodeLadderZdf24dbLowpassFilter", "Nonlinear Diode Ladder ZDF 24dB/oct Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Ladder Cutoff Corner Frequency", 20.0, 20000.0, 1200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback_q", "Diode Ladder Resonance Feedback (Q)", 0.5, 12.0, 4.2, "Q", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_saturation_thermal_voltage_mv", "Diode Junction Thermal Voltage (Vt)", 15.0, 45.0, 26.0, "mV", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("passband_bass_drop_compensation", "Passband Low-End Punch Compensation", 0.0, 1.0, 0.6, "comp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("input_overdrive_drive_gain_db", "Input Diode Stage Drive Amplification", 0.0, 24.0, 6.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_adaa_second_order_antialiasing", "Second-Order ADAA Anti-Aliasing", true, (34, 197, 94)))
        ),
        "MorphingSpectralVowelResonanceFilter" => Box::new(
            GenericDspNodeUi::new("MorphingSpectralVowelResonanceFilter", "3-Formant Morphing Vowel Resonance Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_interpolation_x", "Horizontal Formant Coordinate (Front-Back)", 0.0, 1.0, 0.25, "X", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vowel_interpolation_y", "Vertical Formant Coordinate (Open-Closed)", 0.0, 1.0, 0.5, "Y", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_length_scaling", "Vocal Tract Length Scaling Factor", 0.7, 1.4, 1.0, "scale", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_resonance_q_sharpness", "Formant Peak Q Sharpness / Articulation", 2.0, 25.0, 8.5, "Q", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("nasal_coupling_bandwidth_hz", "Velum Nasal Cavity Anti-Resonance Width", 50.0, 500.0, 180.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_formant_jitter_vibrato", "Organic Vocal Fold Jitter / Vibrato", true, (34, 197, 94)))
        ),
        "NonlinearSallenKeyActive12dbFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearSallenKeyActive12dbFilter", "Nonlinear Sallen-Key Active 12dB Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff", "Filter Resonant Cutoff Frequency", 10.0, 22000.0, 1200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_q", "Feedback Resonance Q Factor", 0.5, 25.0, 4.2, "Q", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("opamp_slew_rate", "Op-Amp Slew Rate Limit", 0.1, 50.0, 8.0, "V/us", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_feedback_clip", "Feedback Diode Soft-Clipping", 0.0, 1.0, 0.35, "sat", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("filter_mode", "Filter State Low/High Morph", 0.0, 1.0, 0.0, "morph", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("input_drive", "Core Preamplifier Input Drive", 0.0, 24.0, 3.0, "dB", (34, 197, 94)))
        ),
        "SpectralCombFilterBankFormantMorpher" => Box::new(
            GenericDspNodeUi::new("SpectralCombFilterBankFormantMorpher", "Spectral Comb Filter Bank Formant Morpher", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("formant_vowel", "Vowel Formant Morph Target", 0.0, 4.0, 0.0, "vowel", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_feedback", "Comb Peak Feedback Intensity", 0.0, 0.99, 0.88, "fb", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("comb_spacing", "Harmonic Comb Spacing Ratio", 0.5, 2.0, 1.0, "ratio", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking", "Key Pitch Tracking Amount", 0.0, 1.0, 0.50, "track", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("damping_hf", "Comb Loop High Damping", 1000.0, 20000.0, 9500.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dry_wet_mix", "Formant Resonator Wet / Dry", 0.0, 1.0, 0.70, "mix", (34, 197, 94)))
        ),
        "NonlinearKorg35StateVariable24dbFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearKorg35StateVariable24dbFilter", "Nonlinear Korg 35 State Variable 24dB Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency", "Filter Cutoff Corner Frequency", 15.0, 22000.0, 950.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("resonance_peak", "Feedback Screaming Resonance Peak", 0.0, 1.0, 0.75, "peak", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lowpass_highpass_morph", "Filter Mode Lowpass/Highpass Morph", 0.0, 1.0, 0.0, "morph", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("diode_saturation_threshold", "Diode Limiter Clamping Voltage", 0.1, 1.5, 0.65, "V", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("pre_filter_gain", "Core Input Overdrive Gain", 0.0, 24.0, 6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("key_pitch_tracking", "Keyboard Note Pitch Tracking", 0.0, 1.0, 0.50, "track", (34, 197, 94)))
        ),
        "SpectralMultiNotchPhaseCombFilter" => Box::new(
            GenericDspNodeUi::new("SpectralMultiNotchPhaseCombFilter", "Spectral Multi-Notch Phase Comb Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("center_frequency", "Notch Bank Center Anchor Frequency", 50.0, 10000.0, 1200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("notch_spread_octaves", "Notch Frequency Spread Width", 0.2, 4.0, 1.5, "oct", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stage_cascade_depth", "Active Allpass Stage Count", 4.0, 24.0, 12.0, "stages", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("feedback_phase_color", "Resonant Notch Feedback Amount", -0.95, 0.95, 0.60, "fb", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lfo_sweep_speed", "Triangular LFO Sweep Frequency", 0.02, 10.0, 0.45, "Hz", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stereo_phase_quadrature", "Stereo L/R Sweep Phase Offset", 0.0, 180.0, 90.0, "deg", (34, 197, 94)))
        ),
        "NonlinearSteinerParkerSynth12dbFilter" => Box::new(
            GenericDspNodeUi::new("NonlinearSteinerParkerSynth12dbFilter", "Nonlinear Steiner-Parker Synth 12dB Multi-Mode Filter", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("cutoff_frequency_hz", "Filter Cutoff Tuning Frequency", 20.0, 20000.0, 1000.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonance_peak_q", "Steiner Resonance Feedback", 0.5, 25.0, 4.5, "res", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_topology_mode", "Multi-Mode (LP / BP / HP / Notch)", 0.0, 3.0, 0.0, "mode", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diode_drive_saturation", "Diode Switching Asymmetry Drive", 0.0, 1.0, 0.50, "drive", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("envelope_mod_depth", "Bi-Polar Envelope Mod Depth", -1.0, 1.0, 0.60, "env", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("keytrack_pitch_scaling", "Keyboard Center Note Tracking", 0.0, 1.0, 1.0, "track", (34, 197, 94)))
        ),
        "MorphingSpectralFormantResonatorBank" => Box::new(
            GenericDspNodeUi::new("MorphingSpectralFormantResonatorBank", "Morphing Spectral Formant Resonator Bank", DspNodeCategory::FilterEq)
                .with_param(DspParamDescriptor::new_linear("vowel_morph_position", "Continuous Vowel Morph Coordinate", 0.0, 4.0, 0.0, "vowel", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("vocal_tract_gender_scale", "Vocal Tract Throat Length Scale", 0.5, 1.5, 1.0, "scale", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_bandwidth_q", "Resonance Formant Peak Sharpness", 2.0, 30.0, 8.0, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nasal_cavity_injection", "Nasal Cavity Resonant Antiformant", 0.0, 1.0, 0.25, "nasal", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("harmonic_drive_overtone", "Excitation Non-Linear Saturation", 0.0, 1.0, 0.30, "drive", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("filter_wet_dry_mix", "Formant Bank Parallel Output Mix", 0.0, 1.0, 1.0, "mix", (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
