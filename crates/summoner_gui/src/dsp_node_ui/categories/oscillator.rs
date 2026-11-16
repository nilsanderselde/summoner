// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `Oscillator`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("OscSine".to_string(), DspNodeDescriptor::new("OscSine", "Pure Sine Wave Oscillator", DspNodeCategory::Oscillator, "Pure Sine Wave Oscillator")
        .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Phase Offset parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
    );
    descriptors.insert("OscSaw".to_string(), DspNodeDescriptor::new("OscSaw", "Band-limited Sawtooth Oscillator", DspNodeCategory::Oscillator, "Band-limited Sawtooth Oscillator")
        .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Phase Offset parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
        .with_param(DspParamSchema::knob("bandlimit", "Bandlimit Order", 1.0, 8.0, 4.0, "", MacroRole::Character, "Bandlimit Order integer control"))
    );
    descriptors.insert("OscPulse".to_string(), DspNodeDescriptor::new("OscPulse", "Pulse Width Modulation Oscillator", DspNodeCategory::Oscillator, "Pulse Width Modulation Oscillator")
        .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("pulse_width", "Pulse Width", 0.01, 0.99, 0.5, "%", MacroRole::Tone, "Pulse Width parameter"))
        .with_param(DspParamSchema::knob("pwm_rate", "PWM LFO Rate", 0.1, 20.0, 1.0, "Hz", MacroRole::Tone, "PWM LFO Rate parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
    );
    descriptors.insert("OscTriangle".to_string(), DspNodeDescriptor::new("OscTriangle", "Anti-aliased Triangle Generator", DspNodeCategory::Oscillator, "Anti-aliased Triangle Generator")
        .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("symmetry", "Symmetry Ramp", 0.01, 0.99, 0.5, "%", MacroRole::Tone, "Symmetry Ramp parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
    );
    descriptors.insert("OscWavetable".to_string(), DspNodeDescriptor::new("OscWavetable", "SIMD Wavetable Morphing Synthesizer", DspNodeCategory::Oscillator, "SIMD Wavetable Morphing Synthesizer")
        .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Morph Position parameter"))
        .with_param(DspParamSchema::choice("table", "Table Bank", &["Basic Shapes", "Harmonics", "Formants", "Metallic"], 0, "Table Bank mode selector"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
    );
    descriptors.insert("SimdPolyWavetableOscillator".to_string(), DspNodeDescriptor::new("SimdPolyWavetableOscillator", "AVX2/NEON Polyphonic Wavetable Voice Bank", DspNodeCategory::Oscillator, "AVX2/NEON Polyphonic Wavetable Voice Bank")
        .with_param(DspParamSchema::knob("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Morph Position parameter"))
        .with_param(DspParamSchema::knob("unison_detune", "Unison Detune", 0.0, 50.0, 8.0, "cents", MacroRole::Tone, "Unison Detune parameter"))
        .with_param(DspParamSchema::knob("unison_voices", "Voice Count", 1.0, 16.0, 4.0, "voices", MacroRole::Character, "Voice Count integer control"))
        .with_param(DspParamSchema::knob("sub_osc_gain", "Sub Osc Level", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Sub Osc Level parameter"))
        .with_param(DspParamSchema::knob("stereo_spread", "Stereo Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Stereo Spread parameter"))
    );
    descriptors.insert("GranularSynthNode".to_string(), DspNodeDescriptor::new("GranularSynthNode", "Cloud Granular Audio Streamer", DspNodeCategory::Oscillator, "Cloud Granular Audio Streamer")
        .with_param(DspParamSchema::knob("density", "Grain Density", 1.0, 100.0, 25.0, "grains/s", MacroRole::Tone, "Grain Density parameter"))
        .with_param(DspParamSchema::knob("spray", "Position Spray", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Position Spray parameter"))
        .with_param(DspParamSchema::knob("pitch_ratio", "Pitch Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Tone, "Pitch Ratio parameter"))
        .with_param(DspParamSchema::knob("grain_size", "Grain Size", 0.01, 0.5, 0.06, "s", MacroRole::Tone, "Grain Size parameter"))
        .with_param(DspParamSchema::knob("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Reverse Probability parameter"))
    );
    descriptors.insert("PlasmaArcSynthesizer".to_string(), DspNodeDescriptor::new("PlasmaArcSynthesizer", "High-Voltage Plasma Arc Audio Generator", DspNodeCategory::Oscillator, "High-Voltage Plasma Arc Audio Generator")
        .with_param(DspParamSchema::log_knob("arc_voltage", "Arc Voltage", 100.0, 5000.0, 1500.0, "V", MacroRole::Tone, "Arc Voltage logarithmic parameter"))
        .with_param(DspParamSchema::knob("plasma_density", "Plasma Density", 0.1, 1.0, 0.65, "%", MacroRole::Tone, "Plasma Density parameter"))
        .with_param(DspParamSchema::knob("ionization", "Ionization Rate", 10.0, 500.0, 120.0, "kHz", MacroRole::Tone, "Ionization Rate parameter"))
        .with_param(DspParamSchema::knob("hiss_level", "Sub-thermal Hiss", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Sub-thermal Hiss parameter"))
    );
    descriptors.insert("NoiseGen".to_string(), DspNodeDescriptor::new("NoiseGen", "Multi-Color Spectral Noise Generator", DspNodeCategory::Oscillator, "Multi-Color Spectral Noise Generator")
        .with_param(DspParamSchema::choice("noise_type", "Noise Color", &["White", "Pink (1/f)", "Brownian", "Blue"], 0, "Noise Color mode selector"))
        .with_param(DspParamSchema::log_knob("cutoff", "Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Filter Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Output Gain parameter"))
    );
    descriptors.insert("PluckSynth".to_string(), DspNodeDescriptor::new("PluckSynth", "Karplus-Strong Plucked String Synthesis Core", DspNodeCategory::Oscillator, "Karplus-Strong Plucked String Synthesis Core")
        .with_param(DspParamSchema::knob("damping", "String Damping Cutoff", 100.0, 10000.0, 3500.0, "Hz", MacroRole::Tone, "String Damping Cutoff parameter"))
        .with_param(DspParamSchema::knob("tension", "Feedback Loop Tension", 0.5, 0.999, 0.98, "%", MacroRole::Tone, "Feedback Loop Tension parameter"))
        .with_param(DspParamSchema::knob("decay", "Exciter Burst Decay", 0.001, 0.1, 0.015, "s", MacroRole::Tone, "Exciter Burst Decay parameter"))
        .with_param(DspParamSchema::knob("brightness", "Noise Exciter Brightness", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Noise Exciter Brightness parameter"))
    );
    descriptors.insert("FmOperatorPair".to_string(), DspNodeDescriptor::new("FmOperatorPair", "2-Operator Phase Modulation FM Synthesizer", DspNodeCategory::Oscillator, "2-Operator Phase Modulation FM Synthesizer")
        .with_param(DspParamSchema::log_knob("carrier_freq", "Carrier Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Carrier Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("mod_ratio", "Modulator Ratio", 0.25, 16.0, 2.0, "x", MacroRole::Tone, "Modulator Ratio parameter"))
        .with_param(DspParamSchema::knob("fm_depth", "FM Modulation Depth", 0.0, 10.0, 2.0, "idx", MacroRole::Tone, "FM Modulation Depth parameter"))
        .with_param(DspParamSchema::log_knob("mod_decay", "Modulator Env Decay", 10.0, 2000.0, 300.0, "ms", MacroRole::Tone, "Modulator Env Decay logarithmic parameter"))
    );
    descriptors.insert("AnalogDrumVoice".to_string(), DspNodeDescriptor::new("AnalogDrumVoice", "808/909 Style Analog Synthesized Drum Voice", DspNodeCategory::Oscillator, "808/909 Style Analog Synthesized Drum Voice")
        .with_param(DspParamSchema::log_knob("tune", "Fundamental Pitch", 30.0, 500.0, 60.0, "Hz", MacroRole::Tone, "Fundamental Pitch logarithmic parameter"))
        .with_param(DspParamSchema::log_knob("decay", "Amp Envelope Decay", 10.0, 1500.0, 250.0, "ms", MacroRole::Tone, "Amp Envelope Decay logarithmic parameter"))
        .with_param(DspParamSchema::knob("snap", "Click / Pitch Snap", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Click / Pitch Snap parameter"))
        .with_param(DspParamSchema::knob("punch", "Transient Punch", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Transient Punch parameter"))
    );
    descriptors.insert("QuantumStateVectorOscillator".to_string(), DspNodeDescriptor::new("QuantumStateVectorOscillator", "Qubit Phase Superposition & Hadamard Gate Synth", DspNodeCategory::Oscillator, "Qubit Phase Superposition & Hadamard Gate Synth")
        .with_param(DspParamSchema::log_knob("freq", "Superposition Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Superposition Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("phase_alpha", "Qubit Alpha Phase", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Qubit Alpha Phase parameter"))
        .with_param(DspParamSchema::knob("phase_beta", "Qubit Beta Phase", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Qubit Beta Phase parameter"))
        .with_param(DspParamSchema::knob("hadamard_mix", "Hadamard Transformation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Hadamard Transformation parameter"))
        .with_param(DspParamSchema::toggle("pauli_x", "Pauli-X State Flip", false, "Pauli-X State Flip toggle switch"))
    );
    descriptors.insert("QuantumHarmonicOscillatorVoice".to_string(), DspNodeDescriptor::new("QuantumHarmonicOscillatorVoice", "Quantum Harmonic Oscillator Voice Bank", DspNodeCategory::Oscillator, "Quantum Harmonic Oscillator Voice Bank")
        .with_param(DspParamSchema::knob("energy_level", "Quantum State Level (n)", 0.0, 10.0, 2.0, "n", MacroRole::Character, "Quantum State Level (n) integer control"))
        .with_param(DspParamSchema::knob("wavefunction_sigma", "Wavefunction Spatial Width", 0.1, 5.0, 1.2, "sigma", MacroRole::Tone, "Wavefunction Spatial Width parameter"))
        .with_param(DspParamSchema::knob("potential_well", "Harmonic Well Depth", 0.1, 10.0, 4.0, "eV", MacroRole::Tone, "Harmonic Well Depth parameter"))
        .with_param(DspParamSchema::knob("output_gain", "Voice Output Gain", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Voice Output Gain parameter"))
    );
    descriptors.insert("NavierStokesFluidNode".to_string(), DspNodeDescriptor::new("NavierStokesFluidNode", "Navier-Stokes 3D Acoustic Fluid Dynamics", DspNodeCategory::Oscillator, "Navier-Stokes 3D Acoustic Fluid Dynamics")
        .with_param(DspParamSchema::knob("sound_speed", "Fluid Sound Speed", 100.0, 2000.0, 343.0, "m/s", MacroRole::Tone, "Fluid Sound Speed parameter"))
        .with_param(DspParamSchema::knob("viscosity", "Kinematic Viscosity", 0.001, 1.0, 0.05, "", MacroRole::Tone, "Kinematic Viscosity parameter"))
        .with_param(DspParamSchema::knob("pressure_damping", "Poisson Pressure Damping", 0.9, 0.999, 0.995, "", MacroRole::Tone, "Poisson Pressure Damping parameter"))
        .with_param(DspParamSchema::knob("grid_res", "3D Lattice Grid Size", 2.0, 8.0, 4.0, "cube", MacroRole::Character, "3D Lattice Grid Size integer control"))
    );
    descriptors.insert("MolecularVibrationResonator".to_string(), DspNodeDescriptor::new("MolecularVibrationResonator", "Molecular Vibration Resonator Crystalline Lattice", DspNodeCategory::Oscillator, "Molecular Vibration Resonator Crystalline Lattice")
        .with_param(DspParamSchema::log_knob("fundamental_hz", "Lattice Base Frequency", 20.0, 10000.0, 520.0, "Hz", MacroRole::Tone, "Lattice Base Frequency logarithmic parameter"))
        .with_param(DspParamSchema::knob("resonance", "Modal Resonance Q", 0.1, 0.99, 0.85, "%", MacroRole::Tone, "Modal Resonance Q parameter"))
        .with_param(DspParamSchema::choice("lattice_type", "Crystalline Lattice", &["Quartz (SiO2)", "Diamond (Carbon)", "Graphene 2D", "Silicon Carbide"], 0, "Crystalline Lattice mode selector"))
        .with_param(DspParamSchema::knob("thermal_damping", "Thermal Damping Loss", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Thermal Damping Loss parameter"))
    );
    descriptors.insert("ClosedLoopNeuroFeedbackOscillator".to_string(), DspNodeDescriptor::new("ClosedLoopNeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator, "Closed-Loop Neuro-Feedback Relaxation Oscillator")
        .with_param(DspParamSchema::knob("target_alpha_hz", "Target Alpha Frequency", 7.0, 14.0, 10.5, "Hz", MacroRole::Tone, "Target Alpha Frequency parameter"))
        .with_param(DspParamSchema::knob("feedback_gain", "Neuro-Feedback Loop Gain", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Neuro-Feedback Loop Gain parameter"))
        .with_param(DspParamSchema::knob("relaxation_depth", "Entrainment Depth", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Entrainment Depth parameter"))
        .with_param(DspParamSchema::log_knob("binaural_carrier", "Binaural Carrier Center", 100.0, 500.0, 216.0, "Hz", MacroRole::Tone, "Binaural Carrier Center logarithmic parameter"))
    );
    descriptors.insert("BrainwaveEntrainmentBinauralBeat".to_string(), DspNodeDescriptor::new("BrainwaveEntrainmentBinauralBeat", "Brainwave Entrainment Binaural Beat Generator", DspNodeCategory::Oscillator, "Brainwave Entrainment Binaural Beat Generator")
        .with_param(DspParamSchema::log_knob("carrier_freq_hz", "Binaural Carrier Pitch", 50.0, 800.0, 250.0, "Hz", MacroRole::Tone, "Binaural Carrier Pitch logarithmic parameter"))
        .with_param(DspParamSchema::knob("beat_freq_hz", "Differential Beat Freq", 0.5, 30.0, 7.83, "Hz", MacroRole::Tone, "Differential Beat Freq parameter"))
        .with_param(DspParamSchema::knob("entrainment_depth", "Modulation Amplitude", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Modulation Amplitude parameter"))
        .with_param(DspParamSchema::toggle("isochronic_pulse", "Isochronic Pulse Mode", false, "Isochronic Pulse Mode toggle switch"))
    );
    descriptors.insert("SubharmonicSynthNode".to_string(), DspNodeDescriptor::new("SubharmonicSynthNode", "Subharmonic Multi-Octave Sub Oscillator", DspNodeCategory::Oscillator, "Subharmonic Multi-Octave Sub Oscillator")
        .with_param(DspParamSchema::knob("sub1_oct_gain", "Sub-1 Octave (-12st)", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Sub-1 Octave (-12st) parameter"))
        .with_param(DspParamSchema::knob("sub2_oct_gain", "Sub-2 Octave (-24st)", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Sub-2 Octave (-24st) parameter"))
        .with_param(DspParamSchema::log_knob("lowpass_cutoff", "Sub Lowpass Corner", 40.0, 400.0, 120.0, "Hz", MacroRole::Tone, "Sub Lowpass Corner logarithmic parameter"))
        .with_param(DspParamSchema::knob("drive", "Asymmetric Drive", 1.0, 5.0, 1.5, "x", MacroRole::Tone, "Asymmetric Drive parameter"))
    );
    descriptors.insert("GranularCloudNode".to_string(), DspNodeDescriptor::new("GranularCloudNode", "Ultra-Density Granular Cloud Texture Generator", DspNodeCategory::Oscillator, "High-density asynchronous granular cloud engine with up to 128 micro-grains, pitch spray, and position jitter")
        .with_param(DspParamSchema::knob("grain_density", "Grain Density", 1.0, 128.0, 32.0, "grains/s", MacroRole::Tone, "Continuous grain spawn frequency parameter"))
        .with_param(DspParamSchema::knob("grain_duration", "Grain Duration", 0.01, 1.0, 0.08, "s", MacroRole::Tone, "Individual micro-grain window length parameter"))
        .with_param(DspParamSchema::knob("pitch_spray", "Pitch Randomization", 0.0, 24.0, 3.0, "semitones", MacroRole::Character, "Random microtonal pitch dispersion parameter"))
        .with_param(DspParamSchema::knob("position_jitter", "Buffer Scramble", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Playback position randomization jitter parameter"))
        .with_param(DspParamSchema::knob("pan_spread", "Stereo Cloud Spread", 0.0, 1.0, 0.80, "%", MacroRole::Space, "Spatial panning divergence across grains parameter"))
        .with_param(DspParamSchema::knob("reverse_probability", "Reverse Grain Chance", 0.0, 1.0, 0.10, "%", MacroRole::Character, "Stochastic reverse grain playback probability parameter"))
    );
    descriptors.insert("BinauralEntrainmentGen".to_string(), DspNodeDescriptor::new("BinauralEntrainmentGen", "Binaural Beat Brainwave Entrainment Generator", DspNodeCategory::Oscillator, "Stereo offset frequency generator targeting Alpha, Beta, Theta, and Delta brainwave entrainment states")
        .with_param(DspParamSchema::knob("carrier_freq", "Carrier Frequency", 40.0, 800.0, 216.0, "Hz", MacroRole::Tone, "Audible carrier base frequency parameter"))
        .with_param(DspParamSchema::knob("beat_freq", "Entrainment Beat Frequency", 0.5, 40.0, 7.83, "Hz", MacroRole::Punch, "Interaural differential frequency offset parameter"))
        .with_param(DspParamSchema::choice("target_state", "Brainwave State Target", &["Schumann (7.83 Hz)", "Theta (5 Hz)", "Alpha (10 Hz)", "Beta (18 Hz)", "Gamma (40 Hz)"], 0, "Neurological target brainwave state selector"))
    );
    descriptors.insert("SineOscillatorNode".to_string(), DspNodeDescriptor::new("SineOscillatorNode", "Core Deterministic Sine Oscillator AudioNode", DspNodeCategory::Oscillator, "Bit-exact, phase-continuous deterministic sine wave oscillator implementing fundamental summoner_core::node::AudioNode")
        .with_param(DspParamSchema::log_knob("frequency", "Oscillator Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Deterministic sine oscillation frequency parameter"))
        .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Initial waveform starting phase parameter"))
        .with_param(DspParamSchema::knob("gain", "Oscillator Gain", 0.0, 1.0, 0.80, "%", MacroRole::Tone, "Output level amplitude scaling parameter"))
    );
    descriptors.insert("NeuroFeedbackOscillator".to_string(), DspNodeDescriptor::new("NeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator, "Real-time EEG biofeedback tone generator whose pitch and harmonic dispersion are modulated by alpha/beta brainwave ratios")
        .with_param(DspParamSchema::log_knob("base_freq", "Base Carrier Frequency", 20.0, 2000.0, 432.0, "Hz", MacroRole::Tone, "Nominal meditation drone fundamental frequency parameter"))
        .with_param(DspParamSchema::knob("cents_offset", "Neuro Tuning Shift", -100.0, 100.0, 0.0, "cents", MacroRole::Character, "Real-time brainwave-induced microtonal pitch sag parameter"))
        .with_param(DspParamSchema::knob("alpha_target", "Alpha Coherence Target", 8.0, 14.0, 10.5, "Hz", MacroRole::Space, "Target neural oscillation frequency for harmonic reinforcement parameter"))
        .with_param(DspParamSchema::knob("feedback_depth", "Biofeedback Depth", 0.0, 1.0, 0.65, "%", MacroRole::Punch, "Closed-loop EEG-to-frequency modulation intensity parameter"))
    );
    descriptors.insert("SpectrogramSoundGenerator".to_string(), DspNodeDescriptor::new("SpectrogramSoundGenerator", "Visual Spectrogram Image-to-Audio Synthesizer HUD", DspNodeCategory::Oscillator, "Inverse STFT oscillator bank transforming raster images and drawn spectral artwork into audible sound waves with logarithmic frequency distribution")
        .with_param(DspParamSchema::knob("playback_speed", "Spectrogram Image Playback Scan Speed", 0.1, 10.0, 1.0, "x", MacroRole::Tone, "Horizontal pixel column scanning rate across the canvas image"))
        .with_param(DspParamSchema::choice("frequency_range_mode", "Audio Frequency Mapping Spectrum Range", &["Full Audible (20Hz-20kHz)", "Musical (55Hz-7kHz)", "Sub-Bass (20Hz-250Hz)", "Vocal Formant (200Hz-4kHz)"], 0, "Vertical pixel axis mapping to acoustic frequency bands"))
        .with_param(DspParamSchema::knob("dynamic_contrast", "Visual Pixel Intensity Dynamic Contrast", 0.5, 4.0, 1.5, "ratio", MacroRole::Character, "Power curve applied to image brightness determining oscillator amplitudes"))
        .with_param(DspParamSchema::choice("harmonic_quantization", "Frequency Bin Scale Quantization", &["Continuous Microtonal", "12-EDO Chromatic", "19-EDO Extended", "31-EDO Just"], 0, "Quantizes vertical oscillator frequencies to musical scale intervals"))
        .with_param(DspParamSchema::knob("stereo_image_spread", "Stereo Spatial Panorama Distribution", 0.0, 100.0, 50.0, "%", MacroRole::Space, "Pans oscillators across the stereo soundstage according to image color balance"))
    );
    descriptors.insert("SimdPolyVoice".to_string(), DspNodeDescriptor::new("SimdPolyVoice", "SIMD Parallel Vectorized Synthesizer Voice Cell", DspNodeCategory::Oscillator, "AVX2/NEON SIMD 8-wide vectorized synthesizer voice instance storing MIDI note number, base oscillator frequency, strike velocity, phase accumulator, and voice lifecycle flags")
        .with_param(DspParamSchema::knob("note", "MIDI Key Note Number", 0.0, 127.0, 60.0, "midi", MacroRole::Tone, "Active MIDI key note index assigned to this SIMD lane"))
        .with_param(DspParamSchema::log_knob("frequency", "Fundamental Pitch Frequency", 10.0, 20000.0, 261.63, "Hz", MacroRole::Tone, "Fundamental voice oscillation pitch frequency in Hertz"))
        .with_param(DspParamSchema::knob("velocity", "Key Strike Dynamic Velocity", 0.0, 127.0, 100.0, "vel", MacroRole::Punch, "Key strike velocity governing dynamic filter cutoff and gain modulation"))
        .with_param(DspParamSchema::knob("env_level", "Instantaneous Envelope Amplitude", 0.0, 1.0, 0.8, "amp", MacroRole::Character, "Instantaneous amplitude envelope level rendered by SIMD voice"))
        .with_param(DspParamSchema::toggle("active", "SIMD Voice Allocation Gate", true, "SIMD voice allocation gate flag currently generating audio samples"))
    );
    descriptors.insert("NoiseType".to_string(), DspNodeDescriptor::new("NoiseType", "Zero-Allocation Multi-Color Noise Generator Core", DspNodeCategory::Oscillator, "High-precision pseudo-random noise generator delivering white noise, 1/f pink noise (-3dB/octave filtered), and 1/f² brown/red integrated random walk noise with zero heap allocation")
        .with_param(DspParamSchema::choice("noise_spectral_color", "Pseudo-Random Noise Spectral Distribution", &["White Noise (Flat Spectral Power Density)", "Pink Noise (1/f Equal Energy Per Octave Band)", "Brown Noise (1/f² Brownian Integrated Random Walk)"], 1, "Spectral slope and color distribution of noise generator"))
        .with_param(DspParamSchema::knob("output_level_db", "Master Noise Generation Level", -60.0, 12.0, -12.0, "dB", MacroRole::Punch, "Output gain of noise generator before routing"))
        .with_param(DspParamSchema::knob("highpass_rumble_filter_hz", "Sub-Bass Highpass Rumble Cutoff", 5.0, 500.0, 20.0, "Hz", MacroRole::Tone, "Cutoff frequency rejecting subsonic DC offset and extreme rumble"))
        .with_param(DspParamSchema::knob("stereo_decorrelation_width", "Stereo Phase Decorrelation Spread", 0.0, 2.0, 1.0, "width", MacroRole::Space, "Spatial width decorrelating left and right noise PRNG seeds"))
        .with_param(DspParamSchema::knob("entropy_seed_randomness", "PRNG Initial Entropy Seed Offset", 1.0, 100.0, 42.0, "seed", MacroRole::Character, "Seed initialization constant for deterministic repeatable noise"))
        .with_param(DspParamSchema::toggle("enable_anti_aliased_bandlimited_noise", "Band-Limited Anti-Aliased PRNG Evaluation", true, "Filter high frequencies near Nyquist to prevent harsh aliased foldover"))
    );
    descriptors.insert("SubHarmonicBassSynthesizer".to_string(), DspNodeDescriptor::new("SubHarmonicBassSynthesizer", "Waveform Sub-Octave Generator & Psychoacoustic Fundamental Synthesizer", DspNodeCategory::Oscillator, "Sub-bass frequency generator tracking monophonic audio inputs and synthesizing phase-coherent sub-octaves with harmonic warmth")
        .with_param(DspParamSchema::choice("sub_octave_interval", "Sub-Harmonic Pitch Interval", &["One Octave Below (-12st)", "Two Octaves Below (-24st)", "Sub-Fifth (-19st)", "Dual Sub Layers (-12st & -24st)"], 0, "Musical interval of synthesized sub-bass frequency below detected fundamental"))
        .with_param(DspParamSchema::knob("sub_blend_level_db", "Sub-Harmonic Volume Level", -36.0, 6.0, -6.0, "dB", MacroRole::Punch, "Volume level of generated sub-bass wave mixed into audio output"))
        .with_param(DspParamSchema::knob("harmonic_saturation_drive", "Harmonic Saturation Drive", 0.0, 1.0, 0.25, "drive", MacroRole::Character, "Generates even and odd harmonics to ensure sub-bass audibility on small speakers"))
        .with_param(DspParamSchema::knob("lowpass_cutoff_frequency_hz", "Sub-Bass Lowpass Filter Cutoff", 40.0, 180.0, 90.0, "Hz", MacroRole::Tone, "Steep 24 dB/octave lowpass filter restricting sub energy to low frequencies"))
        .with_param(DspParamSchema::knob("tracking_sensitivity_threshold", "Pitch Tracking Sensitivity", 0.05, 0.8, 0.2, "sens", MacroRole::None, "Zero-crossing threshold sensitivity for tracking low-frequency fundamental"))
        .with_param(DspParamSchema::toggle("dynamic_envelope_following", "Dynamic Envelope Following", true, "Modulates sub-bass amplitude to match dynamic expression of original signal"))
    );
    descriptors.insert("WavetablePhaseDistortionOsc".to_string(), DspNodeDescriptor::new("WavetablePhaseDistortionOsc", "Hybrid Wavetable & Resonant Phase Distortion Synthesizer Engine", DspNodeCategory::Oscillator, "Synthesis core pairing multi-frame wavetables with Casio CZ-style resonant phase distortion mapping and hard sync for sharp, tearing synth leads and metallic timbres")
        .with_param(DspParamSchema::knob("wavetable_position_index", "Wavetable Scan Position Frame", 0.0, 64.0, 0.0, "pos", MacroRole::Tone, "Continuous interpolation point across multi-cycle wavetable harmonic frames"))
        .with_param(DspParamSchema::knob("phase_distortion_intensity", "Phase Distortion Transfer Intensity", 0.0, 1.0, 0.5, "pd", MacroRole::Character, "Bends phase accumulator with non-linear functions to generate artificial resonant peaks"))
        .with_param(DspParamSchema::choice("pd_resonant_window_form", "Resonant Window Envelope Shape", &["Saw Resonant Peak", "Triangle Peak", "Trapezoid Pulse", "Double Sine Formant"], 0, "Phase distortion transfer function mimicking resonant lowpass and bandpass filters"))
        .with_param(DspParamSchema::knob("sub_oscillator_wave_mix", "Sub-Oscillator Blend Level", 0.0, 1.0, 0.25, "mix", MacroRole::Punch, "Volume level of square sub-oscillator pitched one octave below fundamental"))
        .with_param(DspParamSchema::knob("hard_sync_frequency_ratio", "Slave Hard Sync Pitch Ratio", 1.0, 8.0, 1.0, "ratio", MacroRole::Character, "Forces phase reset at master cycle intervals to produce aggressive screaming sync sweeps"))
        .with_param(DspParamSchema::toggle("anti_aliasing_simd_oversample", "Polyphase Anti-Aliasing Oversampling", true, "Oversamples oscillator kernel 4x using AVX2 SIMD to suppress high-register foldover"))
    );
    descriptors.insert("FranklinGlassArmonica".to_string(), DspNodeDescriptor::new("FranklinGlassArmonica", "37-Bowl Franklin Glass Armonica Physical Acoustic Resonator", DspNodeCategory::Oscillator, "Physical acoustic model of Benjamin Franklin's 1761 Glass Armonica featuring rotating quartz bowls, non-linear Stribeck stick-slip wet-finger friction, thin-shell doublet splitting, and hydro-acoustic water-tuning")
        .with_param(DspParamSchema::knob("spindle_rotation_speed_rad_s", "Spindle Rotational Speed", 0.0, 20.0, 6.28, "rad/s", MacroRole::Tone, "Horizontal spindle rotational angular velocity driving bowl contact speed"))
        .with_param(DspParamSchema::knob("finger_rubbing_pressure_n", "Finger Rubbing Contact Pressure", 0.01, 5.0, 0.85, "N", MacroRole::Punch, "Wet finger contact force onto rotating quartz rim"))
        .with_param(DspParamSchema::choice("quartz_material_preset", "Quartz Crystal Formulation Profile", &["Franklin Authentic 1761 Quartz", "Mesmer Healing Chalice", "Mozart Concert Armonica", "Ethereal Borosilicate", "Water-Tuned Crystal Chime"], 0, "Historical quartz crystal formulation and internal damping profile"))
        .with_param(DspParamSchema::knob("water_mass_loading_ratio", "Hydro-Acoustic Water Mass Loading", 0.0, 1.0, 0.15, "ratio", MacroRole::Space, "Hydro-acoustic water level in trough dampening overtones and lowering pitch"))
        .with_param(DspParamSchema::knob("doublet_beating_shimmer_hz", "Thin-Shell Doublet Beating Shimmer", 0.0, 10.0, 1.8, "Hz", MacroRole::Character, "Acoustic beating frequency between orthogonal (2,0) circular hoop modes"))
        .with_param(DspParamSchema::knob("mahogany_chassis_coupling", "Resonant Mahogany Chassis Coupling", 0.0, 1.0, 0.6, "ratio", MacroRole::Tone, "Spindle vibrational acoustic transmission into resonant wooden case"))
    );
    descriptors.insert("FrenchHurdyGurdy".to_string(), DspNodeDescriptor::new("FrenchHurdyGurdy", "French Vielle à Roue Hurdy-Gurdy Physical Synthesizer", DspNodeCategory::Oscillator, "Acoustic physical modeling of the French Hurdy-Gurdy featuring continuous rosined wheel friction, dual melody chanterelles, 3 drone strings, non-linear trompette buzzing dog bridge, and lute soundbox cavity")
        .with_param(DspParamSchema::knob("wheel_crank_speed_rpm", "Rosined Wheel Cranking Speed", 20.0, 240.0, 110.0, "RPM", MacroRole::Tone, "Rosined wooden wheel rotational cranking speed"))
        .with_param(DspParamSchema::knob("wheel_rosin_friction_grip", "Rosin Stick-Slip Friction Grip", 0.1, 1.0, 0.72, "grip", MacroRole::Character, "Rosin stick-slip friction grip and bow-wheel bite onto strings"))
        .with_param(DspParamSchema::knob("trompette_buzz_dog_level", "Trompette Buzzing Dog Level", 0.0, 1.0, 0.55, "gain", MacroRole::Punch, "Unilateral chattering bridge buzz level driven by wrist accelerations"))
        .with_param(DspParamSchema::knob("coup_de_poignet_accent", "Coup de Poignet Wrist Accent Impulses", 0.0, 2.0, 0.3, "burst", MacroRole::Punch, "Wrist accent impulses triggering rhythmic buzzing dog snarls"))
        .with_param(DspParamSchema::knob("drone_bourdon_mix", "Drone Bourdon Strings Acoustic Mix", 0.0, 1.0, 0.45, "mix", MacroRole::Tone, "Balance of Gros Bourdon, Petit Bourdon, and Mouche drone strings"))
        .with_param(DspParamSchema::knob("sympathetic_tarab_resonance", "Sympathetic Strings Resonant Coupling", 0.0, 1.0, 0.35, "res", MacroRole::Space, "Internal sympathetic strings acoustic coupling and sustain"))
    );
    descriptors.insert("BilinearAirReedJet".to_string(), DspNodeDescriptor::new("BilinearAirReedJet", "Non-Linear Flue Air-Reed Jet Excitation Generator", DspNodeCategory::Oscillator, "Acoustic modeling of woodwind air-jet flue dynamics (recorders, transverse flutes, organ flue pipes) including vortex shedding across labium edge, jet delay travel time, and non-linear polynomial drive")
        .with_param(DspParamSchema::knob("blowing_jet_velocity_m_s", "Flue Air-Jet Stream Velocity", 5.0, 80.0, 24.0, "m/s", MacroRole::Punch, "Stream velocity of air escaping windway fissure toward labium edge"))
        .with_param(DspParamSchema::knob("jet_to_labium_distance_mm", "Windway to Labium Edge Distance", 2.0, 20.0, 7.5, "mm", MacroRole::Tone, "Physical transit distance between flue exit and sharp labium edge"))
        .with_param(DspParamSchema::knob("labium_edge_splitting_offset", "Jet-to-Labium Transverse Alignment", -1.0, 1.0, 0.05, "offset", MacroRole::Character, "Transverse alignment of air jet center relative to labium knife-edge"))
        .with_param(DspParamSchema::knob("vortex_shedding_turbulence", "Labium Vortex Shedding Turbulence", 0.0, 1.0, 0.22, "turb", MacroRole::Tone, "Turbulent acoustic broadband breathiness generated at flue lip"))
        .with_param(DspParamSchema::choice("jet_drive_polynomial_order", "Nonlinear Jet Saturation Curve", &["Cubic Symmetrical Saturation", "Quintic Sharp Flue Curve", "Bilinear Piecewise Linear"], 0, "Nonlinear saturation curve modeling jet switching across edge"))
        .with_param(DspParamSchema::knob("feedback_bore_pressure_coupling", "Acoustic Bore Feedback Pressure", 0.1, 2.0, 1.0, "gain", MacroRole::Punch, "Acoustic feedback gain from bore standing wave onto exiting jet"))
    );
    descriptors.insert("ModalIdiophoneResonator".to_string(), DspNodeDescriptor::new("ModalIdiophoneResonator", "Undercut Tuned Idiophone Bar & Resonator Tube Bank", DspNodeCategory::Oscillator, "Physical acoustic synthesis of undercut wooden and metallic idiophone bars (marimba, xylophone, vibraphone, celesta) with tuned 1:4:10 modal ratios and coupled Helmholtz tubular resonators")
        .with_param(DspParamSchema::choice("idiophone_bar_material", "Resonant Bar Material & Hardness", &["Honduran Rosewood (Marimba)", "Padauk African Hardwood (Xylophone)", "Tempered Aluminum Alloy (Vibraphone)", "Hardened High-Carbon Steel (Glockenspiel)"], 0, "Elastic modulus, sound speed, and internal damping of bar material"))
        .with_param(DspParamSchema::choice("undercut_tuning_profile", "Undercut Arch Modal Tuning Ratio", &["Quint Tuning (1:3:9)", "Octave Tuning (1:4:10)", "Harmonic Triple Octave (1:4:8)", "Untuned Free Bar"], 1, "Arch undercut carving geometry tuning the first two overtone modes"))
        .with_param(DspParamSchema::knob("resonator_tube_coupling", "Tubular Air Resonator Coupling", 0.0, 1.0, 0.75, "gain", MacroRole::Space, "Acoustic coupling into tuned quarter-wave tubular air resonator"))
        .with_param(DspParamSchema::knob("tube_q_tuning_sharpness", "Resonator Tube Q Sharpness", 5.0, 60.0, 24.0, "Q", MacroRole::Tone, "Q factor of air column inside tubular resonator"))
        .with_param(DspParamSchema::knob("strike_mallet_hardness", "Mallet Yarn/Rubber Strike Hardness", 0.1, 1.0, 0.45, "hard", MacroRole::Punch, "Mallet yarn/rubber stiffness exciting high frequency bar modes"))
        .with_param(DspParamSchema::knob("vibraphone_tremolo_fan_speed_hz", "Vibraphone Tremolo Butterfly Fan Speed", 0.0, 8.0, 0.0, "Hz", MacroRole::Character, "Rotary butterfly damper fan speed inside tubular resonators"))
    );
    descriptors.insert("TineResonatorPickup".to_string(), DspNodeDescriptor::new("TineResonatorPickup", "Asymmetric Tine-Tonebar Electro-Acoustic Pickup Engine", DspNodeCategory::Oscillator, "Physical modeling of the asymmetric Rhodes tine and heavy steel tonebar resonator assembly, variable reluctance magnetic coil pickup geometry, and non-linear bark overdrive under heavy hammer strikes")
        .with_param(DspParamSchema::knob("tine_to_pickup_gap_mm", "Tine Tip to Pickup Air Gap", 0.5, 10.0, 2.2, "mm", MacroRole::Punch, "Clearance gap between vibrating tine tip and magnetic pickup pole piece"))
        .with_param(DspParamSchema::knob("pickup_vertical_alignment", "Pickup Neutral Axis Vertical Offset", -1.0, 1.0, 0.25, "align", MacroRole::Character, "Vertical offset of pickup axis relative to tine neutral plane (governs 2nd harmonic bark)"))
        .with_param(DspParamSchema::knob("tonebar_mass_ratio", "Tonebar to Tine Mass Ratio", 5.0, 50.0, 22.0, "ratio", MacroRole::Tone, "Mass ratio of resonant tonebar to flexible tine governing sustain envelope"))
        .with_param(DspParamSchema::choice("neoprene_hammer_tip_hardness", "Neoprene Rubber Hammer Tip Grade", &["Soft Neoprene (Bass Tines)", "Medium Neoprene (Mid Tines)", "Hard Neoprene (Treble Tines)", "Wooden Tangent Insert"], 1, "Stiffness of rubber hammer tip hitting the tine"))
        .with_param(DspParamSchema::knob("pickup_coil_inductance_h", "Electromagnetic Coil Inductance", 0.5, 5.0, 1.8, "H", MacroRole::Tone, "Electromagnetic pickup coil inductance shaping high-frequency resonant peak"))
        .with_param(DspParamSchema::knob("magnetic_reluctance_distortion", "Magnetic Reluctance Bark Overdrive", 0.0, 1.0, 0.4, "drive", MacroRole::Character, "Nonlinear reluctance saturation and bell-like tonal distortion"))
    );
    descriptors.insert("PercussionMembrane2D".to_string(), DspNodeDescriptor::new("PercussionMembrane2D", "2D Circular Bessel Drumhead Membrane & Air-Loaded Cavity", DspNodeCategory::Oscillator, "Physical modal synthesis of circular tensioned percussion membranes (snare head, kettledrum, tom-tom) solving 2D wave equations with Bessel J_m(alpha r) eigenvalues and enclosed kettle air loading")
        .with_param(DspParamSchema::choice("membrane_skin_material", "Drumhead Membrane Skin Material", &["Single-Ply 10-Mil Mylar", "Two-Ply Coated Hydraulic", "Natural Calfskin Vellum", "Woven Kevlar Marching Snare"], 0, "Membrane areal density, elasticity, and internal frictional damping"))
        .with_param(DspParamSchema::knob("membrane_tuning_tension_n_m", "Radial Tension Lug Tuning Force", 500.0, 8000.0, 2400.0, "N/m", MacroRole::Tone, "Uniform radial tension applied by tension lugs anchoring base pitch"))
        .with_param(DspParamSchema::knob("kettle_air_cavity_depth_cm", "Enclosed Shell Air Cavity Depth", 5.0, 80.0, 32.0, "cm", MacroRole::Space, "Enclosed shell cavity air depth coupling air stiffness and lowering (0,1) mode"))
        .with_param(DspParamSchema::knob("strike_radial_impact_point", "Drumstick Radial Impact Position", 0.0, 0.95, 0.65, "radius", MacroRole::Punch, "Strike location (0.0 = dead center deadens odd modes, 0.7 = rich overtone sweet spot)"))
        .with_param(DspParamSchema::knob("rimshot_hoop_impact_mix", "Rimshot Hoop Strike Transient Mix", 0.0, 1.0, 0.0, "mix", MacroRole::Punch, "Simultaneous metallic rim hoop strike transient mix"))
        .with_param(DspParamSchema::knob("bottom_resonant_head_coupling", "Resonant Bottom Head & Snare Coupling", 0.0, 1.0, 0.7, "ratio", MacroRole::Tone, "Acoustic air-coupling into lower resonant drumhead and snare wire bed"))
    );
    descriptors.insert("WaveguideBrassMouthpiece".to_string(), DspNodeDescriptor::new("WaveguideBrassMouthpiece", "Lip-Reed Waveguide Brass Acoustic Mouthpiece Exciter", DspNodeCategory::Oscillator, "Acoustic physical modeling of human lip vibrations (outward-striking mass-spring lip reed) coupled to cup-shaped brass instrument mouthpieces (trumpet, trombone, French horn, tuba)")
        .with_param(DspParamSchema::choice("mouthpiece_cup_profile", "Brass Mouthpiece Cup Profile", &["Shallow Trumpet Cup (14A4a)", "Standard Medium Bowl (Bach 7C)", "Deep Symphonic Horn Funnel", "Wide Tuba Cavern"], 1, "Mouthpiece cup volume, throat diameter, and backbore taper"))
        .with_param(DspParamSchema::knob("lip_reed_equilibrium_bias", "Lip Embouchure Equilibrium Opening", 0.05, 0.8, 0.28, "bias", MacroRole::Character, "Player lip embouchure tension and resting aperture opening"))
        .with_param(DspParamSchema::knob("player_mouth_blowing_pressure_kpa", "Mouth Blowing Air Pressure", 0.5, 15.0, 4.2, "kPa", MacroRole::Punch, "Subglottal mouth blowing air pressure driving lip oscillation"))
        .with_param(DspParamSchema::knob("lip_mass_resonance_hz", "Lip Tissue Mechanical Resonance", 80.0, 1200.0, 349.23, "Hz", MacroRole::Tone, "Natural mechanical resonance frequency of player's lip tissue"))
        .with_param(DspParamSchema::knob("lip_reed_q_factor", "Lip Tissue Mechanical Q Factor", 2.0, 20.0, 6.5, "Q", MacroRole::Tone, "Mechanical sharpness of lip tissue resonance"))
        .with_param(DspParamSchema::knob("bernoulli_flow_nonlinearity", "Bernoulli Hydrodynamic Closing Force", 0.1, 1.0, 0.75, "coeff", MacroRole::Punch, "Nonlinear Bernoulli hydrodynamic flow force pulling lips shut"))
    );
    descriptors.insert("GlottalPulseGenerator".to_string(), DspNodeDescriptor::new("GlottalPulseGenerator", "Liljencrants-Fant (LF) Glottal Flow Acoustic Voice Source", DspNodeCategory::Oscillator, "Physiologically accurate Liljencrants-Fant (LF) parametric glottal waveform generator modeling vocal fold open quotient, return phase, dynamic jitter, shimmer, and vocal tract aspiration noise")
        .with_param(DspParamSchema::knob("open_quotient_oq", "Vocal Fold Open Quotient (OQ)", 0.3, 0.9, 0.58, "OQ", MacroRole::Character, "Fraction of glottal cycle during which vocal folds are parted (warmth vs brassiness)"))
        .with_param(DspParamSchema::knob("return_phase_abruptness_rg", "Closure Abruptness Return Phase", 0.005, 0.2, 0.04, "Ta", MacroRole::Tone, "Vocal fold closure abruptness setting high-frequency spectral tilt slope"))
        .with_param(DspParamSchema::knob("speed_quotient_opening_asymmetry", "Opening/Closing Speed Quotient", 1.0, 3.5, 2.2, "ratio", MacroRole::Character, "Ratio of opening duration to closing duration of glottal aperture"))
        .with_param(DspParamSchema::knob("pitch_jitter_percent", "Fundamental Frequency Jitter", 0.0, 5.0, 0.45, "%", MacroRole::Space, "Micro-fluctuations in fundamental period modeling organic human vocal cord drift"))
        .with_param(DspParamSchema::knob("amplitude_shimmer_db", "Cycle Amplitude Shimmer Perturbation", 0.0, 2.0, 0.15, "dB", MacroRole::Space, "Cycle-to-cycle amplitude perturbation simulating biological vocal tremor"))
        .with_param(DspParamSchema::knob("glottal_aspiration_breath_mix", "Transglottal Aspiration Breath Mix", 0.0, 1.0, 0.08, "mix", MacroRole::Tone, "Turbulent transglottal aspiration air noise bleeding into vocal tract"))
    );
    descriptors.insert("ElectroAcousticResonator".to_string(), DspNodeDescriptor::new("ElectroAcousticResonator", "Electromagnetic Pickup Feedback & Transducer Plate Resonator", DspNodeCategory::Oscillator, "Hybrid electro-acoustic physical resonator modeling transducer contact drivers, vibrating sheet metal plates, magnetostrictive pickup coils, and regenerative magnetic acoustic feedback")
        .with_param(DspParamSchema::knob("drive_current_ma", "Transducer Excitation Drive Current", 1.0, 500.0, 85.0, "mA", MacroRole::Punch, "Current supplied to electromagnetic transducer coil driving plate vibration"))
        .with_param(DspParamSchema::knob("plate_resonance_damping", "Sheet Metal Plate Viscous Loss", 0.01, 1.0, 0.22, "damp", MacroRole::Space, "Internal mechanical friction and acoustic radiation damping of plate"))
        .with_param(DspParamSchema::knob("magnetic_airgap_mm", "Pickup Reluctance Magnetic Air Gap", 0.2, 10.0, 2.0, "mm", MacroRole::Tone, "Physical clearance airgap separating magnetic pole pieces from resonant plate"))
        .with_param(DspParamSchema::knob("feedback_pickup_mix", "Regenerative Magnetic Feedback Mix", 0.0, 1.5, 0.4, "mix", MacroRole::Character, "Regenerative acoustic feedback routing pickup voltage back into driver coil"))
        .with_param(DspParamSchema::knob("transducer_saturation_ratio", "Core Magnetic Hysteresis Saturation", 0.0, 1.0, 0.35, "sat", MacroRole::Punch, "Non-linear core saturation of driver electromagnet transformer"))
        .with_param(DspParamSchema::knob("subharmonic_hysteresis_flux", "Subharmonic Magnetostriction Flux", 0.0, 1.0, 0.15, "flux", MacroRole::Tone, "Subharmonic mechanical vibration generated by magnetostrictive domain flipping"))
    );
    descriptors.insert("MorphologicalWaveTerrainOsc".to_string(), DspNodeDescriptor::new("MorphologicalWaveTerrainOsc", "Dual-Axis Non-Euclidean 3D Wave Terrain Surface Oscillator", DspNodeCategory::Oscillator, "Continuous 3D wave terrain synthesis engine navigating 2D parametric mathematical surfaces with bicubic interpolation, elliptic orbital trajectories, and chaotic attractor modulation")
        .with_param(DspParamSchema::choice("terrain_surface_topology", "Mathematical Terrain Surface Function", &["Rastrigin Multi-Modal Surface", "Toroidal Klein Bottle Contour", "Sine-Gordon Soliton Lattice", "Riemann Zeta Real Projection", "Hyperbolic Paraboloid Saddle"], 0, "Mathematical surface topology over which orbital path is mapped"))
        .with_param(DspParamSchema::knob("orbit_frequency_x_hz", "Trajectory Orbit X Frequency", 10.0, 2000.0, 110.0, "Hz", MacroRole::Tone, "Horizontal orbital navigation carrier frequency"))
        .with_param(DspParamSchema::knob("orbit_frequency_y_hz", "Trajectory Orbit Y Frequency", 10.0, 2000.0, 165.0, "Hz", MacroRole::Tone, "Vertical orbital navigation carrier frequency"))
        .with_param(DspParamSchema::knob("orbit_radius_eccentricity", "Orbital Elliptic Eccentricity", 0.1, 5.0, 1.2, "ecc", MacroRole::Character, "Ratio of X to Y orbital radii shaping Lissajous timbral figures"))
        .with_param(DspParamSchema::knob("terrain_depth_elevation", "Terrain Surface Peak Elevation", 0.1, 3.0, 1.0, "gain", MacroRole::Punch, "Z-axis terrain elevation multiplier amplifying wave folding saturation"))
        .with_param(DspParamSchema::knob("chaotic_trajectory_drift", "Chaotic Orbit Precession Drift", 0.0, 1.0, 0.25, "drift", MacroRole::Space, "Lissajous orbit rotation rate causing continuously shifting overtone patterns"))
    );
    descriptors.insert("AcousticSoundboardResonator".to_string(), DspNodeDescriptor::new("AcousticSoundboardResonator", "Orthotropic Spruce Soundboard Finite-Element Resonator", DspNodeCategory::Oscillator, "Finite-element acoustic modal resonator modeling radial quarter-sawn spruce piano and cello soundboards with anisotropic grain sound velocities, curved crown radius, and rib stiffening")
        .with_param(DspParamSchema::knob("wood_grain_anisotropy_ratio", "Spruce Grain Acoustic Anisotropy", 1.5, 6.0, 3.8, "ratio", MacroRole::Tone, "Ratio of longitudinal sound velocity (along grain) to radial velocity (across grain)"))
        .with_param(DspParamSchema::knob("soundboard_surface_area_m2", "Soundboard Total Radiating Area", 0.2, 4.0, 1.6, "m^2", MacroRole::Space, "Physical radiating surface area determining low-frequency acoustic coupling"))
        .with_param(DspParamSchema::knob("rib_stiffening_factor", "Perpendicular Rib Reinforcement", 0.1, 2.0, 0.85, "stiff", MacroRole::Tone, "Stiffening ribs glued across grain raising transverse modal frequencies"))
        .with_param(DspParamSchema::knob("bridge_mechanical_impedance", "Bridge Mechanical Load Impedance", 10.0, 500.0, 120.0, "kg/s", MacroRole::Punch, "Mechanical impedance of bridge transferring string kinetic energy into plate"))
        .with_param(DspParamSchema::knob("fundamental_plate_mode_hz", "Lowest Plate Eigenmode Resonance", 30.0, 300.0, 78.0, "Hz", MacroRole::Tone, "Fundamental acoustic breathing mode (0,0) of the soundboard assembly"))
        .with_param(DspParamSchema::knob("radiation_efficiency_loss", "High-Frequency Radiation Damping", 0.01, 0.8, 0.18, "loss", MacroRole::Space, "Acoustic radiation damping into surrounding room air at upper harmonics"))
    );
    descriptors.insert("VectorWaveSynthesisEngine".to_string(), DspNodeDescriptor::new("VectorWaveSynthesisEngine", "Four-Quadrant 2D Planar Vector Wavetable Synthesis Engine", DspNodeCategory::Oscillator, "Four-corner planar vector synthesis oscillator with 2D orbital trajectories, joystick crossfading across 4 distinct timbral wavetable quadrants, and syncable vector envelope loops")
        .with_param(DspParamSchema::knob("vector_position_x", "Vector Joystick X Position", -1.0, 1.0, 0.0, "X", MacroRole::Tone, "Horizontal position across planar vector grid (-1.0 = Left quadrants, +1.0 = Right quadrants)"))
        .with_param(DspParamSchema::knob("vector_position_y", "Vector Joystick Y Position", -1.0, 1.0, 0.0, "Y", MacroRole::Tone, "Vertical position across planar vector grid (-1.0 = Bottom quadrants, +1.0 = Top quadrants)"))
        .with_param(DspParamSchema::choice("quadrant_a_wavetable", "Quadrant A Wavetable (Top-Left)", &["Warm Analog Sawtooth", "Pristine Bell Sine", "Digital FM Metallic", "Vocal Formant Choir"], 0, "Wavetable assignment anchored at top-left corner"))
        .with_param(DspParamSchema::choice("quadrant_b_wavetable", "Quadrant B Wavetable (Top-Right)", &["Warm Analog Sawtooth", "Pristine Bell Sine", "Digital FM Metallic", "Vocal Formant Choir"], 1, "Wavetable assignment anchored at top-right corner"))
        .with_param(DspParamSchema::choice("quadrant_c_wavetable", "Quadrant C Wavetable (Bottom-Left)", &["Warm Analog Sawtooth", "Pristine Bell Sine", "Digital FM Metallic", "Vocal Formant Choir"], 2, "Wavetable assignment anchored at bottom-left corner"))
        .with_param(DspParamSchema::knob("vector_orbit_rate_hz", "Automated Vector Orbit LFO Rate", 0.0, 10.0, 0.5, "Hz", MacroRole::Character, "Speed of automated circular orbit traversing all four quadrants"))
    );
    descriptors.insert("QuantumSuperpositionOscillator".to_string(), DspNodeDescriptor::new("QuantumSuperpositionOscillator", "Qubit Wavefunction Superposition & Entanglement Oscillator", DspNodeCategory::Oscillator, "Quantum computing probabilistic state vector oscillator modeling Bloch sphere rotation, wave-function collapse upon sampling, and quantum phase kickback")
        .with_param(DspParamSchema::knob("qubit_theta_angle_rad", "Bloch Sphere Superposition Polar Angle", 0.0, 3.14159, 1.57, "rad", MacroRole::Tone, "State vector polar angle theta balancing probability amplitude between |0> and |1>"))
        .with_param(DspParamSchema::knob("qubit_phi_phase_rad", "Bloch Sphere Azimuthal Phase Angle", 0.0, 6.28318, 0.0, "rad", MacroRole::Space, "State vector relative phase angle phi introducing complex quantum interference"))
        .with_param(DspParamSchema::knob("collapse_rate_hz", "Wavefunction Measurement Collapse Rate", 1.0, 2000.0, 440.0, "Hz", MacroRole::Punch, "Rate of projective measurement wave collapse producing stochastic harmonic bursts"))
        .with_param(DspParamSchema::knob("entanglement_coupling", "Inter-Qubit Bell State Entanglement Depth", 0.0, 1.0, 0.65, "ent", MacroRole::Character, "Degree of quantum correlation linking parallel oscillator voice registers"))
        .with_param(DspParamSchema::knob("decoherence_noise_db", "Quantum Environmental Decoherence Entropy", -60.0, 0.0, -36.0, "dB", MacroRole::Tone, "Decoherence noise floor simulating thermal interaction with environment"))
        .with_param(DspParamSchema::knob("hadamard_gate_probability", "Continuous Hadamard Transformation Spread", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Continuous quantum state mixing rotating basis vectors towards equal superposition"))
    );
    descriptors.insert("NeuromorphicSpikingOscillator".to_string(), DspNodeDescriptor::new("NeuromorphicSpikingOscillator", "Biomimetic Leaky Integrate-and-Fire Spiking Neural Oscillator", DspNodeCategory::Oscillator, "Biologically accurate neuromorphic oscillator modeling soma membrane potential, action potential spike discharge, and synaptic refractory kinetics")
        .with_param(DspParamSchema::knob("membrane_threshold_mv", "Soma Action Potential Threshold", -60.0, -30.0, -50.0, "mV", MacroRole::Tone, "Voltage threshold triggering explosive membrane depolarization and spike"))
        .with_param(DspParamSchema::knob("leak_conductance_ns", "Membrane Leaky Conductance", 1.0, 50.0, 10.0, "nS", MacroRole::Character, "Passive ion channel leak rate determining integration time constant"))
        .with_param(DspParamSchema::knob("synaptic_current_na", "Injected Synaptic Drive Current", 0.0, 10.0, 2.5, "nA", MacroRole::Punch, "Input synaptic current setting baseline spike firing frequency"))
        .with_param(DspParamSchema::knob("refractory_period_ms", "Post-Spike Absolute Refractory Time", 0.5, 10.0, 2.0, "ms", MacroRole::Punch, "Time window following spike discharge where neuron cannot fire again"))
        .with_param(DspParamSchema::knob("spike_pulse_width_us", "Action Potential Spike Duration", 50.0, 1000.0, 250.0, "us", MacroRole::Tone, "Temporal width of synthesized action potential voltage pulse"))
        .with_param(DspParamSchema::knob("synaptic_noise_std_dev", "Membrane Voltage Thermal Noise Jitter", 0.0, 5.0, 0.8, "mV", MacroRole::Character, "Stochastic Gaussian voltage noise adding biological irregularity"))
    );
    descriptors.insert("SubharmonicBassSynthesizerEngine".to_string(), DspNodeDescriptor::new("SubharmonicBassSynthesizerEngine", "Phase-Locked Dual-Octave Subharmonic Bass Generator", DspNodeCategory::Oscillator, "Precision low-frequency psychoacoustic bass enhancer tracking input fundamental pitch and synthesizing phase-locked -1 and -2 octave sinusoidal subharmonics")
        .with_param(DspParamSchema::knob("sub_octave_1_mix_db", "Subharmonic -1 Octave Synthesis Level", -36.0, 12.0, 0.0, "dB", MacroRole::Punch, "Level of synthesized -1 octave subharmonic sine wave"))
        .with_param(DspParamSchema::knob("sub_octave_2_mix_db", "Subharmonic -2 Octave Deep Bass Level", -36.0, 12.0, -6.0, "dB", MacroRole::Punch, "Level of synthesized -2 octave deep earthquake subharmonic"))
        .with_param(DspParamSchema::knob("sub_tracking_crossover_hz", "Pitch Tracking Frequency Cutoff", 40.0, 250.0, 90.0, "Hz", MacroRole::Tone, "Upper frequency boundary for fundamental pitch tracking detection"))
        .with_param(DspParamSchema::knob("phase_lock_offset_deg", "Subharmonic Phase Lock Alignment", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Phase angle alignment between synthesized subharmonics and dry fundamental"))
        .with_param(DspParamSchema::knob("saturation_drive_sub", "Subharmonic Upper Harmonic Saturation", 0.0, 1.0, 0.35, "sat", MacroRole::Character, "Soft harmonic overdrive generating upper odd harmonics for small speaker audibility"))
        .with_param(DspParamSchema::toggle("direct_dry_sub_crossover_cut", "Highpass Filter Weak Dry Bass", true, "Applies steep high-pass filter to dry audio removing weak fundamental to prevent phase clash"))
    );
    descriptors.insert("PolyphonicGranularCloudSynthesizer".to_string(), DspNodeDescriptor::new("PolyphonicGranularCloudSynthesizer", "Polyphonic Asynchronous Granular Cloud Synthesizer", DspNodeCategory::Oscillator, "Micro-sound texture generator scattering thousands of overlapping grains with stochastic pitch, playback position, stereo pan, and grain envelope shaping")
        .with_param(DspParamSchema::knob("grain_density_grains_per_sec", "Asynchronous Grain Cloud Density", 1.0, 500.0, 65.0, "gr/s", MacroRole::Character, "Number of stochastic grains triggered per second forming acoustic texture"))
        .with_param(DspParamSchema::knob("grain_duration_ms", "Individual Grain Window Duration", 5.0, 1000.0, 85.0, "ms", MacroRole::Space, "Temporal length of each discrete grain envelope"))
        .with_param(DspParamSchema::knob("stochastic_pitch_jitter_semitones", "Grain Pitch Randomization Variance", 0.0, 24.0, 0.0, "st", MacroRole::Tone, "Random pitch displacement applied across individual grains in cloud"))
        .with_param(DspParamSchema::knob("stereo_spatial_dispersion", "Stereo Panorama Cloud Width", 0.0, 1.0, 0.7, "%", MacroRole::Space, "Spatial pan spread randomly positioning grains across stereo panorama"))
        .with_param(DspParamSchema::choice("grain_envelope_shape", "Grain Windowing Envelope Geometry", &["Hann Window", "Gaussian Bell", "Tukey 25%", "Exponential Decay"], 0, "Mathematical window function tapering grain onset and release to eliminate clicks"))
        .with_param(DspParamSchema::toggle("freeze_buffer_playback", "Freeze Audio Buffer Grain Loop", false, "Freezes position pointer creating endless stationary timbral ambient drone"))
    );
    descriptors.insert("PlasmaArcSonificationSynthesizer".to_string(), DspNodeDescriptor::new("PlasmaArcSonificationSynthesizer", "Singing High-Voltage Plasma Arc Acoustic Spark Synthesizer", DspNodeCategory::Oscillator, "Electro-acoustic audio transducer and synthesis model simulating thermal air expansion across an oscillating high-voltage electrical plasma discharge")
        .with_param(DspParamSchema::knob("arc_breakdown_voltage_kv", "Dielectric Spark Breakdown Voltage", 1.0, 50.0, 15.0, "kV", MacroRole::Punch, "Threshold voltage ionizing air gap and igniting singing plasma conduction channel"))
        .with_param(DspParamSchema::knob("plasma_channel_current_ma", "Conduction Channel Arc Current", 1.0, 200.0, 35.0, "mA", MacroRole::Character, "Continuous electrical current setting thermal volume expansion and acoustic loudness"))
        .with_param(DspParamSchema::knob("spark_gap_distance_mm", "Electrode Spark Gap Distance", 0.5, 30.0, 8.0, "mm", MacroRole::Tone, "Physical distance between tungsten discharge electrodes dictating arc resistance"))
        .with_param(DspParamSchema::knob("ionization_thermal_inertia_us", "Thermal Plasma Relaxation Inertia", 1.0, 500.0, 45.0, "us", MacroRole::Tone, "Thermal lag smoothing audio transitions and rolling off ultrasonic harmonics"))
        .with_param(DspParamSchema::knob("corona_discharge_crackle_ratio", "Corona Discharge Stochastic Hiss", 0.0, 1.0, 0.15, "%", MacroRole::Character, "Micro-spark corona discharge sputtering around sharp needle tips"))
        .with_param(DspParamSchema::toggle("enable_high_frequency_spark_singing", "High-Frequency Ultrasonic PWM Singing", true, "Carrier modulation singing mode producing shimmering crystalline harmonics"))
    );
    descriptors.insert("SupercriticalFluidPhaseNoise".to_string(), DspNodeDescriptor::new("SupercriticalFluidPhaseNoise", "Supercritical CO2 Phase-Transition Cavitation Noise Generator", DspNodeCategory::Oscillator, "Thermodynamic state-space noise synthesizer generating non-Gaussian stochastic fluctuations near critical point vapor-liquid transitions")
        .with_param(DspParamSchema::knob("critical_point_temperature_k", "Fluid Thermodynamic Temperature", 250.0, 450.0, 304.13, "K", MacroRole::Tone, "Temperature relative to critical point (304.13 K for CO2) tuning noise character"))
        .with_param(DspParamSchema::knob("critical_pressure_megapascal", "Fluid Thermodynamic Static Pressure", 5.0, 20.0, 7.38, "MPa", MacroRole::Punch, "Operating pressure crossing critical boundary (7.38 MPa for CO2)"))
        .with_param(DspParamSchema::knob("opalescence_density_fluctuation", "Critical Opalescence Density Fluctuation", 0.0, 1.0, 0.55, "%", MacroRole::Character, "Density cluster correlation length causing divergent acoustic scattering"))
        .with_param(DspParamSchema::knob("cavitation_bubble_collapse_rate", "Micro-Cavitation Bubble Collapse Rate", 10.0, 5000.0, 420.0, "Hz", MacroRole::Space, "Rate of transient microbubble formations and implosions in fluid medium"))
        .with_param(DspParamSchema::knob("spectral_entropy_color_slope", "Noise Power Spectral Entropy Slope", -6.0, 6.0, -1.5, "dB/oct", MacroRole::Tone, "Power spectral density tilt ranging from brownian rumble to violet hiss"))
        .with_param(DspParamSchema::toggle("enable_critical_opalescence_resonance", "Critical Point Divergence Resonance", true, "Amplifies resonant density fluctuations as temperature approaches critical threshold"))
    );
    descriptors.insert("GravitationalWaveBinaryChirp".to_string(), DspNodeDescriptor::new("GravitationalWaveBinaryChirp", "Compact Binary Inspiral Quadrupole Gravitational Wave Chirp Synth", DspNodeCategory::Oscillator, "Post-Newtonian gravitational wave waveform model synthesizing binary black hole / neutron star inspiral chirp, merger plunge, and ringdown quasinormal modes")
        .with_param(DspParamSchema::knob("primary_component_mass_solar", "Primary Component Mass (Solar Masses)", 1.4, 100.0, 36.0, "M_sun", MacroRole::Punch, "Mass of primary compact object shaping orbital frequency acceleration"))
        .with_param(DspParamSchema::knob("secondary_component_mass_solar", "Secondary Component Mass (Solar Masses)", 1.4, 100.0, 29.0, "M_sun", MacroRole::Tone, "Mass of companion compact object setting binary mass ratio"))
        .with_param(DspParamSchema::knob("inspiral_starting_frequency_hz", "Inspiral Horizon Starting Frequency", 15.0, 200.0, 32.0, "Hz", MacroRole::Tone, "Starting orbital gravitational wave frequency entering audible detector band"))
        .with_param(DspParamSchema::knob("orbital_eccentricity_ratio", "Keplerian Orbital Eccentricity", 0.0, 0.8, 0.05, "e", MacroRole::Character, "Non-circular orbital eccentricity introducing periodic harmonic sidebands"))
        .with_param(DspParamSchema::knob("ringdown_quasinormal_damping_q", "Post-Merger Ringdown Q-Factor", 1.0, 50.0, 12.0, "Q", MacroRole::Space, "Quality factor of damped exponential sinusoidal ringdown of merged remnant"))
        .with_param(DspParamSchema::toggle("enable_spin_precession_polarization", "Orbital Plane Spin Precession Modulation", true, "Modulates plus and cross polarization amplitudes from tilted angular momentum"))
    );
    descriptors.insert("CasimirQuantumVacuumNoise".to_string(), DspNodeDescriptor::new("CasimirQuantumVacuumNoise", "Quantum Electrodynamic Vacuum Fluctuation & Casimir Force Noise", DspNodeCategory::Oscillator, "Zero-point quantum fluctuation noise source modulated by nanometer-spaced conductive plate boundaries, exhibiting high-frequency mode exclusion")
        .with_param(DspParamSchema::knob("plate_separation_distance_nm", "Conductive Plate Gap Separation", 5.0, 500.0, 45.0, "nm", MacroRole::Tone, "Distance between parallel conductive boundary plates cutting off long vacuum modes"))
        .with_param(DspParamSchema::knob("quantum_vacuum_field_intensity", "Zero-Point Vacuum Field Intensity", 0.01, 2.0, 0.85, "E_0", MacroRole::Punch, "Overall amplitude of virtual photon electromagnetic field fluctuations"))
        .with_param(DspParamSchema::knob("thermal_blackbody_correction_k", "Thermal Photon Correction Temperature", 0.0, 500.0, 77.0, "K", MacroRole::Tone, "Planck blackbody radiation spectrum blending into zero-point quantum noise"))
        .with_param(DspParamSchema::knob("finite_conductivity_skin_depth", "Boundary Plate Metal Skin Depth", 0.1, 50.0, 12.0, "nm", MacroRole::Character, "Finite electrical conductivity of metallic mirrors softening boundary cutoffs"))
        .with_param(DspParamSchema::knob("dynamic_casimir_photon_production", "Dynamical Casimir Real Photon Generation", 0.0, 1.0, 0.18, "%", MacroRole::Space, "Parametric creation of real photons from relativistic boundary acceleration"))
        .with_param(DspParamSchema::toggle("enable_sub_kelvin_ground_state", "Pure Zero-Kelvin Quantum Ground State", true, "Removes classical thermal noise leaving purely quantum mechanical fluctuations"))
    );
    descriptors.insert("ThermonuclearFusionResonator".to_string(), DspNodeDescriptor::new("ThermonuclearFusionResonator", "Tokamak Magnetic Confinement Fusion Plasma Resonance Synthesizer", DspNodeCategory::Oscillator, "Deuterium-tritium high-temperature plasma simulator synthesizing ion cyclotron resonance frequencies, neoclassical tearing modes, and magnetic field ripple")
        .with_param(DspParamSchema::knob("toroidal_magnetic_field_tesla", "Toroidal Confinement Magnetic Field", 1.0, 15.0, 5.3, "T", MacroRole::Punch, "Toroidal magnetic field strength setting ion cyclotron gyro-frequency harmonics"))
        .with_param(DspParamSchema::knob("plasma_core_temperature_kev", "Core Plasma Kinetic Temperature", 1.0, 50.0, 15.0, "keV", MacroRole::Tone, "Core thermal temperature (15 keV = 150 million Kelvin) driving thermal velocity"))
        .with_param(DspParamSchema::knob("ion_cyclotron_resonance_harmonic", "Ion Cyclotron Gyro-Frequency Harmonic", 1.0, 16.0, 3.0, "harm", MacroRole::Tone, "Harmonic overtone multiplier for radiofrequency plasma heating resonance"))
        .with_param(DspParamSchema::knob("neoclassical_tearing_mode_drift", "Magnetic Island Tearing Mode Drift", 0.1, 20.0, 4.2, "Hz", MacroRole::Character, "Rotational frequency of magnetic reconnection islands introducing subharmonic throbbing"))
        .with_param(DspParamSchema::knob("plasma_current_megaamperes", "Poloidal Ohmic Heating Plasma Current", 0.5, 20.0, 7.5, "MA", MacroRole::Punch, "Total plasma current generating poloidal field and driving high-energy saturation"))
        .with_param(DspParamSchema::toggle("enable_h_mode_edge_transport_barrier", "High-Confinement H-Mode Edge Barrier", true, "Sharpens plasma edge boundary creating intense harmonic definition"))
    );
    descriptors.insert("QuantumAnnealingOptimizerSynth".to_string(), DspNodeDescriptor::new("QuantumAnnealingOptimizerSynth", "Quantum Annealing Energy Minimization Landscape Audio Synthesizer", DspNodeCategory::Oscillator, "Simulates adiabatic quantum annealing through multi-dimensional energy potential landscapes to generate evolving harmonic trajectories")
        .with_param(DspParamSchema::knob("annealing_schedule_time_ms", "Annealing Schedule Cooling Duration", 10.0, 5000.0, 250.0, "ms", MacroRole::Tone, "Cooling duration of transverse magnetic field driving quantum tunneling"))
        .with_param(DspParamSchema::knob("transverse_field_strength", "Transverse Magnetic Tunneling Field", 0.01, 10.0, 1.5, "T", MacroRole::Punch, "Initial transverse field strength driving wavepacket barrier penetration"))
        .with_param(DspParamSchema::knob("spin_glass_qubit_count", "Ising Spin Glass Qubit Dimensions", 4.0, 64.0, 16.0, "qubits", MacroRole::Character, "Total interacting qubit spins constructing optimization landscape topology"))
        .with_param(DspParamSchema::knob("thermal_fluctuation_noise", "Thermal Bath Quantum Decoherence", 0.0, 1.0, 0.15, "%", MacroRole::Space, "Environmental thermal fluctuations inducing stochastic state transitions"))
        .with_param(DspParamSchema::knob("ground_state_convergence_ratio", "Ground State Convergence Factor", 0.1, 1.0, 0.85, "%", MacroRole::Tone, "Degree of adiabaticity ensuring system settles in true global energy minimum"))
        .with_param(DspParamSchema::toggle("enable_quantum_tunneling_transitions", "Enable Non-Local Quantum Tunneling", true, "Allows barrier traversal without requiring classic thermal barrier hopping"))
    );
    descriptors.insert("RiemannSurfaceComplexOscillator".to_string(), DspNodeDescriptor::new("RiemannSurfaceComplexOscillator", "Conformal Mapping Multi-Sheeted Riemann Surface Synthesizer", DspNodeCategory::Oscillator, "Mathematical oscillator tracing branch points and spiral sheet topologies across complex Riemann surfaces to produce dense evolving phase shifts")
        .with_param(DspParamSchema::knob("branch_point_order_n", "Riemann Branch Point Order (n)", 1.0, 8.0, 2.0, "sheets", MacroRole::Tone, "Number of interwoven Riemannian sheets traversed per full orbital revolution"))
        .with_param(DspParamSchema::knob("conformal_pole_offset_real", "Complex Pole Real Axis Offset", -2.0, 2.0, 0.25, "Re", MacroRole::Punch, "Real component coordinate of holomorphic pole singularity in complex plane"))
        .with_param(DspParamSchema::knob("conformal_pole_offset_imag", "Complex Pole Imaginary Offset", -2.0, 2.0, 0.75, "Im", MacroRole::Punch, "Imaginary component coordinate of pole singularity shaping winding resonance"))
        .with_param(DspParamSchema::knob("spiral_winding_number", "Surface Winding Angular Number", 1.0, 12.0, 3.0, "turns", MacroRole::Space, "Rate of spiral sheet progression across logarithmic helical cuts"))
        .with_param(DspParamSchema::knob("analytic_continuation_drift", "Analytic Continuation Drift Rate", 0.0, 1.0, 0.2, "%", MacroRole::Character, "Continuous parametric modulation rate across branch cuts generating evolving timbre"))
        .with_param(DspParamSchema::toggle("enable_biholomorphic_mapping", "Biholomorphic Invariant Normalization", true, "Preserves angles and harmonicity across multi-sheeted conformal projections"))
    );
    descriptors.insert("BicubicInterpolatedWavetableMorph".to_string(), DspNodeDescriptor::new("BicubicInterpolatedWavetableMorph", "2D Grid Bicubic Spline Continuous Wavetable Morpher", DspNodeCategory::Oscillator, "Ultra-smooth wavetable synthesizer interpolating across a 2D matrix of single-cycle waveforms using bicubic surface splines with zero aliasing")
        .with_param(DspParamSchema::knob("wavetable_grid_coord_x", "Wavetable Terrain X-Axis Morpher", 0.0, 1.0, 0.0, "X", MacroRole::Tone, "Continuous horizontal position across 2D wavetable matrix terrain"))
        .with_param(DspParamSchema::knob("wavetable_grid_coord_y", "Wavetable Terrain Y-Axis Morpher", 0.0, 1.0, 0.0, "Y", MacroRole::Space, "Continuous vertical position across 2D wavetable matrix terrain"))
        .with_param(DspParamSchema::knob("spectral_anti_aliasing_guard", "Band-Limited Harmonic Cutoff Margin", 0.5, 1.0, 0.95, "%", MacroRole::Tone, "Dynamic anti-aliasing guard eliminating Nyquist foldover across all pitches"))
        .with_param(DspParamSchema::knob("spline_tension_parameter", "Bicubic Spline Curve Tension", 0.0, 1.0, 0.5, "tau", MacroRole::Character, "Tension parameter of bicubic Catmull-Rom spline preventing waveform overshoot"))
        .with_param(DspParamSchema::knob("sub_harmonic_foundation_level", "Sub-Harmonic Foundation Sine Reinforce", 0.0, 1.0, 0.25, "%", MacroRole::Punch, "Pure sine wave reinforcer anchoring low fundamental during wild morphs"))
        .with_param(DspParamSchema::toggle("enable_continuous_gradient_rotation", "Continuous Autonomous Terrain Rotation", false, "Automates smooth circular orbiting across 2D wavetable plane at LFO rates"))
    );
    descriptors.insert("ChaoticChuaCircuitOscillator".to_string(), DspNodeDescriptor::new("ChaoticChuaCircuitOscillator", "Chua Circuit Double-Scroll Strange Attractor Generator", DspNodeCategory::Oscillator, "Autonomous electronic non-linear Chua oscillator generating chaotic bifurcation, period doubling, and wild analog double-scroll trajectories")
        .with_param(DspParamSchema::knob("chua_resistor_conductance_g", "Chua Resistor Conductance (G)", 0.1, 2.0, 0.7, "mS", MacroRole::Tone, "Linear coupling conductance between primary and secondary resonant LC tanks"))
        .with_param(DspParamSchema::knob("inner_capacitance_c1_nf", "Primary Tank Capacitance (C1)", 1.0, 50.0, 10.0, "nF", MacroRole::Tone, "Capacitance across Chua nonlinear diode governing attractor orbit speed"))
        .with_param(DspParamSchema::knob("outer_capacitance_c2_nf", "Coupled Tank Capacitance (C2)", 10.0, 500.0, 100.0, "nF", MacroRole::Space, "Secondary parallel capacitance setting low-frequency chaotic lobe transitions"))
        .with_param(DspParamSchema::knob("chua_diode_break_slope_ga", "Inner Piecewise Diode Slope (Ga)", -1.5, -0.1, -0.75, "slope", MacroRole::Punch, "Inner negative conductance slope driving spontaneous self-oscillation"))
        .with_param(DspParamSchema::knob("chua_diode_outer_slope_gb", "Outer Piecewise Diode Slope (Gb)", -0.8, 0.5, -0.41, "slope", MacroRole::Character, "Outer saturation conductance bounding attractor trajectory inside double-scroll"))
        .with_param(DspParamSchema::toggle("enable_double_scroll_attractor_lock", "Enforce Chaotic Double-Scroll Regime", true, "Constrains component ratios into proven chaotic double-scroll strange attractor state"))
    );
    descriptors.insert("SubwooferHarmonicSynthesizer".to_string(), DspNodeDescriptor::new("SubwooferHarmonicSynthesizer", "Infrasonic Sub-Harmonic Waveform Extender & Chest-Thump Synth", DspNodeCategory::Oscillator, "Generates phase-locked sub-octaves (-1 oct, -2 oct) from input bass lines using zero-crossing tracking, dynamic waveshaping, and tight low-end envelope gating")
        .with_param(DspParamSchema::knob("sub_octave_1_mix_level_db", "Sub-Octave -1 Octave Synthesis Level", -40.0, 6.0, -3.0, "dB", MacroRole::Punch, "Volume level of first synthesized sub-octave (half fundamental frequency)"))
        .with_param(DspParamSchema::knob("sub_octave_2_infrasonic_level_db", "Sub-Octave -2 Infrasonic Rumble Level", -40.0, 6.0, -12.0, "dB", MacroRole::Punch, "Volume level of second synthesized sub-octave (quarter fundamental frequency)"))
        .with_param(DspParamSchema::knob("pitch_tracking_lock_speed_ms", "Zero-Crossing Pitch Tracking Response", 1.0, 50.0, 6.0, "ms", MacroRole::Tone, "Speed of pitch detector zero-crossing tracker locking to input fundamental"))
        .with_param(DspParamSchema::knob("harmonic_saturation_overtones_db", "Sub-Bass Upper Harmonic Saturation", 0.0, 24.0, 6.0, "dB", MacroRole::Character, "Saturates sub-bass to make low frequencies audible on small speakers"))
        .with_param(DspParamSchema::knob("sub_bass_lowpass_cutoff_hz", "Infrasonic Sub-Bass Lowpass Filter", 30.0, 160.0, 85.0, "Hz", MacroRole::Tone, "Steep lowpass filter confining generated sub-octaves to subwoofer band"))
        .with_param(DspParamSchema::toggle("enable_monophonic_bass_phase_alignment", "Monophonic Center Phase-Locking", true, "Forces generated sub-harmonics to mono center and phase-locks to dry bass transient"))
    );
    descriptors.insert("InfrasonicSubBassExciter".to_string(), DspNodeDescriptor::new("InfrasonicSubBassExciter", "Infrasonic Pitch-Locked Sub-Bass Synthesizer", DspNodeCategory::Oscillator, "Tracks fundamental bass notes in real-time and synthesizes ultra-clean phase-aligned 20-60 Hz sine/triangle infrasonic sub-bass with dynamic pitch glides and drive")
        .with_param(DspParamSchema::knob("sub_frequency_floor_hz", "Sub-Bass Lower Infrasonic Floor", 20.0, 80.0, 35.0, "Hz", MacroRole::Punch, "Target lower frequency cutoff limit for synthetic sub-bass generation"))
        .with_param(DspParamSchema::knob("infrasonic_output_level_db", "Infrasonic Sub Waveform Output Gain", -36.0, 6.0, 0.0, "dB", MacroRole::Punch, "Output level of synthesized infrasonic sub-bass waveform"))
        .with_param(DspParamSchema::knob("pitch_tracking_latency_ms", "Fundamental Pitch Lock Speed", 2.0, 40.0, 8.0, "ms", MacroRole::Tone, "Zero-crossing pitch detector responsiveness and lock time"))
        .with_param(DspParamSchema::knob("harmonics_drive_saturation_db", "Audible Overtones Harmonic Saturation", 0.0, 18.0, 4.0, "dB", MacroRole::Character, "Even-order soft saturation ensuring audible definition on smaller speakers"))
        .with_param(DspParamSchema::knob("dynamic_envelope_follow_speed_ms", "Dynamics Envelope Follower Response", 5.0, 200.0, 30.0, "ms", MacroRole::Space, "Envelope follower attack/release tracking source bass instrument dynamics"))
        .with_param(DspParamSchema::toggle("enable_phase_lock_to_source", "Cycle-Accurate Center Phase-Locking", true, "Locks synthesized sub-oscillator phase to incoming kick or bass fundamental"))
    );
    descriptors.insert("PsychoacousticFundamentalSynthesizer".to_string(), DspNodeDescriptor::new("PsychoacousticFundamentalSynthesizer", "Psychoacoustic Missing Fundamental Synthesizer", DspNodeCategory::Oscillator, "Analyzes inaudible sub-bass program material and generates perceptually aligned harmonic overtone series exploiting acoustic residue pitch perception for small-speaker translation")
        .with_param(DspParamSchema::knob("target_virtual_sub_frequency_hz", "Target Sub-Bass Virtual Fundamental", 30.0, 100.0, 45.0, "Hz", MacroRole::Punch, "Target fundamental pitch perceived via generated upper harmonics"))
        .with_param(DspParamSchema::knob("harmonic_series_order_count", "Generated Harmonic Series Multiplier", 2.0, 8.0, 4.0, "harm", MacroRole::Tone, "Number of upper harmonic overtones synthesized above virtual sub"))
        .with_param(DspParamSchema::knob("harmonic_amplitude_decay_slope_db", "Harmonic Series Attenuation Slope", -12.0, -1.0, -4.5, "dB/oct", MacroRole::Character, "Spectral rolloff slope across generated harmonic overtone series"))
        .with_param(DspParamSchema::knob("sub_fundamental_cut_steepness_db", "Original Inaudible Sub Attenuation", 0.0, 48.0, 18.0, "dB", MacroRole::Punch, "Attenuates raw inaudible sub-bass preventing small speaker distortion"))
        .with_param(DspParamSchema::knob("sidechain_crossover_frequency_hz", "Sub-Bass Detection Crossover", 50.0, 250.0, 90.0, "Hz", MacroRole::Tone, "High cutoff of input sidechain isolating sub-bass triggers"))
        .with_param(DspParamSchema::toggle("enable_odd_harmonic_interlock", "Odd/Even Harmonic Dynamic Interlock", true, "Maintains psychoacoustic balance between 2nd, 3rd, and 4th harmonics"))
    );
    descriptors.insert("GaussianStochasticGrainSynthesizer".to_string(), DspNodeDescriptor::new("GaussianStochasticGrainSynthesizer", "Gaussian Stochastic Granular Cloud Synthesizer", DspNodeCategory::Oscillator, "Synthesizes complex ambient textures and organic clusters by emitting micro-grains whose frequency, duration, amplitude, and spatial position follow Gaussian random distributions")
        .with_param(DspParamSchema::knob("grain_emission_density_hz", "Average Grain Cloud Generation Rate", 2.0, 300.0, 65.0, "Hz", MacroRole::Punch, "Mean emission rate of overlapping micro-grains per second"))
        .with_param(DspParamSchema::knob("center_frequency_pitch_hz", "Gaussian Pitch Distribution Center", 40.0, 4000.0, 440.0, "Hz", MacroRole::Tone, "Center fundamental frequency of the granular pitch cloud"))
        .with_param(DspParamSchema::knob("frequency_variance_spread_semitones", "Pitch Gaussian Spread Standard Deviation", 0.0, 24.0, 3.5, "st", MacroRole::Tone, "Standard deviation of random Gaussian pitch detuning across grains"))
        .with_param(DspParamSchema::knob("grain_duration_mean_ms", "Mean Grain Envelope Window Duration", 5.0, 500.0, 45.0, "ms", MacroRole::Space, "Average duration of individual micro-grain Gaussian envelopes"))
        .with_param(DspParamSchema::knob("spatial_binaural_spread_angle_deg", "Stereo Cloud Azimuth Angular Scatter", 0.0, 180.0, 90.0, "deg", MacroRole::Space, "Stereo panorama angular dispersion standard deviation"))
        .with_param(DspParamSchema::toggle("enable_quasi_synchronous_bursts", "Rhythmic Synchronous Burst Mode", false, "Quantizes grain emission into periodic rhythmic bursts rather than Poisson noise"))
    );
    descriptors.insert("HyperdimensionalQuaternionOscillator".to_string(), DspNodeDescriptor::new("HyperdimensionalQuaternionOscillator", "4D Quaternion Rotation Waveform Oscillator", DspNodeCategory::Oscillator, "Hyper-dimensional four-vector oscillator rotating across 4D hyperspherical space with S3 manifold projection into stereo quadrature audio channels")
        .with_param(DspParamSchema::knob("quaternion_rotation_speed_hz", "4D Rotor Frequency Speed", 0.1, 2000.0, 110.0, "Hz", MacroRole::Tone, "Governs angular velocity of rotation through 4D quaternion hyperspace"))
        .with_param(DspParamSchema::knob("stereographic_projection_pole", "Hyperspherical Projection Focal Pole", -2.0, 2.0, 1.0, "coord", MacroRole::Character, "Adjusts geometric pole for 4D stereographic projection onto 2D audio planes"))
        .with_param(DspParamSchema::knob("quaternion_rotor_axis_tilt", "Euler-Rodrigues Axis Nutation", 0.0, 360.0, 45.0, "deg", MacroRole::Space, "Inclination angle of the 4D rotation axis producing harmonic precession"))
        .with_param(DspParamSchema::knob("spinor_phase_warp_curvature", "Spinor Trajectory Phase Curvature", 0.0, 1.0, 0.25, "%", MacroRole::Punch, "Nonlinear phase curvature morphing sinusoidal orbits toward Chebyshev harmonics"))
        .with_param(DspParamSchema::knob("hypersurface_symmetry_folding", "4D Manifold Symmetry Fold Count", 1.0, 8.0, 2.0, "folds", MacroRole::Tone, "Folds 4D surface trajectories introducing complex sub-harmonic ripples"))
        .with_param(DspParamSchema::toggle("enable_symplectic_energy_conservation", "Symplectic Integrator Phase Lock", true, "Preserves total Hamiltonian orbital energy eliminating numerical amplitude drift"))
    );
    descriptors.insert("SubharmonicBassReinforcementExciter".to_string(), DspNodeDescriptor::new("SubharmonicBassReinforcementExciter", "Sub-Harmonic Resonant Bass Reinforcement", DspNodeCategory::Oscillator, "Phase-locked sub-octave synthesizer generating sinusoidal fundamental tones one octave below input bass transients with envelope following and tuned harmonic distortion")
        .with_param(DspParamSchema::knob("sub_octave_blend_gain_db", "Sub-Harmonic Output Blend Level", -30.0, 12.0, 0.0, "dB", MacroRole::Punch, "Gain of generated sub-octave fundamental signal"))
        .with_param(DspParamSchema::knob("sub_tracking_lowpass_cutoff_hz", "Pitch Tracker Input Low-Pass Cutoff", 40.0, 250.0, 110.0, "Hz", MacroRole::Tone, "Isolates clean fundamental frequencies for tracking"))
        .with_param(DspParamSchema::knob("envelope_attack_snap_ms", "Dynamic Envelope Attack Tracking Time", 0.5, 50.0, 4.0, "ms", MacroRole::Punch, "Speed at which sub-synth tracks the envelope onset of kick and bass"))
        .with_param(DspParamSchema::knob("sub_waveform_saturation_drive", "Sub Waveform Harmonic Distortion Drive", 0.0, 18.0, 3.0, "dB", MacroRole::Character, "Saturates generated pure sine wave adding audible second harmonic"))
        .with_param(DspParamSchema::knob("phase_alignment_offset_deg", "Sub-To-Dry Phase Alignment Offset", -180.0, 180.0, 0.0, "deg", MacroRole::Space, "Adjusts phase relationship eliminating destructive bass cancellation"))
        .with_param(DspParamSchema::toggle("enable_infrasonic_highpass_guard", "18Hz Infrasonic Subwoofer High-Pass Guard", true, "Rolls off inaudible DC offset and power-wasting frequencies below 18Hz"))
    );
    descriptors.insert("PolyphonicPulsarCloudSynthesizer".to_string(), DspNodeDescriptor::new("PolyphonicPulsarCloudSynthesizer", "Polyphonic Pulsar Cloud Synthesizer", DspNodeCategory::Oscillator, "Generative polyphonic pulsar synthesis engine generating micro-sound rhythmic pulsar trains with formant masking, stochastic burst scatter, and spatial aperture dispersion")
        .with_param(DspParamSchema::knob("pulsar_emission_rate_hz", "Pulsar Train Emission Rate", 1.0, 500.0, 80.0, "Hz", MacroRole::Tone, "Emission frequency of micro-sound pulsar trains"))
        .with_param(DspParamSchema::knob("pulsaret_duty_cycle_ratio", "Pulsaret Duty Cycle Formant Width", 0.01, 1.0, 0.25, "ratio", MacroRole::Character, "Duration ratio of individual pulsaret waveforms relative to fundamental period"))
        .with_param(DspParamSchema::knob("formant_masking_frequency_hz", "Formant Envelope Center Frequency", 100.0, 10000.0, 1800.0, "Hz", MacroRole::Tone, "Center resonance frequency for the bandpass formant filter envelope"))
        .with_param(DspParamSchema::knob("pulsar_burst_jitter_ms", "Stochastic Pulsar Interval Jitter", 0.0, 100.0, 15.0, "ms", MacroRole::Space, "Micro-timing scatter preventing periodic flutter and comb filtering"))
        .with_param(DspParamSchema::knob("stereo_pulsar_dispersion_width", "Stereo Soundstage Aperture Width", 0.0, 1.0, 0.8, "%", MacroRole::Space, "Stereo width distribution for pulsaret trains across left/right channels"))
        .with_param(DspParamSchema::toggle("enable_pulsaret_envelope_windowing", "Hann Window Envelope Smoothing", true, "Applies smooth Hann window envelope preventing click artifacts"))
    );
    descriptors.insert("StochasticChaoticJitterSynthesizer".to_string(), DspNodeDescriptor::new("StochasticChaoticJitterSynthesizer", "Stochastic Chaotic Jitter Frequency Synthesizer", DspNodeCategory::Oscillator, "High-stability digital oscillator modulated by fractional Brownian motion, thermal noise jitter, and chaotic fractal drift for organic analog timbre")
        .with_param(DspParamSchema::knob("oscillator_fundamental_pitch_hz", "Oscillator Carrier Pitch Frequency", 20.0, 8000.0, 220.0, "Hz", MacroRole::Tone, "Fundamental carrier oscillator frequency"))
        .with_param(DspParamSchema::knob("brownian_jitter_frequency_deviation_cents", "Brownian Motion Pitch Jitter Deviation", 0.0, 50.0, 8.0, "cents", MacroRole::Punch, "Stochastic frequency wander modeling vintage analog component drift"))
        .with_param(DspParamSchema::knob("fractal_hurst_exponent", "Fractal Drift Hurst Exponent (0.1-0.9)", 0.1, 0.9, 0.7, "H", MacroRole::Character, "Governs persistence of random walk from rough flicker to smooth drift"))
        .with_param(DspParamSchema::knob("thermal_amplitude_noise_db", "Thermal Shot Noise Floor Modulation", -90.0, -20.0, -54.0, "dB", MacroRole::Space, "Microscopic analog noise added to oscillator waveform"))
        .with_param(DspParamSchema::knob("oscillator_waveform_morph", "Waveform Morph (Sine -> Tri -> Saw -> Square)", 0.0, 3.0, 2.0, "wave", MacroRole::Tone, "Continuous geometric morphing across standard analog waveforms"))
        .with_param(DspParamSchema::toggle("enable_anti_aliased_polyblep", "PolyBLEP Band-Limited Edge Correction", true, "Suppresses digital aliasing artifacts on sharp waveform transitions"))
    );
    descriptors.insert("CavitationBubbleCollapseExciter".to_string(), DspNodeDescriptor::new("CavitationBubbleCollapseExciter", "Sonoluminescent Cavitation Bubble Collapse Synthesizer", DspNodeCategory::Oscillator, "Rayleigh-Plesset equation acoustic cavitation generator simulating high-energy micro-bubble collapse, extreme pressure spike transients, and thermal plasma photon sound impulses")
        .with_param(DspParamSchema::knob("ambient_liquid_static_pressure_bar", "Liquid Static Ambient Pressure", 0.5, 50.0, 1.01, "bar", MacroRole::Punch, "Static ambient liquid pressure governing cavitation bubble collapse acceleration"))
        .with_param(DspParamSchema::knob("acoustic_driving_amplitude_kpa", "Ultrasonic Drive Acoustic Amplitude", 10.0, 500.0, 135.0, "kPa", MacroRole::Tone, "Amplitude of ultrasonic acoustic wave driving fluid into cavitation"))
        .with_param(DspParamSchema::knob("bubble_equilibrium_radius_microns", "Bubble Equilibrium Radius", 1.0, 100.0, 5.0, "um", MacroRole::Character, "Initial resting radius of gas micro-bubble prior to acoustic excitation"))
        .with_param(DspParamSchema::knob("liquid_surface_tension_dyn_cm", "Fluid Solvent Surface Tension", 10.0, 80.0, 72.8, "dyn/cm", MacroRole::Tone, "Surface tension resisting bubble expansion and accelerating final collapse shock"))
        .with_param(DspParamSchema::knob("bubble_collapse_implosion_gain_db", "Cavitation Implosion Transient Output", -48.0, 24.0, 0.0, "dB", MacroRole::Space, "Post-implosion acoustic shockwave impulse gain level"))
        .with_param(DspParamSchema::toggle("enable_polytropic_gas_compression", "Polytropic Gas Heating & Plasma Luminescence", true, "Simulates adiabatic gas heating and extreme shock temperatures during final collapse"))
    );
    descriptors.insert("MicrotonalHarmonicLatticeOscillator".to_string(), DspNodeDescriptor::new("MicrotonalHarmonicLatticeOscillator", "Multi-Dimensional Just Intonation Harmonic Lattice", DspNodeCategory::Oscillator, "7-limit and 11-limit prime-factor tonal lattice generator calculating just intonation intervals, comma pumps, and psychoacoustic harmonic alignment chords")
        .with_param(DspParamSchema::knob("lattice_base_tuning_frequency_hz", "Lattice Master Root Pitch", 100.0, 880.0, 216.0, "Hz", MacroRole::Tone, "Base reference tuning frequency for tonic note 1/1"))
        .with_param(DspParamSchema::knob("prime_limit_harmonic_expansion", "Prime Factor Harmonic Horizon (3-19)", 3.0, 19.0, 7.0, "limit", MacroRole::Character, "Maximum prime number permitted in frequency ratios (e.g. 5-limit, 7-limit, 11-limit)"))
        .with_param(DspParamSchema::knob("syntonic_comma_drift_depth_cents", "Syntonic Comma Microtonal Offset", -50.0, 50.0, 0.0, "cents", MacroRole::Space, "Microtonal drift offset simulating historical comma accumulation"))
        .with_param(DspParamSchema::knob("overtone_spectral_richness", "Harmonic Series Falloff Exponent", 0.1, 2.0, 1.0, "exp", MacroRole::Tone, "Amplitude rolloff across higher just harmonics within each lattice voice"))
        .with_param(DspParamSchema::knob("chordal_voice_spread_count", "Active Microtonal Voices Count", 1.0, 16.0, 6.0, "voices", MacroRole::Punch, "Number of simultaneous just-intoned polyphonic voices sounding across the lattice"))
        .with_param(DspParamSchema::toggle("enable_pythagorean_wolf_correction", "Pythagorean Wolf Fifth Correction", true, "Attenuates beating in unresolved circular comma closures"))
    );
    descriptors.insert("PsychoacousticBarkScaleSubBassSynthesizer".to_string(), DspNodeDescriptor::new("PsychoacousticBarkScaleSubBassSynthesizer", "Bark Critical-Band Sub-Bass Psychoacoustic Generator", DspNodeCategory::Oscillator, "Missing fundamental harmonic synthesizer generating upper critical-band overtones that fool the human ear into hearing deep sub-bass frequencies on compact speakers")
        .with_param(DspParamSchema::knob("target_perceived_fundamental_hz", "Virtual Perceived Sub-Bass Pitch", 25.0, 120.0, 42.0, "Hz", MacroRole::Punch, "Sub-bass fundamental frequency synthesized through harmonic illusion"))
        .with_param(DspParamSchema::knob("bark_critical_band_spacing", "Harmonic Critical-Band Spacing Ratio", 1.0, 3.0, 1.35, "bark", MacroRole::Tone, "Harmonic spacing optimized to stimulate adjacent Bark filter channels in the cochlea"))
        .with_param(DspParamSchema::knob("harmonic_odd_even_color_blend", "Odd vs Even Overtone Distribution", -1.0, 1.0, -0.2, "color", MacroRole::Character, "Blends between warm tube-like even overtones and punchy aggressive odd overtones"))
        .with_param(DspParamSchema::knob("sub_harmonic_mix_level_db", "Direct Pure Sub Sine Injection", -60.0, 0.0, -12.0, "dB", MacroRole::Punch, "Direct sub-audible fundamental sine wave added to mix for capable subwoofers"))
        .with_param(DspParamSchema::knob("psychoacoustic_overtone_saturation_db", "Virtual Harmonic Drive Amplification", 0.0, 24.0, 8.0, "dB", MacroRole::Tone, "Drive level applied to generated psychoacoustic upper overtone series"))
        .with_param(DspParamSchema::toggle("enable_speaker_highpass_protection", "Speaker Excursion Protection High-Pass", true, "Removes sub-audible low frequencies below 40Hz preventing small speaker distortion"))
    );
    descriptors.insert("ChaoticStrangeAttractorSynthesizer".to_string(), DspNodeDescriptor::new("ChaoticStrangeAttractorSynthesizer", "Lorenz & Rössler Strange Attractor Audio Oscillator", DspNodeCategory::Oscillator, "Nonlinear dynamic differential equation solver generating deterministic chaotic orbits (Lorenz, Rössler, Chua) projected into audible audio frequencies")
        .with_param(DspParamSchema::knob("attractor_fundamental_frequency_hz", "Chaotic Orbit Integration Rate", 10.0, 4000.0, 165.0, "Hz", MacroRole::Tone, "Speed of numerical trajectory integration through phase space"))
        .with_param(DspParamSchema::knob("prandtl_parameter_sigma", "Lorenz Sigma Prandtl Number", 1.0, 30.0, 10.0, "sigma", MacroRole::Punch, "Ratio of momentum diffusivity to thermal diffusivity"))
        .with_param(DspParamSchema::knob("rayleigh_parameter_rho", "Lorenz Rho Rayleigh Number", 5.0, 60.0, 28.0, "rho", MacroRole::Tone, "Dimensionless number driving the system past bifurcation into chaos"))
        .with_param(DspParamSchema::knob("geometric_parameter_beta", "Lorenz Beta Geometric Damping", 0.5, 5.0, 2.667, "beta", MacroRole::Character, "Geometric aspect ratio parameter shaping the attractor butterfly wings"))
        .with_param(DspParamSchema::knob("trajectory_nonlinearity_drive", "Attractor Orbit Non-Linearity Drive", 0.1, 5.0, 1.2, "drive", MacroRole::Space, "Non-linear cross-coupling intensity between X, Y, and Z phase trajectories"))
        .with_param(DspParamSchema::toggle("enable_chua_double_scroll_mode", "Chua Double-Scroll Attractor Mode", false, "Switches differential equation topology from Lorenz to piecewise-linear Chua circuit"))
    );
    descriptors.insert("DynamicSubharmonicResonanceExciter".to_string(), DspNodeDescriptor::new("DynamicSubharmonicResonanceExciter", "Adaptive Frequency-Division Subharmonic Synthesizer", DspNodeCategory::Oscillator, "Real-time digital subharmonic generator using envelope-followed flip-flop frequency division, anti-aliased integration, and dynamic phase lock for chest-thumping bass")
        .with_param(DspParamSchema::knob("sub_octave_division_ratio", "Frequency Division Sub Ratio (1:2, 1:3)", 1.0, 4.0, 2.0, "div", MacroRole::Punch, "Integer divider creating fundamental sub-octaves (-1 octave, -2 octaves)"))
        .with_param(DspParamSchema::knob("envelope_tracking_attack_ms", "Subharmonic Attack Tracking Speed", 0.1, 50.0, 3.5, "ms", MacroRole::Tone, "Transient tracking speed following incoming musical attack envelopes precisely"))
        .with_param(DspParamSchema::knob("envelope_tracking_release_ms", "Subharmonic Release Tail Time", 10.0, 500.0, 85.0, "ms", MacroRole::Space, "Decay tracking preventing sudden bass dropouts on sustained musical passages"))
        .with_param(DspParamSchema::knob("sub_saturation_harmonics_db", "Sub Overtone Saturation Drive", 0.0, 30.0, 9.0, "dB", MacroRole::Tone, "Generates upper odd harmonics from pure sub sine to cut through smaller speakers"))
        .with_param(DspParamSchema::knob("highpass_phase_align_deg", "Sub-Bass Phase Alignment Offset", -180.0, 180.0, 0.0, "deg", MacroRole::Character, "Continuous phase rotation aligning generated sub wave with original dry low end"))
        .with_param(DspParamSchema::toggle("enable_octave_fundamental_blend", "Clean Fundamental Reinforcement Blend", true, "Preserves and phase-aligns existing fundamental while summing subharmonic tone"))
    );
    descriptors.insert("AsynchronousGranularWavetableOscillator".to_string(), DspNodeDescriptor::new("AsynchronousGranularWavetableOscillator", "Granular Morphing Wavetable Synthesizer", DspNodeCategory::Oscillator, "Advanced hybrid oscillator combining 2048-sample single-cycle wavetable morphing with an asynchronous granular cloud engine for lush, evolving textures")
        .with_param(DspParamSchema::knob("wavetable_position_scan_speed", "Wavetable Frame Position Index", 0.0, 64.0, 16.5, "frame", MacroRole::Tone, "Interpolates through wavetable bank frames with band-limited polyBLEP filtering"))
        .with_param(DspParamSchema::knob("grain_window_overlap_factor", "Granular Cloud Overlap Density", 1.0, 16.0, 4.0, "grains", MacroRole::Space, "Number of simultaneous overlapping wavetable micro-grains active at once"))
        .with_param(DspParamSchema::knob("spectral_wave_morph_index", "Nonlinear Spectral Table Morphing", 0.0, 1.0, 0.45, "morph", MacroRole::Character, "Spectral interpolation morphing wave harmonic peaks towards new timbres"))
        .with_param(DspParamSchema::knob("grain_pitch_micro_detune_cents", "Inter-Grain Detune Jitter Spread", 0.0, 100.0, 18.0, "cents", MacroRole::Punch, "Randomized pitch deviation applied across grains for huge unison width"))
        .with_param(DspParamSchema::knob("formant_center_frequency_hz", "Formant Filter Center Frequency", 100.0, 8000.0, 1200.0, "Hz", MacroRole::Tone, "Fixed spectral formant resonance that does not track MIDI note pitch"))
        .with_param(DspParamSchema::toggle("enable_window_sync_to_table_cycle", "Pitch-Synchronous Grain Windowing", true, "Locks grain envelopes to wavetable cycle boundaries eliminating phase clicks"))
    );
    descriptors.insert("AdditiveHyperbolicPartialSynthesizer".to_string(), DspNodeDescriptor::new("AdditiveHyperbolicPartialSynthesizer", "Hyperbolic Phase Additive Partial Synthesizer", DspNodeCategory::Oscillator, "64-partial additive synthesis engine featuring hyperbolic phase modulation per partial, continuous spectral centroid tilt, and independent odd/even harmonic morphing")
        .with_param(DspParamSchema::knob("spectral_centroid_tilt_slope", "Spectral Centroid Slope Tilt", -24.0, 12.0, -6.0, "dB/oct", MacroRole::Tone, "Logarithmic high-frequency rolloff across higher partials"))
        .with_param(DspParamSchema::knob("hyperbolic_phase_warp_factor", "Hyperbolic Phase Warp Modulation", 0.0, 2.0, 0.45, "warp", MacroRole::Character, "Nonlinear phase curvature warping sinusoids towards asymmetric punch"))
        .with_param(DspParamSchema::knob("odd_even_harmonic_balance", "Odd vs Even Harmonic Energy Balance", -1.0, 1.0, 0.15, "bal", MacroRole::Punch, "Crossfades between square-like odd harmonics and saw-like full spectra"))
        .with_param(DspParamSchema::knob("partial_inharmonicity_stiffness", "String Inharmonicity Dispersion", 0.0, 0.1, 0.003, "B", MacroRole::Space, "Shifts upper partials sharp according to acoustic stiff string equations"))
        .with_param(DspParamSchema::knob("dynamic_envelope_decay_damping", "High Partial Damping Rate", 0.1, 10.0, 2.2, "damp", MacroRole::Tone, "Accelerates decay rate of higher partials for natural plucked acoustic realism"))
        .with_param(DspParamSchema::toggle("enable_polyblep_anti_aliasing", "Spectral Boundary Anti-Aliasing Clamp", true, "Suppresses partials exceeding Nyquist frequency preventing digital foldback"))
    );
    descriptors.insert("PhaseDistortionResonantCasioOscillator".to_string(), DspNodeDescriptor::new("PhaseDistortionResonantCasioOscillator", "Phase Distortion Resonant Waveform Synthesizer", DspNodeCategory::Oscillator, "Classic 1980s Phase Distortion (PD) synthesis recreating Casio CZ-style variable reading pointer transfer functions, simulated resonant peak sweeps, and dual-window sync")
        .with_param(DspParamSchema::knob("phase_distortion_amount_index", "Phase Distortion Modulation Index", 0.0, 1.0, 0.65, "mod", MacroRole::Tone, "Nonlinear distortion applied to sine reading pointer creating bright waveforms"))
        .with_param(DspParamSchema::knob("resonant_waveform_shape_type", "PD Resonant Waveform Algorithm", 0.0, 7.0, 1.0, "type", MacroRole::Character, "Selects PD curve (Resonant Saw, Resonant Triangle, Trapezoid, Impulse)"))
        .with_param(DspParamSchema::knob("simulated_resonance_frequency_hz", "Simulated Filter Peak Resonance", 50.0, 12000.0, 1600.0, "Hz", MacroRole::Punch, "Frequency of internal ringing sine burst during phase window"))
        .with_param(DspParamSchema::knob("amplitude_window_decay_rate", "Envelope Window Decay Damping", 0.1, 10.0, 2.5, "damp", MacroRole::Space, "Steepness of amplitude window tapering off resonant burst cycles"))
        .with_param(DspParamSchema::knob("sub_oscillator_detune_cents", "Second Line Phase Detuning Offset", 0.0, 50.0, 8.0, "cents", MacroRole::Tone, "Detune spread between primary and secondary phase distortion lines"))
        .with_param(DspParamSchema::toggle("enable_octave_ring_multiplication", "Ring Modulation Between PD Lines", false, "Multiplies both phase distortion lines for metallic bell-like timbres"))
    );
    descriptors.insert("FractalIteratedFunctionNoiseSynthesizer".to_string(), DspNodeDescriptor::new("FractalIteratedFunctionNoiseSynthesizer", "Iterated Function System (IFS) Fractal Noise Synthesizer", DspNodeCategory::Oscillator, "Self-similar stochastic oscillator based on affine transformation IFS attractors, producing evolving organic textures, brownian bursts, and deterministic chaos")
        .with_param(DspParamSchema::knob("fractal_dimension_hausdorff", "Hausdorff Self-Similarity Dimension", 1.0, 2.0, 1.45, "dim", MacroRole::Tone, "Fractal dimension controlling complexity and texture roughness"))
        .with_param(DspParamSchema::knob("affine_contraction_factor", "IFS Transformation Contraction Ratio", 0.1, 0.99, 0.68, "scale", MacroRole::Character, "Contraction mapping coefficient maintaining attractor stability"))
        .with_param(DspParamSchema::knob("attractor_rotation_angle_deg", "Phase Space Affine Rotation", 0.0, 360.0, 72.0, "deg", MacroRole::Space, "Phase space rotation angle between successive fractal iterations"))
        .with_param(DspParamSchema::knob("stochastic_perturbation_entropy", "Brownian Perturbation Jitter", 0.0, 1.0, 0.22, "chaos", MacroRole::Punch, "Random walk entropy added to orbit coordinates per sample"))
        .with_param(DspParamSchema::knob("bandpass_formant_tracking_hz", "Fractal Resonance Focus Center", 50.0, 12000.0, 680.0, "Hz", MacroRole::Tone, "Resonant filter center frequency tracking fractal density"))
        .with_param(DspParamSchema::toggle("enable_deterministic_seed_lock", "Deterministic PRNG Playback Seed", false, "Locks pseudo-random fractal sequence seed for deterministic DAW playback"))
    );
    descriptors.insert("SubharmonicBassSynthesizerSubGenerator".to_string(), DspNodeDescriptor::new("SubharmonicBassSynthesizerSubGenerator", "Subharmonic Sub-Octave Bass Synthesizer", DspNodeCategory::Oscillator, "Low-frequency energy reconstructor analyzing incoming fundamental pitches and generating clean, phase-aligned synthetic sub-octave sine waves with saturation")
        .with_param(DspParamSchema::knob("sub_octave_blend_gain_db", "Subharmonic Level Blend Gain", -48.0, 6.0, -3.0, "dB", MacroRole::Punch, "Gain level of synthesized subharmonic sine wave added to output"))
        .with_param(DspParamSchema::knob("sub_divider_octave_selector", "Sub-Octave Division Ratio (1 or 2 oct)", 1.0, 2.0, 1.0, "oct", MacroRole::Tone, "Divides fundamental pitch by 2 (-1 octave) or 4 (-2 octaves)"))
        .with_param(DspParamSchema::knob("sub_harmonic_drive_saturation", "Low-End Soft Saturation Warmth", 0.0, 1.0, 0.35, "drive", MacroRole::Character, "Even harmonic saturation enhancing sub-bass audibility on small monitors"))
        .with_param(DspParamSchema::knob("pitch_tracking_lowpass_corner_hz", "Fundamental Pitch Tracker Lowpass Cutoff", 40.0, 300.0, 110.0, "Hz", MacroRole::Tone, "Lowpass filter cutoff isolating fundamental from harmonics for clean tracking"))
        .with_param(DspParamSchema::knob("sub_decay_damping_time_ms", "Sub-Bass Sustain Decay Smoothing", 10.0, 1000.0, 180.0, "ms", MacroRole::Space, "Release envelope smoothing avoiding abrupt sub-bass drops on note off"))
        .with_param(DspParamSchema::toggle("enable_phase_alignment_lock", "Transient Phase Alignment Lock", true, "Dynamically phase-aligns subharmonic peaks to incoming kick drum transients"))
    );
    descriptors.insert("SuperformulaChaoticGeometryOscillator".to_string(), DspNodeDescriptor::new("SuperformulaChaoticGeometryOscillator", "Gielis Superformula 3D Geometric Oscillator", DspNodeCategory::Oscillator, "Advanced mathematical oscillator generating complex organic periodic and chaotic waveforms using Gielis' superformula geometry parameters with real-time morphing")
        .with_param(DspParamSchema::knob("rotational_symmetry_m_lobes", "Rotational Symmetry Order (M Lobes)", 1.0, 24.0, 5.0, "lobes", MacroRole::Character, "Number of rotational symmetry lobes shaping fundamental cycle waveform geometry"))
        .with_param(DspParamSchema::knob("superformula_exponent_n1", "Radial Curvature Shape Exponent (N1)", 0.1, 10.0, 1.2, "n1", MacroRole::Tone, "Primary exponent controlling inner sharp pinch vs ballooned round geometry curvature"))
        .with_param(DspParamSchema::knob("superformula_exponent_n2", "Asymmetry Harmonic Exponent (N2)", 0.1, 10.0, 2.5, "n2", MacroRole::Character, "Secondary exponent controlling cosine component weighting and odd-harmonic edge sharpness"))
        .with_param(DspParamSchema::knob("superformula_exponent_n3", "Quadrant Balance Exponent (N3)", 0.1, 10.0, 2.5, "n3", MacroRole::Tone, "Tertiary exponent controlling sine component weighting and even-harmonic wave asymmetry"))
        .with_param(DspParamSchema::knob("orbit_angular_velocity_rad_sec", "Phase Trajectory Orbital Rotation Rate", 0.0, 50.0, 3.14, "rad/s", MacroRole::Space, "Angular rotation velocity of the 2D parametric geometry coordinate frame over time"))
        .with_param(DspParamSchema::toggle("enable_bandlimited_supersampling", "PolyBLEP Antialiased Oversampling", true, "Applies polynomial bandlimited step smoothing to sharp geometric cusp transitions"))
    );
    descriptors.insert("AdditivePartialsResonatorBankOscillator".to_string(), DspNodeDescriptor::new("AdditivePartialsResonatorBankOscillator", "128-Partial Spectral Inharmonic Resonator Bank", DspNodeCategory::Oscillator, "High-density 128-partial sinusoidal additive synthesis voice with individual partial decay times, inharmonic frequency detuning, and spectral partial morphing")
        .with_param(DspParamSchema::knob("harmonic_inharmonicity_stiffness", "Inharmonic Dispersion Stiffness Factor", 0.0, 0.05, 0.0018, "stiff", MacroRole::Character, "Stretched frequency partial equation detuning higher overtones sharp like stiff piano strings"))
        .with_param(DspParamSchema::knob("spectral_tilt_slope_db_per_oct", "Spectral Partials Amplitude Roll-off Slope", -24.0, 0.0, -6.0, "dB/oct", MacroRole::Tone, "Decibel attenuation per octave applied across the 128 partials shaping brightness"))
        .with_param(DspParamSchema::knob("odd_to_even_partial_ratio", "Odd vs Even Partials Amplitude Ratio", 0.0, 1.0, 0.5, "ratio", MacroRole::Character, "Balance between square-like odd harmonics and saw-like even/odd combination"))
        .with_param(DspParamSchema::knob("partial_decay_time_damping_scale", "High-Frequency Partial Decay Time Damping", 0.1, 5.0, 1.0, "damp", MacroRole::Space, "Accelerates decay rate of high-order partials replicating physical instrument damping"))
        .with_param(DspParamSchema::knob("spectral_comb_notch_spread", "Spectral Formant Comb Notch Spacing", 0.0, 1.0, 0.35, "comb", MacroRole::Character, "Imposes periodic spectral notches across partial bank for bell and metallic chime timbres"))
        .with_param(DspParamSchema::toggle("enable_simd_avx2_vectorized_evaluation", "AVX2 SIMD Vectorized Batch Evaluation", true, "Evaluates all 128 partials concurrently in 8-lane SIMD vector registers with zero jitter"))
    );
    descriptors.insert("RelativisticDopplerChaoticWaveformOscillator".to_string(), DspNodeDescriptor::new("RelativisticDopplerChaoticWaveformOscillator", "Relativistic Spacetime Lorentz Waveform Oscillator", DspNodeCategory::Oscillator, "Chaotic continuous waveform generator utilizing relativistic Lorentz coordinate transformation equations, velocity-dependent time dilation, and strange attractor trajectories")
        .with_param(DspParamSchema::knob("relativistic_velocity_beta_c", "Velocity Ratio Relative to Speed of Light (v/c)", 0.0, 0.99, 0.45, "beta", MacroRole::Character, "Relativistic orbital speed causing nonlinear time dilation pitch stretching and contraction"))
        .with_param(DspParamSchema::knob("lorentz_strange_attractor_prandtl", "Chaotic Prandtl Number Convection Parameter", 1.0, 30.0, 10.0, "sigma", MacroRole::Tone, "Prandtl parameter of underlying Lorentz attractor governing rate of trajectory chaos"))
        .with_param(DspParamSchema::knob("chaotic_rayleigh_instability_param", "Rayleigh Instability Strange Attractor Driving", 10.0, 60.0, 28.0, "rho", MacroRole::Character, "Rayleigh number driving the oscillator across dual strange attractor lobe transitions"))
        .with_param(DspParamSchema::knob("phase_space_orbital_radius", "Chaotic Phase Space Orbital Radius", 0.1, 5.0, 1.2, "rad", MacroRole::Punch, "Geometric scaling of 3D attractor phase trajectory mapped to audio output amplitude"))
        .with_param(DspParamSchema::knob("polyblep_subsample_smoothing", "Anti-Aliased PolyBLEP Waveform Smoothing", 0.0, 1.0, 0.85, "smooth", MacroRole::Tone, "Polynomial bandlimited step correction preventing Nyquist aliasing on sharp trajectories"))
        .with_param(DspParamSchema::toggle("enable_relativistic_headlight_beaming", "Relativistic Beam Forward Intensity Modulation", true, "Applies relativistic Doppler boosting concentrating waveform energy along forward velocity vector"))
    );
    descriptors.insert("FractionalPuckettGranularCloudOscillator".to_string(), DspNodeDescriptor::new("FractionalPuckettGranularCloudOscillator", "Fractional Brownian Granular Particle Cloud Synthesizer", DspNodeCategory::Oscillator, "Stochastic micro-acoustic synthesis engine based on fractional Brownian motion, Puckett granular time-frequency cells, and power-law fractal cloud dispersion")
        .with_param(DspParamSchema::knob("fractional_hurst_exponent_h", "Fractional Brownian Motion Hurst Exponent (H)", 0.05, 0.95, 0.62, "H", MacroRole::Character, "Hurst parameter controlling persistence (smooth pink 0.9) vs antipersistence (rough blue 0.1)"))
        .with_param(DspParamSchema::knob("grain_emission_density_per_sec", "Granular Particle Emission Rate", 5.0, 500.0, 85.0, "grains/s", MacroRole::Space, "Density of discrete micro-acoustic grains spawned per second into the soundfield"))
        .with_param(DspParamSchema::knob("grain_window_gaussian_sigma_ms", "Gaussian Grain Window Half-Width", 2.0, 150.0, 25.0, "ms", MacroRole::Tone, "Gaussian envelope duration parameter shaping grain spectral bandwidth and smoothness"))
        .with_param(DspParamSchema::knob("spectral_fractal_tilt_db_per_oct", "Fractal 1/f^alpha Spectral Slope", -18.0, 0.0, -4.5, "dB/oct", MacroRole::Tone, "Power-law spectral tilt across granular frequencies shaping dark to shimmering bright"))
        .with_param(DspParamSchema::knob("stochastic_pitch_dispersion_cents", "Granular Cloud Random Pitch Scatter", 0.0, 600.0, 45.0, "cents", MacroRole::Space, "Gaussian pitch randomization applied to each newly spawned micro-grain voice"))
        .with_param(DspParamSchema::toggle("enable_quantum_shot_noise_burst", "Micro-Acoustic Quantum Shot Noise Bursts", true, "Injects sub-Poissonian micro-acoustic shot noise bursts at grain onset boundaries"))
    );
    descriptors.insert("FractionalChaosDuffingOscillator".to_string(), DspNodeDescriptor::new("FractionalChaosDuffingOscillator", "Fractional Duffing Double-Well Chaotic Waveform Generator", DspNodeCategory::Oscillator, "Nonlinear physical oscillator modeling a driven Duffing twin-well potential system with fractional derivative viscoelastic damping, strange attractor chaos, and period-doubling bifurcation")
        .with_param(DspParamSchema::knob("cubic_nonlinearity_stiffness_beta", "Duffing Cubic Potential Well Nonlinearity (Beta)", 0.1, 10.0, 2.4, "beta", MacroRole::Character, "Stiffness of the double-well restoring force shaping waveform harmonic spectrum"))
        .with_param(DspParamSchema::knob("periodic_driving_force_gamma", "Sinusoidal External Driving Force Amplitude (Gamma)", 0.0, 5.0, 1.65, "amp", MacroRole::Punch, "Driving force level; past threshold the system transitions into deterministic chaotic motion"))
        .with_param(DspParamSchema::knob("fractional_damping_order_alpha", "Fractional Riemann-Liouville Damping Order (Alpha)", 0.1, 1.9, 0.85, "order", MacroRole::Tone, "Fractional derivative damping order introducing memory effects and power-law decay"))
        .with_param(DspParamSchema::knob("driving_frequency_omega_hz", "Periodic Driving Frequency (Omega)", 10.0, 2000.0, 128.0, "Hz", MacroRole::Tone, "Base fundamental frequency driving the Duffing strange attractor"))
        .with_param(DspParamSchema::knob("phase_plane_orbit_cross_feedback", "Attractor Phase-Plane Position Velocity Cross-Mod", 0.0, 1.0, 0.35, "fdbk", MacroRole::Space, "Feedback between position x(t) and velocity dx/dt modulating stereo panning width"))
        .with_param(DspParamSchema::toggle("enable_polyblep_bifurcation_antialias", "PolyBLEP Sub-Sample Chaotic Attractor Anti-Aliasing", true, "Suppresses ultrasonic digital aliasing generated during abrupt potential well jumping"))
    );
    descriptors.insert("SuperformulaStochasticMorphOscillator".to_string(), DspNodeDescriptor::new("SuperformulaStochasticMorphOscillator", "Gielis 3D Superformula Stochastic Morphing Oscillator", DspNodeCategory::Oscillator, "Morphological synthesizer generating geometric waveforms through Johan Gielis's superformula equations with rotational symmetry, exponent pinching, and stochastic particle cloud modulation")
        .with_param(DspParamSchema::knob("superformula_symmetry_rotations_m", "Rotational Radial Symmetry Index (m)", 1.0, 24.0, 6.0, "m", MacroRole::Tone, "Number of rotational symmetry lobes defining primary harmonic overtone ratio"))
        .with_param(DspParamSchema::knob("radial_pinching_exponent_n1", "Superformula Radial Pinching Exponent (n1)", 0.1, 20.0, 2.2, "n1", MacroRole::Punch, "Overall waveform convexity; lower values produce sharp spike waves, higher values produce squares"))
        .with_param(DspParamSchema::knob("lobed_contour_asymmetry_n2_n3", "Quadrant Exponent Curvature Ratio (n2/n3)", 0.2, 5.0, 1.0, "ratio", MacroRole::Character, "Asymmetry between horizontal and vertical lobes introducing even harmonics"))
        .with_param(DspParamSchema::knob("stochastic_morph_orbit_speed_hz", "Chaotic Morphing Orbital Trajectory Speed", 0.02, 20.0, 0.45, "Hz", MacroRole::Space, "LFO sweep speed moving through parameter manifolds in 3D superformula space"))
        .with_param(DspParamSchema::knob("wavetable_subsample_oversample_ratio", "Sub-Sample Sinc Kernel Oversampling Depth", 1.0, 8.0, 4.0, "x", MacroRole::Tone, "Oversampling multiplier ensuring pristine aliasing-free performance at high pitches"))
        .with_param(DspParamSchema::toggle("enable_golden_ratio_phi_modulation", "Fibonacci Golden Ratio Phi Harmonics Lock", true, "Locks symmetry exponents to Fibonacci proportions for psychoacoustically pleasing timbre"))
    );
    descriptors.insert("CoupledChaoticVanDerPolNonlinearOscillator".to_string(), DspNodeDescriptor::new("CoupledChaoticVanDerPolNonlinearOscillator", "Coupled Van der Pol Limit-Cycle Chaotic Oscillator", DspNodeCategory::Oscillator, "Autonomous chaotic synthesizer built from dual cross-coupled Van der Pol nonlinear oscillators with limit-cycle entrainment, relaxation oscillations, and deterministic phase bifurcation")
        .with_param(DspParamSchema::knob("base_oscillation_frequency_hz", "Master Oscillator Fundamental Limit-Cycle Frequency", 10.0, 2000.0, 110.0, "Hz", MacroRole::Tone, "Natural resonant frequency of primary Van der Pol limit cycle"))
        .with_param(DspParamSchema::knob("nonlinear_damping_epsilon", "Van der Pol Nonlinear Damping Factor (Epsilon)", 0.01, 10.0, 1.8, "eps", MacroRole::Character, "Controls waveform shape: 0.1 is pure sinusoidal, >2 produces harsh relaxation jumps"))
        .with_param(DspParamSchema::knob("cross_coupling_conductance_k", "Inter-Oscillator Mutual Coupling Conductance", 0.0, 5.0, 0.85, "coup", MacroRole::Tone, "Strength of cross-coupling; past threshold oscillators lock into chaotic beating"))
        .with_param(DspParamSchema::knob("frequency_ratio_detuning", "Secondary Oscillator Detune Frequency Ratio", 0.5, 3.0, 1.333, "ratio", MacroRole::Tone, "Frequency detuning between the two coupled oscillators"))
        .with_param(DspParamSchema::knob("phase_space_limit_cycle_tilt", "Phase-Space Orbit Asymmetry Skew", -1.0, 1.0, 0.0, "skew", MacroRole::Punch, "Asymmetrical voltage bias tilting limit cycle and producing even harmonics"))
        .with_param(DspParamSchema::toggle("enable_bifurcation_sync_reset", "Hard Phase Sync Reset on Transient Impulse", false, "Resets chaotic phase orbit to origin on incoming MIDI note or gate trigger"))
    );
    descriptors.insert("GranularStochasticPoissonTextureSynthesizer".to_string(), DspNodeDescriptor::new("GranularStochasticPoissonTextureSynthesizer", "Poisson Stochastic Asynchronous Granular Synthesizer", DspNodeCategory::Oscillator, "Asynchronous granular synthesizer scheduling audio micro-grains via Poisson stochastic point processes with window shape morphing, stochastic pitch scatter, and spatial cloud dispersion")
        .with_param(DspParamSchema::knob("grain_density_poisson_rate_hz", "Poisson Mean Grain Emission Rate", 1.0, 1000.0, 120.0, "Hz", MacroRole::Tone, "Average number of grains emitted per second following Poisson distribution"))
        .with_param(DspParamSchema::knob("grain_duration_length_ms", "Individual Audio Grain Duration Length", 2.0, 250.0, 35.0, "ms", MacroRole::Space, "Length of each micro-grain envelope"))
        .with_param(DspParamSchema::knob("stochastic_pitch_scatter_semitones", "Grain Pitch Random Distribution Jitter", 0.0, 24.0, 3.5, "st", MacroRole::Character, "Pitch randomization spread across emitted grains"))
        .with_param(DspParamSchema::knob("grain_envelope_window_morph", "Grain Envelope Window Shape Morphing", 0.0, 1.0, 0.5, "morph", MacroRole::Tone, "Morphs grain envelope: 0.0: Hanning, 0.5: Blackman-Harris, 1.0: Percussive Exponential"))
        .with_param(DspParamSchema::knob("stereo_spatial_cloud_spread", "Panoramic Stereo Cloud Position Scattering", 0.0, 1.0, 0.85, "spread", MacroRole::Space, "Randomly pans emitted grains across stereo field creating wide ambient textures"))
        .with_param(DspParamSchema::toggle("enable_reverse_grain_probability", "Reverse Grain Playback Direction Probability", false, "Randomly plays a percentage of grains backwards for shimmering reverse effects"))
    );
    descriptors.insert("HyperchaoticLorentzAttractorSynthesizer".to_string(), DspNodeDescriptor::new("HyperchaoticLorentzAttractorSynthesizer", "4D Hyperchaotic Lorenz Strange Attractor Synthesizer", DspNodeCategory::Oscillator, "4-dimensional hyperchaotic system based on coupled Lorenz nonlinear differential equations generating complex non-repeating subharmonic drones and metallic percussion")
        .with_param(DspParamSchema::knob("lorentz_prandtl_sigma", "Lorenz Prandtl Fluid Parameter (Sigma)", 1.0, 30.0, 10.0, "sigma", MacroRole::Tone, "Ratio of momentum diffusivity to thermal diffusivity"))
        .with_param(DspParamSchema::knob("lorentz_rayleigh_rho", "Lorenz Rayleigh Convection Parameter (Rho)", 5.0, 60.0, 28.0, "rho", MacroRole::Character, "Temperature difference driving chaotic convective rolling"))
        .with_param(DspParamSchema::knob("hyperchaotic_dimension_w_gain", "4th Hyperchaotic Dimension (w) Feedback", 0.0, 5.0, 1.8, "w", MacroRole::Punch, "Coupling strength of 4th state variable producing multiple positive Lyapunov exponents"))
        .with_param(DspParamSchema::knob("phase_space_integration_speed_hz", "Phase Space Trajectory Speed", 20.0, 4000.0, 220.0, "Hz", MacroRole::Tone, "Time step scaling factor mapping orbit cycles to audible audio frequencies"))
        .with_param(DspParamSchema::knob("attractor_butterfly_wing_bias", "Dual-Lobe Butterfly Symmetry Bias", -1.0, 1.0, 0.0, "bias", MacroRole::Tone, "Biases chaotic trajectory toward left or right attractor wing"))
        .with_param(DspParamSchema::toggle("enable_runaway_boundary_limiting", "Phase Space Elastic Boundary Clamping", true, "Clamps state variables within stable orbits preventing divergence to infinity"))
    );
    descriptors.insert("AdditiveMorphingInharmonicBellGenerator".to_string(), DspNodeDescriptor::new("AdditiveMorphingInharmonicBellGenerator", "64-Partial Additive Inharmonic Bell Synthesizer", DspNodeCategory::Oscillator, "64-partial additive synthesis engine with user-definable inharmonic spectral partial distributions, dynamic mode decay rates, and striking mallet envelope contours")
        .with_param(DspParamSchema::knob("fundamental_strike_pitch_hz", "Bell Fundamental Strike Pitch (Hum Tone)", 30.0, 1200.0, 261.6, "Hz", MacroRole::Tone, "Fundamental resonance frequency of cast bell"))
        .with_param(DspParamSchema::knob("inharmonic_dispersion_morph", "Inharmonic Partial Distribution (Chime to Gong)", 0.5, 3.5, 1.85, "disp", MacroRole::Character, "Partial frequency spacing: 1.0 harmonic, 1.8-2.4 church bell / carillon, 3.0+ gong"))
        .with_param(DspParamSchema::knob("high_frequency_damping_rate", "High-Order Partial Decay Damping Rate", 0.1, 10.0, 2.5, "damp", MacroRole::Space, "Controls how much faster upper metallic partials decay compared to fundamental"))
        .with_param(DspParamSchema::knob("strike_transient_brightness", "Mallet Impact Noise & Upper Partial Burst", 0.0, 1.0, 0.65, "bright", MacroRole::Punch, "Excitation burst brightness at moment of clapper strike"))
        .with_param(DspParamSchema::knob("hum_tone_sustain_seconds", "Fundamental Hum Tone Free Decay Time", 0.5, 30.0, 8.0, "s", MacroRole::Space, "Long-decay sustain time of lowest resonant cavity hum tone"))
        .with_param(DspParamSchema::toggle("enable_clapper_double_strike", "Clapper Double-Strike Re-excitation", false, "Simulates clapper bouncing twice against bell rim on heavy strokes"))
    );
    descriptors.insert("NonlinearCoupledVanDerPolChaosSynthesizer".to_string(), DspNodeDescriptor::new("NonlinearCoupledVanDerPolChaosSynthesizer", "Nonlinear Coupled Van der Pol Chaotic Oscillator", DspNodeCategory::Oscillator, "Dual coupled non-linear Van der Pol limit-cycle relaxation oscillators generating rich self-oscillating timbres, chaotic frequency entrainment, and bifurcation subharmonics")
        .with_param(DspParamSchema::knob("oscillator1_natural_frequency_hz", "Master Oscillator 1 Natural Frequency", 20.0, 2000.0, 110.0, "Hz", MacroRole::Tone, "Natural resonance frequency of master Van der Pol oscillator"))
        .with_param(DspParamSchema::knob("oscillator2_natural_frequency_hz", "Slave Oscillator 2 Natural Frequency", 20.0, 2000.0, 165.0, "Hz", MacroRole::Tone, "Natural resonance frequency of slave Van der Pol oscillator"))
        .with_param(DspParamSchema::knob("nonlinearity_parameter_mu", "Nonlinear Damping Coefficient (Mu)", 0.1, 10.0, 2.5, "mu", MacroRole::Character, "Nonlinear damping coefficient: low = harmonic sine, high = jagged relaxation wave"))
        .with_param(DspParamSchema::knob("mutual_coupling_strength_k", "Mutual Oscillator Cross-Coupling (k)", 0.0, 5.0, 1.2, "k", MacroRole::Punch, "Cross-coupling parameter driving the dual oscillators into chaotic lock or beating"))
        .with_param(DspParamSchema::knob("phase_state_variable_mix", "Position (x) vs Velocity (dx/dt) Mix", 0.0, 1.0, 0.5, "mix", MacroRole::Tone, "Blends position state x and velocity state dx/dt for distinct harmonic spectra"))
        .with_param(DspParamSchema::toggle("enable_subharmonic_period_doubling", "Bifurcation Subharmonic Period Doubling", true, "Allows bifurcation transitions into subharmonic chaos and octave drops"))
    );
    descriptors.insert("AdditiveSpectralInharmonicPartialsSynthesizer".to_string(), DspNodeDescriptor::new("AdditiveSpectralInharmonicPartialsSynthesizer", "128-Partial Additive Inharmonic Synthesizer", DspNodeCategory::Oscillator, "High-precision 128-partial dynamic additive synthesizer with non-integer frequency overtone stretching, individual exponential damping decay laws, and partial phase randomized initialization")
        .with_param(DspParamSchema::knob("partial_count", "Active Additive Sinusoid Count", 8.0, 128.0, 64.0, "parts", MacroRole::Tone, "Number of active additive sinusoids evaluated per audio block"))
        .with_param(DspParamSchema::knob("inharmonic_stretch_power", "Inharmonic Overtone Frequency Stretch Power", 0.5, 3.0, 1.0, "exp", MacroRole::Character, "Frequency formula exponent fn = f0 * n^p (1.0 = pure harmonic, >1 = metallic/bell)"))
        .with_param(DspParamSchema::knob("spectral_tilt_slope_db_oct", "Harmonic Roll-Off Slope Across Spectrum", -18.0, 0.0, -6.0, "dB/oct", MacroRole::Tone, "Harmonic roll-off slope across partial frequency spectrum"))
        .with_param(DspParamSchema::knob("high_partial_decay_acceleration", "High-Partial Decay Damping Acceleration", 0.5, 5.0, 2.2, "accel", MacroRole::Punch, "Damps upper partials faster than lower partials replicating acoustic physics"))
        .with_param(DspParamSchema::knob("initial_phase_randomization_deg", "Initial Sinusoid Phase Randomization", 0.0, 360.0, 180.0, "deg", MacroRole::Space, "Randomizes initial sinusoid phases preventing harsh coherent peak spikes"))
        .with_param(DspParamSchema::toggle("enable_odd_even_partial_comb_balance", "Even Partial Attenuation (Hollow Square)", false, "Attenuates even partials to generate hollow square/clarinet-like timbre"))
    );
    descriptors.insert("NonlinearDoublePendulumChaosSynthesizer".to_string(), DspNodeDescriptor::new("NonlinearDoublePendulumChaosSynthesizer", "Nonlinear Double Pendulum Chaos Synthesizer", DspNodeCategory::Oscillator, "Numerical simulation of a coupled chaotic double pendulum generating continuous quasi-periodic and turbulent audio waveforms")
        .with_param(DspParamSchema::knob("pendulum_mass_ratio", "Pendulum Arm Mass Ratio", 0.1, 10.0, 1.5, "ratio", MacroRole::Character, "Ratio between upper and lower bob masses dictating trajectory bifurcation"))
        .with_param(DspParamSchema::knob("rod_length_ratio", "Rod Arm Length Ratio", 0.1, 5.0, 1.0, "ratio", MacroRole::Tone, "Ratio between rod lengths determining modal fundamental coupling"))
        .with_param(DspParamSchema::knob("gravity_energy", "Gravitational Acceleration", 0.1, 50.0, 9.81, "m/s2", MacroRole::Character, "Gravitational potential driving the physical integration speed"))
        .with_param(DspParamSchema::knob("friction_damping", "Joint Bearing Friction Loss", 0.0, 0.5, 0.02, "fric", MacroRole::Character, "Viscous damping on both pendulum joints regulating chaotic decay"))
        .with_param(DspParamSchema::knob("chaos_orbit_mod", "Orbit Audio Projection Depth", 0.0, 1.0, 0.70, "depth", MacroRole::Character, "Chaotic angular coordinate mapping depth into audio output"))
        .with_param(DspParamSchema::knob("frequency_warp", "Base Oscillator Fundamental", 20.0, 4000.0, 220.0, "Hz", MacroRole::Tone, "Simulation integration clock rate mapped to audio fundamental"))
    );
    descriptors.insert("WavetableFormantGranularHybridOscillator".to_string(), DspNodeDescriptor::new("WavetableFormantGranularHybridOscillator", "Wavetable Formant Granular Hybrid Oscillator", DspNodeCategory::Oscillator, "Hybrid sound generator combining interpolated wavetable scanning with synchronized micro-granular formant grains")
        .with_param(DspParamSchema::knob("wavetable_pos", "Wavetable Frame Scanning Position", 0.0, 1.0, 0.25, "pos", MacroRole::Character, "Smooth frame interpolation across the multi-cycle wavetable spectrum"))
        .with_param(DspParamSchema::knob("grain_size", "Granular Formant Window Length", 5.0, 250.0, 40.0, "ms", MacroRole::Space, "Duration of synchronized micro-granular wave packets"))
        .with_param(DspParamSchema::knob("grain_density", "Grains Per Audio Cycle", 1.0, 64.0, 16.0, "grains", MacroRole::Character, "Density rate of emitted formant grains per pitch cycle"))
        .with_param(DspParamSchema::knob("formant_shift", "Formant Center Transposition", -24.0, 24.0, 0.0, "semi", MacroRole::Tone, "Shifts vocal formant spectral peaks independently of pitch"))
        .with_param(DspParamSchema::knob("spray_jitter", "Granular Stochastic Jitter", 0.0, 1.0, 0.15, "jitter", MacroRole::Character, "Randomization of grain onset times and micro-pitch deviation"))
        .with_param(DspParamSchema::knob("sub_osc_level", "Sub-Octave Sine Reinforcement", 0.0, 1.0, 0.30, "gain", MacroRole::Punch, "Phase-aligned sub-octave fundamental sine oscillator blend"))
    );
    descriptors.insert("NonlinearChuaCircuitAttractorSynthesizer".to_string(), DspNodeDescriptor::new("NonlinearChuaCircuitAttractorSynthesizer", "Nonlinear Chua Circuit Attractor Synthesizer", DspNodeCategory::Oscillator, "Physical simulation of Leon Chua's autonomous chaotic circuit featuring Chua's diode nonlinear piecewise-linear resistance, double-scroll strange attractor, and audio bifurcation")
        .with_param(DspParamSchema::knob("inductance_l", "Resonant Inductor Value (L)", 0.1, 10.0, 1.42, "H", MacroRole::Tone, "Inductance controlling base tank oscillation and chaotic transition frequency"))
        .with_param(DspParamSchema::knob("capacitance_c1", "State Variable Fast Capacitor (C1)", 1.0, 50.0, 10.0, "nF", MacroRole::Character, "Primary fast state variable governing voltage trajectory evolution"))
        .with_param(DspParamSchema::knob("capacitance_c2", "State Variable Slow Capacitor (C2)", 10.0, 500.0, 100.0, "nF", MacroRole::Tone, "Secondary state variable capacitor shaping low-frequency attractor orbital loop"))
        .with_param(DspParamSchema::knob("diode_slope_inner", "Chua Diode Inner Slope (Ga)", -2.0, 0.0, -0.75, "mS", MacroRole::Character, "Negative conductance slope within inner diode voltage boundaries"))
        .with_param(DspParamSchema::knob("diode_slope_outer", "Chua Diode Outer Slope (Gb)", -1.0, 1.0, -0.41, "mS", MacroRole::Character, "Conductance slope in outer piecewise-linear saturation regions"))
        .with_param(DspParamSchema::knob("base_clock_frequency", "Simulation Audio Clock Rate", 20.0, 5000.0, 261.6, "Hz", MacroRole::Tone, "Numerical integrator time-step scaling mapped to musical audio pitch"))
    );
    descriptors.insert("AdditiveSpectralFormantHarmonicOscillator".to_string(), DspNodeDescriptor::new("AdditiveSpectralFormantHarmonicOscillator", "Additive Spectral Formant Harmonic Oscillator", DspNodeCategory::Oscillator, "SIMD additive harmonic oscillator generating up to 64 sinusoidal partials aligned to dynamic vocal formant envelopes and spectral tilt curves")
        .with_param(DspParamSchema::knob("fundamental_freq", "Oscillator Fundamental Pitch", 20.0, 4000.0, 220.0, "Hz", MacroRole::Tone, "Base pitch frequency of the synthesized harmonic partial series"))
        .with_param(DspParamSchema::knob("partial_count", "Simultaneous Active Partials", 8.0, 64.0, 32.0, "partials", MacroRole::Character, "Number of additive sinusoidal partials computed in parallel"))
        .with_param(DspParamSchema::knob("formant_vowel_focus", "Vowel Formant Profile Index", 0.0, 4.0, 1.0, "vowel", MacroRole::Character, "Applies vocal formant filter envelope: A(0), E(1), I(2), O(3), U(4)"))
        .with_param(DspParamSchema::knob("spectral_tilt_db", "Spectral High Roll-Off Tilt", -18.0, 0.0, -6.0, "dB/oct", MacroRole::Tone, "High-frequency amplitude attenuation slope per octave"))
        .with_param(DspParamSchema::knob("odd_even_harmonic_ratio", "Odd vs Even Partials Ratio", 0.0, 1.0, 0.50, "ratio", MacroRole::Character, "Harmonic weighting: 0.0 = Pure square (odd), 1.0 = Pure sawtooth (all)"))
        .with_param(DspParamSchema::knob("inharmonic_spectral_stretch", "Partials Inharmonicity Stretch", 0.9, 1.5, 1.0, "warp", MacroRole::Character, "Stretches partial spacing above integer multiples for bell and metallic tones"))
    );
    descriptors.insert("NonlinearRabinovichFabrikantChaosSynthesizer".to_string(), DspNodeDescriptor::new("NonlinearRabinovichFabrikantChaosSynthesizer", "Rabinovich-Fabrikant Chaotic Attractor Synthesizer", DspNodeCategory::Oscillator, "Continuous dynamical oscillator integrating the Rabinovich-Fabrikant differential equations exhibiting multi-stable chaotic limit cycles and turbulent self-oscillating audio timbres")
        .with_param(DspParamSchema::knob("chaos_parameter_alpha", "Dissipation Parameter (Alpha)", 0.01, 0.5, 0.14, "alpha", MacroRole::Tone, "Nonlinear dissipation factor governing transition into turbulent chaos"))
        .with_param(DspParamSchema::knob("chaos_parameter_gamma", "Bifurcation Control (Gamma)", 0.05, 1.5, 0.28, "gamma", MacroRole::Character, "System bifurcation parameter controlling limit cycle density and trajectory branching"))
        .with_param(DspParamSchema::knob("integration_time_step", "Runge-Kutta Step Frequency", 20.0, 2000.0, 220.0, "Hz", MacroRole::Tone, "Fundamental orbital loop frequency scaling dynamical integration clock"))
        .with_param(DspParamSchema::knob("orbital_radius_projection", "Strange Attractor Orbit Scale", 0.1, 5.0, 1.2, "rad", MacroRole::Punch, "3D orbital trajectory excursion radius scaling output signal amplitude"))
        .with_param(DspParamSchema::knob("attractor_axis_blend", "X/Y/Z Coordinate Projection Mix", 0.0, 2.0, 0.5, "axis", MacroRole::Tone, "Blends between X-axis (harmonic), Y-axis (chaotic), and Z-axis (pulsed)"))
        .with_param(DspParamSchema::knob("subharmonic_fold_feedback", "Nonlinear Cross-Axis Feedback", 0.0, 1.0, 0.40, "fb", MacroRole::Character, "Cross-axis polynomial feedback coupling creating subharmonic frequency divisions"))
    );
    descriptors.insert("AdditiveHarmonicSpectralMorphOscillator".to_string(), DspNodeDescriptor::new("AdditiveHarmonicSpectralMorphOscillator", "Additive Harmonic Spectral Morph Oscillator", DspNodeCategory::Oscillator, "64-partial additive bank with dynamic spectral tilt morphing, odd/even partial harmonic skewing, inharmonic spectral stretching, and individual envelope decimation")
        .with_param(DspParamSchema::knob("base_fundamental_frequency", "Oscillator Fundamental Tuning", 20.0, 2000.0, 130.81, "Hz", MacroRole::Tone, "Master fundamental frequency controlling all 64 additive sine generators"))
        .with_param(DspParamSchema::knob("spectral_tilt_rolloff", "Partial Harmonic Spectral Tilt", -24.0, 6.0, -6.0, "dB/oct", MacroRole::Tone, "Continuous spectral envelope slope from dark mellow tone to piercing bright"))
        .with_param(DspParamSchema::knob("odd_even_harmonic_balance", "Odd vs Even Partial Ratio", -1.0, 1.0, 0.0, "ratio", MacroRole::Character, "Shifts energy balance: -1.0 = Pure square odd harmonics, +1.0 = Saw/string even harmonics"))
        .with_param(DspParamSchema::knob("inharmonic_frequency_stretch", "Inharmonic Partials Frequency Stretch", 0.0, 1.0, 0.12, "stretch", MacroRole::Character, "Stretches partial frequencies f_k = k * f_0 * (1 + B * k^2)^0.5 for metallic/bell timbres"))
        .with_param(DspParamSchema::knob("formant_peak_center", "Resonant Spectral Formant Center", 200.0, 5000.0, 1200.0, "Hz", MacroRole::Tone, "Center frequency of an artificial resonant bump emphasized across the partial bank"))
        .with_param(DspParamSchema::knob("active_partials_count", "Simultaneous Partials Density", 8.0, 64.0, 32.0, "pts", MacroRole::Space, "Number of active harmonic partial oscillators rendered in real time"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "OscSine" => Box::new(
            GenericDspNodeUi::new("OscSine", "Pure Sine Wave Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
        ),
        "OscSaw" => Box::new(
            GenericDspNodeUi::new("OscSaw", "Band-limited Sawtooth Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("bandlimit", "Bandlimit Order", 1, 8, 4, "", (153, 102, 255)))
        ),
        "OscPulse" => Box::new(
            GenericDspNodeUi::new("OscPulse", "Pulse Width Modulation Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("pulse_width", "Pulse Width", 0.01, 0.99, 0.5, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("pwm_rate", "PWM LFO Rate", 0.1, 20.0, 1.0, "Hz", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
        ),
        "OscTriangle" => Box::new(
            GenericDspNodeUi::new("OscTriangle", "Anti-aliased Triangle Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("symmetry", "Symmetry Ramp", 0.01, 0.99, 0.5, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
        ),
        "OscWavetable" => Box::new(
            GenericDspNodeUi::new("OscWavetable", "SIMD Wavetable Morphing Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("table", "Table Bank", vec!["Basic Shapes".into(), "Harmonics".into(), "Formants".into(), "Metallic".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
        ),
        "SimdPolyWavetableOscillator" => Box::new(
            GenericDspNodeUi::new("SimdPolyWavetableOscillator", "AVX2/NEON Polyphonic Wavetable Voice Bank", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("unison_detune", "Unison Detune", 0.0, 50.0, 8.0, "cents", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("unison_voices", "Voice Count", 1, 16, 4, "voices", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("sub_osc_gain", "Sub Osc Level", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("stereo_spread", "Stereo Spread", 0.0, 1.0, 0.7, "%", (0, 184, 217)))
        ),
        "GranularSynthNode" => Box::new(
            GenericDspNodeUi::new("GranularSynthNode", "Cloud Granular Audio Streamer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("density", "Grain Density", 1.0, 100.0, 25.0, "grains/s", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("spray", "Position Spray", 0.0, 1.0, 0.15, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("pitch_ratio", "Pitch Ratio", 0.25, 4.0, 1.0, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("grain_size", "Grain Size", 0.01, 0.5, 0.06, "s", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.0, "%", (255, 61, 113)))
        ),
        "PlasmaArcSynthesizer" => Box::new(
            GenericDspNodeUi::new("PlasmaArcSynthesizer", "High-Voltage Plasma Arc Audio Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("arc_voltage", "Arc Voltage", 100.0, 5000.0, 1500.0, "V", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("plasma_density", "Plasma Density", 0.1, 1.0, 0.65, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("ionization", "Ionization Rate", 10.0, 500.0, 120.0, "kHz", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("hiss_level", "Sub-thermal Hiss", 0.0, 1.0, 0.1, "%", (180, 195, 215)))
        ),
        "NoiseGen" => Box::new(
            GenericDspNodeUi::new("NoiseGen", "Multi-Color Spectral Noise Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_enum("noise_type", "Noise Color", vec!["White".into(), "Pink (1/f)".into(), "Brownian".into(), "Blue".into()], 0, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("cutoff", "Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "PluckSynth" => Box::new(
            GenericDspNodeUi::new("PluckSynth", "Karplus-Strong Plucked String Synthesis Core", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("damping", "String Damping Cutoff", 100.0, 10000.0, 3500.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("tension", "Feedback Loop Tension", 0.5, 0.999, 0.98, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("decay", "Exciter Burst Decay", 0.001, 0.1, 0.015, "s", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("brightness", "Noise Exciter Brightness", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
        ),
        "FmOperatorPair" => Box::new(
            GenericDspNodeUi::new("FmOperatorPair", "2-Operator Phase Modulation FM Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("carrier_freq", "Carrier Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("mod_ratio", "Modulator Ratio", 0.25, 16.0, 2.0, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("fm_depth", "FM Modulation Depth", 0.0, 10.0, 2.0, "idx", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("mod_decay", "Modulator Env Decay", 10.0, 2000.0, 300.0, "ms", (0, 230, 118)))
        ),
        "AnalogDrumVoice" => Box::new(
            GenericDspNodeUi::new("AnalogDrumVoice", "808/909 Style Analog Synthesized Drum Voice", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("tune", "Fundamental Pitch", 30.0, 500.0, 60.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("decay", "Amp Envelope Decay", 10.0, 1500.0, 250.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("snap", "Click / Pitch Snap", 0.0, 1.0, 0.5, "%", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("punch", "Transient Punch", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
        ),
        "QuantumStateVectorOscillator" => Box::new(
            GenericDspNodeUi::new("QuantumStateVectorOscillator", "Qubit Phase Superposition & Hadamard Gate Synth", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("freq", "Superposition Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("phase_alpha", "Qubit Alpha Phase", 0.0, 360.0, 45.0, "deg", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("phase_beta", "Qubit Beta Phase", 0.0, 360.0, 45.0, "deg", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("hadamard_mix", "Hadamard Transformation", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("pauli_x", "Pauli-X State Flip", false, (255, 61, 113)))
        ),
        "QuantumHarmonicOscillatorVoice" => Box::new(
            GenericDspNodeUi::new("QuantumHarmonicOscillatorVoice", "Quantum Harmonic Oscillator Voice Bank", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_int("energy_level", "Quantum State Level (n)", 0, 10, 2, "n", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("wavefunction_sigma", "Wavefunction Spatial Width", 0.1, 5.0, 1.2, "sigma", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("potential_well", "Harmonic Well Depth", 0.1, 10.0, 4.0, "eV", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("output_gain", "Voice Output Gain", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
        ),
        "NavierStokesFluidNode" => Box::new(
            GenericDspNodeUi::new("NavierStokesFluidNode", "Navier-Stokes 3D Acoustic Fluid Dynamics", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sound_speed", "Fluid Sound Speed", 100.0, 2000.0, 343.0, "m/s", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("viscosity", "Kinematic Viscosity", 0.001, 1.0, 0.05, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("pressure_damping", "Poisson Pressure Damping", 0.9, 0.999, 0.995, "", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_int("grid_res", "3D Lattice Grid Size", 2, 8, 4, "cube", (0, 230, 118)))
        ),
        "MolecularVibrationResonator" => Box::new(
            GenericDspNodeUi::new("MolecularVibrationResonator", "Molecular Vibration Resonator Crystalline Lattice", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("fundamental_hz", "Lattice Base Frequency", 20.0, 10000.0, 520.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("resonance", "Modal Resonance Q", 0.1, 0.99, 0.85, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_enum("lattice_type", "Crystalline Lattice", vec!["Quartz (SiO2)".into(), "Diamond (Carbon)".into(), "Graphene 2D".into(), "Silicon Carbide".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("thermal_damping", "Thermal Damping Loss", 0.0, 1.0, 0.15, "%", (0, 230, 118)))
        ),
        "ClosedLoopNeuroFeedbackOscillator" => Box::new(
            GenericDspNodeUi::new("ClosedLoopNeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("target_alpha_hz", "Target Alpha Frequency", 7.0, 14.0, 10.5, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("feedback_gain", "Neuro-Feedback Loop Gain", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("relaxation_depth", "Entrainment Depth", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("binaural_carrier", "Binaural Carrier Center", 100.0, 500.0, 216.0, "Hz", (153, 102, 255)))
        ),
        "BrainwaveEntrainmentBinauralBeat" => Box::new(
            GenericDspNodeUi::new("BrainwaveEntrainmentBinauralBeat", "Brainwave Entrainment Binaural Beat Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("carrier_freq_hz", "Binaural Carrier Pitch", 50.0, 800.0, 250.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("beat_freq_hz", "Differential Beat Freq", 0.5, 30.0, 7.83, "Hz", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("entrainment_depth", "Modulation Amplitude", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_bool("isochronic_pulse", "Isochronic Pulse Mode", false, (153, 102, 255)))
        ),
        "SubharmonicSynthNode" => Box::new(
            GenericDspNodeUi::new("SubharmonicSynthNode", "Subharmonic Multi-Octave Sub Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub1_oct_gain", "Sub-1 Octave (-12st)", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("sub2_oct_gain", "Sub-2 Octave (-24st)", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("lowpass_cutoff", "Sub Lowpass Corner", 40.0, 400.0, 120.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("drive", "Asymmetric Drive", 1.0, 5.0, 1.5, "x", (255, 61, 113)))
        ),
        "NeuroFeedbackOscillator" => Box::new(
            GenericDspNodeUi::new("NeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_log("base_freq", "Base Carrier Frequency", 20.0, 2000.0, 432.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("cents_offset", "Neuro Tuning Shift", -100.0, 100.0, 0.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("alpha_target", "Alpha Coherence Target", 8.0, 14.0, 10.5, "Hz", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("feedback_depth", "Biofeedback Depth", 0.0, 1.0, 0.65, "%", (239, 68, 68)))
        ),
        "SpectrogramSoundGenerator" => Box::new(
            GenericDspNodeUi::new("SpectrogramSoundGenerator", "Visual Spectrogram Image-to-Audio Synthesizer HUD", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("playback_speed", "Spectrogram Image Playback Scan Speed", 0.1, 10.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("frequency_range_mode", "Audio Frequency Mapping Spectrum Range", vec!["Full Audible (20Hz-20kHz)".into(), "Musical (55Hz-7kHz)".into(), "Sub-Bass (20Hz-250Hz)".into(), "Vocal Formant (200Hz-4kHz)".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("dynamic_contrast", "Visual Pixel Intensity Dynamic Contrast", 0.5, 4.0, 1.5, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("harmonic_quantization", "Frequency Bin Scale Quantization", vec!["Continuous Microtonal".into(), "12-EDO Chromatic".into(), "19-EDO Extended".into(), "31-EDO Just".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_image_spread", "Stereo Spatial Panorama Distribution", 0.0, 100.0, 50.0, "%", (34, 197, 94)))
        ),
        "SimdPolyVoice" => Box::new(
            GenericDspNodeUi::new("SimdPolyVoice", "SIMD Parallel Vectorized Synthesizer Voice Cell", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("note", "MIDI Key Note Number", 0.0, 127.0, 60.0, "midi", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("frequency", "Fundamental Pitch Frequency", 10.0, 20000.0, 261.63, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("velocity", "Key Strike Dynamic Velocity", 0.0, 127.0, 100.0, "vel", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("env_level", "Instantaneous Envelope Amplitude", 0.0, 1.0, 0.8, "amp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("active", "SIMD Voice Allocation Gate", true, (34, 197, 94)))
        ),
        "NoiseType" => Box::new(
            GenericDspNodeUi::new("NoiseType", "Zero-Allocation Multi-Color Noise Generator Core", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("noise_spectral_color", "Pseudo-Random Noise Spectral Distribution", 0.0, 2.0, 1.0, "color", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("output_level_db", "Master Noise Generation Level", -60.0, 12.0, -12.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("highpass_rumble_filter_hz", "Sub-Bass Highpass Rumble Cutoff", 5.0, 500.0, 20.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_decorrelation_width", "Stereo Phase Decorrelation Spread", 0.0, 2.0, 1.0, "width", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("entropy_seed_randomness", "PRNG Initial Entropy Seed Offset", 1.0, 100.0, 42.0, "seed", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_anti_aliased_bandlimited_noise", "Band-Limited Anti-Aliased PRNG Evaluation", true, (14, 165, 233)))
        ),
        "SubHarmonicBassSynthesizer" => Box::new(
            GenericDspNodeUi::new("SubHarmonicBassSynthesizer", "Waveform Sub-Octave Generator & Psychoacoustic Fundamental Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_interval", "Sub-Harmonic Pitch Interval", 0.0, 3.0, 0.0, "int", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_blend_level_db", "Sub-Harmonic Volume Level", -36.0, 6.0, -6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("harmonic_saturation_drive", "Harmonic Saturation Drive", 0.0, 1.0, 0.25, "drive", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lowpass_cutoff_frequency_hz", "Sub-Bass Lowpass Filter Cutoff", 40.0, 180.0, 90.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("tracking_sensitivity_threshold", "Pitch Tracking Sensitivity", 0.05, 0.8, 0.2, "sens", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("dynamic_envelope_following", "Dynamic Envelope Following", true, (14, 165, 233)))
        ),
        "WavetablePhaseDistortionOsc" => Box::new(
            GenericDspNodeUi::new("WavetablePhaseDistortionOsc", "Hybrid Wavetable & Resonant Phase Distortion Synthesizer Engine", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wavetable_position_index", "Wavetable Scan Position Frame", 0.0, 64.0, 0.0, "pos", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("phase_distortion_intensity", "Phase Distortion Transfer Intensity", 0.0, 1.0, 0.5, "pd", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pd_resonant_window_form", "Resonant Window Envelope Shape", 0.0, 3.0, 0.0, "shape", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_oscillator_wave_mix", "Sub-Oscillator Blend Level", 0.0, 1.0, 0.25, "mix", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("hard_sync_frequency_ratio", "Slave Hard Sync Pitch Ratio", 1.0, 8.0, 1.0, "ratio", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("anti_aliasing_simd_oversample", "Polyphase Anti-Aliasing Oversampling", true, (14, 165, 233)))
        ),
        "FranklinGlassArmonica" => Box::new(
            GenericDspNodeUi::new("FranklinGlassArmonica", "37-Bowl Franklin Glass Armonica Physical Acoustic Resonator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("spindle_rotation_speed_rad_s", "Spindle Rotational Speed", 0.0, 20.0, 6.28, "rad/s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("finger_rubbing_pressure_n", "Finger Rubbing Contact Pressure", 0.01, 5.0, 0.85, "N", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("quartz_material_preset", "Quartz Formulation Profile", 0.0, 4.0, 0.0, "preset", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("water_mass_loading_ratio", "Hydro-Acoustic Water Mass Loading", 0.0, 1.0, 0.15, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("doublet_beating_shimmer_hz", "Thin-Shell Doublet Beating Shimmer", 0.0, 10.0, 1.8, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("mahogany_chassis_coupling", "Resonant Mahogany Chassis Coupling", 0.0, 1.0, 0.6, "ratio", (14, 165, 233)))
        ),
        "FrenchHurdyGurdy" => Box::new(
            GenericDspNodeUi::new("FrenchHurdyGurdy", "French Vielle à Roue Hurdy-Gurdy Physical Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wheel_crank_speed_rpm", "Rosined Wheel Cranking Speed", 20.0, 240.0, 110.0, "RPM", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("wheel_rosin_friction_grip", "Rosin Stick-Slip Friction Grip", 0.1, 1.0, 0.72, "grip", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("trompette_buzz_dog_level", "Trompette Buzzing Dog Level", 0.0, 1.0, 0.55, "gain", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("coup_de_poignet_accent", "Coup de Poignet Wrist Accent Impulses", 0.0, 2.0, 0.3, "burst", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("drone_bourdon_mix", "Drone Bourdon Strings Acoustic Mix", 0.0, 1.0, 0.45, "mix", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("sympathetic_tarab_resonance", "Sympathetic Strings Resonant Coupling", 0.0, 1.0, 0.35, "res", (14, 165, 233)))
        ),
        "BilinearAirReedJet" => Box::new(
            GenericDspNodeUi::new("BilinearAirReedJet", "Non-Linear Flue Air-Reed Jet Excitation Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("blowing_jet_velocity_m_s", "Flue Air-Jet Stream Velocity", 5.0, 80.0, 24.0, "m/s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("jet_to_labium_distance_mm", "Windway to Labium Edge Distance", 2.0, 20.0, 7.5, "mm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("labium_edge_splitting_offset", "Jet-to-Labium Transverse Alignment", -1.0, 1.0, 0.05, "offset", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("vortex_shedding_turbulence", "Labium Vortex Shedding Turbulence", 0.0, 1.0, 0.22, "turb", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("jet_drive_polynomial_order", "Nonlinear Jet Saturation Curve", 0.0, 2.0, 0.0, "order", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("feedback_bore_pressure_coupling", "Acoustic Bore Feedback Pressure", 0.1, 2.0, 1.0, "gain", (14, 165, 233)))
        ),
        "ModalIdiophoneResonator" => Box::new(
            GenericDspNodeUi::new("ModalIdiophoneResonator", "Undercut Tuned Idiophone Bar & Resonator Tube Bank", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("idiophone_bar_material", "Resonant Bar Material & Hardness", 0.0, 3.0, 0.0, "mat", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("undercut_tuning_profile", "Undercut Arch Modal Tuning Ratio", 0.0, 3.0, 1.0, "tuning", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("resonator_tube_coupling", "Tubular Air Resonator Coupling", 0.0, 1.0, 0.75, "gain", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tube_q_tuning_sharpness", "Resonator Tube Q Sharpness", 5.0, 60.0, 24.0, "Q", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("strike_mallet_hardness", "Mallet Yarn/Rubber Strike Hardness", 0.1, 1.0, 0.45, "hard", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("vibraphone_tremolo_fan_speed_hz", "Vibraphone Tremolo Butterfly Fan Speed", 0.0, 8.0, 0.0, "Hz", (14, 165, 233)))
        ),
        "TineResonatorPickup" => Box::new(
            GenericDspNodeUi::new("TineResonatorPickup", "Asymmetric Tine-Tonebar Electro-Acoustic Pickup Engine", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("tine_to_pickup_gap_mm", "Tine Tip to Pickup Air Gap", 0.5, 10.0, 2.2, "mm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pickup_vertical_alignment", "Pickup Neutral Axis Vertical Offset", -1.0, 1.0, 0.25, "align", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("tonebar_mass_ratio", "Tonebar to Tine Mass Ratio", 5.0, 50.0, 22.0, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("neoprene_hammer_tip_hardness", "Neoprene Rubber Hammer Tip Grade", 0.0, 3.0, 1.0, "grade", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pickup_coil_inductance_h", "Electromagnetic Coil Inductance", 0.5, 5.0, 1.8, "H", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("magnetic_reluctance_distortion", "Magnetic Reluctance Bark Overdrive", 0.0, 1.0, 0.4, "drive", (14, 165, 233)))
        ),
        "PercussionMembrane2D" => Box::new(
            GenericDspNodeUi::new("PercussionMembrane2D", "2D Circular Bessel Drumhead Membrane & Air-Loaded Cavity", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("membrane_skin_material", "Drumhead Membrane Skin Material", 0.0, 3.0, 0.0, "skin", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("membrane_tuning_tension_n_m", "Radial Tension Lug Tuning Force", 500.0, 8000.0, 2400.0, "N/m", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("kettle_air_cavity_depth_cm", "Enclosed Shell Air Cavity Depth", 5.0, 80.0, 32.0, "cm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("strike_radial_impact_point", "Drumstick Radial Impact Position", 0.0, 0.95, 0.65, "radius", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("rimshot_hoop_impact_mix", "Rimshot Hoop Strike Transient Mix", 0.0, 1.0, 0.0, "mix", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("bottom_resonant_head_coupling", "Resonant Bottom Head & Snare Coupling", 0.0, 1.0, 0.7, "ratio", (14, 165, 233)))
        ),
        "WaveguideBrassMouthpiece" => Box::new(
            GenericDspNodeUi::new("WaveguideBrassMouthpiece", "Lip-Reed Waveguide Brass Acoustic Mouthpiece Exciter", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("mouthpiece_cup_profile", "Brass Mouthpiece Cup Profile", 0.0, 3.0, 1.0, "cup", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("lip_reed_equilibrium_bias", "Lip Embouchure Equilibrium Opening", 0.05, 0.8, 0.28, "bias", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("player_mouth_blowing_pressure_kpa", "Mouth Blowing Air Pressure", 0.5, 15.0, 4.2, "kPa", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lip_mass_resonance_hz", "Lip Tissue Mechanical Resonance", 80.0, 1200.0, 349.23, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lip_reed_q_factor", "Lip Tissue Mechanical Q Factor", 2.0, 20.0, 6.5, "Q", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("bernoulli_flow_nonlinearity", "Bernoulli Hydrodynamic Closing Force", 0.1, 1.0, 0.75, "coeff", (14, 165, 233)))
        ),
        "GlottalPulseGenerator" => Box::new(
            GenericDspNodeUi::new("GlottalPulseGenerator", "Liljencrants-Fant (LF) Glottal Flow Acoustic Voice Source", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("open_quotient_oq", "Vocal Fold Open Quotient (OQ)", 0.3, 0.9, 0.58, "OQ", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("return_phase_abruptness_rg", "Closure Abruptness Return Phase", 0.005, 0.2, 0.04, "Ta", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("speed_quotient_opening_asymmetry", "Opening/Closing Speed Quotient", 1.0, 3.5, 2.2, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pitch_jitter_percent", "Fundamental Frequency Jitter", 0.0, 5.0, 0.45, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("amplitude_shimmer_db", "Cycle Amplitude Shimmer Perturbation", 0.0, 2.0, 0.15, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("glottal_aspiration_breath_mix", "Transglottal Aspiration Breath Mix", 0.0, 1.0, 0.08, "mix", (14, 165, 233)))
        ),
        "ElectroAcousticResonator" => Box::new(
            GenericDspNodeUi::new("ElectroAcousticResonator", "Electromagnetic Pickup Feedback & Transducer Plate Resonator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("drive_current_ma", "Transducer Excitation Drive Current", 1.0, 500.0, 85.0, "mA", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("plate_resonance_damping", "Sheet Metal Plate Viscous Loss", 0.01, 1.0, 0.22, "damp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("magnetic_airgap_mm", "Pickup Reluctance Magnetic Air Gap", 0.2, 10.0, 2.0, "mm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("feedback_pickup_mix", "Regenerative Magnetic Feedback Mix", 0.0, 1.5, 0.4, "mix", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("transducer_saturation_ratio", "Core Magnetic Hysteresis Saturation", 0.0, 1.0, 0.35, "sat", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("subharmonic_hysteresis_flux", "Subharmonic Magnetostriction Flux", 0.0, 1.0, 0.15, "flux", (14, 165, 233)))
        ),
        "MorphologicalWaveTerrainOsc" => Box::new(
            GenericDspNodeUi::new("MorphologicalWaveTerrainOsc", "Dual-Axis Non-Euclidean 3D Wave Terrain Surface Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("terrain_surface_topology", "Mathematical Terrain Surface Function", 0.0, 4.0, 0.0, "topo", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("orbit_frequency_x_hz", "Trajectory Orbit X Frequency", 10.0, 2000.0, 110.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("orbit_frequency_y_hz", "Trajectory Orbit Y Frequency", 10.0, 2000.0, 165.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("orbit_radius_eccentricity", "Orbital Elliptic Eccentricity", 0.1, 5.0, 1.2, "ecc", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("terrain_depth_elevation", "Terrain Surface Peak Elevation", 0.1, 3.0, 1.0, "gain", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("chaotic_trajectory_drift", "Chaotic Orbit Precession Drift", 0.0, 1.0, 0.25, "drift", (14, 165, 233)))
        ),
        "AcousticSoundboardResonator" => Box::new(
            GenericDspNodeUi::new("AcousticSoundboardResonator", "Orthotropic Spruce Soundboard Finite-Element Resonator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wood_grain_anisotropy_ratio", "Spruce Grain Acoustic Anisotropy", 1.5, 6.0, 3.8, "ratio", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("soundboard_surface_area_m2", "Soundboard Total Radiating Area", 0.2, 4.0, 1.6, "m^2", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("rib_stiffening_factor", "Perpendicular Rib Reinforcement", 0.1, 2.0, 0.85, "stiff", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("bridge_mechanical_impedance", "Bridge Mechanical Load Impedance", 10.0, 500.0, 120.0, "kg/s", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("fundamental_plate_mode_hz", "Lowest Plate Eigenmode Resonance", 30.0, 300.0, 78.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("radiation_efficiency_loss", "High-Frequency Radiation Damping", 0.01, 0.8, 0.18, "loss", (14, 165, 233)))
        ),
        "VectorWaveSynthesisEngine" => Box::new(
            GenericDspNodeUi::new("VectorWaveSynthesisEngine", "Four-Quadrant 2D Planar Vector Wavetable Synthesis Engine", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("vector_position_x", "Vector Joystick X Position", -1.0, 1.0, 0.0, "X", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("vector_position_y", "Vector Joystick Y Position", -1.0, 1.0, 0.0, "Y", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("quadrant_a_wavetable", "Quadrant A Wavetable (Top-Left)", 0.0, 3.0, 0.0, "table", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("quadrant_b_wavetable", "Quadrant B Wavetable (Top-Right)", 0.0, 3.0, 1.0, "table", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("quadrant_c_wavetable", "Quadrant C Wavetable (Bottom-Left)", 0.0, 3.0, 2.0, "table", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("vector_orbit_rate_hz", "Automated Vector Orbit LFO Rate", 0.0, 10.0, 0.5, "Hz", (14, 165, 233)))
        ),
        "QuantumSuperpositionOscillator" => Box::new(
            GenericDspNodeUi::new("QuantumSuperpositionOscillator", "Qubit Wavefunction Superposition & Entanglement Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("qubit_theta_angle_rad", "Bloch Sphere Superposition Polar Angle", 0.0, 3.14159, 1.57, "rad", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("qubit_phi_phase_rad", "Bloch Sphere Azimuthal Phase Angle", 0.0, 6.28318, 0.0, "rad", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("collapse_rate_hz", "Wavefunction Measurement Collapse Rate", 1.0, 2000.0, 440.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("entanglement_coupling", "Inter-Qubit Bell State Entanglement Depth", 0.0, 1.0, 0.65, "ent", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("decoherence_noise_db", "Quantum Environmental Decoherence Entropy", -60.0, 0.0, -36.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("hadamard_gate_probability", "Continuous Hadamard Transformation Spread", 0.0, 1.0, 0.5, "%", (14, 165, 233)))
        ),
        "NeuromorphicSpikingOscillator" => Box::new(
            GenericDspNodeUi::new("NeuromorphicSpikingOscillator", "Biomimetic Leaky Integrate-and-Fire Spiking Neural Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("membrane_threshold_mv", "Soma Action Potential Threshold", -60.0, -30.0, -50.0, "mV", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("leak_conductance_ns", "Membrane Leaky Conductance", 1.0, 50.0, 10.0, "nS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("synaptic_current_na", "Injected Synaptic Drive Current", 0.0, 10.0, 2.5, "nA", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("refractory_period_ms", "Post-Spike Absolute Refractory Time", 0.5, 10.0, 2.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spike_pulse_width_us", "Action Potential Spike Duration", 50.0, 1000.0, 250.0, "us", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("synaptic_noise_std_dev", "Membrane Voltage Thermal Noise Jitter", 0.0, 5.0, 0.8, "mV", (14, 165, 233)))
        ),
        "SubharmonicBassSynthesizerEngine" => Box::new(
            GenericDspNodeUi::new("SubharmonicBassSynthesizerEngine", "Phase-Locked Dual-Octave Subharmonic Bass Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_1_mix_db", "Subharmonic -1 Octave Synthesis Level", -36.0, 12.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_octave_2_mix_db", "Subharmonic -2 Octave Deep Bass Level", -36.0, 12.0, -6.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_tracking_crossover_hz", "Pitch Tracking Frequency Cutoff", 40.0, 250.0, 90.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_lock_offset_deg", "Subharmonic Phase Lock Alignment", -180.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("saturation_drive_sub", "Subharmonic Upper Harmonic Saturation", 0.0, 1.0, 0.35, "sat", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("direct_dry_sub_crossover_cut", "Highpass Filter Weak Dry Bass", true, (14, 165, 233)))
        ),
        "PolyphonicGranularCloudSynthesizer" => Box::new(
            GenericDspNodeUi::new("PolyphonicGranularCloudSynthesizer", "Polyphonic Asynchronous Granular Cloud Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("grain_density_grains_per_sec", "Asynchronous Grain Cloud Density", 1.0, 500.0, 65.0, "gr/s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_ms", "Individual Grain Window Duration", 5.0, 1000.0, 85.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stochastic_pitch_jitter_semitones", "Grain Pitch Randomization Variance", 0.0, 24.0, 0.0, "st", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("stereo_spatial_dispersion", "Stereo Panorama Cloud Width", 0.0, 1.0, 0.7, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("grain_envelope_shape", "Grain Windowing Envelope Mode", 0.0, 3.0, 0.0, "shape", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("freeze_buffer_playback", "Freeze Audio Buffer Grain Loop", false, (14, 165, 233)))
        ),
        "PlasmaArcSonificationSynthesizer" => Box::new(
            GenericDspNodeUi::new("PlasmaArcSonificationSynthesizer", "Singing High-Voltage Plasma Arc Acoustic Spark Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("arc_breakdown_voltage_kv", "Dielectric Spark Breakdown Voltage", 1.0, 50.0, 15.0, "kV", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("plasma_channel_current_ma", "Conduction Channel Arc Current", 1.0, 200.0, 35.0, "mA", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spark_gap_distance_mm", "Electrode Spark Gap Distance", 0.5, 30.0, 8.0, "mm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("ionization_thermal_inertia_us", "Thermal Plasma Relaxation Inertia", 1.0, 500.0, 45.0, "us", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("corona_discharge_crackle_ratio", "Corona Discharge Stochastic Hiss", 0.0, 1.0, 0.15, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_high_frequency_spark_singing", "High-Frequency Ultrasonic PWM Singing", true, (14, 165, 233)))
        ),
        "SupercriticalFluidPhaseNoise" => Box::new(
            GenericDspNodeUi::new("SupercriticalFluidPhaseNoise", "Supercritical CO2 Phase-Transition Cavitation Noise Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("critical_point_temperature_k", "Fluid Thermodynamic Temperature", 250.0, 450.0, 304.13, "K", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("critical_pressure_megapascal", "Fluid Thermodynamic Static Pressure", 5.0, 20.0, 7.38, "MPa", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("opalescence_density_fluctuation", "Critical Opalescence Density Fluctuation", 0.0, 1.0, 0.55, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("cavitation_bubble_collapse_rate", "Micro-Cavitation Bubble Collapse Rate", 10.0, 5000.0, 420.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_entropy_color_slope", "Noise Power Spectral Entropy Slope", -6.0, 6.0, -1.5, "dB/oct", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_critical_opalescence_resonance", "Critical Point Divergence Resonance", true, (14, 165, 233)))
        ),
        "GravitationalWaveBinaryChirp" => Box::new(
            GenericDspNodeUi::new("GravitationalWaveBinaryChirp", "Compact Binary Inspiral Quadrupole Gravitational Wave Chirp Synth", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("primary_component_mass_solar", "Primary Component Mass (Solar Masses)", 1.4, 100.0, 36.0, "M_sun", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("secondary_component_mass_solar", "Secondary Component Mass (Solar Masses)", 1.4, 100.0, 29.0, "M_sun", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("inspiral_starting_frequency_hz", "Inspiral Horizon Starting Frequency", 15.0, 200.0, 32.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("orbital_eccentricity_ratio", "Keplerian Orbital Eccentricity", 0.0, 0.8, 0.05, "e", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ringdown_quasinormal_damping_q", "Post-Merger Ringdown Q-Factor", 1.0, 50.0, 12.0, "Q", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_spin_precession_polarization", "Orbital Plane Spin Precession Modulation", true, (14, 165, 233)))
        ),
        "CasimirQuantumVacuumNoise" => Box::new(
            GenericDspNodeUi::new("CasimirQuantumVacuumNoise", "Quantum Electrodynamic Vacuum Fluctuation & Casimir Force Noise", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("plate_separation_distance_nm", "Conductive Plate Gap Separation", 5.0, 500.0, 45.0, "nm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("quantum_vacuum_field_intensity", "Zero-Point Vacuum Field Intensity", 0.01, 2.0, 0.85, "E_0", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("thermal_blackbody_correction_k", "Thermal Photon Correction Temperature", 0.0, 500.0, 77.0, "K", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("finite_conductivity_skin_depth", "Boundary Plate Metal Skin Depth", 0.1, 50.0, 12.0, "nm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamic_casimir_photon_production", "Dynamical Casimir Real Photon Generation", 0.0, 1.0, 0.18, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_sub_kelvin_ground_state", "Pure Zero-Kelvin Quantum Ground State", true, (14, 165, 233)))
        ),
        "ThermonuclearFusionResonator" => Box::new(
            GenericDspNodeUi::new("ThermonuclearFusionResonator", "Tokamak Magnetic Confinement Fusion Plasma Resonance Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("toroidal_magnetic_field_tesla", "Toroidal Confinement Magnetic Field", 1.0, 15.0, 5.3, "T", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("plasma_core_temperature_kev", "Core Plasma Kinetic Temperature", 1.0, 50.0, 15.0, "keV", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("ion_cyclotron_resonance_harmonic", "Ion Cyclotron Gyro-Frequency Harmonic", 1.0, 16.0, 3.0, "harm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("neoclassical_tearing_mode_drift", "Magnetic Island Tearing Mode Drift", 0.1, 20.0, 4.2, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("plasma_current_megaamperes", "Poloidal Ohmic Heating Plasma Current", 0.5, 20.0, 7.5, "MA", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_h_mode_edge_transport_barrier", "High-Confinement H-Mode Edge Barrier", true, (14, 165, 233)))
        ),
        "QuantumAnnealingOptimizerSynth" => Box::new(
            GenericDspNodeUi::new("QuantumAnnealingOptimizerSynth", "Quantum Annealing Energy Minimization Landscape Audio Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("annealing_schedule_time_ms", "Annealing Schedule Cooling Duration", 10.0, 5000.0, 250.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transverse_field_strength", "Transverse Magnetic Tunneling Field", 0.01, 10.0, 1.5, "T", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spin_glass_qubit_count", "Ising Spin Glass Qubit Dimensions", 4.0, 64.0, 16.0, "qubits", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("thermal_fluctuation_noise", "Thermal Bath Quantum Decoherence", 0.0, 1.0, 0.15, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ground_state_convergence_ratio", "Ground State Convergence Factor", 0.1, 1.0, 0.85, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_quantum_tunneling_transitions", "Enable Non-Local Quantum Tunneling", true, (14, 165, 233)))
        ),
        "RiemannSurfaceComplexOscillator" => Box::new(
            GenericDspNodeUi::new("RiemannSurfaceComplexOscillator", "Conformal Mapping Multi-Sheeted Riemann Surface Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("branch_point_order_n", "Riemann Branch Point Order (n)", 1.0, 8.0, 2.0, "sheets", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("conformal_pole_offset_real", "Complex Pole Real Axis Offset", -2.0, 2.0, 0.25, "Re", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("conformal_pole_offset_imag", "Complex Pole Imaginary Offset", -2.0, 2.0, 0.75, "Im", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spiral_winding_number", "Surface Winding Angular Number", 1.0, 12.0, 3.0, "turns", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("analytic_continuation_drift", "Analytic Continuation Drift Rate", 0.0, 1.0, 0.2, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_biholomorphic_mapping", "Biholomorphic Invariant Normalization", true, (14, 165, 233)))
        ),
        "BicubicInterpolatedWavetableMorph" => Box::new(
            GenericDspNodeUi::new("BicubicInterpolatedWavetableMorph", "2D Grid Bicubic Spline Continuous Wavetable Morpher", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wavetable_grid_coord_x", "Wavetable Terrain X-Axis Morpher", 0.0, 1.0, 0.0, "X", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("wavetable_grid_coord_y", "Wavetable Terrain Y-Axis Morpher", 0.0, 1.0, 0.0, "Y", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_anti_aliasing_guard", "Band-Limited Harmonic Cutoff Margin", 0.5, 1.0, 0.95, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spline_tension_parameter", "Bicubic Spline Curve Tension", 0.0, 1.0, 0.5, "tau", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_foundation_level", "Sub-Harmonic Foundation Sine Reinforce", 0.0, 1.0, 0.25, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_continuous_gradient_rotation", "Continuous Autonomous Terrain Rotation", false, (14, 165, 233)))
        ),
        "ChaoticChuaCircuitOscillator" => Box::new(
            GenericDspNodeUi::new("ChaoticChuaCircuitOscillator", "Chua Circuit Double-Scroll Strange Attractor Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("chua_resistor_conductance_g", "Chua Resistor Conductance (G)", 0.1, 2.0, 0.7, "mS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("inner_capacitance_c1_nf", "Primary Tank Capacitance (C1)", 1.0, 50.0, 10.0, "nF", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("outer_capacitance_c2_nf", "Coupled Tank Capacitance (C2)", 10.0, 500.0, 100.0, "nF", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("chua_diode_break_slope_ga", "Inner Piecewise Diode Slope (Ga)", -1.5, -0.1, -0.75, "slope", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("chua_diode_outer_slope_gb", "Outer Piecewise Diode Slope (Gb)", -0.8, 0.5, -0.41, "slope", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_double_scroll_attractor_lock", "Enforce Chaotic Double-Scroll Regime", true, (14, 165, 233)))
        ),
        "SubwooferHarmonicSynthesizer" => Box::new(
            GenericDspNodeUi::new("SubwooferHarmonicSynthesizer", "Infrasonic Sub-Harmonic Waveform Extender & Chest-Thump Synth", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_1_mix_level_db", "Sub-Octave -1 Octave Synthesis Level", -40.0, 6.0, -3.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_octave_2_infrasonic_level_db", "Sub-Octave -2 Infrasonic Rumble Level", -40.0, 6.0, -12.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_lock_speed_ms", "Zero-Crossing Pitch Tracking Response", 1.0, 50.0, 6.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonic_saturation_overtones_db", "Sub-Bass Upper Harmonic Saturation", 0.0, 24.0, 6.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_bass_lowpass_cutoff_hz", "Infrasonic Sub-Bass Lowpass Filter", 30.0, 160.0, 85.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_monophonic_bass_phase_alignment", "Monophonic Center Phase-Locking", true, (14, 165, 233)))
        ),
        "InfrasonicSubBassExciter" => Box::new(
            GenericDspNodeUi::new("InfrasonicSubBassExciter", "Infrasonic Pitch-Locked Sub-Bass Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_frequency_floor_hz", "Sub-Bass Lower Infrasonic Floor", 20.0, 80.0, 35.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("infrasonic_output_level_db", "Infrasonic Sub Waveform Output Gain", -36.0, 6.0, 0.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_latency_ms", "Fundamental Pitch Lock Speed", 2.0, 40.0, 8.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonics_drive_saturation_db", "Audible Overtones Harmonic Saturation", 0.0, 18.0, 4.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("dynamic_envelope_follow_speed_ms", "Dynamics Envelope Follower Response", 5.0, 200.0, 30.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_phase_lock_to_source", "Cycle-Accurate Center Phase-Locking", true, (14, 165, 233)))
        ),
        "PsychoacousticFundamentalSynthesizer" => Box::new(
            GenericDspNodeUi::new("PsychoacousticFundamentalSynthesizer", "Psychoacoustic Missing Fundamental Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("target_virtual_sub_frequency_hz", "Target Sub-Bass Virtual Fundamental", 30.0, 100.0, 45.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_series_order_count", "Generated Harmonic Series Multiplier", 2.0, 8.0, 4.0, "harm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("harmonic_amplitude_decay_slope_db", "Harmonic Series Attenuation Slope", -12.0, -1.0, -4.5, "dB/oct", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_fundamental_cut_steepness_db", "Original Inaudible Sub Attenuation", 0.0, 48.0, 18.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sidechain_crossover_frequency_hz", "Sub-Bass Detection Crossover", 50.0, 250.0, 90.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_odd_harmonic_interlock", "Odd/Even Harmonic Dynamic Interlock", true, (14, 165, 233)))
        ),
        "GaussianStochasticGrainSynthesizer" => Box::new(
            GenericDspNodeUi::new("GaussianStochasticGrainSynthesizer", "Gaussian Stochastic Granular Cloud Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("grain_emission_density_hz", "Average Grain Cloud Generation Rate", 2.0, 300.0, 65.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("center_frequency_pitch_hz", "Gaussian Pitch Distribution Center", 40.0, 4000.0, 440.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("frequency_variance_spread_semitones", "Pitch Gaussian Spread Standard Deviation", 0.0, 24.0, 3.5, "st", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_mean_ms", "Mean Grain Envelope Window Duration", 5.0, 500.0, 45.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spatial_binaural_spread_angle_deg", "Stereo Cloud Azimuth Angular Scatter", 0.0, 180.0, 90.0, "deg", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_quasi_synchronous_bursts", "Rhythmic Synchronous Burst Mode", false, (14, 165, 233)))
        ),
        "HyperdimensionalQuaternionOscillator" => Box::new(
            GenericDspNodeUi::new("HyperdimensionalQuaternionOscillator", "4D Quaternion Rotation Waveform Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("quaternion_rotation_speed_hz", "4D Rotor Frequency Speed", 0.1, 2000.0, 110.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereographic_projection_pole", "Hyperspherical Projection Focal Pole", -2.0, 2.0, 1.0, "coord", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("quaternion_rotor_axis_tilt", "Euler-Rodrigues Axis Nutation", 0.0, 360.0, 45.0, "deg", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spinor_phase_warp_curvature", "Spinor Trajectory Phase Curvature", 0.0, 1.0, 0.25, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("hypersurface_symmetry_folding", "4D Manifold Symmetry Fold Count", 1.0, 8.0, 2.0, "folds", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_symplectic_energy_conservation", "Symplectic Integrator Phase Lock", true, (14, 165, 233)))
        ),
        "SubharmonicBassReinforcementExciter" => Box::new(
            GenericDspNodeUi::new("SubharmonicBassReinforcementExciter", "Sub-Harmonic Resonant Bass Reinforcement", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_blend_gain_db", "Sub-Harmonic Output Blend Level", -30.0, 12.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_tracking_lowpass_cutoff_hz", "Pitch Tracker Input Low-Pass Cutoff", 40.0, 250.0, 110.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("envelope_attack_snap_ms", "Dynamic Envelope Attack Tracking Time", 0.5, 50.0, 4.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_waveform_saturation_drive", "Sub Waveform Harmonic Distortion Drive", 0.0, 18.0, 3.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_alignment_offset_deg", "Sub-To-Dry Phase Alignment Offset", -180.0, 180.0, 0.0, "deg", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_infrasonic_highpass_guard", "18Hz Infrasonic Subwoofer High-Pass Guard", true, (14, 165, 233)))
        ),
        "PolyphonicPulsarCloudSynthesizer" => Box::new(
            GenericDspNodeUi::new("PolyphonicPulsarCloudSynthesizer", "Polyphonic Pulsar Cloud Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("pulsar_emission_rate_hz", "Pulsar Train Emission Rate", 1.0, 500.0, 80.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("pulsaret_duty_cycle_ratio", "Pulsaret Duty Cycle Formant Width", 0.01, 1.0, 0.25, "ratio", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_masking_frequency_hz", "Formant Envelope Center Frequency", 100.0, 10000.0, 1800.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pulsar_burst_jitter_ms", "Stochastic Pulsar Interval Jitter", 0.0, 100.0, 15.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_pulsar_dispersion_width", "Stereo Soundstage Aperture Width", 0.0, 1.0, 0.8, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_pulsaret_envelope_windowing", "Hann Window Envelope Smoothing", true, (14, 165, 233)))
        ),
        "StochasticChaoticJitterSynthesizer" => Box::new(
            GenericDspNodeUi::new("StochasticChaoticJitterSynthesizer", "Stochastic Chaotic Jitter Frequency Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("oscillator_fundamental_pitch_hz", "Oscillator Carrier Pitch Frequency", 20.0, 8000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("brownian_jitter_frequency_deviation_cents", "Brownian Motion Pitch Jitter Deviation", 0.0, 50.0, 8.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("fractal_hurst_exponent", "Fractal Drift Hurst Exponent (0.1-0.9)", 0.1, 0.9, 0.7, "H", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("thermal_amplitude_noise_db", "Thermal Shot Noise Floor Modulation", -90.0, -20.0, -54.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("oscillator_waveform_morph", "Waveform Morph (Sine -> Tri -> Saw -> Square)", 0.0, 3.0, 2.0, "wave", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_anti_aliased_polyblep", "PolyBLEP Band-Limited Edge Correction", true, (14, 165, 233)))
        ),
        "CavitationBubbleCollapseExciter" => Box::new(
            GenericDspNodeUi::new("CavitationBubbleCollapseExciter", "Sonoluminescent Cavitation Bubble Collapse Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("ambient_liquid_static_pressure_bar", "Liquid Static Ambient Pressure", 0.5, 50.0, 1.01, "bar", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("acoustic_driving_amplitude_kpa", "Ultrasonic Drive Acoustic Amplitude", 10.0, 500.0, 135.0, "kPa", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bubble_equilibrium_radius_microns", "Bubble Equilibrium Radius", 1.0, 100.0, 5.0, "um", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("liquid_surface_tension_dyn_cm", "Fluid Solvent Surface Tension", 10.0, 80.0, 72.8, "dyn/cm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("bubble_collapse_implosion_gain_db", "Cavitation Implosion Transient Output", -48.0, 24.0, 0.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_polytropic_gas_compression", "Polytropic Gas Heating & Plasma Luminescence", true, (34, 197, 94)))
        ),
        "MicrotonalHarmonicLatticeOscillator" => Box::new(
            GenericDspNodeUi::new("MicrotonalHarmonicLatticeOscillator", "Multi-Dimensional Just Intonation Harmonic Lattice", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("lattice_base_tuning_frequency_hz", "Lattice Master Root Pitch", 100.0, 880.0, 216.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("prime_limit_harmonic_expansion", "Prime Factor Harmonic Horizon (3-19)", 3.0, 19.0, 7.0, "limit", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("syntonic_comma_drift_depth_cents", "Syntonic Comma Microtonal Offset", -50.0, 50.0, 0.0, "cents", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("overtone_spectral_richness", "Harmonic Series Falloff Exponent", 0.1, 2.0, 1.0, "exp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("chordal_voice_spread_count", "Active Microtonal Voices Count", 1.0, 16.0, 6.0, "voices", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_pythagorean_wolf_correction", "Pythagorean Wolf Fifth Correction", true, (34, 197, 94)))
        ),
        "PsychoacousticBarkScaleSubBassSynthesizer" => Box::new(
            GenericDspNodeUi::new("PsychoacousticBarkScaleSubBassSynthesizer", "Bark Critical-Band Sub-Bass Psychoacoustic Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("target_perceived_fundamental_hz", "Virtual Perceived Sub-Bass Pitch", 25.0, 120.0, 42.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("bark_critical_band_spacing", "Harmonic Critical-Band Spacing Ratio", 1.0, 3.0, 1.35, "bark", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_odd_even_color_blend", "Odd vs Even Overtone Distribution", -1.0, 1.0, -0.2, "color", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_mix_level_db", "Direct Pure Sub Sine Injection", -60.0, 0.0, -12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_overtone_saturation_db", "Virtual Harmonic Drive Amplification", 0.0, 24.0, 8.0, "dB", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_speaker_highpass_protection", "Speaker Excursion Protection High-Pass", true, (34, 197, 94)))
        ),
        "ChaoticStrangeAttractorSynthesizer" => Box::new(
            GenericDspNodeUi::new("ChaoticStrangeAttractorSynthesizer", "Lorenz & Rössler Strange Attractor Audio Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("attractor_fundamental_frequency_hz", "Chaotic Orbit Integration Rate", 10.0, 4000.0, 165.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("prandtl_parameter_sigma", "Lorenz Sigma Prandtl Number", 1.0, 30.0, 10.0, "sigma", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("rayleigh_parameter_rho", "Lorenz Rho Rayleigh Number", 5.0, 60.0, 28.0, "rho", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("geometric_parameter_beta", "Lorenz Beta Geometric Damping", 0.5, 5.0, 2.667, "beta", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("trajectory_nonlinearity_drive", "Attractor Orbit Non-Linearity Drive", 0.1, 5.0, 1.2, "drive", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_chua_double_scroll_mode", "Chua Double-Scroll Attractor Mode", false, (34, 197, 94)))
        ),
        "DynamicSubharmonicResonanceExciter" => Box::new(
            GenericDspNodeUi::new("DynamicSubharmonicResonanceExciter", "Adaptive Frequency-Division Subharmonic Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_division_ratio", "Frequency Division Sub Ratio (1:2, 1:3)", 1.0, 4.0, 2.0, "div", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("envelope_tracking_attack_ms", "Subharmonic Attack Tracking Speed", 0.1, 50.0, 3.5, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("envelope_tracking_release_ms", "Subharmonic Release Tail Time", 10.0, 500.0, 85.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sub_saturation_harmonics_db", "Sub Overtone Saturation Drive", 0.0, 30.0, 9.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("highpass_phase_align_deg", "Sub-Bass Phase Alignment Offset", -180.0, 180.0, 0.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_octave_fundamental_blend", "Clean Fundamental Reinforcement Blend", true, (34, 197, 94)))
        ),
        "AsynchronousGranularWavetableOscillator" => Box::new(
            GenericDspNodeUi::new("AsynchronousGranularWavetableOscillator", "Granular Morphing Wavetable Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wavetable_position_scan_speed", "Wavetable Frame Position Index", 0.0, 64.0, 16.5, "frame", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_window_overlap_factor", "Granular Cloud Overlap Density", 1.0, 16.0, 4.0, "grains", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("spectral_wave_morph_index", "Nonlinear Spectral Table Morphing", 0.0, 1.0, 0.45, "morph", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("grain_pitch_micro_detune_cents", "Inter-Grain Detune Jitter Spread", 0.0, 100.0, 18.0, "cents", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_center_frequency_hz", "Formant Filter Center Frequency", 100.0, 8000.0, 1200.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_window_sync_to_table_cycle", "Pitch-Synchronous Grain Windowing", true, (34, 197, 94)))
        ),
        "AdditiveHyperbolicPartialSynthesizer" => Box::new(
            GenericDspNodeUi::new("AdditiveHyperbolicPartialSynthesizer", "Hyperbolic Phase Additive Partial Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("spectral_centroid_tilt_slope", "Spectral Centroid Slope Tilt", -24.0, 12.0, -6.0, "dB/oct", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("hyperbolic_phase_warp_factor", "Hyperbolic Phase Warp Modulation", 0.0, 2.0, 0.45, "warp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("odd_even_harmonic_balance", "Odd vs Even Harmonic Energy Balance", -1.0, 1.0, 0.15, "bal", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("partial_inharmonicity_stiffness", "String Inharmonicity Dispersion", 0.0, 0.1, 0.003, "B", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("dynamic_envelope_decay_damping", "High Partial Damping Rate", 0.1, 10.0, 2.2, "damp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_polyblep_anti_aliasing", "Spectral Boundary Anti-Aliasing Clamp", true, (34, 197, 94)))
        ),
        "PhaseDistortionResonantCasioOscillator" => Box::new(
            GenericDspNodeUi::new("PhaseDistortionResonantCasioOscillator", "Phase Distortion Resonant Waveform Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("phase_distortion_amount_index", "Phase Distortion Modulation Index", 0.0, 1.0, 0.65, "mod", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("resonant_waveform_shape_type", "PD Resonant Waveform Algorithm", 0.0, 7.0, 1.0, "type", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("simulated_resonance_frequency_hz", "Simulated Filter Peak Resonance", 50.0, 12000.0, 1600.0, "Hz", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("amplitude_window_decay_rate", "Envelope Window Decay Damping", 0.1, 10.0, 2.5, "damp", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sub_oscillator_detune_cents", "Second Line Phase Detuning Offset", 0.0, 50.0, 8.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_octave_ring_multiplication", "Ring Modulation Between PD Lines", false, (34, 197, 94)))
        ),
        "FractalIteratedFunctionNoiseSynthesizer" => Box::new(
            GenericDspNodeUi::new("FractalIteratedFunctionNoiseSynthesizer", "Iterated Function System (IFS) Fractal Noise Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("fractal_dimension_hausdorff", "Hausdorff Self-Similarity Dimension", 1.0, 2.0, 1.45, "dim", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("affine_contraction_factor", "IFS Transformation Contraction Ratio", 0.1, 0.99, 0.68, "scale", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("attractor_rotation_angle_deg", "Phase Space Affine Rotation", 0.0, 360.0, 72.0, "deg", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stochastic_perturbation_entropy", "Brownian Perturbation Jitter", 0.0, 1.0, 0.22, "chaos", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("bandpass_formant_tracking_hz", "Fractal Resonance Focus Center", 50.0, 12000.0, 680.0, "Hz", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_deterministic_seed_lock", "Deterministic PRNG Playback Seed", false, (99, 102, 241)))
        ),
        "SubharmonicBassSynthesizerSubGenerator" => Box::new(
            GenericDspNodeUi::new("SubharmonicBassSynthesizerSubGenerator", "Subharmonic Sub-Octave Bass Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("sub_octave_blend_gain_db", "Subharmonic Level Blend Gain", -48.0, 6.0, -3.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("sub_divider_octave_selector", "Sub-Octave Division Ratio (1 or 2 oct)", 1.0, 2.0, 1.0, "oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_drive_saturation", "Low-End Soft Saturation Warmth", 0.0, 1.0, 0.35, "drive", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_lowpass_corner_hz", "Fundamental Pitch Tracker Lowpass Cutoff", 40.0, 300.0, 110.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_decay_damping_time_ms", "Sub-Bass Sustain Decay Smoothing", 10.0, 1000.0, 180.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_phase_alignment_lock", "Transient Phase Alignment Lock", true, (34, 197, 94)))
        ),
        "SuperformulaChaoticGeometryOscillator" => Box::new(
            GenericDspNodeUi::new("SuperformulaChaoticGeometryOscillator", "Gielis Superformula 3D Geometric Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("rotational_symmetry_m_lobes", "Rotational Symmetry Order (M Lobes)", 1.0, 24.0, 5.0, "lobes", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("superformula_exponent_n1", "Radial Curvature Shape Exponent (N1)", 0.1, 10.0, 1.2, "n1", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("superformula_exponent_n2", "Asymmetry Harmonic Exponent (N2)", 0.1, 10.0, 2.5, "n2", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("superformula_exponent_n3", "Quadrant Balance Exponent (N3)", 0.1, 10.0, 2.5, "n3", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("orbit_angular_velocity_rad_sec", "Phase Trajectory Orbital Rotation Rate", 0.0, 50.0, 3.14, "rad/s", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_bandlimited_supersampling", "PolyBLEP Antialiased Oversampling", true, (34, 197, 94)))
        ),
        "AdditivePartialsResonatorBankOscillator" => Box::new(
            GenericDspNodeUi::new("AdditivePartialsResonatorBankOscillator", "128-Partial Spectral Inharmonic Resonator Bank", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("harmonic_inharmonicity_stiffness", "Inharmonic Dispersion Stiffness Factor", 0.0, 0.05, 0.0018, "stiff", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_slope_db_per_oct", "Spectral Partials Amplitude Roll-off Slope", -24.0, 0.0, -6.0, "dB/oct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("odd_to_even_partial_ratio", "Odd vs Even Partials Amplitude Ratio", 0.0, 1.0, 0.5, "ratio", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("partial_decay_time_damping_scale", "High-Frequency Partial Decay Time Damping", 0.1, 5.0, 1.0, "damp", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("spectral_comb_notch_spread", "Spectral Formant Comb Notch Spacing", 0.0, 1.0, 0.35, "comb", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_simd_avx2_vectorized_evaluation", "AVX2 SIMD Vectorized Batch Evaluation", true, (34, 197, 94)))
        ),
        "RelativisticDopplerChaoticWaveformOscillator" => Box::new(
            GenericDspNodeUi::new("RelativisticDopplerChaoticWaveformOscillator", "Relativistic Spacetime Lorentz Waveform Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("relativistic_velocity_beta_c", "Velocity Ratio Relative to Speed of Light (v/c)", 0.0, 0.99, 0.45, "beta", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("lorentz_strange_attractor_prandtl", "Chaotic Prandtl Number Convection Parameter", 1.0, 30.0, 10.0, "sigma", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("chaotic_rayleigh_instability_param", "Rayleigh Instability Strange Attractor Driving", 10.0, 60.0, 28.0, "rho", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("phase_space_orbital_radius", "Chaotic Phase Space Orbital Radius", 0.1, 5.0, 1.2, "rad", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("polyblep_subsample_smoothing", "Anti-Aliased PolyBLEP Waveform Smoothing", 0.0, 1.0, 0.85, "smooth", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_relativistic_headlight_beaming", "Relativistic Beam Forward Intensity Modulation", true, (34, 197, 94)))
        ),
        "FractionalPuckettGranularCloudOscillator" => Box::new(
            GenericDspNodeUi::new("FractionalPuckettGranularCloudOscillator", "Fractional Brownian Granular Particle Cloud Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("fractional_hurst_exponent_h", "Fractional Brownian Motion Hurst Exponent (H)", 0.05, 0.95, 0.62, "H", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("grain_emission_density_per_sec", "Granular Particle Emission Rate", 5.0, 500.0, 85.0, "grains/s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("grain_window_gaussian_sigma_ms", "Gaussian Grain Window Half-Width", 2.0, 150.0, 25.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_fractal_tilt_db_per_oct", "Fractal 1/f^alpha Spectral Slope", -18.0, 0.0, -4.5, "dB/oct", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stochastic_pitch_dispersion_cents", "Granular Cloud Random Pitch Scatter", 0.0, 600.0, 45.0, "cents", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_quantum_shot_noise_burst", "Micro-Acoustic Quantum Shot Noise Bursts", true, (34, 197, 94)))
        ),
        "FractionalChaosDuffingOscillator" => Box::new(
            GenericDspNodeUi::new("FractionalChaosDuffingOscillator", "Fractional Duffing Double-Well Chaotic Waveform Generator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("cubic_nonlinearity_stiffness_beta", "Duffing Cubic Potential Well Nonlinearity (Beta)", 0.1, 10.0, 2.4, "beta", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("periodic_driving_force_gamma", "Sinusoidal External Driving Force Amplitude (Gamma)", 0.0, 5.0, 1.65, "amp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("fractional_damping_order_alpha", "Fractional Riemann-Liouville Damping Order (Alpha)", 0.1, 1.9, 0.85, "order", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("driving_frequency_omega_hz", "Periodic Driving Frequency (Omega)", 10.0, 2000.0, 128.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("phase_plane_orbit_cross_feedback", "Attractor Phase-Plane Position Velocity Cross-Mod", 0.0, 1.0, 0.35, "fdbk", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_polyblep_bifurcation_antialias", "PolyBLEP Sub-Sample Chaotic Attractor Anti-Aliasing", true, (34, 197, 94)))
        ),
        "SuperformulaStochasticMorphOscillator" => Box::new(
            GenericDspNodeUi::new("SuperformulaStochasticMorphOscillator", "Gielis 3D Superformula Stochastic Morphing Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("superformula_symmetry_rotations_m", "Rotational Radial Symmetry Index (m)", 1.0, 24.0, 6.0, "m", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("radial_pinching_exponent_n1", "Superformula Radial Pinching Exponent (n1)", 0.1, 20.0, 2.2, "n1", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lobed_contour_asymmetry_n2_n3", "Quadrant Exponent Curvature Ratio (n2/n3)", 0.2, 5.0, 1.0, "ratio", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stochastic_morph_orbit_speed_hz", "Chaotic Morphing Orbital Trajectory Speed", 0.02, 20.0, 0.45, "Hz", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("wavetable_subsample_oversample_ratio", "Sub-Sample Sinc Kernel Oversampling Depth", 1.0, 8.0, 4.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_golden_ratio_phi_modulation", "Fibonacci Golden Ratio Phi Harmonics Lock", true, (34, 197, 94)))
        ),
        "CoupledChaoticVanDerPolNonlinearOscillator" => Box::new(
            GenericDspNodeUi::new("CoupledChaoticVanDerPolNonlinearOscillator", "Coupled Van der Pol Limit-Cycle Chaotic Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("base_oscillation_frequency_hz", "Master Oscillator Fundamental Limit-Cycle Frequency", 10.0, 2000.0, 110.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nonlinear_damping_epsilon", "Van der Pol Nonlinear Damping Factor (Epsilon)", 0.01, 10.0, 1.8, "eps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cross_coupling_conductance_k", "Inter-Oscillator Mutual Coupling Conductance", 0.0, 5.0, 0.85, "coup", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("frequency_ratio_detuning", "Secondary Oscillator Detune Frequency Ratio", 0.5, 3.0, 1.333, "ratio", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("phase_space_limit_cycle_tilt", "Phase-Space Orbit Asymmetry Skew", -1.0, 1.0, 0.0, "skew", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_bifurcation_sync_reset", "Hard Phase Sync Reset on Transient Impulse", false, (34, 197, 94)))
        ),
        "GranularStochasticPoissonTextureSynthesizer" => Box::new(
            GenericDspNodeUi::new("GranularStochasticPoissonTextureSynthesizer", "Poisson Stochastic Asynchronous Granular Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("grain_density_poisson_rate_hz", "Poisson Mean Grain Emission Rate", 1.0, 1000.0, 120.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("grain_duration_length_ms", "Individual Audio Grain Duration Length", 2.0, 250.0, 35.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stochastic_pitch_scatter_semitones", "Grain Pitch Random Distribution Jitter", 0.0, 24.0, 3.5, "st", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("grain_envelope_window_morph", "Grain Envelope Window Shape Morphing", 0.0, 1.0, 0.5, "morph", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stereo_spatial_cloud_spread", "Panoramic Stereo Cloud Position Scattering", 0.0, 1.0, 0.85, "spread", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_reverse_grain_probability", "Reverse Grain Playback Direction Probability", false, (34, 197, 94)))
        ),
        "HyperchaoticLorentzAttractorSynthesizer" => Box::new(
            GenericDspNodeUi::new("HyperchaoticLorentzAttractorSynthesizer", "4D Hyperchaotic Lorenz Strange Attractor Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("lorentz_prandtl_sigma", "Lorenz Prandtl Fluid Parameter (Sigma)", 1.0, 30.0, 10.0, "sigma", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("lorentz_rayleigh_rho", "Lorenz Rayleigh Convection Parameter (Rho)", 5.0, 60.0, 28.0, "rho", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hyperchaotic_dimension_w_gain", "4th Hyperchaotic Dimension (w) Feedback", 0.0, 5.0, 1.8, "w", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_space_integration_speed_hz", "Phase Space Trajectory Speed", 20.0, 4000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("attractor_butterfly_wing_bias", "Dual-Lobe Butterfly Symmetry Bias", -1.0, 1.0, 0.0, "bias", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_runaway_boundary_limiting", "Phase Space Elastic Boundary Clamping", true, (34, 197, 94)))
        ),
        "AdditiveMorphingInharmonicBellGenerator" => Box::new(
            GenericDspNodeUi::new("AdditiveMorphingInharmonicBellGenerator", "64-Partial Additive Inharmonic Bell Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("fundamental_strike_pitch_hz", "Bell Fundamental Strike Pitch (Hum Tone)", 30.0, 1200.0, 261.6, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_dispersion_morph", "Inharmonic Partial Distribution (Chime to Gong)", 0.5, 3.5, 1.85, "disp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_damping_rate", "High-Order Partial Decay Damping Rate", 0.1, 10.0, 2.5, "damp", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("strike_transient_brightness", "Mallet Impact Noise & Upper Partial Burst", 0.0, 1.0, 0.65, "bright", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("hum_tone_sustain_seconds", "Fundamental Hum Tone Free Decay Time", 0.5, 30.0, 8.0, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_clapper_double_strike", "Clapper Double-Strike Re-excitation", false, (34, 197, 94)))
        ),
        "NonlinearCoupledVanDerPolChaosSynthesizer" => Box::new(
            GenericDspNodeUi::new("NonlinearCoupledVanDerPolChaosSynthesizer", "Nonlinear Coupled Van der Pol Chaotic Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("oscillator1_natural_frequency_hz", "Master Oscillator 1 Natural Frequency", 20.0, 2000.0, 110.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("oscillator2_natural_frequency_hz", "Slave Oscillator 2 Natural Frequency", 20.0, 2000.0, 165.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("nonlinearity_parameter_mu", "Nonlinear Damping Coefficient (Mu)", 0.1, 10.0, 2.5, "mu", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("mutual_coupling_strength_k", "Mutual Oscillator Cross-Coupling (k)", 0.0, 5.0, 1.2, "k", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("phase_state_variable_mix", "Position (x) vs Velocity (dx/dt) Mix", 0.0, 1.0, 0.5, "mix", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_bool("enable_subharmonic_period_doubling", "Bifurcation Subharmonic Period Doubling", true, (34, 197, 94)))
        ),
        "AdditiveSpectralInharmonicPartialsSynthesizer" => Box::new(
            GenericDspNodeUi::new("AdditiveSpectralInharmonicPartialsSynthesizer", "128-Partial Additive Inharmonic Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("partial_count", "Active Additive Sinusoid Count", 8.0, 128.0, 64.0, "parts", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_stretch_power", "Inharmonic Overtone Frequency Stretch Power", 0.5, 3.0, 1.0, "exp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_slope_db_oct", "Harmonic Roll-Off Slope Across Spectrum", -18.0, 0.0, -6.0, "dB/oct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("high_partial_decay_acceleration", "High-Partial Decay Damping Acceleration", 0.5, 5.0, 2.2, "accel", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("initial_phase_randomization_deg", "Initial Sinusoid Phase Randomization", 0.0, 360.0, 180.0, "deg", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_odd_even_partial_comb_balance", "Even Partial Attenuation (Hollow Square)", false, (34, 197, 94)))
        ),
        "NonlinearDoublePendulumChaosSynthesizer" => Box::new(
            GenericDspNodeUi::new("NonlinearDoublePendulumChaosSynthesizer", "Nonlinear Double Pendulum Chaos Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("pendulum_mass_ratio", "Pendulum Arm Mass Ratio", 0.1, 10.0, 1.5, "ratio", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("rod_length_ratio", "Rod Arm Length Ratio", 0.1, 5.0, 1.0, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gravity_energy", "Gravitational Acceleration", 0.1, 50.0, 9.81, "m/s2", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("friction_damping", "Joint Bearing Friction Loss", 0.0, 0.5, 0.02, "fric", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("chaos_orbit_mod", "Orbit Audio Projection Depth", 0.0, 1.0, 0.70, "depth", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("frequency_warp", "Base Oscillator Fundamental", 20.0, 4000.0, 220.0, "Hz", (34, 197, 94)))
        ),
        "WavetableFormantGranularHybridOscillator" => Box::new(
            GenericDspNodeUi::new("WavetableFormantGranularHybridOscillator", "Wavetable Formant Granular Hybrid Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("wavetable_pos", "Wavetable Frame Scanning Position", 0.0, 1.0, 0.25, "pos", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("grain_size", "Granular Formant Window Length", 5.0, 250.0, 40.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("grain_density", "Grains Per Audio Cycle", 1.0, 64.0, 16.0, "grains", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_shift", "Formant Center Transposition", -24.0, 24.0, 0.0, "semi", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spray_jitter", "Granular Stochastic Jitter", 0.0, 1.0, 0.15, "jitter", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sub_osc_level", "Sub-Octave Sine Reinforcement", 0.0, 1.0, 0.30, "gain", (34, 197, 94)))
        ),
        "NonlinearChuaCircuitAttractorSynthesizer" => Box::new(
            GenericDspNodeUi::new("NonlinearChuaCircuitAttractorSynthesizer", "Nonlinear Chua Circuit Attractor Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("inductance_l", "Resonant Inductor Value (L)", 0.1, 10.0, 1.42, "H", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("capacitance_c1", "State Variable Fast Capacitor (C1)", 1.0, 50.0, 10.0, "nF", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("capacitance_c2", "State Variable Slow Capacitor (C2)", 10.0, 500.0, 100.0, "nF", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("diode_slope_inner", "Chua Diode Inner Slope (Ga)", -2.0, 0.0, -0.75, "mS", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diode_slope_outer", "Chua Diode Outer Slope (Gb)", -1.0, 1.0, -0.41, "mS", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("base_clock_frequency", "Simulation Audio Clock Rate", 20.0, 5000.0, 261.6, "Hz", (34, 197, 94)))
        ),
        "AdditiveSpectralFormantHarmonicOscillator" => Box::new(
            GenericDspNodeUi::new("AdditiveSpectralFormantHarmonicOscillator", "Additive Spectral Formant Harmonic Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("fundamental_freq", "Oscillator Fundamental Pitch", 20.0, 4000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("partial_count", "Simultaneous Active Partials", 8.0, 64.0, 32.0, "partials", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("formant_vowel_focus", "Vowel Formant Profile Index", 0.0, 4.0, 1.0, "vowel", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_db", "Spectral High Roll-Off Tilt", -18.0, 0.0, -6.0, "dB/oct", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("odd_even_harmonic_ratio", "Odd vs Even Partials Ratio", 0.0, 1.0, 0.50, "ratio", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_spectral_stretch", "Partials Inharmonicity Stretch", 0.9, 1.5, 1.0, "warp", (34, 197, 94)))
        ),
        "NonlinearRabinovichFabrikantChaosSynthesizer" => Box::new(
            GenericDspNodeUi::new("NonlinearRabinovichFabrikantChaosSynthesizer", "Rabinovich-Fabrikant Chaotic Attractor Synthesizer", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("chaos_parameter_alpha", "Dissipation Parameter (Alpha)", 0.01, 0.5, 0.14, "alpha", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("chaos_parameter_gamma", "Bifurcation Control (Gamma)", 0.05, 1.5, 0.28, "gamma", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("integration_time_step", "Runge-Kutta Step Frequency", 20.0, 2000.0, 220.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("orbital_radius_projection", "Strange Attractor Orbit Scale", 0.1, 5.0, 1.2, "rad", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("attractor_axis_blend", "X/Y/Z Coordinate Projection Mix", 0.0, 2.0, 0.5, "axis", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("subharmonic_fold_feedback", "Nonlinear Cross-Axis Feedback", 0.0, 1.0, 0.40, "fb", (34, 197, 94)))
        ),
        "AdditiveHarmonicSpectralMorphOscillator" => Box::new(
            GenericDspNodeUi::new("AdditiveHarmonicSpectralMorphOscillator", "Additive Harmonic Spectral Morph Oscillator", DspNodeCategory::Oscillator)
                .with_param(DspParamDescriptor::new_linear("base_fundamental_frequency", "Oscillator Fundamental Tuning", 20.0, 2000.0, 130.81, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt_rolloff", "Partial Harmonic Spectral Tilt", -24.0, 6.0, -6.0, "dB/oct", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("odd_even_harmonic_balance", "Odd vs Even Partial Ratio", -1.0, 1.0, 0.0, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("inharmonic_frequency_stretch", "Inharmonic Partials Frequency Stretch", 0.0, 1.0, 0.12, "stretch", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_peak_center", "Resonant Spectral Formant Center", 200.0, 5000.0, 1200.0, "Hz", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("active_partials_count", "Simultaneous Partials Density", 8.0, 64.0, 32.0, "pts", (34, 197, 94)))
        ),
        _ => return None,
    };
    Some(node)
}
