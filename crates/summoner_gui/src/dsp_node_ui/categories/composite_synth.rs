// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `CompositeSynth`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("AetherSynth".to_string(), DspNodeDescriptor::new("AetherSynth", "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter", DspNodeCategory::CompositeSynth, "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter")
        .with_param(DspParamSchema::knob("osc_mix", "Saw / Pulse Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Saw / Pulse Mix parameter"))
        .with_param(DspParamSchema::log_knob("filter_cutoff", "Moog Filter Cutoff", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Moog Filter Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("filter_res", "Ladder Resonance", 0.1, 4.0, 1.0, "Q", MacroRole::Tone, "Ladder Resonance parameter"))
        .with_param(DspParamSchema::log_knob("lfo_speed", "PWM LFO Speed", 0.1, 20.0, 2.0, "Hz", MacroRole::Tone, "PWM LFO Speed logarithmic parameter"))
    );
    descriptors.insert("CyberpunkSubSynth".to_string(), DspNodeDescriptor::new("CyberpunkSubSynth", "Heavy Sub-bass Synth with Asymmetric Wavefolding", DspNodeCategory::CompositeSynth, "Heavy Sub-bass Synth with Asymmetric Wavefolding")
        .with_param(DspParamSchema::log_knob("sub_freq", "Sub Oscillator Freq", 20.0, 200.0, 55.0, "Hz", MacroRole::Tone, "Sub Oscillator Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("drive_gain", "Wavefolder Drive", 1.0, 10.0, 4.0, "x", MacroRole::Tone, "Wavefolder Drive parameter"))
        .with_param(DspParamSchema::log_knob("filter_cutoff", "SVF Filter Cutoff", 40.0, 2000.0, 600.0, "Hz", MacroRole::Tone, "SVF Filter Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("pitch_wobble", "LFO Pitch Wobble", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "LFO Pitch Wobble parameter"))
    );
    descriptors.insert("AtmosphericPadSynth".to_string(), DspNodeDescriptor::new("AtmosphericPadSynth", "Detuned Ambient Pad with SVF Filter & Space Reverb", DspNodeCategory::CompositeSynth, "Detuned Ambient Pad with SVF Filter & Space Reverb")
        .with_param(DspParamSchema::knob("saw_tri_mix", "Saw / Detuned Tri Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Saw / Detuned Tri Mix parameter"))
        .with_param(DspParamSchema::log_knob("filter_cutoff", "SVF Filter Cutoff", 100.0, 5000.0, 800.0, "Hz", MacroRole::Tone, "SVF Filter Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("reverb_mix", "Space Reverb Level", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Space Reverb Level parameter"))
        .with_param(DspParamSchema::knob("delay_mix", "Stereo Delay Level", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Stereo Delay Level parameter"))
    );
    descriptors.insert("GlitchPercussionSynth".to_string(), DspNodeDescriptor::new("GlitchPercussionSynth", "Noise Burst & Granular Stutter Percussion Engine", DspNodeCategory::CompositeSynth, "Noise Burst & Granular Stutter Percussion Engine")
        .with_param(DspParamSchema::log_knob("pitch_freq", "Pitch Drop Center", 30.0, 800.0, 120.0, "Hz", MacroRole::Tone, "Pitch Drop Center logarithmic parameter"))
        .with_param(DspParamSchema::knob("bitcrush_depth", "Bitcrusher Depth", 1.0, 16.0, 8.0, "bits", MacroRole::Character, "Bitcrusher Depth integer control"))
        .with_param(DspParamSchema::log_knob("comb_freq", "Comb Resonator Freq", 50.0, 2000.0, 240.0, "Hz", MacroRole::Tone, "Comb Resonator Freq logarithmic parameter"))
        .with_param(DspParamSchema::knob("stutter_rate", "Buffer Stutter Speed", 1.0, 64.0, 16.0, "div", MacroRole::Tone, "Buffer Stutter Speed parameter"))
    );
    descriptors.insert("GlitchAetherMachine".to_string(), DspNodeDescriptor::new("GlitchAetherMachine", "Flagship Chopper & Tape-Stop Glitch Synthesizer", DspNodeCategory::CompositeSynth, "Flagship Chopper & Tape-Stop Glitch Synthesizer")
        .with_param(DspParamSchema::knob("drive", "Tube Drive Saturation", 1.0, 20.0, 3.0, "x", MacroRole::Tone, "Tube Drive Saturation parameter"))
        .with_param(DspParamSchema::log_knob("filter_cutoff", "Moog Ladder Cutoff", 50.0, 10000.0, 2500.0, "Hz", MacroRole::Tone, "Moog Ladder Cutoff logarithmic parameter"))
        .with_param(DspParamSchema::knob("chop_rate", "Glitch Chopper Rate", 1.0, 32.0, 8.0, "Hz", MacroRole::Tone, "Glitch Chopper Rate parameter"))
        .with_param(DspParamSchema::toggle("tape_stop", "Tape Stop Effect Active", false, "Tape Stop Effect Active toggle switch"))
    );
    descriptors.insert("FmMatrixSynthesizer".to_string(), DspNodeDescriptor::new("FmMatrixSynthesizer", "6-Operator DX-Style FM Matrix Synthesizer", DspNodeCategory::CompositeSynth, "6-Operator DX-Style FM Matrix Synthesizer")
        .with_param(DspParamSchema::knob("mod_index", "FM Modulation Index", 0.0, 10.0, 3.5, "idx", MacroRole::Tone, "FM Modulation Index parameter"))
        .with_param(DspParamSchema::choice("algorithm", "FM Routing Algorithm", &["Alg 1 (Serial 6-Op)", "Alg 5 (Parallel Pairs)", "Alg 16 (3-Stack Hybrid)", "Alg 32 (All Parallel)"], 0, "FM Routing Algorithm mode selector"))
        .with_param(DspParamSchema::knob("feedback", "Op 6 Self-Feedback", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Op 6 Self-Feedback parameter"))
        .with_param(DspParamSchema::knob("master_level", "Master Output Level", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Master Output Level parameter"))
    );
    descriptors.insert("FusionResonanceSynth".to_string(), DspNodeDescriptor::new("FusionResonanceSynth", "Tokamak Fusion Magnetic Plasma Resonance Synth", DspNodeCategory::CompositeSynth, "Tokamak Fusion Magnetic Plasma Resonance Synth")
        .with_param(DspParamSchema::knob("magnetic_confinement_t", "Magnetic Field Flux (B)", 1.0, 15.0, 5.5, "Tesla", MacroRole::Tone, "Magnetic Field Flux (B) parameter"))
        .with_param(DspParamSchema::knob("plasma_temp_kev", "Core Plasma Temp", 1.0, 50.0, 15.0, "keV", MacroRole::Tone, "Core Plasma Temp parameter"))
        .with_param(DspParamSchema::log_knob("cyclotron_freq_hz", "Ion Cyclotron Pitch", 100.0, 10000.0, 880.0, "Hz", MacroRole::Tone, "Ion Cyclotron Pitch logarithmic parameter"))
        .with_param(DspParamSchema::knob("fusion_gain", "Acoustic Output Level", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Acoustic Output Level parameter"))
    );
    descriptors.insert("TonewheelOrganNode".to_string(), DspNodeDescriptor::new("TonewheelOrganNode", "Vintage 91-Wheel Synchronous Magnetic Tonewheel Organ Console", DspNodeCategory::CompositeSynth, "Electromechanical 91-wheel synchronous tonewheel organ console with 9 harmonic drawbars, key click, scanner vibrato, and preamp overdrive")
        .with_param(DspParamSchema::choice("drawbar_preset", "Harmonic Drawbar Registration", &["888000000 (Jazz Jimmy)", "888888888 (Gospel Full)", "800000008 (Rock Lead)", "008800000 (Warm Mellowness)", "803800000 (Blues Flute)"], 0, "Classic harmonic drawbar registration setup"))
        .with_param(DspParamSchema::knob("key_click_gain", "Mechanical Key Click Attack", 0.0, 1.0, 0.45, "gain", MacroRole::Punch, "Mechanical contact bounce noise when nine contacts engage"))
        .with_param(DspParamSchema::knob("leakage_crosstalk", "Adjacent Tonewheel Crosstalk", 0.0, 0.3, 0.05, "leak", MacroRole::Tone, "Magnetic leakage pickup crosstalk from neighboring spinning wheels"))
        .with_param(DspParamSchema::choice("vibrato_chorus_mode", "Scanner Vibrato/Chorus", &["Off", "V-1", "C-1", "V-2", "C-2", "V-3", "C-3 (Deep Chorus)"], 6, "Electromechanical delay line scanner rotor vibrato/chorus"))
        .with_param(DspParamSchema::knob("percussion_decay_ms", "Percussion Harmonic Decay", 100.0, 1000.0, 350.0, "ms", MacroRole::Character, "Decay envelope duration of harmonic percussion"))
        .with_param(DspParamSchema::knob("preamp_drive", "12AU7 Tube Preamp Drive", 0.0, 1.0, 0.35, "drive", MacroRole::Punch, "Harmonic saturation from the integrated tube console preamplifier"))
    );
    descriptors.insert("MolecularVibrationResonatorNode".to_string(), DspNodeDescriptor::new("MolecularVibrationResonatorNode", "Zero-Gravity Fluid Molecular Vibrational Quantum Resonance Synthesizer", DspNodeCategory::CompositeSynth, "Zero-gravity fluid molecular vibrational quantum resonance synthesizer modeling diatomic infrared vibrational band spectra")
        .with_param(DspParamSchema::choice("molecule_profile", "Target Molecule Species", &["Water H2O", "Carbon Dioxide CO2", "Methane CH4", "Benzene C6H6", "Fullerene C60"], 0, "Molecular vibrational eigenmode frequency matrix"))
        .with_param(DspParamSchema::log_knob("base_frequency_hz", "Audible Scaled Fundamental", 30.0, 2000.0, 160.0, "Hz", MacroRole::Tone, "Transposed base frequency of fundamental stretching mode"))
        .with_param(DspParamSchema::knob("anharmonicity_factor", "Morse Potential Anharmonicity", 0.0, 0.5, 0.08, "x_e", MacroRole::Character, "Deviation from harmonic oscillation at higher energy states"))
        .with_param(DspParamSchema::knob("thermal_excitation_temp_k", "Kinetic Temperature", 50.0, 2000.0, 298.0, "K", MacroRole::Punch, "Maxwell-Boltzmann thermal population of excited vibrational levels"))
        .with_param(DspParamSchema::knob("collisional_broadening", "Collisional Band Broadening", 0.1, 10.0, 1.8, "Q", MacroRole::Space, "Lorentzian resonance line broadening from collisions"))
    );
    descriptors.insert("PlasmaArcSynthesizerNode".to_string(), DspNodeDescriptor::new("PlasmaArcSynthesizerNode", "Modulated High-Voltage Singing Plasma Arc Spark Acoustic Generator", DspNodeCategory::CompositeSynth, "Modulated high-voltage singing plasma arc electric spark acoustic generator with thermal expansion shockwave sound synthesis")
        .with_param(DspParamSchema::knob("arc_current_ma", "Discharge Arc Current", 10.0, 500.0, 120.0, "mA", MacroRole::Punch, "Electrical current flowing through ionized air plasma channel"))
        .with_param(DspParamSchema::knob("spark_gap_mm", "Electrode Spark Gap", 1.0, 50.0, 15.0, "mm", MacroRole::Tone, "Physical separation distance between tungsten electrodes"))
        .with_param(DspParamSchema::knob("thermal_expansion_gain", "Thermal Shock Expansion Gain", 0.1, 5.0, 1.5, "gain", MacroRole::Punch, "Rapid Joule-heating expansion pressure impulse intensity"))
        .with_param(DspParamSchema::log_knob("carrier_switching_khz", "PWM Ultrasonic Carrier", 20.0, 100.0, 48.0, "kHz", MacroRole::Tone, "Inverter switching frequency modulating the plasma flame"))
        .with_param(DspParamSchema::knob("ozone_saturator_depth", "Plasma Non-Linear Saturation", 0.0, 1.0, 0.40, "%", MacroRole::Character, "Non-linear ionization resistance saturation curve"))
    );
    descriptors.insert("FusionResonanceSynthNode".to_string(), DspNodeDescriptor::new("FusionResonanceSynthNode", "Magnetic Confinement Fusion Tokamak Turbulence Plasma Sound Synthesizer", DspNodeCategory::CompositeSynth, "Magnetic confinement fusion plasma turbulence sound synthesis engine simulating tokamak Alfvén waves and geodesic acoustic modes")
        .with_param(DspParamSchema::knob("magnetic_field_tesla", "Toroidal Field Strength B", 1.0, 12.0, 5.3, "T", MacroRole::Tone, "Tokamak toroidal magnetic field determining Alfvén wave frequency"))
        .with_param(DspParamSchema::knob("plasma_density_10e19", "Core Plasma Density", 1.0, 50.0, 15.0, "m⁻³", MacroRole::Punch, "Core plasma ion density governing acoustic cutoff velocity"))
        .with_param(DspParamSchema::knob("alfven_velocity_km_s", "Alfvén Wave Speed", 500.0, 15000.0, 3200.0, "km/s", MacroRole::Tone, "Characteristic phase velocity of shear Alfvén eigenmodes"))
        .with_param(DspParamSchema::knob("turbulence_chaos_amount", "Drift Wave Micro-Turbulence", 0.0, 1.0, 0.45, "%", MacroRole::Character, "Chaotic plasma fluctuation density across resonant magnetic surfaces"))
        .with_param(DspParamSchema::knob("geodesic_mode_mix", "Geodesic Acoustic Mode (GAM) Mix", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Harmonic mix of poloidally symmetric electrostatic oscillations"))
    );
    descriptors.insert("MultiSampleBank".to_string(), DspNodeDescriptor::new("MultiSampleBank", "Multi-Velocity Multi-Zone Sample Bank HUD", DspNodeCategory::CompositeSynth, "Zero-allocation multi-sample soundfont and instrument bank with multi-velocity crossfading, round-robin sample alternation, and keyzone splitting")
        .with_param(DspParamSchema::choice("velocity_curve", "Velocity Response Dynamic Curve", &["Linear Normal", "Soft Touch (Exp)", "Hard Touch (Log)", "Fixed 127 Max"], 0, "MIDI velocity to dynamic layer gain and filter mapping response"))
        .with_param(DspParamSchema::choice("round_robin_mode", "Round-Robin Sample Alternation Mode", &["Sequential Cycle", "True Random", "Smart Non-Repeating", "Velocity Swapped"], 2, "Sample selection algorithm preventing machine-gun repetition on rapid notes"))
        .with_param(DspParamSchema::choice("sample_interpolation", "Sub-Sample Pitch Shift Interpolation", &["Hermite Cubic 4-Point", "Sinc 16-Tap Bandlimited", "Linear 2-Point", "Nearest Neighbor"], 0, "Audio sample resampler algorithm for transposing samples across keyzones"))
        .with_param(DspParamSchema::knob("release_trigger_level_db", "Key Release Sample Trigger Gain", -48.0, 0.0, -12.0, "dB", MacroRole::Tone, "Gain of mechanical key release and damper noise samples triggered on note-off"))
        .with_param(DspParamSchema::choice("voice_polyphony_limit", "Maximum Active Voice Polyphony", &["16 Voices", "32 Voices", "64 Voices", "128 Voices Poly"], 2, "Maximum simultaneous streaming voice playback threads"))
    );
    descriptors.insert("SamplerMacroStrip".to_string(), DspNodeDescriptor::new("SamplerMacroStrip", "Tactile Multi-Sampler Macro Performance HUD", DspNodeCategory::CompositeSynth, "Simplified macro performance strip for sampler instruments exposing instant sample start, filter cutoff, attack/decay contours, and room ambience")
        .with_param(DspParamSchema::knob("sample_start_offset_ms", "Sample Playback Start Time Offset", 0.0, 1000.0, 0.0, "ms", MacroRole::Tone, "Skips initial transient attack portion of loaded audio sample"))
        .with_param(DspParamSchema::knob("filter_macro_cutoff", "Master Sampler Lowpass Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Global dynamic filter brightness threshold across all instrument zones"))
        .with_param(DspParamSchema::knob("amp_attack_macro_ms", "Global Amplitude Envelope Attack Time", 0.5, 2000.0, 5.0, "ms", MacroRole::Character, "Fade-in duration when triggering notes on the sampler"))
        .with_param(DspParamSchema::knob("amp_decay_macro_ms", "Global Amplitude Envelope Decay Time", 10.0, 5000.0, 650.0, "ms", MacroRole::Space, "Duration for sound to settle from peak attack level to sustain"))
        .with_param(DspParamSchema::knob("room_ambience_mix", "Master Acoustic Room Convolution Mix", 0.0, 100.0, 15.0, "%", MacroRole::Space, "Dry/wet balance of integrated impulse response room reverberation"))
    );
    descriptors.insert("AetherMacroStrip".to_string(), DspNodeDescriptor::new("AetherMacroStrip", "Aether Complex Physical-Spectral Macro Performance HUD", DspNodeCategory::CompositeSynth, "Unified 8-macro performance strip controlling multi-dimensional physical modeling, spectral morphing, and resonant filter engines simultaneously")
        .with_param(DspParamSchema::knob("timbre_morph", "Aether Complex Timbre Morph Factor", 0.0, 1.0, 0.40, "%", MacroRole::Tone, "Sweeps spectral harmonic distribution from organic acoustic to synthetic metallic"))
        .with_param(DspParamSchema::knob("space_density", "Aether Reverb Tail Space Density", 0.0, 1.0, 0.55, "%", MacroRole::Space, "Expands late reverberation diffusion and room cavity reflections"))
        .with_param(DspParamSchema::knob("acoustic_friction", "Physical Bow & Hammer Friction Intensity", 0.0, 1.0, 0.30, "%", MacroRole::Character, "Simulates mechanical resistance and tactile bow grip across virtual strings"))
        .with_param(DspParamSchema::knob("harmonic_dispersion", "Waveguide Allpass Dispersion Stiffening", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Inharmonic modal dispersion stretching higher partials into bell-like spectra"))
        .with_param(DspParamSchema::knob("sub_weight", "Subharmonic Resonance Low-End Foundation", 0.0, 1.0, 0.50, "%", MacroRole::Punch, "Adds saturated sub-bass foundation fundamental an octave below performance pitch"))
    );
    descriptors.insert("VoicePool".to_string(), DspNodeDescriptor::new("VoicePool", "Dynamic Polyphonic Voice Allocation & Stealing Engine", DspNodeCategory::CompositeSynth, "Real-time polyphonic synthesizer voice pool managing note allocation, LRU/oldest/quietest voice stealing algorithms, and release damping")
        .with_param(DspParamSchema::knob("max_voices", "Maximum Polyphonic Voice Limit", 1.0, 64.0, 16.0, "voices", MacroRole::Character, "Total maximum concurrent sounding synthesizer voices"))
        .with_param(DspParamSchema::choice("stealing_policy", "Polyphonic Voice Stealing Strategy", &["Steal Oldest Note", "Steal Quietest Release", "Steal Lowest Frequency", "Strict Hard Limit (Drop)"], 1, "Algorithm selected when note count exceeds allocated voice capacity"))
        .with_param(DspParamSchema::knob("unison_count", "Multi-Voice Unison Stacking Count", 1.0, 8.0, 1.0, "unison", MacroRole::Punch, "Number of concurrent voice instances stacked per played note"))
        .with_param(DspParamSchema::knob("unison_detune_cents", "Unison Detuning Spread", 0.0, 50.0, 12.0, "cents", MacroRole::Tone, "Pitch detuning spread applied across stacked unison voices"))
        .with_param(DspParamSchema::knob("voice_fade_out_ms", "Voice Steal Fast Fade-Out Time", 0.5, 50.0, 5.0, "ms", MacroRole::Space, "Micro-fade crossfade duration eliminating clicks during voice re-allocation"))
    );
    descriptors.insert("AetherMacroView".to_string(), DspNodeDescriptor::new("AetherMacroView", "Aether Subtractive Synth Macro Performance Controller", DspNodeCategory::CompositeSynth, "Consolidated high-level performance macro strip for Aether subtractive synthesizer exposing dual oscillator blend, ladder cutoff, resonance, LFO rate, and master output level")
        .with_param(DspParamSchema::knob("osc_mix", "Oscillator Blend (Saw/Pulse)", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Sawtooth to Pulse oscillator waveform crossfade blend"))
        .with_param(DspParamSchema::log_knob("filter_cutoff", "Moog Ladder Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Moog 4-pole transistor ladder filter cutoff frequency"))
        .with_param(DspParamSchema::knob("filter_res", "Ladder Resonance Peak Q", 0.0, 4.0, 1.2, "Q", MacroRole::Character, "Resonance peak feedback at cutoff frequency"))
        .with_param(DspParamSchema::knob("lfo_speed", "LFO Modulation Rate", 0.05, 40.0, 2.5, "Hz", MacroRole::Space, "Low frequency modulation oscillator rate"))
        .with_param(DspParamSchema::knob("master_level", "Synth Master Output Trim", -36.0, 12.0, 0.0, "dB", MacroRole::Punch, "Overall instrument output level trim"))
    );
    descriptors.insert("GrainWindowProfile".to_string(), DspNodeDescriptor::new("GrainWindowProfile", "Granular Synthesizer Grain Window Envelope Profile", DspNodeCategory::CompositeSynth, "Granular audio micro-windowing contour selector configuring smooth Hann, ultra-pure Blackman-Harris, Gaussian bell curve, Tukey flat-top, and percussive exponential grains")
        .with_param(DspParamSchema::choice("grain_window_shape", "Granular Window Envelope Tapering Contour", &["Hann Window (Zero Sidelobe Leakage)", "Blackman-Harris (-92dB Sidelobe Suppression)", "Gaussian Bell (Minimum Time-Bandwidth)", "Tukey Flat-Top Window (High Energy)", "Exponential Decay (Percussive Pluck)"], 0, "Grain envelope tapering curve profile"))
        .with_param(DspParamSchema::knob("window_symmetry_skew", "Grain Window Left-Right Temporal Skew", -0.8, 0.8, 0.0, "skew", MacroRole::Punch, "Asymmetric shift of peak amplitude within grain window"))
        .with_param(DspParamSchema::knob("window_tukey_cosine_fraction", "Tukey Window Taper Edge Cosine Width", 0.1, 1.0, 0.5, "ratio", MacroRole::Character, "Fraction of window allocated to cosine edge tapers"))
        .with_param(DspParamSchema::knob("gaussian_standard_deviation", "Gaussian Bell Curve Sigma Spread", 0.1, 0.5, 0.25, "sigma", MacroRole::Tone, "Bell curve standard deviation defining window width"))
        .with_param(DspParamSchema::knob("grain_window_crossfade_smoothing", "Micro-Grain Overlap Splicing Smoothing", 0.0, 1.0, 0.2, "ratio", MacroRole::Space, "Inter-grain boundary smoothing to suppress clicks"))
        .with_param(DspParamSchema::toggle("invert_grain_phase_bipolar", "Bipolar Alternate Grain Phase Inversion", false, "Invert phase of alternating grains to widen stereo field"))
    );
    descriptors.insert("VocalChantChoirSynthesizer".to_string(), DspNodeDescriptor::new("VocalChantChoirSynthesizer", "Formant-Synthesized Polyphonic Gregorian Chant & Vocal Choir", DspNodeCategory::CompositeSynth, "Generates rich liturgical chant and vocal ensemble textures using parallel formant cascades, glottal pulse trains, micro-vibrato detuning, and breath turbulence")
        .with_param(DspParamSchema::knob("vowel_formant_morph_position", "Vocal Formant Vowel Morph Position", 0.0, 4.0, 0.0, "vowel", MacroRole::Tone, "Morphs continuously between vowel formants: A (0), E (1), I (2), O (3), U (4)"))
        .with_param(DspParamSchema::knob("choir_unison_voices_count", "Polyphonic Ensemble Voice Density", 1.0, 16.0, 8.0, "voices", MacroRole::Space, "Number of virtual choir singers blended into ensemble texture"))
        .with_param(DspParamSchema::knob("vocalist_pitch_detune_spread_cents", "Chamber Choir Intonation Detune", 1.0, 50.0, 14.0, "ct", MacroRole::Space, "Intonation variance between individual choir members"))
        .with_param(DspParamSchema::knob("glottal_pulse_open_quotient", "Glottal Excitation Open Quotient", 0.2, 0.8, 0.55, "%", MacroRole::Punch, "Open quotient of glottal airflow pulse setting voice brightness and buzz"))
        .with_param(DspParamSchema::knob("aspiration_breath_noise_mix_db", "Pharyngeal Aspiration Breathiness Level", -40.0, -6.0, -22.0, "dB", MacroRole::Character, "Air turbulence and throat breathiness added to harmonic formant tones"))
        .with_param(DspParamSchema::toggle("enable_liturgical_cathedral_acoustic", "Gothic Cathedral Stone Acoustic Early Echo", true, "Renders stone cathedral acoustic pre-delay and diffuse early reflections"))
    );
    descriptors.insert("HarmonicResonanceVocoder".to_string(), DspNodeDescriptor::new("HarmonicResonanceVocoder", "32-Channel Harmonic Resonance Spectral Vocoder", DspNodeCategory::CompositeSynth, "High-resolution 32-band vocoder pairing carrier synthesizer waveforms with modulator microphone audio, complete with formant transposer, formant blur, and unvoiced sibilance bypass")
        .with_param(DspParamSchema::knob("carrier_modulator_blend_pct", "Vocoded Spectral Cross-Synthesis Mix", 0.0, 100.0, 100.0, "%", MacroRole::Tone, "Balance between raw input modulator audio and vocoded spectral output"))
        .with_param(DspParamSchema::knob("formant_transposition_semitones", "Filter Bank Formant Shifter", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Shifts filter bank analysis bands up or down to alter vocal character"))
        .with_param(DspParamSchema::knob("filter_band_resonance_q", "Analysis Channel Bandpass Q Factor", 2.0, 40.0, 12.0, "Q", MacroRole::Punch, "Resonance sharpness of the 32 bandpass filter analysis channels"))
        .with_param(DspParamSchema::knob("envelope_follower_decay_ms", "Spectral Envelope Follower Decay", 2.0, 250.0, 22.0, "ms", MacroRole::Space, "Decay time of spectral envelope followers per vocoder band"))
        .with_param(DspParamSchema::knob("unvoiced_sibilance_bleed_db", "Consonant Sibilance Direct Injection", -40.0, 0.0, -16.0, "dB", MacroRole::Character, "Injects unvoiced high-frequency consonants (T, S, P) directly into output"))
        .with_param(DspParamSchema::toggle("enable_high_band_noise_carrier", "High-Band Noise Articulation Carrier", true, "Supplements carrier with white noise in high bands for crisp consonant articulation"))
    );
    descriptors.insert("ArticulatoryVocalTractSynthesizer".to_string(), DspNodeDescriptor::new("ArticulatoryVocalTractSynthesizer", "Articulatory Acoustic Vocal Tract Synthesizer", DspNodeCategory::CompositeSynth, "Physical Kelly-Lochbaum vocal tract model parameterized by biomechanical tongue position, jaw opening, lip protrusion, velum opening, and glottal flow waveforms")
        .with_param(DspParamSchema::knob("glottal_excitation_frequency_hz", "Vocal Cord Fundamental Pitch", 40.0, 1000.0, 130.0, "Hz", MacroRole::Tone, "Fundamental oscillation frequency of simulated vocal fold vibration"))
        .with_param(DspParamSchema::knob("tongue_body_position_cm", "Tongue Body Front/Back Position", 1.0, 16.0, 8.5, "cm", MacroRole::Character, "Anatomical front-to-back coordinate position of tongue body arch"))
        .with_param(DspParamSchema::knob("jaw_opening_aperture_height_mm", "Jaw Opening Vertical Aperture", 1.0, 25.0, 12.0, "mm", MacroRole::Tone, "Vertical aperture dimension of mandibular jaw opening"))
        .with_param(DspParamSchema::knob("lip_rounding_protrusion_mm", "Lip Rounding Acoustic Protrusion", 0.0, 20.0, 4.0, "mm", MacroRole::Space, "Acoustic tube extension caused by forward lip rounding and protrusion"))
        .with_param(DspParamSchema::knob("glottal_chink_aspiration_noise", "Glottal Chink Breathiness Hiss", 0.0, 1.0, 0.08, "%", MacroRole::Punch, "Turbulent aspiration air noise escaping incomplete vocal cord closure"))
        .with_param(DspParamSchema::toggle("enable_nasal_cavity_coupling", "Velum Soft Palate Nasal Coupling", false, "Opens velopharyngeal port coupling nasal resonance cavity"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "AetherSynth" => Box::new(
            GenericDspNodeUi::new("AetherSynth", "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("osc_mix", "Saw / Pulse Mix", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "Moog Filter Cutoff", 20.0, 20000.0, 1200.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("filter_res", "Ladder Resonance", 0.1, 4.0, 1.0, "Q", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_log("lfo_speed", "PWM LFO Speed", 0.1, 20.0, 2.0, "Hz", (0, 230, 118)))
        ),
        "CyberpunkSubSynth" => Box::new(
            GenericDspNodeUi::new("CyberpunkSubSynth", "Heavy Sub-bass Synth with Asymmetric Wavefolding", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_log("sub_freq", "Sub Oscillator Freq", 20.0, 200.0, 55.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("drive_gain", "Wavefolder Drive", 1.0, 10.0, 4.0, "x", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "SVF Filter Cutoff", 40.0, 2000.0, 600.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("pitch_wobble", "LFO Pitch Wobble", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
        ),
        "AtmosphericPadSynth" => Box::new(
            GenericDspNodeUi::new("AtmosphericPadSynth", "Detuned Ambient Pad with SVF Filter & Space Reverb", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("saw_tri_mix", "Saw / Detuned Tri Mix", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "SVF Filter Cutoff", 100.0, 5000.0, 800.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("reverb_mix", "Space Reverb Level", 0.0, 1.0, 0.4, "%", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("delay_mix", "Stereo Delay Level", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
        ),
        "GlitchPercussionSynth" => Box::new(
            GenericDspNodeUi::new("GlitchPercussionSynth", "Noise Burst & Granular Stutter Percussion Engine", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_log("pitch_freq", "Pitch Drop Center", 30.0, 800.0, 120.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("bitcrush_depth", "Bitcrusher Depth", 1, 16, 8, "bits", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_log("comb_freq", "Comb Resonator Freq", 50.0, 2000.0, 240.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("stutter_rate", "Buffer Stutter Speed", 1.0, 64.0, 16.0, "div", (255, 171, 0)))
        ),
        "GlitchAetherMachine" => Box::new(
            GenericDspNodeUi::new("GlitchAetherMachine", "Flagship Chopper & Tape-Stop Glitch Synthesizer", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("drive", "Tube Drive Saturation", 1.0, 20.0, 3.0, "x", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "Moog Ladder Cutoff", 50.0, 10000.0, 2500.0, "Hz", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("chop_rate", "Glitch Chopper Rate", 1.0, 32.0, 8.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_bool("tape_stop", "Tape Stop Effect Active", false, (255, 171, 0)))
        ),
        "FmMatrixSynthesizer" => Box::new(
            GenericDspNodeUi::new("FmMatrixSynthesizer", "6-Operator DX-Style FM Matrix Synthesizer", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("mod_index", "FM Modulation Index", 0.0, 10.0, 3.5, "idx", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("algorithm", "FM Routing Algorithm", vec!["Alg 1 (Serial 6-Op)".into(), "Alg 5 (Parallel Pairs)".into(), "Alg 16 (3-Stack Hybrid)".into(), "Alg 32 (All Parallel)".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("feedback", "Op 6 Self-Feedback", 0.0, 1.0, 0.45, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("master_level", "Master Output Level", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
        ),
        "FusionResonanceSynth" => Box::new(
            GenericDspNodeUi::new("FusionResonanceSynth", "Tokamak Fusion Magnetic Plasma Resonance Synth", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("magnetic_confinement_t", "Magnetic Field Flux (B)", 1.0, 15.0, 5.5, "Tesla", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("plasma_temp_kev", "Core Plasma Temp", 1.0, 50.0, 15.0, "keV", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_log("cyclotron_freq_hz", "Ion Cyclotron Pitch", 100.0, 10000.0, 880.0, "Hz", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("fusion_gain", "Acoustic Output Level", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
        ),
        "TonewheelOrganNode" => Box::new(
            GenericDspNodeUi::new("TonewheelOrganNode", "Vintage 91-Wheel Synchronous Magnetic Tonewheel Organ Console", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_enum("drawbar_preset", "Harmonic Drawbar Registration", vec!["888000000 (Jazz Jimmy)".into(), "888888888 (Gospel Full)".into(), "800000008 (Rock Lead)".into(), "008800000 (Warm Mellowness)".into(), "803800000 (Blues Flute)".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("key_click_gain", "Mechanical Key Click Attack", 0.0, 1.0, 0.45, "gain", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("leakage_crosstalk", "Adjacent Tonewheel Crosstalk", 0.0, 0.3, 0.05, "leak", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("vibrato_chorus_mode", "Scanner Vibrato/Chorus", vec!["Off".into(), "V-1".into(), "C-1".into(), "V-2".into(), "C-2".into(), "V-3".into(), "C-3 (Deep Chorus)".into()], 6, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("percussion_decay_ms", "Percussion Harmonic Decay", 100.0, 1000.0, 350.0, "ms", (16, 185, 129)))
                .with_param(DspParamDescriptor::new_linear("preamp_drive", "12AU7 Tube Preamp Drive", 0.0, 1.0, 0.35, "drive", (236, 72, 153)))
        ),
        "MolecularVibrationResonatorNode" => Box::new(
            GenericDspNodeUi::new("MolecularVibrationResonatorNode", "Zero-Gravity Fluid Molecular Vibrational Quantum Resonance Synthesizer", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_enum("molecule_profile", "Target Molecule Species", vec!["Water H2O".into(), "Carbon Dioxide CO2".into(), "Methane CH4".into(), "Benzene C6H6".into(), "Fullerene C60".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("base_frequency_hz", "Audible Scaled Fundamental", 30.0, 2000.0, 160.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("anharmonicity_factor", "Morse Potential Anharmonicity", 0.0, 0.5, 0.08, "x_e", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("thermal_excitation_temp_k", "Kinetic Temperature", 50.0, 2000.0, 298.0, "K", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("collisional_broadening", "Collisional Band Broadening", 0.1, 10.0, 1.8, "Q", (16, 185, 129)))
        ),
        "PlasmaArcSynthesizerNode" => Box::new(
            GenericDspNodeUi::new("PlasmaArcSynthesizerNode", "Modulated High-Voltage Singing Plasma Arc Spark Acoustic Generator", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("arc_current_ma", "Discharge Arc Current", 10.0, 500.0, 120.0, "mA", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spark_gap_mm", "Electrode Spark Gap", 1.0, 50.0, 15.0, "mm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("thermal_expansion_gain", "Thermal Shock Expansion Gain", 0.1, 5.0, 1.5, "gain", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_log("carrier_switching_khz", "PWM Ultrasonic Carrier", 20.0, 100.0, 48.0, "kHz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ozone_saturator_depth", "Plasma Non-Linear Saturation", 0.0, 1.0, 0.40, "%", (16, 185, 129)))
        ),
        "FusionResonanceSynthNode" => Box::new(
            GenericDspNodeUi::new("FusionResonanceSynthNode", "Magnetic Confinement Fusion Tokamak Turbulence Plasma Sound Synthesizer", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("magnetic_field_tesla", "Toroidal Field Strength B", 1.0, 12.0, 5.3, "T", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("plasma_density_10e19", "Core Plasma Density", 1.0, 50.0, 15.0, "m⁻³", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("alfven_velocity_km_s", "Alfvén Wave Speed", 500.0, 15000.0, 3200.0, "km/s", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("turbulence_chaos_amount", "Drift Wave Micro-Turbulence", 0.0, 1.0, 0.45, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("geodesic_mode_mix", "Geodesic Acoustic Mode (GAM) Mix", 0.0, 1.0, 0.35, "%", (16, 185, 129)))
        ),
        "MultiSampleBank" => Box::new(
            GenericDspNodeUi::new("MultiSampleBank", "Multi-Velocity Multi-Zone Sample Bank HUD", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_enum("velocity_curve", "Velocity Response Dynamic Curve", vec!["Linear Normal".into(), "Soft Touch (Exp)".into(), "Hard Touch (Log)".into(), "Fixed 127 Max".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("round_robin_mode", "Round-Robin Sample Alternation Mode", vec!["Sequential Cycle".into(), "True Random".into(), "Smart Non-Repeating".into(), "Velocity Swapped".into()], 2, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("sample_interpolation", "Sub-Sample Pitch Shift Interpolation", vec!["Hermite Cubic 4-Point".into(), "Sinc 16-Tap Bandlimited".into(), "Linear 2-Point".into(), "Nearest Neighbor".into()], 0, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("release_trigger_level_db", "Key Release Sample Trigger Gain", -48.0, 0.0, -12.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_enum("voice_polyphony_limit", "Maximum Active Voice Polyphony", vec!["16 Voices".into(), "32 Voices".into(), "64 Voices".into(), "128 Voices Poly".into()], 2, (34, 197, 94)))
        ),
        "SamplerMacroStrip" => Box::new(
            GenericDspNodeUi::new("SamplerMacroStrip", "Tactile Multi-Sampler Macro Performance HUD", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("sample_start_offset_ms", "Sample Playback Start Time Offset", 0.0, 1000.0, 0.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("filter_macro_cutoff", "Master Sampler Lowpass Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("amp_attack_macro_ms", "Global Amplitude Envelope Attack Time", 0.5, 2000.0, 5.0, "ms", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("amp_decay_macro_ms", "Global Amplitude Envelope Decay Time", 10.0, 5000.0, 650.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("room_ambience_mix", "Master Acoustic Room Convolution Mix", 0.0, 100.0, 15.0, "%", (34, 197, 94)))
        ),
        "AetherMacroStrip" => Box::new(
            GenericDspNodeUi::new("AetherMacroStrip", "Aether Complex Physical-Spectral Macro Performance HUD", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("timbre_morph", "Aether Complex Timbre Morph Factor", 0.0, 1.0, 0.40, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("space_density", "Aether Reverb Tail Space Density", 0.0, 1.0, 0.55, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("acoustic_friction", "Physical Bow & Hammer Friction Intensity", 0.0, 1.0, 0.30, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("harmonic_dispersion", "Waveguide Allpass Dispersion Stiffening", 0.0, 1.0, 0.65, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_weight", "Subharmonic Resonance Low-End Foundation", 0.0, 1.0, 0.50, "%", (34, 197, 94)))
        ),
        "VoicePool" => Box::new(
            GenericDspNodeUi::new("VoicePool", "Dynamic Polyphonic Voice Allocation & Stealing Engine HUD", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("max_voices", "Maximum Polyphonic Voice Limit", 1.0, 64.0, 16.0, "voices", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("stealing_policy", "Polyphonic Voice Stealing Strategy", vec!["Steal Oldest Note".into(), "Steal Quietest Release".into(), "Steal Lowest Frequency".into(), "Strict Hard Limit (Drop)".into()], 1, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("unison_count", "Multi-Voice Unison Stacking Count", 1.0, 8.0, 1.0, "unison", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("unison_detune_cents", "Unison Detuning Spread", 0.0, 50.0, 12.0, "cents", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("voice_fade_out_ms", "Voice Steal Fast Fade-Out Time", 0.5, 50.0, 5.0, "ms", (34, 197, 94)))
        ),
        "AetherMacroView" => Box::new(
            GenericDspNodeUi::new("AetherMacroView", "Aether Subtractive Synth Macro Performance Controller", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("osc_mix", "Oscillator Blend (Saw/Pulse)", 0.0, 1.0, 0.5, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("filter_cutoff", "Moog Ladder Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_res", "Ladder Resonance Peak Q", 0.0, 4.0, 1.2, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lfo_speed", "LFO Modulation Rate", 0.05, 40.0, 2.5, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("master_level", "Synth Master Output Trim", -36.0, 12.0, 0.0, "dB", (34, 197, 94)))
        ),
        "GrainWindowProfile" => Box::new(
            GenericDspNodeUi::new("GrainWindowProfile", "Granular Synthesizer Grain Window Envelope Profile", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("grain_window_shape", "Granular Window Envelope Tapering Contour", 0.0, 4.0, 0.0, "shape", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("window_symmetry_skew", "Grain Window Left-Right Temporal Skew", -0.8, 0.8, 0.0, "skew", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("window_tukey_cosine_fraction", "Tukey Window Taper Edge Cosine Width", 0.1, 1.0, 0.5, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("gaussian_standard_deviation", "Gaussian Bell Curve Sigma Spread", 0.1, 0.5, 0.25, "sig", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("grain_window_crossfade_smoothing", "Micro-Grain Overlap Splicing Smoothing", 0.0, 1.0, 0.2, "ratio", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("invert_grain_phase_bipolar", "Bipolar Alternate Grain Phase Inversion", false, (14, 165, 233)))
        ),
        "VocalChantChoirSynthesizer" => Box::new(
            GenericDspNodeUi::new("VocalChantChoirSynthesizer", "Formant-Synthesized Polyphonic Gregorian Chant & Vocal Choir", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("vowel_formant_morph_position", "Vocal Formant Vowel Morph Position", 0.0, 4.0, 0.0, "vowel", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("choir_unison_voices_count", "Polyphonic Ensemble Voice Density", 1.0, 16.0, 8.0, "voices", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("vocalist_pitch_detune_spread_cents", "Chamber Choir Intonation Detune", 1.0, 50.0, 14.0, "ct", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("glottal_pulse_open_quotient", "Glottal Excitation Open Quotient", 0.2, 0.8, 0.55, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("aspiration_breath_noise_mix_db", "Pharyngeal Aspiration Breathiness Level", -40.0, -6.0, -22.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_liturgical_cathedral_acoustic", "Gothic Cathedral Stone Acoustic Early Echo", true, (14, 165, 233)))
        ),
        "HarmonicResonanceVocoder" => Box::new(
            GenericDspNodeUi::new("HarmonicResonanceVocoder", "32-Channel Harmonic Resonance Spectral Vocoder", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("carrier_modulator_blend_pct", "Vocoded Spectral Cross-Synthesis Mix", 0.0, 100.0, 100.0, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_transposition_semitones", "Filter Bank Formant Shifter", -24.0, 24.0, 0.0, "st", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("filter_band_resonance_q", "Analysis Channel Bandpass Q Factor", 2.0, 40.0, 12.0, "Q", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("envelope_follower_decay_ms", "Spectral Envelope Follower Decay", 2.0, 250.0, 22.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("unvoiced_sibilance_bleed_db", "Consonant Sibilance Direct Injection", -40.0, 0.0, -16.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_high_band_noise_carrier", "High-Band Noise Articulation Carrier", true, (14, 165, 233)))
        ),
        "ArticulatoryVocalTractSynthesizer" => Box::new(
            GenericDspNodeUi::new("ArticulatoryVocalTractSynthesizer", "Articulatory Acoustic Vocal Tract Synthesizer", DspNodeCategory::CompositeSynth)
                .with_param(DspParamDescriptor::new_linear("glottal_excitation_frequency_hz", "Vocal Cord Fundamental Pitch", 40.0, 1000.0, 130.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("tongue_body_position_cm", "Tongue Body Front/Back Position", 1.0, 16.0, 8.5, "cm", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("jaw_opening_aperture_height_mm", "Jaw Opening Vertical Aperture", 1.0, 25.0, 12.0, "mm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("lip_rounding_protrusion_mm", "Lip Rounding Acoustic Protrusion", 0.0, 20.0, 4.0, "mm", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("glottal_chink_aspiration_noise", "Glottal Chink Breathiness Hiss", 0.0, 1.0, 0.08, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_nasal_cavity_coupling", "Velum Soft Palate Nasal Coupling", false, (14, 165, 233)))
        ),
        _ => return None,
    };
    Some(node)
}
