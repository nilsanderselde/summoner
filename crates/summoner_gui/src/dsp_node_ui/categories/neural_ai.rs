// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Declarative DSP node descriptors and UI builders for `NeuralAi`.

use std::collections::HashMap;

use super::super::types::*;
use super::super::{DspNodeUi, GenericDspNodeUi};

pub(in crate::dsp_node_ui) fn register_descriptors(
    descriptors: &mut HashMap<String, DspNodeDescriptor>,
) {
    descriptors.insert("NeuralWavetable".to_string(), DspNodeDescriptor::new("NeuralWavetable", "Latent AI Neural Wavetable Synthesizer", DspNodeCategory::NeuralAi, "Latent AI Neural Wavetable Synthesizer")
        .with_param(DspParamSchema::knob("latent_x", "Latent Timbre X", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre X parameter"))
        .with_param(DspParamSchema::knob("latent_y", "Latent Timbre Y", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Y parameter"))
        .with_param(DspParamSchema::knob("spectral_tilt", "Spectral Tilt", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Spectral Tilt parameter"))
        .with_param(DspParamSchema::knob("morph_speed", "Morph Slew Rate", 0.1, 10.0, 1.0, "Hz", MacroRole::Tone, "Morph Slew Rate parameter"))
    );
    descriptors.insert("SingingSynthesisNode".to_string(), DspNodeDescriptor::new("SingingSynthesisNode", "Differentiable Neural Phoneme Singing Synthesizer", DspNodeCategory::NeuralAi, "Differentiable Neural Phoneme Singing Synthesizer")
        .with_param(DspParamSchema::log_knob("f0_pitch", "Vocal Fundamental F0", 50.0, 1000.0, 220.0, "Hz", MacroRole::Tone, "Vocal Fundamental F0 logarithmic parameter"))
        .with_param(DspParamSchema::knob("formant_scale", "Formant Scale", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Formant Scale parameter"))
        .with_param(DspParamSchema::knob("breathiness", "Glottal Breathiness", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Glottal Breathiness parameter"))
        .with_param(DspParamSchema::knob("vibrato_depth", "Vocal Vibrato Depth", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Vocal Vibrato Depth parameter"))
        .with_param(DspParamSchema::choice("phoneme", "Current Phoneme", &["/a/", "/e/", "/i/", "/o/", "/u/", "/m/", "/n/"], 0, "Current Phoneme mode selector"))
    );
    descriptors.insert("NeuralAudioRepairNode".to_string(), DspNodeDescriptor::new("NeuralAudioRepairNode", "AI Spectral Audio Inpainting & De-Clicking Engine", DspNodeCategory::NeuralAi, "AI Spectral Audio Inpainting & De-Clicking Engine")
        .with_param(DspParamSchema::knob("noise_reduction", "Spectral De-Noise Depth", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Spectral De-Noise Depth parameter"))
        .with_param(DspParamSchema::knob("inpaint_strength", "Inpaint Reconstruction", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Inpaint Reconstruction parameter"))
        .with_param(DspParamSchema::toggle("transient_protect", "Protect Fast Transients", true, "Protect Fast Transients toggle switch"))
        .with_param(DspParamSchema::knob("blend", "Dry / Wet Repair Blend", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Dry / Wet Repair Blend parameter"))
    );
    descriptors.insert("NeuralTransientShaperNode".to_string(), DspNodeDescriptor::new("NeuralTransientShaperNode", "Deep Neural Network Transient & Sustain Sculptor", DspNodeCategory::NeuralAi, "Deep Neural Network Transient & Sustain Sculptor")
        .with_param(DspParamSchema::knob("attack_boost", "Transient Attack Gain", -12.0, 12.0, 3.0, "dB", MacroRole::Tone, "Transient Attack Gain parameter"))
        .with_param(DspParamSchema::knob("sustain_tail", "Sustain Tail Gain", -12.0, 12.0, -1.5, "dB", MacroRole::Tone, "Sustain Tail Gain parameter"))
        .with_param(DspParamSchema::knob("neural_sensitivity", "AI Detection Sensitivity", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "AI Detection Sensitivity parameter"))
        .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer", 0.0, 20.0, 5.0, "ms", MacroRole::Tone, "Lookahead Buffer parameter"))
    );
    descriptors.insert("NeuralBassGeneratorNode".to_string(), DspNodeDescriptor::new("NeuralBassGeneratorNode", "Subharmonic Low-End Neural Bass Synthesizer", DspNodeCategory::NeuralAi, "Subharmonic Low-End Neural Bass Synthesizer")
        .with_param(DspParamSchema::knob("sub_octave_gain", "Subharmonic Level", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Subharmonic Level parameter"))
        .with_param(DspParamSchema::knob("harmonic_density", "Generated Harmonics", 1.0, 8.0, 3.0, "bands", MacroRole::Character, "Generated Harmonics integer control"))
        .with_param(DspParamSchema::choice("saturation_color", "Saturation Flavor", &["Warm Tube", "Solid State", "Tape Flux"], 0, "Saturation Flavor mode selector"))
        .with_param(DspParamSchema::toggle("mono_lows", "Mono Bass Summing (<120Hz)", true, "Mono Bass Summing (<120Hz) toggle switch"))
    );
    descriptors.insert("NeuralHarmonicExciterNode".to_string(), DspNodeDescriptor::new("NeuralHarmonicExciterNode", "AI Spectral Sheen & High-Frequency Air Polish", DspNodeCategory::NeuralAi, "AI Spectral Sheen & High-Frequency Air Polish")
        .with_param(DspParamSchema::knob("air_sheen", "Ultra-High Air Presence", 0.0, 12.0, 4.0, "dB", MacroRole::Tone, "Ultra-High Air Presence parameter"))
        .with_param(DspParamSchema::knob("warmth_body", "Low-Mid Harmonic Warmth", 0.0, 12.0, 2.0, "dB", MacroRole::Tone, "Low-Mid Harmonic Warmth parameter"))
        .with_param(DspParamSchema::choice("ai_profile", "Exciter Neural Profile", &["Modern Polish", "Vintage Tube", "Transformer Color", "Silk Top"], 0, "Exciter Neural Profile mode selector"))
        .with_param(DspParamSchema::knob("wet_mix", "Harmonic Blend Level", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Harmonic Blend Level parameter"))
    );
    descriptors.insert("BciEegDecoderNode".to_string(), DspNodeDescriptor::new("BciEegDecoderNode", "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder", DspNodeCategory::NeuralAi, "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder")
        .with_param(DspParamSchema::knob("sampling_rate_hz", "EEG Electrode Sample Rate", 128.0, 1024.0, 256.0, "Hz", MacroRole::Tone, "EEG Electrode Sample Rate parameter"))
        .with_param(DspParamSchema::knob("alpha_band_gain", "Alpha Power Band (8-12Hz)", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Alpha Power Band (8-12Hz) parameter"))
        .with_param(DspParamSchema::knob("theta_band_gain", "Theta Power Band (4-8Hz)", 0.0, 2.0, 0.8, "x", MacroRole::Tone, "Theta Power Band (4-8Hz) parameter"))
        .with_param(DspParamSchema::toggle("artifact_rejection_filter", "Ocular Blink Artifact Filter", true, "Ocular Blink Artifact Filter toggle switch"))
    );
    descriptors.insert("NeuroAffectiveEmotionalStateAnalyzer".to_string(), DspNodeDescriptor::new("NeuroAffectiveEmotionalStateAnalyzer", "Russell Circumplex Valence & Arousal Emotion Analyzer", DspNodeCategory::NeuralAi, "Russell Circumplex Valence & Arousal Emotion Analyzer")
        .with_param(DspParamSchema::knob("valence_score", "Musical Valence (Pleasantness)", -1.0, 1.0, 0.55, "score", MacroRole::Tone, "Musical Valence (Pleasantness) parameter"))
        .with_param(DspParamSchema::knob("arousal_intensity", "Musical Arousal (Energy)", 0.0, 1.0, 0.65, "score", MacroRole::Tone, "Musical Arousal (Energy) parameter"))
        .with_param(DspParamSchema::knob("emotional_smoothing_window", "Affective Slew Smoothing", 0.1, 10.0, 2.0, "s", MacroRole::Tone, "Affective Slew Smoothing parameter"))
        .with_param(DspParamSchema::knob("harmony_feedback_gain", "Emotional Harmony Influence", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Emotional Harmony Influence parameter"))
    );
    descriptors.insert("NeuralImpulseResponseSynthesizer".to_string(), DspNodeDescriptor::new("NeuralImpulseResponseSynthesizer", "Latent Space Acoustic Impulse Response Generator", DspNodeCategory::NeuralAi, "Latent Space Acoustic Impulse Response Generator")
        .with_param(DspParamSchema::knob("latent_room_dimension_x", "Latent Room Geometry X", -3.0, 3.0, 0.5, "", MacroRole::Tone, "Latent Room Geometry X parameter"))
        .with_param(DspParamSchema::knob("latent_absorption_y", "Latent Wall Material Y", -3.0, 3.0, -0.2, "", MacroRole::Tone, "Latent Wall Material Y parameter"))
        .with_param(DspParamSchema::knob("early_reflection_decay", "Early Diffusion Slew", 0.1, 5.0, 1.5, "s", MacroRole::Tone, "Early Diffusion Slew parameter"))
        .with_param(DspParamSchema::knob("synthesis_quality", "Neural Inference Latent Depth", 1.0, 4.0, 3.0, "steps", MacroRole::Character, "Neural Inference Latent Depth integer control"))
    );
    descriptors.insert("SubcorticalBrainstemPitchTracker".to_string(), DspNodeDescriptor::new("SubcorticalBrainstemPitchTracker", "Auditory Nerve Frequency-Following Response Pitch Tracker", DspNodeCategory::NeuralAi, "Auditory Nerve Frequency-Following Response Pitch Tracker")
        .with_param(DspParamSchema::knob("ffr_sensitivity", "Brainstem FFR Sensitivity", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Brainstem FFR Sensitivity parameter"))
        .with_param(DspParamSchema::knob("cochlear_filterbank_channels", "Cochlear Gammatone Channels", 16.0, 128.0, 64.0, "ch", MacroRole::Character, "Cochlear Gammatone Channels integer control"))
        .with_param(DspParamSchema::log_knob("fundamental_f0_estimate_hz", "Detected Pitch F0", 30.0, 2000.0, 220.0, "Hz", MacroRole::Tone, "Detected Pitch F0 logarithmic parameter"))
        .with_param(DspParamSchema::knob("tracking_slew_ms", "Pitch Tracking Response", 1.0, 100.0, 15.0, "ms", MacroRole::Tone, "Pitch Tracking Response parameter"))
    );
    descriptors.insert("MentalImageryPatternClassifier".to_string(), DspNodeDescriptor::new("MentalImageryPatternClassifier", "Motor/Auditory Imagery Neural Intention Classifier", DspNodeCategory::NeuralAi, "Motor/Auditory Imagery Neural Intention Classifier")
        .with_param(DspParamSchema::knob("classifier_confidence_threshold", "Trigger Confidence Floor", 0.5, 0.99, 0.85, "%", MacroRole::Tone, "Trigger Confidence Floor parameter"))
        .with_param(DspParamSchema::knob("pattern_bank_index", "Intention Pattern Slot", 0.0, 15.0, 0.0, "slot", MacroRole::Character, "Intention Pattern Slot integer control"))
        .with_param(DspParamSchema::knob("trigger_event_latency_ms", "Pattern Recognition Latency", 5.0, 200.0, 35.0, "ms", MacroRole::Tone, "Pattern Recognition Latency parameter"))
        .with_param(DspParamSchema::knob("adaptation_rate", "Online Neural Plasticity Slew", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Online Neural Plasticity Slew parameter"))
    );
    descriptors.insert("BiometricHrvTempoSync".to_string(), DspNodeDescriptor::new("BiometricHrvTempoSync", "Photoplethysmography Heart Rate Variability Tempo Sync", DspNodeCategory::NeuralAi, "Photoplethysmography Heart Rate Variability Tempo Sync")
        .with_param(DspParamSchema::knob("target_heart_coherence", "Target HRV Coherence", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Target HRV Coherence parameter"))
        .with_param(DspParamSchema::knob("bpm_sync_multiplier", "BPM Metric Ratio", 0.5, 4.0, 1.0, "x", MacroRole::Tone, "BPM Metric Ratio parameter"))
        .with_param(DspParamSchema::knob("hrv_smoothing_interval_s", "Heart Rate Rolling Average", 5.0, 60.0, 20.0, "s", MacroRole::Tone, "Heart Rate Rolling Average parameter"))
        .with_param(DspParamSchema::knob("rhythm_entrainment_depth", "Tempo Entrainment Depth", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tempo Entrainment Depth parameter"))
    );
    descriptors.insert("NeuroCognitiveFatigueDetector".to_string(), DspNodeDescriptor::new("NeuroCognitiveFatigueDetector", "Listening Fatigue & Auditory Habituation Detector", DspNodeCategory::NeuralAi, "Listening Fatigue & Auditory Habituation Detector")
        .with_param(DspParamSchema::knob("listening_duration_minutes", "Session Exposure Time", 0.0, 480.0, 65.0, "min", MacroRole::Tone, "Session Exposure Time parameter"))
        .with_param(DspParamSchema::knob("spectral_brightness_fatigue_index", "Harshness Fatigue Metric", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Harshness Fatigue Metric parameter"))
        .with_param(DspParamSchema::knob("auto_soft_filter_db", "Fatigue Relief Soft High Cut", -6.0, 0.0, 0.0, "dB", MacroRole::Tone, "Fatigue Relief Soft High Cut parameter"))
        .with_param(DspParamSchema::toggle("habituation_warning", "Break Reminder Notification", true, "Break Reminder Notification toggle switch"))
    );
    descriptors.insert("NeuroAestheticHarmonyScorer".to_string(), DspNodeDescriptor::new("NeuroAestheticHarmonyScorer", "Computational Neuro-Aesthetic Musical Harmony Scorer", DspNodeCategory::NeuralAi, "Computational Neuro-Aesthetic Musical Harmony Scorer")
        .with_param(DspParamSchema::knob("harmony_concordance_weight", "Concordance Sensation Bias", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Concordance Sensation Bias parameter"))
        .with_param(DspParamSchema::knob("voice_leading_fluency", "Voice Leading Smoothness", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Voice Leading Smoothness parameter"))
        .with_param(DspParamSchema::knob("tension_resolution_score", "Tension Cycle Resolution", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Tension Cycle Resolution parameter"))
        .with_param(DspParamSchema::knob("scale_fit_percent", "Tonal Scale Alignment", 0.0, 100.0, 94.0, "%", MacroRole::Tone, "Tonal Scale Alignment parameter"))
    );
    descriptors.insert("SubsensoryTactileHapticTransducer".to_string(), DspNodeDescriptor::new("SubsensoryTactileHapticTransducer", "Vibrotactile Sub-Bass Haptic Transducer Driver", DspNodeCategory::NeuralAi, "Vibrotactile Sub-Bass Haptic Transducer Driver")
        .with_param(DspParamSchema::log_knob("haptic_frequency_hz", "Transducer Resonance Pitch", 15.0, 120.0, 40.0, "Hz", MacroRole::Tone, "Transducer Resonance Pitch logarithmic parameter"))
        .with_param(DspParamSchema::knob("transducer_intensity", "Haptic Shaker Force", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Haptic Shaker Force parameter"))
        .with_param(DspParamSchema::choice("subharmonic_synthesis_oct", "Sub-Harmonic Generation", &["None (Direct)", "-1 Octave Sub", "-2 Octave Infrasound"], 1, "Sub-Harmonic Generation mode selector"))
        .with_param(DspParamSchema::knob("rumble_envelope_decay", "Haptic Rumble Decay", 0.05, 2.0, 0.4, "s", MacroRole::Tone, "Haptic Rumble Decay parameter"))
    );
    descriptors.insert("NeuralChoirFormantNode".to_string(), DspNodeDescriptor::new("NeuralChoirFormantNode", "Multi-Voice Neural Formant Choir & Vowel Morph Engine", DspNodeCategory::NeuralAi, "Multi-Voice Neural Formant Choir & Vowel Morph Engine")
        .with_param(DspParamSchema::knob("voice_count", "Ensemble Choir Voices", 4.0, 32.0, 16.0, "voices", MacroRole::Character, "Ensemble Choir Voices integer control"))
        .with_param(DspParamSchema::knob("vowel_morph_coordinate", "Vowel Space Morph (A-E-I-O-U)", 0.0, 4.0, 1.5, "vowel", MacroRole::Tone, "Vowel Space Morph (A-E-I-O-U) parameter"))
        .with_param(DspParamSchema::knob("formant_spread_cents", "Formant Detuning Width", 0.0, 50.0, 15.0, "cents", MacroRole::Tone, "Formant Detuning Width parameter"))
        .with_param(DspParamSchema::knob("choir_detune_cents", "Choral Micro-Pitch Detune", 0.0, 40.0, 8.0, "cents", MacroRole::Tone, "Choral Micro-Pitch Detune parameter"))
    );
    descriptors.insert("NeuralDereverbNode".to_string(), DspNodeDescriptor::new("NeuralDereverbNode", "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb", DspNodeCategory::NeuralAi, "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb")
        .with_param(DspParamSchema::knob("dereverb_intensity", "De-Reverberation Depth", 0.0, 100.0, 65.0, "%", MacroRole::Tone, "De-Reverberation Depth parameter"))
        .with_param(DspParamSchema::knob("room_size_estimate_m", "Estimated Chamber Size", 2.0, 50.0, 12.0, "m", MacroRole::Tone, "Estimated Chamber Size parameter"))
        .with_param(DspParamSchema::knob("early_reflection_suppress_db", "Early Reflection Attenuation", -24.0, 0.0, -10.0, "dB", MacroRole::Tone, "Early Reflection Attenuation parameter"))
        .with_param(DspParamSchema::knob("tail_dry_boost_db", "Direct Signal Presence Boost", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Direct Signal Presence Boost parameter"))
    );
    descriptors.insert("NeuralInpaintNode".to_string(), DspNodeDescriptor::new("NeuralInpaintNode", "Neural Audio Inpainting & Dropout Reconstruction Engine", DspNodeCategory::NeuralAi, "Neural Audio Inpainting & Dropout Reconstruction Engine")
        .with_param(DspParamSchema::knob("dropout_threshold_ms", "Dropout Mask Detection Window", 1.0, 100.0, 20.0, "ms", MacroRole::Tone, "Dropout Mask Detection Window parameter"))
        .with_param(DspParamSchema::knob("reconstruction_depth", "AI Inpainting Strength", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "AI Inpainting Strength parameter"))
        .with_param(DspParamSchema::knob("context_window_ms", "Surrounding Context Frame", 50.0, 1000.0, 250.0, "ms", MacroRole::Character, "Surrounding Context Frame integer control"))
        .with_param(DspParamSchema::toggle("spectral_continuity_protect", "Preserve Spectral Phase", true, "Preserve Spectral Phase toggle switch"))
    );
    descriptors.insert("NeuralPhonemeNode".to_string(), DspNodeDescriptor::new("NeuralPhonemeNode", "International Phonetic Alphabet Differentiable Phoneme Synth", DspNodeCategory::NeuralAi, "International Phonetic Alphabet Differentiable Phoneme Synth")
        .with_param(DspParamSchema::knob("ipa_symbol_index", "IPA Phoneme Symbol Index", 0.0, 64.0, 12.0, "ipa", MacroRole::Character, "IPA Phoneme Symbol Index integer control"))
        .with_param(DspParamSchema::knob("nasalization_index", "Velum Nasal Coupling", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Velum Nasal Coupling parameter"))
        .with_param(DspParamSchema::knob("voicing_glottal_mix", "Voiced Glottal Mix", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Voiced Glottal Mix parameter"))
        .with_param(DspParamSchema::knob("tongue_arch_position", "Tongue Constriction Place", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tongue Constriction Place parameter"))
    );
    descriptors.insert("NeuralRadianceNode".to_string(), DspNodeDescriptor::new("NeuralRadianceNode", "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field", DspNodeCategory::NeuralAi, "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field")
        .with_param(DspParamSchema::knob("listener_pos_xyz", "Virtual Listener X Position", -10.0, 10.0, 0.0, "m", MacroRole::Tone, "Virtual Listener X Position parameter"))
        .with_param(DspParamSchema::knob("emitter_pos_xyz", "Virtual Emitter Y Position", -10.0, 10.0, 2.5, "m", MacroRole::Tone, "Virtual Emitter Y Position parameter"))
        .with_param(DspParamSchema::knob("nerf_rendering_samples", "Ray Sample Marching Steps", 32.0, 256.0, 128.0, "rays", MacroRole::Character, "Ray Sample Marching Steps integer control"))
        .with_param(DspParamSchema::knob("spatial_interpolation_slew", "Acoustic Field Slew Rate", 0.01, 1.0, 0.15, "s", MacroRole::Tone, "Acoustic Field Slew Rate parameter"))
    );
    descriptors.insert("NeuralSpeechToSingingNode".to_string(), DspNodeDescriptor::new("NeuralSpeechToSingingNode", "Speech-to-Singing AI Prosody & Intonation Transformer", DspNodeCategory::NeuralAi, "Speech-to-Singing AI Prosody & Intonation Transformer")
        .with_param(DspParamSchema::toggle("target_pitch_quantize", "Snap Speech to Scale Notes", true, "Snap Speech to Scale Notes toggle switch"))
        .with_param(DspParamSchema::knob("vibrato_infusion_depth", "Singing Vibrato Infusion", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Singing Vibrato Infusion parameter"))
        .with_param(DspParamSchema::knob("note_hold_extension_ms", "Vowel Duration Extension", 50.0, 1000.0, 350.0, "ms", MacroRole::Tone, "Vowel Duration Extension parameter"))
        .with_param(DspParamSchema::choice("singing_style_preset", "AI Singing Style", &["Bel Canto Opera", "Pop Vibrato Smooth", "R&B Micro-Riff", "Choir Polyphony"], 1, "AI Singing Style mode selector"))
    );
    descriptors.insert("NeuralTimbreMorphNode".to_string(), DspNodeDescriptor::new("NeuralTimbreMorphNode", "Non-Linear Latent Timbre Vector Interpolator", DspNodeCategory::NeuralAi, "Non-Linear Latent Timbre Vector Interpolator")
        .with_param(DspParamSchema::knob("timbre_vector_x", "Latent Timbre Axis X (Warm/Bright)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis X (Warm/Bright) parameter"))
        .with_param(DspParamSchema::knob("timbre_vector_y", "Latent Timbre Axis Y (Wood/Metal)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis Y (Wood/Metal) parameter"))
        .with_param(DspParamSchema::knob("timbre_vector_z", "Latent Timbre Axis Z (Acoustic/Synth)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis Z (Acoustic/Synth) parameter"))
        .with_param(DspParamSchema::knob("morph_trajectory_speed_hz", "LFO Timbre Orbit Speed", 0.05, 10.0, 0.5, "Hz", MacroRole::Tone, "LFO Timbre Orbit Speed parameter"))
    );
    descriptors.insert("NeuralTimbreNode".to_string(), DspNodeDescriptor::new("NeuralTimbreNode", "Latent Timbre Space Feature Extractor & Profiler", DspNodeCategory::NeuralAi, "Latent Timbre Space Feature Extractor & Profiler")
        .with_param(DspParamSchema::knob("spectral_centroid_weight", "Spectral Brightness Weight", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Spectral Brightness Weight parameter"))
        .with_param(DspParamSchema::knob("flux_roughness_weight", "Spectral Roughness Weight", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Spectral Roughness Weight parameter"))
        .with_param(DspParamSchema::knob("harmonic_entropy_weight", "Harmonic Order vs Noise", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Harmonic Order vs Noise parameter"))
        .with_param(DspParamSchema::toggle("timbre_snapshot_trigger", "Capture Timbre Vector", false, "Capture Timbre Vector toggle switch"))
    );
    descriptors.insert("NeuralVocalStylizerNode".to_string(), DspNodeDescriptor::new("NeuralVocalStylizerNode", "Vocal Timbre Style Transfer & Character Transformer", DspNodeCategory::NeuralAi, "Vocal Timbre Style Transfer & Character Transformer")
        .with_param(DspParamSchema::knob("source_vocal_id", "Source Vocal Track ID", 0.0, 32.0, 1.0, "trk", MacroRole::Character, "Source Vocal Track ID integer control"))
        .with_param(DspParamSchema::choice("target_style_preset", "AI Timbre Target Profile", &["Vintage Ribbon Mic Warmth", "Modern Hyper-Pop Crisp", "Analog Tube Intimate", "Classic 80s Broadcast"], 0, "AI Timbre Target Profile mode selector"))
        .with_param(DspParamSchema::knob("formant_character_scale", "Formant Character Intensity", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Formant Character Intensity parameter"))
        .with_param(DspParamSchema::knob("style_transfer_blend", "Style Transfer Dry/Wet Blend", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Style Transfer Dry/Wet Blend parameter"))
    );
    descriptors.insert("NeuralVocoderMorphNode".to_string(), DspNodeDescriptor::new("NeuralVocoderMorphNode", "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher", DspNodeCategory::NeuralAi, "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher")
        .with_param(DspParamSchema::choice("vocoder_architecture", "Neural Vocoder Backbone", &["BigVGAN High-Fidelity", "HiFi-GAN V1", "Vocos Ultra-Fast", "WaveGlow Multi-Scale"], 0, "Neural Vocoder Backbone mode selector"))
        .with_param(DspParamSchema::knob("latent_dimension_scale", "Latent Manifold Scale", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Latent Manifold Scale parameter"))
        .with_param(DspParamSchema::knob("periodic_component_mix", "Harmonic Periodic Component", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Harmonic Periodic Component parameter"))
        .with_param(DspParamSchema::knob("aperiodic_noise_ratio", "Aperiodic Breath Noise Level", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Aperiodic Breath Noise Level parameter"))
    );
    descriptors.insert("NeuralQuantumAnnealerTopologicalSort".to_string(), DspNodeDescriptor::new("NeuralQuantumAnnealerTopologicalSort", "Simulated Quantum Annealing Audio Graph Optimizer", DspNodeCategory::NeuralAi, "Simulated Quantum Annealing Audio Graph Optimizer")
        .with_param(DspParamSchema::knob("annealing_temperature", "Annealing Temperature (T)", 0.01, 10.0, 1.0, "T", MacroRole::Tone, "Annealing Temperature (T) parameter"))
        .with_param(DspParamSchema::knob("transverse_field_gamma", "Transverse Quantum Field", 0.0, 5.0, 1.5, "gamma", MacroRole::Tone, "Transverse Quantum Field parameter"))
        .with_param(DspParamSchema::knob("graph_latency_minimize_weight", "Latency Minimization Weight", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Latency Minimization Weight parameter"))
        .with_param(DspParamSchema::toggle("optimization_step_trigger", "Trigger Topological Sort", false, "Trigger Topological Sort toggle switch"))
    );
    descriptors.insert("DemucsV4Separator".to_string(), DspNodeDescriptor::new("DemucsV4Separator", "Demucs v4 Hybrid Neural Stem Separator", DspNodeCategory::NeuralAi, "Deep Neural Network 6-Stem Audio Source Separation Transformer")
        .with_param(DspParamSchema::choice("stems_mask", "Stem Mode", &["6-Stem Full", "4-Stem Standard", "Vocal Isolated", "Instrumental"], 0, "Stem separation target mode selector"))
        .with_param(DspParamSchema::knob("overlap", "FFT Overlap", 0.1, 0.9, 0.25, "%", MacroRole::Space, "Transformer chunk overlap ratio parameter"))
        .with_param(DspParamSchema::knob("sensitivity", "Separation Sensitivity", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Stem isolation sensitivity gain parameter"))
        .with_param(DspParamSchema::knob("residual_bleed", "Residual Bleed", -48.0, 0.0, -24.0, "dB", MacroRole::Character, "Cross-stem residual attenuation parameter"))
        .with_param(DspParamSchema::knob("spectral_power", "Mask Exponent", 1.0, 4.0, 2.0, "p", MacroRole::Punch, "Wiener-EM mask sharpness exponent parameter"))
        .with_param(DspParamSchema::toggle("gpu_accel", "GPU Acceleration", true, "Hardware AVX2/CUDA inference toggle"))
    );
    descriptors.insert("NeuralRoomAcousticMatcher".to_string(), DspNodeDescriptor::new("NeuralRoomAcousticMatcher", "Neural Room Acoustic Matcher", DspNodeCategory::NeuralAi, "Impulse Response Convolution Profiler & Architectural Space Matcher")
        .with_param(DspParamSchema::knob("rt60_decay", "RT60 Decay Time", 0.1, 8.0, 1.2, "s", MacroRole::Space, "Estimated reverberation time parameter"))
        .with_param(DspParamSchema::knob("brightness", "Brightness Ratio", 0.2, 3.0, 1.15, "x", MacroRole::Tone, "High frequency absorption compensation parameter"))
        .with_param(DspParamSchema::log_knob("room_volume", "Room Volume", 10.0, 5000.0, 120.0, "m³", MacroRole::Space, "Architectural hall volumetric air estimation logarithmic parameter"))
        .with_param(DspParamSchema::knob("early_reflections", "Early Reflections", 0.0, 1.0, 0.65, "%", MacroRole::Space, "Specular discrete echo wall ratio parameter"))
        .with_param(DspParamSchema::knob("tail_density", "Late Tail Density", 0.1, 1.0, 0.80, "%", MacroRole::Character, "Stochastic modal diffusion build rate parameter"))
        .with_param(DspParamSchema::knob("stereo_spread", "Stereo Field Width", 0.0, 1.0, 0.90, "%", MacroRole::Space, "Binaural cross-correlation spatial separation parameter"))
    );
    descriptors.insert("NamAmpNode".to_string(), DspNodeDescriptor::new("NamAmpNode", "Neural Amp Modeler (NAM) WaveNet Engine", DspNodeCategory::NeuralAi, "Deep neural network guitar/bass amplifier and cabinet modeling using dilated convolutional WaveNet architecture")
        .with_param(DspParamSchema::knob("drive", "Pre-Amp Drive", 0.0, 5.0, 1.5, "x", MacroRole::Punch, "Neural network input excitation drive gain parameter"))
        .with_param(DspParamSchema::knob("output_gain", "Master Volume", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Linear post-amp master output gain parameter"))
        .with_param(DspParamSchema::knob("gate_threshold", "Noise Gate", -90.0, 0.0, -60.0, "dB", MacroRole::Character, "Integrated pre-neural noise gate threshold parameter"))
        .with_param(DspParamSchema::choice("model_profile", "Amp Profile", &["Clean Tube", "British Crunch", "Modern High-Gain", "Bass Slap"], 1, "Loaded neural amp model profile selector"))
    );
    descriptors.insert("RnnoiseNode".to_string(), DspNodeDescriptor::new("RnnoiseNode", "RNNoise Recurrent Neural Denoising Node", DspNodeCategory::NeuralAi, "Recurrent neural network deep-learning background noise suppression and speech enhancement filter")
        .with_param(DspParamSchema::knob("suppression_db", "Noise Suppression", -48.0, 0.0, -12.0, "dB", MacroRole::Tone, "Maximum neural recurrent noise attenuation floor parameter"))
        .with_param(DspParamSchema::knob("speech_threshold", "Voice Detection Sensitivity", 0.0, 1.0, 0.50, "%", MacroRole::Character, "Recurrent voice activity detection classification threshold parameter"))
        .with_param(DspParamSchema::knob("band_smoothing", "Band Gain Smoothing", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Temporal smoothing across 22 critical psychoacoustic bands parameter"))
    );
    descriptors.insert("DdspTimbreTransferNode".to_string(), DspNodeDescriptor::new("DdspTimbreTransferNode", "DDSP Differentiable Timbre Transfer", DspNodeCategory::NeuralAi, "Differentiable Digital Signal Processing harmonic additive + filtered noise AI timbre resynthesis")
        .with_param(DspParamSchema::knob("harmonic_count", "Harmonic Count", 1.0, 64.0, 32.0, "harmonics", MacroRole::Tone, "Number of additive sinusoidal oscillator tracks parameter"))
        .with_param(DspParamSchema::knob("noise_level", "Filtered Noise Residual", 0.0, 1.0, 0.05, "%", MacroRole::Character, "Stochastic noise component gain parameter"))
        .with_param(DspParamSchema::knob("timbre_morph", "Timbre Morph", 0.0, 1.0, 0.50, "%", MacroRole::Space, "Interpolation between source acoustics and target neural timbre parameter"))
        .with_param(DspParamSchema::knob("confidence_thresh", "Pitch Tracking Confidence", 0.1, 0.9, 0.40, "%", MacroRole::Tone, "Minimum fundamental tracking confidence for harmonic synthesis parameter"))
    );
    descriptors.insert("VocalHarmonyGeneratorNode".to_string(), DspNodeDescriptor::new("VocalHarmonyGeneratorNode", "AI Polyphonic Vocal Harmonizer", DspNodeCategory::NeuralAi, "Intelligent multi-voice vocal harmony generator with scale-aware semitone interval pitch shifting")
        .with_param(DspParamSchema::knob("harmony_voices", "Harmony Voices", 1.0, 4.0, 3.0, "voices", MacroRole::Tone, "Number of concurrent generated harmony voices parameter"))
        .with_param(DspParamSchema::knob("harmony_spread", "Stereo Spread", 0.0, 1.0, 0.60, "%", MacroRole::Space, "Stereo panning separation between harmony voices parameter"))
        .with_param(DspParamSchema::choice("interval_preset", "Interval Preset", &["Major 3rd + 5th", "Minor 3rd + Octave", "Sus4 + 7th", "Power 5ths"], 0, "Musical chord harmony interval pattern selector"))
        .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.50, "%", MacroRole::Character, "Direct vocal versus generated harmony balance parameter"))
        .with_param(DspParamSchema::toggle("formant_preserve", "Preserve Natural Formants", true, "Spectral envelope formant correction toggle"))
    );
    descriptors.insert("NeuralSuperResolutionNode".to_string(), DspNodeDescriptor::new("NeuralSuperResolutionNode", "Neural Audio Super-Resolution & Bandwidth Extension", DspNodeCategory::NeuralAi, "AI neural upsampling and high-frequency spectral extension reconstructs lost air and sparkle above Nyquist")
        .with_param(DspParamSchema::knob("hf_boost", "High-Frequency Boost", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Reconstructed ultrasonic energy gain parameter"))
        .with_param(DspParamSchema::knob("spectral_continuity", "Spectral Continuity", 0.0, 1.0, 0.85, "%", MacroRole::Character, "Phase alignment between baseband and generated harmonics parameter"))
        .with_param(DspParamSchema::knob("harmonic_reconstruction", "Harmonic Density", 0.0, 1.0, 0.70, "%", MacroRole::Punch, "AI synthesized upper-register harmonic richness parameter"))
    );
    descriptors.insert("AiAutoGainNode".to_string(), DspNodeDescriptor::new("AiAutoGainNode", "AI Intelligent Auto-Gain Leveler", DspNodeCategory::NeuralAi, "Real-time ITU-R BS.1770 LUFS tracking intelligent gain riding prevents clipping and matches target loudness")
        .with_param(DspParamSchema::knob("target_lufs", "Target Loudness", -36.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Target integrated loudness anchor parameter"))
        .with_param(DspParamSchema::knob("max_boost_db", "Max Boost Limit", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Upward leveling headroom cap parameter"))
        .with_param(DspParamSchema::knob("max_cut_db", "Max Attenuation Limit", -24.0, 0.0, -12.0, "dB", MacroRole::Tone, "Downward clamping attenuation floor parameter"))
        .with_param(DspParamSchema::knob("response_speed", "Reaction Time", 0.1, 10.0, 2.0, "s", MacroRole::Character, "RMS detector temporal tracking window parameter"))
    );
    descriptors.insert("CrepePitchTracker".to_string(), DspNodeDescriptor::new("CrepePitchTracker", "CREPE Deep Neural Polyphonic Pitch & Frequency Estimator", DspNodeCategory::NeuralAi, "Deep convolutional neural network pitch tracker operating directly on raw audio waveforms with sub-cent fundamental frequency accuracy")
        .with_param(DspParamSchema::knob("confidence_thresh", "Confidence Cutoff", 0.1, 0.99, 0.75, "%", MacroRole::Character, "Neural tracking activation probability threshold parameter"))
        .with_param(DspParamSchema::knob("slew_smoothing", "Temporal Slew Smoothing", 0.0, 1.0, 0.30, "%", MacroRole::Tone, "Pitch trajectory inter-frame smoothing filter parameter"))
        .with_param(DspParamSchema::choice("model_size", "Neural Model Capacity", &["Tiny (Low CPU)", "Small", "Medium (Balanced)", "Full (Studio Grade)"], 2, "CREPE model complexity selector"))
        .with_param(DspParamSchema::knob("cent_tolerance", "Microtonal Quantize", 1.0, 50.0, 5.0, "cents", MacroRole::Tone, "Microtonal pitch quantization tolerance window parameter"))
    );
    descriptors.insert("NeuralAudioStyleTransferPreviewRenderer".to_string(), DspNodeDescriptor::new("NeuralAudioStyleTransferPreviewRenderer", "Neural Audio Style Transfer & Timbre Morphing Preview", DspNodeCategory::NeuralAi, "High-speed latent diffusion audio style transfer network imposing reference timbre and texture onto source dry performances in real time")
        .with_param(DspParamSchema::knob("content_weight", "Content Preservation", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Original performance preservation weight parameter"))
        .with_param(DspParamSchema::knob("style_weight", "Timbre Style Imprint", 0.0, 1.0, 0.60, "%", MacroRole::Character, "Target timbre imprint intensity parameter"))
        .with_param(DspParamSchema::knob("latent_steps", "Diffusion Steps", 5.0, 50.0, 20.0, "steps", MacroRole::Punch, "Latent space reconstruction steps integer parameter"))
        .with_param(DspParamSchema::choice("fft_window", "Spectral Window", &["512 samples", "1024 samples", "2048 samples", "4096 samples"], 1, "FFT spectral analysis frame size selector"))
    );
    descriptors.insert("NeuroAffectiveAnalyzer".to_string(), DspNodeDescriptor::new("NeuroAffectiveAnalyzer", "Affective Emotional State & Listener Valence/Arousal Analyzer", DspNodeCategory::NeuralAi, "Bio-acoustic affective state estimation engine mapping harmonic tension, spectral centroid flux, and rhythm entropy into circumplex valence/arousal space")
        .with_param(DspParamSchema::knob("valence_target", "Target Valence (Pleasure)", -1.0, 1.0, 0.0, "", MacroRole::Tone, "Emotional valence positive/negative bias parameter"))
        .with_param(DspParamSchema::knob("arousal_target", "Target Arousal (Energy)", 0.0, 1.0, 0.50, "%", MacroRole::Punch, "Emotional arousal stimulation energy parameter"))
        .with_param(DspParamSchema::knob("integration_time_s", "State Integration Window", 0.5, 10.0, 2.0, "s", MacroRole::Space, "Affective temporal smoothing time parameter"))
        .with_param(DspParamSchema::knob("confidence_cutoff", "Classifier Confidence", 0.5, 0.99, 0.85, "%", MacroRole::Character, "Neural emotion prediction confidence threshold parameter"))
    );
    descriptors.insert("MentalImageryClassifier".to_string(), DspNodeDescriptor::new("MentalImageryClassifier", "EEG Mental Imagery & Motor Cortex Pattern Classifier", DspNodeCategory::NeuralAi, "Brain-Computer Interface (BCI) motor imagery pattern classifier decoding auditory/visual mental focus into discrete and continuous DAW control signals")
        .with_param(DspParamSchema::knob("electrode_channels", "Active EEG Electrodes", 4.0, 64.0, 16.0, "ch", MacroRole::Character, "Number of active spatial EEG electrode channels parameter"))
        .with_param(DspParamSchema::choice("spatial_filter", "Spatial Decomposition", &["Common Spatial Pattern (CSP)", "Independent Component Analysis (ICA)", "Surface Laplacian"], 0, "EEG spatial filtering algorithm selector"))
        .with_param(DspParamSchema::knob("bandpass_low_hz", "Mu/Alpha Low Cutoff", 4.0, 12.0, 8.0, "Hz", MacroRole::Tone, "Sensory motor rhythm low cutoff frequency parameter"))
        .with_param(DspParamSchema::knob("bandpass_high_hz", "Beta/Gamma High Cutoff", 20.0, 50.0, 30.0, "Hz", MacroRole::Tone, "Active cognitive band high cutoff frequency parameter"))
    );
    descriptors.insert("NeuroAestheticScorer".to_string(), DspNodeDescriptor::new("NeuroAestheticScorer", "Neuro-Aesthetic Psychoacoustic Harmony & Groove Scorer", DspNodeCategory::NeuralAi, "Evaluates acoustic recordings against neuro-computational aesthetic models of Plomp-Levelt roughness, rhythmic syncopation groove, and spectral entropy")
        .with_param(DspParamSchema::knob("consonance_weight", "Harmonic Consonance", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Weight of critical band harmonic consonance parameter"))
        .with_param(DspParamSchema::knob("entropy_weight", "Spectral Novelty Entropy", 0.0, 1.0, 0.50, "%", MacroRole::Character, "Weight of spectral informational complexity parameter"))
        .with_param(DspParamSchema::knob("groove_weight", "Syncopation Groove", 0.0, 1.0, 0.60, "%", MacroRole::Punch, "Weight of micro-timing rhythmic syncopation parameter"))
        .with_param(DspParamSchema::knob("eval_rate_hz", "Score Evaluation Rate", 0.1, 5.0, 1.0, "Hz", MacroRole::Space, "Neuro-aesthetic score refresh frequency parameter"))
    );
    descriptors.insert("AiMixBalanceAnalyzer".to_string(), DspNodeDescriptor::new("AiMixBalanceAnalyzer", "AI Multi-Track Frequency Masking Analyzer", DspNodeCategory::NeuralAi, "Neural and psychoacoustic multi-track frequency collision and masking analyzer providing intelligent corrective EQ recommendations")
        .with_param(DspParamSchema::knob("sensitivity", "Masking Detection Threshold", 0.0, 1.0, 0.50, "%", MacroRole::Tone, "Psychoacoustic masking sensitivity threshold parameter"))
        .with_param(DspParamSchema::knob("spectral_bands", "Analysis Band Count", 4.0, 32.0, 16.0, "bands", MacroRole::Character, "Critical band filter bank resolution parameter"))
        .with_param(DspParamSchema::knob("smoothing_sec", "Temporal Smoothing Window", 0.05, 2.0, 0.30, "s", MacroRole::Space, "Time-constant smoothing integration parameter"))
        .with_param(DspParamSchema::knob("auto_recommend_cut", "Auto-Cut Recommended Depth", 0.0, 12.0, 3.0, "dB", MacroRole::Punch, "Recommended notch attenuation depth parameter"))
    );
    descriptors.insert("AiSongStructureDetector".to_string(), DspNodeDescriptor::new("AiSongStructureDetector", "AI Song Structure & Arrangement Classifier", DspNodeCategory::NeuralAi, "Neural arrangement analyzer segmenting audio into Intro, Verse, Chorus, Bridge, and Outro sections with automated marker placement")
        .with_param(DspParamSchema::knob("min_section_bars", "Minimum Section Length", 4.0, 32.0, 8.0, "bars", MacroRole::Space, "Minimum bars per song section parameter"))
        .with_param(DspParamSchema::knob("energy_contrast", "Chorus Energy Contrast", 0.0, 1.0, 0.65, "%", MacroRole::Punch, "Dynamic drop/chorus contrast threshold parameter"))
        .with_param(DspParamSchema::knob("tempo_stability", "Beat Grid Confidence", 0.50, 1.0, 0.95, "%", MacroRole::Tone, "BPM and downbeat detection confidence parameter"))
        .with_param(DspParamSchema::toggle("auto_marker_placement", "Auto Place Markers", true, "Automatic timeline marker generation toggle"))
    );
    descriptors.insert("AiPolyphonicChordExtractor".to_string(), DspNodeDescriptor::new("AiPolyphonicChordExtractor", "Real-Time AI Polyphonic Chord Extractor", DspNodeCategory::NeuralAi, "Deep learning polyphonic audio chord extraction transcribing harmonic progressions into MIDI chords and lead sheets")
        .with_param(DspParamSchema::knob("confidence_threshold", "Chord Detection Confidence", 0.20, 0.99, 0.70, "%", MacroRole::Tone, "Minimum probability for chord classification parameter"))
        .with_param(DspParamSchema::knob("bass_octave_weight", "Bass Root Note Weight", 0.0, 2.0, 1.20, "x", MacroRole::Punch, "Low-end chromagram weighting factor parameter"))
        .with_param(DspParamSchema::choice("tempo_quantize", "Quantize Grid", &["Quarter Note (1/4)", "Eighth Note (1/8)", "Sixteenth Note (1/16)", "Free / Unquantized"], 0, "Sequencer grid timing quantization"))
        .with_param(DspParamSchema::knob("velocity_scaling", "Extracted Velocity Scale", 0.1, 2.0, 1.0, "x", MacroRole::Character, "Extracted MIDI note velocity scale factor parameter"))
    );
    descriptors.insert("NeuralIrSynthesizer".to_string(), DspNodeDescriptor::new("NeuralIrSynthesizer", "Deep Neural Parametric Room Impulse Generator", DspNodeCategory::NeuralAi, "Deep learning neural impulse response generator synthesizing photorealistic room impulse responses (RIR) from physical architectural geometry and wall boundary materials")
        .with_param(DspParamSchema::log_knob("room_volume_m3", "Target Room Acoustic Volume", 10.0, 50000.0, 600.0, "m³", MacroRole::Space, "Architectural room volume metric parameter"))
        .with_param(DspParamSchema::knob("rt60_decay_s", "Reverberation Time RT60", 0.1, 10.0, 1.8, "s", MacroRole::Space, "Statistical RT60 reverberation duration parameter"))
        .with_param(DspParamSchema::knob("absorption_coeff", "Wall Material Absorption", 0.01, 0.95, 0.22, "", MacroRole::Tone, "Boundary material acoustic absorption coefficient parameter"))
        .with_param(DspParamSchema::knob("latent_seed", "Neural Diffusion Latent Seed", 0.0, 9999.0, 42.0, "", MacroRole::Character, "Generative neural room diffusion seed parameter"))
    );
    descriptors.insert("NeuralMidiTranscriber".to_string(), DspNodeDescriptor::new("NeuralMidiTranscriber", "CRNN Polyphonic Audio-to-MIDI Neural Transcriber", DspNodeCategory::NeuralAi, "Convolutional Recurrent Neural Network (CRNN) polyphonic pitch, note onset, duration, and velocity audio-to-MIDI live transcription engine")
        .with_param(DspParamSchema::knob("onset_threshold", "Neural Onset Detection Sensitivity", 0.05, 0.95, 0.50, "%", MacroRole::Punch, "Transient onset probability cutoff parameter"))
        .with_param(DspParamSchema::knob("pitch_confidence", "Minimum Pitch Confidence", 0.20, 0.99, 0.70, "%", MacroRole::Tone, "Posterior pitch class confidence threshold parameter"))
        .with_param(DspParamSchema::knob("min_note_duration_ms", "Minimum Note Duration", 20.0, 500.0, 60.0, "ms", MacroRole::Character, "Shortest permitted MIDI note duration parameter"))
        .with_param(DspParamSchema::choice("velocity_tracking", "MIDI Velocity Estimation Mode", &["Dynamic Envelope", "Fixed 100", "Neural Velocity Estimator"], 2, "Note velocity calculation method"))
    );
    descriptors.insert("DrumTranscriptor".to_string(), DspNodeDescriptor::new("DrumTranscriptor", "Neural Multi-Class Drum & Percussion Transcriber", DspNodeCategory::NeuralAi, "Multi-class spectral neural network drum transcriber extracting separate Kick, Snare, Hi-hat, and Percussion MIDI tracks from mixed drum loops or live acoustic kits")
        .with_param(DspParamSchema::knob("kick_sensitivity", "Kick Drum Classification Bias", 0.0, 1.0, 0.60, "%", MacroRole::Punch, "Low frequency transient classification bias parameter"))
        .with_param(DspParamSchema::knob("snare_sensitivity", "Snare Drum Classification Bias", 0.0, 1.0, 0.60, "%", MacroRole::Punch, "Broadband snap classification bias parameter"))
        .with_param(DspParamSchema::knob("hihat_sensitivity", "Hi-Hat Classification Bias", 0.0, 1.0, 0.50, "%", MacroRole::Tone, "High frequency metallic classification bias parameter"))
        .with_param(DspParamSchema::choice("quantize_resolution", "MIDI Transcription Grid", &["Free", "1/16 Note", "1/16 Triplet", "1/32 Note"], 1, "Rhythmic grid snap alignment"))
    );
    descriptors.insert("AiChordGenerator".to_string(), DspNodeDescriptor::new("AiChordGenerator", "AI Voice-Leading & Harmonic Progression Solver", DspNodeCategory::NeuralAi, "Algorithmic and neural voice-leading harmony solver generating musically coherent chord sequences, root motion, and smooth soprano/bass voice distributions")
        .with_param(DspParamSchema::knob("harmonic_complexity", "Progression Harmonic Complexity", 1.0, 5.0, 2.0, "lvl", MacroRole::Tone, "Extended chord complexity level parameter"))
        .with_param(DspParamSchema::choice("cadence_type", "Target Cadence Resolution", &["Authentic (V-I)", "Plagal (IV-I)", "Deceptive (V-vi)", "Jazz ii-V-I", "Modal Dorian"], 0, "Musical cadence structure selector"))
        .with_param(DspParamSchema::choice("root_tonic", "Root Key Tonic", &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"], 0, "Musical key root center"))
        .with_param(DspParamSchema::knob("voice_leading_smoothness", "Voice Leading Minimal Movement", 0.0, 1.0, 0.85, "%", MacroRole::Space, "Minimal voice movement optimization weight parameter"))
    );
    descriptors.insert("AiMixAssistant".to_string(), DspNodeDescriptor::new("AiMixAssistant", "Autonomous Headroom & Frequency Collision Assistant", DspNodeCategory::NeuralAi, "Autonomous neural mix assistant evaluating multi-bus headroom, frequency overlap collisions, stereo correlation phase issues, and loudness standards")
        .with_param(DspParamSchema::knob("target_lufs", "Integrated Target Loudness", -24.0, -8.0, -14.0, "LUFS", MacroRole::Tone, "Broadcasting/streaming LUFS loudness target parameter"))
        .with_param(DspParamSchema::knob("crest_factor_target", "Dynamic Crest Factor Target", 6.0, 20.0, 12.0, "dB", MacroRole::Punch, "Peak-to-RMS dynamic headroom target parameter"))
        .with_param(DspParamSchema::knob("masking_mitigation", "Inter-Track Masking Suppression", 0.0, 1.0, 0.40, "%", MacroRole::Tone, "Automatic masking EQ carve intensity parameter"))
        .with_param(DspParamSchema::knob("stereo_correlation_target", "Minimum Stereo Phase Target", 0.20, 1.0, 0.80, "", MacroRole::Space, "Stereo correlation meter threshold parameter"))
    );
    descriptors.insert("BrainstemPitchTracker".to_string(), DspNodeDescriptor::new("BrainstemPitchTracker", "Sub-Cortical Auditory Brainstem Pitch Tracking Simulator", DspNodeCategory::NeuralAi, "Physiologically accurate auditory brainstem frequency following response (FFR) pitch detector with cycle zero-crossing extraction")
        .with_param(DspParamSchema::log_knob("tracked_freq_hz", "Tracked Pitch Reference", 20.0, 2000.0, 440.0, "Hz", MacroRole::Tone, "Detected fundamental pitch from auditory brainstem model parameter"))
        .with_param(DspParamSchema::knob("window_size", "Analysis Window Size", 256.0, 4096.0, 1024.0, "smp", MacroRole::Space, "Zero-crossing integration sample window parameter"))
        .with_param(DspParamSchema::knob("sensitivity", "Neuronal Threshold", 0.01, 1.0, 0.20, "%", MacroRole::Punch, "Minimum signal amplitude required to trigger synaptic firing parameter"))
        .with_param(DspParamSchema::choice("auditory_nerve_profile", "Synaptic Response Profile", &["Auditory Nerve Fiber A", "High-Spontaneous Rate", "Low-Spontaneous Rate", "Cochlear Nucleus Bushy Cell"], 0, "Brainstem neural model preset parameter"))
    );
    descriptors.insert("NeuralQuantumAnnealer".to_string(), DspNodeDescriptor::new("NeuralQuantumAnnealer", "Neural Quantum Annealer Topological Sort", DspNodeCategory::NeuralAi, "Simulated quantum annealing optimization engine computing lowest-latency topological sorts and energy-efficient DSP routing")
        .with_param(DspParamSchema::knob("annealing_temp", "Quantum Annealing Temperature", 0.01, 10.0, 1.0, "K", MacroRole::Tone, "Thermal energy in quantum transverse Ising Hamiltonian parameter"))
        .with_param(DspParamSchema::knob("transverse_field", "Transverse Magnetic Field Gamma", 0.1, 5.0, 2.0, "B", MacroRole::Character, "Quantum tunneling rate across DSP routing energy barriers parameter"))
        .with_param(DspParamSchema::knob("quantum_fluctuation", "Vacuum Fluctuations", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Stochastic tunneling perturbations escaping local graph minima parameter"))
        .with_param(DspParamSchema::knob("cooling_rate", "Annealing Schedule Cooling Rate", 0.80, 0.999, 0.985, "", MacroRole::Punch, "Geometric temperature reduction per iteration parameter"))
    );
    descriptors.insert("AudioTagger".to_string(), DspNodeDescriptor::new("AudioTagger", "Deep Neural Audio Tagging & Timbre Classifier", DspNodeCategory::NeuralAi, "MobileNet-derived convolutional neural network classifying audio genre, instrumentation, mood, and key signature")
        .with_param(DspParamSchema::knob("classification_threshold", "Acceptance Confidence", 0.1, 0.99, 0.65, "prob", MacroRole::Tone, "Prediction probability threshold for tag activation parameter"))
        .with_param(DspParamSchema::knob("analysis_window_sec", "Analysis Window Length", 0.5, 10.0, 3.0, "s", MacroRole::Character, "Temporal STFT frame buffer size for inference parameter"))
        .with_param(DspParamSchema::knob("top_k_candidates", "Top-K Candidates", 1.0, 10.0, 5.0, "tags", MacroRole::Punch, "Maximum number of multi-label classification tags returned parameter"))
        .with_param(DspParamSchema::toggle("auto_categorize_asset", "Auto-Tag New Samples", true, "Automatically tag incoming samples in asset browser parameter"))
        .with_param(DspParamSchema::toggle("realtime_inference", "Background Inference", false, "Continuous low-priority background inference thread parameter"))
    );
    descriptors.insert("NeuralWavetableInterpolator".to_string(), DspNodeDescriptor::new("NeuralWavetableInterpolator", "Latent-Space Neural Wavetable Continuous Interpolator", DspNodeCategory::NeuralAi, "Variational autoencoder providing smooth, artifact-free non-linear morphing between arbitrary single-cycle wavetables")
        .with_param(DspParamSchema::knob("latent_dim_x", "Latent Axis X (Bright)", -3.0, 3.0, 0.0, "z1", MacroRole::Tone, "First principal latent space coordinate for timbre morphing parameter"))
        .with_param(DspParamSchema::knob("latent_dim_y", "Latent Axis Y (Contour)", -3.0, 3.0, 0.0, "z2", MacroRole::Space, "Second principal latent space coordinate for harmonic contour parameter"))
        .with_param(DspParamSchema::knob("spectral_flatness_bias", "Spectral Flatness Bias", 0.0, 1.0, 0.3, "bias", MacroRole::Character, "Latent projection bias towards bright or mellow waveforms parameter"))
        .with_param(DspParamSchema::knob("decoder_temperature", "Decoder Temperature", 0.1, 2.0, 1.0, "temp", MacroRole::Punch, "Stochastic decoder temperature introducing microscopic harmonic variation parameter"))
        .with_param(DspParamSchema::toggle("bandlimited_resynthesis", "Bandlimited Resynthesis", true, "Strict anti-aliased Dirichlet kernel waveform resynthesis parameter"))
    );
    descriptors.insert("NamWaveNetEngine".to_string(), DspNodeDescriptor::new("NamWaveNetEngine", "WaveNet Dilated Convolution Neural Amp Model Inference Engine", DspNodeCategory::NeuralAi, "High-performance dilated 1D convolutional WaveNet neural network runtime for tube amplifier modeling")
        .with_param(DspParamSchema::knob("input_gain_db", "Pre-Amp Input Drive", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Pre-amp model drive and boosting gain parameter"))
        .with_param(DspParamSchema::knob("dilated_layers_count", "Dilated Conv Layers", 4.0, 16.0, 10.0, "layers", MacroRole::Character, "Active receptive field dilated convolution layers parameter"))
        .with_param(DspParamSchema::knob("hidden_channels", "Residual Channels", 8.0, 32.0, 16.0, "channels", MacroRole::Character, "Neural residual channel width per layer parameter"))
        .with_param(DspParamSchema::knob("output_level_db", "Output Normalization", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Post-neural inference output level normalization parameter"))
        .with_param(DspParamSchema::toggle("simd_vectorization", "SIMD Tensor Acceleration", true, "Enable AVX2/NEON SIMD dilated tensor acceleration parameter"))
    );
    descriptors.insert("FederatedMixLearner".to_string(), DspNodeDescriptor::new("FederatedMixLearner", "Privacy-Preserving Federated Mix Model Learner HUD", DspNodeCategory::NeuralAi, "Decentralized machine learning worker aggregating local mixing balances and EQ adjustments into a global collaborative model without leaking raw audio")
        .with_param(DspParamSchema::knob("local_epochs_per_round", "Local Training Epochs Per Round", 1.0, 50.0, 5.0, "epochs", MacroRole::Punch, "Number of local gradient descent passes evaluated over user mixing adjustments before federated parameter sync"))
        .with_param(DspParamSchema::knob("differential_privacy_epsilon", "Differential Privacy Noise Budget Epsilon", 0.1, 10.0, 1.0, "eps", MacroRole::Tone, "Laplace/Gaussian noise calibration parameter providing mathematically bounded privacy guarantees against training data inversion"))
        .with_param(DspParamSchema::knob("gradient_clipping_norm", "Gradient L2 Clipping Norm", 0.1, 5.0, 1.0, "norm", MacroRole::Punch, "Threshold ceiling bounding individual participant update magnitudes to defend against malicious weight poisoning"))
        .with_param(DspParamSchema::choice("federated_aggregation_strategy", "Federated Weight Aggregation Strategy", &["FedAvg (Standard Averaging)", "FedProx (Proximal Regularization)", "FedAdam (Adaptive Momentum)", "SCAFFOLD (Control Variates)"], 0, "Consensus optimization algorithm applied across participating DAW nodes to merge local mixing models"))
        .with_param(DspParamSchema::toggle("participate_in_federated_mesh", "Participate in Decentralized Model Mesh", true, "Enables continuous anonymous background gradient synchronization with peer mixing stations"))
    );
    descriptors.insert("NeuroAffectiveState".to_string(), DspNodeDescriptor::new("NeuroAffectiveState", "EEG Brainwave Emotion Valence & Cognitive State", DspNodeCategory::NeuralAi, "Real-time neuro-affective emotional telemetry state representing EEG spectral power ratios (Beta / (Alpha + Theta)), emotional valence, and autonomic physiological arousal")
        .with_param(DspParamSchema::knob("focus_index", "Beta Attentional Focus Index", 0.0, 1.0, 0.65, "norm", MacroRole::Tone, "Cognitive attentional focus calculated from Beta over Alpha plus Theta"))
        .with_param(DspParamSchema::knob("valence", "Emotional Valence Pleasure/Aversion", -1.0, 1.0, 0.25, "val", MacroRole::Character, "Emotional valence coordinate from unpleasant/negative to pleasant/positive"))
        .with_param(DspParamSchema::knob("arousal", "Physiological Autonomic Arousal", 0.0, 1.0, 0.55, "arousal", MacroRole::Punch, "Autonomic nervous system arousal from calm/drowsy to excited/activated"))
        .with_param(DspParamSchema::knob("alpha_theta_ratio", "Zen Meditation & Relaxation Index", 0.1, 10.0, 1.8, "ratio", MacroRole::Space, "Zen meditative relaxation index derived from parietal EEG leads"))
        .with_param(DspParamSchema::toggle("adaptive_damping", "Smooth Biometric Telemetry Damping", true, "Smooth biometric fluctuations with adaptive psychoacoustic damping"))
    );
    descriptors.insert("DrumClassClassification".to_string(), DspNodeDescriptor::new("DrumClassClassification", "Deep Neural Drum Hit Categorization & Transient Tagging HUD", DspNodeCategory::NeuralAi, "Real-time AI audio classifier identifying incoming drum transients into 7 standard percussion classes (Kick, Snare, Closed HiHat, Open HiHat, Tom, Cymbal, Percussion) with confidence scoring")
        .with_param(DspParamSchema::choice("classified_drum_class", "Predicted Percussion Classification", &["Kick Drum (Bass)", "Snare Drum (Acoustic/Electronic)", "Closed Hi-Hat", "Open Hi-Hat", "Tom-Tom (Floor/Rack)", "Crash/Ride Cymbal", "Auxiliary Percussion"], 0, "Real-time neural inference drum category output"))
        .with_param(DspParamSchema::knob("prediction_confidence_pct", "Neural Classifier Inference Confidence", 0.0, 100.0, 94.5, "%", MacroRole::Tone, "Inference certainty metric for classified transient"))
        .with_param(DspParamSchema::knob("transient_sensitivity", "Onset Transient Detection Sensitivity", 0.05, 1.0, 0.45, "thresh", MacroRole::Punch, "Threshold for isolating discrete drum hits from audio stream"))
        .with_param(DspParamSchema::toggle("auto_tag_sample_markers", "Auto-Tag Slices with Classified Drum Name", true, "Automatically label slice markers in sampler with predicted drum class"))
        .with_param(DspParamSchema::choice("inference_model_precision", "Neural Model Quantization & Inference Speed", &["INT8 Quantized Fast (0.2ms)", "FP16 TensorRT Accurate (0.6ms)", "FP32 Full Analytical"], 0, "Weight precision for the ONNX drum classifier network"))
    );
    descriptors.insert("AudioStylePreset".to_string(), DspNodeDescriptor::new("AudioStylePreset", "Neural AI Audio Style Transfer & Genre Timbre Preset", DspNodeCategory::NeuralAi, "Latent ONNX neural style transfer modeler injecting timbre characteristics across stems with genre embeddings, content/style loss trade-offs, and neural latent interpolation")
        .with_param(DspParamSchema::choice("neural_style_target", "Target Timbre Style Preset", &["Lo-Fi Vintage Vinyl", "Hyperpop Aggressive Saturated", "Modern Cinematic Orchestral", "Deep Dark Techno Sub", "Shoegaze Shimmer Wall"], 0, "Selected genre latent embedding target"))
        .with_param(DspParamSchema::knob("style_transfer_strength", "Style Transfer Latent Strength", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Degree of latent timbre style conditioning applied"))
        .with_param(DspParamSchema::knob("content_preservation_weight", "Content Phase & Pitch Preservation", 0.1, 1.0, 0.85, "wt", MacroRole::Character, "Penalty weight preserving original harmonic content"))
        .with_param(DspParamSchema::knob("diffusion_denoising_steps", "Neural Inference Denoising Steps", 4.0, 50.0, 20.0, "steps", MacroRole::Punch, "Number of denoising diffusion steps during inference"))
        .with_param(DspParamSchema::knob("neural_latent_temperature", "Latent Sampling Temperature / Chaos", 0.1, 2.0, 0.90, "temp", MacroRole::Space, "Stochastic variance in the latent feature mapping"))
        .with_param(DspParamSchema::toggle("enable_realtime_gpu_acceleration", "Real-Time GPU DirectML Acceleration", true, "Execute latent UNet convolutions on local GPU hardware"))
    );
    descriptors.insert("TargetCurve".to_string(), DspNodeDescriptor::new("TargetCurve", "AI Autonomous Mastering Spectral Curve Target Profile", DspNodeCategory::NeuralAi, "Target spectral distribution curves for autonomous neural mastering matching Pink Warmth, Bright Pop, Modern High-Gloss Pop, Club Sub-Bass Loudness, or custom multi-band FFT curves")
        .with_param(DspParamSchema::choice("spectral_target_profile", "Target Frequency Spectrum Curve Archetype", &["Warm (Vintage Tube / Low-Mid Analog Body)", "Bright (Acoustic Sparkle / High Air Top-End)", "Modern Pop (Balanced Sub-Bass & Crisp Presence)", "Club Loudness (Powerful 40Hz Bass & Controlled Midrange)", "Custom Multi-Band (Learned User Reference Profile)"], 2, "Selected target equalization curve for autonomous mastering"))
        .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Integrated Loudness Level", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Target program loudness meeting streaming or broadcast specs"))
        .with_param(DspParamSchema::knob("true_peak_ceiling_dbfs", "True Peak Brickwall Ceiling Limiter Level", -2.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "Maximum allowable inter-sample peak amplitude"))
        .with_param(DspParamSchema::knob("spectral_matching_amount", "Target Spectral Matching EQ Intensity", 0.0, 1.0, 0.80, "%", MacroRole::Character, "Strength of autonomous dynamic EQ correction toward target curve"))
        .with_param(DspParamSchema::knob("dynamic_range_crest_factor_db", "Target Dynamic Range Crest Factor", 4.0, 18.0, 9.0, "dB", MacroRole::Space, "Target ratio between peak energy and RMS background volume"))
        .with_param(DspParamSchema::toggle("enable_psychoacoustic_equal_loudness", "Match ISO 226 Equal-Loudness Contours", true, "Weight spectral corrections against human Fletcher-Munson curves"))
    );
    descriptors.insert("TargetTimbre".to_string(), DspNodeDescriptor::new("TargetTimbre", "Real-Time Spectral Neural Resynthesizer Voice Morpher", DspNodeCategory::NeuralAi, "Deep neural audio synthesis voice morpher transferring source input timbres into target instruments (Solo Violin, Orchestral Brass, Concert Flute, Vintage Analog Synthesizer) with instant pitch tracking")
        .with_param(DspParamSchema::choice("neural_target_instrument", "Neural Resynthesis Target Timbre Archetype", &["Solo Violin (Bowed Formant Resonances)", "Orchestral Brass (Bright Metallic Horn Flare)", "Concert Flute (Air Jet Breath & Pure Chiff)", "Analog PolySynth (Dual VCO Saw with Ladder Filter)"], 0, "Selected acoustic or electronic timbre target for neural transfer"))
        .with_param(DspParamSchema::knob("neural_morph_amount", "Timbral Transformation Morphing Depth", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Crossfade ratio between original timbre and neural resynthesis"))
        .with_param(DspParamSchema::knob("formant_retention_ratio", "Source Formant Acoustic Retention Factor", 0.0, 1.0, 0.50, "ratio", MacroRole::Character, "Preserve distinctive acoustic vocal or instrument resonances"))
        .with_param(DspParamSchema::knob("latent_space_temperature", "Neural Generative Latent Temperature", 0.1, 2.0, 0.8, "temp", MacroRole::Space, "Randomness and harmonic richness during neural latent decoding"))
        .with_param(DspParamSchema::knob("transient_preservation_strength", "Input Transient Strike Preservation", 0.0, 1.0, 0.90, "%", MacroRole::Punch, "Bypass neural morphing during initial attack transients"))
        .with_param(DspParamSchema::toggle("enable_gpu_tensor_acceleration", "Hardware Accelerated Tensor Resynthesis", true, "Offload neural inference matrix multiplications to GPU hardware"))
    );
    descriptors.insert("NeuralTimbreTransferStyleEngine".to_string(), DspNodeDescriptor::new("NeuralTimbreTransferStyleEngine", "Real-Time Neural Timbre Transfer Style Diffusion Engine", DspNodeCategory::NeuralAi, "Deep neural network inference engine mapping pitch and loudness contours onto rich target instrument timbral distributions in real time")
        .with_param(DspParamSchema::knob("timbre_transfer_blend_ratio", "Target Style Timbre Wet/Dry Blend", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Balance between input source timbre and synthetic neural target timbre"))
        .with_param(DspParamSchema::knob("neural_model_latent_temperature", "Diffusion Latent Stochastic Temperature", 0.0, 1.5, 0.6, "temp", MacroRole::Character, "Sampling temperature: lower values yield deterministic clarity, higher adds grit"))
        .with_param(DspParamSchema::knob("pitch_tracking_fidelity_weight", "Fundamental Pitch Tracking Weight", 0.5, 1.0, 0.98, "%", MacroRole::Punch, "Tracking confidence locking resynthesized harmonics onto input performance pitch"))
        .with_param(DspParamSchema::knob("dynamic_envelope_expression_weight", "Dynamic Envelope Expression Follower", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Follows nuanced micro-dynamics and articulations of input performer"))
        .with_param(DspParamSchema::knob("inference_chunk_latency_ms", "Neural Inference Chunk Buffer Size", 2.0, 50.0, 10.0, "ms", MacroRole::Space, "Inference block latency window balancing CPU overhead against real-time feel"))
        .with_param(DspParamSchema::toggle("enable_harmonic_spectral_residual", "Reinject Unvoiced High-Frequency Residual", true, "Preserves natural breath, bow scratches, and pick noise alongside neural timbre"))
    );
    descriptors.insert("NeuralSpeechProsodyCloner".to_string(), DspNodeDescriptor::new("NeuralSpeechProsodyCloner", "Neural Speech Prosody & Pitch Contour Inflection Transfer Engine", DspNodeCategory::NeuralAi, "Extracts fundamental pitch contours (F0), phonetic energy envelopes, and micro-timing cadences from reference speech and superimposes them onto target vocal tracks")
        .with_param(DspParamSchema::knob("prosody_transfer_intensity_ratio", "Prosodic Expressive Inflection Depth", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Strength of prosodic inflection and emotional contour applied to target"))
        .with_param(DspParamSchema::knob("f0_pitch_contour_conform_weight", "F0 Melodic Pitch Trajectory Adherence", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Degree of adherence to source speaker's melodic and vibrato curve"))
        .with_param(DspParamSchema::knob("syllable_timing_warping_fluidity", "Dynamic Time Warping Cadence Elasticity", 0.1, 2.0, 1.0, "warp", MacroRole::Character, "Dynamic time warping stiffness mapping syllable durations"))
        .with_param(DspParamSchema::knob("formant_envelope_preserve_ratio", "Singer Vocal Timbre Identity Guard", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Maintains original singer's vocal identity while adopting foreign prosody"))
        .with_param(DspParamSchema::knob("energy_stress_dynamics_boost_db", "Emphatic Syllable Stress Boost", 0.0, 12.0, 3.0, "dB", MacroRole::Punch, "Accentuates stressed syllables for emphatic natural performance"))
        .with_param(DspParamSchema::toggle("enable_creaky_voice_glottalization", "Creaky Voice & Glottal Stop Synthesis", false, "Replicates source speaker's vocal fry and glottal stop inflections"))
    );
    descriptors.insert("NeuralStemInterferenceRemover".to_string(), DspNodeDescriptor::new("NeuralStemInterferenceRemover", "Deep Spectral Neural Stem Crosstalk Remover", DspNodeCategory::NeuralAi, "Neural network spectrogram masking processor that isolates and removes vocal bleed from drum mics, cymbal bleed from vocal mics, and stage monitor spill")
        .with_param(DspParamSchema::knob("bleed_suppression_depth_db", "Neural Crosstalk Attenuation Depth", 0.0, 40.0, 20.0, "dB", MacroRole::Punch, "Maximum depth of neural bleed and crosstalk attenuation"))
        .with_param(DspParamSchema::knob("neural_model_confidence_threshold", "Stem Separation Confidence Gate", 0.1, 0.99, 0.75, "conf", MacroRole::Character, "Model probability threshold separating desired instrument from spill"))
        .with_param(DspParamSchema::knob("transient_phase_guard_sensitivity", "Primary Transients Phase Protection", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Preserves micro-transient punch in primary signal during neural suppression"))
        .with_param(DspParamSchema::knob("frequency_focus_band_min_hz", "Surgical Neural Processing Low Cut", 100.0, 3000.0, 800.0, "Hz", MacroRole::Tone, "Lower frequency cutoff for surgical neural masking processing"))
        .with_param(DspParamSchema::knob("spectral_mask_temporal_smoothing_ms", "Continuous Mask Time Smoothing", 1.0, 60.0, 12.0, "ms", MacroRole::Space, "Temporal smoothing of neural binary mask avoiding watery artifacts"))
        .with_param(DspParamSchema::toggle("enable_isolated_bleed_monitoring", "Solo Extracted Bleed Spill Signal", false, "Solos removed bleed and leakage noise for acoustic verification"))
    );
    descriptors.insert("NeuralHarmonicMainsHumRemover".to_string(), DspNodeDescriptor::new("NeuralHarmonicMainsHumRemover", "Neural Harmonic Mains Hum & Buzz Remover", DspNodeCategory::NeuralAi, "Deep neural network filter identifying and eliminating 50Hz/60Hz alternating current ground hum, power supply ripple, and fluorescent dimmer hash with zero phase distortion")
        .with_param(DspParamSchema::knob("mains_fundamental_base_frequency", "Base Electrical Grid Standard (50/60Hz)", 48.0, 62.0, 60.0, "Hz", MacroRole::Tone, "Base power line grid frequency for notch tracking"))
        .with_param(DspParamSchema::knob("hum_suppression_depth_db", "Harmonic Hum Attenuation Depth", 0.0, 60.0, 24.0, "dB", MacroRole::Punch, "Maximum attenuation depth applied to detected hum and buzz harmonics"))
        .with_param(DspParamSchema::knob("harmonic_series_tracking_limit", "Upper Harmonic Buzz Overtone Limit", 4.0, 40.0, 20.0, "harm", MacroRole::Character, "Highest electrical harmonic multiple tracked by neural suppression filter"))
        .with_param(DspParamSchema::knob("neural_adaptation_speed_ms", "Neural Drift Tracking Adaptation Rate", 20.0, 1000.0, 120.0, "ms", MacroRole::Space, "Adaptation speed tracking slow drift in power grid frequency"))
        .with_param(DspParamSchema::knob("low_end_tonal_guard_retention", "Sub-Bass Musical Content Guard", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Protects musical bass instruments and kick drums near hum frequencies"))
        .with_param(DspParamSchema::toggle("enable_difference_solo_monitoring", "Solo Extracted Ground Buzz Signal", false, "Solos removed electrical noise and ripple for acoustic verification"))
    );
    descriptors.insert("NeuralHumAndHarmonicsDeclicker".to_string(), DspNodeDescriptor::new("NeuralHumAndHarmonicsDeclicker", "Deep Neural Hum & Sub-Harmonic De-Clicker", DspNodeCategory::NeuralAi, "Recurrent neural network trained to separate 50Hz/60Hz ground mains hum, thyristor buzzer buzzes, and impulsive clicks from delicate acoustic performances")
        .with_param(DspParamSchema::knob("mains_fundamental_frequency_hz", "Mains Hum Fundamental (50/60 Hz)", 45.0, 65.0, 50.0, "Hz", MacroRole::Tone, "Base power-line frequency target for harmonic comb rejection"))
        .with_param(DspParamSchema::knob("harmonic_tracking_depth", "Tracked Harmonics Filter Count", 2.0, 32.0, 16.0, "count", MacroRole::Character, "Number of higher harmonics tracked and suppressed by neural network"))
        .with_param(DspParamSchema::knob("neural_confidence_threshold", "Neural Separation Mask Threshold", 0.0, 1.0, 0.85, "conf", MacroRole::Punch, "Confidence threshold before neural attenuation is engaged"))
        .with_param(DspParamSchema::knob("click_impulse_rejection_sensitivity", "Impulsive Click Detection Sensitivity", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Sensitivity to vinyl scratch, digital tick, and mechanical click events"))
        .with_param(DspParamSchema::knob("acoustic_air_reconstruction_ratio", "Masked Timbre Interpolation Fill", 0.0, 1.0, 0.4, "%", MacroRole::Space, "Synthesizes underlying musical timbre into excised frequency notches"))
        .with_param(DspParamSchema::toggle("enable_sidechain_isolated_listen", "Solo Neutralized Noise Components", false, "Auditions isolated hum, buzz, and click residuals"))
    );
    descriptors.insert("BiometricPupillometricEmotionMapper".to_string(), DspNodeDescriptor::new("BiometricPupillometricEmotionMapper", "Pupillometric & Heart Rate Audio Emotion Modulator", DspNodeCategory::NeuralAi, "Affective computing neural network transforming biometric sensor streams (pupil dilation, galvanic skin response, pulse rate) into musical valence and arousal modulation curves")
        .with_param(DspParamSchema::knob("arousal_autonomic_nervous_level", "Physiological Arousal Intensity", 0.0, 1.0, 0.5, "norm", MacroRole::Punch, "Sympathetic nervous system activation mapped to sonic tempo and dynamics"))
        .with_param(DspParamSchema::knob("valence_emotional_positivity", "Emotional Valence Positivity Factor", -1.0, 1.0, 0.2, "val", MacroRole::Tone, "Affective positivity index mapped to consonant harmonic structures and brightness"))
        .with_param(DspParamSchema::knob("heart_rate_variability_ms", "HRV R-R Interval Variation", 10.0, 150.0, 65.0, "ms", MacroRole::Character, "Vagal parasympathetic tone modulating subtle micro-rhythmic humanization"))
        .with_param(DspParamSchema::knob("sensor_integration_time_constant_sec", "Biometric Smoothing Integration Time", 0.1, 10.0, 1.2, "sec", MacroRole::Space, "Temporal integration window filtering sudden sensor artifacts and blinks"))
        .with_param(DspParamSchema::knob("modulation_routing_depth_pct", "Global Synthesis Modulation Depth", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Overall influence of emotional prediction on downstream audio parameters"))
        .with_param(DspParamSchema::toggle("enable_circadian_drift_compensation", "Circadian Baseline Drift Compensation", true, "Removes physiological baseline drift across prolonged creative studio sessions"))
    );
    descriptors.insert("QuantumAnnealingHarmonicOptimizer".to_string(), DspNodeDescriptor::new("QuantumAnnealingHarmonicOptimizer", "Quantum Annealing Multi-Track Spectral Optimizer", DspNodeCategory::NeuralAi, "Ising model Hamiltonian quantum annealing processor resolving multi-track frequency collisions and phase conflicts across overlapping stems via ground-state energy minimization")
        .with_param(DspParamSchema::knob("transverse_magnetic_field_gamma", "Transverse Tunneling Magnetic Field", 0.0, 10.0, 3.5, "Gamma", MacroRole::Punch, "Quantum fluctuation energy permitting tunneling through local spectral masking barriers"))
        .with_param(DspParamSchema::knob("annealing_schedule_tau_ms", "Annealing Transition Duration", 5.0, 500.0, 80.0, "ms", MacroRole::Space, "Time constant for adiabatic evolution from quantum driver to problem Hamiltonian"))
        .with_param(DspParamSchema::knob("spectral_collision_penalty_lambda", "Spectral Masking Collision Penalty", 0.1, 50.0, 12.0, "lambda", MacroRole::Tone, "Optimization weighting punishing unmasking interference across adjacent stems"))
        .with_param(DspParamSchema::knob("qubit_coupling_matrix_gain", "Inter-Track Cross-Talk Coupling J", -1.0, 1.0, 0.65, "J", MacroRole::Character, "Pairwise coupling interaction parameter between conflicting instrument frequency bins"))
        .with_param(DspParamSchema::knob("energy_ground_state_convergence", "Ground State Minimum Tolerance", 0.001, 1.0, 0.05, "eps", MacroRole::Tone, "Convergence stopping condition for globally unmasked master spectral balance"))
        .with_param(DspParamSchema::toggle("enable_adiabatic_quantum_tunneling", "Adiabatic Quantum Tunneling Solver", true, "Enables quantum tunneling past steep local minima without transient distortion"))
    );
    descriptors.insert("NeuralPerceptualMaskingDenoise".to_string(), DspNodeDescriptor::new("NeuralPerceptualMaskingDenoise", "Psychoacoustic Critical-Band Perceptual Spectral De-Noiser", DspNodeCategory::NeuralAi, "Neural psychoacoustic speech and music denoiser calculating human cochlear masking thresholds (Bark scale) to surgically suppress noise only when audible")
        .with_param(DspParamSchema::knob("psychoacoustic_masking_depth_db", "Perceptual Masking Attenuation Depth", 0.0, 48.0, 18.0, "dB", MacroRole::Punch, "Maximum noise attenuation applied below psychoacoustic masking threshold"))
        .with_param(DspParamSchema::knob("noise_floor_profile_learning_rate", "Stationary Noise Profile Tracking", 0.001, 0.5, 0.04, "rate", MacroRole::Character, "Adaptive speed of stationary background noise floor statistical estimation"))
        .with_param(DspParamSchema::knob("bark_scale_smoothing_filter", "Cochlear Bark Critical Band Smoothing", 0.1, 4.0, 1.2, "bark", MacroRole::Tone, "Spectral smoothing width conforming to human ear basilar membrane filters"))
        .with_param(DspParamSchema::knob("transient_unmasking_protection_db", "Percussive Attack Preservation Pad", 0.0, 18.0, 6.0, "dB", MacroRole::Punch, "Prevents phase smearing and pre-echo artifacts on sharp percussive transients"))
        .with_param(DspParamSchema::knob("speech_harmonic_isolation_gain", "Vocal Harmonic Tracking Enhancer", 0.0, 12.0, 3.5, "dB", MacroRole::Tone, "Reinforces harmonic comb structure of lead vocals in noisy acoustic environments"))
        .with_param(DspParamSchema::toggle("enable_spectral_subtraction_overdrive", "Over-Subtraction Musical Artifact Suppression", true, "Eliminates musical 'birdie' noise artifacts during deep noise attenuation"))
    );
    descriptors.insert("NeuralLatentTimbreInterpolationAutoencoder".to_string(), DspNodeDescriptor::new("NeuralLatentTimbreInterpolationAutoencoder", "Variational Latent Timbre Space Interpolator", DspNodeCategory::NeuralAi, "Deep neural audio autoencoder mapping harmonic spectra into an 8-dimensional continuous latent manifold with geodesic vector path interpolation and orthogonal disentanglement")
        .with_param(DspParamSchema::knob("latent_dimension_x_morph", "Latent Manifold Dimension X (Warmth)", -3.0, 3.0, 0.0, "dim", MacroRole::Tone, "First principal axis interpolating from bright metallic to warm wood"))
        .with_param(DspParamSchema::knob("latent_dimension_y_morph", "Latent Manifold Dimension Y (Acoustic Density)", -3.0, 3.0, 0.5, "dim", MacroRole::Character, "Second principal axis governing density of partial overtones and breathiness"))
        .with_param(DspParamSchema::knob("variational_prior_temperature", "Sampling Distribution Temperature", 0.1, 2.0, 0.85, "T", MacroRole::Space, "Stochastic variance across latent dimensions creating generative timbre micro-variations"))
        .with_param(DspParamSchema::knob("encoder_reconstruction_fidelity", "Neural Reconstruction Fidelity", 0.1, 1.0, 0.92, "fidelity", MacroRole::Punch, "Balance between smooth latent interpolation and raw harmonic spectral precision"))
        .with_param(DspParamSchema::knob("geodesic_trajectory_smoothing_ms", "Manifold Trajectory Inertia", 5.0, 500.0, 65.0, "ms", MacroRole::Tone, "Slew rate smoothing rapid parameter jumps across latent coordinate space"))
        .with_param(DspParamSchema::toggle("enable_adversarial_phase_coherence", "Adversarial Phase Coherence Restoration", true, "Neural discriminator ensuring minimum-phase coherence during rapid interpolation"))
    );
    descriptors.insert("NeuralDifferentiableIirEqualizer".to_string(), DspNodeDescriptor::new("NeuralDifferentiableIirEqualizer", "Differentiable Gradient IIR Biquad Equalizer", DspNodeCategory::NeuralAi, "Machine-learning trained minimum-phase 12-band parametric equalizer adapting in real time via differentiable gradient descent to match target perceptual curves and remove masking")
        .with_param(DspParamSchema::knob("gradient_learning_rate_alpha", "Adaptive Filter Convergence Speed", 0.001, 0.5, 0.05, "alpha", MacroRole::Punch, "Learning rate step size for gradient-based filter curve adaptation"))
        .with_param(DspParamSchema::knob("target_perceptual_curve_match_db", "Target Curve Matching Depth", 0.0, 24.0, 12.0, "dB", MacroRole::Tone, "Maximum equalization boost or cut allowed during curve convergence"))
        .with_param(DspParamSchema::knob("biquad_q_regularization_penalty", "Resonance Q Factor Regularization", 0.1, 10.0, 1.5, "lambda", MacroRole::Character, "Penalty term preventing overly sharp ringing filter resonances"))
        .with_param(DspParamSchema::knob("spectral_smoothness_constraint", "Spectral Curve Smoothness Constraint", 0.1, 5.0, 1.2, "beta", MacroRole::Space, "Enforces broad, natural musical EQ curves over spiky corrections"))
        .with_param(DspParamSchema::knob("auditory_weighting_iso226", "ISO 226 Equal-Loudness Weighting", 0.0, 1.0, 0.7, "iso", MacroRole::Tone, "Weights adaptation error according to human psychoacoustic sensitivity"))
        .with_param(DspParamSchema::toggle("enable_minimum_phase_warp", "Guaranteed Minimum-Phase Biquad Stability", true, "Constrains all filter poles strictly inside the unit circle preventing instability"))
    );
    descriptors.insert("NeuralPerceptualSpectralEnhancer".to_string(), DspNodeDescriptor::new("NeuralPerceptualSpectralEnhancer", "Deep Neural Psychoacoustic Harmonic Restorer", DspNodeCategory::NeuralAi, "Lightweight deep neural network model predicting missing high-frequency harmonics and restoring lost air band presence from degraded or band-limited audio")
        .with_param(DspParamSchema::knob("neural_inference_blend", "Neural Harmonic Reconstruction Depth", 0.0, 1.0, 0.65, "mix", MacroRole::Tone, "Depth of generated high-frequency harmonics added to output signal"))
        .with_param(DspParamSchema::knob("air_band_excitation_freq_hz", "Psychoacoustic Air Excitation Corner", 6000.0, 20000.0, 12500.0, "Hz", MacroRole::Character, "Corner frequency above which neural model reconstructs upper partials"))
        .with_param(DspParamSchema::knob("spectral_envelope_tracking_speed", "Neural Spectral Envelope Tracking", 1.0, 100.0, 24.0, "ms", MacroRole::Punch, "Inference temporal smoothing window tracking input dynamics"))
        .with_param(DspParamSchema::knob("perceptual_masking_threshold_db", "Psychoacoustic Masking Protection", -60.0, -12.0, -32.0, "dB", MacroRole::Space, "Threshold below which faint artifacts are masked by Bark critical bands"))
        .with_param(DspParamSchema::knob("odd_even_generated_harmonic_ratio", "Restored Harmonic Odd/Even Ratio", 0.1, 2.0, 0.85, "ratio", MacroRole::Tone, "Balance between tube-like even harmonics and edge odd harmonics"))
        .with_param(DspParamSchema::toggle("enable_phase_coherent_synthesis", "Phase Coherent Carrier Locking", true, "Locks generated partials to fundamental carrier phase preventing comb filtering"))
    );
    descriptors.insert("NeuralHarmonicExciterAirRestorer".to_string(), DspNodeDescriptor::new("NeuralHarmonicExciterAirRestorer", "Neural Audio Spectral Harmonic Tube Exciter", DspNodeCategory::NeuralAi, "Deep neural audio model emulating vintage vacuum tube harmonic exciters, generating phase-aligned psychoacoustic sheen, high-frequency sparkle, and vocal clarity")
        .with_param(DspParamSchema::knob("exciter_spectral_drive_amount", "Neural Tube Saturation Drive", 0.0, 1.0, 0.45, "drive", MacroRole::Punch, "Saturation drive level feeding audio into the neural harmonic model"))
        .with_param(DspParamSchema::knob("high_frequency_shelf_hinge_hz", "Air Band High-Shelf Corner Frequency", 3000.0, 15000.0, 7500.0, "Hz", MacroRole::Tone, "High-pass frequency above which harmonic excitation is injected"))
        .with_param(DspParamSchema::knob("second_harmonic_sheen_level", "Even-Order Harmonic Sparkle Boost", 0.0, 1.0, 0.58, "sheen", MacroRole::Character, "Relative generation ratio of warm second-order tube harmonics"))
        .with_param(DspParamSchema::knob("third_harmonic_edge_density", "Odd-Order Harmonic Presence Edge", 0.0, 1.0, 0.32, "edge", MacroRole::Tone, "Relative generation ratio of cutting third-order edge harmonics"))
        .with_param(DspParamSchema::knob("stereo_exciter_crosstalk_spread", "Binaural Spectral Width Spread", 0.0, 1.0, 0.4, "spread", MacroRole::Space, "Inter-channel harmonic decorrelation widening apparent acoustic soundstage"))
        .with_param(DspParamSchema::toggle("enable_dynamic_high_band_sidechain", "Dynamic High-Band Transient Gate", true, "Modulates exciter drive dynamically based on input high-frequency transient density"))
    );
    descriptors.insert("NeuralSpectralEnvelopeMorpher".to_string(), DspNodeDescriptor::new("NeuralSpectralEnvelopeMorpher", "Deep Latent Neural Spectral Formant Morpher", DspNodeCategory::NeuralAi, "Deep neural cross-synthesis network morphing the spectral envelopes and formant topologies of dual input audio sources with differentiable FFT bins and latent timbre projection")
        .with_param(DspParamSchema::knob("morph_latent_interpolation_ratio", "Latent Timbre Interpolation Position", 0.0, 1.0, 0.5, "morph", MacroRole::Character, "Continuous interpolation between Carrier and Modulator latent spectral representations"))
        .with_param(DspParamSchema::knob("spectral_bin_resolution_bands", "Neural Latent Formant Band Resolution", 16.0, 256.0, 64.0, "bands", MacroRole::Tone, "Number of differentiable spectral formant bands extracted across human vocal hearing range"))
        .with_param(DspParamSchema::knob("formant_frequency_shift_semitones", "Formant Envelope Frequency Shift", -24.0, 24.0, 0.0, "semi", MacroRole::Tone, "Pitch-independent vertical shifting of extracted vocal tract formant resonances"))
        .with_param(DspParamSchema::knob("latent_space_temperature_chaos", "Neural Latent Sampling Temperature", 0.1, 2.0, 0.85, "temp", MacroRole::Space, "Stochastic variance injected into latent manifold traversal generating spectral flutter"))
        .with_param(DspParamSchema::knob("residual_unvoiced_noise_mix", "Unvoiced Consonant Noise Pass-Through", 0.0, 1.0, 0.25, "mix", MacroRole::Punch, "Ratio of high-frequency unvoiced sibilance and noise burst passed un-morphed for clarity"))
        .with_param(DspParamSchema::toggle("enable_phase_vocoder_lock", "Phase Vocoder Bin Phase Coherence Lock", true, "Maintains vertical phase coherence across adjacent FFT bins to eliminate transient phase smearing"))
    );
    descriptors.insert("NeuralPerceptualTransientEnhancer".to_string(), DspNodeDescriptor::new("NeuralPerceptualTransientEnhancer", "Neural Attention Perceptual Transient Sculptor", DspNodeCategory::NeuralAi, "Transformer-based neural transient separator and shape sculptor that adaptively detects attack transients, expands attack punch, and restores lost acoustic detail")
        .with_param(DspParamSchema::knob("transient_attack_boost_gain_db", "Transient Attack Punch Boost Gain", -18.0, 18.0, 4.5, "dB", MacroRole::Punch, "Gain applied exclusively to neural-detected transient attack components"))
        .with_param(DspParamSchema::knob("sustain_body_level_gain_db", "Sustain Body Decay Level Gain", -18.0, 18.0, -1.5, "dB", MacroRole::Tone, "Gain applied to underlying stationary tonal and decay components"))
        .with_param(DspParamSchema::knob("transient_detection_temporal_sharpness", "Neural Attention Temporal Window Sharpness", 0.5, 5.0, 1.8, "sharp", MacroRole::Character, "Attention window width focusing transient identification on fast micro-events"))
        .with_param(DspParamSchema::knob("high_frequency_air_restoration_db", "Lost Ultrasonic Air Transient Restoration", 0.0, 12.0, 3.2, "dB", MacroRole::Tone, "Synthesizes high-frequency spectral components above 12 kHz lost during compression"))
        .with_param(DspParamSchema::knob("transient_stereo_width_spread", "Transient Stereo Field Spatial Spread", 0.0, 2.0, 1.25, "width", MacroRole::Space, "Widens transient attacks across stereo panorama while keeping sustain centered"))
        .with_param(DspParamSchema::toggle("enable_zero_latency_predictive_mode", "Zero-Latency Causal Neural Inference Mode", true, "Enforces strict causality and zero lookahead buffer delay for real-time live performance"))
    );
    descriptors.insert("NeuralSpectralHarmonicRestorer".to_string(), DspNodeDescriptor::new("NeuralSpectralHarmonicRestorer", "Neural Latent Transformer Spectral Harmonic Restorer", DspNodeCategory::NeuralAi, "Deep latent attention transformer network inferring and resynthesizing missing ultrasonic air and high-order harmonics lost in lossy compression or bandlimited recording")
        .with_param(DspParamSchema::knob("harmonic_restoration_amount", "Synthesized Harmonic Injection Level", 0.0, 2.0, 1.0, "ratio", MacroRole::Tone, "Amount of neural-inferred high-order harmonic spectral energy mixed back into output signal"))
        .with_param(DspParamSchema::knob("ultrasonic_air_ceiling_khz", "Ultrasonic Reconstruction Ceiling", 12.0, 48.0, 24.0, "kHz", MacroRole::Tone, "Upper frequency boundary target for reconstructed harmonic overtones and air shimmer"))
        .with_param(DspParamSchema::knob("latent_prediction_confidence", "Neural Manifold Confidence Threshold", 0.1, 1.0, 0.85, "conf", MacroRole::Character, "Attention probability cutoff preventing hallucinated artifacts in noisy or chaotic sources"))
        .with_param(DspParamSchema::knob("transient_phase_alignment_lock", "Transient Coherent Phase Re-Alignment", 0.0, 1.0, 0.95, "lock", MacroRole::Punch, "Phase-locks newly synthesized partials to fundamental transient attack onset edges"))
        .with_param(DspParamSchema::knob("even_to_odd_generated_harmonic_bias", "Generated Even vs Odd Harmonic Weight", 0.0, 1.0, 0.45, "bias", MacroRole::Character, "Balance between tube-like even-order harmonics and tape/transistor odd-order harmonics"))
        .with_param(DspParamSchema::toggle("enable_psychoacoustic_masking_guard", "Psychoacoustic Masking Masking Guard", true, "Suppresses generated harmonics that fall below auditory masking thresholds to prevent mud"))
    );
    descriptors.insert("NeuralZeroPhaseTransientRestorer".to_string(), DspNodeDescriptor::new("NeuralZeroPhaseTransientRestorer", "Neural Attention Zero-Phase Attack Transient Sculptor", DspNodeCategory::NeuralAi, "Causal deep neural network that detects smeared attack wavefronts and synthesizes zero-phase transient impulses to restore drum punch and acoustic impact")
        .with_param(DspParamSchema::knob("transient_attack_reconstruction_gain_db", "Restored Transient Peak Boost Gain", 0.0, 18.0, 6.0, "dB", MacroRole::Punch, "Direct gain boost applied to neural-extracted attack wavefront transients"))
        .with_param(DspParamSchema::knob("wavefront_dispersion_correction", "Attack Wavefront Dispersion Unsmear", 0.0, 1.0, 0.65, "corr", MacroRole::Punch, "Re-aligns frequency-dispersed transient arrival times into a single coherent acoustic impact"))
        .with_param(DspParamSchema::knob("temporal_attention_resolution_ms", "Neural Attention Temporal Window Span", 0.2, 10.0, 1.5, "ms", MacroRole::Character, "Temporal sliding attention span isolating micro-percussive clicks from tonal sustain"))
        .with_param(DspParamSchema::knob("sustain_tail_suppression_ratio", "Sustain Residual Masking Suppression", 0.0, 1.0, 0.35, "ratio", MacroRole::Tone, "Attenuates smeared reverberant wash immediately following attack for crisp definition"))
        .with_param(DspParamSchema::knob("stereo_spatial_transient_focuser", "Transient Stereo Imaging Center Focuser", 0.0, 2.0, 1.1, "focus", MacroRole::Space, "Tightens transient spatial localization while maintaining wide ambient stereo sustain"))
        .with_param(DspParamSchema::toggle("enable_anti_aliased_impulse_generation", "Sub-Sample Bandlimited Impulse Interpolation", true, "Interpolates reconstructed transient impulses with sinc kernel to eliminate digital aliasing"))
    );
    descriptors.insert("NeuralSubHarmonicFundamentalSynthesizer".to_string(), DspNodeDescriptor::new("NeuralSubHarmonicFundamentalSynthesizer", "Neural Attention Deep Sub-Harmonic Fundamental Synthesizer", DspNodeCategory::NeuralAi, "Deep causal neural attention network inferring and reconstructing lost sub-octave fundamental sine waves and chest resonance from bandlimited or small-speaker audio")
        .with_param(DspParamSchema::knob("sub_octave_injection_level_db", "Neural Reconstructed Sub-Bass Injection Level", -36.0, 12.0, 0.0, "dB", MacroRole::Punch, "Direct gain boost of neural-generated fundamental sine component mixed into output"))
        .with_param(DspParamSchema::knob("sub_harmonic_divider_order", "Subharmonic Frequency Division Ratio", 1.0, 3.0, 1.0, "oct", MacroRole::Tone, "Selects first (-1 oct), second (-2 oct), or fractional sub-harmonic frequency generation"))
        .with_param(DspParamSchema::knob("neural_pitch_tracking_confidence", "Neural Pitch Manifold Confidence Threshold", 0.2, 1.0, 0.88, "conf", MacroRole::Character, "Confidence cutoff gating sub synthesis on unpitched or noisy input frames"))
        .with_param(DspParamSchema::knob("sub_transient_phase_lock_speed_ms", "Sub-Bass Wavefront Phase Lock Response Speed", 0.5, 20.0, 2.5, "ms", MacroRole::Punch, "Locks synthesized sub waveform zero-crossings to higher-order transient attack fronts"))
        .with_param(DspParamSchema::knob("psychoacoustic_low_shelf_hinge_hz", "Sub Synthesis Upper Boundary Frequency Hinge", 40.0, 160.0, 90.0, "Hz", MacroRole::Tone, "Upper frequency boundary below which the neural model operates"))
        .with_param(DspParamSchema::toggle("enable_even_order_sub_saturation", "Quadratic Asymmetric Tube Saturation on Sub Bass", true, "Applies musical warmth and slight harmonic presence to synthesized sub waves"))
    );
    descriptors.insert("NeuralStereoFieldDeCorrelator".to_string(), DspNodeDescriptor::new("NeuralStereoFieldDeCorrelator", "Neural Latent Attention Stereo Field Decorrelator", DspNodeCategory::NeuralAi, "Deep neural network estimating optimal non-comb-filtering allpass phase transforms to widen mono and narrow stereo sources while maintaining 100% mono downmix phase cancellation safety")
        .with_param(DspParamSchema::knob("spatial_decorrelation_width_factor", "Neural Stereo Field Decorrelation Width Factor", 0.0, 2.5, 1.25, "width", MacroRole::Space, "Spread factor applied to latent psychoacoustic spatial eigenvectors"))
        .with_param(DspParamSchema::knob("mono_downmix_null_protection", "Mono Summing Phase Cancellation Safety Guard", 0.5, 1.0, 0.98, "prot", MacroRole::Tone, "Prevents negative correlation by dynamically pinning lateral phase displacement under +0.2"))
        .with_param(DspParamSchema::knob("spectral_clustering_band_count", "Latent Attention Spectral Band Partitioning", 4.0, 64.0, 24.0, "bands", MacroRole::Character, "Number of independent critical bark bands evaluated for spatial unmasking"))
        .with_param(DspParamSchema::knob("transient_decorrelation_bypass_ratio", "Transient Center Anchor Preservation Ratio", 0.0, 1.0, 0.72, "ratio", MacroRole::Punch, "Preserves center mono imaging on punchy percussive transient wavefronts"))
        .with_param(DspParamSchema::knob("diffuse_ambience_extraction_db", "Synthesized Diffuse Ambient Side Gain", -18.0, 12.0, 1.5, "dB", MacroRole::Space, "Gain boost on extracted diffuse reverberant components panned to sides"))
        .with_param(DspParamSchema::toggle("enable_binaural_pinna_cues", "HRTF Pinna Spectral Notch Imprinting", true, "Synthesizes subtle 7-9 kHz elevation notches for 3D out-of-head headphone spatialization"))
    );
    descriptors.insert("NeuralDiffusionLatentTransientSynthesizer".to_string(), DspNodeDescriptor::new("NeuralDiffusionLatentTransientSynthesizer", "Neural Latent Diffusion Micro-Transient Synthesizer", DspNodeCategory::NeuralAi, "Score-based conditional latent diffusion model generating micro-transients, attack percussions, and acoustic onset textures in real-time with continuous latent space conditioning")
        .with_param(DspParamSchema::knob("diffusion_denoising_steps", "Reverse Latent Diffusion Denoising Step Count", 2.0, 16.0, 4.0, "steps", MacroRole::Punch, "Inference step count balancing ultra-low latency with high acoustic detail"))
        .with_param(DspParamSchema::knob("transient_density_temperature", "Latent Stochastic Sampling Temperature", 0.1, 1.5, 0.75, "temp", MacroRole::Character, "Sampling entropy controlling diversity and organic irregularity of generated onsets"))
        .with_param(DspParamSchema::knob("attack_sharpness_latent_bias", "Micro-Transient Attack Crest Factor Bias", 0.0, 2.0, 1.25, "bias", MacroRole::Punch, "Shapes neural onset profile from smooth swell to razor-sharp percussive spike"))
        .with_param(DspParamSchema::knob("conditioning_spectral_centroid_hz", "Spectral Conditioning Brightness Centroid", 500.0, 12000.0, 4500.0, "Hz", MacroRole::Tone, "Conditioning vector target frequency guiding diffusion generation"))
        .with_param(DspParamSchema::knob("latent_temporal_coherence", "Cross-Frame Latent Manifold Coherence", 0.0, 1.0, 0.82, "coh", MacroRole::Space, "Preserves phase continuity across successive synthesized audio grain blocks"))
        .with_param(DspParamSchema::toggle("enable_realtime_speculative_decoding", "Real-Time Speculative Neural Decoding Cache", true, "Enables zero-latency speculative decoding acceleration on GPU/NPU"))
    );
    descriptors.insert("NeuralWaveletAcousticDenoisingDenoiser".to_string(), DspNodeDescriptor::new("NeuralWaveletAcousticDenoisingDenoiser", "Neural Wavelet Multi-Scale Acoustic Denoiser", DspNodeCategory::NeuralAi, "Continuous wavelet neural network providing multi-scale psychoacoustic background noise suppression, room hum elimination, and HVAC removal without musical transient smearing")
        .with_param(DspParamSchema::knob("denoising_suppression_depth_db", "Background Noise Suppression Attenuation Depth", 0.0, 48.0, 18.0, "dB", MacroRole::Tone, "Maximum attenuation applied to stationary and non-stationary background noise"))
        .with_param(DspParamSchema::knob("harmonic_preservation_threshold", "Musical Harmonic Component Protection Margin", 0.1, 1.0, 0.88, "prot", MacroRole::Character, "Confidence threshold protecting instrument and vocal upper partials from gating"))
        .with_param(DspParamSchema::knob("wavelet_octave_decomposition_bands", "Continuous Wavelet Scale Band Count", 4.0, 16.0, 8.0, "scales", MacroRole::Tone, "Number of dyadic wavelet octaves analyzed for multi-resolution denoising"))
        .with_param(DspParamSchema::knob("transient_pass_through_sensitivity", "Transient Wavefront Pass-Through Sensitivity", 0.0, 1.0, 0.75, "sens", MacroRole::Punch, "Detects fast onsets and instantly bypasses noise reduction to prevent soft attacks"))
        .with_param(DspParamSchema::knob("spectral_floor_whitening_db", "Subtle Comfort Noise Floor Injection", -80.0, -36.0, -60.0, "dB", MacroRole::Space, "Injects ultra-low psychoacoustic comfort noise to prevent unnerving dead silence"))
        .with_param(DspParamSchema::toggle("enable_adaptive_room_noise_learning", "Dynamic Adaptive Noise Profile Learning", true, "Continuously learns changing ambient room noise during audio pauses"))
    );
    descriptors.insert("NeuralContinuousNormalizingFlowTimbreMorpher".to_string(), DspNodeDescriptor::new("NeuralContinuousNormalizingFlowTimbreMorpher", "Neural Continuous Normalizing Flow Timbre Morpher", DspNodeCategory::NeuralAi, "Continuous Normalizing Flow (CNF) neural ordinary differential equation (ODE) generative model smoothly warping arbitrary spectral audio distributions in continuous latent phase space")
        .with_param(DspParamSchema::knob("ode_integration_steps", "Continuous ODE Integration Step Count", 2.0, 24.0, 6.0, "steps", MacroRole::Punch, "Integration steps along neural vector field balancing detail and latency"))
        .with_param(DspParamSchema::knob("latent_flow_trajectory", "Latent Normalizing Flow Morph Trajectory", 0.0, 1.0, 0.5, "pos", MacroRole::Tone, "Continuous morph parameter flowing base distribution toward target timbre"))
        .with_param(DspParamSchema::knob("vector_field_curl_damping", "Neural Vector Field Curl / Divergence Damping", 0.0, 1.0, 0.7, "damp", MacroRole::Character, "Constrains chaotic eddies in probability flow ensuring smooth timbre transitions"))
        .with_param(DspParamSchema::knob("spectral_prior_temperature", "Base Gaussian Latent Prior Temperature", 0.1, 2.0, 0.9, "temp", MacroRole::Tone, "Prior entropy controlling richness vs predictability of synthesized timbre"))
        .with_param(DspParamSchema::knob("phase_field_coherence_weight", "Spectral Phase Field Coherence Weight", 0.0, 1.0, 0.82, "coh", MacroRole::Space, "Maintains vertical phase coherence across neural frequency bins"))
        .with_param(DspParamSchema::toggle("enable_adaptive_runge_kutta", "Adaptive Runge-Kutta 4th-Order Solver", true, "Dynamically scales step size to eliminate phase discontinuities during rapid sweeps"))
    );
    descriptors.insert("NeuralDifferentiableWavetablePhaseVocoder".to_string(), DspNodeDescriptor::new("NeuralDifferentiableWavetablePhaseVocoder", "Neural Differentiable Wavetable Phase Vocoder", DspNodeCategory::NeuralAi, "End-to-end differentiable neural phase vocoder with learned gradient-based pitch-synchronous wavetable frame extraction and phase alignment")
        .with_param(DspParamSchema::knob("analysis_window_size_ms", "Differentiable Analysis Frame Window", 10.0, 120.0, 35.0, "ms", MacroRole::Tone, "Temporal analysis window for gradient-based wavetable extraction"))
        .with_param(DspParamSchema::knob("pitch_synchronous_overlap", "Pitch-Synchronous Window Overlap Ratio", 0.5, 0.95, 0.75, "ratio", MacroRole::Space, "Overlap factor preventing phase smearing during dynamic pitch shifts"))
        .with_param(DspParamSchema::knob("neural_phase_regularization", "Loss-Regularized Phase Continuity Weight", 0.0, 1.0, 0.88, "reg", MacroRole::Character, "Neural phase loss penalty maintaining pristine transient punch"))
        .with_param(DspParamSchema::knob("latent_harmonic_smoothing", "Latent Harmonic Envelope Regularization", 0.0, 1.0, 0.45, "smooth", MacroRole::Tone, "Smooths spectral peak variations across wavetable frames"))
        .with_param(DspParamSchema::knob("transient_phase_lock_threshold", "Transient Wavefront Phase-Lock Sensitivity", 0.0, 1.0, 0.65, "sens", MacroRole::Punch, "Locks phases of multi-bin transients to preserve percussion punch"))
        .with_param(DspParamSchema::toggle("enable_gradient_timbre_interpolation", "Continuous Latent Gradient Interpolation", true, "Generates infinite intermediate wavetable waveforms via differentiable interpolation"))
    );
    descriptors.insert("NeuralDifferentialWavetableLatentOscillator".to_string(), DspNodeDescriptor::new("NeuralDifferentialWavetableLatentOscillator", "Neural Differentiable Latent Wavetable Synthesizer", DspNodeCategory::NeuralAi, "Continuous differentiable wavetable synthesizer driven by deep neural autoencoder latent vectors with real-time gradient descent timbre matching and anti-aliased phase accumulation")
        .with_param(DspParamSchema::knob("latent_dimension_z1_morph", "Primary Neural Latent Vector (Z1 Morph)", -3.0, 3.0, 0.0, "z1", MacroRole::Tone, "Primary neural timbre latent dimension representing spectral tilt and brightness"))
        .with_param(DspParamSchema::knob("latent_dimension_z2_complexity", "Secondary Neural Latent Vector (Z2 Complexity)", -3.0, 3.0, 0.5, "z2", MacroRole::Character, "Secondary latent dimension controlling harmonic density and formant distribution"))
        .with_param(DspParamSchema::knob("differentiable_learning_rate", "Real-Time Gradient Adaptation Rate", 0.001, 0.1, 0.015, "lr", MacroRole::Character, "Gradient step adaptation rate for real-time target spectrum following"))
        .with_param(DspParamSchema::knob("neural_spectral_loss_penalty", "Multi-Scale Spectral Loss Regularizer", 0.0, 2.0, 0.65, "loss", MacroRole::Tone, "Multi-scale spectral convergence regularizer preventing unvoiced phase hash"))
        .with_param(DspParamSchema::knob("wavetable_anti_aliasing_oversample", "Internal Neural Oversampling Factor", 1.0, 8.0, 4.0, "x", MacroRole::Punch, "Internal neural evaluation oversampling factor eliminating Nyquist foldover"))
        .with_param(DspParamSchema::toggle("enable_continuous_latent_interpolation", "Continuous Cubic Spline Latent Interpolation", true, "Smooth cubic spline interpolation between discrete neural latent vectors"))
    );
    descriptors.insert("NeuralAdaptiveSpectralDeReverberator".to_string(), DspNodeDescriptor::new("NeuralAdaptiveSpectralDeReverberator", "Neural Adaptive Spectral De-Reverberator", DspNodeCategory::NeuralAi, "Deep recurrent spectral enhancement network isolating and attenuating late reverberant energy tails and room reflections while preserving direct-path transient coherence")
        .with_param(DspParamSchema::knob("dereverb_suppression_intensity_db", "Late Reverberation Suppression Depth", 0.0, 36.0, 18.0, "dB", MacroRole::Space, "Depth of late reverberant tail suppression and room room-tone attenuation"))
        .with_param(DspParamSchema::knob("room_acoustic_coherence_threshold", "Direct-Path Acoustic Coherence Threshold", 0.1, 0.95, 0.62, "coh", MacroRole::Character, "Threshold differentiating direct-path wavefronts from diffuse reverberation"))
        .with_param(DspParamSchema::knob("transient_preservation_weight", "Attack Wavefront Transient Preservation", 0.0, 1.0, 0.88, "trans", MacroRole::Punch, "Protects percussive and vocal speech onset transients from attenuation"))
        .with_param(DspParamSchema::knob("spectral_subband_tracking_speed_ms", "Recursive Power Spectrum Tracking Speed", 5.0, 100.0, 24.0, "ms", MacroRole::Tone, "Adaptation speed of recursive neural power spectral density estimator"))
        .with_param(DspParamSchema::knob("diffuse_soundfield_coloration_trim_db", "Post-Enhancement Air & Tone Trim", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Post-enhancement EQ compensating for perceived HF dulling"))
        .with_param(DspParamSchema::toggle("enable_zero_phase_residual_synthesis", "Zero-Phase Room Breath Residual Synthesis", true, "Synthesizes natural low-level room breath to prevent unnatural noise gating silence"))
    );
    descriptors.insert("NeuralRecurrentReservoirComputingSynthesizer".to_string(), DspNodeDescriptor::new("NeuralRecurrentReservoirComputingSynthesizer", "Recurrent Reservoir Computing Timbre Synthesizer", DspNodeCategory::NeuralAi, "Echo state recurrent neural network reservoir oscillator generating evolving nonlinear spectral states and chaotic attractors")
        .with_param(DspParamSchema::knob("reservoir_size", "Reservoir Neuron Node Count", 16.0, 512.0, 128.0, "nodes", MacroRole::Character, "Dimensions of the recurrent echo-state neural reservoir network"))
        .with_param(DspParamSchema::knob("spectral_radius", "Matrix Spectral Radius", 0.1, 1.5, 0.95, "rad", MacroRole::Character, "Reservoir weight matrix spectral radius governing chaos transition"))
        .with_param(DspParamSchema::knob("leak_rate", "Neuron State Leaky Rate", 0.01, 1.0, 0.35, "rate", MacroRole::Character, "Leaky integrator temporal updating rate of internal neurons"))
        .with_param(DspParamSchema::knob("input_coupling", "Input Modulation Drive", 0.0, 1.0, 0.60, "drive", MacroRole::Punch, "Input audio and control injection drive into the reservoir pool"))
        .with_param(DspParamSchema::knob("nonlinear_activation", "Tanh Nonlinear Activation Gain", 0.0, 1.0, 0.80, "sat", MacroRole::Character, "Steepness of hyperbolic tangent activation function"))
        .with_param(DspParamSchema::knob("timbre_drift", "Autonomous Timbre Evolution Rate", 0.0, 1.0, 0.25, "drift", MacroRole::Tone, "Rate of slow internal weight drift creating perpetual motion"))
    );
    descriptors.insert("NeuralSubharmonicBassRestorationSynthesizer".to_string(), DspNodeDescriptor::new("NeuralSubharmonicBassRestorationSynthesizer", "Neural Subharmonic Bass Restoration Engine", DspNodeCategory::NeuralAi, "Deep neural network analyzing input low-mid harmonics to synthesize missing sub-octave fundamental frequencies with zero latency")
        .with_param(DspParamSchema::knob("sub_mix", "Synthesized Subharmonic Mix", 0.0, 1.0, 0.50, "wet", MacroRole::Punch, "Wet/dry level blend of generated subharmonic fundamentals"))
        .with_param(DspParamSchema::knob("sub_octave_depth", "Sub-Octave Generation Intensity", 0.0, 1.0, 0.80, "depth", MacroRole::Punch, "Deep sub-octave -1 and -2 reinforcement intensity"))
        .with_param(DspParamSchema::knob("harmonic_tracker", "Tracking Frequency Ceiling", 20.0, 250.0, 80.0, "Hz", MacroRole::Tone, "Upper boundary for neural tracking of bass harmonic profiles"))
        .with_param(DspParamSchema::knob("transient_preservation", "Transient Snap Preservation", 0.0, 1.0, 0.75, "snap", MacroRole::Punch, "Preserves kick drum attack transients while generating smooth sustain"))
        .with_param(DspParamSchema::knob("sub_saturation", "Sub-Bass Tape Saturation", 0.0, 1.0, 0.20, "sat", MacroRole::Character, "Warm analog tape saturation applied to the synthesized low frequencies"))
        .with_param(DspParamSchema::knob("low_shelf_cutoff", "Sub Low-Shelf Anchor Cutoff", 20.0, 120.0, 50.0, "Hz", MacroRole::Tone, "Anchoring frequency shelf for low-end phase alignment"))
    );
    descriptors.insert("NeuralDiffusionLatentHarmonicExciter".to_string(), DspNodeDescriptor::new("NeuralDiffusionLatentHarmonicExciter", "Neural Diffusion Latent Harmonic Exciter", DspNodeCategory::NeuralAi, "Latent diffusion generative network modeling multi-band psychoacoustic harmonic excitation and high-frequency air recovery")
        .with_param(DspParamSchema::knob("diffusion_steps", "Denoising Diffusion Inference Steps", 1.0, 20.0, 4.0, "steps", MacroRole::Character, "Number of denoising diffusion iterations refining psychoacoustic harmonics"))
        .with_param(DspParamSchema::knob("harmonic_air_generation", "High-Frequency Air Synthesis Depth", 0.0, 1.0, 0.65, "air", MacroRole::Tone, "Generates synthetic shimmering air above 12kHz based on mid harmonic content"))
        .with_param(DspParamSchema::knob("guidance_scale", "Latent Classifier-Free Guidance", 0.5, 5.0, 1.8, "scale", MacroRole::Character, "Conditioning strength adhering to the input signal's original timbre envelope"))
        .with_param(DspParamSchema::knob("sub_band_split_freq", "Excitation Crossover Frequency", 2000.0, 14000.0, 6500.0, "Hz", MacroRole::Tone, "Lower frequency cutoff boundary where neural harmonic generation begins"))
        .with_param(DspParamSchema::knob("latent_noise_seed", "Stochastic Micro-Texture Seed", 0.0, 1.0, 0.42, "seed", MacroRole::Character, "Latent noise injection generating microscopic acoustic texture variations"))
        .with_param(DspParamSchema::knob("transient_spectral_focus", "Transient Dynamic Focus Weight", 0.0, 1.0, 0.78, "focus", MacroRole::Punch, "Prioritizes excitation on percussive attack transients to preserve clarity"))
    );
    descriptors.insert("NeuralPhaseUnwrappingStereoSynthesizer".to_string(), DspNodeDescriptor::new("NeuralPhaseUnwrappingStereoSynthesizer", "Neural Phase Unwrapping Stereo Synthesizer", DspNodeCategory::NeuralAi, "Deep neural complex spectrogram transformer unwrapping wrapped phase discontinuities and synthesizing wide, mono-compatible binaural spatial fields")
        .with_param(DspParamSchema::knob("phase_unwrapping_depth", "Complex Phase Unwrapping Depth", 0.0, 1.0, 0.80, "depth", MacroRole::Space, "Deep learning reconstruction depth for unrolling 2pi phase discontinuities"))
        .with_param(DspParamSchema::knob("complex_coherence", "Binaural Spectral Coherence", 0.0, 1.0, 0.75, "coh", MacroRole::Character, "Preserves inter-channel coherence to eliminate comb-filtering in mono folddown"))
        .with_param(DspParamSchema::knob("stereo_spread_angle", "Synthesized Stereo Field Width", 0.0, 2.0, 1.25, "width", MacroRole::Space, "Angular stereophonic panorama expansion angle"))
        .with_param(DspParamSchema::knob("phase_jitter_reduction", "Temporal Phase Jitter Smoothing", 0.0, 1.0, 0.45, "smooth", MacroRole::Tone, "Removes high-frequency phase flutter artifacts in sustained notes"))
        .with_param(DspParamSchema::knob("low_frequency_mono_guard", "Bass Mono Safeguard Crossover", 40.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Forces phase alignment below crossover frequency to protect low-end punch"))
        .with_param(DspParamSchema::knob("neural_confidence_threshold", "Model Inference Confidence Gate", 0.1, 0.99, 0.70, "gate", MacroRole::Punch, "Confidence threshold required before applying artificial stereo expansion"))
    );
    descriptors.insert("NeuralAutoregressivePredictiveSampleSynthesizer".to_string(), DspNodeDescriptor::new("NeuralAutoregressivePredictiveSampleSynthesizer", "Neural Autoregressive Predictive Sample Synthesizer", DspNodeCategory::NeuralAi, "Causal autoregressive WaveNet neural network predicting continuous sample probability distributions with latent conditioning vectors and real-time inference")
        .with_param(DspParamSchema::knob("conditioning_latent_dim", "Latent Style Embedding Vector", 0.0, 1.0, 0.50, "vec", MacroRole::Tone, "Continuous latent space vector interpolating synthesis style and acoustic timbre"))
        .with_param(DspParamSchema::knob("temperature_sampling", "Autoregressive Sampling Temperature", 0.1, 2.0, 0.85, "temp", MacroRole::Character, "Softmax temperature governing generative entropy: lower = deterministic, higher = wild noise"))
        .with_param(DspParamSchema::knob("causal_dilated_receptive_field", "Dilated Convolution Stack Span", 64.0, 2048.0, 512.0, "spl", MacroRole::Space, "Temporal receptive field depth capturing long-range acoustic dependencies"))
        .with_param(DspParamSchema::knob("quantization_companding", "Mu-Law Nonlinear Amplitude Compander", 0.0, 1.0, 0.70, "comp", MacroRole::Punch, "Mu-law logarithmic dynamics companding matching acoustic perceptual sensitivity"))
        .with_param(DspParamSchema::knob("top_p_nucleus_filter", "Nucleus Probability Mass Threshold (Top-P)", 0.5, 1.0, 0.92, "prob", MacroRole::Tone, "Top-P nucleus probability truncation eliminating low-probability artifact tails"))
        .with_param(DspParamSchema::knob("context_history_blend", "Prior Audio History Recurrence Blend", 0.0, 1.0, 0.60, "ratio", MacroRole::Punch, "Recurrent feed-forward blend ratio from past audio sample memory buffers"))
    );
    descriptors.insert("NeuralSpectralCrossSynthesisVocoder".to_string(), DspNodeDescriptor::new("NeuralSpectralCrossSynthesisVocoder", "Neural Spectral Cross-Synthesis Vocoder", DspNodeCategory::NeuralAi, "Deep learned multi-band phase vocoder performing dynamic spectral envelope transfer and harmonic formant extraction between carrier and modulator audio streams")
        .with_param(DspParamSchema::knob("spectral_envelope_transfer", "Neural Formant Transfer Depth", 0.0, 1.0, 0.80, "depth", MacroRole::Tone, "Deep neural transfer factor imprinting modulator spectral envelope onto carrier"))
        .with_param(DspParamSchema::knob("latent_formant_shift", "Vocal Tract Latent Formant Shift", -24.0, 24.0, 0.0, "semi", MacroRole::Tone, "Continuous vocal tract formant scale shift without altering fundamental pitch"))
        .with_param(DspParamSchema::knob("phase_coherence_recovery", "Neural Phase Coherence Preservation", 0.0, 1.0, 0.90, "coher", MacroRole::Character, "Multi-frame neural phase unwrapping preserving natural transient attack punch"))
        .with_param(DspParamSchema::knob("cross_synthesis_dry_wet", "Cross-Synthesis Modulation Mix", 0.0, 1.0, 1.0, "mix", MacroRole::Punch, "Balance between raw carrier input and fully modulated cross-synthesis output"))
        .with_param(DspParamSchema::knob("unvoiced_consonant_injection", "Unvoiced Consonant Sibilance Blend", 0.0, 1.0, 0.35, "sib", MacroRole::Punch, "Direct unvoiced transient noise injector enhancing speech intelligibility"))
        .with_param(DspParamSchema::knob("pitch_tracking_confidence", "Autocorrelation Pitch Tracker Gate", 0.1, 1.0, 0.75, "conf", MacroRole::Tone, "Carrier pitch tracking certainty threshold for harmonic alignment"))
    );
}

pub(in crate::dsp_node_ui) fn create_node_ui(
    target_name: &str,
) -> Option<Box<dyn DspNodeUi>> {
    let node: Box<dyn DspNodeUi> = match target_name {
        "NeuralWavetable" => Box::new(
            GenericDspNodeUi::new("NeuralWavetable", "Latent AI Neural Wavetable Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("latent_x", "Latent Timbre X", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("latent_y", "Latent Timbre Y", -3.0, 3.0, 0.0, "", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("spectral_tilt", "Spectral Tilt", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("morph_speed", "Morph Slew Rate", 0.1, 10.0, 1.0, "Hz", (0, 230, 118)))
        ),
        "SingingSynthesisNode" => Box::new(
            GenericDspNodeUi::new("SingingSynthesisNode", "Differentiable Neural Phoneme Singing Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_log("f0_pitch", "Vocal Fundamental F0", 50.0, 1000.0, 220.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("formant_scale", "Formant Scale", 0.5, 2.0, 1.0, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("breathiness", "Glottal Breathiness", 0.0, 1.0, 0.15, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("vibrato_depth", "Vocal Vibrato Depth", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_enum("phoneme", "Current Phoneme", vec!["/a/".into(), "/e/".into(), "/i/".into(), "/o/".into(), "/u/".into(), "/m/".into(), "/n/".into()], 0, (255, 61, 113)))
        ),
        "NeuralAudioRepairNode" => Box::new(
            GenericDspNodeUi::new("NeuralAudioRepairNode", "AI Spectral Audio Inpainting & De-Clicking Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("noise_reduction", "Spectral De-Noise Depth", 0.0, 1.0, 0.6, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("inpaint_strength", "Inpaint Reconstruction", 0.0, 1.0, 0.8, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_bool("transient_protect", "Protect Fast Transients", true, (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("blend", "Dry / Wet Repair Blend", 0.0, 1.0, 1.0, "%", (255, 171, 0)))
        ),
        "NeuralTransientShaperNode" => Box::new(
            GenericDspNodeUi::new("NeuralTransientShaperNode", "Deep Neural Network Transient & Sustain Sculptor", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("attack_boost", "Transient Attack Gain", -12.0, 12.0, 3.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("sustain_tail", "Sustain Tail Gain", -12.0, 12.0, -1.5, "dB", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("neural_sensitivity", "AI Detection Sensitivity", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer", 0.0, 20.0, 5.0, "ms", (153, 102, 255)))
        ),
        "NeuralBassGeneratorNode" => Box::new(
            GenericDspNodeUi::new("NeuralBassGeneratorNode", "Subharmonic Low-End Neural Bass Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("sub_octave_gain", "Subharmonic Level", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("harmonic_density", "Generated Harmonics", 1, 8, 3, "bands", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("saturation_color", "Saturation Flavor", vec!["Warm Tube".into(), "Solid State".into(), "Tape Flux".into()], 0, (255, 61, 113)))
                .with_param(DspParamDescriptor::new_bool("mono_lows", "Mono Bass Summing (<120Hz)", true, (0, 230, 118)))
        ),
        "NeuralHarmonicExciterNode" => Box::new(
            GenericDspNodeUi::new("NeuralHarmonicExciterNode", "AI Spectral Sheen & High-Frequency Air Polish", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("air_sheen", "Ultra-High Air Presence", 0.0, 12.0, 4.0, "dB", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("warmth_body", "Low-Mid Harmonic Warmth", 0.0, 12.0, 2.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_enum("ai_profile", "Exciter Neural Profile", vec!["Modern Polish".into(), "Vintage Tube".into(), "Transformer Color".into(), "Silk Top".into()], 0, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("wet_mix", "Harmonic Blend Level", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
        ),
        "BciEegDecoderNode" => Box::new(
            GenericDspNodeUi::new("BciEegDecoderNode", "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("sampling_rate_hz", "EEG Electrode Sample Rate", 128.0, 1024.0, 256.0, "Hz", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("alpha_band_gain", "Alpha Power Band (8-12Hz)", 0.0, 2.0, 1.0, "x", (0, 230, 118)))
                .with_param(DspParamDescriptor::new_linear("theta_band_gain", "Theta Power Band (4-8Hz)", 0.0, 2.0, 0.8, "x", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_bool("artifact_rejection_filter", "Ocular Blink Artifact Filter", true, (255, 171, 0)))
        ),
        "NeuroAffectiveEmotionalStateAnalyzer" => Box::new(
            GenericDspNodeUi::new("NeuroAffectiveEmotionalStateAnalyzer", "Russell Circumplex Valence & Arousal Emotion Analyzer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("valence_score", "Musical Valence (Pleasantness)", -1.0, 1.0, 0.55, "score", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("arousal_intensity", "Musical Arousal (Energy)", 0.0, 1.0, 0.65, "score", (255, 61, 113)))
                .with_param(DspParamDescriptor::new_linear("emotional_smoothing_window", "Affective Slew Smoothing", 0.1, 10.0, 2.0, "s", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("harmony_feedback_gain", "Emotional Harmony Influence", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "NeuralImpulseResponseSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralImpulseResponseSynthesizer", "Latent Space Acoustic Impulse Response Generator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("latent_room_dimension_x", "Latent Room Geometry X", -3.0, 3.0, 0.5, "", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("latent_absorption_y", "Latent Wall Material Y", -3.0, 3.0, -0.2, "", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("early_reflection_decay", "Early Diffusion Slew", 0.1, 5.0, 1.5, "s", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_int("synthesis_quality", "Neural Inference Latent Depth", 1, 4, 3, "steps", (0, 230, 118)))
        ),
        "SubcorticalBrainstemPitchTracker" => Box::new(
            GenericDspNodeUi::new("SubcorticalBrainstemPitchTracker", "Auditory Nerve Frequency-Following Response Pitch Tracker", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("ffr_sensitivity", "Brainstem FFR Sensitivity", 0.1, 2.0, 1.0, "x", (0, 184, 217)))
                .with_param(DspParamDescriptor::new_int("cochlear_filterbank_channels", "Cochlear Gammatone Channels", 16, 128, 64, "ch", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_log("fundamental_f0_estimate_hz", "Detected Pitch F0", 30.0, 2000.0, 220.0, "Hz", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("tracking_slew_ms", "Pitch Tracking Response", 1.0, 100.0, 15.0, "ms", (0, 230, 118)))
        ),
        "MentalImageryPatternClassifier" => Box::new(
            GenericDspNodeUi::new("MentalImageryPatternClassifier", "Motor/Auditory Imagery Neural Intention Classifier", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("classifier_confidence_threshold", "Trigger Confidence Floor", 0.5, 0.99, 0.85, "%", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_int("pattern_bank_index", "Intention Pattern Slot", 0, 15, 0, "slot", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("trigger_event_latency_ms", "Pattern Recognition Latency", 5.0, 200.0, 35.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("adaptation_rate", "Online Neural Plasticity Slew", 0.0, 1.0, 0.1, "%", (0, 230, 118)))
        ),
        "BiometricHrvTempoSync" => Box::new(
            GenericDspNodeUi::new("BiometricHrvTempoSync", "Photoplethysmography Heart Rate Variability Tempo Sync", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("target_heart_coherence", "Target HRV Coherence", 0.0, 1.0, 0.75, "%", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("bpm_sync_multiplier", "BPM Metric Ratio", 0.5, 4.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("hrv_smoothing_interval_s", "Heart Rate Rolling Average", 5.0, 60.0, 20.0, "s", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("rhythm_entrainment_depth", "Tempo Entrainment Depth", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "NeuroCognitiveFatigueDetector" => Box::new(
            GenericDspNodeUi::new("NeuroCognitiveFatigueDetector", "Listening Fatigue & Auditory Habituation Detector", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("listening_duration_minutes", "Session Exposure Time", 0.0, 480.0, 65.0, "min", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_brightness_fatigue_index", "Harshness Fatigue Metric", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("auto_soft_filter_db", "Fatigue Relief Soft High Cut", -6.0, 0.0, 0.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("habituation_warning", "Break Reminder Notification", true, (0, 230, 118)))
        ),
        "NeuroAestheticHarmonyScorer" => Box::new(
            GenericDspNodeUi::new("NeuroAestheticHarmonyScorer", "Computational Neuro-Aesthetic Musical Harmony Scorer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("harmony_concordance_weight", "Concordance Sensation Bias", 0.0, 1.0, 0.8, "%", (255, 51, 153)))
                .with_param(DspParamDescriptor::new_linear("voice_leading_fluency", "Voice Leading Smoothness", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("tension_resolution_score", "Tension Cycle Resolution", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("scale_fit_percent", "Tonal Scale Alignment", 0.0, 100.0, 94.0, "%", (0, 230, 118)))
        ),
        "SubsensoryTactileHapticTransducer" => Box::new(
            GenericDspNodeUi::new("SubsensoryTactileHapticTransducer", "Vibrotactile Sub-Bass Haptic Transducer Driver", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_log("haptic_frequency_hz", "Transducer Resonance Pitch", 15.0, 120.0, 40.0, "Hz", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("transducer_intensity", "Haptic Shaker Force", 0.0, 1.0, 0.75, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("subharmonic_synthesis_oct", "Sub-Harmonic Generation", vec!["None (Direct)".into(), "-1 Octave Sub".into(), "-2 Octave Infrasound".into()], 1, (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("rumble_envelope_decay", "Haptic Rumble Decay", 0.05, 2.0, 0.4, "s", (0, 230, 118)))
        ),
        "NeuralChoirFormantNode" => Box::new(
            GenericDspNodeUi::new("NeuralChoirFormantNode", "Multi-Voice Neural Formant Choir & Vowel Morph Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_int("voice_count", "Ensemble Choir Voices", 4, 32, 16, "voices", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("vowel_morph_coordinate", "Vowel Space Morph (A-E-I-O-U)", 0.0, 4.0, 1.5, "vowel", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("formant_spread_cents", "Formant Detuning Width", 0.0, 50.0, 15.0, "cents", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("choir_detune_cents", "Choral Micro-Pitch Detune", 0.0, 40.0, 8.0, "cents", (0, 230, 118)))
        ),
        "NeuralDereverbNode" => Box::new(
            GenericDspNodeUi::new("NeuralDereverbNode", "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("dereverb_intensity", "De-Reverberation Depth", 0.0, 100.0, 65.0, "%", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("room_size_estimate_m", "Estimated Chamber Size", 2.0, 50.0, 12.0, "m", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("early_reflection_suppress_db", "Early Reflection Attenuation", -24.0, 0.0, -10.0, "dB", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("tail_dry_boost_db", "Direct Signal Presence Boost", 0.0, 12.0, 2.5, "dB", (0, 230, 118)))
        ),
        "NeuralInpaintNode" => Box::new(
            GenericDspNodeUi::new("NeuralInpaintNode", "Neural Audio Inpainting & Dropout Reconstruction Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("dropout_threshold_ms", "Dropout Mask Detection Window", 1.0, 100.0, 20.0, "ms", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("reconstruction_depth", "AI Inpainting Strength", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("context_window_ms", "Surrounding Context Frame", 50, 1000, 250, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("spectral_continuity_protect", "Preserve Spectral Phase", true, (0, 230, 118)))
        ),
        "NeuralPhonemeNode" => Box::new(
            GenericDspNodeUi::new("NeuralPhonemeNode", "International Phonetic Alphabet Differentiable Phoneme Synth", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_int("ipa_symbol_index", "IPA Phoneme Symbol Index", 0, 64, 12, "ipa", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("nasalization_index", "Velum Nasal Coupling", 0.0, 1.0, 0.2, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("voicing_glottal_mix", "Voiced Glottal Mix", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("tongue_arch_position", "Tongue Constriction Place", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
        ),
        "NeuralRadianceNode" => Box::new(
            GenericDspNodeUi::new("NeuralRadianceNode", "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("listener_pos_xyz", "Virtual Listener X Position", -10.0, 10.0, 0.0, "m", (101, 84, 171)))
                .with_param(DspParamDescriptor::new_linear("emitter_pos_xyz", "Virtual Emitter Y Position", -10.0, 10.0, 2.5, "m", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_int("nerf_rendering_samples", "Ray Sample Marching Steps", 32, 256, 128, "rays", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("spatial_interpolation_slew", "Acoustic Field Slew Rate", 0.01, 1.0, 0.15, "s", (0, 230, 118)))
        ),
        "NeuralSpeechToSingingNode" => Box::new(
            GenericDspNodeUi::new("NeuralSpeechToSingingNode", "Speech-to-Singing AI Prosody & Intonation Transformer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_bool("target_pitch_quantize", "Snap Speech to Scale Notes", true, (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("vibrato_infusion_depth", "Singing Vibrato Infusion", 0.0, 1.0, 0.5, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("note_hold_extension_ms", "Vowel Duration Extension", 50.0, 1000.0, 350.0, "ms", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_enum("singing_style_preset", "AI Singing Style", vec!["Bel Canto Opera".into(), "Pop Vibrato Smooth".into(), "R&B Micro-Riff".into(), "Choir Polyphony".into()], 1, (0, 230, 118)))
        ),
        "NeuralTimbreMorphNode" => Box::new(
            GenericDspNodeUi::new("NeuralTimbreMorphNode", "Non-Linear Latent Timbre Vector Interpolator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("timbre_vector_x", "Latent Timbre Axis X (Warm/Bright)", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("timbre_vector_y", "Latent Timbre Axis Y (Wood/Metal)", -3.0, 3.0, 0.0, "", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("timbre_vector_z", "Latent Timbre Axis Z (Acoustic/Synth)", -3.0, 3.0, 0.0, "", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("morph_trajectory_speed_hz", "LFO Timbre Orbit Speed", 0.05, 10.0, 0.5, "Hz", (0, 230, 118)))
        ),
        "NeuralTimbreNode" => Box::new(
            GenericDspNodeUi::new("NeuralTimbreNode", "Latent Timbre Space Feature Extractor & Profiler", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("spectral_centroid_weight", "Spectral Brightness Weight", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("flux_roughness_weight", "Spectral Roughness Weight", 0.0, 1.0, 0.4, "%", (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("harmonic_entropy_weight", "Harmonic Order vs Noise", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("timbre_snapshot_trigger", "Capture Timbre Vector", false, (0, 230, 118)))
        ),
        "NeuralVocalStylizerNode" => Box::new(
            GenericDspNodeUi::new("NeuralVocalStylizerNode", "Vocal Timbre Style Transfer & Character Transformer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_int("source_vocal_id", "Source Vocal Track ID", 0, 32, 1, "trk", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_enum("target_style_preset", "AI Timbre Target Profile", vec!["Vintage Ribbon Mic Warmth".into(), "Modern Hyper-Pop Crisp".into(), "Analog Tube Intimate".into(), "Classic 80s Broadcast".into()], 0, (153, 102, 255)))
                .with_param(DspParamDescriptor::new_linear("formant_character_scale", "Formant Character Intensity", 0.5, 2.0, 1.0, "x", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("style_transfer_blend", "Style Transfer Dry/Wet Blend", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
        ),
        "NeuralVocoderMorphNode" => Box::new(
            GenericDspNodeUi::new("NeuralVocoderMorphNode", "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_enum("vocoder_architecture", "Neural Vocoder Backbone", vec!["BigVGAN High-Fidelity".into(), "HiFi-GAN V1".into(), "Vocos Ultra-Fast".into(), "WaveGlow Multi-Scale".into()], 0, (0, 184, 217)))
                .with_param(DspParamDescriptor::new_linear("latent_dimension_scale", "Latent Manifold Scale", 0.1, 3.0, 1.0, "x", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("periodic_component_mix", "Harmonic Periodic Component", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_linear("aperiodic_noise_ratio", "Aperiodic Breath Noise Level", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
        ),
        "NeuralQuantumAnnealerTopologicalSort" => Box::new(
            GenericDspNodeUi::new("NeuralQuantumAnnealerTopologicalSort", "Simulated Quantum Annealing Audio Graph Optimizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("annealing_temperature", "Annealing Temperature (T)", 0.01, 10.0, 1.0, "T", (180, 195, 215)))
                .with_param(DspParamDescriptor::new_linear("transverse_field_gamma", "Transverse Quantum Field", 0.0, 5.0, 1.5, "gamma", (0, 229, 255)))
                .with_param(DspParamDescriptor::new_linear("graph_latency_minimize_weight", "Latency Minimization Weight", 0.0, 1.0, 0.9, "%", (255, 171, 0)))
                .with_param(DspParamDescriptor::new_bool("optimization_step_trigger", "Trigger Topological Sort", false, (0, 230, 118)))
        ),
        "DemucsV4Separator" => Box::new(
            GenericDspNodeUi::new("DemucsV4Separator", "Demucs v4 Hybrid Neural Stem Separator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_enum("stems_mask", "Stem Mode", vec!["6-Stem Full".into(), "4-Stem Standard".into(), "Vocal Isolated".into(), "Instrumental".into()], 0, (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("overlap", "FFT Overlap", 0.1, 0.9, 0.25, "%", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Separation Sensitivity", 0.1, 2.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("residual_bleed", "Residual Bleed", -48.0, 0.0, -24.0, "dB", (249, 115, 22)))
        ),
        "NeuralRoomAcousticMatcher" => Box::new(
            GenericDspNodeUi::new("NeuralRoomAcousticMatcher", "Neural Room Acoustic Matcher", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("rt60_decay", "RT60 Decay Time", 0.1, 8.0, 1.2, "s", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("brightness", "Brightness Ratio", 0.2, 3.0, 1.15, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_log("room_volume", "Room Volume", 10.0, 5000.0, 120.0, "m³", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("early_reflections", "Early Reflections", 0.0, 1.0, 0.65, "%", (236, 72, 153)))
        ),
        "CrepePitchTracker" => Box::new(
            GenericDspNodeUi::new("CrepePitchTracker", "CREPE Deep Neural Polyphonic Pitch & Frequency Estimator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("confidence_thresh", "Confidence Cutoff", 0.1, 0.99, 0.75, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("slew_smoothing", "Temporal Slew Smoothing", 0.0, 1.0, 0.30, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("model_size", "Neural Model Capacity", vec!["Tiny (Low CPU)".into(), "Small".into(), "Medium (Balanced)".into(), "Full (Studio Grade)".into()], 2, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("cent_tolerance", "Microtonal Quantize", 1.0, 50.0, 5.0, "cents", (16, 185, 129)))
        ),
        "NeuralAudioStyleTransferPreviewRenderer" => Box::new(
            GenericDspNodeUi::new("NeuralAudioStyleTransferPreviewRenderer", "Neural Audio Style Transfer & Timbre Morphing Preview", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("content_weight", "Content Preservation", 0.0, 1.0, 0.70, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("style_weight", "Timbre Style Imprint", 0.0, 1.0, 0.60, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("latent_steps", "Diffusion Steps", 5.0, 50.0, 20.0, "steps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_enum("fft_window", "Spectral Window", vec!["512 samples".into(), "1024 samples".into(), "2048 samples".into(), "4096 samples".into()], 1, (16, 185, 129)))
        ),
        "NeuroAffectiveAnalyzer" => Box::new(
            GenericDspNodeUi::new("NeuroAffectiveAnalyzer", "Affective Emotional State & Listener Valence/Arousal Analyzer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("valence_target", "Target Valence (Pleasure)", -1.0, 1.0, 0.0, "", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("arousal_target", "Target Arousal (Energy)", 0.0, 1.0, 0.50, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("integration_time_s", "State Integration Window", 0.5, 10.0, 2.0, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("confidence_cutoff", "Classifier Confidence", 0.5, 0.99, 0.85, "%", (16, 185, 129)))
        ),
        "MentalImageryClassifier" => Box::new(
            GenericDspNodeUi::new("MentalImageryClassifier", "EEG Mental Imagery & Motor Cortex Pattern Classifier", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("electrode_channels", "Active EEG Electrodes", 4.0, 64.0, 16.0, "ch", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_enum("spatial_filter", "Spatial Decomposition", vec!["Common Spatial Pattern (CSP)".into(), "Independent Component Analysis (ICA)".into(), "Surface Laplacian".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bandpass_low_hz", "Mu/Alpha Low Cutoff", 4.0, 12.0, 8.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("bandpass_high_hz", "Beta/Gamma High Cutoff", 20.0, 50.0, 30.0, "Hz", (16, 185, 129)))
        ),
        "NeuroAestheticScorer" => Box::new(
            GenericDspNodeUi::new("NeuroAestheticScorer", "Neuro-Aesthetic Psychoacoustic Harmony & Groove Scorer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("consonance_weight", "Harmonic Consonance", 0.0, 1.0, 0.70, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("entropy_weight", "Spectral Novelty Entropy", 0.0, 1.0, 0.50, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("groove_weight", "Syncopation Groove", 0.0, 1.0, 0.60, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("eval_rate_hz", "Score Evaluation Rate", 0.1, 5.0, 1.0, "Hz", (16, 185, 129)))
        ),
        "AiMixBalanceAnalyzer" => Box::new(
            GenericDspNodeUi::new("AiMixBalanceAnalyzer", "AI Multi-Track Frequency Masking Analyzer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Masking Detection Threshold", 0.0, 1.0, 0.50, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("spectral_bands", "Analysis Band Count", 4.0, 32.0, 16.0, "bands", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("smoothing_sec", "Temporal Smoothing Window", 0.05, 2.0, 0.30, "s", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("auto_recommend_cut", "Auto-Cut Recommended Depth", 0.0, 12.0, 3.0, "dB", (239, 68, 68)))
        ),
        "AiSongStructureDetector" => Box::new(
            GenericDspNodeUi::new("AiSongStructureDetector", "AI Song Structure & Arrangement Classifier", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("min_section_bars", "Minimum Section Length", 4.0, 32.0, 8.0, "bars", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("energy_contrast", "Chorus Energy Contrast", 0.0, 1.0, 0.65, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("tempo_stability", "Beat Grid Confidence", 0.50, 1.0, 0.95, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("auto_marker_placement", "Auto Place Markers", vec!["Disabled".into(), "Enabled".into()], 1, (16, 185, 129)))
        ),
        "AiPolyphonicChordExtractor" => Box::new(
            GenericDspNodeUi::new("AiPolyphonicChordExtractor", "Real-Time AI Polyphonic Chord Extractor", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("confidence_threshold", "Chord Detection Confidence", 0.20, 0.99, 0.70, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("bass_octave_weight", "Bass Root Note Weight", 0.0, 2.0, 1.20, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("tempo_quantize", "Quantize Grid", vec!["Quarter Note (1/4)".into(), "Eighth Note (1/8)".into(), "Sixteenth Note (1/16)".into(), "Free / Unquantized".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("velocity_scaling", "Extracted Velocity Scale", 0.1, 2.0, 1.0, "x", (245, 158, 11)))
        ),
        "NeuralIrSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralIrSynthesizer", "Deep Neural Parametric Room Impulse Generator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_log("room_volume_m3", "Target Room Acoustic Volume", 10.0, 50000.0, 600.0, "m³", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("rt60_decay_s", "Reverberation Time RT60", 0.1, 10.0, 1.8, "s", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("absorption_coeff", "Wall Material Absorption", 0.01, 0.95, 0.22, "", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("latent_seed", "Neural Diffusion Latent Seed", 0.0, 9999.0, 42.0, "", (236, 72, 153)))
        ),
        "NeuralMidiTranscriber" => Box::new(
            GenericDspNodeUi::new("NeuralMidiTranscriber", "CRNN Polyphonic Audio-to-MIDI Neural Transcriber", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("onset_threshold", "Neural Onset Detection Sensitivity", 0.05, 0.95, 0.50, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("pitch_confidence", "Minimum Pitch Confidence", 0.20, 0.99, 0.70, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("min_note_duration_ms", "Minimum Note Duration", 20.0, 500.0, 60.0, "ms", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_enum("velocity_tracking", "MIDI Velocity Estimation Mode", vec!["Dynamic Envelope".into(), "Fixed 100".into(), "Neural Velocity Estimator".into()], 2, (245, 158, 11)))
        ),
        "DrumTranscriptor" => Box::new(
            GenericDspNodeUi::new("DrumTranscriptor", "Neural Multi-Class Drum & Percussion Transcriber", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("kick_sensitivity", "Kick Drum Classification Bias", 0.0, 1.0, 0.60, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("snare_sensitivity", "Snare Drum Classification Bias", 0.0, 1.0, 0.60, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hihat_sensitivity", "Hi-Hat Classification Bias", 0.0, 1.0, 0.50, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("quantize_resolution", "MIDI Transcription Grid", vec!["Free".into(), "1/16 Note".into(), "1/16 Triplet".into(), "1/32 Note".into()], 1, (16, 185, 129)))
        ),
        "AiChordGenerator" => Box::new(
            GenericDspNodeUi::new("AiChordGenerator", "AI Voice-Leading & Harmonic Progression Solver", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("harmonic_complexity", "Progression Harmonic Complexity", 1.0, 5.0, 2.0, "lvl", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_enum("cadence_type", "Target Cadence Resolution", vec!["Authentic (V-I)".into(), "Plagal (IV-I)".into(), "Deceptive (V-vi)".into(), "Jazz ii-V-I".into(), "Modal Dorian".into()], 0, (56, 189, 248)))
                .with_param(DspParamDescriptor::new_enum("root_tonic", "Root Key Tonic", vec!["C".into(), "C#".into(), "D".into(), "D#".into(), "E".into(), "F".into(), "F#".into(), "G".into(), "G#".into(), "A".into(), "A#".into(), "B".into()], 0, (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("voice_leading_smoothness", "Voice Leading Minimal Movement", 0.0, 1.0, 0.85, "%", (99, 102, 241)))
        ),
        "AiMixAssistant" => Box::new(
            GenericDspNodeUi::new("AiMixAssistant", "Autonomous Headroom & Frequency Collision Assistant", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("target_lufs", "Integrated Target Loudness", -24.0, -8.0, -14.0, "LUFS", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("crest_factor_target", "Dynamic Crest Factor Target", 6.0, 20.0, 12.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("masking_mitigation", "Inter-Track Masking Suppression", 0.0, 1.0, 0.40, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("stereo_correlation_target", "Minimum Stereo Phase Target", 0.20, 1.0, 0.80, "", (16, 185, 129)))
        ),
        "BrainstemPitchTracker" => Box::new(
            GenericDspNodeUi::new("BrainstemPitchTracker", "Sub-Cortical Auditory Brainstem Pitch Tracking Simulator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_log("tracked_freq_hz", "Tracked Pitch Reference", 20.0, 2000.0, 440.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("window_size", "Analysis Window Size", 256.0, 4096.0, 1024.0, "smp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sensitivity", "Neuronal Threshold", 0.01, 1.0, 0.20, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("auditory_nerve_profile", "Synaptic Response Profile", vec!["Auditory Nerve Fiber A".into(), "High-Spontaneous Rate".into(), "Low-Spontaneous Rate".into(), "Cochlear Nucleus Bushy Cell".into()], 0, (99, 102, 241)))
        ),
        "NeuralQuantumAnnealer" => Box::new(
            GenericDspNodeUi::new("NeuralQuantumAnnealer", "Neural Quantum Annealer Topological Sort", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("annealing_temp", "Quantum Annealing Temperature", 0.01, 10.0, 1.0, "K", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transverse_field", "Transverse Magnetic Field Gamma", 0.1, 5.0, 2.0, "B", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("quantum_fluctuation", "Vacuum Fluctuations", 0.0, 1.0, 0.25, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("cooling_rate", "Annealing Schedule Cooling Rate", 0.80, 0.999, 0.985, "", (16, 185, 129)))
        ),
        "AudioTagger" => Box::new(
            GenericDspNodeUi::new("AudioTagger", "Deep Neural Audio Tagging & Timbre Classifier", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("classification_threshold", "Acceptance Confidence", 0.1, 0.99, 0.65, "prob", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("analysis_window_sec", "Analysis Window Length", 0.5, 10.0, 3.0, "s", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("top_k_candidates", "Top-K Candidates", 1.0, 10.0, 5.0, "tags", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("auto_categorize_asset", "Auto-Tag New Samples", true, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("realtime_inference", "Background Inference", false, (16, 185, 129)))
        ),
        "NeuralWavetableInterpolator" => Box::new(
            GenericDspNodeUi::new("NeuralWavetableInterpolator", "Latent-Space Neural Wavetable Continuous Interpolator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("latent_dim_x", "Latent Axis X (Bright)", -3.0, 3.0, 0.0, "z1", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("latent_dim_y", "Latent Axis Y (Contour)", -3.0, 3.0, 0.0, "z2", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_flatness_bias", "Spectral Flatness Bias", 0.0, 1.0, 0.3, "bias", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("decoder_temperature", "Decoder Temperature", 0.1, 2.0, 1.0, "temp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("bandlimited_resynthesis", "Bandlimited Resynthesis", true, (16, 185, 129)))
        ),
        "NamWaveNetEngine" => Box::new(
            GenericDspNodeUi::new("NamWaveNetEngine", "WaveNet Dilated Convolution Neural Amp Model Inference Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("input_gain_db", "Pre-Amp Input Drive", -24.0, 24.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("dilated_layers_count", "Dilated Conv Layers", 4.0, 16.0, 10.0, "layers", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("hidden_channels", "Residual Channels", 8.0, 32.0, 16.0, "channels", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("output_level_db", "Output Normalization", -24.0, 12.0, 0.0, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("simd_vectorization", "SIMD Tensor Acceleration", true, (16, 185, 129)))
        ),
        "FederatedMixLearner" => Box::new(
            GenericDspNodeUi::new("FederatedMixLearner", "Privacy-Preserving Federated Mix Model Learner HUD", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("local_epochs_per_round", "Local Training Epochs Per Round", 1.0, 50.0, 5.0, "epochs", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("differential_privacy_epsilon", "Differential Privacy Noise Budget Epsilon", 0.1, 10.0, 1.0, "eps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("gradient_clipping_norm", "Gradient L2 Clipping Norm", 0.1, 5.0, 1.0, "norm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_enum("federated_aggregation_strategy", "Federated Weight Aggregation Strategy", vec!["FedAvg (Standard Averaging)".into(), "FedProx (Proximal Regularization)".into(), "FedAdam (Adaptive Momentum)".into(), "SCAFFOLD (Control Variates)".into()], 0, (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("participate_in_federated_mesh", "Participate in Decentralized Model Mesh", true, (34, 197, 94)))
        ),
        "NeuroAffectiveState" => Box::new(
            GenericDspNodeUi::new("NeuroAffectiveState", "EEG Brainwave Emotion Valence & Cognitive State", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("focus_index", "Beta Attentional Focus Index", 0.0, 1.0, 0.65, "norm", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("valence", "Emotional Valence Pleasure/Aversion", -1.0, 1.0, 0.25, "val", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("arousal", "Physiological Autonomic Arousal", 0.0, 1.0, 0.55, "arousal", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("alpha_theta_ratio", "Zen Meditation & Relaxation Index", 0.1, 10.0, 1.8, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("adaptive_damping", "Smooth Biometric Telemetry Damping", true, (34, 197, 94)))
        ),
        "DrumClassClassification" => Box::new(
            GenericDspNodeUi::new("DrumClassClassification", "Deep Neural Drum Hit Categorization & Transient Tagging HUD", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("classified_drum_class", "Predicted Percussion Classification", 0.0, 6.0, 0.0, "class", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("prediction_confidence_pct", "Neural Classifier Inference Confidence", 0.0, 100.0, 94.5, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_sensitivity", "Onset Transient Detection Sensitivity", 0.05, 1.0, 0.45, "thresh", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("auto_tag_sample_markers", "Auto-Tag Slices with Classified Drum Name", true, (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("inference_model_precision", "Neural Model Quantization & Inference Speed", 0.0, 2.0, 0.0, "prec", (34, 197, 94)))
        ),
        "AudioStylePreset" => Box::new(
            GenericDspNodeUi::new("AudioStylePreset", "Neural AI Audio Style Transfer & Genre Timbre Preset", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("neural_style_target", "Target Timbre Style Preset", 0.0, 4.0, 0.0, "preset", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("style_transfer_strength", "Style Transfer Latent Strength", 0.0, 1.0, 0.65, "%", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("content_preservation_weight", "Content Phase & Pitch Preservation", 0.1, 1.0, 0.85, "wt", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("diffusion_denoising_steps", "Neural Inference Denoising Steps", 4.0, 50.0, 20.0, "steps", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("neural_latent_temperature", "Latent Sampling Temperature / Chaos", 0.1, 2.0, 0.90, "temp", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_realtime_gpu_acceleration", "Real-Time GPU DirectML Acceleration", true, (14, 165, 233)))
        ),
        "TargetCurve" => Box::new(
            GenericDspNodeUi::new("TargetCurve", "AI Autonomous Mastering Spectral Curve Target Profile", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("spectral_target_profile", "Target Frequency Spectrum Curve Archetype", 0.0, 4.0, 2.0, "curve", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Integrated Loudness Level", -24.0, -6.0, -14.0, "LUFS", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("true_peak_ceiling_dbfs", "True Peak Brickwall Ceiling Limiter Level", -2.0, 0.0, -0.3, "dBFS", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_matching_amount", "Target Spectral Matching EQ Intensity", 0.0, 1.0, 0.80, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dynamic_range_crest_factor_db", "Target Dynamic Range Crest Factor", 4.0, 18.0, 9.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_psychoacoustic_equal_loudness", "Match ISO 226 Equal-Loudness Contours", true, (14, 165, 233)))
        ),
        "TargetTimbre" => Box::new(
            GenericDspNodeUi::new("TargetTimbre", "Real-Time Spectral Neural Resynthesizer Voice Morpher", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("neural_target_instrument", "Neural Resynthesis Target Timbre Archetype", 0.0, 3.0, 0.0, "inst", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("neural_morph_amount", "Timbral Transformation Morphing Depth", 0.0, 1.0, 0.65, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("formant_retention_ratio", "Source Formant Acoustic Retention Factor", 0.0, 1.0, 0.50, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("latent_space_temperature", "Neural Generative Latent Temperature", 0.1, 2.0, 0.8, "temp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation_strength", "Input Transient Strike Preservation", 0.0, 1.0, 0.90, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_gpu_tensor_acceleration", "Hardware Accelerated Tensor Resynthesis", true, (14, 165, 233)))
        ),
        "NeuralTimbreTransferStyleEngine" => Box::new(
            GenericDspNodeUi::new("NeuralTimbreTransferStyleEngine", "Real-Time Neural Timbre Transfer Style Diffusion Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("timbre_transfer_blend_ratio", "Target Style Timbre Wet/Dry Blend", 0.0, 1.0, 0.85, "%", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("neural_model_latent_temperature", "Diffusion Latent Stochastic Temperature", 0.0, 1.5, 0.6, "temp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_fidelity_weight", "Fundamental Pitch Tracking Weight", 0.5, 1.0, 0.98, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("dynamic_envelope_expression_weight", "Dynamic Envelope Expression Follower", 0.0, 1.0, 0.8, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("inference_chunk_latency_ms", "Neural Inference Chunk Buffer Size", 2.0, 50.0, 10.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_harmonic_spectral_residual", "Reinject Unvoiced High-Frequency Residual", true, (14, 165, 233)))
        ),
        "NeuralSpeechProsodyCloner" => Box::new(
            GenericDspNodeUi::new("NeuralSpeechProsodyCloner", "Neural Speech Prosody & Pitch Contour Inflection Transfer Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("prosody_transfer_intensity_ratio", "Prosodic Expressive Inflection Depth", 0.0, 2.0, 1.0, "x", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("f0_pitch_contour_conform_weight", "F0 Melodic Pitch Trajectory Adherence", 0.0, 1.0, 0.85, "%", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("syllable_timing_warping_fluidity", "Dynamic Time Warping Cadence Elasticity", 0.1, 2.0, 1.0, "warp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("formant_envelope_preserve_ratio", "Singer Vocal Timbre Identity Guard", 0.0, 1.0, 0.9, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("energy_stress_dynamics_boost_db", "Emphatic Syllable Stress Boost", 0.0, 12.0, 3.0, "dB", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_creaky_voice_glottalization", "Creaky Voice & Glottal Stop Synthesis", false, (14, 165, 233)))
        ),
        "NeuralStemInterferenceRemover" => Box::new(
            GenericDspNodeUi::new("NeuralStemInterferenceRemover", "Deep Spectral Neural Stem Crosstalk Remover", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("bleed_suppression_depth_db", "Neural Crosstalk Attenuation Depth", 0.0, 40.0, 20.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("neural_model_confidence_threshold", "Stem Separation Confidence Gate", 0.1, 0.99, 0.75, "conf", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_phase_guard_sensitivity", "Primary Transients Phase Protection", 0.0, 1.0, 0.85, "%", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("frequency_focus_band_min_hz", "Surgical Neural Processing Low Cut", 100.0, 3000.0, 800.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_mask_temporal_smoothing_ms", "Continuous Mask Time Smoothing", 1.0, 60.0, 12.0, "ms", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_isolated_bleed_monitoring", "Solo Extracted Bleed Spill Signal", false, (14, 165, 233)))
        ),
        "NeuralHarmonicMainsHumRemover" => Box::new(
            GenericDspNodeUi::new("NeuralHarmonicMainsHumRemover", "Neural Harmonic Mains Hum & Buzz Remover", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("mains_fundamental_base_frequency", "Base Electrical Grid Standard (50/60Hz)", 48.0, 62.0, 60.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("hum_suppression_depth_db", "Harmonic Hum Attenuation Depth", 0.0, 60.0, 24.0, "dB", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("harmonic_series_tracking_limit", "Upper Harmonic Buzz Overtone Limit", 4.0, 40.0, 20.0, "harm", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("neural_adaptation_speed_ms", "Neural Drift Tracking Adaptation Rate", 20.0, 1000.0, 120.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("low_end_tonal_guard_retention", "Sub-Bass Musical Content Guard", 0.0, 1.0, 0.85, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_difference_solo_monitoring", "Solo Extracted Ground Buzz Signal", false, (14, 165, 233)))
        ),
        "NeuralHumAndHarmonicsDeclicker" => Box::new(
            GenericDspNodeUi::new("NeuralHumAndHarmonicsDeclicker", "Deep Neural Hum & Sub-Harmonic De-Clicker", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("mains_fundamental_frequency_hz", "Mains Hum Fundamental (50/60 Hz)", 45.0, 65.0, 50.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_tracking_depth", "Tracked Harmonics Filter Count", 2.0, 32.0, 16.0, "count", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("neural_confidence_threshold", "Neural Separation Mask Threshold", 0.0, 1.0, 0.85, "conf", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("click_impulse_rejection_sensitivity", "Impulsive Click Detection Sensitivity", 0.0, 1.0, 0.6, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("acoustic_air_reconstruction_ratio", "Masked Timbre Interpolation Fill", 0.0, 1.0, 0.4, "%", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_bool("enable_sidechain_isolated_listen", "Solo Neutralized Noise Components", false, (14, 165, 233)))
        ),
        "BiometricPupillometricEmotionMapper" => Box::new(
            GenericDspNodeUi::new("BiometricPupillometricEmotionMapper", "Pupillometric & Heart Rate Audio Emotion Modulator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("arousal_autonomic_nervous_level", "Physiological Arousal Intensity", 0.0, 1.0, 0.5, "norm", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("valence_emotional_positivity", "Emotional Valence Positivity Factor", -1.0, 1.0, 0.2, "val", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("heart_rate_variability_ms", "HRV R-R Interval Variation", 10.0, 150.0, 65.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sensor_integration_time_constant_sec", "Biometric Smoothing Integration Time", 0.1, 10.0, 1.2, "sec", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("modulation_routing_depth_pct", "Global Synthesis Modulation Depth", 0.0, 1.0, 0.45, "%", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_circadian_drift_compensation", "Circadian Baseline Drift Compensation", true, (34, 197, 94)))
        ),
        "QuantumAnnealingHarmonicOptimizer" => Box::new(
            GenericDspNodeUi::new("QuantumAnnealingHarmonicOptimizer", "Quantum Annealing Multi-Track Spectral Optimizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("transverse_magnetic_field_gamma", "Transverse Tunneling Magnetic Field", 0.0, 10.0, 3.5, "Gamma", (217, 70, 239)))
                .with_param(DspParamDescriptor::new_linear("annealing_schedule_tau_ms", "Annealing Transition Duration", 5.0, 500.0, 80.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_collision_penalty_lambda", "Spectral Masking Collision Penalty", 0.1, 50.0, 12.0, "lambda", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("qubit_coupling_matrix_gain", "Inter-Track Cross-Talk Coupling J", -1.0, 1.0, 0.65, "J", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("energy_ground_state_convergence", "Ground State Minimum Tolerance", 0.001, 1.0, 0.05, "eps", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_adiabatic_quantum_tunneling", "Adiabatic Quantum Tunneling Solver", true, (34, 197, 94)))
        ),
        "NeuralPerceptualMaskingDenoise" => Box::new(
            GenericDspNodeUi::new("NeuralPerceptualMaskingDenoise", "Psychoacoustic Critical-Band Perceptual Spectral De-Noiser", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_masking_depth_db", "Perceptual Masking Attenuation Depth", 0.0, 48.0, 18.0, "dB", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("noise_floor_profile_learning_rate", "Stationary Noise Profile Tracking", 0.001, 0.5, 0.04, "rate", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("bark_scale_smoothing_filter", "Cochlear Bark Critical Band Smoothing", 0.1, 4.0, 1.2, "bark", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_unmasking_protection_db", "Percussive Attack Preservation Pad", 0.0, 18.0, 6.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("speech_harmonic_isolation_gain", "Vocal Harmonic Tracking Enhancer", 0.0, 12.0, 3.5, "dB", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_spectral_subtraction_overdrive", "Over-Subtraction Musical Artifact Suppression", true, (34, 197, 94)))
        ),
        "NeuralLatentTimbreInterpolationAutoencoder" => Box::new(
            GenericDspNodeUi::new("NeuralLatentTimbreInterpolationAutoencoder", "Variational Latent Timbre Space Interpolator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("latent_dimension_x_morph", "Latent Manifold Dimension X (Warmth)", -3.0, 3.0, 0.0, "dim", (217, 70, 239)))
                .with_param(DspParamDescriptor::new_linear("latent_dimension_y_morph", "Latent Manifold Dimension Y (Acoustic Density)", -3.0, 3.0, 0.5, "dim", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("variational_prior_temperature", "Sampling Distribution Temperature", 0.1, 2.0, 0.85, "T", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("encoder_reconstruction_fidelity", "Neural Reconstruction Fidelity", 0.1, 1.0, 0.92, "fidelity", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("geodesic_trajectory_smoothing_ms", "Manifold Trajectory Inertia", 5.0, 500.0, 65.0, "ms", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_adversarial_phase_coherence", "Adversarial Phase Coherence Restoration", true, (34, 197, 94)))
        ),
        "NeuralDifferentiableIirEqualizer" => Box::new(
            GenericDspNodeUi::new("NeuralDifferentiableIirEqualizer", "Differentiable Gradient IIR Biquad Equalizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("gradient_learning_rate_alpha", "Adaptive Filter Convergence Speed", 0.001, 0.5, 0.05, "alpha", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("target_perceptual_curve_match_db", "Target Curve Matching Depth", 0.0, 24.0, 12.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("biquad_q_regularization_penalty", "Resonance Q Factor Regularization", 0.1, 10.0, 1.5, "lambda", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_smoothness_constraint", "Spectral Curve Smoothness Constraint", 0.1, 5.0, 1.2, "beta", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("auditory_weighting_iso226", "ISO 226 Equal-Loudness Weighting", 0.0, 1.0, 0.7, "iso", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_bool("enable_minimum_phase_warp", "Guaranteed Minimum-Phase Biquad Stability", true, (34, 197, 94)))
        ),
        "NeuralPerceptualSpectralEnhancer" => Box::new(
            GenericDspNodeUi::new("NeuralPerceptualSpectralEnhancer", "Deep Neural Psychoacoustic Harmonic Restorer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("neural_inference_blend", "Neural Harmonic Reconstruction Depth", 0.0, 1.0, 0.65, "mix", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("air_band_excitation_freq_hz", "Psychoacoustic Air Excitation Corner", 6000.0, 20000.0, 12500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("spectral_envelope_tracking_speed", "Neural Spectral Envelope Tracking", 1.0, 100.0, 24.0, "ms", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("perceptual_masking_threshold_db", "Psychoacoustic Masking Protection", -60.0, -12.0, -32.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("odd_even_generated_harmonic_ratio", "Restored Harmonic Odd/Even Ratio", 0.1, 2.0, 0.85, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_phase_coherent_synthesis", "Phase Coherent Carrier Locking", true, (34, 197, 94)))
        ),
        "NeuralHarmonicExciterAirRestorer" => Box::new(
            GenericDspNodeUi::new("NeuralHarmonicExciterAirRestorer", "Neural Audio Spectral Harmonic Tube Exciter", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("exciter_spectral_drive_amount", "Neural Tube Saturation Drive", 0.0, 1.0, 0.45, "drive", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_shelf_hinge_hz", "Air Band High-Shelf Corner Frequency", 3000.0, 15000.0, 7500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("second_harmonic_sheen_level", "Even-Order Harmonic Sparkle Boost", 0.0, 1.0, 0.58, "sheen", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("third_harmonic_edge_density", "Odd-Order Harmonic Presence Edge", 0.0, 1.0, 0.32, "edge", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("stereo_exciter_crosstalk_spread", "Binaural Spectral Width Spread", 0.0, 1.0, 0.4, "spread", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_dynamic_high_band_sidechain", "Dynamic High-Band Transient Gate", true, (34, 197, 94)))
        ),
        "NeuralSpectralEnvelopeMorpher" => Box::new(
            GenericDspNodeUi::new("NeuralSpectralEnvelopeMorpher", "Deep Latent Neural Spectral Formant Morpher", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("morph_latent_interpolation_ratio", "Latent Timbre Interpolation Position", 0.0, 1.0, 0.5, "morph", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_bin_resolution_bands", "Neural Latent Formant Band Resolution", 16.0, 256.0, 64.0, "bands", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("formant_frequency_shift_semitones", "Formant Envelope Frequency Shift", -24.0, 24.0, 0.0, "semi", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("latent_space_temperature_chaos", "Neural Latent Sampling Temperature", 0.1, 2.0, 0.85, "temp", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("residual_unvoiced_noise_mix", "Unvoiced Consonant Noise Pass-Through", 0.0, 1.0, 0.25, "mix", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_phase_vocoder_lock", "Phase Vocoder Bin Phase Coherence Lock", true, (34, 197, 94)))
        ),
        "NeuralPerceptualTransientEnhancer" => Box::new(
            GenericDspNodeUi::new("NeuralPerceptualTransientEnhancer", "Neural Attention Perceptual Transient Sculptor", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("transient_attack_boost_gain_db", "Transient Attack Punch Boost Gain", -18.0, 18.0, 4.5, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sustain_body_level_gain_db", "Sustain Body Decay Level Gain", -18.0, 18.0, -1.5, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("transient_detection_temporal_sharpness", "Neural Attention Temporal Window Sharpness", 0.5, 5.0, 1.8, "sharp", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("high_frequency_air_restoration_db", "Lost Ultrasonic Air Transient Restoration", 0.0, 12.0, 3.2, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_stereo_width_spread", "Transient Stereo Field Spatial Spread", 0.0, 2.0, 1.25, "width", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_latency_predictive_mode", "Zero-Latency Causal Neural Inference Mode", true, (34, 197, 94)))
        ),
        "NeuralSpectralHarmonicRestorer" => Box::new(
            GenericDspNodeUi::new("NeuralSpectralHarmonicRestorer", "Neural Latent Transformer Spectral Harmonic Restorer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("harmonic_restoration_amount", "Synthesized Harmonic Injection Level", 0.0, 2.0, 1.0, "ratio", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("ultrasonic_air_ceiling_khz", "Ultrasonic Reconstruction Ceiling", 12.0, 48.0, 24.0, "kHz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("latent_prediction_confidence", "Neural Manifold Confidence Threshold", 0.1, 1.0, 0.85, "conf", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_phase_alignment_lock", "Transient Coherent Phase Re-Alignment", 0.0, 1.0, 0.95, "lock", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("even_to_odd_generated_harmonic_bias", "Generated Even vs Odd Harmonic Weight", 0.0, 1.0, 0.45, "bias", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_psychoacoustic_masking_guard", "Psychoacoustic Masking Masking Guard", true, (34, 197, 94)))
        ),
        "NeuralZeroPhaseTransientRestorer" => Box::new(
            GenericDspNodeUi::new("NeuralZeroPhaseTransientRestorer", "Neural Attention Zero-Phase Attack Transient Sculptor", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("transient_attack_reconstruction_gain_db", "Restored Transient Peak Boost Gain", 0.0, 18.0, 6.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("wavefront_dispersion_correction", "Attack Wavefront Dispersion Unsmear", 0.0, 1.0, 0.65, "corr", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("temporal_attention_resolution_ms", "Neural Attention Temporal Window Span", 0.2, 10.0, 1.5, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sustain_tail_suppression_ratio", "Sustain Residual Masking Suppression", 0.0, 1.0, 0.35, "ratio", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("stereo_spatial_transient_focuser", "Transient Stereo Imaging Center Focuser", 0.0, 2.0, 1.1, "focus", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_anti_aliased_impulse_generation", "Sub-Sample Bandlimited Impulse Interpolation", true, (34, 197, 94)))
        ),
        "NeuralSubHarmonicFundamentalSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralSubHarmonicFundamentalSynthesizer", "Neural Attention Deep Sub-Harmonic Fundamental Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("sub_octave_injection_level_db", "Neural Reconstructed Sub-Bass Injection Level", -36.0, 12.0, 0.0, "dB", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_harmonic_divider_order", "Subharmonic Frequency Division Ratio", 1.0, 3.0, 1.0, "oct", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("neural_pitch_tracking_confidence", "Neural Pitch Manifold Confidence Threshold", 0.2, 1.0, 0.88, "conf", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_transient_phase_lock_speed_ms", "Sub-Bass Wavefront Phase Lock Response Speed", 0.5, 20.0, 2.5, "ms", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("psychoacoustic_low_shelf_hinge_hz", "Sub Synthesis Upper Boundary Frequency Hinge", 40.0, 160.0, 90.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_even_order_sub_saturation", "Quadratic Asymmetric Tube Saturation on Sub Bass", true, (34, 197, 94)))
        ),
        "NeuralStereoFieldDeCorrelator" => Box::new(
            GenericDspNodeUi::new("NeuralStereoFieldDeCorrelator", "Neural Latent Attention Stereo Field Decorrelator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("spatial_decorrelation_width_factor", "Neural Stereo Field Decorrelation Width Factor", 0.0, 2.5, 1.25, "width", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("mono_downmix_null_protection", "Mono Summing Phase Cancellation Safety Guard", 0.5, 1.0, 0.98, "prot", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_clustering_band_count", "Latent Attention Spectral Band Partitioning", 4.0, 64.0, 24.0, "bands", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_decorrelation_bypass_ratio", "Transient Center Anchor Preservation Ratio", 0.0, 1.0, 0.72, "ratio", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("diffuse_ambience_extraction_db", "Synthesized Diffuse Ambient Side Gain", -18.0, 12.0, 1.5, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_binaural_pinna_cues", "HRTF Pinna Spectral Notch Imprinting", true, (34, 197, 94)))
        ),
        "NeuralDiffusionLatentTransientSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralDiffusionLatentTransientSynthesizer", "Neural Latent Diffusion Micro-Transient Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("diffusion_denoising_steps", "Reverse Latent Diffusion Denoising Step Count", 2.0, 16.0, 4.0, "steps", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("transient_density_temperature", "Latent Stochastic Sampling Temperature", 0.1, 1.5, 0.75, "temp", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("attack_sharpness_latent_bias", "Micro-Transient Attack Crest Factor Bias", 0.0, 2.0, 1.25, "bias", (249, 115, 22)))
                .with_param(DspParamDescriptor::new_linear("conditioning_spectral_centroid_hz", "Spectral Conditioning Brightness Centroid", 500.0, 12000.0, 4500.0, "Hz", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("latent_temporal_coherence", "Cross-Frame Latent Manifold Coherence", 0.0, 1.0, 0.82, "coh", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_realtime_speculative_decoding", "Real-Time Speculative Neural Decoding Cache", true, (34, 197, 94)))
        ),
        "NeuralWaveletAcousticDenoisingDenoiser" => Box::new(
            GenericDspNodeUi::new("NeuralWaveletAcousticDenoisingDenoiser", "Neural Wavelet Multi-Scale Acoustic Denoiser", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("denoising_suppression_depth_db", "Background Noise Suppression Attenuation Depth", 0.0, 48.0, 18.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_preservation_threshold", "Musical Harmonic Component Protection Margin", 0.1, 1.0, 0.88, "prot", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("wavelet_octave_decomposition_bands", "Continuous Wavelet Scale Band Count", 4.0, 16.0, 8.0, "scales", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_pass_through_sensitivity", "Transient Wavefront Pass-Through Sensitivity", 0.0, 1.0, 0.75, "sens", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_floor_whitening_db", "Subtle Comfort Noise Floor Injection", -80.0, -36.0, -60.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_adaptive_room_noise_learning", "Dynamic Adaptive Noise Profile Learning", true, (34, 197, 94)))
        ),
        "NeuralContinuousNormalizingFlowTimbreMorpher" => Box::new(
            GenericDspNodeUi::new("NeuralContinuousNormalizingFlowTimbreMorpher", "Neural Continuous Normalizing Flow Timbre Morpher", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("ode_integration_steps", "Continuous ODE Integration Step Count", 2.0, 24.0, 6.0, "steps", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("latent_flow_trajectory", "Latent Normalizing Flow Morph Trajectory", 0.0, 1.0, 0.5, "pos", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("vector_field_curl_damping", "Neural Vector Field Curl / Divergence Damping", 0.0, 1.0, 0.7, "damp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("spectral_prior_temperature", "Base Gaussian Latent Prior Temperature", 0.1, 2.0, 0.9, "temp", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("phase_field_coherence_weight", "Spectral Phase Field Coherence Weight", 0.0, 1.0, 0.82, "coh", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_bool("enable_adaptive_runge_kutta", "Adaptive Runge-Kutta 4th-Order Solver", true, (34, 197, 94)))
        ),
        "NeuralDifferentiableWavetablePhaseVocoder" => Box::new(
            GenericDspNodeUi::new("NeuralDifferentiableWavetablePhaseVocoder", "Neural Differentiable Wavetable Phase Vocoder", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("analysis_window_size_ms", "Differentiable Analysis Frame Window", 10.0, 120.0, 35.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("pitch_synchronous_overlap", "Pitch-Synchronous Window Overlap Ratio", 0.5, 0.95, 0.75, "ratio", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("neural_phase_regularization", "Loss-Regularized Phase Continuity Weight", 0.0, 1.0, 0.88, "reg", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("latent_harmonic_smoothing", "Latent Harmonic Envelope Regularization", 0.0, 1.0, 0.45, "smooth", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("transient_phase_lock_threshold", "Transient Wavefront Phase-Lock Sensitivity", 0.0, 1.0, 0.65, "sens", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_gradient_timbre_interpolation", "Continuous Latent Gradient Interpolation", true, (34, 197, 94)))
        ),
        "NeuralDifferentialWavetableLatentOscillator" => Box::new(
            GenericDspNodeUi::new("NeuralDifferentialWavetableLatentOscillator", "Neural Differentiable Latent Wavetable Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("latent_dimension_z1_morph", "Primary Neural Latent Vector (Z1 Morph)", -3.0, 3.0, 0.0, "z1", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("latent_dimension_z2_complexity", "Secondary Neural Latent Vector (Z2 Complexity)", -3.0, 3.0, 0.5, "z2", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("differentiable_learning_rate", "Real-Time Gradient Adaptation Rate", 0.001, 0.1, 0.015, "lr", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("neural_spectral_loss_penalty", "Multi-Scale Spectral Loss Regularizer", 0.0, 2.0, 0.65, "loss", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("wavetable_anti_aliasing_oversample", "Internal Neural Oversampling Factor", 1.0, 8.0, 4.0, "x", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_bool("enable_continuous_latent_interpolation", "Continuous Cubic Spline Latent Interpolation", true, (34, 197, 94)))
        ),
        "NeuralAdaptiveSpectralDeReverberator" => Box::new(
            GenericDspNodeUi::new("NeuralAdaptiveSpectralDeReverberator", "Neural Adaptive Spectral De-Reverberator", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("dereverb_suppression_intensity_db", "Late Reverberation Suppression Depth", 0.0, 36.0, 18.0, "dB", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("room_acoustic_coherence_threshold", "Direct-Path Acoustic Coherence Threshold", 0.1, 0.95, 0.62, "coh", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation_weight", "Attack Wavefront Transient Preservation", 0.0, 1.0, 0.88, "trans", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("spectral_subband_tracking_speed_ms", "Recursive Power Spectrum Tracking Speed", 5.0, 100.0, 24.0, "ms", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("diffuse_soundfield_coloration_trim_db", "Post-Enhancement Air & Tone Trim", -12.0, 6.0, 0.0, "dB", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_bool("enable_zero_phase_residual_synthesis", "Zero-Phase Room Breath Residual Synthesis", true, (34, 197, 94)))
        ),
        "NeuralRecurrentReservoirComputingSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralRecurrentReservoirComputingSynthesizer", "Recurrent Reservoir Computing Timbre Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("reservoir_size", "Reservoir Neuron Node Count", 16.0, 512.0, 128.0, "nodes", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("spectral_radius", "Matrix Spectral Radius", 0.1, 1.5, 0.95, "rad", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("leak_rate", "Neuron State Leaky Rate", 0.01, 1.0, 0.35, "rate", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("input_coupling", "Input Modulation Drive", 0.0, 1.0, 0.60, "drive", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("nonlinear_activation", "Tanh Nonlinear Activation Gain", 0.0, 1.0, 0.80, "sat", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("timbre_drift", "Autonomous Timbre Evolution Rate", 0.0, 1.0, 0.25, "drift", (34, 197, 94)))
        ),
        "NeuralSubharmonicBassRestorationSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralSubharmonicBassRestorationSynthesizer", "Neural Subharmonic Bass Restoration Engine", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("sub_mix", "Synthesized Subharmonic Mix", 0.0, 1.0, 0.50, "wet", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("sub_octave_depth", "Sub-Octave Generation Intensity", 0.0, 1.0, 0.80, "depth", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("harmonic_tracker", "Tracking Frequency Ceiling", 20.0, 250.0, 80.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("transient_preservation", "Transient Snap Preservation", 0.0, 1.0, 0.75, "snap", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("sub_saturation", "Sub-Bass Tape Saturation", 0.0, 1.0, 0.20, "sat", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("low_shelf_cutoff", "Sub Low-Shelf Anchor Cutoff", 20.0, 120.0, 50.0, "Hz", (34, 197, 94)))
        ),
        "NeuralDiffusionLatentHarmonicExciter" => Box::new(
            GenericDspNodeUi::new("NeuralDiffusionLatentHarmonicExciter", "Neural Diffusion Latent Harmonic Exciter", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("diffusion_steps", "Denoising Diffusion Inference Steps", 1.0, 20.0, 4.0, "steps", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("harmonic_air_generation", "High-Frequency Air Synthesis Depth", 0.0, 1.0, 0.65, "air", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("guidance_scale", "Latent Classifier-Free Guidance", 0.5, 5.0, 1.8, "scale", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("sub_band_split_freq", "Excitation Crossover Frequency", 2000.0, 14000.0, 6500.0, "Hz", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("latent_noise_seed", "Stochastic Micro-Texture Seed", 0.0, 1.0, 0.42, "seed", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("transient_spectral_focus", "Transient Dynamic Focus Weight", 0.0, 1.0, 0.78, "focus", (239, 68, 68)))
        ),
        "NeuralPhaseUnwrappingStereoSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralPhaseUnwrappingStereoSynthesizer", "Neural Phase Unwrapping Stereo Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("phase_unwrapping_depth", "Complex Phase Unwrapping Depth", 0.0, 1.0, 0.80, "depth", (236, 72, 153)))
                .with_param(DspParamDescriptor::new_linear("complex_coherence", "Binaural Spectral Coherence", 0.0, 1.0, 0.75, "coh", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("stereo_spread_angle", "Synthesized Stereo Field Width", 0.0, 2.0, 1.25, "width", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("phase_jitter_reduction", "Temporal Phase Jitter Smoothing", 0.0, 1.0, 0.45, "smooth", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("low_frequency_mono_guard", "Bass Mono Safeguard Crossover", 40.0, 300.0, 120.0, "Hz", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("neural_confidence_threshold", "Model Inference Confidence Gate", 0.1, 0.99, 0.70, "gate", (239, 68, 68)))
        ),
        "NeuralAutoregressivePredictiveSampleSynthesizer" => Box::new(
            GenericDspNodeUi::new("NeuralAutoregressivePredictiveSampleSynthesizer", "Neural Autoregressive Predictive Sample Synthesizer", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("conditioning_latent_dim", "Latent Style Embedding Vector", 0.0, 1.0, 0.50, "vec", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("temperature_sampling", "Autoregressive Sampling Temperature", 0.1, 2.0, 0.85, "temp", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("causal_dilated_receptive_field", "Dilated Convolution Stack Span", 64.0, 2048.0, 512.0, "spl", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("quantization_companding", "Mu-Law Nonlinear Amplitude Compander", 0.0, 1.0, 0.70, "comp", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("top_p_nucleus_filter", "Nucleus Probability Mass Threshold (Top-P)", 0.5, 1.0, 0.92, "prob", (99, 102, 241)))
                .with_param(DspParamDescriptor::new_linear("context_history_blend", "Prior Audio History Recurrence Blend", 0.0, 1.0, 0.60, "ratio", (34, 197, 94)))
        ),
        "NeuralSpectralCrossSynthesisVocoder" => Box::new(
            GenericDspNodeUi::new("NeuralSpectralCrossSynthesisVocoder", "Neural Spectral Cross-Synthesis Vocoder", DspNodeCategory::NeuralAi)
                .with_param(DspParamDescriptor::new_linear("spectral_envelope_transfer", "Neural Formant Transfer Depth", 0.0, 1.0, 0.80, "depth", (56, 189, 248)))
                .with_param(DspParamDescriptor::new_linear("latent_formant_shift", "Vocal Tract Latent Formant Shift", -24.0, 24.0, 0.0, "semi", (168, 85, 247)))
                .with_param(DspParamDescriptor::new_linear("phase_coherence_recovery", "Neural Phase Coherence Preservation", 0.0, 1.0, 0.90, "coher", (34, 197, 94)))
                .with_param(DspParamDescriptor::new_linear("cross_synthesis_dry_wet", "Cross-Synthesis Modulation Mix", 0.0, 1.0, 1.0, "mix", (239, 68, 68)))
                .with_param(DspParamDescriptor::new_linear("unvoiced_consonant_injection", "Unvoiced Consonant Sibilance Blend", 0.0, 1.0, 0.35, "sib", (245, 158, 11)))
                .with_param(DspParamDescriptor::new_linear("pitch_tracking_confidence", "Autocorrelation Pitch Tracker Gate", 0.1, 1.0, 0.75, "conf", (99, 102, 241)))
        ),
        _ => return None,
    };
    Some(node)
}
